//! # Tool Definition System
//!
//! This module provides structures for defining tools that can be called by LLMs.
//! It includes comprehensive parameter validation and JSON schema generation.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::error::{OrchestraError, Result};

/// Defines a tool that can be called by an LLM
///
/// A tool definition includes the tool's name, description, and parameters.
/// This information is sent to the LLM so it knows what tools are available
/// and how to use them.
///
/// ## For Rust Beginners
///
/// - `#[derive(Debug, Clone, Serialize, Deserialize)]` automatically implements
///   common traits for this struct
/// - `Debug` allows printing the struct for debugging
/// - `Clone` allows making copies of the struct
/// - `Serialize`/`Deserialize` allow converting to/from JSON
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    /// The name of the tool (must be unique)
    pub name: String,
    
    /// A description of what the tool does
    pub description: String,
    
    /// The parameters this tool accepts
    pub parameters: HashMap<String, ToolParameter>,
    
    /// Whether this tool is deprecated
    #[serde(default)]
    pub deprecated: bool,
}

impl ToolDefinition {
    /// Create a new tool definition
    ///
    /// # Arguments
    /// * `name` - The tool's name (should be snake_case)
    /// * `description` - What the tool does
    ///
    /// # Example
    /// ```rust
    /// use orchestra_core::tools::ToolDefinition;
    ///
    /// let tool = ToolDefinition::new(
    ///     "get_weather",
    ///     "Get current weather information for a location"
    /// );
    /// ```
    pub fn new<S: Into<String>>(name: S, description: S) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            parameters: HashMap::new(),
            deprecated: false,
        }
    }
    
    /// Add a parameter to this tool
    ///
    /// Parameters define what inputs the tool expects. Each parameter has a name,
    /// type, and optional constraints like whether it's required.
    ///
    /// # Example
    /// ```rust
    /// use orchestra_core::tools::{ToolDefinition, ToolParameter, ToolParameterType};
    ///
    /// let tool = ToolDefinition::new("calculator", "Basic calculator")
    ///     .with_parameter(
    ///         ToolParameter::new("operation", ToolParameterType::String)
    ///             .with_description("The operation to perform")
    ///             .required()
    ///     );
    /// ```
    pub fn with_parameter(mut self, parameter: ToolParameter) -> Self {
        self.parameters.insert(parameter.name.clone(), parameter);
        self
    }
    
    /// Mark this tool as deprecated
    pub fn deprecated(mut self) -> Self {
        self.deprecated = true;
        self
    }
    
    /// Validate the tool definition
    ///
    /// This checks that the tool definition is valid:
    /// - Name is not empty and follows naming conventions
    /// - Description is not empty
    /// - All parameters are valid
    pub fn validate(&self) -> Result<()> {
        // Validate name
        if self.name.is_empty() {
            return Err(OrchestraError::config("Tool name cannot be empty"));
        }
        
        // Tool names should follow snake_case convention
        if !self.name.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_') {
            return Err(OrchestraError::config(
                "Tool name should use snake_case (lowercase letters, numbers, and underscores only)"
            ));
        }
        
        // Validate description
        if self.description.is_empty() {
            return Err(OrchestraError::config("Tool description cannot be empty"));
        }
        
        // Validate all parameters
        for (name, parameter) in &self.parameters {
            if name != &parameter.name {
                return Err(OrchestraError::config(
                    &format!("Parameter name mismatch: key '{}' vs parameter name '{}'", name, parameter.name)
                ));
            }
            parameter.validate()?;
        }
        
        Ok(())
    }
    
    /// Get required parameters
    pub fn required_parameters(&self) -> Vec<&ToolParameter> {
        self.parameters.values().filter(|p| p.required).collect()
    }
    
    /// Get optional parameters
    pub fn optional_parameters(&self) -> Vec<&ToolParameter> {
        self.parameters.values().filter(|p| !p.required).collect()
    }
    
    /// Convert to JSON schema format
    ///
    /// This generates a JSON schema that describes the tool's parameters.
    /// This schema can be sent to LLMs to help them understand how to call the tool.
    pub fn to_json_schema(&self) -> serde_json::Value {
        let mut properties = serde_json::Map::new();
        let mut required = Vec::new();
        
        for parameter in self.parameters.values() {
            properties.insert(parameter.name.clone(), parameter.to_json_schema());
            if parameter.required {
                required.push(parameter.name.clone());
            }
        }
        
        serde_json::json!({
            "type": "object",
            "properties": properties,
            "required": required,
            "additionalProperties": false
        })
    }
}

