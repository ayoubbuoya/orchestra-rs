pub mod gemini;
pub mod types;

use anyhow::{Error, Result};

use crate::{messages::Message, model::ModelConfig, providers::types::ChatResponse};

/// A trait for all providers to implement.
pub trait Provider: Sized {
    fn new() -> Self;

    /// Gets base url used for all requests.
    fn get_base_url(&self) -> &str;

    /// Get a list of all predefined models for this provider.
    fn get_predefined_models(&self) -> Result<Vec<String>, Error>;

    /// Sends a chat request to the provider.
    /// It is an async function that returns a future.
    fn chat(
        &self,
        model_config: ModelConfig,
        message: Message,
        chat_history: Vec<Message>,
    ) -> impl std::future::Future<Output = Result<ChatResponse, Error>> + Send;

    /// Sends a prompt request to the provider.
    /// INternally this just calls the chat function with a single message.
    /// It is an async function that returns a future.
    fn prompt(
        &self,
        model_config: ModelConfig,
        prompt: String,
    ) -> impl std::future::Future<Output = Result<ChatResponse, Error>> + Send;
}
