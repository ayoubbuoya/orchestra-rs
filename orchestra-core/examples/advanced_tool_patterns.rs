//! # Advanced Tool Calling Patterns
//!
//! This example demonstrates advanced patterns and best practices for tool calling
//! in Orchestra-rs, including tool composition, error recovery, and workflow management.
//!
//! ## Advanced Concepts Covered
//!
//! - Tool composition and chaining
//! - Error recovery strategies
//! - Tool result validation
//! - Workflow orchestration
//! - Performance optimization
//! - Tool categorization and discovery

use orchestra_core::{
    tools::{
        Tool, ToolDefinition, ToolParameter, ToolParameterType, ToolRegistry,
        ToolResult, ToolExecutor, ToolHandler,
        execution::SimpleToolImpl,
        result::{ToolError, ToolErrorType},
        boxed_tool,
    },
    error::Result,
};
use async_trait::async_trait;
use serde_json::{json, Value};

/// A tool that validates and formats email addresses
///
/// This demonstrates input validation and data transformation patterns.
#[derive(Debug)]
struct EmailValidatorTool {
    definition: ToolDefinition,
}

impl EmailValidatorTool {
    fn new() -> Self {
        let definition = ToolDefinition::new(
            "validate_email",
            "Validate and format email addresses"
        )
        .with_parameter(
            ToolParameter::new("email", ToolParameterType::String)
                .with_description("The email address to validate")
                .with_length_range(Some(5), Some(254))
                .required()
        )
        .with_parameter(
            ToolParameter::new("normalize", ToolParameterType::Boolean)
                .with_description("Whether to normalize the email format")
                .with_default(json!(true))
        );

        Self { definition }
    }

    /// Simple email validation (in a real app, use a proper email validation library)
    fn is_valid_email(email: &str) -> bool {
        email.contains('@') && email.contains('.') && email.len() >= 5
    }

    /// Normalize email format
    fn normalize_email(email: &str) -> String {
        email.trim().to_lowercase()
    }
}

#[async_trait]
impl Tool for EmailValidatorTool {
    fn definition(&self) -> &ToolDefinition {
        &self.definition
    }

    async fn execute(&self, arguments: Value) -> Result<ToolResult> {
        let email = arguments["email"]
            .as_str()
            .ok_or_else(|| orchestra_core::error::OrchestraError::config("Missing email parameter"))?;

        let normalize = arguments.get("normalize")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        // Validate email
        if !Self::is_valid_email(email) {
            return Ok(ToolResult::error_with_details(
                "Invalid email format",
                ToolError::new(ToolErrorType::InvalidInput, "Email validation failed")
                    .with_context("email", json!(email))
            ));
        }

        // Normalize if requested
        let processed_email = if normalize {
            Self::normalize_email(email)
        } else {
            email.to_string()
        };

        Ok(ToolResult::success(json!({
            "original": email,
            "processed": processed_email,
            "is_valid": true,
            "normalized": normalize
        })))
    }
}

/// A tool handler that uses a closure for simple operations
///
/// This demonstrates the ToolHandler trait for lightweight tool implementations.
struct ClosureHandler<F>
where
    F: Fn(Value) -> Result<ToolResult> + Send + Sync,
{
    handler: F,
}

impl<F> std::fmt::Debug for ClosureHandler<F>
where
    F: Fn(Value) -> Result<ToolResult> + Send + Sync,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ClosureHandler")
            .field("handler", &"<closure>")
            .finish()
    }
}

impl<F> ClosureHandler<F>
where
    F: Fn(Value) -> Result<ToolResult> + Send + Sync,
{
    fn new(handler: F) -> Self {
        Self { handler }
    }
}

#[async_trait]
impl<F> ToolHandler for ClosureHandler<F>
where
    F: Fn(Value) -> Result<ToolResult> + Send + Sync,
{
    async fn handle(&self, arguments: Value) -> Result<ToolResult> {
        (self.handler)(arguments)
    }
}

