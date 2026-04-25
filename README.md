# Orchestra-rs

[![crates.io](https://img.shields.io/crates/v/orchestra-rs.svg?style=flat-square)](https://crates.io/crates/orchestra-rs)
[![docs.rs](https://img.shields.io/docsrs/orchestra-rs?style=flat-square)](https://docs.rs/orchestra-rs)
[![License](https://img.shields.io/badge/license-MIT-blue.svg?style=flat-square)](LICENSE)

A provider-agnostic orchestration framework for building AI-powered applications in Rust. Unify any LLM behind a single interface — switch providers by changing a string.

## Why Orchestra-rs?

Every LLM provider has a different API, different message format, different streaming semantics, and different error handling. Orchestra-rs normalizes all of that behind a clean, type-safe, async Rust interface.

- **One interface, any provider** — Gemini, OpenAI, Anthropic, Ollama, or your own. Implement the `Provider` trait once and it just works.
- **Zero-cost abstractions** — Feature-gated providers. Only compile what you use.
- **Progressive complexity** — One-liner for simple prompts. Full control when you need it.
- **Streaming built-in** — Unified `StreamEvent` type across all providers.
- **Tool calling** — Define tools, pass them in, handle results. Same API regardless of provider.
- **Ergonomic API** — Builder patterns, environment variable auto-detection, sensible defaults.

## Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
orchestra-rs = { version = "0.2", features = ["gemini"] }
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
```

Set your API key:

```bash
export GEMINI_API_KEY=your-api-key
```

Start using it:

```rust
use orchestra_rs::{Orchestra, providers::GeminiProvider};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Orchestra::new()
        .with_provider(GeminiProvider::from_env()?);

    let response = client.chat("gemini-2.5-pro", "Explain Rust ownership in one sentence.").await?;
    println!("{}", response.message);

    Ok(())
}
```

That's it. One line to create the client, one line to chat.

## Usage

### Chat with History

```rust
use orchestra_rs::{Orchestra, Message, providers::GeminiProvider};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Orchestra::new()
        .with_provider(GeminiProvider::from_env()?);

    let messages = vec![
        Message::system("You are a concise coding assistant."),
        Message::user("What is Result<T, E> in Rust?"),
        Message::assistant("Result<T, E> is an enum representing either success (Ok(T)) or failure (Err(E))."),
        Message::user("How is it different from Option<T>?"),
    ];

    let response = client.chat_with("gemini-2.5-pro", &messages, &Default::default()).await?;
    println!("{}", response.message);

    Ok(())
}
```

### Full Configuration

```rust
use orchestra_rs::{Orchestra, RequestConfig, providers::GeminiProvider};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Orchestra::new()
        .with_provider(GeminiProvider::from_env()?);

    let config = RequestConfig {
        temperature: Some(0.7),
        top_p: Some(0.9),
        max_tokens: Some(2048),
        stop_sequences: vec!["```".into()],
        ..Default::default()
    };

    let response = client.chat_with("gemini-2.5-pro", &messages, &config).await?;
    println!("Tokens used: {}", response.usage.total_tokens);

    Ok(())
}
```

### Streaming

```rust
use orchestra_rs::{Orchestra, StreamEvent, providers::GeminiProvider};
use futures::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Orchestra::new()
        .with_provider(GeminiProvider::from_env()?);

    let mut stream = client.chat_stream("gemini-2.5-pro", "Tell me a short story.").await?;

    while let Some(event) = stream.next().await {
        match event? {
            StreamEvent::Delta { text } => print!("{text}"),
            StreamEvent::Finish(reason) => println!("\nFinished: {reason:?}"),
            _ => {}
        }
    }

    Ok(())
}
```

### Tool Calling

```rust
use orchestra_rs::{Orchestra, RequestConfig, Tool, Message, Content, providers::GeminiProvider};
use serde_json::{json, Value};

struct WeatherTool;

#[async_trait::async_trait]
impl Tool for WeatherTool {
    fn name(&self) -> &str { "get_weather" }
    fn description(&self) -> &str { "Get the current weather for a location" }
    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": { "location": { "type": "string" } },
            "required": ["location"]
        })
    }
    async fn execute(&self, input: Value) -> Result<Value, orchestra_rs::OrchestraError> {
        let location = input["location"].as_str().unwrap();
        Ok(json!({ "temperature": 22, "condition": "sunny", "location": location }))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Orchestra::new()
        .with_provider(GeminiProvider::from_env()?);

    let config = RequestConfig {
        tools: vec![std::sync::Arc::new(WeatherTool)],
        ..Default::default()
    };

    let mut messages = vec![Message::user("What's the weather in Tokyo?")];

    let response = client.chat_with("gemini-2.5-pro", &messages, &config).await?;
    messages.push(response.message.clone());

    // Handle tool calls in the response
    for content in response.message.content_parts() {
        if let Content::ToolCall { id, name, input } = content {
            let result = /* execute tool by name */ json!({"temperature": 22});
            messages.push(Message::tool_result(id, name, result.to_string()));
        }
    }

    // Send tool results back
    let final_response = client.chat_with("gemini-2.5-pro", &messages, &config).await?;
    println!("{}", final_response.message);

    Ok(())
}
```

### Multiple Providers

```rust
use orchestra_rs::{Orchestra, providers::{GeminiProvider, OpenAIProvider}};

