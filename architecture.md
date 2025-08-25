# Orchestra-rs Architecture Documentation

## Overview

Orchestra-rs is designed with a modular, type-safe architecture that provides a unified interface for interacting with different Large Language Model (LLM) providers. The architecture follows Rust best practices with clear separation of concerns, comprehensive error handling, and extensible design patterns.

## High-Level Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Application Layer                        │
│  ┌─────────────────┐  ┌─────────────────┐  ┌──────────────┐ │
│  │   User Code     │  │    Examples     │  │    Tests     │ │
│  └─────────────────┘  └─────────────────┘  └──────────────┘ │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                      LLM Interface                         │
│  ┌─────────────────────────────────────────────────────────┐ │
│  │                    LLM Struct                          │ │
│  │  - provider_source: ProviderSource                     │ │
│  │  - model_name: String                                  │ │
│  │  - provider: Box<dyn ProviderExt>                      │ │
│  │  - config: ModelConfig                                 │ │
│  └─────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                    Core Types Layer                        │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────┐   │
│  │   Messages   │  │    Model     │  │      Error       │   │
│  │              │  │   Config     │  │    Handling      │   │
│  └──────────────┘  └──────────────┘  └──────────────────┘   │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                   Provider Layer                           │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────┐   │
│  │    Gemini    │  │     Mock     │  │   Future: OpenAI │   │
│  │   Provider   │  │   Provider   │  │   Anthropic, etc │   │
│  └──────────────┘  └──────────────┘  └──────────────────┘   │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                  External Services                         │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────┐   │
│  │ Google Gemini│  │  Test Mocks  │  │   Future APIs    │   │
│  │     API      │  │              │  │                  │   │
│  └──────────────┘  └──────────────┘  └──────────────────┘   │
└─────────────────────────────────────────────────────────────┘
```

## Core Components

### 1. LLM Interface (`src/llm/mod.rs`)

The main entry point for users, providing a unified interface across all providers.

**Key Responsibilities:**
- Abstracts provider-specific implementations
- Manages model configuration
- Provides high-level methods (`prompt()`, `chat()`)
- Handles provider instantiation

**Communication Flow:**
```
User Code → LLM::prompt() → ProviderSource → ProviderExt implementation → Specific Provider → External API
```

**Key Types:**
```rust
pub struct LLM {
    pub provider_source: ProviderSource,
    pub model_name: String,
    // The concrete provider implementation is held behind the ProviderExt trait
    // (trait object) so the LLM can call provider-specific methods polymorphically.
    pub provider: Box<dyn ProviderExt>,
    pub config: ModelConfig,
}

// Provider selection is declared by ProviderSource. Concrete providers implement
// a common trait (ProviderExt / Provider) and are instantiated when building
// the LLM. This avoids a central enum that must be updated for every new
// provider.

pub enum ProviderSource {
    Gemini,
    Mock,
    // Future: OpenAI, Anthropic, etc.
}

// Example instantiation (conceptual):
// let provider: Box<dyn ProviderExt> = match provider_source {
//     ProviderSource::Gemini => Box::new(GeminiProvider::with_default_config()),
//     ProviderSource::Mock => Box::new(MockProvider::new(mock_config)),
// };
```

### 2. Message System (`src/messages/mod.rs`)

Handles conversation structure and content types.

**Architecture:**
```
Message (enum)
├── Human(HumanMessage)
├── Assistant(AssistantMessage)
└── System(SystemMessage)

MessageContent (enum)
├── Text(String)
└── Mixed { text: Option<String>, tool_calls: Vec<ToolCall> }
```

**Communication:**
- Messages flow from user code through LLM interface to providers
- Providers convert messages to their specific API formats
- Responses are converted back to standard Message types

### 3. Model Configuration (`src/model/config.rs`)

Manages LLM behavior settings with validation and presets.

**Builder Pattern:**
```rust
ModelConfig::new("model-name")
    .with_temperature(0.7)?
    .with_system_instruction("You are helpful")
    .with_max_tokens(1000)