/// Defines a parameter for a tool
///
/// Parameters specify what inputs a tool expects, including the type,
/// whether it's required, and any constraints on the values.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolParameter {
    /// The parameter name
    pub name: String,
    
    /// The parameter type
    pub parameter_type: ToolParameterType,
    
    /// Description of what this parameter does
    pub description: Option<String>,
    
    /// Whether this parameter is required
    #[serde(default)]
    pub required: bool,
    
    /// Default value if not provided
    pub default: Option<serde_json::Value>,
    
    /// For string types: allowed values
    pub enum_values: Option<Vec<String>>,
    
    /// For numeric types: minimum value
    pub minimum: Option<f64>,
    
    /// For numeric types: maximum value
    pub maximum: Option<f64>,
    
    /// For string types: minimum length
    pub min_length: Option<usize>,
    
    /// For string types: maximum length
    pub max_length: Option<usize>,
    
    /// For array types: minimum number of items
    pub min_items: Option<usize>,
    
    /// For array types: maximum number of items
    pub max_items: Option<usize>,
}

impl ToolParameter {
    /// Create a new parameter
    pub fn new<S: Into<String>>(name: S, parameter_type: ToolParameterType) -> Self {
        Self {
            name: name.into(),
            parameter_type,
            description: None,
            required: false,
            default: None,
            enum_values: None,
            minimum: None,
            maximum: None,
            min_length: None,
            max_length: None,
            min_items: None,
            max_items: None,
        }
    }
    
    /// Add a description
    pub fn with_description<S: Into<String>>(mut self, description: S) -> Self {
        self.description = Some(description.into());
        self
    }
    
    /// Mark as required
    pub fn required(mut self) -> Self {
        self.required = true;
        self
    }
    
    /// Set default value
    pub fn with_default(mut self, default: serde_json::Value) -> Self {
        self.default = Some(default);
        self
    }
    
    /// Set allowed string values (enum)
    pub fn with_enum_values(mut self, values: Vec<&str>) -> Self {
        self.enum_values = Some(values.into_iter().map(|s| s.to_string()).collect());
        self
    }
    
    /// Set numeric range
    pub fn with_range(mut self, min: Option<f64>, max: Option<f64>) -> Self {
        self.minimum = min;
        self.maximum = max;
        self
    }
    
    /// Set string length constraints
    pub fn with_length_range(mut self, min: Option<usize>, max: Option<usize>) -> Self {
        self.min_length = min;
        self.max_length = max;
        self
    }
    
    /// Set array size constraints
    pub fn with_items_range(mut self, min: Option<usize>, max: Option<usize>) -> Self {
        self.min_items = min;
        self.max_items = max;
        self
    }
    
    /// Validate the parameter definition
    pub fn validate(&self) -> Result<()> {
        if self.name.is_empty() {
            return Err(OrchestraError::config("Parameter name cannot be empty"));
        }
        
        // Validate numeric ranges
        if let (Some(min), Some(max)) = (self.minimum, self.maximum) {
            if min > max {
                return Err(OrchestraError::config("Minimum value cannot be greater than maximum"));
            }
        }
        
        // Validate length ranges
        if let (Some(min), Some(max)) = (self.min_length, self.max_length) {
            if min > max {
                return Err(OrchestraError::config("Minimum length cannot be greater than maximum"));
            }
        }
        
        // Validate item ranges
        if let (Some(min), Some(max)) = (self.min_items, self.max_items) {
            if min > max {
                return Err(OrchestraError::config("Minimum items cannot be greater than maximum"));
            }
        }
        
        Ok(())
    }
    
