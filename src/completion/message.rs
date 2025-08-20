use serde::{Deserialize, Serialize};

// ================================================================
// Message models
// ================================================================

/// A message represents a run of input (user) and output (assistant).
/// Each provider is responsible with converting the generic message into it's provider specific
///  type using `From` or `TryFrom` traits.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(tag = "role", rename_all = "lowercase")]
pub enum Message {
    /// User message containing one or more content types defined by `UserContent`.
    User { content: Vec<UserContent> },

    /// Assistant message containing one or more content types defined by `AssistantContent`.
    Assistant {
        id: Option<String>,
        content: Vec<AssistantContent>,
    },
}

/// Describes the content of a message, which can be text, and other types in the future.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum UserContent {
    Text(Text),
}

/// Describes the content of a message, which can be text, and other types in the future.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum AssistantContent {
    Text(Text),
}

// ================================================================
// Base content models
// ================================================================

/// Basic text content.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct Text {
    pub text: String,
}
