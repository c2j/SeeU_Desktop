use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

pub mod plugin;
pub mod mcp;
pub mod host;
pub mod macros;

pub use plugin::*;
pub use mcp::*;
pub use host::*;

/// Plugin SDK version
pub const SDK_VERSION: &str = "0.1.0";

/// Plugin capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginCapabilities {
    pub provides_tools: bool,
    pub provides_resources: bool,
    pub provides_prompts: bool,
    pub supports_sampling: bool,
    pub supports_notifications: bool,
    pub supports_progress: bool,
}

impl Default for PluginCapabilities {
    fn default() -> Self {
        Self {
            provides_tools: false,
            provides_resources: false,
            provides_prompts: false,
            supports_sampling: false,
            supports_notifications: false,
            supports_progress: false,
        }
    }
}

/// Plugin metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMetadata {
    pub name: String,
    pub display_name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub license: String,
    pub homepage: Option<String>,
    pub repository: Option<String>,
    pub keywords: Vec<String>,
    pub categories: Vec<String>,
}

/// Plugin permission request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginPermission {
    pub permission_type: PermissionType,
    pub resource: String,
    pub description: String,
    pub required: bool,
    pub level: PermissionLevel,
}

/// Permission types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PermissionType {
    FileSystem,
    Network,
    SystemInfo,
    UserData,
    ExternalApi,
    ProcessExecution,
    DatabaseAccess,
    EnvironmentVariables,
}

/// Permission levels
#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum PermissionLevel {
    Low,
    Medium,
    High,
    Critical,
}

/// Plugin request/response types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginRequest {
    pub id: String,
    pub method: String,
    pub params: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginResponse {
    pub id: String,
    pub result: Option<serde_json::Value>,
    pub error: Option<PluginError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginError {
    pub code: i32,
    pub message: String,
    pub data: Option<serde_json::Value>,
}

/// Tool definition for MCP
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
    pub output_schema: Option<serde_json::Value>,
}

/// Resource definition for MCP
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceDefinition {
    pub uri: String,
    pub name: String,
    pub description: String,
    pub mime_type: Option<String>,
}

/// Prompt definition for MCP
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptDefinition {
    pub name: String,
    pub description: String,
    pub arguments: Vec<PromptArgument>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptArgument {
    pub name: String,
    pub description: String,
    pub required: bool,
    pub argument_type: String,
}

/// Plugin trait that all plugins must implement
pub trait Plugin {
    /// Initialize the plugin
    fn init(&mut self) -> Result<(), PluginError>;
    
    /// Get plugin capabilities
    fn get_capabilities(&self) -> PluginCapabilities;
    
    /// Get plugin metadata
    fn get_metadata(&self) -> PluginMetadata;
    
    /// Get required permissions
    fn get_permissions(&self) -> Vec<PluginPermission>;
    
    /// Handle a request
    fn handle_request(&mut self, request: PluginRequest) -> PluginResponse;
    
    /// Cleanup resources
    fn cleanup(&mut self);
}

/// Tool trait for plugins that provide tools
pub trait ToolProvider {
    /// Get available tools
    fn get_tools(&self) -> Vec<ToolDefinition>;
    
    /// Execute a tool
    fn execute_tool(&mut self, name: &str, arguments: serde_json::Value) -> Result<serde_json::Value, PluginError>;
}

/// Resource trait for plugins that provide resources
pub trait ResourceProvider {
    /// Get available resources
    fn get_resources(&self) -> Vec<ResourceDefinition>;
    
    /// Get resource content
    fn get_resource(&self, uri: &str) -> Result<Vec<u8>, PluginError>;
}

/// Prompt trait for plugins that provide prompts
pub trait PromptProvider {
    /// Get available prompts
    fn get_prompts(&self) -> Vec<PromptDefinition>;
    
    /// Get prompt content
    fn get_prompt(&self, name: &str, arguments: HashMap<String, String>) -> Result<String, PluginError>;
}

/// Utility functions
pub mod utils {
    use super::*;
    
    /// Create a successful response
    pub fn success_response(id: String, result: serde_json::Value) -> PluginResponse {
        PluginResponse {
            id,
            result: Some(result),
            error: None,
        }
    }
    
    /// Create an error response
    pub fn error_response(id: String, code: i32, message: String) -> PluginResponse {
        PluginResponse {
            id,
            result: None,
            error: Some(PluginError {
                code,
                message,
                data: None,
            }),
        }
    }
    
    /// Parse JSON safely
    pub fn parse_json<T: for<'de> Deserialize<'de>>(value: &serde_json::Value) -> Result<T, PluginError> {
        serde_json::from_value(value.clone()).map_err(|e| PluginError {
            code: -32602,
            message: format!("Invalid params: {}", e),
            data: None,
        })
    }
}

/// Re-export commonly used types
pub use serde_json::{json, Value};
pub use uuid::Uuid;
pub use chrono::{DateTime, Utc};
