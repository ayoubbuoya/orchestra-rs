//! # Tool Calling Example
//!
//! This example demonstrates how to use the tool calling system in Orchestra-rs.
//! It shows how to define tools, register them, and use them with LLMs.
//!
//! ## What You'll Learn
//!
//! - How to define custom tools
//! - How to register tools in a registry
//! - How to use tools with LLMs
//! - How to handle tool execution results
//! - Best practices for tool calling workflows
//!
//! ## For Rust Beginners
//!
//! This example demonstrates several important Rust concepts:
//! - Async programming with `async`/`await`
//! - Error handling with `Result` types
//! - Trait implementation for custom types
//! - JSON serialization and deserialization
//! - Module organization and imports

use orchestra_core::{
    llm::LLM,
    tools::{
        Tool, ToolDefinition, ToolParameter, ToolParameterType, ToolRegistry,
        ToolResult, ToolExecutor, builtin::create_builtin_registry,
        boxed_tool,
    },
    error::Result,
};
use async_trait::async_trait;
use serde_json::{json, Value};

/// A custom weather tool that simulates getting weather information
///
/// This demonstrates how to create a custom tool that could integrate
/// with external APIs in a real application.
///
/// ## For Rust Beginners
///
/// - `#[derive(Debug)]` automatically implements the Debug trait
/// - The struct contains a `ToolDefinition` that describes the tool
/// - We implement the `Tool` trait to make this usable by the system
#[derive(Debug)]
struct WeatherTool {
    definition: ToolDefinition,
}

impl WeatherTool {
    /// Create a new weather tool with proper parameter definitions
    fn new() -> Self {
        let definition = ToolDefinition::new(
            "get_weather",
            "Get current weather information for a specified location"
        )
        .with_parameter(
            ToolParameter::new("location", ToolParameterType::String)
                .with_description("The city and country (e.g., 'Paris, France')")
                .required()
        )
        .with_parameter(
            ToolParameter::new("units", ToolParameterType::String)
                .with_description("Temperature units")
                .with_enum_values(vec!["celsius", "fahrenheit"])
                .with_default(json!("celsius"))
        );

        Self { definition }
    }
}

/// Implement the Tool trait for our WeatherTool
///
/// This is where we define how the tool actually works when called.
#[async_trait]
impl Tool for WeatherTool {
    fn definition(&self) -> &ToolDefinition {
        &self.definition
    }

    async fn execute(&self, arguments: Value) -> Result<ToolResult> {
        // Extract the location parameter
        let location = arguments["location"]
            .as_str()
            .ok_or_else(|| orchestra_core::error::OrchestraError::config("Missing location parameter"))?;

        // Extract the units parameter (with default)
        let units = arguments.get("units")
            .and_then(|v| v.as_str())
            .unwrap_or("celsius");

        // Simulate API call delay
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // Simulate weather data (in a real app, this would call a weather API)
        let (temp, temp_unit) = match units {
            "fahrenheit" => (72, "°F"),
            _ => (22, "°C"),
        };

        let weather_data = json!({
            "location": location,
            "temperature": temp,
            "temperature_unit": temp_unit,
            "condition": "sunny",
            "humidity": 65,
            "wind_speed": 10,
            "wind_unit": "km/h"
        });

        Ok(ToolResult::success(weather_data))
    }
}

/// Demonstrate basic tool definition and execution
async fn basic_tool_usage() -> Result<()> {
    println!("🔧 Basic Tool Usage");
    println!("===============================================================================");

    // Create a tool registry and add our custom weather tool
    let registry = ToolRegistry::new();
    registry.register(boxed_tool(WeatherTool::new()))?;

    // Create a tool executor
    let executor = ToolExecutor::new(registry);

    // Execute the weather tool
    let result = executor.execute("get_weather", json!({
        "location": "Tokyo, Japan",
        "units": "celsius"
    })).await?;

    println!("Tool execution result:");
    println!("Status: {:?}", result.status);
    if let Some(data) = &result.data {
        println!("Data: {}", serde_json::to_string_pretty(data)?);
    }
    if let Some(duration) = result.duration_ms() {
        println!("Execution time: {}ms", duration);
    }

    println!();
    Ok(())
}

