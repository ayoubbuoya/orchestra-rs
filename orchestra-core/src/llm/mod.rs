//! # LLM Interface
//!
//! This module provides the high-level [`LLM`] interface for interacting with Large Language Models.
//!
//! The [`LLM`] struct abstracts over different provider implementations and provides a unified
//! interface for sending prompts, managing conversations, and configuring model behavior.
//!
//! ## Examples
//!
//! ### Basic Usage
//!
//! ```rust,no_run
//! use orchestra_core::llm::LLM;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let llm = LLM::gemini("gemini-2.5-flash");
//!     let response = llm.prompt("Hello, world!").await?;
//!     println!("Response: {}", response.text);
//!     Ok(())
//! }
//! ```
//!
//! ### Chat with History
//!
//! ```rust,no_run
//! use orchestra_core::{llm::LLM, messages::Message};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let llm = LLM::gemini("gemini-2.5-flash");
//!
//!     let history = vec![
//!         Message::human("What is Rust?"),
//!         Message::assistant("Rust is a systems programming language..."),
//!     ];
//!
//!     let response = llm.chat(Message::human("Tell me more"), history).await?;
//!     println!("Response: {}", response.text);
//!     Ok(())
//! }
//! ```

use crate::{
    error::Result,
    messages::Message,
    model::ModelConfig,
    providers::{Provider, gemini::GeminiProvider, types::{ProviderSource, ChatResponse}},
};

/// Enum to hold different provider implementations
#[derive(Debug)]
pub enum ProviderInstance {
    /// Google Gemini provider instance
    Gemini(GeminiProvider),
}

/// High-level interface for interacting with Large Language Models.
///
/// The [`LLM`] struct provides a unified interface for working with different LLM providers.
/// It handles provider-specific details and provides a consistent API for sending prompts,
/// managing conversations, and configuring model behavior.
///
/// ## Examples
///
/// ```rust,no_run
/// use orchestra_core::llm::LLM;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     // Create an LLM instance with default settings
///     let llm = LLM::gemini("gemini-2.5-flash");
///
///     // Send a simple prompt
///     let response = llm.prompt("What is Rust?").await?;
///     println!("Response: {}", response.text);
///
///     Ok(())
/// }
/// ```
#[derive(Debug)]
pub struct LLM {
    /// The provider source (e.g., Gemini, OpenAI)
    pub provider_source: ProviderSource,
    /// The name of the model to use
    pub model_name: String,
    /// The provider instance
    pub provider: ProviderInstance,
    /// Model configuration settings
    pub config: ModelConfig,
}

impl LLM {
    /// Create a new LLM instance with default configuration
    pub fn new(provider_source: ProviderSource, model_name: String) -> Self {
        let default_model_config = ModelConfig::new(&model_name);

        let provider = match provider_source {
            ProviderSource::Gemini => ProviderInstance::Gemini(GeminiProvider::with_default_config()),
            _ => panic!("Unsupported provider source"),
        };

        LLM {
            provider_source,
            model_name,
            provider,
            config: default_model_config,
        }
    }

    /// Create a new LLM instance with Gemini provider
    pub fn gemini<S: Into<String>>(model_name: S) -> Self {
        Self::new(ProviderSource::Gemini, model_name.into())
    }

    /// Create a new LLM instance with conservative settings
    pub fn conservative(provider_source: ProviderSource, model_name: String) -> Self {
        let config = ModelConfig::conservative(&model_name);
        Self::new(provider_source, model_name).with_custom_config(config)
    }

    /// Create a new LLM instance with creative settings
    pub fn creative(provider_source: ProviderSource, model_name: String) -> Self {
        let config = ModelConfig::creative(&model_name);
        Self::new(provider_source, model_name).with_custom_config(config)
    }

    /// Create a new LLM instance with balanced settings
    pub fn balanced(provider_source: ProviderSource, model_name: String) -> Self {
        let config = ModelConfig::balanced(&model_name);
        Self::new(provider_source, model_name).with_custom_config(config)
    }

    pub fn with_custom_config(mut self, config: ModelConfig) -> Self {
        self.config = config;
        self
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

    /// Send a prompt to the LLM and get a response.
    ///
    /// This is the simplest way to interact with an LLM. It sends a single prompt
    /// and returns the response.
    ///
    /// # Arguments
    ///
    /// * `prompt` - The text prompt to send to the LLM
    ///
    /// # Returns
    ///
    /// A [`ChatResponse`] containing the LLM's response text.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use orchestra_core::llm::LLM;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let llm = LLM::gemini("gemini-2.5-flash");
    ///     let response = llm.prompt("What is the capital of France?").await?;
    ///     println!("Answer: {}", response.text);
    ///     Ok(())
    /// }
    /// ```
    pub async fn prompt<S: Into<String>>(&self, prompt: S) -> Result<ChatResponse> {
        let mut config = self.config.clone();
        config.name = self.model_name.clone();

        match &self.provider {
            ProviderInstance::Gemini(provider) => provider.prompt(config, prompt.into()).await,
        }
    }

    /// Send a chat message with conversation history.
    ///
    /// This method allows you to maintain conversation context by providing
    /// previous messages in the conversation.
    ///
    /// # Arguments
    ///
    /// * `message` - The new message to send
    /// * `history` - Previous messages in the conversation
    ///
    /// # Returns
    ///
    /// A [`ChatResponse`] containing the LLM's response text.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use orchestra_core::{llm::LLM, messages::Message};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let llm = LLM::gemini("gemini-2.5-flash");
    ///
    ///     let history = vec![
    ///         Message::human("What is Rust?"),
    ///         Message::assistant("Rust is a systems programming language..."),
    ///     ];
    ///
    ///     let response = llm.chat(
    ///         Message::human("What are its main benefits?"),
    ///         history
    ///     ).await?;
    ///
    ///     println!("Response: {}", response.text);
    ///     Ok(())
    /// }
    /// ```
    pub async fn chat(&self, message: Message, history: Vec<Message>) -> Result<ChatResponse> {
        let mut config = self.config.clone();
        config.name = self.model_name.clone();

        match &self.provider {
            ProviderInstance::Gemini(provider) => provider.chat(config, message, history).await,
        }
    }

    /// Get the provider name
    pub fn provider_name(&self) -> &'static str {
        match &self.provider {
            ProviderInstance::Gemini(_) => "gemini",
        }
    }

    /// Check if the provider supports streaming
    pub fn supports_streaming(&self) -> bool {
        match &self.provider {
            ProviderInstance::Gemini(provider) => provider.supports_streaming(),
        }
    }

    /// Check if the provider supports tools
    pub fn supports_tools(&self) -> bool {
        match &self.provider {
            ProviderInstance::Gemini(provider) => provider.supports_tools(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::providers::gemini;

    use super::*;

    #[tokio::test]
    async fn test_llm_creation() {
        let _llm = LLM::new(
            ProviderSource::Gemini,
            gemini::PREDEFINED_MODELS[0].to_string(),
        );

        // Test that we can create different LLM configurations
        let _conservative = LLM::conservative(
            ProviderSource::Gemini,
            gemini::PREDEFINED_MODELS[0].to_string(),
        );

        let _creative = LLM::creative(
            ProviderSource::Gemini,
            gemini::PREDEFINED_MODELS[0].to_string(),
        );

        let _balanced = LLM::balanced(
            ProviderSource::Gemini,
            gemini::PREDEFINED_MODELS[0].to_string(),
        );
    }
}
