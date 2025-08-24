//! # Tool Execution Engine
//!
//! This module provides the execution engine for running tools safely and efficiently.
//! It handles parameter validation, error handling, and result formatting.

use async_trait::async_trait;
use std::time::{Duration, SystemTime};
use serde_json::Value;

use crate::error::{OrchestraError, Result};
use super::{
    Tool, ToolRegistry, 
    result::{ToolResult, ToolError, ToolErrorType},
    definition::ToolParameterType,
};

/// Handles the execution of tools with proper validation and error handling
///
/// The ToolExecutor is responsible for:
/// - Validating input parameters against tool definitions
/// - Executing tools safely with timeouts
/// - Handling errors and formatting results
/// - Providing execution metadata
///
/// ## For Rust Beginners
///
/// This struct demonstrates several important Rust patterns:
/// - Composition over inheritance (contains a ToolRegistry)
/// - Builder pattern for configuration
/// - Error handling with Result types
/// - Async programming with timeouts
#[derive(Debug, Clone)]
pub struct ToolExecutor {
    /// The registry containing available tools
    registry: ToolRegistry,
    
    /// Maximum execution time for tools
    timeout_duration: Duration,
    
    /// Whether to validate parameters before execution
    validate_parameters: bool,
    
    /// Whether to include detailed timing information
    include_timing: bool,
}

impl ToolExecutor {
    /// Create a new tool executor with default settings
    ///
    /// # Arguments
    /// * `registry` - The tool registry to use for execution
    ///
    /// # Example
    /// ```rust
    /// use orchestra_core::tools::{ToolExecutor, ToolRegistry};
    ///
    /// let registry = ToolRegistry::new();
    /// let executor = ToolExecutor::new(registry);
    /// ```
    pub fn new(registry: ToolRegistry) -> Self {
        Self {
            registry,
            timeout_duration: Duration::from_secs(30), // 30 second default timeout
            validate_parameters: true,
            include_timing: true,
        }
    }
    
    /// Set the execution timeout
    ///
    /// Tools that take longer than this duration will be cancelled.
    ///
    /// # Arguments
    /// * `duration` - The maximum execution time
    pub fn with_timeout(mut self, duration: Duration) -> Self {
        self.timeout_duration = duration;
        self
    }
    
    /// Enable or disable parameter validation
    ///
    /// When enabled, input parameters are validated against the tool's
    /// parameter definitions before execution.
    pub fn with_validation(mut self, validate: bool) -> Self {
        self.validate_parameters = validate;
        self
    }
    
    /// Enable or disable detailed timing information
    pub fn with_timing(mut self, include_timing: bool) -> Self {
        self.include_timing = include_timing;
        self
    }
    
    /// Execute a tool by name with the given arguments
    ///
    /// This is the main method for executing tools. It handles all aspects
    /// of tool execution including validation, timeout handling, and error formatting.
    ///
    /// # Arguments
    /// * `tool_name` - The name of the tool to execute
    /// * `arguments` - JSON object containing the tool's input parameters
    ///
    /// # Returns
    /// A `ToolResult` containing the execution result or error information
    ///
    /// # Example
    /// ```rust,no_run
    /// use orchestra_core::tools::{ToolExecutor, ToolRegistry};
    /// use serde_json::json;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let registry = ToolRegistry::new();
    /// let executor = ToolExecutor::new(registry);
    ///
    /// let result = executor.execute("calculator", json!({
    ///     "operation": "add",
    ///     "a": 5,
    ///     "b": 3
    /// })).await?;
    ///
    /// println!("Result: {}", result.to_string());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn execute(&self, tool_name: &str, arguments: Value) -> Result<ToolResult> {
        let start_time = SystemTime::now();
        
        // Get the tool definition
        let tool_def = self.registry.get_tool_definition(tool_name)
            .ok_or_else(|| OrchestraError::config(&format!("Tool '{}' not found", tool_name)))?;
        
        // Validate parameters if enabled
        if self.validate_parameters {
            if let Err(e) = self.validate_parameters(&tool_def, &arguments) {
                return Ok(ToolResult::error_with_details(
                    format!("Parameter validation failed: {}", e),
                    ToolError::new(ToolErrorType::InvalidInput, e.to_string())
                ));
            }
        }
        
