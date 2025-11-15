// API key authentication middleware
use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use tracing::warn;

pub async fn check_api_key(
    request: Request,
    next: Next,
) -> Result<Response, ApiKeyError> {
    let api_key = request
        .headers()
        .get("x-api-key")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    let path = request.uri().path();
    
    // Check if path is public (no auth required)
    if path == "/health" || path == "/status" {
        return Ok(next.run(request).await);
    }

    match api_key {
        Some(key) => {
            // Validate the key
            let stored_key = std::env::var("API_KEY")
                .unwrap_or_else(|_| "dev_key_12345_change_in_production".to_string());

            if key == stored_key {
                Ok(next.run(request).await)
            } else {
                warn!("Invalid API key attempt on {}", path);
                Err(ApiKeyError::InvalidKey)
            }
        }
        None => {
            warn!("Missing API key on {}", path);
            Err(ApiKeyError::MissingKey)
        }
    }
}

pub enum ApiKeyError {
    MissingKey,
    InvalidKey,
}

impl IntoResponse for ApiKeyError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            ApiKeyError::MissingKey => (
                StatusCode::UNAUTHORIZED,
                "Missing x-api-key header",
            ),
            ApiKeyError::InvalidKey => (
                StatusCode::FORBIDDEN,
                "Invalid API key",
            ),
        };

        let body = Json(json!({
            "error": error_message,
            "code": status.as_u16(),
            "timestamp": chrono::Utc::now().to_rfc3339(),
        }));

        (status, body).into_response()
    }
}
