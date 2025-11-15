use anyhow::Result;
use tracing::{info, error, debug};
use vosk::{Model, Recognizer};

#[derive(Clone)]
pub struct VoskService {
    model_path: String,
}

impl VoskService {
    pub fn new(model_path: String) -> Self {
        Self { model_path }
    }

    pub async fn transcribe(&self, audio_data: Vec<u8>) -> Result<String> {
        let model_path = self.model_path.clone();
        
        tokio::task::spawn_blocking(move || {
            Self::transcribe_sync(&model_path, audio_data)
        })
        .await?
    }

    fn transcribe_sync(model_path: &str, audio_data: Vec<u8>) -> Result<String> {
        // Validate audio is WAV format
        let mut cursor = std::io::Cursor::new(&audio_data);
        let reader = hound::WavReader::new(&mut cursor)
            .map_err(|e| anyhow::anyhow!("Failed to read WAV: {}", e))?;

        // Validate audio format
        let spec = reader.spec();
        if spec.channels != 1 || spec.sample_rate != 16000 {
            return Err(anyhow::anyhow!(
                "Audio must be 16kHz mono WAV. Got: {}Hz {}ch",
                spec.sample_rate,
                spec.channels
            ));
        }

        info!("Processing {} bytes of 16kHz mono audio", audio_data.len());

        // Load Vosk model
        debug!("Loading Vosk model from: {}", model_path);
        let model = Model::new(model_path)
            .ok_or_else(|| anyhow::anyhow!("Failed to load Vosk model from: {}", model_path))?;

        // Create recognizer
        let mut recognizer = Recognizer::new(&model, 16000.0)
            .ok_or_else(|| anyhow::anyhow!("Failed to create Vosk recognizer"))?;

        // Extract audio samples (16-bit PCM)
        let mut cursor = std::io::Cursor::new(&audio_data);
        let mut reader = hound::WavReader::new(&mut cursor)
            .map_err(|e| anyhow::anyhow!("Failed to reopen WAV: {}", e))?;

        let samples: Vec<i16> = reader
            .samples::<i16>()
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| anyhow::anyhow!("Failed to read audio samples: {}", e))?;

        debug!("Feeding {} i16 samples to Vosk", samples.len());

        // Feed audio to recognizer in chunks (i16 samples, not bytes)
        let chunk_size = 2000; // Process 2000 samples at a time
        for chunk in samples.chunks(chunk_size) {
            recognizer.accept_waveform(chunk)?;
        }

        // Get final result (returns CompleteResult)
        let result = recognizer.final_result();
        
        debug!("Vosk raw result: {:?}", result);

        // Serialize CompleteResult to JSON string
        let result_json = serde_json::to_string(&result)
            .map_err(|e| anyhow::anyhow!("Failed to serialize Vosk result: {}", e))?;
        
        let parsed: serde_json::Value = serde_json::from_str(&result_json)
            .map_err(|e| anyhow::anyhow!("Failed to parse Vosk result: {}", e))?;

        let transcription = parsed["text"]
            .as_str()
            .unwrap_or("")
            .trim()
            .to_string();

        if transcription.is_empty() {
            error!("Vosk returned empty transcription");
            return Err(anyhow::anyhow!("No speech detected in audio"));
        }

        info!("Transcription: '{}'", transcription);
        Ok(transcription)
    }

    pub async fn transcribe_streaming(&self, audio_chunks: Vec<Vec<u8>>) -> Result<String> {
        let model_path = self.model_path.clone();

        tokio::task::spawn_blocking(move || {
            Self::transcribe_streaming_sync(&model_path, audio_chunks)
        })
        .await?
    }

    fn transcribe_streaming_sync(model_path: &str, audio_chunks: Vec<Vec<u8>>) -> Result<String> {
        let total_size: usize = audio_chunks.iter().map(|c| c.len()).sum();
        info!("Processing {} chunks totaling {} bytes", audio_chunks.len(), total_size);

        // Load Vosk model
        let model = Model::new(model_path)
            .ok_or_else(|| anyhow::anyhow!("Failed to load Vosk model from: {}", model_path))?;

        let mut recognizer = Recognizer::new(&model, 16000.0)
            .ok_or_else(|| anyhow::anyhow!("Failed to create Vosk recognizer"))?;

        // Process each chunk (convert u8 bytes to i16 samples)
        for chunk in audio_chunks {
            let samples: Vec<i16> = chunk
                .chunks_exact(2)
                .map(|b| i16::from_le_bytes([b[0], b[1]]))
                .collect();
            recognizer.accept_waveform(&samples)?;
        }

        // Get final result (returns CompleteResult)
        let result = recognizer.final_result();
        
        debug!("Vosk streaming result: {:?}", result);

        // Serialize CompleteResult to JSON string
        let result_json = serde_json::to_string(&result)
            .map_err(|e| anyhow::anyhow!("Failed to serialize Vosk result: {}", e))?;
        
        let parsed: serde_json::Value = serde_json::from_str(&result_json)
            .map_err(|e| anyhow::anyhow!("Failed to parse Vosk result: {}", e))?;

        let transcription = parsed["text"]
            .as_str()
            .unwrap_or("")
            .trim()
            .to_string();

        if transcription.is_empty() {
            return Err(anyhow::anyhow!("No speech detected in streaming audio"));
        }

        Ok(transcription)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vosk_service_creation() {
        let service = VoskService::new("/models/test".to_string());
        assert_eq!(service.model_path, "/models/test");
    }

    #[tokio::test]
    async fn test_transcribe_rejects_empty_audio() {
        let service = VoskService::new("/models/test".to_string());
        let result = service.transcribe(vec![]).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_transcribe_rejects_invalid_wav() {
        let service = VoskService::new("/models/test".to_string());
        let invalid_audio = vec![0xFF, 0xFE, 0x00, 0x00];
        let result = service.transcribe(invalid_audio).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_transcribe_valid_mock_audio() {
        let service = VoskService::new("/models/test".to_string());
        // Create a minimal valid WAV structure (44 bytes header + empty audio)
        let mut wav = vec![];
        wav.extend_from_slice(b"RIFF");
        wav.extend_from_slice(&(36u32).to_le_bytes()); // chunk size
        wav.extend_from_slice(b"WAVE");
        wav.extend_from_slice(b"fmt ");
        wav.extend_from_slice(&(16u32).to_le_bytes()); // subchunk1size
        wav.extend_from_slice(&(1u16).to_le_bytes());  // audio format (PCM)
        wav.extend_from_slice(&(1u16).to_le_bytes());  // channels (mono)
        wav.extend_from_slice(&(16000u32).to_le_bytes()); // sample rate
        wav.extend_from_slice(&(32000u32).to_le_bytes()); // byte rate
        wav.extend_from_slice(&(2u16).to_le_bytes());  // block align
        wav.extend_from_slice(&(16u16).to_le_bytes()); // bits per sample
        wav.extend_from_slice(b"data");
        wav.extend_from_slice(&(0u32).to_le_bytes()); // subchunk2size

        let result = service.transcribe(wav).await;
        assert!(result.is_ok());
        let text = result.unwrap();
        assert!(text.contains("transcription"));
    }

    #[tokio::test]
    async fn test_transcribe_streaming() {
        let service = VoskService::new("/models/test".to_string());
        let audio_chunks = vec![vec![0; 1024], vec![0; 1024]];
        let result = service.transcribe_streaming(audio_chunks).await;
        assert!(result.is_ok());
        let text = result.unwrap();
        assert!(text.contains("Streaming"));
    }
}
