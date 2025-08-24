use crate::{
    error::{OrchestraError, Result},
    messages::{Message, ToolCall, ToolFunction},
    providers::{
        Provider, config::GeminiConfig, gemini::types::GeminiChatResponse,
        types::{ChatResponse, ChatResponseMetadata, TokenUsage},
    },
    tools::ToolDefinition,
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
                    parts: vec![GeminiRequestPart::text(s)],
                }
            }),
            contents,
            generation_config: Some(generation_config),
            tools: None, // No tools for regular chat
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

        Ok(ChatResponse::text(text.clone()))
    }

    /// Implementation of chat_with_tools for Gemini provider
    ///
    /// This method extends the regular chat functionality to support tool calling.
    /// It sends tool definitions to Gemini and parses any tool calls in the response.
    async fn chat_with_tools(
        &self,
        model_config: crate::model::ModelConfig,
        message: Message,
        chat_history: Vec<Message>,
        tools: Vec<ToolDefinition>,
    ) -> Result<ChatResponse> {
        let api_key = self.config.get_api_key().ok_or_else(|| {
            OrchestraError::api_key("API key not found in configuration or environment")
        })?;

        let client = reqwest::Client::new();

        let mut headers = reqwest::header::HeaderMap::new();
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

        let contents: Vec<super::types::GeminiContent> = messages_to_send
            .iter()
            .map(|m| super::types::GeminiContent::from(m))
            .collect();

        let generation_config = super::types::GeminiGenerationConfig::from_model_config(&model_config);

        // Convert tools to Gemini format
        let gemini_tools = if tools.is_empty() {
            None
        } else {
            Some(vec![super::types::GeminiTool::from(tools)])
        };

        let request_body = super::types::GeminiRequestBody {
            system_instruction: model_config.system_instruction.clone().map(|s| {
                super::types::SystemInstruction {
                    parts: vec![super::types::GeminiRequestPart::text(s)],
                }
            }),
            contents,
            generation_config: Some(generation_config),
            tools: gemini_tools,
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

        let response_body: GeminiChatResponse = resp.json().await?;

        // Check for API errors
        if let Some(error) = response_body.error {
            return Err(OrchestraError::provider(
                "gemini",
                &format!("Gemini API error {}: {}", error.code, error.message),
            ));
        }

        // Extract the response
        let candidate = response_body
            .candidates
            .first()
            .ok_or_else(|| OrchestraError::invalid_response("No candidates in response"))?;

        let content = &candidate.content;

        // Parse response parts for text and function calls
        let mut response_text = String::new();
        let mut tool_calls = Vec::new();

        for (index, part) in content.parts.iter().enumerate() {
            if let Some(text) = &part.text {
                if !response_text.is_empty() {
                    response_text.push(' ');
                }
                response_text.push_str(text);
            }

            if let Some(function_call) = &part.function_call {
                tool_calls.push(ToolCall {
                    id: format!("call_{}", index), // Generate a unique ID
                    call_id: Some(format!("call_{}", index)),
                    function: ToolFunction {
                        name: function_call.name.clone(),
                        arguments: function_call.args.clone(),
                    },
                });
            }
        }

        // Create metadata
        let metadata = ChatResponseMetadata {
            usage: response_body.usage_metadata.map(|usage| TokenUsage {
                prompt_tokens: usage.prompt_token_count,
                completion_tokens: usage.candidates_token_count,
                total_tokens: usage.total_token_count,
            }),
            model: response_body.model_version,
            response_id: response_body.response_id,
            processing_time_ms: None, // We don't track this currently
            finish_reason: candidate.finish_reason.clone(),
        };

        // Create the response
        let mut response = if tool_calls.is_empty() {
            ChatResponse::text(response_text)
        } else {
            ChatResponse::with_tool_calls(response_text, tool_calls)
        };

        response = response.with_metadata(metadata);

        Ok(response)
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
