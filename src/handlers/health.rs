use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde_json::json;
use std::sync::Arc;

use crate::AppState;

pub async fn health_check(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let response = json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "version": state.version,
    });

    (StatusCode::OK, Json(response))
}

pub async fn server_status(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let response = json!({
        "service": state.name,
        "status": "online",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "version": state.version,
        "endpoints": {
            "health": "/health",
            "status": "/status",
            "transcribe_batch": "POST /api/v1/transcriptions",
            "transcribe_stream": "WebSocket /api/v1/transcribe/stream",
        }
    });

    (StatusCode::OK, Json(response))
}
