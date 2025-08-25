# Orchestra-rs

[![crates.io](https://img.shields.io/crates/v/orchestra-rs.svg?style=flat-square)](https://crates.io/crates/orchestra-rs) <!-- TODO: Update when published -->
[![docs.rs](https://img.shields.io/docsrs/orchestra-rs?style=flat-square)](https://docs.rs/orchestra-rs) <!-- TODO: Update when published -->
[![CI](https://img.shields.io/github/actions/workflow/status/YourUsername/orchestra-rs/rust.yml?branch=main&style=flat-square)](https://github.com/YourUsername/orchestra-rs/actions) <!-- TODO: Update with username and repo -->
[![License](https://img.shields.io/crates/l/orchestra-rs.svg?style=flat-square)](https://github.com/YourUsername/orchestra-rs/blob/main/LICENSE) <!-- TODO: Update with  username and repo -->

A Rust crate for building AI agent workflows and applications. Orchestra-rs provides a powerful, type-safe framework for orchestrating production-ready applications powered by Large Language Models (LLMs).

## Vision

The goal of **Orchestra-rs** is to be the `LangChain` of the Rust ecosystem. We aim to provide a composable, safe, and efficient set of tools to chain together calls to LLMs, APIs, and other data sources. By leveraging Rust's powerful type system and performance, Orchestra-rs empowers developers to build reliable and scalable AI applications and intelligent agents with confidence.

## Features

- 🚀 **Type-safe LLM interactions** - Leverage Rust's type system for reliable AI applications
- 🔌 **Multiple provider support** - Currently supports Google Gemini, with more providers coming
- 🛠️ **Flexible configuration** - Builder patterns and validation for model configurations
- 📝 **Rich message types** - Support for text, mixed content, and future tool calling
- 🧪 **Comprehensive testing** - Built-in mock providers and extensive test coverage
- ⚡ **Async/await support** - Built for modern async Rust applications
- 🔒 **Error handling** - Comprehensive error types with context

## Quick Start

Add Orchestra-rs to your `Cargo.toml`:

```toml

[dependencies]
orchestra-rs = { path = "." }  # Will be published to crates.io soon
tokio = { version = "1.0", features = ["full"] }
```

### Basic Usage


```rust
use orchestra_rs::{
    llm::LLM,
    providers::types::ProviderSource,
    messages::Message,
    model::ModelConfig,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Set your API key as an environment variable
    std::env::set_var("GEMINI_API_KEY", "your-api-key-here");

    // Create an LLM instance with Gemini
    let llm = LLM::gemini("gemini-2.5-flash");

    // Simple prompt
    let response = llm.prompt("Hello, how are you today?").await?;
    println!("Response: {}", response.text);

    Ok(())
}
```

### Advanced Configuration


```rust
use orchestra_rs::{
    llm::LLM,
    providers::types::ProviderSource,
    model::ModelConfig,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a custom model configuration
    let config = ModelConfig::new("gemini-2.5-flash")
        .with_system_instruction("You are a helpful coding assistant")
        .with_temperature(0.7)?
        .with_top_p(0.9)?
        .with_max_tokens(1000)
        .with_stop_sequence("```");

    // Create LLM with custom configuration
    let llm = LLM::new(ProviderSource::Gemini, "gemini-2.5-flash".to_string())
        .with_custom_config(config);

    let response = llm.prompt("Write a simple Rust function").await?;
    println!("Response: {}", response.text);

    Ok(())
}
````

### Chat with History


```rust
use orchestra_rs::{
    llm::LLM,
    messages::Message,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let llm = LLM::gemini("gemini-2.5-flash");

    // Build conversation history
    let history = vec![
        Message::human("Hi, I'm working on a Rust project"),
        Message::assistant("Great! I'd be happy to help with your Rust project. What are you working on?"),
    ];

    // Continue the conversation
    let response = llm.chat(
        Message::human("I need help with error handling"),
        history
    ).await?;

    println!("Response: {}", response.text);
    Ok(())
}
```

### Using Presets


```rust
use orchestra_rs::{
    llm::LLM,
    providers::types::ProviderSource,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Conservative settings (lower temperature, more focused)
    let conservative_llm = LLM::conservative(
        ProviderSource::Gemini,
        "gemini-2.5-flash".to_string()
    );

    // Creative settings (higher temperature, more diverse)
    let creative_llm = LLM::creative(
        ProviderSource::Gemini,
        "gemini-2.5-flash".to_string()
    );

    // Balanced settings (moderate temperature)
    let balanced_llm = LLM::balanced(
        ProviderSource::Gemini,
        "gemini-2.5-flash".to_string()
    );

    let response = conservative_llm.prompt("Explain Rust ownership").await?;
    println!("Conservative response: {}", response.text);

    Ok(())
}
```

## Supported Providers

### Google Gemini

Currently supported models:

- `gemini-2.5-flash-lite`
- `gemini-2.5-pro`
- `gemini-2.5-flash`
- `gemini-2.0-flash-lite`
- `gemini-2.0-flash`
- `gemini-1.5-pro`

**Setup:**

1. Get an API key from [Google AI Studio](https://aistudio.google.com/)
2. Set the environment variable: `GEMINI_API_KEY=your-api-key`

### Coming Soon

- OpenAI GPT models
- Anthropic Claude
- Local models via Ollama
- Azure OpenAI

## Architecture

Orchestra-rs is built with a modular architecture:

- **Core Types**: Message types, model configurations, and error handling
- **Providers**: Pluggable LLM provider implementations
- **LLM Interface**: High-level interface for interacting with any provider
- **Configuration**: Flexible configuration with validation and presets

## Error Handling

Orchestra-rs provides comprehensive error handling with context:

```rust

use orchestra_rs::{error::OrchestraError, llm::LLM};

#[tokio::main]
async fn main() {
    let llm = LLM::gemini("gemini-2.5-flash");

    match llm.prompt("Hello").await {
        Ok(response) => println!("Success: {}", response.text),
        Err(OrchestraError::ApiKey { message }) => {
            eprintln!("API key error: {}", message);
        },
        Err(OrchestraError::Provider { provider, message }) => {
            eprintln!("Provider {} error: {}", provider, message);
        },
        Err(e) => eprintln!("Other error: {}", e),
    }
}
```

## Testing

Orchestra-rs includes comprehensive testing utilities:

```rust

use orchestra_rs::providers::mock::{MockProvider, MockConfig};

#[tokio::test]
async fn test_my_ai_function() {
    let mock_config = MockConfig::new()
        .with_responses(vec!["Mocked response 1", "Mocked response 2"]);

    let provider = MockProvider::new(mock_config);
    // Use the mock provider in your tests
}
```

## Architecture Documentation

For a detailed overview of the library's architecture, please refer to the [architecture documentation](architecture.md).

## Project Status

**🌱 Early Development Stage**

Orchestra-rs is in active development. The core APIs are stabilizing, but may still change. This is a great time to get involved and help shape the future of the framework.

### Roadmap

- [x] Core message and configuration types
- [x] Google Gemini provider
- [x] Comprehensive error handling
- [x] Testing utilities and mock providers
- [ ] Tool calling support
- [ ] Streaming responses
- [ ] Additional providers (OpenAI, Anthropic, etc.)
- [ ] Agent workflows and chains
- [ ] Memory and context management
- [ ] Plugin system

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
