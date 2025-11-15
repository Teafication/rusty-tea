use anyhow::{Context, Result};
use bytes::Bytes;
use reqwest::Client;
use serde::Serialize;
use tracing::{info, warn};

#[derive(Debug, Clone, Serialize)]
struct VoiceSettings {
    stability: f32,
    similarity_boost: f32,
    style: f32,
    use_speaker_boost: bool,
}

impl Default for VoiceSettings {
    fn default() -> Self {
        Self {
            stability: 0.5,
            similarity_boost: 0.75,
            style: 0.0,
            use_speaker_boost: true,
        }
    }
}

#[derive(Debug, Serialize)]
struct TextToSpeechRequest {
    text: String,
    model_id: String,
    voice_settings: VoiceSettings,
}

#[derive(Debug, Clone)]
pub struct ElevenLabsService {
    client: Client,
    api_key: String,
    voice_id: String,
    base_url: String,
}

impl ElevenLabsService {
    pub fn new(api_key: String, voice_id: String) -> Result<Self> {
        info!("Initializing ElevenLabs TTS service with voice_id: {}", voice_id);
        
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .context("Failed to create HTTP client for ElevenLabs")?;

        Ok(Self {
            client,
            api_key,
            voice_id,
            base_url: "https://api.elevenlabs.io/v1".to_string(),
        })
    }

    /// Convert text to speech using ElevenLabs API
    /// Returns MP3 audio bytes
    pub async fn text_to_speech(&self, text: &str) -> Result<Bytes> {
        let url = format!("{}/text-to-speech/{}", self.base_url, self.voice_id);
        
        let request_body = TextToSpeechRequest {
            text: text.to_string(),
            model_id: "eleven_turbo_v2_5".to_string(),
            voice_settings: VoiceSettings::default(),
        };

        info!("Sending TTS request to ElevenLabs (text length: {} chars)", text.len());

        let response = self.client
            .post(&url)
            .header("xi-api-key", &self.api_key)
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await
            .context("Failed to send request to ElevenLabs API")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_body = response.text().await.unwrap_or_default();
            warn!("ElevenLabs API error ({}): {}", status, error_body);
            anyhow::bail!("ElevenLabs API returned error status {}: {}", status, error_body);
        }

        let audio_bytes = response
            .bytes()
            .await
            .context("Failed to read audio bytes from ElevenLabs response")?;

        info!("Successfully generated {} bytes of MP3 audio", audio_bytes.len());

        Ok(audio_bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_elevenlabs_service_creation() {
        let service = ElevenLabsService::new(
            "test_api_key".to_string(),
            "test_voice_id".to_string(),
        );
        assert!(service.is_ok());
    }

    #[test]
    fn test_voice_settings_defaults() {
        let settings = VoiceSettings::default();
        assert_eq!(settings.stability, 0.5);
        assert_eq!(settings.similarity_boost, 0.75);
        assert_eq!(settings.style, 0.0);
        assert!(settings.use_speaker_boost);
    }
}
