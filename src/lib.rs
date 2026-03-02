use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgPoolOptions, PgPool, FromRow};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Topic {
    pub id: i32,
    pub name: String,
    pub description: String,
    pub outcomes: Vec<String>,
    pub query: String,
    pub hashtags: Option<Vec<String>>,
    pub category: String,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Post {
    pub id: i32,
    pub topic_id: i32,
    pub run_id: i32,
    pub post_id: String,
    pub text: String,
    pub posted_at: DateTime<Utc>,
    pub inserted_at: Option<DateTime<Utc>>,
    pub like_count: i32,
    pub retweet_count: i32,
    pub reply_count: i32,
    pub quote_count: i32,
    pub author_id: String,
    pub author_username: String,
    pub author_name: String,
    pub author_followers: i32,
    pub author_is_verified: bool,
    pub relevancy_score: Option<f32>,
    pub weight: Option<f32>,
    pub probabilities: Option<Vec<f32>>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct PostWithScore {
    pub id: i32,
    pub topic_id: i32,
    pub run_id: i32,
    pub post_id: String,
    pub text: String,
    pub posted_at: DateTime<Utc>,
    pub inserted_at: Option<DateTime<Utc>>,
    pub like_count: i32,
    pub retweet_count: i32,
    pub reply_count: i32,
    pub quote_count: i32,
    pub author_id: String,
    pub author_username: String,
    pub author_name: String,
    pub author_followers: i32,
    pub author_is_verified: bool,
    pub relevancy_score: Option<f32>,
    pub weight: Option<f32>,
    pub probabilities: Option<Vec<f32>>,
    pub score: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OutcomeSentiment {
    pub outcome_name: String,
    pub probability: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SentimentResponse {
    pub topic_id: i32,
    pub topic_name: String,
    pub sentiments: Vec<OutcomeSentiment>,
}

pub async fn create_pool() -> Result<PgPool, sqlx::Error> {
    // Try DATABASE_URL first, otherwise build from individual components
    let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        let server = std::env::var("SQL_SERVER").expect("SQL_SERVER must be set");
        let port = std::env::var("SQL_PORT").unwrap_or_else(|_| "5432".to_string());
        let db = std::env::var("SQL_DB").expect("SQL_DB must be set");
        let username = std::env::var("SQL_USERNAME").expect("SQL_USERNAME must be set");
        let password = std::env::var("SQL_PASSWORD").expect("SQL_PASSWORD must be set");

        format!(
            "postgres://{}:{}@{}:{}/{}",
            username, password, server, port, db
        )
    });

    PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
}

pub async fn get_topics_by_category(pool: &PgPool, category: &str) -> Result<Vec<Topic>, sqlx::Error> {
    sqlx::query_as::<_, Topic>(
        "SELECT * FROM topics WHERE category = $1"
    )
    .bind(category)
    .fetch_all(pool)
    .await
}

pub async fn get_top_posts_by_topic(
    pool: &PgPool,
    topic_id: i32,
    limit: i32,
) -> Result<Vec<PostWithScore>, sqlx::Error> {
    sqlx::query_as::<_, PostWithScore>(
        r#"
        SELECT
            *,
            (COALESCE(weight, 0) * COALESCE(relevancy_score, 0))::float8 as score
        FROM posts
        WHERE topic_id = $1
        ORDER BY (COALESCE(weight, 0) * COALESCE(relevancy_score, 0)) DESC
        LIMIT $2
        "#
    )
    .bind(topic_id)
    .bind(limit)
    .fetch_all(pool)
    .await
}

pub async fn get_sentiment_for_topic(
    pool: &PgPool,
    topic_id: i32,
) -> Result<SentimentResponse, sqlx::Error> {
    // Get topic info including outcomes
    let topic = sqlx::query_as::<_, Topic>(
        "SELECT * FROM topics WHERE id = $1"
    )
    .bind(topic_id)
    .fetch_one(pool)
    .await?;

    // Get all posts for this topic that have probabilities
    let posts = sqlx::query_as::<_, Post>(
        "SELECT * FROM posts WHERE topic_id = $1 AND probabilities IS NOT NULL"
    )
    .bind(topic_id)
    .fetch_all(pool)
    .await?;

    let num_outcomes = topic.outcomes.len();

    // Calculate weighted scores for each outcome
    // For each probability: weight * relevancy_score * probability
    let mut outcome_scores: HashMap<usize, f64> = HashMap::new();

    for post in &posts {
        let weight = post.weight.unwrap_or(0.0) as f64;
        let relevancy = post.relevancy_score.unwrap_or(0.0) as f64;

        if let Some(ref probs) = post.probabilities {
            for (idx, &prob) in probs.iter().enumerate() {
                if idx < num_outcomes {
                    let score = weight * relevancy * (prob as f64);
                    *outcome_scores.entry(idx).or_insert(0.0) += score;
                }
            }
        }
    }

    // Calculate total score for normalization
    let total_score: f64 = outcome_scores.values().sum();

    // Build sentiment response with normalized probabilities
    let sentiments: Vec<OutcomeSentiment> = topic.outcomes
        .iter()
        .enumerate()
        .map(|(idx, outcome_name)| {
            let raw_score = outcome_scores.get(&idx).copied().unwrap_or(0.0);
            let normalized_probability = if total_score > 0.0 {
                raw_score / total_score
            } else {
                0.0
            };

            OutcomeSentiment {
                outcome_name: outcome_name.clone(),
                probability: normalized_probability,
            }
        })
        .collect();

    Ok(SentimentResponse {
        topic_id: topic.id,
        topic_name: topic.name,
        sentiments,
    })
}
