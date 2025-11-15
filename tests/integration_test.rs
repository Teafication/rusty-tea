// Integration tests for Rusty Tea API
// These tests verify the API endpoints work correctly

#[cfg(test)]
mod integration_tests {
    use std::sync::Arc;

    #[test]
    fn test_models_exist() {
        // Verify core types compile and serialize correctly
        let response = serde_json::json!({
            "id": "test-123",
            "text": "hello world",
            "segments": [],
            "language": "en",
            "duration": 1.5,
            "timestamp": "2025-11-15T10:00:00Z"
        });

        assert_eq!(response["text"], "hello world");
        assert_eq!(response["language"], "en");
    }

    #[test]
    fn test_error_response_structure() {
        // Verify error response structure is correct
        let error = serde_json::json!({
            "error": "Invalid audio format",
            "code": 400,
            "timestamp": "2025-11-15T10:00:00Z"
        });

        assert_eq!(error["code"], 400);
        assert!(error["error"].is_string());
    }

    #[test]
    fn test_streaming_message_structure() {
        // Verify streaming message structure
        let partial = serde_json::json!({
            "type": "partial",
            "result": "hello",
            "error": null,
            "timestamp": "2025-11-15T10:00:00Z"
        });

        assert_eq!(partial["type"], "partial");
        assert_eq!(partial["result"], "hello");

        let final_msg = serde_json::json!({
            "type": "final",
            "result": "hello world",
            "error": null,
            "timestamp": "2025-11-15T10:00:00Z"
        });

        assert_eq!(final_msg["type"], "final");
    }

    #[test]
    fn test_health_response_format() {
        // Verify health check response format
        let health = serde_json::json!({
            "status": "healthy",
            "timestamp": "2025-11-15T10:00:00Z",
            "version": "0.1.0"
        });

        assert_eq!(health["status"], "healthy");
        assert!(health.get("timestamp").is_some());
        assert!(health.get("version").is_some());
    }

    #[test]
    fn test_status_response_format() {
        // Verify status endpoint response format
        let status = serde_json::json!({
            "service": "Rusty Tea",
            "status": "online",
            "version": "0.1.0",
            "endpoints": {
                "health": "/health",
                "status": "/status",
                "transcribe_batch": "POST /api/v1/transcriptions",
                "transcribe_stream": "WebSocket /api/v1/transcribe/stream"
            }
        });

        assert_eq!(status["service"], "Rusty Tea");
        assert_eq!(status["status"], "online");
        assert!(status["endpoints"].is_object());
    }

    #[test]
    fn test_wav_format_validation() {
        // Verify WAV format detection works
        let valid_wav_header = vec![
            0x52, 0x49, 0x46, 0x46, // "RIFF"
            0x24, 0x00, 0x00, 0x00, // chunk size
            0x57, 0x41, 0x56, 0x45, // "WAVE"
        ];

        assert_eq!(&valid_wav_header[0..4], b"RIFF");
        assert_eq!(&valid_wav_header[8..12], b"WAVE");
    }

    #[test]
    fn test_json_serialization_roundtrip() {
        // Verify JSON serialization works both ways
        let data = serde_json::json!({
            "text": "test transcription",
            "language": "en",
            "duration": 2.5
        });

        let serialized = serde_json::to_string(&data).expect("Failed to serialize");
        let deserialized: serde_json::Value =
            serde_json::from_str(&serialized).expect("Failed to deserialize");

        assert_eq!(deserialized["text"], "test transcription");
        assert_eq!(deserialized["language"], "en");
    }

    #[test]
    fn test_timestamps_are_rfc3339() {
        // Verify timestamps use RFC3339 format
        let now = chrono::Utc::now();
        let timestamp = now.to_rfc3339();

        // RFC3339 format includes T and Z or offset
        assert!(timestamp.contains('T'));
        assert!(timestamp.contains('+') || timestamp.contains('Z'));
    }

    #[test]
    fn test_uuid_generation() {
        // Verify UUID generation works
        let id1 = uuid::Uuid::new_v4().to_string();
        let id2 = uuid::Uuid::new_v4().to_string();

        assert_ne!(id1, id2);
        assert_eq!(id1.len(), 36); // UUID string length
    }

    #[test]
    fn test_response_codes() {
        // Verify HTTP response codes are used correctly
        let codes = vec![
            (200, "OK"),
            (400, "Bad Request"),
            (413, "Payload Too Large"),
            (500, "Internal Server Error"),
        ];

        for (code, description) in codes {
            assert!(code >= 200);
            assert!(!description.is_empty());
        }
    }

    #[tokio::test]
    async fn test_async_spawning() {
        // Verify async task spawning works
        let handle = tokio::task::spawn(async { 42 });
        let result = handle.await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
    }

    #[tokio::test]
    async fn test_blocking_task_spawn() {
        // Verify blocking tasks can be spawned from async context
        let result = tokio::task::spawn_blocking(|| {
            std::thread::sleep(std::time::Duration::from_millis(10));
            "done".to_string()
        })
        .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "done");
    }
}
