use axum::{
    extract::{Multipart, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
};
use std::sync::Arc;
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::{models::ErrorResponse, AppState};

/// POST /voice-chat
/// Handles voice chat: audio input -> transcription -> LLM -> TTS -> audio output
/// Uses ephemeral in-memory sessions (no database storage)
pub async fn voice_chat(
    State(state): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> Result<Response, VoiceChatError> {
    info!("Received voice chat request");

    let mut audio_data: Option<Vec<u8>> = None;
    let mut voice_session_id: Option<Uuid> = None;

    // Parse multipart form data
    while let Some(field) = multipart.next_field().await? {
        let name = field.name().unwrap_or("").to_string();

        match name.as_str() {
            "audio" => {
                let data = field.bytes().await?;
                info!("Received audio file: {} bytes", data.len());
                audio_data = Some(data.to_vec());
            }
            "voice_session_id" => {
                let text = field.text().await?;
                match Uuid::parse_str(&text) {
                    Ok(id) => {
                        info!("Voice session ID: {}", id);
                        voice_session_id = Some(id);
                    }
                    Err(e) => {
                        warn!("Invalid voice_session_id format: {}", e);
                        return Err(VoiceChatError::InvalidSessionId);
                    }
                }
            }
            _ => {
                warn!("Unknown field: {}", name);
            }
        }
    }

    // Validate required fields
    let audio = audio_data.ok_or(VoiceChatError::MissingAudio)?;
    let session_id = voice_session_id.ok_or(VoiceChatError::MissingSessionId)?;

    // Step 1: Transcribe audio to text
    info!("Transcribing audio ({} bytes)", audio.len());
    let transcription = state
        .vosk_service
        .transcribe(audio)
        .await
        .map_err(|e| {
            error!("Transcription failed: {}", e);
            VoiceChatError::TranscriptionFailed
        })?;

    info!("Transcription: '{}'", transcription);

    if transcription.trim().is_empty() {
        warn!("Empty transcription received");
        return Err(VoiceChatError::EmptyTranscription);
    }

    // Step 2: Get conversation history from in-memory session
    let history = state.voice_sessions.get_history(session_id).await;
    info!("Retrieved {} messages from voice session history", history.len());

    // Step 3: Generate LLM response
    info!("Generating LLM response");
    let llm_response = state
        .llm_service
        .generate_voice_response(&history, &transcription)
        .await
        .map_err(|e| {
            error!("LLM generation failed: {}", e);
            VoiceChatError::LlmFailed
        })?;

    info!("LLM response: '{}'", llm_response);

    // Step 4: Save to in-memory session (ephemeral, no database)
    state.voice_sessions.add_message(session_id, "user", &transcription).await;
    state.voice_sessions.add_message(session_id, "assistant", &llm_response).await;
    info!("Saved messages to ephemeral voice session");

    // Step 5: Convert LLM response to speech using ElevenLabs
    info!("Converting text to speech");
    let audio_response = state
        .elevenlabs_service
        .text_to_speech(&llm_response)
        .await
        .map_err(|e| {
            error!("TTS generation failed: {}", e);
            VoiceChatError::TtsFailed
        })?;

    info!("Generated {} bytes of MP3 audio", audio_response.len());

    // Step 6: Return MP3 audio
    Ok((
        StatusCode::OK,
        [(header::CONTENT_TYPE, "audio/mpeg")],
        audio_response,
    )
        .into_response())
}

#[derive(Debug)]
pub enum VoiceChatError {
    MissingAudio,
    MissingSessionId,
    InvalidSessionId,
    TranscriptionFailed,
    EmptyTranscription,
    LlmFailed,
    TtsFailed,
    MultipartError(axum::extract::multipart::MultipartError),
}

impl From<axum::extract::multipart::MultipartError> for VoiceChatError {
    fn from(err: axum::extract::multipart::MultipartError) -> Self {
        VoiceChatError::MultipartError(err)
    }
}

impl IntoResponse for VoiceChatError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            VoiceChatError::MissingAudio => (StatusCode::BAD_REQUEST, "Missing audio file"),
            VoiceChatError::MissingSessionId => {
                (StatusCode::BAD_REQUEST, "Missing voice_session_id")
            }
            VoiceChatError::InvalidSessionId => {
                (StatusCode::BAD_REQUEST, "Invalid voice_session_id format")
            }
            VoiceChatError::TranscriptionFailed => {
                (StatusCode::UNPROCESSABLE_ENTITY, "Failed to transcribe audio")
            }
            VoiceChatError::EmptyTranscription => {
                (StatusCode::UNPROCESSABLE_ENTITY, "No speech detected in audio")
            }
            VoiceChatError::LlmFailed => {
                (StatusCode::INTERNAL_SERVER_ERROR, "LLM generation failed")
            }
            VoiceChatError::TtsFailed => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Text-to-speech failed")
            }
            VoiceChatError::MultipartError(_) => {
                (StatusCode::BAD_REQUEST, "Invalid multipart form data")
            }
        };

        (
            status,
            axum::Json(ErrorResponse::new(message.to_string(), status.as_u16())),
        )
            .into_response()
    }
}
