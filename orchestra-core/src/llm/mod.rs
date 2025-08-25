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
    providers::{
        ProviderExt,
        gemini::GeminiProvider,
        types::{ChatResponse, ProviderSource},
    },
};

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
    // The provider instance stored as a trait object.
    ///
    /// Explanation:
    /// - `dyn ProviderExt` is a trait object: it erases the concrete provider type
    ///   (e.g., `GeminiProvider`) so this field can hold any provider implementation.
    /// - `Box<dyn ProviderExt>` stores the trait object on the heap. `Box` is a
    ///   smart pointer that keeps a fixed-size pointer in the struct while the
    ///   actual provider value lives on the heap. This is necessary because
    ///   different providers can have different sizes, and struct fields must
    ///   have a known size at compile time.
    /// - Using `dyn` enables dynamic dispatch: method calls (like `prompt`)
    ///   go through a vtable so the correct implementation for the concrete
    ///   provider runs at runtime.
    ///
    /// Trade-offs:
    /// - Pros: simple runtime polymorphism, one `LLM` type can hold any provider,
    ///   and you avoid repeating `match` on provider variants.
    /// - Cons: one level of indirection (heap allocation) and vtable calls (small
    ///   runtime cost). If you need zero-cost static dispatch, consider making
    ///   `LLM` generic over the provider type (`LLM<P: ProviderExt>`).
    pub provider: Box<dyn ProviderExt>,
    /// Model configuration settings
    pub config: ModelConfig,
}

impl LLM {
    /// Creates a new LLM backed by the specified provider with a default ModelConfig.
    ///
    /// The returned LLM uses a provider implementation chosen from `provider_source` and
    /// initializes `config` using `ModelConfig::new(&model_name)`.
    ///
    /// Panics if `provider_source` is not supported. Currently only `ProviderSource::Gemini` is supported.
    ///
    /// # Examples
    ///
    /// ```
    /// let llm = orchestra_core::llm::LLM::new(
    ///     orchestra_core::llm::ProviderSource::Gemini,
    ///     "gemini-small".to_string(),
    /// );
    /// assert_eq!(llm.get_model_name(), "gemini-small");
    /// ```
    pub fn new(provider_source: ProviderSource, model_name: String) -> Self {
        let default_model_config = ModelConfig::new(&model_name);

        let provider: Box<dyn ProviderExt> = match provider_source {
            ProviderSource::Gemini => Box::new(GeminiProvider::with_default_config()),
            _ => panic!(
                "Unsupported provider source: {:?}. Supported providers: Gemini",
                provider_source
            ),
        };

        LLM {
            provider_source,
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
        &self.config.name
    }

    /// Send a single prompt to the LLM and return the model's response.
    ///
    /// This sends `prompt` using the LLM's current configuration and delegates to the
    /// underlying provider implementation. Errors from the provider are propagated.
    ///
    /// # Returns
    ///
    /// A `Result` containing a `ChatResponse` on success.
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
        let config = self.config.clone();
        self.provider.prompt(config, prompt.into()).await
    }

    /// Send a chat message with conversation history and return the model's response.
    ///
    /// The provided `history` is used to establish conversational context for `message`.
    /// History should be the prior messages in chronological order (oldest first).
    ///
    /// # Returns
    ///
    /// A `Result` containing a [`ChatResponse`] with the model's reply on success.
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
    ///     let response = llm
    ///         .chat(Message::human("What are its main benefits?"), history)
    ///         .await?;
    ///
    ///     println!("Response: {}", response.text);
    ///     Ok(())
    /// }
    /// ```
    pub async fn chat(&self, message: Message, history: Vec<Message>) -> Result<ChatResponse> {
        let config = self.config.clone();
        self.provider.chat(config, message, history).await
    }

    /// Returns the provider's static name.
    ///
    /// # Examples
    ///
    /// ```
    /// let llm = LLM::gemini("example-model");
    /// let name = llm.provider_name();
    /// assert!(!name.is_empty());
    /// ```
    pub fn provider_name(&self) -> &'static str {
        self.provider.name()
    }

    /// Returns true if the underlying provider supports streaming.
    ///
    /// Delegates the capability check to the configured provider implementation.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// // Assuming `llm` is an initialized `LLM`:
    /// if llm.supports_streaming() {
    ///     // use streaming-specific code path
    /// }
    /// ```
    pub fn supports_streaming(&self) -> bool {
        self.provider.supports_streaming()
    }

    /// Returns true if the underlying provider supports executing or integrating external tools.
    ///
    /// This delegates to the provider implementation's `supports_tools` capability flag.
    ///
    —
    /// # Examples
    ///
    /// ```
    /// let llm = LLM::gemini("example-model");
    /// if llm.supports_tools() {
    ///     // safe to request tool-enabled behaviors
    /// }
    /// ```
    pub fn supports_tools(&self) -> bool {
        self.provider.supports_tools()
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
