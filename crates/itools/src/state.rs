use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::roles::{Role, UserRole};
use crate::plugins::{Plugin, PluginStatus, PluginManager};
use crate::mcp::McpClient;

/// Main state for the iTools module
#[derive(Debug)]
pub struct IToolsState {
    /// Current user role
    pub current_role: UserRole,

    /// Plugin manager
    pub plugin_manager: PluginManager,

    /// MCP client for protocol communication
    pub mcp_client: McpClient,

    /// UI state
    pub ui_state: UiState,

    /// Security context
    pub security_context: SecurityContext,
}

/// UI state for the iTools interface
#[derive(Debug, Default)]
pub struct UiState {
    /// Currently selected view
    pub current_view: IToolsView,

    /// Plugin market search query
    pub search_query: String,

    /// Selected plugin for details view
    pub selected_plugin: Option<Uuid>,

    /// Show plugin installation dialog
    pub show_install_dialog: bool,

    /// Show role selection dialog
    pub show_role_dialog: bool,

    /// Plugin market filters
    pub filters: MarketFilters,
}

/// Different views in the iTools interface
#[derive(Debug, Default, PartialEq, Clone, Copy)]
pub enum IToolsView {
    #[default]
    Dashboard,
    PluginMarket,
    InstalledPlugins,
}

/// Plugin market filters
#[derive(Debug, Default)]
pub struct MarketFilters {
    pub role_filter: Option<UserRole>,
    pub category_filter: Option<String>,
    pub permission_level: Option<PermissionLevel>,
    pub verified_only: bool,
    pub featured_only: bool,
}

/// Permission levels for plugins
#[derive(Debug, Clone, PartialEq, PartialOrd, Ord, Eq, Serialize, Deserialize)]
pub enum PermissionLevel {
    Low,
    Medium,
    High,
    Critical,
}

/// Security context for the current session
#[derive(Debug)]
pub struct SecurityContext {
    pub session_id: Uuid,
    pub permissions: HashMap<String, bool>,
    pub audit_log: Vec<AuditEntry>,
}

/// Audit log entry
#[derive(Debug, Clone)]
pub struct AuditEntry {
    pub timestamp: DateTime<Utc>,
    pub action: String,
    pub plugin_id: Option<Uuid>,
    pub user_role: UserRole,
    pub result: AuditResult,
}

/// Result of an audited action
#[derive(Debug, Clone)]
pub enum AuditResult {
    Success,
    Denied(String),
    Error(String),
}



impl IToolsState {
    /// Create a new iTools state
    pub fn new() -> Self {
        let session_id = Uuid::new_v4();

        Self {
            current_role: UserRole::BusinessUser, // Default role
            plugin_manager: PluginManager::new(),
            mcp_client: McpClient::new(),
            ui_state: UiState::default(),
            security_context: SecurityContext {
                session_id,
                permissions: HashMap::new(),
                audit_log: Vec::new(),
            },
        }
    }

    /// Initialize the iTools state
    pub fn initialize(&mut self) {
        log::info!("Initializing iTools state with session ID: {}", self.security_context.session_id);

        // Initialize plugin manager
        self.plugin_manager.initialize();

        // Load user preferences
        self.load_user_preferences();

        // Initialize MCP client
        self.mcp_client.initialize();
    }

    /// Update background tasks
    pub fn update(&mut self) {
        // Update plugin manager
        self.plugin_manager.update();

        // Process MCP messages
        self.mcp_client.process_messages();

        // Clean up old audit entries
        self.cleanup_audit_log();
    }

    /// Load user preferences from storage
    fn load_user_preferences(&mut self) {
        // TODO: Implement user preferences loading
        log::debug!("Loading user preferences");
    }

    /// Clean up old audit log entries
    fn cleanup_audit_log(&mut self) {
        let cutoff = Utc::now() - chrono::Duration::days(30);
        self.security_context.audit_log.retain(|entry| entry.timestamp > cutoff);
    }



    /// Log an audit entry
    pub fn log_audit(&mut self, action: String, plugin_id: Option<Uuid>, result: AuditResult) {
        let entry = AuditEntry {
            timestamp: Utc::now(),
            action,
            plugin_id,
            user_role: self.current_role.clone(),
            result,
        };

        self.security_context.audit_log.push(entry);
    }
}

impl Default for IToolsState {
    fn default() -> Self {
        Self::new()
    }
}
