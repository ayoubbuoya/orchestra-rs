use crate::providers::types::ProviderSource;

#[derive(Debug, Clone)]
pub struct ModelConfig {
    pub name: String,
    pub system_instruction: Option<String>,
    pub temperature: f32,
    pub top_p: f32,
    pub thinking_mode: Option<bool>,
}
