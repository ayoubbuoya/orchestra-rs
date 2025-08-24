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

#[derive(Debug, Clone)]
pub struct ChatResponse {
    pub text: String,
}