    /// Convert to JSON schema
    pub fn to_json_schema(&self) -> serde_json::Value {
        let mut schema = serde_json::Map::new();
        
        // Set type
        schema.insert("type".to_string(), self.parameter_type.to_json_schema_type());
        
        // Add description
        if let Some(ref desc) = self.description {
            schema.insert("description".to_string(), serde_json::Value::String(desc.clone()));
        }
        
        // Add constraints based on type
        match self.parameter_type {
            ToolParameterType::String => {
                if let Some(ref enum_vals) = self.enum_values {
                    schema.insert("enum".to_string(), serde_json::Value::Array(
                        enum_vals.iter().map(|s| serde_json::Value::String(s.clone())).collect()
                    ));
                }
                if let Some(min) = self.min_length {
                    schema.insert("minLength".to_string(), serde_json::Value::Number(min.into()));
                }
                if let Some(max) = self.max_length {
                    schema.insert("maxLength".to_string(), serde_json::Value::Number(max.into()));
                }
            }
            ToolParameterType::Number | ToolParameterType::Integer => {
                if let Some(min) = self.minimum {
                    schema.insert("minimum".to_string(), serde_json::json!(min));
                }
                if let Some(max) = self.maximum {
                    schema.insert("maximum".to_string(), serde_json::json!(max));
                }
            }
            ToolParameterType::Array => {
                if let Some(min) = self.min_items {
                    schema.insert("minItems".to_string(), serde_json::Value::Number(min.into()));
                }
                if let Some(max) = self.max_items {
                    schema.insert("maxItems".to_string(), serde_json::Value::Number(max.into()));
                }
            }
            _ => {}
        }
        
        serde_json::Value::Object(schema)
    }
}

/// The type of a tool parameter
///
/// This enum defines all the supported parameter types for tools.
/// Each type corresponds to a JSON schema type.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ToolParameterType {
    /// Text string
    String,
    /// Floating point number
    Number,
    /// Whole number
    Integer,
    /// True/false value
    Boolean,
    /// Array of values
    Array,
    /// JSON object
    Object,
}

