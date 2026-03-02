use http::StatusCode;
use serde::Deserialize;
use vercel_runtime::{run, service_fn, Error, Request, Response, ResponseBody};

#[derive(Deserialize)]
struct TopicsQuery {
    category: String,
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
    let query: TopicsQuery = match serde_urlencoded::from_str(query_str) {
        Ok(q) => q,
        Err(_) => {
            return Ok(Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .header("Content-Type", "application/json")
                .body(ResponseBody::from(
                    r#"{"error": "Missing required query parameter: category"}"#,
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

    // Fetch topics by category
    match sentiment_api::get_topics_by_category(&pool, &query.category).await {
        Ok(topics) => {
            let json = serde_json::to_string(&topics).unwrap();
            Ok(Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", "application/json")
                .body(ResponseBody::from(json))?)
        }
        Err(e) => Ok(Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .header("Content-Type", "application/json")
            .body(ResponseBody::from(format!(
                r#"{{"error": "Failed to fetch topics: {}"}}"#,
                e
            )))?),
    }
}
