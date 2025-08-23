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