impl ToolParameterType {
    /// Convert to JSON schema type string
    pub fn to_json_schema_type(&self) -> serde_json::Value {
        let type_str = match self {
            ToolParameterType::String => "string",
            ToolParameterType::Number => "number",
            ToolParameterType::Integer => "integer",
            ToolParameterType::Boolean => "boolean",
            ToolParameterType::Array => "array",
            ToolParameterType::Object => "object",
        };
        serde_json::Value::String(type_str.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_tool_definition_creation() {
        let tool = ToolDefinition::new("test_tool", "A test tool");
        assert_eq!(tool.name, "test_tool");
        assert_eq!(tool.description, "A test tool");
        assert!(tool.parameters.is_empty());
        assert!(!tool.deprecated);
    }

    #[test]
    fn test_tool_definition_with_parameters() {
        let tool = ToolDefinition::new("calculator", "Basic calculator")
            .with_parameter(
                ToolParameter::new("operation", ToolParameterType::String)
                    .with_description("The operation to perform")
                    .required()
            )
            .with_parameter(
                ToolParameter::new("a", ToolParameterType::Number)
                    .with_description("First number")
                    .required()
            );

        assert_eq!(tool.parameters.len(), 2);
        assert!(tool.parameters.contains_key("operation"));
        assert!(tool.parameters.contains_key("a"));

        let required_params = tool.required_parameters();
        assert_eq!(required_params.len(), 2);

        let optional_params = tool.optional_parameters();
        assert_eq!(optional_params.len(), 0);
    }

    #[test]
    fn test_tool_definition_validation() {
        // Valid tool
        let valid_tool = ToolDefinition::new("valid_tool", "A valid tool");
        assert!(valid_tool.validate().is_ok());

        // Invalid name (empty)
        let invalid_tool = ToolDefinition::new("", "Description");
        assert!(invalid_tool.validate().is_err());

        // Invalid name (not snake_case)
        let invalid_tool = ToolDefinition::new("InvalidTool", "Description");
        assert!(invalid_tool.validate().is_err());

        // Invalid description (empty)
        let invalid_tool = ToolDefinition::new("valid_tool", "");
        assert!(invalid_tool.validate().is_err());
    }

    #[test]
    fn test_tool_parameter_creation() {
        let param = ToolParameter::new("test_param", ToolParameterType::String)
            .with_description("A test parameter")
            .required()
            .with_default(json!("default_value"));

        assert_eq!(param.name, "test_param");
        assert_eq!(param.parameter_type, ToolParameterType::String);
        assert_eq!(param.description, Some("A test parameter".to_string()));
        assert!(param.required);
        assert_eq!(param.default, Some(json!("default_value")));
    }

    #[test]
    fn test_tool_parameter_constraints() {
        let string_param = ToolParameter::new("text", ToolParameterType::String)
            .with_enum_values(vec!["option1", "option2"])
            .with_length_range(Some(5), Some(50));

        assert_eq!(string_param.enum_values, Some(vec!["option1".to_string(), "option2".to_string()]));
        assert_eq!(string_param.min_length, Some(5));
        assert_eq!(string_param.max_length, Some(50));

        let number_param = ToolParameter::new("value", ToolParameterType::Number)
            .with_range(Some(0.0), Some(100.0));

        assert_eq!(number_param.minimum, Some(0.0));
        assert_eq!(number_param.maximum, Some(100.0));
    }

    #[test]
    fn test_tool_parameter_validation() {
        // Valid parameter
        let valid_param = ToolParameter::new("valid", ToolParameterType::String);
        assert!(valid_param.validate().is_ok());

        // Invalid name (empty)
        let invalid_param = ToolParameter::new("", ToolParameterType::String);
        assert!(invalid_param.validate().is_err());

        // Invalid range (min > max)
        let invalid_param = ToolParameter::new("invalid", ToolParameterType::Number)
            .with_range(Some(100.0), Some(50.0));
        assert!(invalid_param.validate().is_err());
    }

    #[test]
    fn test_json_schema_generation() {
        let tool = ToolDefinition::new("test_tool", "Test tool")
            .with_parameter(
                ToolParameter::new("required_string", ToolParameterType::String)
                    .with_description("A required string parameter")
                    .required()
            )
            .with_parameter(
                ToolParameter::new("optional_number", ToolParameterType::Number)
                    .with_description("An optional number parameter")
                    .with_range(Some(0.0), Some(100.0))
            );

        let schema = tool.to_json_schema();

        // Check basic structure
        assert_eq!(schema["type"], "object");
        assert!(schema["properties"].is_object());
        assert!(schema["required"].is_array());
        assert_eq!(schema["additionalProperties"], false);

        // Check required parameters
        let required = schema["required"].as_array().unwrap();
        assert_eq!(required.len(), 1);
        assert_eq!(required[0], "required_string");

        // Check parameter properties
        let properties = schema["properties"].as_object().unwrap();
        assert!(properties.contains_key("required_string"));
        assert!(properties.contains_key("optional_number"));

        let string_prop = &properties["required_string"];
        assert_eq!(string_prop["type"], "string");
        assert_eq!(string_prop["description"], "A required string parameter");

        let number_prop = &properties["optional_number"];
        assert_eq!(number_prop["type"], "number");
        assert_eq!(number_prop["minimum"], 0.0);
        assert_eq!(number_prop["maximum"], 100.0);
    }

    #[test]
    fn test_parameter_type_json_schema() {
        assert_eq!(ToolParameterType::String.to_json_schema_type(), json!("string"));
        assert_eq!(ToolParameterType::Number.to_json_schema_type(), json!("number"));
        assert_eq!(ToolParameterType::Integer.to_json_schema_type(), json!("integer"));
        assert_eq!(ToolParameterType::Boolean.to_json_schema_type(), json!("boolean"));
        assert_eq!(ToolParameterType::Array.to_json_schema_type(), json!("array"));
        assert_eq!(ToolParameterType::Object.to_json_schema_type(), json!("object"));
    }
}
