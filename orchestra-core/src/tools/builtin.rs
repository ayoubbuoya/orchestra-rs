//! # Built-in Tools
//!
//! This module provides a collection of commonly used tools that can be
//! registered with the tool system out of the box.

use async_trait::async_trait;
use serde_json::{json, Value};
use std::time::SystemTime;

use crate::error::Result;
use super::{
    Tool, ToolDefinition, ToolParameter, ToolParameterType,
    result::{ToolResult, ToolError, ToolErrorType},
};

/// A simple calculator tool for basic arithmetic operations
///
/// This tool demonstrates how to implement the Tool trait and provides
/// a useful example for beginners learning the system.
///
/// ## For Rust Beginners
///
/// This struct shows several important patterns:
/// - Implementing traits for custom types
/// - Using enums for type-safe operation selection
/// - Error handling with custom error types
/// - JSON serialization/deserialization
#[derive(Debug)]
pub struct CalculatorTool {
    definition: ToolDefinition,
}

impl CalculatorTool {
    /// Create a new calculator tool
    pub fn new() -> Self {
        let definition = ToolDefinition::new(
            "calculator",
            "Performs basic arithmetic operations on two numbers"
        )
        .with_parameter(
            ToolParameter::new("operation", ToolParameterType::String)
                .with_description("The operation to perform")
                .with_enum_values(vec!["add", "subtract", "multiply", "divide"])
                .required()
        )
        .with_parameter(
            ToolParameter::new("a", ToolParameterType::Number)
                .with_description("First number")
                .required()
        )
        .with_parameter(
            ToolParameter::new("b", ToolParameterType::Number)
                .with_description("Second number")
                .required()
        );

        Self { definition }
    }
}

#[async_trait]
impl Tool for CalculatorTool {
    fn definition(&self) -> &ToolDefinition {
        &self.definition
    }

    async fn execute(&self, arguments: Value) -> Result<ToolResult> {
        // Extract arguments
        let operation = arguments["operation"]
            .as_str()
            .ok_or_else(|| crate::error::OrchestraError::config("Missing operation parameter"))?;
        
        let a = arguments["a"]
            .as_f64()
            .ok_or_else(|| crate::error::OrchestraError::config("Missing or invalid parameter 'a'"))?;
        
        let b = arguments["b"]
            .as_f64()
            .ok_or_else(|| crate::error::OrchestraError::config("Missing or invalid parameter 'b'"))?;

        // Perform calculation
        let result = match operation {
            "add" => a + b,
            "subtract" => a - b,
            "multiply" => a * b,
            "divide" => {
                if b == 0.0 {
                    return Ok(ToolResult::error_with_details(
                        "Division by zero",
                        ToolError::new(ToolErrorType::InvalidInput, "Cannot divide by zero")
                    ));
                }
                a / b
            }
            _ => {
                return Ok(ToolResult::error_with_details(
                    format!("Unknown operation: {}", operation),
                    ToolError::new(ToolErrorType::InvalidInput, "Invalid operation")
                ));
            }
        };

        Ok(ToolResult::success(json!({
            "result": result,
            "operation": operation,
            "operands": [a, b]
        })))
    }
}

/// A tool that provides current timestamp information
///
/// This tool demonstrates working with system time and different output formats.
#[derive(Debug)]
pub struct TimestampTool {
    definition: ToolDefinition,
}

impl TimestampTool {
    /// Create a new timestamp tool
    pub fn new() -> Self {
        let definition = ToolDefinition::new(
            "get_timestamp",
            "Get the current timestamp in various formats"
        )
        .with_parameter(
            ToolParameter::new("format", ToolParameterType::String)
                .with_description("The format for the timestamp")
                .with_enum_values(vec!["unix", "iso8601", "human"])
                .with_default(json!("unix"))
        );

        Self { definition }
    }
}

#[async_trait]
impl Tool for TimestampTool {
    fn definition(&self) -> &ToolDefinition {
        &self.definition
    }

