use crate::{
    model::ModelConfig,
    providers::{Provider, gemini::GeminiProvider, types::ProviderSource},
};

#[derive(Debug, Clone)]
pub struct LLM {
    pub provider_source: ProviderSource,
    pub model_name: String, 
    pub provider: Box<dyn Provider + Send + Sync>,
    pub config: ModelConfig,
}

impl LLM {
    pub fn new(provider_source: ProviderSource, model_name: String) -> Self {
        let default_model_config = ModelConfig::default();

        let provider = match provider_source {
            ProviderSource::Gemini => Box::new(GeminiProvider::new()),
            _ => panic!("Unsupported provider source"),
        };

        LLM {
            provider_source,
            model_name,
            provider,
            config: default_model_config,
        }
    }

    pub fn with_custom_config(self, config: ModelConfig) -> Self {
        LLM { config, ..self }
    }

    pub fn temperature(&mut self, temperature: f32) -> &mut Self {
        self.config.temperature = temperature;

        self
    }

    pub fn system_instruction(&mut self, system_instruction: String) -> &mut Self {
        self.config.system_instruction = Some(system_instruction);

        self
    }

    pub fn get_config(&self) -> &ModelConfig {
        &self.config
    }

    pub fn get_provider_source(&self) -> &ProviderSource {
        &self.provider_source
    }

    pub fn get_model_name(&self) -> &str {
        &self.model_name
    }
}

#[cfg(test)]
mod tests {
    use crate::providers::gemini;

    use super::*;

    #[tokio::test]
    async fn test_llm_prompt() {
        let llm = LLM::new(
            ProviderSource::Gemini,
            gemini::PREDEFINED_MODELS[0].to_string(),
        );
    }
}