/// Demonstrate tool composition and chaining
async fn tool_composition_demo() -> Result<()> {
    println!("🔗 Tool Composition Demo");
    println!("===============================================================================");

    let registry = ToolRegistry::new();

    // Register the email validator
    registry.register(boxed_tool(EmailValidatorTool::new()))?;

    // Create a simple text processor using the closure handler
    let text_processor_def = ToolDefinition::new(
        "process_text",
        "Process text with various transformations"
    )
    .with_parameter(
        ToolParameter::new("text", ToolParameterType::String)
            .with_description("The text to process")
            .required()
    )
    .with_parameter(
        ToolParameter::new("operation", ToolParameterType::String)
            .with_description("The operation to perform")
            .with_enum_values(vec!["uppercase", "lowercase", "reverse", "word_count"])
            .required()
    );

    let text_processor = SimpleToolImpl::new(
        text_processor_def,
        ClosureHandler::new(|args| {
            let text = args["text"].as_str().unwrap_or("");
            let operation = args["operation"].as_str().unwrap_or("lowercase");

            let result = match operation {
                "uppercase" => text.to_uppercase(),
                "lowercase" => text.to_lowercase(),
                "reverse" => text.chars().rev().collect(),
                "word_count" => text.split_whitespace().count().to_string(),
                _ => text.to_string(),
            };

            Ok(ToolResult::success(json!({
                "original": text,
                "operation": operation,
                "result": result
            })))
        })
    );

    registry.register(boxed_tool(text_processor))?;

    let executor = ToolExecutor::new(registry);

    // Demonstrate tool chaining: process email, then process the result
    println!("Step 1: Validate and normalize email");
    let email_result = executor.execute("validate_email", json!({
        "email": "  USER@EXAMPLE.COM  ",
        "normalize": true
    })).await?;

    if let Some(email_data) = &email_result.data {
        println!("Email processing result: {}", serde_json::to_string_pretty(email_data)?);

        // Extract the processed email and use it in the next tool
        if let Some(processed_email) = email_data["processed"].as_str() {
            println!("\nStep 2: Process the normalized email");
            let text_result = executor.execute("process_text", json!({
                "text": processed_email,
                "operation": "uppercase"
            })).await?;

            if let Some(text_data) = &text_result.data {
                println!("Text processing result: {}", serde_json::to_string_pretty(text_data)?);
            }
        }
    }

    println!();
    Ok(())
}

/// Demonstrate error recovery and retry strategies
async fn error_recovery_demo() -> Result<()> {
    println!("🔄 Error Recovery Demo");
    println!("===============================================================================");

    let registry = ToolRegistry::new();
    
    // Create a tool that sometimes fails (simulating network issues)
    let unreliable_tool_def = ToolDefinition::new(
        "unreliable_service",
        "A service that sometimes fails to simulate real-world conditions"
    )
    .with_parameter(
        ToolParameter::new("data", ToolParameterType::String)
            .with_description("Data to process")
            .required()
    );

    let unreliable_tool = SimpleToolImpl::new(
        unreliable_tool_def,
        ClosureHandler::new(|args| {
            let data = args["data"].as_str().unwrap_or("");
            
            // Simulate failure based on data content
            if data.contains("fail") {
                return Ok(ToolResult::error_with_details(
                    "Service temporarily unavailable",
                    ToolError::new(ToolErrorType::ExternalService, "Simulated service failure")
                        .retryable()
                ));
            }

            Ok(ToolResult::success(json!({
                "processed_data": format!("Processed: {}", data),
                "timestamp": std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs()
            })))
        })
    );

    registry.register(boxed_tool(unreliable_tool))?;
    let executor = ToolExecutor::new(registry);

    // Demonstrate retry logic
    async fn execute_with_retry(
        executor: &ToolExecutor,
        tool_name: &str,
        args: Value,
        max_retries: u32,
    ) -> Result<ToolResult> {
        for attempt in 1..=max_retries {
            println!("Attempt {}/{}", attempt, max_retries);
            
            let result = executor.execute(tool_name, args.clone()).await?;
            
            if result.is_success() {
                return Ok(result);
            }
            
            if let Some(error_details) = &result.error_details {
                if error_details.retryable && attempt < max_retries {
                    println!("  ❌ Failed (retryable): {}", result.error.as_ref().unwrap_or(&"Unknown error".to_string()));
                    tokio::time::sleep(tokio::time::Duration::from_millis(100 * attempt as u64)).await;
                    continue;
                }
            }
            
            return Ok(result);
        }
        
        Ok(ToolResult::error("Max retries exceeded"))
    }

    // Test with data that will fail
    println!("Testing with failing data:");
    let result = execute_with_retry(&executor, "unreliable_service", json!({
        "data": "this will fail"
    }), 3).await?;

    if result.is_error() {
        println!("❌ Final result: {}", result.error.as_ref().unwrap_or(&"Unknown error".to_string()));
    }

    // Test with data that will succeed
    println!("\nTesting with successful data:");
    let result = execute_with_retry(&executor, "unreliable_service", json!({
        "data": "this will succeed"
    }), 3).await?;

    if result.is_success() {
        println!("✅ Success: {}", serde_json::to_string_pretty(result.data.as_ref().unwrap())?);
    }

    println!();
    Ok(())
}

