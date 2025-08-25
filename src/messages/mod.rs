use serde::{Deserialize, Serialize};

/// Represents different types of messages in a conversation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Message {
    /// Message from a human user
    Human(HumanMessage),
    /// Message from an AI assistant
    Assistant(AssistantMessage),
    /// System instruction or context message
    System(SystemMessage),
}

/// Message from a human user
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HumanMessage {
    pub content: MessageContent,
}

/// Message from an AI assistant
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AssistantMessage {
    pub content: MessageContent,
}

/// System instruction or context message
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SystemMessage {
    pub content: String,
}

/// Content of a message, which can be text or include tool calls
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MessageContent {
    /// Simple text content
    Text(String),
    /// Mixed content with text and tool calls
    Mixed {
        text: Option<String>,
        tool_calls: Vec<ToolCall>,
    },
}

impl MessageContent {
    /// Create a new text content
    pub fn text<S: Into<String>>(text: S) -> Self {
        Self::Text(text.into())
    }

    /// Create mixed content with text and tool calls
    pub fn mixed<S: Into<String>>(text: Option<S>, tool_calls: Vec<ToolCall>) -> Self {
        Self::Mixed {
            text: text.map(|t| t.into()),
            tool_calls,
        }
    }

    /// Get the text content, if any
    pub fn as_text(&self) -> Option<&str> {
        match self {
            Self::Text(text) => Some(text),
            Self::Mixed { text, .. } => text.as_deref(),
        }
    }

    /// Get the text content as a string, combining all text parts
    pub fn to_text(&self) -> String {
        match self {
            Self::Text(text) => text.clone(),
            Self::Mixed { text, .. } => text.clone().unwrap_or_default(),
        }
    }

    /// Check if this content has tool calls
    pub fn has_tool_calls(&self) -> bool {
        matches!(self, Self::Mixed { tool_calls, .. } if !tool_calls.is_empty())
    }

    /// Get tool calls, if any
    pub fn tool_calls(&self) -> &[ToolCall] {
        match self {
            Self::Text(_) => &[],
            Self::Mixed { tool_calls, .. } => tool_calls,
        }
    }
}

impl From<String> for MessageContent {
    fn from(text: String) -> Self {
        Self::Text(text)
    }
}

impl From<&str> for MessageContent {
    fn from(text: &str) -> Self {
        Self::Text(text.to_string())
    }
}

impl Message {
    /// Create a new human message with text content
    pub fn human<S: Into<String>>(content: S) -> Self {
        Self::Human(HumanMessage {
            content: MessageContent::text(content),
        })
    }

    /// Create a new assistant message with text content
    pub fn assistant<S: Into<String>>(content: S) -> Self {
        Self::Assistant(AssistantMessage {
            content: MessageContent::text(content),
        })
    }

    /// Create a new system message
    pub fn system<S: Into<String>>(content: S) -> Self {
        Self::System(SystemMessage {
            content: content.into(),
        })
    }

    /// Get the role of this message as a string
    pub fn role(&self) -> &'static str {
        match self {
            Self::Human(_) => "user",
            Self::Assistant(_) => "assistant",
            Self::System(_) => "system",
        }
    }

    /// Get the text content of this message
    pub fn content_text(&self) -> String {
        match self {
            Self::Human(msg) => msg.content.to_text(),
            Self::Assistant(msg) => msg.content.to_text(),
            Self::System(msg) => msg.content.clone(),
        }
    }
}

impl HumanMessage {
    /// Create a new human message with text content
    pub fn new<S: Into<String>>(content: S) -> Self {
        Self {
            content: MessageContent::text(content),
        }
    }

    /// Create a new human message with mixed content
    pub fn with_tool_calls<S: Into<String>>(text: Option<S>, tool_calls: Vec<ToolCall>) -> Self {
        Self {
            content: MessageContent::mixed(text, tool_calls),
        }
    }
}

impl AssistantMessage {
    /// Create a new assistant message with text content
    pub fn new<S: Into<String>>(content: S) -> Self {
        Self {
            content: MessageContent::text(content),
        }
    }

