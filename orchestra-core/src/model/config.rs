use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    pub name: String,
    pub system_instruction: Option<String>,
    pub temperature: f32,
    pub top_p: f32,
    pub thinking_mode: Option<bool>,
}

impl Default for ModelConfig {
    fn default() -> Self {
        ModelConfig {
            name: "".to_string(),
            system_instruction: None,
            temperature: 1.0,
            top_p: 0.95,
            thinking_mode: None,
        }
    }
}