```

**Validation Flow:**
```
User Input → Builder Methods → Validation → ModelConfig → Provider
```

### 4. Provider Architecture (`src/providers/`)

Pluggable provider system with standardized interface.

**Provider Trait:**
```rust
#[async_trait]
pub trait Provider: Send + Sync + std::fmt::Debug {
    type Config: Send + Sync + std::fmt::Debug;
    
    fn new(config: Self::Config) -> Self;
    fn get_base_url(&self) -> &str;
    fn get_predefined_models(&self) -> Result<Vec<String>>;
    async fn chat(&self, model_config: ModelConfig, message: Message, chat_history: Vec<Message>) -> Result<ChatResponse>;
    async fn prompt(&self, model_config: ModelConfig, prompt: String) -> Result<ChatResponse>;
    fn name(&self) -> &'static str;
    fn supports_streaming(&self) -> bool;
    fn supports_tools(&self) -> bool;
}
```

**Provider Communication Flow:**
```
LLM Interface → Provider Trait → Specific Implementation → HTTP Client → External API
                                                        ↓
                                Error Handling ← Response Processing ← API Response
```

### 5. Error Handling (`src/error.rs`)

Comprehensive error system with context and recovery information.

**Error Hierarchy:**
```
OrchestraError (enum)
├── Http(reqwest::Error)
├── Json(serde_json::Error)
├── InvalidHeader(InvalidHeaderValue)
├── ApiKey { message: String }
├── Provider { provider: String, message: String }
├── Config { message: String }
├── Model { message: String }
├── RateLimit { message: String }
├── Authentication { message: String }
├── InvalidResponse { message: String }
├── Timeout { message: String }
└── Generic { message: String }
```

## Data Flow

### 1. Simple Prompt Flow

```
User Code
    │ llm.prompt("Hello")
    ▼
LLM::prompt()
    │ Message::human("Hello")
    ▼
LLM::chat()
    │ (message, empty_history)
    ▼
ProviderSource::Gemini (instantiated as a ProviderExt implementation)
    │ provider.chat(config, message, history)
    ▼
GeminiProvider::chat()
    │ Convert to GeminiRequestBody
    ▼
HTTP Client (reqwest)
    │ POST to Gemini API
    ▼
Gemini API
    │ GeminiChatResponse
    ▼
Response Processing
    │ Extract text, handle errors
    ▼
ChatResponse { text: String }
    │
    ▼
User Code
```

### 2. Configuration Flow

```
User Code
    │ ModelConfig::new("model")
    ▼
Builder Pattern
    │ .with_temperature(0.7)?
    │ .with_system_instruction("...")
    ▼
Validation
    │ Check ranges, required fields
    ▼
ModelConfig
    │ Passed to provider
    ▼
Provider-Specific Conversion
    │ GeminiGenerationConfig::from_model_config()
    ▼
API Request
```

### 3. Error Propagation

```
External API Error
    │ HTTP 401, 429, 500, etc.
    ▼
Provider Error Handling
    │ Check status codes, parse error responses
    ▼
OrchestraError Creation
    │ Categorize error type, add context
    ▼
Error Propagation
    │ ? operator through call stack
    ▼
User Code
    │ Pattern match on error types
```

## Provider Implementation Details

### Gemini Provider (`src/providers/gemini/`)

**File Structure:**
```
gemini/
├── mod.rs          # Public exports
├── impl.rs         # Provider trait implementation
└── types.rs        # Gemini-specific types
```

**Request/Response Flow:**
```
ModelConfig + Messages
    │
    ▼
GeminiRequestBody {
    system_instruction: Option<SystemInstruction>,
    contents: Vec<GeminiContent>,
    generation_config: Option<GeminiGenerationConfig>,
}
    │ JSON serialization
    ▼
