use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// User roles in the iTools system
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum UserRole {
    BusinessUser,
    Developer,
    Operations,
    Administrator,
    Custom(String),
}

/// Role definition with permissions and capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Role {
    pub name: String,
    pub description: String,
    pub permissions: Vec<Permission>,
    pub ui_components: Vec<UiComponent>,
    pub plugin_access: PluginAccess,
}

/// Permission definition
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Permission {
    pub action: String,
    pub resource: String,
    pub conditions: Option<PermissionConditions>,
}

/// Conditions for permission evaluation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PermissionConditions {
    pub time_window: Option<TimeWindow>,
    pub ip_range: Option<String>,
    pub resource_pattern: Option<String>,
}

/// Time window for permission
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TimeWindow {
    pub start: String, // HH:MM format
    pub end: String,   // HH:MM format
}

/// UI components available to a role
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum UiComponent {
    DataDashboard,
    DocumentTemplates,
    AnalysisButtons,
    VisualizationCharts,
    CodeEditor,
    ApiDebugger,
    PluginDevTools,
    LogViewer,
    SystemMonitor,
    SecurityAudit,
    PluginStatusBoard,
    RolePermissionManager,
    PluginMarketReview,
    PolicyConfiguration,
}

/// Plugin access configuration for a role
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginAccess {
    pub allowed_categories: Vec<String>,
    pub max_permission_level: crate::state::PermissionLevel,
    pub auto_approve_threshold: Option<crate::state::PermissionLevel>,
}

impl UserRole {
    /// Get the display name for the role
    pub fn display_name(&self) -> &str {
        match self {
            UserRole::BusinessUser => "业务用户",
            UserRole::Developer => "开发者",
            UserRole::Operations => "运维人员",
            UserRole::Administrator => "管理员",
            UserRole::Custom(name) => name,
        }
    }
    
