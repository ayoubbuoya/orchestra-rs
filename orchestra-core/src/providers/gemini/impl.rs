use crate::{
    messages::Message,
    providers::{Provider, types::ChatResponse},
};

use anyhow::{Error, Result};
use reqwest::header::HeaderMap;
use serde::{Deserialize, Serialize};

pub struct GeminiProvider;

impl GeminiProvider {
    pub const DEFAULT_API_KEY_ENV: &str = "GEMINI_API_KEY";
}

const PREDEFINED_MODELS: &[&str] = &[
    "gemini-2.5-flash-lite",
    "gemini-2.5-pro",
    "gemini-2.5-flash",
    "gemini-2.0-flash-lite",
    "gemini-2.0-flash",
    "gemini-1.5-pro",
];

const BASE_URL: &str = "https://generativelanguage.googleapis.com/v1beta";

impl Provider for GeminiProvider {
    fn get_base_url(&self) -> &str {
        BASE_URL
    }

    fn get_predefined_models(&self) -> Result<Vec<String>, Error> {
        Ok(PREDEFINED_MODELS.iter().map(|s| s.to_string()).collect())
    }

    async fn chat(
        &self,
        model_config: crate::model::ModelConfig,
        message: Message,
    ) -> Result<ChatResponse> {
        let api_key = std::env::var(Self::DEFAULT_API_KEY_ENV)
            .map_err(|e| Error::msg(format!("Failed to get API key from environment: {}", e)))?;

        let client = reqwest::Client::new();

        let mut headers = HeaderMap::new();

        headers.insert("x-goog-api-key", api_key.parse()?);
        headers.insert("Content-Type", "application/json".parse()?);

        let model_id = model_config.name;

        let request_url = format!("{}/models/{}:generateContent", BASE_URL, model_id);

        let request_body = GeminiRequestBody {
            system_instruction: model_config.system_instruction.map(|s| SystemInstruction {
                parts: vec![GeminiRequestPart { text: s }],
            }),
            contents: vec![GeminiContent::from(&message)],
        };

        let resp = client
            .post(request_url)
            .headers(headers)
            .json(&request_body)
            .send()
            .await?;

        let response_bod = resp.text().await?;

        println!("Response: {}", response_bod);

        Ok(ChatResponse {
            content: response_bod,
        })
    }
}

pub struct GeminiGenerationConfig {
    pub temperature: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeminiRequestBody {
    pub system_instruction: Option<SystemInstruction>,
    pub contents: Vec<GeminiContent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInstruction {
    pub parts: Vec<GeminiRequestPart>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeminiContent {
    pub role: String,
    pub parts: Vec<GeminiRequestPart>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeminiRequestPart {
    pub text: String,
}

impl From<&Message> for GeminiContent {
    fn from(msg: &Message) -> Self {
        match msg {
            Message::Human(h) => GeminiContent {
                role: "user".to_string(),
                parts: vec![GeminiRequestPart {
                    text: h.content.clone(),
                }],
            },
            Message::Assistant(a) => GeminiContent {
                role: "model".to_string(),
                parts: vec![GeminiRequestPart {
                    text: a.content.clone(),
                }],
            },
            Message::System(s) => GeminiContent {
                role: "system".to_string(),
                parts: vec![GeminiRequestPart {
                    text: s.content.clone(),
                }],
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Add a test for the chat function.
    #[tokio::test]
    async fn test_chat() {
        let provider = GeminiProvider;
        let model_config = crate::model::ModelConfig {
            name: PREDEFINED_MODELS[0].to_string(),
            system_instruction: None,
            temperature: 0.5,
            top_p: 0.5,
            thinking_mode: None,
        };

        let message = Message::Human(crate::messages::HumanMessage {
            content: "Hello World".to_string(),
        });

        let resp = provider.chat(model_config, message).await.unwrap();

        assert!(!resp.content.is_empty());
    }
}