HTTP POST to generativelanguage.googleapis.com
    │
    ▼
GeminiChatResponse {
    candidates: Vec<GeminiCandidate>,
    usage_metadata: Option<UsageMetadata>,
    error: Option<GeminiError>,
}
    │ JSON deserialization
    ▼
ChatResponse { text: String }
```

### Mock Provider (`src/providers/mock.rs`)

**Purpose:**
- Testing without external API calls
- Simulating different response scenarios
- Performance testing with controlled delays

**Configuration:**
```rust
MockConfig::new()
    .with_responses(vec!["Response 1", "Response 2"])
    .with_error(false)
    .with_delay(100) // milliseconds
```

## Extension Points

### Adding New Providers

1. **Create provider module:**
   ```
   src/providers/new_provider/
   ├── mod.rs
   ├── impl.rs
   └── types.rs
   ```

2. **Implement Provider trait:**
   ```rust
   #[async_trait]
   impl Provider for NewProvider {
       type Config = NewProviderConfig;
       // ... implement all required methods
   }
   ```

3. **Provider selection and instantiation (current design):**

Instead of a large enum that owns every concrete provider, the codebase uses a
`ProviderSource` enum to declare which provider to use and trait objects (the
`ProviderExt` / `Provider` trait) for the concrete implementation. This keeps
the LLM API stable while allowing new providers to be added without editing a
centralized enum with every change.

Example steps to add a new provider:

1. Create provider module:
```
src/providers/new_provider/
├── mod.rs
├── impl.rs
└── types.rs
```

2. Implement the provider trait (ProviderExt / Provider) for the provider type:
```rust
#[async_trait]
impl Provider for NewProvider {
    type Config = NewProviderConfig;
    // ... implement required methods
}
```

3. Add a variant to `ProviderSource` and provide an instantiation path when
   constructing the LLM (or use a factory helper). Example:
```rust
pub enum ProviderSource { Gemini, NewProvider, /* ... */ }

let provider: Box<dyn ProviderExt> = match provider_source {
    ProviderSource::Gemini => Box::new(GeminiProvider::with_default_config()),
    ProviderSource::NewProvider => Box::new(NewProvider::from_config(cfg)),
};
```

This pattern keeps provider wiring local to the constructor or a small factory
module and avoids a monolithic provider enum that must be edited for every new
implementation.

### Adding New Message Types

1. **Extend MessageContent enum:**
   ```rust
   pub enum MessageContent {
       Text(String),
       Mixed { text: Option<String>, tool_calls: Vec<ToolCall> },
       NewType(NewContentType),
   }
   ```

2. **Update provider conversions:**
   ```rust
   impl From<&Message> for ProviderSpecificType {
       fn from(msg: &Message) -> Self {
           // Handle new message types
       }
   }
   ```

## Testing Architecture

### Test Structure
```
tests/
├── Unit Tests (in each module)
├── Integration Tests (examples/)
└── Mock Provider Tests
```

### Mock Provider Usage
```rust
let mock_config = MockConfig::new()
    .with_responses(vec!["Expected response"]);
let provider = MockProvider::new(mock_config);

// Use in tests without external API calls
```

## Performance Considerations

### Async Design
- All I/O operations are async
- Non-blocking provider implementations
- Efficient error propagation with `?` operator

### Memory Management
- Zero-copy where possible
- Efficient string handling
- Minimal allocations in hot paths

### Error Handling
- Early error detection with validation
- Structured error types for better debugging
- Context preservation through error chain

## Security Considerations

### API Key Management
- Environment variable based configuration
- No hardcoded credentials
- Secure error messages (no key leakage)

### Input Validation
- Model configuration validation
- Message content sanitization
- Provider-specific input checks

### Network Security
- HTTPS only for external communications
- Proper certificate validation
- Timeout handling for network requests

This architecture provides a solid foundation for building AI applications in Rust while maintaining type safety, performance, and extensibility.
