use thiserror::Error;

/// Completion errors
#[derive(Debug, Error)]
pub enum CompletionError {
    /// Http error (e.g.: connection error, timeout, etc.)
    #[error("HttpError: {0}")]
    HttpError(#[from] reqwest::Error),

    /// Json error (e.g.: serialization, deserialization)
    #[error("JsonError: {0}")]
    JsonError(#[from] serde_json::Error),

    /// Url error (e.g.: invalid URL)
    #[error("UrlError: {0}")]
    UrlError(#[from] url::ParseError),

    /// Error building the completion request
    /// This will allows to catch all errors that can hold any type implementing std::error::Error, Send(Can be sent between threads), Sync (Can be shared between threads), and 'static (Live as long as the program).
    #[error("RequestError: {0}")]
    RequestError(#[from] Box<dyn std::error::Error + Send + Sync + 'static>),

    /// Error parsing the completion response
    #[error("ResponseError: {0}")]
    ResponseError(String),

    /// Error returned by the completion model provider
    #[error("ProviderError: {0}")]
    ProviderError(String),
}

/// Prompt errors
#[derive(Debug, Error)]
pub enum PromptError {
    /// Something went wrong with the completion
    #[error("CompletionError: {0}")]
    CompletionError(#[from] CompletionError),
}
