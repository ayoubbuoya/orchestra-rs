use serde::{Deserialize, Serialize};

use crate::{messages::Message, tools::ToolDefinition};

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
    /// Tool definitions for function calling
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<GeminiTool>>,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(rename = "functionCall", skip_serializing_if = "Option::is_none")]
    pub function_call: Option<GeminiFunctionCall>,
    #[serde(rename = "functionResponse", skip_serializing_if = "Option::is_none")]
    pub function_response: Option<GeminiFunctionResponse>,
}

impl GeminiRequestPart {
    /// Create a text part
    pub fn text<S: Into<String>>(text: S) -> Self {
        Self {
            text: Some(text.into()),
            function_call: None,
            function_response: None,
        }
    }

    /// Create a function call part
    pub fn function_call(call: GeminiFunctionCall) -> Self {
        Self {
            text: None,
            function_call: Some(call),
            function_response: None,
        }
    }

    /// Create a function response part
    pub fn function_response(response: GeminiFunctionResponse) -> Self {
        Self {
            text: None,
            function_call: None,
            function_response: Some(response),
        }
    }
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
            Message::Human(h) => {
                let mut parts = vec![];

                // Add text content if present
                if let Some(text) = h.content.as_text() {
                    if !text.is_empty() {
                        parts.push(GeminiRequestPart::text(text));
                    }
                }

                // Add tool calls if present
                for tool_call in h.content.tool_calls() {
                    parts.push(GeminiRequestPart::function_call(GeminiFunctionCall {
                        name: tool_call.function.name.clone(),
                        args: tool_call.function.arguments.clone(),
                    }));
                }

                // If no parts, add empty text
                if parts.is_empty() {
                    parts.push(GeminiRequestPart::text(""));
                }

                GeminiContent {
                    role: "user".to_string(),
                    parts,
                }
            },
            Message::Assistant(a) => {
                let mut parts = vec![];

                // Add text content if present
                if let Some(text) = a.content.as_text() {
                    if !text.is_empty() {
                        parts.push(GeminiRequestPart::text(text));
                    }
                }

                // Add tool calls if present
                for tool_call in a.content.tool_calls() {
                    parts.push(GeminiRequestPart::function_call(GeminiFunctionCall {
                        name: tool_call.function.name.clone(),
                        args: tool_call.function.arguments.clone(),
                    }));
                }

                // If no parts, add empty text
                if parts.is_empty() {
                    parts.push(GeminiRequestPart::text(""));
                }

                GeminiContent {
                    role: "model".to_string(),
                    parts,
                }
            },
            Message::System(s) => GeminiContent {
                role: "system".to_string(),
                parts: vec![GeminiRequestPart::text(s.content.clone())],
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
    #[serde(rename = "functionCall")]
    pub function_call: Option<GeminiFunctionCall>,
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

// Tool calling support structures

/// Represents a tool definition in Gemini's format
///
/// Gemini uses a specific format for function calling that differs slightly
/// from our internal ToolDefinition format.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeminiTool {
    #[serde(rename = "functionDeclarations")]
    pub function_declarations: Vec<GeminiFunctionDeclaration>,
}

/// A function declaration for Gemini's function calling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeminiFunctionDeclaration {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value, // JSON schema
}

/// Convert our ToolDefinition to Gemini's format
impl From<&ToolDefinition> for GeminiFunctionDeclaration {
    fn from(tool_def: &ToolDefinition) -> Self {
        Self {
            name: tool_def.name.clone(),
            description: tool_def.description.clone(),
            parameters: tool_def.to_json_schema(),
        }
    }
}

/// Convert a list of ToolDefinitions to a GeminiTool
impl From<Vec<ToolDefinition>> for GeminiTool {
    fn from(tool_defs: Vec<ToolDefinition>) -> Self {
        Self {
            function_declarations: tool_defs.iter()
                .map(GeminiFunctionDeclaration::from)
                .collect(),
        }
    }
}

/// Represents a function call in Gemini's format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeminiFunctionCall {
    pub name: String,
    pub args: serde_json::Value,
}

/// Represents a function response in Gemini's format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeminiFunctionResponse {
    pub name: String,
    pub response: serde_json::Value,
}
