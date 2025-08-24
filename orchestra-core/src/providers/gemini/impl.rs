use crate::{
    error::{OrchestraError, Result},
    messages::Message,
    providers::{
        Provider, config::GeminiConfig, gemini::types::GeminiChatResponse, types::ChatResponse,
    },
};

use async_trait::async_trait;
use reqwest::header::HeaderMap;

use super::types::{
    GeminiContent, GeminiGenerationConfig, GeminiRequestBody, GeminiRequestPart, PREDEFINED_MODELS,
    SystemInstruction,
};

#[derive(Debug)]
pub struct GeminiProvider {
    config: GeminiConfig,
}

impl GeminiProvider {
    pub const DEFAULT_API_KEY_ENV: &str = "GEMINI_API_KEY";

    /// Create a new GeminiProvider with default configuration
    pub fn with_default_config() -> Self {
        Self {
            config: GeminiConfig::default(),
        }
    }
}

#[async_trait]
impl Provider for GeminiProvider {
    type Config = GeminiConfig;

    fn new(config: Self::Config) -> Self {
        Self { config }
    }

    fn get_base_url(&self) -> &str {
        // We'll store the base URL in the provider for efficiency
        "https://generativelanguage.googleapis.com/v1beta"
    }

    fn name(&self) -> &'static str {
        "gemini"
    }

    fn supports_tools(&self) -> bool {
        true // Gemini supports function calling
    }

    fn get_predefined_models(&self) -> Result<Vec<String>> {
        Ok(PREDEFINED_MODELS.iter().map(|s| s.to_string()).collect())
    }

    async fn prompt(
        &self,
        model_config: crate::model::ModelConfig,
        prompt: String,
    ) -> Result<ChatResponse> {
        self.chat(model_config, Message::human(prompt), vec![])
            .await
    }

    async fn chat(
        &self,
        model_config: crate::model::ModelConfig,
        message: Message,
        chat_history: Vec<Message>,
    ) -> Result<ChatResponse> {
        let api_key = self.config.get_api_key().ok_or_else(|| {
            OrchestraError::api_key("API key not found in configuration or environment")
        })?;

        let client = reqwest::Client::new();

        let mut headers = HeaderMap::new();

        headers.insert("x-goog-api-key", api_key.parse()?);
        headers.insert("Content-Type", "application/json".parse()?);

        // Combine history + new_message
        let mut messages_to_send = chat_history.clone();
        messages_to_send.push(message);

        let model_id = &model_config.name;
        let request_url = format!(
            "{}/models/{}:generateContent",
            self.get_base_url(),
            model_id
        );

        let contents: Vec<GeminiContent> = messages_to_send
            .iter()
            .map(|m| GeminiContent::from(m))
            .collect();

        let generation_config = GeminiGenerationConfig::from_model_config(&model_config);

        let request_body = GeminiRequestBody {
            system_instruction: model_config.system_instruction.clone().map(|s| {
                SystemInstruction {
                    parts: vec![GeminiRequestPart { text: s }],
                }
            }),
            contents,
            generation_config: Some(generation_config),
        };

        let resp = client
            .post(request_url)
            .headers(headers)
            .json(&request_body)
            .send()
            .await?;

        // Check for HTTP errors
        if !resp.status().is_success() {
            let status = resp.status();
            let error_body = resp.text().await.unwrap_or_default();
            return Err(OrchestraError::provider(
                "gemini",
                &format!("HTTP {} error: {}", status, error_body),
            ));
        }

        let response_body = resp.text().await?;

        println!("Response body: {}", response_body);

        let gemini_response: GeminiChatResponse = serde_json::from_str(&response_body)?;

        // Check for API errors in the response
        if let Some(error) = gemini_response.error {
            return Err(OrchestraError::provider(
                "gemini",
                &format!(
                    "API error {}: {} ({})",
                    error.code, error.message, error.status
                ),
            ));
        }

        // Better error handling for response structure
        let candidate = gemini_response
            .candidates
            .first()
            .ok_or_else(|| OrchestraError::invalid_response("No candidates in response"))?;

        let part = candidate
            .content
            .parts
            .first()
            .ok_or_else(|| OrchestraError::invalid_response("No parts in response content"))?;

        let text = part
            .text
            .as_ref()
            .ok_or_else(|| OrchestraError::invalid_response("No text in response part"))?;

        Ok(ChatResponse { text: text.clone() })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::messages::Message;

    #[tokio::test]
    async fn test_prompt() {
        let provider = GeminiProvider::with_default_config();
        let model_config = crate::model::ModelConfig::new(PREDEFINED_MODELS[0])
            .with_temperature(0.5)
            .unwrap()
            .with_top_p(0.5)
            .unwrap();

        let resp = provider
            .prompt(model_config, "Hello how you doing today?".to_string())
            .await
            .unwrap();

        assert!(!resp.text.is_empty());
    }

    #[tokio::test]
    async fn test_chat_with_history() {
        let provider = GeminiProvider::with_default_config();
        let model_config = crate::model::ModelConfig::new(PREDEFINED_MODELS[0])
            .with_temperature(0.5)
            .unwrap()
            .with_top_p(0.5)
            .unwrap();

        // Simulate previous conversation history
        let history = vec![
            Message::human("Hi, I'm Ayoub. I need you to remember my name when I ask for."),
            Message::assistant("Got it!"),
        ];

        // New user message
        let new_message = Message::human("What is My Name ?");

        let resp = provider
            .chat(model_config, new_message, history)
            .await
            .unwrap();

        println!("Chat response with history: {}", resp.text);
        assert!(!resp.text.is_empty());
    }

    #[tokio::test]
    async fn test_chat_with_system_instruction() {
        let provider = GeminiProvider::with_default_config();
        let model_config = crate::model::ModelConfig::new(PREDEFINED_MODELS[0])
            .with_system_instruction("You are a helpful assistant that you'll add your name to the end of each response which is BuoyaAI.")
            .with_temperature(0.5)
            .unwrap()
            .with_top_p(0.5)
            .unwrap();

        let resp = provider
            .prompt(model_config, "Hello how you doing today?".to_string())
            .await
            .unwrap();

        println!("Chat response with system instruction: {}", resp.text);

        assert!(!resp.text.is_empty());
        assert!(resp.text.contains("BuoyaAI"));
    }
}
