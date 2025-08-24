//! # Tool Registry
//!
//! This module provides a registry system for managing available tools.
//! It allows registering, discovering, and organizing tools in a type-safe way.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use crate::error::{OrchestraError, Result};
use super::{BoxedTool, ToolDefinition};

/// A registry for managing available tools
///
/// The tool registry is the central place where all available tools are stored
/// and managed. It provides thread-safe access to tools and their definitions.
///
/// ## For Rust Beginners
///
/// - `Arc<RwLock<T>>` provides thread-safe shared access to data
/// - `Arc` (Atomically Reference Counted) allows multiple owners
/// - `RwLock` allows multiple readers OR one writer (but not both)
/// - This pattern is common in Rust for shared mutable state
#[derive(Debug, Clone)]
pub struct ToolRegistry {
    /// The tools stored in the registry
    /// We use Arc<RwLock<>> to allow safe concurrent access
    tools: Arc<RwLock<HashMap<String, BoxedTool>>>,
    
    /// Metadata about tool categories and organization
    categories: Arc<RwLock<HashMap<String, Vec<String>>>>,
}

impl ToolRegistry {
    /// Create a new empty tool registry
    ///
    /// # Example
    /// ```rust
    /// use orchestra_core::tools::ToolRegistry;
    ///
    /// let registry = ToolRegistry::new();
    /// ```
    pub fn new() -> Self {
        Self {
            tools: Arc::new(RwLock::new(HashMap::new())),
            categories: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Register a tool in the registry
    ///
    /// This adds a tool to the registry, making it available for use.
    /// The tool's name must be unique within the registry.
    ///
    /// # Arguments
    /// * `tool` - The tool to register
    ///
    /// # Returns
    /// `Ok(())` if successful, or an error if the tool name is already taken
    ///
    /// # Example
    /// ```rust,no_run
    /// use orchestra_core::tools::{ToolRegistry, boxed_tool};
    /// // Assuming you have a tool implementation called `MyTool`
    /// // let my_tool = MyTool::new();
    /// // let registry = ToolRegistry::new();
    /// // registry.register(boxed_tool(my_tool))?;
    /// ```
    pub fn register(&self, tool: BoxedTool) -> Result<()> {
        let tool_name = tool.definition().name.clone();
        
        // Validate the tool definition before registering
        tool.definition().validate()?;
        
        // Get a write lock on the tools map
        let mut tools = self.tools.write().map_err(|_| {
            OrchestraError::generic("Failed to acquire write lock on tool registry")
        })?;
        
        // Check if tool name is already taken
        if tools.contains_key(&tool_name) {
            return Err(OrchestraError::config(&format!(
                "Tool with name '{}' is already registered", tool_name
            )));
        }
        
        // Insert the tool
        tools.insert(tool_name, tool);
        
        Ok(())
    }
    
    /// Get a tool by name
    ///
    /// This returns a reference to a tool if it exists in the registry.
    /// Note that this returns a reference, not ownership of the tool.
    ///
    /// # Arguments
    /// * `name` - The name of the tool to retrieve
    ///
    /// # Returns
    /// The tool definition if found, None otherwise
    pub fn get_tool_definition(&self, name: &str) -> Option<ToolDefinition> {
        let tools = self.tools.read().ok()?;
        tools.get(name).map(|tool| tool.definition().clone())
    }
    
    /// Check if a tool exists in the registry
    pub fn has_tool(&self, name: &str) -> bool {
        self.tools.read()
            .map(|tools| tools.contains_key(name))
            .unwrap_or(false)
    }
    
    /// Get all tool names in the registry
    pub fn tool_names(&self) -> Vec<String> {
        self.tools.read()
            .map(|tools| tools.keys().cloned().collect())
            .unwrap_or_default()
    }
    
    /// Get all tool definitions in the registry
    pub fn tool_definitions(&self) -> Vec<ToolDefinition> {
        self.tools.read()
            .map(|tools| {
                tools.values()
                    .map(|tool| tool.definition().clone())
                    .collect()
            })
            .unwrap_or_default()
    }
    
    /// Remove a tool from the registry
    ///
    /// # Arguments
    /// * `name` - The name of the tool to remove
    ///
    /// # Returns
    /// `true` if the tool was removed, `false` if it wasn't found
    pub fn unregister(&self, name: &str) -> bool {
        self.tools.write()
            .map(|mut tools| tools.remove(name).is_some())
            .unwrap_or(false)
    }
    
    /// Clear all tools from the registry
    pub fn clear(&self) {
        if let Ok(mut tools) = self.tools.write() {
            tools.clear();
        }
        if let Ok(mut categories) = self.categories.write() {
            categories.clear();
        }
    }
    
    /// Get the number of registered tools
    pub fn len(&self) -> usize {
        self.tools.read()
            .map(|tools| tools.len())
            .unwrap_or(0)
    }
    
    /// Check if the registry is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    
    /// Add a tool to a category
    ///
    /// Categories help organize tools into logical groups.
    /// A tool can belong to multiple categories.
    ///
    /// # Arguments
    /// * `category` - The category name
    /// * `tool_name` - The name of the tool to add to the category
    pub fn add_to_category<S: Into<String>>(&self, category: S, tool_name: S) -> Result<()> {
        let category = category.into();
        let tool_name = tool_name.into();
        
        // Check if the tool exists
        if !self.has_tool(&tool_name) {
            return Err(OrchestraError::config(&format!(
                "Tool '{}' not found in registry", tool_name
            )));
        }
        
        let mut categories = self.categories.write().map_err(|_| {
            OrchestraError::generic("Failed to acquire write lock on categories")
        })?;
        
        categories.entry(category)
            .or_insert_with(Vec::new)
            .push(tool_name);
        
        Ok(())
    }
    
    /// Get all tools in a category
    pub fn tools_in_category(&self, category: &str) -> Vec<String> {
        self.categories.read()
            .map(|categories| {
                categories.get(category)
                    .cloned()
                    .unwrap_or_default()
            })
            .unwrap_or_default()
    }
    
    /// Get all category names
    pub fn category_names(&self) -> Vec<String> {
        self.categories.read()
            .map(|categories| categories.keys().cloned().collect())
            .unwrap_or_default()
    }
    
    /// Get tool definitions for a specific category
    pub fn category_definitions(&self, category: &str) -> Vec<ToolDefinition> {
        let tool_names = self.tools_in_category(category);
        tool_names.into_iter()
            .filter_map(|name| self.get_tool_definition(&name))
            .collect()
    }
    
    /// Create a registry with commonly used tools
    ///
    /// This is a convenience method that creates a registry pre-populated
    /// with useful built-in tools.
    pub fn with_builtin_tools() -> Self {
        super::builtin::create_builtin_registry()
    }
    
    /// Export all tool definitions as JSON schema
    ///
    /// This creates a JSON representation of all tools that can be sent
    /// to LLMs to describe available functionality.
    pub fn to_json_schema(&self) -> serde_json::Value {
        let definitions = self.tool_definitions();
        
        let tools: Vec<serde_json::Value> = definitions.into_iter()
            .map(|def| {
                serde_json::json!({
                    "type": "function",
                    "function": {
                        "name": def.name,
                        "description": def.description,
                        "parameters": def.to_json_schema()
                    }
                })
            })
            .collect();
        
        serde_json::json!({
            "tools": tools,
            "tool_choice": "auto"
        })
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// Internal helper to execute tools
// This is kept separate to avoid exposing the actual tool instances
impl ToolRegistry {
    /// Execute a tool by name (internal method)
    ///
    /// This method is used internally by the execution engine.
    /// It's not exposed publicly to maintain encapsulation.
    pub(crate) async fn execute_tool(
        &self,
        name: &str,
        arguments: serde_json::Value,
    ) -> Result<super::result::ToolResult> {
        // Get a read lock and execute the tool
        let tools = self.tools.read().map_err(|_| {
            OrchestraError::generic("Failed to acquire read lock on tool registry")
        })?;

        let tool = tools.get(name)
            .ok_or_else(|| OrchestraError::config(&format!("Tool '{}' not found", name)))?;

        // Execute the tool
        tool.execute(arguments).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::{Tool, ToolDefinition, ToolParameter, ToolParameterType, ToolResult};
    use async_trait::async_trait;
    use serde_json::Value;

    // Mock tool for testing
    #[derive(Debug)]
    struct MockTool {
        definition: ToolDefinition,
    }

    impl MockTool {
        fn new(name: &str, description: &str) -> Self {
            Self {
                definition: ToolDefinition::new(name, description),
            }
        }
    }

    #[async_trait]
    impl Tool for MockTool {
        fn definition(&self) -> &ToolDefinition {
            &self.definition
        }

        async fn execute(&self, _arguments: Value) -> crate::error::Result<ToolResult> {
            Ok(ToolResult::success(serde_json::json!({"result": "mock"})))
        }
    }

    #[test]
    fn test_registry_creation() {
        let registry = ToolRegistry::new();
        assert_eq!(registry.len(), 0);
        assert!(registry.is_empty());
        assert!(registry.tool_names().is_empty());
    }

    #[test]
    fn test_tool_registration() {
        let registry = ToolRegistry::new();
        let tool = super::super::boxed_tool(MockTool::new("test_tool", "A test tool"));

        // Register tool
        assert!(registry.register(tool).is_ok());
        assert_eq!(registry.len(), 1);
        assert!(!registry.is_empty());
        assert!(registry.has_tool("test_tool"));

        // Check tool names
        let names = registry.tool_names();
        assert_eq!(names.len(), 1);
        assert!(names.contains(&"test_tool".to_string()));
    }

    #[test]
    fn test_duplicate_tool_registration() {
        let registry = ToolRegistry::new();
        let tool1 = super::super::boxed_tool(MockTool::new("duplicate", "First tool"));
        let tool2 = super::super::boxed_tool(MockTool::new("duplicate", "Second tool"));

        // First registration should succeed
        assert!(registry.register(tool1).is_ok());

        // Second registration with same name should fail
        assert!(registry.register(tool2).is_err());
        assert_eq!(registry.len(), 1);
    }

    #[test]
    fn test_tool_retrieval() {
        let registry = ToolRegistry::new();
        let tool = super::super::boxed_tool(MockTool::new("retrieval_test", "Test retrieval"));

        registry.register(tool).unwrap();

        // Test getting tool definition
        let definition = registry.get_tool_definition("retrieval_test");
        assert!(definition.is_some());
        let def = definition.unwrap();
        assert_eq!(def.name, "retrieval_test");
        assert_eq!(def.description, "Test retrieval");

        // Test non-existent tool
        assert!(registry.get_tool_definition("nonexistent").is_none());
    }

    #[test]
    fn test_tool_unregistration() {
        let registry = ToolRegistry::new();
        let tool = super::super::boxed_tool(MockTool::new("removable", "Will be removed"));

        registry.register(tool).unwrap();
        assert!(registry.has_tool("removable"));

        // Remove tool
        assert!(registry.unregister("removable"));
        assert!(!registry.has_tool("removable"));
        assert_eq!(registry.len(), 0);

        // Try to remove non-existent tool
        assert!(!registry.unregister("nonexistent"));
    }

    #[test]
    fn test_registry_clear() {
        let registry = ToolRegistry::new();

        // Add multiple tools
        registry.register(super::super::boxed_tool(MockTool::new("tool1", "First"))).unwrap();
        registry.register(super::super::boxed_tool(MockTool::new("tool2", "Second"))).unwrap();
        assert_eq!(registry.len(), 2);

        // Clear registry
        registry.clear();
        assert_eq!(registry.len(), 0);
        assert!(registry.is_empty());
    }

    #[test]
    fn test_tool_categories() {
        let registry = ToolRegistry::new();
        let tool = super::super::boxed_tool(MockTool::new("categorized", "A categorized tool"));

        registry.register(tool).unwrap();

        // Add to categories
        assert!(registry.add_to_category("math", "categorized").is_ok());
        assert!(registry.add_to_category("utility", "categorized").is_ok());

        // Check categories
        let categories = registry.category_names();
        assert!(categories.contains(&"math".to_string()));
        assert!(categories.contains(&"utility".to_string()));

        // Check tools in category
        let math_tools = registry.tools_in_category("math");
        assert_eq!(math_tools.len(), 1);
        assert!(math_tools.contains(&"categorized".to_string()));

        // Try to add non-existent tool to category
        assert!(registry.add_to_category("test", "nonexistent").is_err());
    }

    #[test]
    fn test_tool_definitions_retrieval() {
        let registry = ToolRegistry::new();

        registry.register(super::super::boxed_tool(MockTool::new("tool1", "First tool"))).unwrap();
        registry.register(super::super::boxed_tool(MockTool::new("tool2", "Second tool"))).unwrap();

        let definitions = registry.tool_definitions();
        assert_eq!(definitions.len(), 2);

        let names: Vec<String> = definitions.iter().map(|d| d.name.clone()).collect();
        assert!(names.contains(&"tool1".to_string()));
        assert!(names.contains(&"tool2".to_string()));
    }

    #[test]
    fn test_json_schema_export() {
        let registry = ToolRegistry::new();

        let tool_def = ToolDefinition::new("schema_test", "Test schema generation")
            .with_parameter(
                ToolParameter::new("param1", ToolParameterType::String)
                    .required()
            );

        let tool = super::super::boxed_tool(MockTool {
            definition: tool_def,
        });

        registry.register(tool).unwrap();

        let schema = registry.to_json_schema();
        assert!(schema["tools"].is_array());

        let tools = schema["tools"].as_array().unwrap();
        assert_eq!(tools.len(), 1);

        let tool_schema = &tools[0];
        assert_eq!(tool_schema["type"], "function");
        assert!(tool_schema["function"].is_object());

        let function = &tool_schema["function"];
        assert_eq!(function["name"], "schema_test");
        assert_eq!(function["description"], "Test schema generation");
        assert!(function["parameters"].is_object());
    }

    #[test]
    fn test_builtin_registry() {
        let registry = ToolRegistry::with_builtin_tools();

        // Should have some built-in tools
        assert!(registry.len() > 0);

        // Check for expected built-in tools
        assert!(registry.has_tool("calculator"));
        assert!(registry.has_tool("get_timestamp"));
        assert!(registry.has_tool("random_number"));

        // Check categories
        let categories = registry.category_names();
        assert!(categories.contains(&"math".to_string()));
        assert!(categories.contains(&"utility".to_string()));
    }
}
