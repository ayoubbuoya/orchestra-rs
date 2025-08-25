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

    /// Whether the provider supports tool calling (tool-enabled responses).
    ///
    /// Defaults to `false`. Providers that implement tool-calling should override
    /// this method to return `true`.
    ///
    /// # Examples
    ///
    /// ```
    /// struct MyProvider;
    /// impl Provider for MyProvider {
    ///     type Config = ();
    ///     fn new(_: Self::Config) -> Self { Self }
    ///     fn get_base_url(&self) -> &str { "https://example" }
    ///     fn get_predefined_models(&self) -> crate::error::Result<Vec<String>> { Ok(vec![]) }
    ///     fn name(&self) -> &'static str { "my_provider" }
    ///     fn supports_tools(&self) -> bool { true }
    ///     // other required methods omitted for brevity...
    /// }
    ///
    /// let p = MyProvider::new(());
    /// assert!(p.supports_tools());
    /// ```
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

    /// Returns whether the provider supports streaming responses.
    ///
    /// Defaults to `false`. Providers that can deliver partial/streamed responses
    /// should override this method and return `true`.
    ///
    /// # Examples
    ///
    /// ```
    /// struct Dummy;
    /// impl orchestra_core::providers::ProviderExt for Dummy {}
    ///
    /// let d = Dummy;
    /// assert!(!d.supports_streaming());
    /// ```
    fn supports_streaming(&self) -> bool {
        false
    }

    /// Whether the provider supports tool calling (tool-enabled responses).
    ///
    /// Defaults to `false`. Providers that implement tool-calling should override
    /// this method to return `true`.
    ///
    /// # Examples
    ///
    /// ```
    /// struct MyProvider;
    /// impl Provider for MyProvider {
    ///     type Config = ();
    ///     fn new(_: Self::Config) -> Self { Self }
    ///     fn get_base_url(&self) -> &str { "https://example" }
    ///     fn get_predefined_models(&self) -> crate::error::Result<Vec<String>> { Ok(vec![]) }
    ///     fn name(&self) -> &'static str { "my_provider" }
    ///     fn supports_tools(&self) -> bool { true }
    ///     // other required methods omitted for brevity...
    /// }
    ///
    /// let p = MyProvider::new(());
    /// assert!(p.supports_tools());
    /// ```
    fn supports_tools(&self) -> bool {
        false
    }
}

// Short note:
// We intentionally repeat methods in `Provider` and `ProviderExt`.
// - `Provider` is the trait implementors use. It has a type for config and
//   lets implementors write `async fn` easily.
// - `ProviderExt` is a small, object-safe trait used at runtime. `LLM` stores
//   providers as `Box<dyn ProviderExt>`, so the trait must be object-safe.
// The `impl<T> ProviderExt for T where T: Provider` below forwards calls from
// the object-safe API to the implementor API. This keeps implementations easy
// to write while letting `LLM` call providers without matching on an enum.

// Blanket implementation: any concrete type that implements the original
// `Provider` also implements `ProviderExt` by delegating calls. This lets us
// store different providers behind `Box<dyn ProviderExt>` and call methods
// polymorphically without matching on an enum.
#[async_trait]
impl<T> ProviderExt for T
where
    T: Provider + Send + Sync + std::fmt::Debug + 'static,
{
    /// Sends a chat request through the provider's implementation and returns the provider's chat response.
    ///
    /// Delegates to the concrete provider's `Provider::chat` implementation.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// // Inside an async context:
    /// // let response = provider.chat(model_config, user_message, chat_history).await?;
    /// ```
    async fn chat(
        &self,
        model_config: ModelConfig,
        message: Message,
        chat_history: Vec<Message>,
    ) -> Result<ChatResponse> {
        Provider::chat(self, model_config, message, chat_history).await
    }

    /// Forwards a prompt request through the object-safe `ProviderExt` wrapper to the underlying `Provider`.
    ///
    /// This delegates to `Provider::prompt` and returns the provider's `ChatResponse`.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// // `provider` is any type implementing `ProviderExt` (e.g., `Box<dyn ProviderExt>`).
    /// // let resp = provider.prompt(model_config, "Hello".to_string()).await?;
    /// ```
    async fn prompt(&self, model_config: ModelConfig, prompt: String) -> Result<ChatResponse> {
        Provider::prompt(self, model_config, prompt).await
    }

    /// Returns the provider's base URL used for requests.
    ///
    /// This method delegates to the underlying provider's `get_base_url` implementation.
    ///
    /// # Examples
    ///
    /// ```
    /// // `p` is any type that implements `Provider`/`ProviderExt`
    /// let base = p.get_base_url();
    /// assert!(!base.is_empty());
    /// ```
    fn get_base_url(&self) -> &str {
        Provider::get_base_url(self)
    }

    /// Returns the list of predefined model identifiers available from this provider.
    ///
    /// The returned `Vec<String>` contains provider-specific model names (e.g. `"gpt-4"`).
    /// Propagates any error produced while retrieving the list.
    ///
    /// # Returns
    ///
    /// `Result<Vec<String>>` — `Ok` with the model identifiers on success, or an error on failure.
    fn get_predefined_models(&self) -> Result<Vec<String>> {
        Provider::get_predefined_models(self)
    }

    /// Returns the provider's static name.
    ///
    /// This delegates to the underlying `Provider::name` implementation and yields
    /// a `'static` string slice identifying the provider.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// // Given a provider instance `p` implementing `Provider`/`ProviderExt`:
    /// // let p = ...;
    /// // let provider_name = p.name();
    /// ```
    fn name(&self) -> &'static str {
        Provider::name(self)
    }

    /// Returns whether this provider supports streaming responses.
    ///
    /// This method delegates to `Provider::supports_streaming` and preserves the provider's
    /// default (which is `false`).
    fn supports_streaming(&self) -> bool {
        Provider::supports_streaming(self)
    }

    /// Returns whether this provider supports tool calling.
    ///
    /// By default providers do not support tool calling; concrete provider
    /// implementations may override that behavior.
    ///
    /// # Examples
    ///
    /// ```
    /// let supports = provider.supports_tools();
    /// assert!(matches!(supports, bool));
    /// ```
    fn supports_tools(&self) -> bool {
        Provider::supports_tools(self)
    }
}
