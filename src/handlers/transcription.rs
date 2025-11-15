use axum::{
    extract::{ws::WebSocketUpgrade, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use futures::{SinkExt, StreamExt};
use std::sync::Arc;
use tracing::{error, info};

use crate::{
    models::{ErrorResponse, StreamingMessage},
    AppState,
};

pub async fn transcribe_batch(
    State(state): State<Arc<AppState>>,
    body: axum::body::Bytes,
) -> impl IntoResponse {
    if body.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new(
                "No audio data provided".to_string(),
                400,
            )),
        )
            .into_response();
    }

    match state.vosk_service.transcribe(body.to_vec()).await {
        Ok(text) => {
            info!("Transcription completed: {} chars", text.len());
            (StatusCode::OK, Json(serde_json::json!({ "text": text }))).into_response()
        }
        Err(e) => {
            error!("Transcription error: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new(format!("Transcription failed: {}", e), 500)),
            )
                .into_response()
        }
    }
}

pub async fn transcribe_stream(
    State(state): State<Arc<AppState>>,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_streaming(socket, state))
}

async fn handle_streaming(socket: axum::extract::ws::WebSocket, state: Arc<AppState>) {
    let (mut sender, mut receiver) = socket.split();
    let mut audio_chunks = Vec::new();

    while let Some(msg) = receiver.next().await {
        match msg {
            Ok(axum::extract::ws::Message::Binary(data)) => {
                audio_chunks.push(data.to_vec());
                info!("Received audio chunk: {} bytes", data.len());
            }
            Ok(axum::extract::ws::Message::Text(text)) => {
                if text == "FINISH" {
                    info!("Stream finish signal received");
                    break;
                }
            }
            Ok(axum::extract::ws::Message::Close(_)) => {
                info!("WebSocket closed by client");
                break;
            }
            Err(e) => {
                error!("WebSocket error: {}", e);
                let _ = sender
                    .send(axum::extract::ws::Message::Text(
                        serde_json::to_string(&StreamingMessage::error(format!(
                            "WebSocket error: {}",
                            e
                        )))
                        .unwrap(),
                    ))
                    .await;
                return;
            }
            _ => {}
        }
    }

    if audio_chunks.is_empty() {
        let _ = sender
            .send(axum::extract::ws::Message::Text(
                serde_json::to_string(&StreamingMessage::error(
                    "No audio data received".to_string(),
                ))
                .unwrap(),
            ))
            .await;
        return;
    }

    match state.vosk_service.transcribe_streaming(audio_chunks).await {
        Ok(text) => {
            info!("Streaming transcription completed: {}", text);
            let message = StreamingMessage::final_result(text);
            let _ = sender
                .send(axum::extract::ws::Message::Text(
                    serde_json::to_string(&message).unwrap(),
                ))
                .await;
        }
        Err(e) => {
            error!("Streaming transcription error: {}", e);
            let message = StreamingMessage::error(format!("Transcription failed: {}", e));
            let _ = sender
                .send(axum::extract::ws::Message::Text(
                    serde_json::to_string(&message).unwrap(),
                ))
                .await;
        }
    }
}