/// Demonstrate using built-in tools
async fn builtin_tools_demo() -> Result<()> {
    println!("🛠️  Built-in Tools Demo");
    println!("===============================================================================");

    // Create a registry with built-in tools
    let registry = create_builtin_registry();

    println!("Available built-in tools:");
    for tool_name in registry.tool_names() {
        if let Some(def) = registry.get_tool_definition(&tool_name) {
            println!("  - {}: {}", def.name, def.description);
        }
    }
    println!();

    // Create an executor and test the calculator
    let executor = ToolExecutor::new(registry);

    // Test calculator tool
    println!("Testing calculator tool:");
    let calc_result = executor.execute("calculator", json!({
        "operation": "multiply",
        "a": 15,
        "b": 7
    })).await?;

    if calc_result.is_success() {
        if let Some(data) = &calc_result.data {
            println!("Calculator result: {}", serde_json::to_string_pretty(data)?);
        }
    }

    // Test timestamp tool
    println!("\nTesting timestamp tool:");
    let time_result = executor.execute("get_timestamp", json!({
        "format": "human"
    })).await?;

    if time_result.is_success() {
        if let Some(data) = &time_result.data {
            println!("Timestamp result: {}", serde_json::to_string_pretty(data)?);
        }
    }

    println!();
    Ok(())
}

/// Demonstrate LLM integration with tools (simulation)
///
/// Note: This example simulates LLM responses since we don't have a real API key
async fn llm_tool_integration_demo() -> Result<()> {
    println!("🤖 LLM Tool Integration Demo");
    println!("===============================================================================");

    // Create an LLM instance
    let llm = LLM::gemini("gemini-2.5-flash");

    // Check if the provider supports tools
    if llm.supports_tools() {
        println!("✅ Provider supports tool calling");
    } else {
        println!("❌ Provider does not support tool calling");
        return Ok(());
    }

    // Create tools for the LLM
    let weather_tool = WeatherTool::new().definition().clone();
    let tools = vec![weather_tool];

    println!("Tools available to LLM:");
    for tool in &tools {
        println!("  - {}: {}", tool.name, tool.description);
    }

    // In a real scenario, you would do:
    // let response = llm.prompt_with_tools(
    //     "What's the weather like in London?",
    //     tools
    // ).await?;

    // For this demo, we'll simulate what the response might look like
    println!("\n📝 Simulated LLM Response:");
    println!("The LLM would analyze the prompt 'What's the weather like in London?'");
    println!("and decide to call the get_weather tool with appropriate parameters.");
    
    // Simulate tool execution
    let registry = ToolRegistry::new();
    registry.register(boxed_tool(WeatherTool::new()))?;
    let executor = llm.create_tool_executor(registry);

    let simulated_tool_call = json!({
        "location": "London, UK",
        "units": "celsius"
    });

    println!("\n🔧 Executing tool call:");
    println!("Tool: get_weather");
    println!("Arguments: {}", serde_json::to_string_pretty(&simulated_tool_call)?);

    let result = executor.execute("get_weather", simulated_tool_call).await?;
    
    if result.is_success() {
        println!("\n✅ Tool execution successful:");
        if let Some(data) = &result.data {
            println!("{}", serde_json::to_string_pretty(data)?);
        }
    }

    println!();
    Ok(())
}

/// Demonstrate error handling in tool execution
async fn error_handling_demo() -> Result<()> {
    println!("⚠️  Error Handling Demo");
    println!("===============================================================================");

    let registry = create_builtin_registry();
    let executor = ToolExecutor::new(registry);

    // Test with invalid parameters
    println!("Testing with invalid parameters:");
    let result = executor.execute("calculator", json!({
        "operation": "divide",
        "a": 10,
        "b": 0  // Division by zero
    })).await?;

    if result.is_error() {
        println!("❌ Tool execution failed (as expected):");
        if let Some(error) = &result.error {
            println!("Error: {}", error);
        }
    }

    // Test with missing parameters
    println!("\nTesting with missing required parameters:");
    let result2 = executor.execute("calculator", json!({
        "operation": "add"
        // Missing 'a' and 'b' parameters
    })).await?;

    if result2.is_error() {
        println!("❌ Tool execution failed (as expected):");
        if let Some(error) = &result2.error {
            println!("Error: {}", error);
        }
    }

    // Test with unknown tool
    println!("\nTesting with unknown tool:");
    let result3 = executor.execute("nonexistent_tool", json!({})).await;

    match result3 {
        Err(e) => println!("❌ Tool not found (as expected): {}", e),
        Ok(_) => println!("Unexpected success"),
    }

    println!();
    Ok(())
}

/// Main function that runs all the demos
#[tokio::main]
async fn main() -> Result<()> {
    println!("🎯 Orchestra-rs Tool Calling Examples");
    println!("===============================================================================");
    println!("This example demonstrates the tool calling system in Orchestra-rs.");
    println!("Each section shows different aspects of working with tools.\n");

    // Run all the demo functions
    basic_tool_usage().await?;
    builtin_tools_demo().await?;
    llm_tool_integration_demo().await?;
    error_handling_demo().await?;

    println!("🎉 All examples completed successfully!");
    println!("\n💡 Next Steps:");
    println!("  1. Try creating your own custom tools");
    println!("  2. Set up a real API key to test with actual LLMs");
    println!("  3. Explore the built-in tools and their parameters");
    println!("  4. Build a complete tool calling workflow for your use case");

    Ok(())
}
