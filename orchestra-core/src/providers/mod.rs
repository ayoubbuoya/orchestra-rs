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
    providers::types::ChatResponse,
    tools::ToolDefinition,
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

    /// Send a chat request with tool definitions
    ///
    /// This method allows sending a chat request along with tool definitions
    /// that the LLM can choose to call. The response may contain tool calls
    /// that need to be executed.
    ///
    /// # Arguments
    /// * `model_config` - Configuration for the model
    /// * `message` - The message to send
    /// * `chat_history` - Previous messages in the conversation
    /// * `tools` - Available tools that the LLM can call
    ///
    /// # Returns
    /// A `ChatResponse` that may contain tool calls to execute
    ///
    /// ## For Rust Beginners
    ///
    /// This method has a default implementation that falls back to regular chat
    /// if the provider doesn't support tools. Providers that support tools
    /// should override this method.
    async fn chat_with_tools(
        &self,
        model_config: ModelConfig,
        message: Message,
        chat_history: Vec<Message>,
        tools: Vec<ToolDefinition>,
    ) -> Result<ChatResponse> {
        // Default implementation ignores tools and falls back to regular chat
        // Providers that support tools should override this method
        if !tools.is_empty() && self.supports_tools() {
            // If tools are provided but not implemented, return an error
            return Err(crate::error::OrchestraError::provider(
                self.name(),
                "Tool calling not implemented for this provider"
            ));
        }

        // Fall back to regular chat
        self.chat(model_config, message, chat_history).await
    }

    /// Send a prompt with tool definitions
    ///
    /// This is a convenience method that wraps `chat_with_tools` for simple prompts.
    async fn prompt_with_tools(
        &self,
        model_config: ModelConfig,
        prompt: String,
        tools: Vec<ToolDefinition>,
    ) -> Result<ChatResponse> {
        self.chat_with_tools(model_config, Message::human(prompt), vec![], tools).await
    }
}
