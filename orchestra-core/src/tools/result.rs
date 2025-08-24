//! # Tool Execution Results
//!
//! This module defines the result types returned by tool execution.
//! It provides structured ways to represent success, errors, and partial results.

use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime};

/// The result of executing a tool
///
/// This struct contains all information about a tool execution, including
/// the result data, execution status, timing information, and any errors.
///
/// ## For Rust Beginners
///
/// This struct uses several Rust concepts:
/// - `Option<T>` represents a value that might be present or absent
/// - `SystemTime` represents a point in time
/// - `Duration` represents a length of time
/// - The `#[serde(default)]` attribute provides default values during deserialization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    /// The status of the execution
    pub status: ToolResultStatus,
    
    /// The result data (if successful)
    pub data: Option<serde_json::Value>,
    
    /// Error message (if failed)
    pub error: Option<String>,
    
    /// Detailed error information
    pub error_details: Option<ToolError>,
    
    /// When the execution started
    #[serde(default = "SystemTime::now")]
    pub started_at: SystemTime,
    
    /// When the execution completed
    pub completed_at: Option<SystemTime>,
    
    /// How long the execution took
    pub duration: Option<Duration>,
    
    /// Additional metadata about the execution
    #[serde(default)]
    pub metadata: std::collections::HashMap<String, serde_json::Value>,
}

impl ToolResult {
    /// Create a successful result
    ///
    /// # Arguments
    /// * `data` - The result data as a JSON value
    ///
    /// # Example
    /// ```rust
    /// use orchestra_core::tools::ToolResult;
    /// use serde_json::json;
    ///
    /// let result = ToolResult::success(json!({
    ///     "temperature": 22.5,
    ///     "humidity": 65,
    ///     "condition": "sunny"
    /// }));
    /// ```
    pub fn success(data: serde_json::Value) -> Self {
        let now = SystemTime::now();
        Self {
            status: ToolResultStatus::Success,
            data: Some(data),
            error: None,
            error_details: None,
            started_at: now,
            completed_at: Some(now),
            duration: Some(Duration::from_millis(0)), // Instant completion
            metadata: std::collections::HashMap::new(),
        }
    }
    
    /// Create a failed result
    ///
    /// # Arguments
    /// * `error` - The error message
    ///
    /// # Example
    /// ```rust
    /// use orchestra_core::tools::ToolResult;
    ///
    /// let result = ToolResult::error("Invalid API key provided");
    /// ```
    pub fn error<S: Into<String>>(error: S) -> Self {
        let now = SystemTime::now();
        Self {
            status: ToolResultStatus::Error,
            data: None,
            error: Some(error.into()),
            error_details: None,
            started_at: now,
            completed_at: Some(now),
            duration: Some(Duration::from_millis(0)),
            metadata: std::collections::HashMap::new(),
        }
    }
    
    /// Create a failed result with detailed error information
    pub fn error_with_details<S: Into<String>>(error: S, details: ToolError) -> Self {
        let now = SystemTime::now();
        Self {
            status: ToolResultStatus::Error,
            data: None,
            error: Some(error.into()),
            error_details: Some(details),
            started_at: now,
            completed_at: Some(now),
            duration: Some(Duration::from_millis(0)),
            metadata: std::collections::HashMap::new(),
        }
    }
    
    /// Create a partial result (for streaming or long-running operations)
    pub fn partial(data: serde_json::Value) -> Self {
        let now = SystemTime::now();
        Self {
            status: ToolResultStatus::Partial,
            data: Some(data),
            error: None,
            error_details: None,
            started_at: now,
            completed_at: None,
            duration: None,
            metadata: std::collections::HashMap::new(),
        }
    }
    
    /// Mark the result as completed and calculate duration
    pub fn complete(mut self) -> Self {
        let now = SystemTime::now();
        self.completed_at = Some(now);
        
        // Calculate duration if we have a start time
        if let Ok(duration) = now.duration_since(self.started_at) {
            self.duration = Some(duration);
        }
        
        // If it was partial, mark as success
        if self.status == ToolResultStatus::Partial {
            self.status = ToolResultStatus::Success;
        }
        
        self
    }
    
