use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::path::PathBuf;

use crate::state::PermissionLevel;
use crate::roles::UserRole;

/// Plugin definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plugin {
    pub id: Uuid,
    pub metadata: PluginMetadata,
    pub manifest: PluginManifest,
    pub status: PluginStatus,
    pub installation_path: Option<PathBuf>,
    pub installed_at: Option<DateTime<Utc>>,
    pub last_updated: Option<DateTime<Utc>>,
}

/// Plugin metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMetadata {
    pub name: String,
    pub display_name: String,
    pub description: String,
    pub version: String,
    pub author: String,
    pub homepage: Option<String>,
    pub repository: Option<String>,
    pub license: String,
    pub keywords: Vec<String>,
    pub categories: Vec<String>,
    pub target_roles: Vec<UserRole>,
    pub icon: Option<String>,
    pub screenshots: Vec<String>,
}

/// Plugin manifest defining capabilities and requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    pub schema_version: String,
    pub mcp_version: String,
    pub capabilities: PluginCapabilities,
    pub permissions: Vec<PluginPermission>,
    pub dependencies: Vec<PluginDependency>,
    pub resources: Vec<ResourceDefinition>,
    pub tools: Vec<ToolDefinition>,
    pub prompts: Vec<PromptDefinition>,
    pub configuration: Option<PluginConfiguration>,
}

/// Plugin capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginCapabilities {
    pub provides_resources: bool,
    pub provides_tools: bool,
    pub provides_prompts: bool,
    pub supports_sampling: bool,
    pub supports_notifications: bool,
    pub supports_progress: bool,
}

/// Plugin permission requirement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginPermission {
    pub permission_type: PermissionType,
    pub resource: String,
    pub description: String,
    pub required: bool,
    pub level: PermissionLevel,
}

/// Types of permissions a plugin can request
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

/// Plugin dependency
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginDependency {
    pub name: String,
    pub version_requirement: String,
    pub optional: bool,
}

/// Resource definition for MCP
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceDefinition {
    pub uri: String,
    pub name: String,
    pub description: String,
    pub mime_type: Option<String>,
}

/// Tool definition for MCP
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
    pub output_schema: Option<serde_json::Value>,
}

/// Prompt definition for MCP
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptDefinition {
    pub name: String,
    pub description: String,
    pub arguments: Vec<PromptArgument>,
}

/// Prompt argument
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptArgument {
    pub name: String,
    pub description: String,
    pub required: bool,
    pub argument_type: String,
}

/// Plugin configuration schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfiguration {
    pub schema: serde_json::Value,
    pub default_values: HashMap<String, serde_json::Value>,
}

/// Plugin status
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PluginStatus {
    NotInstalled,
    Installing,
    Installed,
    Enabled,
    Disabled,
    Updating,
    Error(String),
    Uninstalling,
}

impl Plugin {
    /// Create a new plugin instance
    pub fn new(metadata: PluginMetadata, manifest: PluginManifest) -> Self {
        Self {
            id: Uuid::new_v4(),
            metadata,
            manifest,
            status: PluginStatus::NotInstalled,
            installation_path: None,
            installed_at: None,
            last_updated: None,
        }
    }
    
    /// Check if plugin is compatible with a user role
    pub fn is_compatible_with_role(&self, role: &UserRole) -> bool {
        self.metadata.target_roles.is_empty() || 
        self.metadata.target_roles.contains(role)
    }
    
    /// Get required permission level
    pub fn get_max_permission_level(&self) -> PermissionLevel {
        self.manifest.permissions
            .iter()
            .map(|p| &p.level)
            .max()
            .cloned()
            .unwrap_or(PermissionLevel::Low)
    }
    
    /// Check if plugin requires critical permissions
    pub fn requires_critical_permissions(&self) -> bool {
        self.manifest.permissions
            .iter()
            .any(|p| p.level == PermissionLevel::Critical)
    }
    
    /// Get plugin size estimate (for UI display)
    pub fn get_size_estimate(&self) -> String {
        // This would be calculated based on actual plugin size
        "~2.5 MB".to_string()
    }
    
    /// Get compatibility info
    pub fn get_compatibility_info(&self) -> Vec<String> {
        let mut info = Vec::new();
        
        info.push(format!("MCP Version: {}", self.manifest.mcp_version));
        
        if !self.manifest.dependencies.is_empty() {
            info.push(format!("Dependencies: {}", self.manifest.dependencies.len()));
        }
        
        if self.requires_critical_permissions() {
            info.push("⚠️ Requires critical permissions".to_string());
        }
        
        info
    }
    
    /// Get user-friendly status text
    pub fn get_status_text(&self) -> String {
        match &self.status {
            PluginStatus::NotInstalled => "未安装".to_string(),
            PluginStatus::Installing => "安装中...".to_string(),
            PluginStatus::Installed => "已安装".to_string(),
            PluginStatus::Enabled => "已启用".to_string(),
            PluginStatus::Disabled => "已禁用".to_string(),
            PluginStatus::Updating => "更新中...".to_string(),
            PluginStatus::Error(msg) => format!("错误: {}", msg),
            PluginStatus::Uninstalling => "卸载中...".to_string(),
        }
    }
    
    /// Check if plugin can be enabled
    pub fn can_be_enabled(&self) -> bool {
        matches!(self.status, PluginStatus::Installed | PluginStatus::Disabled)
    }
    
    /// Check if plugin can be disabled
    pub fn can_be_disabled(&self) -> bool {
        matches!(self.status, PluginStatus::Enabled)
    }
    
    /// Check if plugin can be uninstalled
    pub fn can_be_uninstalled(&self) -> bool {
        matches!(
            self.status, 
            PluginStatus::Installed | PluginStatus::Disabled | PluginStatus::Error(_)
        )
    }
}

impl PluginStatus {
    /// Check if status indicates plugin is operational
    pub fn is_operational(&self) -> bool {
        matches!(self, PluginStatus::Enabled)
    }
    
    /// Check if status indicates plugin is in transition
    pub fn is_transitioning(&self) -> bool {
        matches!(
            self, 
            PluginStatus::Installing | PluginStatus::Updating | PluginStatus::Uninstalling
        )
    }
    
    /// Check if status indicates an error
    pub fn is_error(&self) -> bool {
        matches!(self, PluginStatus::Error(_))
    }
}