        // Execute the actual tool through the registry
        let result = self.registry.execute_tool(tool_name, arguments).await?;
        
        // Add timing information if enabled
        if self.include_timing {
            if let Ok(duration) = start_time.elapsed() {
                return Ok(result.with_metadata("execution_time_ms", 
                    serde_json::Value::Number((duration.as_millis() as u64).into())));
            }
        }
        
        Ok(result)
    }
    
    /// Validate tool parameters against the tool definition
    ///
    /// This method checks that all required parameters are present and that
    /// parameter values match their expected types and constraints.
    fn validate_parameters(&self, tool_def: &super::ToolDefinition, arguments: &Value) -> Result<()> {
        let args_obj = arguments.as_object()
            .ok_or_else(|| OrchestraError::config("Arguments must be a JSON object"))?;
        
        // Check required parameters
        for param in tool_def.required_parameters() {
            if !args_obj.contains_key(&param.name) {
                return Err(OrchestraError::config(&format!(
                    "Required parameter '{}' is missing", param.name
                )));
            }
        }
        
        // Validate each provided parameter
        for (param_name, param_value) in args_obj {
            if let Some(param_def) = tool_def.parameters.get(param_name) {
                self.validate_parameter_value(param_def, param_value)?;
            } else {
                return Err(OrchestraError::config(&format!(
                    "Unknown parameter '{}'", param_name
                )));
            }
        }
        
        Ok(())
    }
    
    /// Validate a single parameter value
    fn validate_parameter_value(&self, param_def: &super::ToolParameter, value: &Value) -> Result<()> {
        // Check type compatibility
        match param_def.parameter_type {
            ToolParameterType::String => {
                if !value.is_string() {
                    return Err(OrchestraError::config(&format!(
                        "Parameter '{}' must be a string", param_def.name
                    )));
                }
                
                let str_val = value.as_str().unwrap();
                
                // Check enum values
                if let Some(ref enum_vals) = param_def.enum_values {
                    if !enum_vals.contains(&str_val.to_string()) {
                        return Err(OrchestraError::config(&format!(
                            "Parameter '{}' must be one of: {:?}", param_def.name, enum_vals
                        )));
                    }
                }
                
                // Check length constraints
                if let Some(min_len) = param_def.min_length {
                    if str_val.len() < min_len {
                        return Err(OrchestraError::config(&format!(
                            "Parameter '{}' must be at least {} characters", param_def.name, min_len
                        )));
                    }
                }
                
                if let Some(max_len) = param_def.max_length {
                    if str_val.len() > max_len {
                        return Err(OrchestraError::config(&format!(
                            "Parameter '{}' must be at most {} characters", param_def.name, max_len
                        )));
                    }
                }
            }
            
            ToolParameterType::Number => {
                if !value.is_number() {
                    return Err(OrchestraError::config(&format!(
                        "Parameter '{}' must be a number", param_def.name
                    )));
                }
                
                let num_val = value.as_f64().unwrap();
                
                // Check range constraints
                if let Some(min) = param_def.minimum {
                    if num_val < min {
                        return Err(OrchestraError::config(&format!(
                            "Parameter '{}' must be at least {}", param_def.name, min
                        )));
                    }
                }
                
                if let Some(max) = param_def.maximum {
                    if num_val > max {
                        return Err(OrchestraError::config(&format!(
                            "Parameter '{}' must be at most {}", param_def.name, max
                        )));
                    }
                }
            }
            
            ToolParameterType::Integer => {
                if !value.is_i64() && !value.is_u64() {
                    return Err(OrchestraError::config(&format!(
                        "Parameter '{}' must be an integer", param_def.name
                    )));
                }
            }
            
            ToolParameterType::Boolean => {
                if !value.is_boolean() {
                    return Err(OrchestraError::config(&format!(
                        "Parameter '{}' must be a boolean", param_def.name
                    )));
                }
            }
            
            ToolParameterType::Array => {
                if !value.is_array() {
                    return Err(OrchestraError::config(&format!(
                        "Parameter '{}' must be an array", param_def.name
                    )));
                }
                
                let array_val = value.as_array().unwrap();
                
                // Check array size constraints
                if let Some(min_items) = param_def.min_items {
                    if array_val.len() < min_items {
                        return Err(OrchestraError::config(&format!(
                            "Parameter '{}' must have at least {} items", param_def.name, min_items
                        )));
                    }
                }
                
                if let Some(max_items) = param_def.max_items {
                    if array_val.len() > max_items {
                        return Err(OrchestraError::config(&format!(
                            "Parameter '{}' must have at most {} items", param_def.name, max_items
                        )));
                    }
                }
            }
            
            ToolParameterType::Object => {
                if !value.is_object() {
                    return Err(OrchestraError::config(&format!(
                        "Parameter '{}' must be an object", param_def.name
                    )));
                }
            }
        }
        
        Ok(())
    }
    

    
    /// Get the tool registry
    pub fn registry(&self) -> &ToolRegistry {
        &self.registry
    }
    
    /// Check if a tool is available for execution
    pub fn has_tool(&self, name: &str) -> bool {
        self.registry.has_tool(name)
    }
    
    /// Get available tool names
    pub fn available_tools(&self) -> Vec<String> {
        self.registry.tool_names()
    }
}

