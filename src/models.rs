use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TranscriptionRequest {
    pub language: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TranscriptionSegment {
    pub id: usize,
    pub start: f32,
    pub end: f32,
    pub text: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TranscriptionResponse {
    pub id: String,
    pub text: String,
    pub segments: Vec<TranscriptionSegment>,
    pub language: String,
    pub duration: f32,
    pub timestamp: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
    pub code: u16,
    pub timestamp: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StreamingMessage {
    pub r#type: String, // "partial", "final", "error"
    pub result: Option<String>,
    pub error: Option<String>,
    pub timestamp: String,
}

impl TranscriptionResponse {
    pub fn new(text: String, language: String, duration: f32) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            text,
            segments: vec![],
            language,
            duration,
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }
}

impl ErrorResponse {
    pub fn new(error: String, code: u16) -> Self {
        Self {
            error,
            code,
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }
}

impl StreamingMessage {
    pub fn partial(result: String) -> Self {
        Self {
            r#type: "partial".to_string(),
            result: Some(result),
            error: None,
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }

    pub fn final_result(result: String) -> Self {
        Self {
            r#type: "final".to_string(),
            result: Some(result),
            error: None,
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }

    pub fn error(error: String) -> Self {
        Self {
            r#type: "error".to_string(),
            result: None,
            error: Some(error),
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transcription_response_creation() {
        let response = TranscriptionResponse::new(
            "Hello world".to_string(),
            "en".to_string(),
            1.5,
        );

        assert_eq!(response.text, "Hello world");
        assert_eq!(response.language, "en");
        assert_eq!(response.duration, 1.5);
        assert!(!response.id.is_empty());
        assert!(response.segments.is_empty());
    }

    #[test]
    fn test_error_response_creation() {
        let error = ErrorResponse::new(
            "Invalid audio format".to_string(),
            400,
        );

        assert_eq!(error.error, "Invalid audio format");
        assert_eq!(error.code, 400);
        assert!(!error.timestamp.is_empty());
    }

    #[test]
    fn test_streaming_message_partial() {
        let msg = StreamingMessage::partial("hello".to_string());
        assert_eq!(msg.r#type, "partial");
        assert_eq!(msg.result, Some("hello".to_string()));
        assert!(msg.error.is_none());
    }

    #[test]
    fn test_streaming_message_final() {
        let msg = StreamingMessage::final_result("complete".to_string());
        assert_eq!(msg.r#type, "final");
        assert_eq!(msg.result, Some("complete".to_string()));
        assert!(msg.error.is_none());
    }

    #[test]
    fn test_streaming_message_error() {
        let msg = StreamingMessage::error("processing failed".to_string());
        assert_eq!(msg.r#type, "error");
        assert!(msg.result.is_none());
        assert_eq!(msg.error, Some("processing failed".to_string()));
    }

    #[test]
    fn test_transcription_response_serialization() {
        let response = TranscriptionResponse::new(
            "test".to_string(),
            "en".to_string(),
            2.0,
        );

        let json = serde_json::to_string(&response).expect("Failed to serialize");
        assert!(json.contains("\"text\":"));
        assert!(json.contains("\"language\":"));
        assert!(json.contains("\"id\":"));
    }
}