    /// Add metadata to the result
    pub fn with_metadata<K: Into<String>>(mut self, key: K, value: serde_json::Value) -> Self {
        self.metadata.insert(key.into(), value);
        self
    }
    
    /// Check if the result is successful
    pub fn is_success(&self) -> bool {
        self.status == ToolResultStatus::Success
    }
    
    /// Check if the result is an error
    pub fn is_error(&self) -> bool {
        self.status == ToolResultStatus::Error
    }
    
    /// Check if the result is partial
    pub fn is_partial(&self) -> bool {
        self.status == ToolResultStatus::Partial
    }
    
    /// Get the execution duration in milliseconds
    pub fn duration_ms(&self) -> Option<u64> {
        self.duration.map(|d| d.as_millis() as u64)
    }
    
    /// Convert to a simple string representation
    pub fn to_string(&self) -> String {
        match &self.status {
            ToolResultStatus::Success => {
                if let Some(data) = &self.data {
                    format!("Success: {}", data)
                } else {
                    "Success".to_string()
                }
            }
            ToolResultStatus::Error => {
                if let Some(error) = &self.error {
                    format!("Error: {}", error)
                } else {
                    "Error: Unknown error".to_string()
                }
            }
            ToolResultStatus::Partial => {
                if let Some(data) = &self.data {
                    format!("Partial: {}", data)
                } else {
                    "Partial result".to_string()
                }
            }
        }
    }
}

/// The status of a tool execution
///
/// This enum represents the different states a tool execution can be in.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ToolResultStatus {
    /// The tool executed successfully
    Success,
    /// The tool execution failed
    Error,
    /// The tool is still executing or returned partial results
    Partial,
}

/// Detailed error information for tool execution failures
///
/// This struct provides structured error information that can help with
/// debugging and error handling.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolError {
    /// The type of error that occurred
    pub error_type: ToolErrorType,
    
    /// The error message
    pub message: String,
    
    /// Additional context about the error
    pub context: Option<std::collections::HashMap<String, serde_json::Value>>,
    
    /// The underlying cause of the error (if any)
    pub cause: Option<String>,
    
    /// Whether this error is retryable
    #[serde(default)]
    pub retryable: bool,
}

impl ToolError {
    /// Create a new tool error
    pub fn new<S: Into<String>>(error_type: ToolErrorType, message: S) -> Self {
        Self {
            error_type,
            message: message.into(),
            context: None,
            cause: None,
            retryable: false,
        }
    }
    
    /// Add context to the error
    pub fn with_context<K: Into<String>>(mut self, key: K, value: serde_json::Value) -> Self {
        if self.context.is_none() {
            self.context = Some(std::collections::HashMap::new());
        }
        self.context.as_mut().unwrap().insert(key.into(), value);
        self
    }
    
    /// Set the underlying cause
    pub fn with_cause<S: Into<String>>(mut self, cause: S) -> Self {
        self.cause = Some(cause.into());
        self
    }
    
    /// Mark the error as retryable
    pub fn retryable(mut self) -> Self {
        self.retryable = true;
        self
    }
}

/// Types of errors that can occur during tool execution
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ToolErrorType {
    /// Invalid input parameters
    InvalidInput,
    /// Authentication or authorization failure
    Authentication,
    /// Network or connectivity error
    Network,
    /// External service error
    ExternalService,
    /// Internal tool error
    Internal,
    /// Timeout error
    Timeout,
    /// Rate limiting error
    RateLimit,
    /// Resource not found
    NotFound,
    /// Permission denied
    PermissionDenied,
    /// Unknown error
    Unknown,
}

impl std::fmt::Display for ToolErrorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            ToolErrorType::InvalidInput => "Invalid Input",
            ToolErrorType::Authentication => "Authentication Error",
            ToolErrorType::Network => "Network Error",
            ToolErrorType::ExternalService => "External Service Error",
            ToolErrorType::Internal => "Internal Error",
            ToolErrorType::Timeout => "Timeout",
            ToolErrorType::RateLimit => "Rate Limit",
            ToolErrorType::NotFound => "Not Found",
            ToolErrorType::PermissionDenied => "Permission Denied",
            ToolErrorType::Unknown => "Unknown Error",
        };
        write!(f, "{}", s)
    }
}