    async fn execute(&self, arguments: Value) -> Result<ToolResult> {
        let format = arguments.get("format")
            .and_then(|v| v.as_str())
            .unwrap_or("unix");

        let now = SystemTime::now();
        let unix_timestamp = now.duration_since(SystemTime::UNIX_EPOCH)
            .map_err(|_| crate::error::OrchestraError::generic("Failed to get system time"))?
            .as_secs();

        let result = match format {
            "unix" => json!({
                "timestamp": unix_timestamp,
                "format": "unix"
            }),
            "iso8601" => {
                // For a real implementation, you'd use a proper datetime library like chrono
                json!({
                    "timestamp": format!("2024-01-01T00:00:{}Z", unix_timestamp % 60),
                    "format": "iso8601",
                    "note": "This is a simplified implementation"
                })
            },
            "human" => json!({
                "timestamp": format!("Current time (simplified): {} seconds since epoch", unix_timestamp),
                "format": "human"
            }),
            _ => {
                return Ok(ToolResult::error_with_details(
                    format!("Unknown format: {}", format),
                    ToolError::new(ToolErrorType::InvalidInput, "Invalid format")
                ));
            }
        };

        Ok(ToolResult::success(result))
    }
}

/// A tool that generates random numbers
///
/// This tool demonstrates parameter validation and random number generation.
#[derive(Debug)]
pub struct RandomNumberTool {
    definition: ToolDefinition,
}

impl RandomNumberTool {
    /// Create a new random number tool
    pub fn new() -> Self {
        let definition = ToolDefinition::new(
            "random_number",
            "Generate a random number within a specified range"
        )
        .with_parameter(
            ToolParameter::new("min", ToolParameterType::Integer)
                .with_description("Minimum value (inclusive)")
                .with_default(json!(0))
        )
        .with_parameter(
            ToolParameter::new("max", ToolParameterType::Integer)
                .with_description("Maximum value (inclusive)")
                .with_default(json!(100))
        );

        Self { definition }
    }
}

#[async_trait]
impl Tool for RandomNumberTool {
    fn definition(&self) -> &ToolDefinition {
        &self.definition
    }

    async fn execute(&self, arguments: Value) -> Result<ToolResult> {
        let min = arguments.get("min")
            .and_then(|v| v.as_i64())
            .unwrap_or(0);
        
        let max = arguments.get("max")
            .and_then(|v| v.as_i64())
            .unwrap_or(100);

        if min > max {
            return Ok(ToolResult::error_with_details(
                "Minimum value cannot be greater than maximum",
                ToolError::new(ToolErrorType::InvalidInput, "Invalid range")
            ));
        }

        // Simple random number generation (in a real implementation, you'd use the `rand` crate)
        let range = (max - min + 1) as u64;
        let random_value = (SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_nanos() % range as u128) as i64 + min;

        Ok(ToolResult::success(json!({
            "value": random_value,
            "min": min,
            "max": max
        })))
    }
}

/// Create a registry with all built-in tools
///
/// This is a convenience function that creates a tool registry pre-populated
/// with useful built-in tools.
///
/// # Example
/// ```rust
/// use orchestra_core::tools::builtin::create_builtin_registry;
///
/// let registry = create_builtin_registry();
/// println!("Available tools: {:?}", registry.tool_names());
/// ```
pub fn create_builtin_registry() -> super::ToolRegistry {
    let registry = super::ToolRegistry::new();
    
    // Register built-in tools
    if let Err(e) = registry.register(super::boxed_tool(CalculatorTool::new())) {
        eprintln!("Failed to register calculator tool: {}", e);
    }
    
    if let Err(e) = registry.register(super::boxed_tool(TimestampTool::new())) {
        eprintln!("Failed to register timestamp tool: {}", e);
    }
    
    if let Err(e) = registry.register(super::boxed_tool(RandomNumberTool::new())) {
        eprintln!("Failed to register random number tool: {}", e);
    }
    
    // Add tools to categories
    let _ = registry.add_to_category("math", "calculator");
    let _ = registry.add_to_category("utility", "get_timestamp");
    let _ = registry.add_to_category("utility", "random_number");
    let _ = registry.add_to_category("math", "random_number");
    
    registry
}