/// Demonstrate tool categorization and discovery
async fn tool_discovery_demo() -> Result<()> {
    println!("🔍 Tool Discovery Demo");
    println!("===============================================================================");

    let registry = ToolRegistry::new();

    // Register tools in different categories
    registry.register(boxed_tool(EmailValidatorTool::new()))?;
    
    // Add tools to categories
    registry.add_to_category("validation", "validate_email")?;
    registry.add_to_category("text_processing", "validate_email")?;

    // Show category organization
    println!("Tool Categories:");
    for category in registry.category_names() {
        println!("  📁 {}", category);
        for tool_name in registry.tools_in_category(&category) {
            if let Some(def) = registry.get_tool_definition(&tool_name) {
                println!("    🔧 {}: {}", def.name, def.description);
            }
        }
    }

    // Generate JSON schema for all tools
    println!("\n📋 Tool Schema (for LLM consumption):");
    let schema = registry.to_json_schema();
    println!("{}", serde_json::to_string_pretty(&schema)?);

    println!();
    Ok(())
}

/// Demonstrate performance monitoring and optimization
async fn performance_demo() -> Result<()> {
    println!("⚡ Performance Monitoring Demo");
    println!("===============================================================================");

    let registry = ToolRegistry::new();
    registry.register(boxed_tool(EmailValidatorTool::new()))?;

    // Create executor with timing enabled
    let executor = ToolExecutor::new(registry)
        .with_timing(true)
        .with_timeout(std::time::Duration::from_secs(5));

    // Execute multiple tools and collect timing data
    let mut execution_times = Vec::new();

    for i in 1..=5 {
        let start = std::time::Instant::now();
        
        let result = executor.execute("validate_email", json!({
            "email": format!("user{}@example.com", i),
            "normalize": true
        })).await?;

        let duration = start.elapsed();
        execution_times.push(duration);

        if let Some(metadata) = result.metadata.get("execution_time_ms") {
            println!("Execution {}: {}ms (internal timing)", i, metadata);
        }
        println!("Execution {}: {}ms (external timing)", i, duration.as_millis());
    }

    // Calculate statistics
    let avg_time = execution_times.iter().sum::<std::time::Duration>() / execution_times.len() as u32;
    let min_time = execution_times.iter().min().unwrap();
    let max_time = execution_times.iter().max().unwrap();

    println!("\n📊 Performance Statistics:");
    println!("  Average: {}ms", avg_time.as_millis());
    println!("  Minimum: {}ms", min_time.as_millis());
    println!("  Maximum: {}ms", max_time.as_millis());

    println!();
    Ok(())
}

/// Main function that runs all advanced demos
#[tokio::main]
async fn main() -> Result<()> {
    println!("🚀 Orchestra-rs Advanced Tool Calling Patterns");
    println!("===============================================================================");
    println!("This example demonstrates advanced patterns for tool calling in Orchestra-rs.\n");

    tool_composition_demo().await?;
    error_recovery_demo().await?;
    tool_discovery_demo().await?;
    performance_demo().await?;

    println!("🎉 All advanced examples completed successfully!");
    println!("\n💡 Key Takeaways:");
    println!("  1. Tools can be composed and chained for complex workflows");
    println!("  2. Error recovery strategies improve reliability");
    println!("  3. Tool categorization helps with discovery and organization");
    println!("  4. Performance monitoring is crucial for production systems");
    println!("  5. The ToolHandler trait provides flexibility for simple tools");

    Ok(())
}
