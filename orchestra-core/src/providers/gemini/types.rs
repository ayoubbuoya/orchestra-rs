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

pub struct GeminiGenerationConfig {
    pub temperature: f32,
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

#[derive(Debug, Deserialize)]
pub struct GeminiChatResponse {
    pub candidates: Vec<GeminiCandidate>,
    #[serde(rename = "usageMetadata")]
    pub usage_metadata: Option<UsageMetadata>,
    #[serde(rename = "modelVersion")]
    pub model_version: Option<String>,
    #[serde(rename = "responseId")]
    pub response_id: Option<String>,
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
