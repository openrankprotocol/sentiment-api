use http::StatusCode;
use serde::Deserialize;
use vercel_runtime::{run, service_fn, Error, Request, Response, ResponseBody};

#[derive(Debug, Deserialize)]
struct SentimentQuery {
    topic_id: i32,
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
    let query: SentimentQuery = match serde_urlencoded::from_str(query_str) {
        Ok(q) => q,
        Err(_) => {
            return Ok(Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .header("Content-Type", "application/json")
                .body(ResponseBody::from(
                    r#"{"error": "Missing or invalid topic_id parameter"}"#,
                ))?);
        }
    };

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

    // Get sentiment for topic
    match sentiment_api::get_sentiment_for_topic(&pool, query.topic_id).await {
        Ok(sentiment) => {
            let json = serde_json::to_string(&sentiment)?;
            Ok(Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", "application/json")
                .body(ResponseBody::from(json))?)
        }
        Err(e) => Ok(Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .header("Content-Type", "application/json")
            .body(ResponseBody::from(format!(
                r#"{{"error": "Failed to get sentiment: {}"}}"#,
                e
            )))?),
    }
}
