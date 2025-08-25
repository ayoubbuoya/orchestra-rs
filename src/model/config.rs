use serde::{Deserialize, Serialize};
use crate::error::{OrchestraError, Result};

/// Configuration for a language model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    pub name: String,
    pub system_instruction: Option<String>,
    pub temperature: f32,
    pub top_p: f32,
    pub top_k: Option<u32>,
    pub max_tokens: Option<u32>,
    pub thinking_mode: Option<bool>,
    pub stop_sequences: Vec<String>,
}

impl Default for ModelConfig {
    fn default() -> Self {
        ModelConfig {
            name: String::new(),
            system_instruction: None,
            temperature: 1.0,
            top_p: 0.95,
            top_k: None,
            max_tokens: None,
            thinking_mode: None,
            stop_sequences: Vec::new(),
        }
    }
}

impl ModelConfig {
    /// Create a new model configuration with the given model name
    pub fn new<S: Into<String>>(name: S) -> Self {
        Self {
            name: name.into(),
            ..Default::default()
        }
    }

    /// Set the model name
    pub fn with_name<S: Into<String>>(mut self, name: S) -> Self {
        self.name = name.into();
        self
    }

    /// Set the system instruction
    pub fn with_system_instruction<S: Into<String>>(mut self, instruction: S) -> Self {
        self.system_instruction = Some(instruction.into());
        self
    }

    /// Set the temperature (0.0 to 2.0)
    pub fn with_temperature(mut self, temperature: f32) -> Result<Self> {
        if !(0.0..=2.0).contains(&temperature) {
            return Err(OrchestraError::config("Temperature must be between 0.0 and 2.0"));
        }
        self.temperature = temperature;
        Ok(self)
    }

    /// Set the top_p (0.0 to 1.0)
    pub fn with_top_p(mut self, top_p: f32) -> Result<Self> {
        if !(0.0..=1.0).contains(&top_p) {
            return Err(OrchestraError::config("top_p must be between 0.0 and 1.0"));
        }
        self.top_p = top_p;
        Ok(self)
    }

    /// Set the top_k
    pub fn with_top_k(mut self, top_k: u32) -> Self {
        self.top_k = Some(top_k);
        self
    }

    /// Set the maximum number of tokens to generate
    pub fn with_max_tokens(mut self, max_tokens: u32) -> Self {
        self.max_tokens = Some(max_tokens);
        self
    }

    /// Enable or disable thinking mode
    pub fn with_thinking_mode(mut self, thinking_mode: bool) -> Self {
        self.thinking_mode = Some(thinking_mode);
        self
    }

    /// Add a stop sequence
    pub fn with_stop_sequence<S: Into<String>>(mut self, stop_sequence: S) -> Self {
        self.stop_sequences.push(stop_sequence.into());
        self
    }

