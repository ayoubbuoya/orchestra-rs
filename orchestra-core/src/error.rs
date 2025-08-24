use thiserror::Error;

/// Main error type for the Orchestra library
#[derive(Error, Debug)]
pub enum OrchestraError {
    /// HTTP request errors
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    /// JSON serialization/deserialization errors
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Invalid header value errors
    #[error("Invalid header value: {0}")]
    InvalidHeader(#[from] reqwest::header::InvalidHeaderValue),

    /// API key not found or invalid
    #[error("API key error: {message}")]
    ApiKey { message: String },

    /// Provider-specific errors
    #[error("Provider error: {provider} - {message}")]
    Provider { provider: String, message: String },

    /// Configuration errors
    #[error("Configuration error: {message}")]
    Config { message: String },

    /// Model not found or invalid
    #[error("Model error: {message}")]
    Model { message: String },

    /// Rate limiting errors
    #[error("Rate limit exceeded: {message}")]
    RateLimit { message: String },

    /// Authentication errors
    #[error("Authentication failed: {message}")]
    Authentication { message: String },

    /// Invalid response format
    #[error("Invalid response format: {message}")]
    InvalidResponse { message: String },

    /// Network timeout
    #[error("Request timeout: {message}")]
    Timeout { message: String },

    /// Generic errors for cases not covered above
    #[error("Orchestra error: {message}")]
    Generic { message: String },
}

impl OrchestraError {
    /// Create a new API key error
    pub fn api_key<S: Into<String>>(message: S) -> Self {
        Self::ApiKey {
            message: message.into(),
        }
    }

    /// Create a new provider error
    pub fn provider<S: Into<String>>(provider: S, message: S) -> Self {
        Self::Provider {
            provider: provider.into(),
            message: message.into(),
        }
    }

    /// Create a new configuration error
    pub fn config<S: Into<String>>(message: S) -> Self {
        Self::Config {
            message: message.into(),
        }
    }

    /// Create a new model error
    pub fn model<S: Into<String>>(message: S) -> Self {
        Self::Model {
            message: message.into(),
        }
    }

    /// Create a new rate limit error
    pub fn rate_limit<S: Into<String>>(message: S) -> Self {
        Self::RateLimit {
            message: message.into(),
        }
    }

    /// Create a new authentication error
    pub fn authentication<S: Into<String>>(message: S) -> Self {
        Self::Authentication {
            message: message.into(),
        }
    }

    /// Create a new invalid response error
    pub fn invalid_response<S: Into<String>>(message: S) -> Self {
        Self::InvalidResponse {
            message: message.into(),
        }
    }

    /// Create a new timeout error
    pub fn timeout<S: Into<String>>(message: S) -> Self {
        Self::Timeout {
            message: message.into(),
        }
    }

    /// Create a new generic error
    pub fn generic<S: Into<String>>(message: S) -> Self {
        Self::Generic {
            message: message.into(),
        }
    }
}

/// Result type alias for Orchestra operations
pub type Result<T> = std::result::Result<T, OrchestraError>;
