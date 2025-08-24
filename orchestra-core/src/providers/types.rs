/// This is a list of all provider sources that are supported.
#[derive(Debug, Clone, Copy)]
pub enum ProviderSource {
    Gemini,
    OpenAI,
}

impl ProviderSource {
    pub fn as_str(&self) -> &str {
        match self {
            ProviderSource::Gemini => "gemini",
            ProviderSource::OpenAI => "openai",
        }
    }

    pub fn from_str(s: &str) -> Option<ProviderSource> {
        match s.to_lowercase().as_str() {
            "gemini" => Some(ProviderSource::Gemini),
            "openai" => Some(ProviderSource::OpenAI),
            _ => None,
        }
    }
}

use serde::{Deserialize, Serialize};
use crate::messages::ToolCall;

/// Response from a chat request to a provider
///
/// This struct contains the response from an LLM, which can include both
/// text content and tool calls that the LLM wants to execute.
///
/// ## For Rust Beginners
///
/// - `Option<T>` means the value might be present or absent (None)
/// - `Vec<T>` is a growable array (vector) of items of type T
/// - The `#[derive(...)]` attributes automatically implement common traits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatResponse {
    /// The text response from the LLM
    pub text: String,

    /// Tool calls requested by the LLM (if any)
    pub tool_calls: Option<Vec<ToolCall>>,

    /// Additional metadata about the response
    pub metadata: Option<ChatResponseMetadata>,
}

impl ChatResponse {
    /// Create a simple text response
    pub fn text<S: Into<String>>(text: S) -> Self {
        Self {
            text: text.into(),
            tool_calls: None,
            metadata: None,
        }
    }

    /// Create a response with tool calls
    pub fn with_tool_calls<S: Into<String>>(text: S, tool_calls: Vec<ToolCall>) -> Self {
        Self {
            text: text.into(),
            tool_calls: Some(tool_calls),
            metadata: None,
        }
    }

    /// Add metadata to the response
    pub fn with_metadata(mut self, metadata: ChatResponseMetadata) -> Self {
        self.metadata = Some(metadata);
        self
    }

    /// Check if this response contains tool calls
    pub fn has_tool_calls(&self) -> bool {
        self.tool_calls.as_ref().map_or(false, |calls| !calls.is_empty())
    }

    /// Get the tool calls, if any
    pub fn get_tool_calls(&self) -> &[ToolCall] {
        self.tool_calls.as_deref().unwrap_or(&[])
    }
}

/// Metadata about a chat response
///
/// This contains additional information about the response that might be
/// useful for debugging, monitoring, or billing purposes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatResponseMetadata {
    /// Token usage information
    pub usage: Option<TokenUsage>,

    /// The model that generated the response
    pub model: Option<String>,

    /// Unique identifier for this response
    pub response_id: Option<String>,

    /// How long the request took to process
    pub processing_time_ms: Option<u64>,

    /// The finish reason (completed, length, tool_calls, etc.)
    pub finish_reason: Option<String>,
}

/// Token usage information
///
/// This tracks how many tokens were used in the request and response,
/// which is important for billing and rate limiting.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsage {
    /// Tokens used in the prompt/input
    pub prompt_tokens: u32,

    /// Tokens used in the completion/output
    pub completion_tokens: u32,

    /// Total tokens used (prompt + completion)
    pub total_tokens: u32,
}