let client = Orchestra::new()
    .with_provider(GeminiProvider::from_env()?)
    .with_provider(OpenAIProvider::from_env()?);

// Use any registered provider by model name
let gemini_response = client.chat("gemini-2.5-pro", "Hello").await?;
let openai_response = client.chat("gpt-4o", "Hello").await?;
```

### Testing with Mocks

```rust
use orchestra_rs::{Orchestra, providers::MockProvider};

#[tokio::test]
async fn test_my_logic() {
    let mock = MockProvider::new().with_response("Expected response");
    let client = Orchestra::new().with_provider(mock);

    let response = client.chat("any-model", "Hello").await.unwrap();
    assert_eq!(response.message.text(), "Expected response");
}
```

## Feature Flags

Only compile the providers you need:

```toml
[dependencies]
orchestra-rs = { version = "0.2", features = ["gemini"] }
```

| Feature | Description |
|---------|-------------|
| `gemini` | Google Gemini provider |
| `openai` | OpenAI provider |
| `anthropic` | Anthropic Claude provider |
| `mock` | Mock provider for testing |
| `streaming` | Streaming support (enabled by default) |

## Supported Providers

### Google Gemini

```toml
orchestra-rs = { version = "0.2", features = ["gemini"] }
```

```bash
export GEMINI_API_KEY=your-api-key
```

Supports: gemini-2.5-pro, gemini-2.5-flash, gemini-2.5-flash-lite, gemini-2.0-flash, gemini-2.0-flash-lite, gemini-1.5-pro, and any other model string Gemini accepts.

### OpenAI

```toml
orchestra-rs = { version = "0.2", features = ["openai"] }
```

```bash
export OPENAI_API_KEY=your-api-key
```

### Anthropic Claude

```toml
orchestra-rs = { version = "0.2", features = ["anthropic"] }
```

```bash
export ANTHROPIC_API_KEY=your-api-key
```

## Implementing a Custom Provider

Any provider is just an implementation of the `Provider` trait:

```rust
use orchestra_rs::provider::Provider;
use orchestra_rs::core::{Message, ChatResponse, RequestConfig, StreamEvent, ModelInfo};
use async_trait::async_trait;
use std::pin::Pin;
use futures::Stream;

pub struct MyCustomProvider {
    api_key: String,
}

#[async_trait]
impl Provider for MyCustomProvider {
    fn id(&self) -> &str { "my-provider" }

    async fn chat(
        &self,
        model: &str,
        messages: &[Message],
        config: &RequestConfig,
    ) -> Result<ChatResponse, orchestra_rs::OrchestraError> {
        // Call your API, convert the response
        todo!()
    }

    async fn chat_stream(
        &self,
        model: &str,
        messages: &[Message],
        config: &RequestConfig,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<StreamEvent, orchestra_rs::OrchestraError>> + Send>>, orchestra_rs::OrchestraError> {
        // Return a streaming response
        todo!()
    }

    async fn list_models(&self) -> Result<Vec<ModelInfo>, orchestra_rs::OrchestraError> {
        todo!()
    }

    fn supports_tools(&self) -> bool { false }
    fn supports_streaming(&self) -> bool { true }
}

// Register it like any built-in provider
let client = Orchestra::new()
    .with_provider(MyCustomProvider::new("key"));
```

## Architecture

```
src/
├── core/                    # Stable types — messages, responses, errors, config
│   ├── message.rs           # Message, Role, Content
│   ├── response.rs          # ChatResponse, StreamEvent, Usage, FinishReason
│   ├── tool.rs              # Tool trait, ToolDefinition
│   ├── config.rs            # RequestConfig
│   └── error.rs             # OrchestraError
│
├── provider/                # The abstraction layer
│   ├── traits.rs            # Provider trait
│   └── stream.rs            # Unified streaming types
│
├── client.rs                # Orchestra — the user-facing client
│
└── providers/               # Built-in implementations (feature-gated)
    ├── gemini/              # Google Gemini
    ├── openai/              # OpenAI
    └── mock/                # Testing mocks
```

**Core principle:** The `Provider` trait is the only contract. Every provider converts to/from core types internally. Adding a new provider never changes core code.

## Error Handling

```rust
use orchestra_rs::OrchestraError;

match client.chat("gemini-2.5-pro", "Hello").await {
    Ok(response) => println!("{}", response.message),
    Err(OrchestraError::Auth { provider, reason }) => {
        eprintln!("Auth failed for {provider}: {reason}");
    }
    Err(OrchestraError::RateLimited { provider, retry_after_ms }) => {
        if let Some(ms) = retry_after_ms {
            eprintln!("Rate limited by {provider}, retry after {ms}ms");
        }
    }
    Err(OrchestraError::ModelNotFound { provider, model }) => {
        eprintln!("Model {model} not found on {provider}");
    }
    Err(e) => eprintln!("Error: {e}"),
}
```

## Roadmap

- [x] Core types (Message, Response, Config, Error)
- [x] Provider trait abstraction
- [x] Google Gemini provider
- [x] Streaming support
- [x] Tool calling
- [x] Mock provider for testing
- [ ] OpenAI provider
- [ ] Anthropic Claude provider
- [ ] Ollama / local model provider
- [ ] Middleware (retry, logging, caching)
- [ ] Agent loop (automatic tool calling)

## Contributing

Contributions are welcome. Please open an issue first to discuss what you'd like to change.

## License

[MIT](LICENSE)
