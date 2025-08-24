//! # Orchestra-rs Core
//!
//! A Rust library for building AI agent workflows and applications with Large Language Models (LLMs).
//!
//! Orchestra-rs provides a type-safe, async-first framework for interacting with various LLM providers,
//! managing conversations, and building intelligent applications.
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use orchestra_core::{llm::LLM, providers::types::ProviderSource};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Set your API key as an environment variable
//!     unsafe { std::env::set_var("GEMINI_API_KEY", "your-api-key-here"); }
//!
//!     // Create an LLM instance
//!     let llm = LLM::gemini("gemini-2.5-flash");
//!
//!     // Send a prompt
//!     let response = llm.prompt("Hello, how are you?").await?;
//!     println!("Response: {}", response.text);
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Features
//!
//! - **Type-safe LLM interactions**: Leverage Rust's type system for reliable AI applications
//! - **Multiple provider support**: Currently supports Google Gemini, with more providers coming
//! - **Flexible configuration**: Builder patterns and validation for model configurations
//! - **Rich message types**: Support for text, mixed content, and future tool calling
//! - **Comprehensive error handling**: Detailed error types with context
//! - **Async/await support**: Built for modern async Rust applications
//!
//! ## Modules
//!
//! - [`llm`]: High-level interface for interacting with LLMs
//! - [`messages`]: Message types for conversations
//! - [`model`]: Model configuration and settings
//! - [`providers`]: LLM provider implementations
//! - [`error`]: Error types and handling

pub mod error;
pub mod llm;
pub mod messages;
pub mod model;
pub mod providers;

// Re-export commonly used types
pub use error::{OrchestraError, Result};
