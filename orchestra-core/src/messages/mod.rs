use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub enum Message {
    Human(HumanMessage),
    Assistant(AssistantMessage),
    System(SystemMessage),
}

#[derive(Debug, Clone)]
pub struct HumanMessage {
    pub content: String,
}

#[derive(Debug, Clone)]
pub struct AssistantMessage {
    pub content: String,
}

#[derive(Debug, Clone)]
pub struct SystemMessage {
    pub content: String,
}

#[derive(Debug, Clone)]
pub struct MessageChatEntry {
    pub role: String,
    pub content: String,
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
