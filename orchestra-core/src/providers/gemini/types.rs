use serde::{Deserialize, Serialize};

use crate::messages::Message;

pub const PREDEFINED_MODELS: &[&str] = &[
    "gemini-2.5-flash-lite",
    "gemini-2.5-pro",
    "gemini-2.5-flash",
    "gemini-2.0-flash-lite",
    "gemini-2.0-flash",
    "gemini-1.5-pro",
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeminiRequestBody {
    pub system_instruction: Option<SystemInstruction>,
    pub contents: Vec<GeminiContent>,
    #[serde(rename = "generationConfig", skip_serializing_if = "Option::is_none")]
    pub generation_config: Option<GeminiGenerationConfig>,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeminiGenerationConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(rename = "topP", skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
    #[serde(rename = "topK", skip_serializing_if = "Option::is_none")]
    pub top_k: Option<u32>,
    #[serde(rename = "maxOutputTokens", skip_serializing_if = "Option::is_none")]
    pub max_output_tokens: Option<u32>,
    #[serde(rename = "stopSequences", skip_serializing_if = "Option::is_none")]
    pub stop_sequences: Option<Vec<String>>,
}

impl GeminiGenerationConfig {
    pub fn from_model_config(config: &crate::model::ModelConfig) -> Self {
        Self {
            temperature: Some(config.temperature),
            top_p: Some(config.top_p),
            top_k: config.top_k,
            max_output_tokens: config.max_tokens,
            stop_sequences: if config.stop_sequences.is_empty() {
                None
            } else {
                Some(config.stop_sequences.clone())
            },
        }
    }
}

impl From<&Message> for GeminiContent {
    fn from(msg: &Message) -> Self {
        match msg {
            Message::Human(h) => GeminiContent {
                role: "user".to_string(),
                parts: vec![GeminiRequestPart {
                    text: h.content.to_text(),
                }],
            },
            Message::Assistant(a) => GeminiContent {
                role: "model".to_string(),
                parts: vec![GeminiRequestPart {
                    text: a.content.to_text(),
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

#[derive(Debug, Deserialize)]
pub struct GeminiChatResponse {
    pub candidates: Vec<GeminiCandidate>,
    #[serde(rename = "usageMetadata")]
    pub usage_metadata: Option<UsageMetadata>,
    #[serde(rename = "modelVersion")]
    pub model_version: Option<String>,
    #[serde(rename = "responseId")]
    pub response_id: Option<String>,
    pub error: Option<GeminiError>,
}

#[derive(Debug, Deserialize)]
pub struct GeminiError {
    pub code: u32,
    pub message: String,
    pub status: String,
}

#[derive(Debug, Deserialize)]
pub struct GeminiCandidate {
    pub content: GeminiContentResponse,
    #[serde(rename = "finishReason")]
    pub finish_reason: Option<String>,
    pub index: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct GeminiContentResponse {
    pub parts: Vec<GeminiPartResponse>,
    pub role: String,
}

#[derive(Debug, Deserialize)]
pub struct GeminiPartResponse {
    pub text: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UsageMetadata {
    #[serde(rename = "promptTokenCount")]
    pub prompt_token_count: u32,
    #[serde(rename = "candidatesTokenCount")]
    pub candidates_token_count: u32,
    #[serde(rename = "totalTokenCount")]
    pub total_token_count: u32,
    #[serde(rename = "promptTokensDetails")]
    pub prompt_tokens_details: Option<Vec<PromptTokensDetail>>,
}

#[derive(Debug, Deserialize)]
pub struct PromptTokensDetail {
    pub modality: String,
    #[serde(rename = "tokenCount")]
    pub token_count: u32,
}
