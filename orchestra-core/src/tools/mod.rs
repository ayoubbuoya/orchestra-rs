//! # Tool Calling System
//!
//! This module provides a comprehensive tool calling system for Orchestra-rs, allowing LLMs
//! to call external functions and tools in a type-safe and structured way.
//!
//! ## Overview
//!
//! The tool system consists of several key components:
//! - **Tool Definitions**: Describe what tools are available and their parameters
//! - **Tool Registry**: Manages available tools and their execution
//! - **Tool Execution**: Safely executes tool calls with proper error handling
//! - **Tool Results**: Structured responses from tool execution
//!
//! ## Key Concepts for Rust Beginners
//!
//! ### Traits and Generics
//! This module makes heavy use of Rust's trait system to provide flexibility while
//! maintaining type safety. Traits define shared behavior that types can implement.
//!
//! ### Async Programming
//! Tool execution is asynchronous, allowing for non-blocking operations like API calls
//! or file I/O. The `async`/`await` syntax makes this easier to work with.
//!
//! ### Error Handling
//! Rust's `Result<T, E>` type is used throughout for explicit error handling.
//! This makes it clear when operations can fail and forces you to handle errors.
//!
//! ## Example Usage
//!
//! ```rust,no_run
//! use orchestra_core::tools::{Tool, ToolDefinition, ToolParameter, ToolParameterType};
//! use serde_json::json;
//!
//! // Define a simple calculator tool
//! let calculator_tool = ToolDefinition::new(
//!     "calculator",
//!     "Performs basic arithmetic operations"
//! )
//! .with_parameter(
//!     ToolParameter::new("operation", ToolParameterType::String)
//!         .with_description("The operation to perform: add, subtract, multiply, divide")
//!         .with_enum_values(vec!["add", "subtract", "multiply", "divide"])
//!         .required()
//! )
//! .with_parameter(
//!     ToolParameter::new("a", ToolParameterType::Number)
//!         .with_description("First number")
//!         .required()
//! )
//! .with_parameter(
//!     ToolParameter::new("b", ToolParameterType::Number)
//!         .with_description("Second number")
//!         .required()
//! );
//! ```

pub mod builtin;
pub mod definition;
pub mod execution;
pub mod registry;
pub mod result;

// Re-export commonly used types for convenience
pub use definition::{ToolDefinition, ToolParameter, ToolParameterType};
pub use execution::{ToolExecutor, ToolHandler};
pub use registry::ToolRegistry;
pub use result::{ToolResult, ToolResultStatus};

use async_trait::async_trait;
use serde_json::Value;
use crate::error::Result;

/// Core trait that all tools must implement.
///
/// This trait defines the interface for executing tools. It uses async to allow
/// for non-blocking operations like network requests or file I/O.
///
/// ## For Rust Beginners
///
/// - `async_trait` is a macro that allows async functions in traits
/// - `Send + Sync` means the trait can be safely used across threads
/// - `std::fmt::Debug` allows the tool to be printed for debugging
#[async_trait]
pub trait Tool: Send + Sync + std::fmt::Debug {
    /// Get the tool's definition (name, description, parameters)
    fn definition(&self) -> &ToolDefinition;
    
    /// Execute the tool with the given arguments
    ///
    /// # Arguments
    /// * `arguments` - JSON object containing the tool's input parameters
    ///
    /// # Returns
    /// A `ToolResult` containing the execution result or error information
    ///
    /// # Example Implementation
    /// ```rust,no_run
    /// use async_trait::async_trait;
    /// use orchestra_core::tools::{Tool, ToolDefinition, ToolResult};
    /// use serde_json::Value;
    /// use orchestra_core::error::Result;
    ///
    /// #[derive(Debug)]
    /// struct CalculatorTool {
    ///     definition: ToolDefinition,
    /// }
    ///
    /// #[async_trait]
    /// impl Tool for CalculatorTool {
    ///     fn definition(&self) -> &ToolDefinition {
    ///         &self.definition
    ///     }
    ///
    ///     async fn execute(&self, arguments: Value) -> Result<ToolResult> {
    ///         // Implementation here
    ///         todo!()
    ///     }
    /// }
    /// ```
    async fn execute(&self, arguments: Value) -> Result<ToolResult>;
}

/// A boxed tool that can be stored in collections
///
/// ## For Rust Beginners
///
/// `Box<dyn Tool>` is a "trait object" - it allows us to store different types
/// that implement the Tool trait in the same collection. The `Box` puts the
/// tool on the heap, and `dyn` means "dynamic dispatch" - the exact type is
/// determined at runtime.
pub type BoxedTool = Box<dyn Tool>;

/// Convenience function to create a boxed tool
///
/// This function takes any type that implements Tool and wraps it in a Box,
/// making it easier to work with collections of different tool types.
pub fn boxed_tool<T: Tool + 'static>(tool: T) -> BoxedTool {
    Box::new(tool)
}
