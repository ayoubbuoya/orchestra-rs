use crate::{
    messages::{HumanMessage, Message},
    providers::{Provider, gemini::types::GeminiChatResponse, types::ChatResponse},
};

use anyhow::{Error, Result};
use reqwest::header::HeaderMap;

use super::types::{
    GeminiContent, GeminiRequestBody, GeminiRequestPart, PREDEFINED_MODELS, SystemInstruction,
};

pub struct GeminiProvider;

impl GeminiProvider {
    pub const DEFAULT_API_KEY_ENV: &str = "GEMINI_API_KEY";
}

const BASE_URL: &str = "https://generativelanguage.googleapis.com/v1beta";

impl Provider for GeminiProvider {
    fn get_base_url(&self) -> &str {
        BASE_URL
    }

    fn get_predefined_models(&self) -> Result<Vec<String>, Error> {
        Ok(PREDEFINED_MODELS.iter().map(|s| s.to_string()).collect())
    }

    async fn prompt(
        &self,
        model_config: crate::model::ModelConfig,
        prompt: String,
    ) -> Result<ChatResponse, Error> {
        self.chat(
            model_config,
            Message::Human(HumanMessage { content: prompt }),
            vec![],
        )
        .await
    }

    async fn chat(
        &self,
        model_config: crate::model::ModelConfig,
        message: Message,
        chat_history: Vec<Message>,
    ) -> Result<ChatResponse> {
        let api_key = std::env::var(Self::DEFAULT_API_KEY_ENV)
            .map_err(|e| Error::msg(format!("Failed to get API key from environment: {}", e)))?;

        let client = reqwest::Client::new();

        let mut headers = HeaderMap::new();

        headers.insert("x-goog-api-key", api_key.parse()?);
        headers.insert("Content-Type", "application/json".parse()?);

        // Combine history + new_message
        let mut messages_to_send = chat_history.clone();
        messages_to_send.push(message);

        let model_id = model_config.name;

        let request_url = format!("{}/models/{}:generateContent", BASE_URL, model_id);

        let contents: Vec<GeminiContent> = messages_to_send
            .iter()
            .map(|m| GeminiContent::from(m))
            .collect();

        let request_body = GeminiRequestBody {
            system_instruction: model_config.system_instruction.map(|s| SystemInstruction {
                parts: vec![GeminiRequestPart { text: s }],
            }),
            contents,
        };

        let resp = client
            .post(request_url)
            .headers(headers)
            .json(&request_body)
            .send()
            .await?;

        let response_bod = resp.text().await?;

        let gemini_response: GeminiChatResponse = serde_json::from_str(&response_bod)?;

        println!("Response: {:?}", gemini_response);

        Ok(ChatResponse {
            text: gemini_response.candidates[0].content.parts[0]
                .text
                .clone()
                .unwrap_or_default(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::messages::{AssistantMessage, HumanMessage, Message};

    #[tokio::test]
    async fn test_prompt() {
        let provider = GeminiProvider;
        let model_config = crate::model::ModelConfig {
            name: PREDEFINED_MODELS[0].to_string(),
            system_instruction: None,
            temperature: 0.5,
            top_p: 0.5,
            thinking_mode: None,
        };

        let resp = provider
            .prompt(model_config, "Hello how you doing today?".to_string())
            .await
            .unwrap();

        assert!(!resp.text.is_empty());
    }

    #[tokio::test]
    async fn test_chat_with_history() {
        let provider = GeminiProvider;
        let model_config = crate::model::ModelConfig {
            name: PREDEFINED_MODELS[0].to_string(),
            system_instruction: None,
            temperature: 0.5,
            top_p: 0.5,
            thinking_mode: None,
        };

        // Simulate previous conversation history
        let history = vec![
            Message::Human(HumanMessage {
                content: "Hi, I'm Ayoub. I need you to remember my name when I ask for.".into(),
            }),
            Message::Assistant(AssistantMessage {
                content: "Got it!".into(),
            }),
        ];

        // New user message
        let new_message = Message::Human(HumanMessage {
            content: "What is My Name ?".into(),
        });

        let resp = provider
            .chat(model_config, new_message, history)
            .await
            .unwrap();

        println!("Chat response with history: {}", resp.text);
        assert!(!resp.text.is_empty());
    }
}
