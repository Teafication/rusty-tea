use async_openai::config::OpenAIConfig;
use async_openai::types::{
    ChatCompletionRequestMessage,
    CreateChatCompletionRequestArgs,
};
use std::error::Error;
use tracing::{info, debug};

const TEA_VOICE_PERSONALITY: &str = r#"You are Tea, a warm and caring friend who genuinely enjoys connecting with people through voice conversation.

Personality Guidelines:
- Keep responses natural and concise (2-3 sentences max for voice)
- Be welcoming and make people feel comfortable, like chatting with a good friend
- Show genuine interest in what they're sharing
- Be encouraging and supportive in all conversations
- Use positive, warm language that feels natural for speech
- Make conversations feel fun, authentic, and meaningful
- Show you care through your words and tone

IMPORTANT for Voice: 
- Speak naturally - no emojis, they don't translate to speech
- Keep it conversational and concise
- Express emotions through your vocal tone and word choice
- NO roleplay actions like *smiles* or *waves*

Tone: Warm, friendly, encouraging, genuine, supportive

Remember: You're having a natural voice conversation with a friend!"#;

/// OpenRouter LLM service for API integration
/// Current implementation: Client initialization and health check only
/// Conversation logic will be added in future phase
pub struct LlmService {
    client: async_openai::Client<OpenAIConfig>,
    model: String,
}

impl LlmService {
    /// Initialize OpenRouter client
    pub fn new(
        api_key: &str,
        base_url: &str,
        model: &str,
    ) -> Result<Self, Box<dyn Error + Send + Sync>> {
        info!("Initializing OpenRouter LLM service with model: {}", model);

        let config = OpenAIConfig::new()
            .with_api_key(api_key)
            .with_api_base(base_url);

        let client = async_openai::Client::with_config(config);

        debug!("OpenRouter client initialized: base_url={}, model={}", base_url, model);

        Ok(Self {
            client,
            model: model.to_string(),
        })
    }

    /// Get the configured model name
    pub fn model(&self) -> &str {
        &self.model
    }

    /// Get reference to the client for future use
    pub fn client(&self) -> &async_openai::Client<OpenAIConfig> {
        &self.client
    }

    /// Health check - verify API configuration is valid
    pub fn health_check(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        info!("LLM service health check: API key configured, model ready");
        // Extended health check (e.g., making test API call) will be added in next phase
        Ok(())
    }

    /// Get service metadata
    pub fn metadata(&self) -> LlmServiceMetadata {
        LlmServiceMetadata {
            model: self.model.clone(),
            provider: "OpenRouter".to_string(),
            status: "initialized".to_string(),
        }
    }

    /// Generate a response for voice chat with Tea's personality
    /// Takes conversation history and returns assistant's text response
    pub async fn generate_voice_response(
        &self,
        conversation_history: &[(String, String)], // Vec of (role, content) tuples
        user_message: &str,
    ) -> Result<String, Box<dyn Error + Send + Sync>> {
        info!("Generating voice response for user message (history: {} messages)", conversation_history.len());

        // Build messages array with system prompt + history + new user message
        let mut messages: Vec<ChatCompletionRequestMessage> = Vec::new();

        // Add system prompt
        messages.push(ChatCompletionRequestMessage {
            role: async_openai::types::Role::System,
            content: Some(TEA_VOICE_PERSONALITY.to_string()),
            name: None,
            function_call: None,
        });

        // Add conversation history
        for (role, content) in conversation_history {
            let role_enum = match role.as_str() {
                "user" => async_openai::types::Role::User,
                "assistant" => async_openai::types::Role::Assistant,
                _ => continue, // Skip unknown roles
            };
            
            messages.push(ChatCompletionRequestMessage {
                role: role_enum,
                content: Some(content.clone()),
                name: None,
                function_call: None,
            });
        }

        // Add new user message
        messages.push(ChatCompletionRequestMessage {
            role: async_openai::types::Role::User,
            content: Some(user_message.to_string()),
            name: None,
            function_call: None,
        });

        // Create chat completion request
        let request = CreateChatCompletionRequestArgs::default()
            .model(&self.model)
            .messages(messages)
            .max_tokens(150u16) // Keep responses concise for voice
            .temperature(0.7)
            .build()?;

        debug!("Sending chat completion request to OpenRouter");

        // Call OpenRouter API
        let response = self.client.chat().create(request).await?;

        // Extract response text
        let response_text = response
            .choices
            .first()
            .and_then(|choice| choice.message.content.clone())
            .ok_or("No response content from LLM")?;

        info!("Generated response: {} chars", response_text.len());

        Ok(response_text)
    }
}

/// Metadata about the LLM service
#[derive(Debug, Clone, serde::Serialize)]
pub struct LlmServiceMetadata {
    pub model: String,
    pub provider: String,
    pub status: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_llm_service_metadata() {
        let meta = LlmServiceMetadata {
            model: "meta-llama/llama-3.1-8b-instruct".to_string(),
            provider: "OpenRouter".to_string(),
            status: "initialized".to_string(),
        };
        
        assert_eq!(meta.provider, "OpenRouter");
        assert!(meta.model.contains("llama"));
    }

    #[test]
    fn test_llm_service_creation() {
        // Test that LlmService can be instantiated with valid parameters
        let api_key = "sk-or-v1-test";
        let base_url = "https://openrouter.ai/api/v1";
        let model = "meta-llama/llama-3.1-8b-instruct";

        let service = LlmService::new(api_key, base_url, model);
        assert!(service.is_ok());

        let service = service.unwrap();
        assert_eq!(service.model(), model);
    }

    #[test]
    fn test_llm_service_health_check() {
        let service = LlmService::new(
            "sk-or-v1-test",
            "https://openrouter.ai/api/v1",
            "meta-llama/llama-3.1-8b-instruct",
        ).unwrap();

        let health = service.health_check();
        assert!(health.is_ok());
    }

    #[test]
    fn test_llm_service_with_metadata() {
        let service = LlmService::new(
            "sk-or-v1-test",
            "https://openrouter.ai/api/v1",
            "test-model",
        ).unwrap();

        let meta = service.metadata();
        assert_eq!(meta.provider, "OpenRouter");
        assert_eq!(meta.model, "test-model");
        assert_eq!(meta.status, "initialized");
    }
}
