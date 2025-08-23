mod gemini;
pub mod types;

use anyhow::{Error, Result};

use crate::{messages::Message, model::ModelConfig, providers::types::ChatResponse};

/// A trait for all providers to implement.
pub trait Provider {
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
    ) -> impl std::future::Future<Output = Result<ChatResponse, Error>> + Send;
}