    /// Create a new assistant message with mixed content
    pub fn with_tool_calls<S: Into<String>>(text: Option<S>, tool_calls: Vec<ToolCall>) -> Self {
        Self {
            content: MessageContent::mixed(text, tool_calls),
        }
    }
}

impl SystemMessage {
    /// Create a new system message
    pub fn new<S: Into<String>>(content: S) -> Self {
        Self {
            content: content.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_content_text() {
        let content = MessageContent::text("Hello world");
        assert_eq!(content.as_text(), Some("Hello world"));
        assert_eq!(content.to_text(), "Hello world");
        assert!(!content.has_tool_calls());
        assert!(content.tool_calls().is_empty());
    }

    #[test]
    fn test_message_content_mixed() {
        let tool_call = ToolCall {
            id: "call_1".to_string(),
            call_id: Some("call_1".to_string()),
            function: ToolFunction {
                name: "test_function".to_string(),
                arguments: serde_json::json!({"param": "value"}),
            },
        };

        let content = MessageContent::mixed(Some("Hello"), vec![tool_call.clone()]);
        assert_eq!(content.as_text(), Some("Hello"));
        assert_eq!(content.to_text(), "Hello");
        assert!(content.has_tool_calls());
        assert_eq!(content.tool_calls().len(), 1);
        assert_eq!(content.tool_calls()[0].id, "call_1");
    }

    #[test]
    fn test_message_content_from_string() {
        let content: MessageContent = "Test message".into();
        assert_eq!(content.as_text(), Some("Test message"));
    }

    #[test]
    fn test_message_constructors() {
        let human_msg = Message::human("Hello");
        assert_eq!(human_msg.role(), "user");
        assert_eq!(human_msg.content_text(), "Hello");

        let assistant_msg = Message::assistant("Hi there");
        assert_eq!(assistant_msg.role(), "assistant");
        assert_eq!(assistant_msg.content_text(), "Hi there");

        let system_msg = Message::system("You are helpful");
        assert_eq!(system_msg.role(), "system");
        assert_eq!(system_msg.content_text(), "You are helpful");
    }

    #[test]
    fn test_human_message_constructors() {
        let msg = HumanMessage::new("Hello");
        assert_eq!(msg.content.to_text(), "Hello");

        let tool_call = ToolCall {
            id: "call_1".to_string(),
            call_id: None,
            function: ToolFunction {
                name: "test".to_string(),
                arguments: serde_json::json!({}),
            },
        };

        let msg_with_tools = HumanMessage::with_tool_calls(Some("Text"), vec![tool_call]);
        assert_eq!(msg_with_tools.content.to_text(), "Text");
        assert!(msg_with_tools.content.has_tool_calls());
    }

    #[test]
    fn test_assistant_message_constructors() {
        let msg = AssistantMessage::new("Response");
        assert_eq!(msg.content.to_text(), "Response");

        let tool_call = ToolCall {
            id: "call_1".to_string(),
            call_id: None,
            function: ToolFunction {
                name: "test".to_string(),
                arguments: serde_json::json!({}),
            },
        };

        let msg_with_tools = AssistantMessage::with_tool_calls(Some("Response"), vec![tool_call]);
        assert_eq!(msg_with_tools.content.to_text(), "Response");
        assert!(msg_with_tools.content.has_tool_calls());
    }

    #[test]
    fn test_message_serialization() {
        let msg = Message::human("Test message");
        let serialized = serde_json::to_string(&msg).unwrap();
        let deserialized: Message = serde_json::from_str(&serialized).unwrap();

        match deserialized {
            Message::Human(human_msg) => {
                assert_eq!(human_msg.content.to_text(), "Test message");
            }
            _ => panic!("Expected human message"),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub enum HumanContent {
    Text(Text),
    ToolCall(ToolCall),
}

/// Basic text content.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct Text {
    pub text: String,
}

/// Describes a tool call with an id and function to call.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct ToolCall {
    pub id: String,
    pub call_id: Option<String>,
    pub function: ToolFunction,
}

/// Describes a tool function to call with a name and arguments.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct ToolFunction {
    pub name: String,
    pub arguments: serde_json::Value,
}
