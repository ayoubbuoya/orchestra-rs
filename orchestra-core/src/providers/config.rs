use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Configuration for a provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    /// API key for authentication
    pub api_key: Option<String>,
    /// Base URL for the provider's API
    pub base_url: Option<String>,
    /// Additional headers to include in requests
    pub headers: HashMap<String, String>,
    /// Request timeout in seconds
    pub timeout_seconds: Option<u64>,
    /// Maximum number of retries for failed requests
    pub max_retries: Option<u32>,
    /// Custom configuration specific to the provider
    pub custom: HashMap<String, serde_json::Value>,
}

impl Default for ProviderConfig {
    fn default() -> Self {
        Self {
            api_key: None,
            base_url: None,
            headers: HashMap::new(),
            timeout_seconds: Some(30),
            max_retries: Some(3),
            custom: HashMap::new(),
        }
    }
}

impl ProviderConfig {
    /// Create a new provider configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the API key
    pub fn with_api_key<S: Into<String>>(mut self, api_key: S) -> Self {
        self.api_key = Some(api_key.into());
        self
    }

    /// Set the base URL
    pub fn with_base_url<S: Into<String>>(mut self, base_url: S) -> Self {
        self.base_url = Some(base_url.into());
        self
    }

    /// Add a header
    pub fn with_header<K: Into<String>, V: Into<String>>(mut self, key: K, value: V) -> Self {
        self.headers.insert(key.into(), value.into());
        self
    }

    /// Set the timeout
    pub fn with_timeout(mut self, timeout_seconds: u64) -> Self {
        self.timeout_seconds = Some(timeout_seconds);
        self
    }

    /// Set the maximum number of retries
    pub fn with_max_retries(mut self, max_retries: u32) -> Self {
        self.max_retries = Some(max_retries);
        self
    }

    /// Add custom configuration
    pub fn with_custom<K: Into<String>, V: Into<serde_json::Value>>(mut self, key: K, value: V) -> Self {
        self.custom.insert(key.into(), value.into());
        self
    }

    /// Get the API key, trying environment variable if not set
    pub fn get_api_key(&self, env_var: &str) -> Option<String> {
        self.api_key.clone().or_else(|| std::env::var(env_var).ok())
    }

    /// Get the base URL
    pub fn get_base_url(&self, default: &str) -> String {
        self.base_url.clone().unwrap_or_else(|| default.to_string())
    }

    /// Get the timeout in seconds
    pub fn get_timeout(&self) -> u64 {
        self.timeout_seconds.unwrap_or(30)
    }

    /// Get the maximum number of retries
    pub fn get_max_retries(&self) -> u32 {
        self.max_retries.unwrap_or(3)
    }

    /// Get a custom configuration value
    pub fn get_custom(&self, key: &str) -> Option<&serde_json::Value> {
        self.custom.get(key)
    }
}

/// Configuration specific to Gemini provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeminiConfig {
    /// Base provider configuration
    pub base: ProviderConfig,
    /// Whether to use the beta API
    pub use_beta: bool,
    /// API version to use
    pub api_version: String,
}

impl Default for GeminiConfig {
    fn default() -> Self {
        Self {
            base: ProviderConfig::default(),
            use_beta: true,
            api_version: "v1beta".to_string(),
        }
    }
}

impl GeminiConfig {
    /// Create a new Gemini configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the API key
    pub fn with_api_key<S: Into<String>>(mut self, api_key: S) -> Self {
        self.base = self.base.with_api_key(api_key);
        self
    }

    /// Set whether to use the beta API
    pub fn with_beta(mut self, use_beta: bool) -> Self {
        self.use_beta = use_beta;
        self
    }

    /// Set the API version
    pub fn with_api_version<S: Into<String>>(mut self, api_version: S) -> Self {
        self.api_version = api_version.into();
        self
    }

    /// Get the base URL for Gemini API
    pub fn get_base_url(&self) -> String {
        if self.use_beta {
            format!("https://generativelanguage.googleapis.com/{}", self.api_version)
        } else {
            "https://generativelanguage.googleapis.com/v1".to_string()
        }
    }

    /// Get the API key from configuration or environment
    pub fn get_api_key(&self) -> Option<String> {
        self.base.get_api_key("GEMINI_API_KEY")
    }
}
