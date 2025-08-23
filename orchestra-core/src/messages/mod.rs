pub enum Message {
    Human(HumanMessage),
    Assistant(AsistantMessage),
    System(SystemMessage),
}

pub struct HumanMessage {
    pub content: String,
}

pub struct AsistantMessage {
    pub content: String,
}

pub struct SystemMessage {
    pub content: String,
}

pub struct MessageChatEntry {
    pub role: String,
    pub content: String,
}


