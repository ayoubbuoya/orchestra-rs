use async_trait::async_trait;
use crate::{
    error::Result,
    messages::Message,
    model::ModelConfig,
    providers::{Provider, types::ChatResponse},
};

/// Mock provider for testing purposes
#[derive(Debug)]
pub struct MockProvider {
    /// Predefined responses to return
    pub responses: Vec<String>,
    /// Current response index
    pub current_index: std::sync::Arc<std::sync::Mutex<usize>>,
    /// Whether to simulate errors
    pub should_error: bool,
    /// Delay to simulate network latency (in milliseconds)
    pub delay_ms: Option<u64>,
}

/// Configuration for the mock provider
#[derive(Debug, Clone)]
pub struct MockConfig {
    pub responses: Vec<String>,
    pub should_error: bool,
    pub delay_ms: Option<u64>,
}

impl Default for MockConfig {
    fn default() -> Self {
        Self {
            responses: vec!["Mock response".to_string()],
            should_error: false,
            delay_ms: None,
        }
    }
}

impl MockConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_responses<I, S>(mut self, responses: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.responses = responses.into_iter().map(|s| s.into()).collect();
        self
    }

    pub fn with_error(mut self, should_error: bool) -> Self {
        self.should_error = should_error;
        self
    }

    pub fn with_delay(mut self, delay_ms: u64) -> Self {
        self.delay_ms = Some(delay_ms);
        self
    }
}

impl MockProvider {
    pub fn new(config: MockConfig) -> Self {
        Self {
            responses: config.responses,
            current_index: std::sync::Arc::new(std::sync::Mutex::new(0)),
            should_error: config.should_error,
            delay_ms: config.delay_ms,
        }
    }

    /// Get the next response from the predefined list
    fn get_next_response(&self) -> String {
        let mut index = self.current_index.lock().unwrap();
        let response = self.responses.get(*index)
            .unwrap_or(&"Default mock response".to_string())
            .clone();
        *index = (*index + 1) % self.responses.len();
        response
    }

    /// Reset the response index
    pub fn reset(&self) {
        let mut index = self.current_index.lock().unwrap();
        *index = 0;
    }
}

#[async_trait]
impl Provider for MockProvider {
    type Config = MockConfig;

    fn new(config: Self::Config) -> Self {
        Self::new(config)
    }

    fn get_base_url(&self) -> &str {
        "https://mock.example.com"
    }

    fn get_predefined_models(&self) -> Result<Vec<String>> {
        Ok(vec![
            "mock-model-1".to_string(),
            "mock-model-2".to_string(),
            "mock-model-large".to_string(),
        ])
    }

    async fn chat(
        &self,
        _model_config: ModelConfig,
        _message: Message,
        _chat_history: Vec<Message>,
    ) -> Result<ChatResponse> {
        if self.should_error {
            return Err(crate::error::OrchestraError::provider(
                "mock",
                "Simulated error",
            ));
        }

        // Simulate network delay if configured
        if let Some(delay) = self.delay_ms {
            tokio::time::sleep(tokio::time::Duration::from_millis(delay)).await;
        }

        Ok(ChatResponse {
            text: self.get_next_response(),
        })
    }

    async fn prompt(
        &self,
        model_config: ModelConfig,
        prompt: String,
    ) -> Result<ChatResponse> {
        self.chat(model_config, Message::human(prompt), vec![]).await
    }

    fn name(&self) -> &'static str {
        "mock"
    }

    fn supports_streaming(&self) -> bool {
        true // Mock provider can simulate streaming
    }

    fn supports_tools(&self) -> bool {
        true // Mock provider can simulate tool support
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_provider_basic() {
        let config = MockConfig::new()
            .with_responses(vec!["Hello", "World", "Test"]);
        let provider = MockProvider::new(config);

        let model_config = ModelConfig::new("mock-model-1");

        // Test multiple responses cycling
        let response1 = provider.prompt(model_config.clone(), "test".to_string()).await.unwrap();
        assert_eq!(response1.text, "Hello");

        let response2 = provider.prompt(model_config.clone(), "test".to_string()).await.unwrap();
        assert_eq!(response2.text, "World");

        let response3 = provider.prompt(model_config.clone(), "test".to_string()).await.unwrap();
        assert_eq!(response3.text, "Test");

        // Should cycle back to the first response
        let response4 = provider.prompt(model_config, "test".to_string()).await.unwrap();
        assert_eq!(response4.text, "Hello");
    }

    #[tokio::test]
    async fn test_mock_provider_error() {
        let config = MockConfig::new().with_error(true);
        let provider = MockProvider::new(config);

        let model_config = ModelConfig::new("mock-model-1");
        let result = provider.prompt(model_config, "test".to_string()).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_mock_provider_delay() {
        let config = MockConfig::new().with_delay(100); // 100ms delay
        let provider = MockProvider::new(config);

        let model_config = ModelConfig::new("mock-model-1");
        
        let start = std::time::Instant::now();
        let _response = provider.prompt(model_config, "test".to_string()).await.unwrap();
        let duration = start.elapsed();

        // Should take at least 100ms due to the delay
        assert!(duration.as_millis() >= 100);
    }

    #[tokio::test]
    async fn test_mock_provider_reset() {
        let config = MockConfig::new()
            .with_responses(vec!["First", "Second"]);
        let provider = MockProvider::new(config);

        let model_config = ModelConfig::new("mock-model-1");

        let response1 = provider.prompt(model_config.clone(), "test".to_string()).await.unwrap();
        assert_eq!(response1.text, "First");

        let response2 = provider.prompt(model_config.clone(), "test".to_string()).await.unwrap();
        assert_eq!(response2.text, "Second");

        // Reset and test again
        provider.reset();
        let response3 = provider.prompt(model_config, "test".to_string()).await.unwrap();
        assert_eq!(response3.text, "First");
    }
}
