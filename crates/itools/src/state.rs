use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::roles::UserRole;
use crate::plugins::PluginManager;
use crate::mcp::{McpClient, McpServerManager};
use crate::ui::mcp_settings::McpSettingsUi;

/// Main state for the iTools module
#[derive(Debug)]
pub struct IToolsState {
    /// Current user role
    pub current_role: UserRole,

    /// Plugin manager
    pub plugin_manager: PluginManager,

    /// MCP client for protocol communication
    pub mcp_client: McpClient,

    /// MCP server manager
    pub mcp_server_manager: Option<McpServerManager>,

    /// MCP settings UI
    pub mcp_settings_ui: Option<McpSettingsUi>,

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
    McpSettings,
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
            mcp_server_manager: None,
            mcp_settings_ui: None,
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

        // Initialize MCP server manager
        self.initialize_mcp_server_manager();

        // Initialize MCP settings UI
        self.initialize_mcp_settings_ui();
    }

    /// Initialize MCP server manager
    pub fn initialize_mcp_server_manager(&mut self) {
        // Get config directory
        if let Some(config_dir) = dirs::config_dir() {
            let mcp_config_path = config_dir.join("seeu_desktop").join("mcp_servers.json");

            // Create directory if it doesn't exist
            if let Some(parent) = mcp_config_path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }

            let mut manager = McpServerManager::new(mcp_config_path);

            // Initialize synchronously for now
            if let Err(e) = manager.initialize_sync() {
                log::warn!("Failed to initialize MCP server manager: {}", e);
                return;
            }

            self.mcp_server_manager = Some(manager);

            log::info!("MCP server manager initialized");
        } else {
            log::warn!("Could not determine config directory for MCP server manager");
        }
    }

    /// Initialize MCP settings UI
    pub fn initialize_mcp_settings_ui(&mut self) {
        // Get config directory
        if let Some(config_dir) = dirs::config_dir() {
            let mcp_config_path = config_dir.join("seeu_desktop").join("mcp_servers.json");

            // Create directory if it doesn't exist
            if let Some(parent) = mcp_config_path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }

            let mcp_settings_ui = McpSettingsUi::new(mcp_config_path);

            // Initialize asynchronously if possible, otherwise skip for now
            // The UI will initialize itself when first rendered
            self.mcp_settings_ui = Some(mcp_settings_ui);

            log::info!("MCP settings UI initialized");
        } else {
            log::warn!("Could not determine config directory for MCP settings UI");
        }
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
        // Removed debug log to reduce noise
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

    /// 获取可用的MCP服务器列表
    pub fn get_available_mcp_servers(&self) -> Vec<(Uuid, String)> {
        if let Some(manager) = &self.mcp_server_manager {
            manager.get_server_directories()
                .into_iter()
                .flat_map(|dir| {
                    dir.servers.into_iter().map(|server| {
                        (server.id, server.name)
                    })
                })
                .collect()
        } else {
            Vec::new()
        }
    }

    /// 获取MCP服务器的能力信息
    pub fn get_mcp_server_capabilities(&self, server_id: Uuid) -> Option<crate::mcp::rmcp_client::ServerCapabilities> {
        if let Some(manager) = &self.mcp_server_manager {
            manager.get_server_capabilities(server_id)
        } else {
            None
        }
    }

    /// 获取MCP服务器管理器的引用
    pub fn get_mcp_server_manager(&self) -> Option<&McpServerManager> {
        self.mcp_server_manager.as_ref()
    }

    /// 获取MCP服务器管理器的可变引用
    pub fn get_mcp_server_manager_mut(&mut self) -> Option<&mut McpServerManager> {
        self.mcp_server_manager.as_mut()
    }
}

impl Default for IToolsState {
    fn default() -> Self {
        Self::new()
    }
}