    /// Set multiple stop sequences
    pub fn with_stop_sequences<I, S>(mut self, stop_sequences: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.stop_sequences = stop_sequences.into_iter().map(|s| s.into()).collect();
        self
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<()> {
        if self.name.is_empty() {
            return Err(OrchestraError::config("Model name cannot be empty"));
        }

        if !(0.0..=2.0).contains(&self.temperature) {
            return Err(OrchestraError::config("Temperature must be between 0.0 and 2.0"));
        }

        if !(0.0..=1.0).contains(&self.top_p) {
            return Err(OrchestraError::config("top_p must be between 0.0 and 1.0"));
        }

        if let Some(max_tokens) = self.max_tokens {
            if max_tokens == 0 {
                return Err(OrchestraError::config("max_tokens must be greater than 0"));
            }
        }

        Ok(())
    }

    /// Create a conservative configuration (lower temperature, more focused)
    pub fn conservative<S: Into<String>>(name: S) -> Self {
        Self::new(name)
            .with_temperature(0.3)
            .unwrap()
            .with_top_p(0.8)
            .unwrap()
    }

    /// Create a creative configuration (higher temperature, more diverse)
    pub fn creative<S: Into<String>>(name: S) -> Self {
        Self::new(name)
            .with_temperature(1.2)
            .unwrap()
            .with_top_p(0.95)
            .unwrap()
    }

    /// Create a balanced configuration (moderate settings)
    pub fn balanced<S: Into<String>>(name: S) -> Self {
        Self::new(name)
            .with_temperature(0.7)
            .unwrap()
            .with_top_p(0.9)
            .unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_config_new() {
        let config = ModelConfig::new("test-model");
        assert_eq!(config.name, "test-model");
        assert_eq!(config.temperature, 1.0);
        assert_eq!(config.top_p, 0.95);
        assert!(config.system_instruction.is_none());
        assert!(config.top_k.is_none());
        assert!(config.max_tokens.is_none());
        assert!(config.thinking_mode.is_none());
        assert!(config.stop_sequences.is_empty());
    }

    #[test]
    fn test_model_config_builder() {
        let config = ModelConfig::new("test-model")
            .with_system_instruction("You are helpful")
            .with_temperature(0.5)
            .unwrap()
            .with_top_p(0.8)
            .unwrap()
            .with_top_k(40)
            .with_max_tokens(1000)
            .with_thinking_mode(true)
            .with_stop_sequence("STOP");

        assert_eq!(config.name, "test-model");
        assert_eq!(config.system_instruction, Some("You are helpful".to_string()));
        assert_eq!(config.temperature, 0.5);
        assert_eq!(config.top_p, 0.8);
        assert_eq!(config.top_k, Some(40));
        assert_eq!(config.max_tokens, Some(1000));
        assert_eq!(config.thinking_mode, Some(true));
        assert_eq!(config.stop_sequences, vec!["STOP"]);
    }

    #[test]
    fn test_model_config_validation() {
        let config = ModelConfig::new("test-model");
        assert!(config.validate().is_ok());

        // Test empty name
        let mut config = ModelConfig::new("");
        assert!(config.validate().is_err());

        // Test invalid temperature
        config = ModelConfig::new("test");
        config.temperature = 3.0;
        assert!(config.validate().is_err());

        config.temperature = -1.0;
        assert!(config.validate().is_err());

        // Test invalid top_p
        config = ModelConfig::new("test");
        config.top_p = 1.5;
        assert!(config.validate().is_err());

        config.top_p = -0.1;
        assert!(config.validate().is_err());

        // Test invalid max_tokens
        config = ModelConfig::new("test");
        config.max_tokens = Some(0);
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_model_config_temperature_validation() {
        let config = ModelConfig::new("test");

        // Valid temperatures
        assert!(config.clone().with_temperature(0.0).is_ok());
        assert!(config.clone().with_temperature(1.0).is_ok());
        assert!(config.clone().with_temperature(2.0).is_ok());

        // Invalid temperatures
        assert!(config.clone().with_temperature(-0.1).is_err());
        assert!(config.clone().with_temperature(2.1).is_err());
    }

    #[test]
    fn test_model_config_top_p_validation() {
        let config = ModelConfig::new("test");

        // Valid top_p values
        assert!(config.clone().with_top_p(0.0).is_ok());
        assert!(config.clone().with_top_p(0.5).is_ok());
        assert!(config.clone().with_top_p(1.0).is_ok());

        // Invalid top_p values
        assert!(config.clone().with_top_p(-0.1).is_err());
        assert!(config.clone().with_top_p(1.1).is_err());
    }

    #[test]
    fn test_model_config_presets() {
        let conservative = ModelConfig::conservative("test-model");
        assert_eq!(conservative.temperature, 0.3);
        assert_eq!(conservative.top_p, 0.8);

        let creative = ModelConfig::creative("test-model");
        assert_eq!(creative.temperature, 1.2);
        assert_eq!(creative.top_p, 0.95);

        let balanced = ModelConfig::balanced("test-model");
        assert_eq!(balanced.temperature, 0.7);
        assert_eq!(balanced.top_p, 0.9);
    }

    #[test]
    fn test_model_config_stop_sequences() {
        let config = ModelConfig::new("test")
            .with_stop_sequences(vec!["STOP", "END", "FINISH"]);

        assert_eq!(config.stop_sequences.len(), 3);
        assert!(config.stop_sequences.contains(&"STOP".to_string()));
        assert!(config.stop_sequences.contains(&"END".to_string()));
        assert!(config.stop_sequences.contains(&"FINISH".to_string()));
    }
}
