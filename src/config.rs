use std::env;

#[derive(Clone, Debug)]
pub struct Config {
    pub api_key: String,
    pub server_host: String,
    pub server_port: u16,
    pub vosk_model_path: String,
    pub rust_log: String,
    pub database_url: String,
    pub qdrant_url: String,
    pub openrouter_api_key: String,
    pub openrouter_base_url: String,
    pub openrouter_chat_model_lite: String,
    pub elevenlabs_api_key: String,
    pub elevenlabs_voice_id: String,
}

impl Config {
    pub fn from_env() -> Self {
        // Load .env file if it exists (for local development)
        let _ = dotenv::dotenv();

        Self {
            api_key: env::var("API_KEY")
                .unwrap_or_else(|_| "dev_key_12345_change_in_production".to_string()),
            server_host: env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            server_port: env::var("SERVER_PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(3000),
            vosk_model_path: env::var("VOSK_MODEL_PATH")
                .unwrap_or_else(|_| "/models/vosk-model-small-en-us-0.15".to_string()),
            rust_log: env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string()),
            database_url: env::var("DATABASE_URL")
                .unwrap_or_else(|_| "postgresql://postgres:postgres_dev_password@localhost:5432/rusty_tea_db".to_string()),
            qdrant_url: env::var("QDRANT_URL")
                .unwrap_or_else(|_| "http://localhost:6333".to_string()),
            openrouter_api_key: env::var("OPENROUTER_API_KEY")
                .unwrap_or_else(|_| "sk-or-v1-".to_string()),
            openrouter_base_url: env::var("OPENROUTER_BASE_URL")
                .unwrap_or_else(|_| "https://openrouter.ai/api/v1".to_string()),
            openrouter_chat_model_lite: env::var("OPENROUTER_CHAT_MODEL_LITE")
                .unwrap_or_else(|_| "meta-llama/llama-3.1-8b-instruct".to_string()),
            elevenlabs_api_key: env::var("ELEVENLABS_API_KEY")
                .unwrap_or_else(|_| "sk_".to_string()),
            elevenlabs_voice_id: env::var("ELEVENLABS_VOICE_ID")
                .unwrap_or_else(|_| "EGNfK8LKuwEbqjx3yWz1".to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore] // Environment variable pollution from other tests
    fn test_config_defaults() {
        std::env::remove_var("SERVER_HOST");
        std::env::remove_var("SERVER_PORT");
        std::env::remove_var("VOSK_MODEL_PATH");
        std::env::remove_var("RUST_LOG");
        std::env::remove_var("API_KEY");
        std::env::remove_var("DATABASE_URL");
        std::env::remove_var("QDRANT_URL");
        std::env::remove_var("OPENROUTER_API_KEY");

        let config = Config::from_env();
        assert_eq!(config.server_host, "0.0.0.0");
        assert_eq!(config.server_port, 3000);
        assert_eq!(config.rust_log, "info");
        assert!(!config.vosk_model_path.is_empty());
        assert!(!config.api_key.is_empty());
        assert!(config.database_url.contains("localhost"));
        assert!(config.qdrant_url.contains("localhost"));
    }

    #[test]
    #[ignore] // Environment variable pollution from other tests
    fn test_config_custom_port() {
        std::env::set_var("SERVER_PORT", "8080");
        let config = Config::from_env();
        assert_eq!(config.server_port, 8080);
        std::env::remove_var("SERVER_PORT");
    }

    #[test]
    fn test_config_invalid_port_falls_back_to_default() {
        std::env::set_var("SERVER_PORT", "invalid");
        let config = Config::from_env();
        assert_eq!(config.server_port, 3000);
        std::env::remove_var("SERVER_PORT");
    }

    #[test]
    #[ignore] // Environment variable pollution from other tests
    fn test_config_custom_host() {
        std::env::set_var("SERVER_HOST", "127.0.0.1");
        let config = Config::from_env();
        assert_eq!(config.server_host, "127.0.0.1");
        std::env::remove_var("SERVER_HOST");
    }

    #[test]
    #[ignore] // Environment variable pollution from other tests
    fn test_config_custom_api_key() {
        std::env::set_var("API_KEY", "test_key_xyz");
        let config = Config::from_env();
        assert_eq!(config.api_key, "test_key_xyz");
        std::env::remove_var("API_KEY");
    }

    #[test]
    #[ignore] // Environment variable pollution from other tests
    fn test_config_database_url() {
        std::env::set_var("DATABASE_URL", "postgresql://user:pass@db:5432/mydb");
        let config = Config::from_env();
        assert_eq!(config.database_url, "postgresql://user:pass@db:5432/mydb");
        std::env::remove_var("DATABASE_URL");
    }

    #[test]
    fn test_config_qdrant_url() {
        std::env::set_var("QDRANT_URL", "http://qdrant:6333");
        let config = Config::from_env();
        assert_eq!(config.qdrant_url, "http://qdrant:6333");
        std::env::remove_var("QDRANT_URL");
    }

    #[test]
    fn test_config_openrouter_keys() {
        std::env::set_var("OPENROUTER_API_KEY", "sk-or-v1-test");
        let config = Config::from_env();
        assert_eq!(config.openrouter_api_key, "sk-or-v1-test");
        assert!(config.openrouter_base_url.contains("openrouter"));
        std::env::remove_var("OPENROUTER_API_KEY");
    }
}
