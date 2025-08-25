pub mod config;
pub mod gemini;
#[cfg(test)]
pub mod mock;
pub mod types;

use async_trait::async_trait;

use crate::{
    error::Result,
    messages::Message,
    model::ModelConfig,
    providers::types::ChatResponse
};

/// A trait for all providers to implement.
#[async_trait]
pub trait Provider: Send + Sync + std::fmt::Debug {
    /// The configuration type for this provider
    type Config: Send + Sync + std::fmt::Debug;

    /// Create a new provider instance with the given configuration
    fn new(config: Self::Config) -> Self;

    /// Gets base url used for all requests.
    fn get_base_url(&self) -> &str;

    /// Get a list of all predefined models for this provider.
    fn get_predefined_models(&self) -> Result<Vec<String>>;

    /// Sends a chat request to the provider.
    async fn chat(
        &self,
        model_config: ModelConfig,
        message: Message,
        chat_history: Vec<Message>,
    ) -> Result<ChatResponse>;

    /// Sends a prompt request to the provider.
    /// Internally this just calls the chat function with a single message.
    async fn prompt(
        &self,
        model_config: ModelConfig,
        prompt: String,
    ) -> Result<ChatResponse>;

    /// Get the provider's name
    fn name(&self) -> &'static str;

    /// Check if the provider supports streaming responses
    fn supports_streaming(&self) -> bool {
        false
    }

    /// Check if the provider supports tool calling
    fn supports_tools(&self) -> bool {
        false
    }
}

/// Object-safe wrapper trait so providers can be stored behind a trait object.
///
/// The main `Provider` trait has an associated type (`Config`) which prevents
/// it from being used as a trait object directly in some places. `ProviderExt`
/// exposes the runtime behaviour we need (chat/prompt/etc.) and is object-safe.
#[async_trait]
pub trait ProviderExt: Send + Sync + std::fmt::Debug {
    async fn chat(
        &self,
        model_config: ModelConfig,
        message: Message,
        chat_history: Vec<Message>,
    ) -> Result<ChatResponse>;

    async fn prompt(&self, model_config: ModelConfig, prompt: String) -> Result<ChatResponse>;

    fn get_base_url(&self) -> &str;

    fn get_predefined_models(&self) -> Result<Vec<String>>;

    fn name(&self) -> &'static str;

    fn supports_streaming(&self) -> bool {
        false
    }

    fn supports_tools(&self) -> bool {
        false
    }
}

// Blanket implementation: any concrete type that implements the original
// `Provider` also implements `ProviderExt` by delegating calls. This lets us
// store different providers behind `Box<dyn ProviderExt>` and call methods
// polymorphically without matching on an enum.
#[async_trait]
impl<T> ProviderExt for T
where
    T: Provider + Send + Sync + std::fmt::Debug + 'static,
{
    async fn chat(
        &self,
        model_config: ModelConfig,
        message: Message,
        chat_history: Vec<Message>,
    ) -> Result<ChatResponse> {
        Provider::chat(self, model_config, message, chat_history).await
    }

    async fn prompt(&self, model_config: ModelConfig, prompt: String) -> Result<ChatResponse> {
        Provider::prompt(self, model_config, prompt).await
    }

    fn get_base_url(&self) -> &str {
        Provider::get_base_url(self)
    }

    fn get_predefined_models(&self) -> Result<Vec<String>> {
        Provider::get_predefined_models(self)
    }

    fn name(&self) -> &'static str {
        Provider::name(self)
    }

    fn supports_streaming(&self) -> bool {
        Provider::supports_streaming(self)
    }

    fn supports_tools(&self) -> bool {
        Provider::supports_tools(self)
    }
}
