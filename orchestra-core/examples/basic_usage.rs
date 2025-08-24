use orchestra_core::{
    error::Result, llm::LLM, messages::Message, model::ModelConfig,
    providers::types::ProviderSource,
};

/// This example demonstrates basic usage of Orchestra-rs with the Gemini provider.
///
/// To run this example:
/// 1. Set your Gemini API key: export GEMINI_API_KEY="your-api-key-here"
/// 2. Run: cargo run --example basic_usage
#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging (optional)
    env_logger::init();

    println!("🎼 Orchestra-rs Basic Usage Example\n");

    // Example 1: Simple prompt
    simple_prompt().await?;

    // Example 2: Chat with history
    chat_with_history().await?;

    // Example 3: Custom configuration
    custom_configuration().await?;

    // Example 4: Using presets
    using_presets().await?;

    println!("\n✅ All examples completed successfully!");
    Ok(())
}

/// Example 1: Simple prompt
async fn simple_prompt() -> Result<()> {
    println!("📝 Example 1: Simple Prompt");
    println!("===============================================================================");

    // Create an LLM instance with Gemini
    let llm = LLM::gemini("gemini-2.5-flash");

    // Send a simple prompt.
    let response = llm
        .prompt("Hello! Can you tell me a fun fact about Rust programming language?")
        .await?;

    println!("🤖 Response: {}\n", response.text);
    Ok(())
}

/// Example 2: Chat with conversation history
async fn chat_with_history() -> Result<()> {
    println!("💬 Example 2: Chat with History");
    println!("===============================================================================");

    let llm = LLM::gemini("gemini-2.5-flash");

    // Build conversation history
    let history = vec![
        Message::human("Hi! I'm learning Rust and I'm confused about ownership."),
        Message::assistant(
            "Hello! I'd be happy to help you understand Rust ownership. It's one of Rust's most important concepts. What specific aspect of ownership would you like me to explain?",
        ),
        Message::human("What's the difference between moving and borrowing?"),
        Message::assistant(
            "Great question! Moving transfers ownership of a value, while borrowing allows temporary access without taking ownership. When you move a value, the original variable can no longer be used. When you borrow, you get a reference that allows you to use the value without owning it.",
        ),
    ];

    // Continue the conversation
    let response = llm
        .chat(
            Message::human("Can you give me a simple code example of both?"),
            history,
        )
        .await?;

    println!("🤖 Response: {}\n", response.text);
    Ok(())
}

/// Example 3: Custom configuration
async fn custom_configuration() -> Result<()> {
    println!("⚙️  Example 3: Custom Configuration");
    println!("===============================================================================");

    // Create a custom model configuration
    let config = ModelConfig::new("gemini-2.5-flash")
        .with_system_instruction("You are a helpful Rust programming tutor. Always provide practical examples and explain concepts clearly.")
        .with_temperature(0.7)?
        .with_top_p(0.9)?
        .with_max_tokens(8000);

    // Create LLM with custom configuration
    let llm =
        LLM::new(ProviderSource::Gemini, "gemini-2.5-flash".to_string()).with_custom_config(config);

    let response = llm
        .prompt("Explain Rust's Result type and how to use it")
        .await?;
    println!("🤖 Response: {}\n", response.text);
    Ok(())
}

/// Example 4: Using configuration presets
async fn using_presets() -> Result<()> {
    println!("🎯 Example 4: Using Presets");
    println!("===============================================================================");

    // Conservative settings (lower temperature, more focused)
    println!("🔒 Conservative preset (focused, deterministic):");
    let conservative_llm =
        LLM::conservative(ProviderSource::Gemini, "gemini-2.5-flash".to_string());
    let conservative_response = conservative_llm
        .prompt("Write a one-sentence summary of what Rust is.")
        .await?;
    println!("Response: {}\n", conservative_response.text);

    // Creative settings (higher temperature, more diverse)
    println!("🎨 Creative preset (diverse, imaginative):");
    let creative_llm = LLM::creative(ProviderSource::Gemini, "gemini-2.5-flash".to_string());
    let creative_response = creative_llm
        .prompt("Write a creative analogy to explain Rust's ownership system.")
        .await?;
    println!("Response: {}\n", creative_response.text);

    // Balanced settings (moderate temperature)
    println!("⚖️  Balanced preset (moderate creativity):");
    let balanced_llm = LLM::balanced(ProviderSource::Gemini, "gemini-2.5-flash".to_string());
    let balanced_response = balanced_llm
        .prompt("Explain the benefits of using Rust for systems programming.")
        .await?;
    println!("Response: {}\n", balanced_response.text);

    Ok(())
}

/// Helper function to demonstrate provider capabilities
#[allow(dead_code)]
async fn provider_capabilities() -> Result<()> {
    println!("🔍 Provider Capabilities");
    println!("===============================================================================");

    let llm = LLM::gemini("gemini-2.5-flash");

    println!("Provider name: {}", llm.provider_name());
    println!("Supports streaming: {}", llm.supports_streaming());
    println!("Supports tools: {}", llm.supports_tools());
    println!("Model name: {}", llm.get_model_name());

    Ok(())
}