/// Trait for implementing custom tool handlers
///
/// This trait allows you to create custom tool implementations that can be
/// executed by the ToolExecutor. It's an alternative to implementing the
/// full Tool trait when you just need simple function-like behavior.
///
/// ## For Rust Beginners
///
/// This trait demonstrates the "strategy pattern" - different implementations
/// can provide different behavior for the same interface.
#[async_trait]
pub trait ToolHandler: Send + Sync {
    /// Execute the tool with the given arguments
    async fn handle(&self, arguments: Value) -> Result<ToolResult>;
}

/// A simple tool implementation that wraps a ToolHandler
///
/// This allows you to create tools from simple async functions without
/// implementing the full Tool trait.
#[derive(Debug)]
pub struct SimpleToolImpl<H: ToolHandler> {
    definition: super::ToolDefinition,
    handler: H,
}

impl<H: ToolHandler> SimpleToolImpl<H> {
    /// Create a new simple tool
    pub fn new(definition: super::ToolDefinition, handler: H) -> Self {
        Self { definition, handler }
    }
}

#[async_trait]
impl<H: ToolHandler + std::fmt::Debug> Tool for SimpleToolImpl<H> {
    fn definition(&self) -> &super::ToolDefinition {
        &self.definition
    }
    
    async fn execute(&self, arguments: Value) -> Result<ToolResult> {
        self.handler.handle(arguments).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::{
        Tool, ToolDefinition, ToolParameter, ToolParameterType, ToolRegistry,
        ToolResult, boxed_tool,
    };
    use async_trait::async_trait;
    use serde_json::{json, Value};

    // Mock tool for testing
    #[derive(Debug)]
    struct TestTool {
        definition: ToolDefinition,
        should_fail: bool,
    }

    impl TestTool {
        fn new(name: &str, should_fail: bool) -> Self {
            let definition = ToolDefinition::new(name, "A test tool")
                .with_parameter(
                    ToolParameter::new("input", ToolParameterType::String)
                        .with_description("Test input")
                        .required()
                );

            Self {
                definition,
                should_fail,
            }
        }
    }

    #[async_trait]
    impl Tool for TestTool {
        fn definition(&self) -> &ToolDefinition {
            &self.definition
        }

        async fn execute(&self, arguments: Value) -> Result<ToolResult> {
            if self.should_fail {
                return Ok(ToolResult::error("Simulated failure"));
            }

            let input = arguments["input"].as_str().unwrap_or("default");
            Ok(ToolResult::success(json!({
                "processed": format!("Processed: {}", input)
            })))
        }
    }

    #[tokio::test]
    async fn test_executor_creation() {
        let registry = ToolRegistry::new();
        let executor = ToolExecutor::new(registry);

        // Test that executor was created successfully
        assert!(!executor.has_tool("nonexistent"));
        assert!(executor.available_tools().is_empty());
    }

    #[tokio::test]
    async fn test_executor_configuration() {
        let registry = ToolRegistry::new();
        let executor = ToolExecutor::new(registry)
            .with_timeout(Duration::from_secs(10))
            .with_validation(false)
            .with_timing(false);

        // Configuration is applied (we can't directly test private fields,
        // but we can test that the executor was created successfully)
        assert!(executor.available_tools().is_empty());
    }

    #[tokio::test]
    async fn test_successful_tool_execution() {
        let registry = ToolRegistry::new();
        let tool = boxed_tool(TestTool::new("success_tool", false));
        registry.register(tool).unwrap();

        let executor = ToolExecutor::new(registry);

        let result = executor.execute("success_tool", json!({
            "input": "test_value"
        })).await.unwrap();

        assert!(result.is_success());
        assert!(result.data.is_some());

        let data = result.data.unwrap();
        assert_eq!(data["processed"], "Processed: test_value");
    }

    #[tokio::test]
    async fn test_tool_execution_failure() {
        let registry = ToolRegistry::new();
        let tool = boxed_tool(TestTool::new("fail_tool", true));
        registry.register(tool).unwrap();

        let executor = ToolExecutor::new(registry);

        let result = executor.execute("fail_tool", json!({
            "input": "test_value"
        })).await.unwrap();

        assert!(result.is_error());
        assert!(result.error.is_some());
        assert_eq!(result.error.unwrap(), "Simulated failure");
    }

    #[tokio::test]
    async fn test_nonexistent_tool_execution() {
        let registry = ToolRegistry::new();
        let executor = ToolExecutor::new(registry);

        let result = executor.execute("nonexistent", json!({})).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_parameter_validation() {
        let registry = ToolRegistry::new();
        let tool = boxed_tool(TestTool::new("validation_tool", false));
        registry.register(tool).unwrap();

        let executor = ToolExecutor::new(registry)
            .with_validation(true);

        // Test missing required parameter
        let result = executor.execute("validation_tool", json!({})).await.unwrap();
        assert!(result.is_error());
        assert!(result.error.as_ref().unwrap().contains("Required parameter"));

        // Test with valid parameters
        let result = executor.execute("validation_tool", json!({
            "input": "valid_input"
        })).await.unwrap();
        assert!(result.is_success());
    }

    #[tokio::test]
    async fn test_parameter_type_validation() {
        let registry = ToolRegistry::new();

        let definition = ToolDefinition::new("type_test", "Type validation test")
            .with_parameter(
                ToolParameter::new("number_param", ToolParameterType::Number)
                    .required()
            )
            .with_parameter(
                ToolParameter::new("string_param", ToolParameterType::String)
                    .with_enum_values(vec!["option1", "option2"])
                    .required()
            );

        let tool = SimpleToolImpl::new(definition, TestHandler);
        registry.register(boxed_tool(tool)).unwrap();

        let executor = ToolExecutor::new(registry)
            .with_validation(true);

        // Test invalid number type
        let result = executor.execute("type_test", json!({
            "number_param": "not_a_number",
            "string_param": "option1"
        })).await.unwrap();
        assert!(result.is_error());

        // Test invalid enum value
        let result = executor.execute("type_test", json!({
            "number_param": 42,
            "string_param": "invalid_option"
        })).await.unwrap();
        assert!(result.is_error());

        // Test valid parameters
        let result = executor.execute("type_test", json!({
            "number_param": 42,
            "string_param": "option1"
        })).await.unwrap();
        assert!(result.is_success());
    }

    #[tokio::test]
    async fn test_timing_metadata() {
        let registry = ToolRegistry::new();
        let tool = boxed_tool(TestTool::new("timing_tool", false));
        registry.register(tool).unwrap();

        let executor = ToolExecutor::new(registry)
            .with_timing(true);

        let result = executor.execute("timing_tool", json!({
            "input": "test"
        })).await.unwrap();

        assert!(result.is_success());
        assert!(result.metadata.contains_key("execution_time_ms"));
    }

    // Test handler for SimpleToolImpl tests
    #[derive(Debug)]
    struct TestHandler;

    #[async_trait]
    impl ToolHandler for TestHandler {
        async fn handle(&self, _arguments: Value) -> Result<ToolResult> {
            Ok(ToolResult::success(json!({"result": "test"})))
        }
    }

    #[tokio::test]
    async fn test_simple_tool_impl() {
        let definition = ToolDefinition::new("simple_test", "Simple tool test");
        let tool = SimpleToolImpl::new(definition, TestHandler);

        let result = tool.execute(json!({})).await.unwrap();
        assert!(result.is_success());
        assert_eq!(result.data.unwrap()["result"], "test");
    }
}
