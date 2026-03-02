use http::StatusCode;
use serde::Deserialize;
use vercel_runtime::{run, service_fn, Error, Request, Response, ResponseBody};

#[derive(Deserialize)]
struct PostsQuery {
    topic_id: i32,
    limit: Option<i32>,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    run(service_fn(handler)).await
}

async fn handler(req: Request) -> Result<Response<ResponseBody>, Error> {
    // Load environment variables
    let _ = dotenvy::dotenv();

    // Parse query parameters
    let query_str = req.uri().query().unwrap_or("");
    let query: PostsQuery = match serde_urlencoded::from_str(query_str) {
        Ok(q) => q,
        Err(e) => {
            return Ok(Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .header("Content-Type", "application/json")
                .body(ResponseBody::from(format!(
                    r#"{{"error": "Invalid query parameters: {}"}}"#,
                    e
                )))?);
        }
    };

    let limit = query.limit.unwrap_or(10);

    // Create database pool
    let pool = match sentiment_api::create_pool().await {
        Ok(p) => p,
        Err(e) => {
            return Ok(Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .header("Content-Type", "application/json")
                .body(ResponseBody::from(format!(
                    r#"{{"error": "Database connection failed: {}"}}"#,
                    e
                )))?);
        }
    };

    // Get top posts sorted by weight * relevancy
    match sentiment_api::get_top_posts_by_topic(&pool, query.topic_id, limit).await {
        Ok(posts) => {
            let json = serde_json::to_string(&posts).unwrap();
            Ok(Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", "application/json")
                .body(ResponseBody::from(json))?)
        }
        Err(e) => Ok(Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .header("Content-Type", "application/json")
            .body(ResponseBody::from(format!(
                r#"{{"error": "Failed to fetch posts: {}"}}"#,
                e
            )))?),
    }
}