    /// Get the role definition
    pub fn get_role_definition(&self) -> Role {
        match self {
            UserRole::BusinessUser => Role {
                name: "业务用户".to_string(),
                description: "专注于数据分析和业务洞察的用户".to_string(),
                permissions: vec![
                    Permission {
                        action: "read".to_string(),
                        resource: "/data/*".to_string(),
                        conditions: Some(PermissionConditions {
                            time_window: Some(TimeWindow {
                                start: "09:00".to_string(),
                                end: "18:00".to_string(),
                            }),
                            ip_range: None,
                            resource_pattern: None,
                        }),
                    },
                    Permission {
                        action: "execute".to_string(),
                        resource: "/tools/analysis/*".to_string(),
                        conditions: None,
                    },
                ],
                ui_components: vec![
                    UiComponent::DataDashboard,
                    UiComponent::DocumentTemplates,
                    UiComponent::AnalysisButtons,
                    UiComponent::VisualizationCharts,
                ],
                plugin_access: PluginAccess {
                    allowed_categories: vec!["data-analysis".to_string(), "visualization".to_string()],
                    max_permission_level: crate::state::PermissionLevel::Medium,
                    auto_approve_threshold: Some(crate::state::PermissionLevel::Low),
                },
            },
            
            UserRole::Developer => Role {
                name: "开发者".to_string(),
                description: "软件开发和代码管理用户".to_string(),
                permissions: vec![
                    Permission {
                        action: "read".to_string(),
                        resource: "/code/*".to_string(),
                        conditions: Some(PermissionConditions {
                            time_window: Some(TimeWindow {
                                start: "09:00".to_string(),
                                end: "18:00".to_string(),
                            }),
                            ip_range: Some("192.168.1.0/24".to_string()),
                            resource_pattern: None,
                        }),
                    },
                    Permission {
                        action: "execute".to_string(),
                        resource: "/tools/dev/*".to_string(),
                        conditions: None,
                    },
                    Permission {
                        action: "write".to_string(),
                        resource: "/code/workspace/*".to_string(),
                        conditions: None,
                    },
                ],
                ui_components: vec![
                    UiComponent::CodeEditor,
                    UiComponent::ApiDebugger,
                    UiComponent::PluginDevTools,
                    UiComponent::LogViewer,
                ],
                plugin_access: PluginAccess {
                    allowed_categories: vec![
                        "development".to_string(),
                        "version-control".to_string(),
                        "debugging".to_string(),
                    ],
                    max_permission_level: crate::state::PermissionLevel::High,
                    auto_approve_threshold: Some(crate::state::PermissionLevel::Medium),
                },
            },
            
            UserRole::Operations => Role {
                name: "运维人员".to_string(),
                description: "系统运维和监控用户".to_string(),
                permissions: vec![
                    Permission {
                        action: "read".to_string(),
                        resource: "/system/*".to_string(),
                        conditions: None,
                    },
                    Permission {
                        action: "execute".to_string(),
                        resource: "/tools/monitoring/*".to_string(),
                        conditions: None,
                    },
                    Permission {
                        action: "read".to_string(),
                        resource: "/logs/*".to_string(),
                        conditions: None,
                    },
                ],
                ui_components: vec![
                    UiComponent::SystemMonitor,
                    UiComponent::SecurityAudit,
                    UiComponent::PluginStatusBoard,
                    UiComponent::LogViewer,
                ],
                plugin_access: PluginAccess {
                    allowed_categories: vec![
                        "monitoring".to_string(),
                        "security".to_string(),
                        "system-admin".to_string(),
                    ],
                    max_permission_level: crate::state::PermissionLevel::High,
                    auto_approve_threshold: Some(crate::state::PermissionLevel::Low),
                },
            },
            
            UserRole::Administrator => Role {
                name: "管理员".to_string(),
                description: "系统管理员，拥有最高权限".to_string(),
                permissions: vec![
                    Permission {
                        action: "*".to_string(),
                        resource: "*".to_string(),
                        conditions: None,
                    },
                ],
                ui_components: vec![
                    UiComponent::RolePermissionManager,
                    UiComponent::PluginMarketReview,
                    UiComponent::PolicyConfiguration,
                    UiComponent::SystemMonitor,
                    UiComponent::SecurityAudit,
                ],
                plugin_access: PluginAccess {
                    allowed_categories: vec!["*".to_string()],
                    max_permission_level: crate::state::PermissionLevel::Critical,
                    auto_approve_threshold: None, // Manual approval for all
                },
            },
            
            UserRole::Custom(name) => Role {
                name: name.clone(),
                description: "自定义角色".to_string(),
                permissions: vec![],
                ui_components: vec![],
                plugin_access: PluginAccess {
                    allowed_categories: vec![],
                    max_permission_level: crate::state::PermissionLevel::Low,
                    auto_approve_threshold: None,
                },
            },
        }
    }
    
    /// Check if the role has a specific permission
    pub fn has_permission(&self, action: &str, resource: &str) -> bool {
        let role_def = self.get_role_definition();
        
        for permission in &role_def.permissions {
            if self.matches_permission(&permission, action, resource) {
                // TODO: Check conditions (time window, IP range, etc.)
                return true;
            }
        }
        
        false
    }
    
    /// Check if a permission matches the given action and resource
    fn matches_permission(&self, permission: &Permission, action: &str, resource: &str) -> bool {
        let action_matches = permission.action == "*" || permission.action == action;
        let resource_matches = permission.resource == "*" || 
                              resource.starts_with(&permission.resource.replace("*", ""));
        
        action_matches && resource_matches
    }
    
    /// Get available UI components for this role
    pub fn get_ui_components(&self) -> Vec<UiComponent> {
        self.get_role_definition().ui_components
    }
    
    /// Check if role can access a plugin category
    pub fn can_access_plugin_category(&self, category: &str) -> bool {
        let role_def = self.get_role_definition();
        role_def.plugin_access.allowed_categories.contains(&"*".to_string()) ||
        role_def.plugin_access.allowed_categories.contains(&category.to_string())
    }
}

impl std::fmt::Display for UserRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}
