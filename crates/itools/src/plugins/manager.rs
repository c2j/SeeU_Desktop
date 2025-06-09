use std::collections::HashMap;
use std::path::PathBuf;
use uuid::Uuid;
use anyhow::{Result, anyhow};
use tokio::sync::mpsc;

use super::plugin::{Plugin, PluginStatus, PluginMetadata, PluginManifest};
use super::marketplace::PluginMarketplace;
use super::sandbox::PluginSandbox;
use crate::roles::UserRole;
use crate::state::{PermissionLevel, AuditResult};

/// Plugin manager handles all plugin lifecycle operations
#[derive(Debug)]
pub struct PluginManager {
    /// Installed plugins
    plugins: HashMap<Uuid, Plugin>,
    
    /// Plugin marketplace client
    marketplace: PluginMarketplace,
    
    /// Plugin sandbox for secure execution
    sandbox: PluginSandbox,
    
    /// Installation directory
    install_dir: PathBuf,
    
    /// Background task channel
    task_sender: Option<mpsc::UnboundedSender<PluginTask>>,
    task_receiver: Option<mpsc::UnboundedReceiver<PluginTask>>,
    
    /// Current operations
    pending_operations: HashMap<Uuid, PluginOperation>,
}

/// Plugin management tasks
#[derive(Debug)]
enum PluginTask {
    Install(Uuid, String), // plugin_id, source_url
    Uninstall(Uuid),
    Enable(Uuid),
    Disable(Uuid),
    Update(Uuid),
    RefreshMarketplace,
}

/// Plugin operation status
#[derive(Debug, Clone)]
pub struct PluginOperation {
    pub plugin_id: Uuid,
    pub operation_type: OperationType,
    pub progress: f32, // 0.0 to 1.0
    pub status_message: String,
}

/// Types of plugin operations
#[derive(Debug, Clone, PartialEq)]
pub enum OperationType {
    Installing,
    Uninstalling,
    Enabling,
    Disabling,
    Updating,
}

impl PluginManager {
    /// Create a new plugin manager
    pub fn new() -> Self {
        let (task_sender, task_receiver) = mpsc::unbounded_channel();
        
        let install_dir = dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("SeeU")
            .join("itools")
            .join("plugins");
        
        Self {
            plugins: HashMap::new(),
            marketplace: PluginMarketplace::new(),
            sandbox: PluginSandbox::new(),
            install_dir,
            task_sender: Some(task_sender),
            task_receiver: Some(task_receiver),
            pending_operations: HashMap::new(),
        }
    }
    
    /// Initialize the plugin manager
    pub fn initialize(&mut self) {
        log::info!("Initializing plugin manager");
        
        // Create installation directory
        if let Err(e) = std::fs::create_dir_all(&self.install_dir) {
            log::error!("Failed to create plugin directory: {}", e);
        }
        
        // Load installed plugins
        self.load_installed_plugins();
        
        // Initialize marketplace
        self.marketplace.initialize();
        
        // Initialize sandbox
        self.sandbox.initialize();
        
        // Start background task processor
        self.start_background_processor();
    }
    
    /// Update the plugin manager (called from main loop)
    pub fn update(&mut self) {
        // Process completed operations
        self.process_completed_operations();
        
        // Update marketplace
        self.marketplace.update();
        
        // Update sandbox
        self.sandbox.update();
    }
    
    /// Get all installed plugins
    pub fn get_installed_plugins(&self) -> Vec<&Plugin> {
        self.plugins.values().collect()
    }
    
    /// Get plugins filtered by role
    pub fn get_plugins_for_role(&self, role: &UserRole) -> Vec<&Plugin> {
        self.plugins
            .values()
            .filter(|plugin| plugin.is_compatible_with_role(role))
            .collect()
    }
    
    /// Get plugin by ID
    pub fn get_plugin(&self, id: &Uuid) -> Option<&Plugin> {
        self.plugins.get(id)
    }
    
    /// Get plugin by name
    pub fn get_plugin_by_name(&self, name: &str) -> Option<&Plugin> {
        self.plugins
            .values()
            .find(|plugin| plugin.metadata.name == name)
    }
    
    /// Install a plugin from marketplace
    pub fn install_plugin(&mut self, plugin_id: Uuid, source_url: String) -> Result<()> {
        if self.plugins.contains_key(&plugin_id) {
            return Err(anyhow!("Plugin already installed"));
        }
        
        // Add to pending operations
        self.pending_operations.insert(plugin_id, PluginOperation {
            plugin_id,
            operation_type: OperationType::Installing,
            progress: 0.0,
            status_message: "开始安装...".to_string(),
        });
        
        // Send task to background processor
        if let Some(sender) = &self.task_sender {
            sender.send(PluginTask::Install(plugin_id, source_url))?;
        }
        
        Ok(())
    }
    
    /// Uninstall a plugin
    pub fn uninstall_plugin(&mut self, plugin_id: Uuid) -> Result<()> {
        let plugin = self.plugins.get_mut(&plugin_id)
            .ok_or_else(|| anyhow!("Plugin not found"))?;
        
        if !plugin.can_be_uninstalled() {
            return Err(anyhow!("Plugin cannot be uninstalled in current state"));
        }
        
        // Add to pending operations
        self.pending_operations.insert(plugin_id, PluginOperation {
            plugin_id,
            operation_type: OperationType::Uninstalling,
            progress: 0.0,
            status_message: "开始卸载...".to_string(),
        });
        
        // Send task to background processor
        if let Some(sender) = &self.task_sender {
            sender.send(PluginTask::Uninstall(plugin_id))?;
        }
        
        Ok(())
    }
    
    /// Enable a plugin
    pub fn enable_plugin(&mut self, plugin_id: Uuid, user_role: &UserRole) -> Result<()> {
        let plugin = self.plugins.get_mut(&plugin_id)
            .ok_or_else(|| anyhow!("Plugin not found"))?;
        
        if !plugin.can_be_enabled() {
            return Err(anyhow!("Plugin cannot be enabled in current state"));
        }
        
        // Check role compatibility
        if !plugin.is_compatible_with_role(user_role) {
            return Err(anyhow!("Plugin not compatible with current role"));
        }
        
        // Check permission level
        let role_def = user_role.get_role_definition();
        if plugin.get_max_permission_level() > role_def.plugin_access.max_permission_level {
            return Err(anyhow!("Plugin requires higher permission level than allowed for role"));
        }
        
        // Add to pending operations
        self.pending_operations.insert(plugin_id, PluginOperation {
            plugin_id,
            operation_type: OperationType::Enabling,
            progress: 0.0,
            status_message: "启用插件...".to_string(),
        });
        
        // Send task to background processor
        if let Some(sender) = &self.task_sender {
            sender.send(PluginTask::Enable(plugin_id))?;
        }
        
        Ok(())
    }
    
    /// Disable a plugin
    pub fn disable_plugin(&mut self, plugin_id: Uuid) -> Result<()> {
        let plugin = self.plugins.get_mut(&plugin_id)
            .ok_or_else(|| anyhow!("Plugin not found"))?;
        
        if !plugin.can_be_disabled() {
            return Err(anyhow!("Plugin cannot be disabled in current state"));
        }
        
        // Add to pending operations
        self.pending_operations.insert(plugin_id, PluginOperation {
            plugin_id,
            operation_type: OperationType::Disabling,
            progress: 0.0,
            status_message: "禁用插件...".to_string(),
        });
        
        // Send task to background processor
        if let Some(sender) = &self.task_sender {
            sender.send(PluginTask::Disable(plugin_id))?;
        }
        
        Ok(())
    }
    
    /// Get current operation status for a plugin
    pub fn get_operation_status(&self, plugin_id: &Uuid) -> Option<&PluginOperation> {
        self.pending_operations.get(plugin_id)
    }
    
    /// Get all pending operations
    pub fn get_pending_operations(&self) -> Vec<&PluginOperation> {
        self.pending_operations.values().collect()
    }
    
    /// Load installed plugins from disk
    fn load_installed_plugins(&mut self) {
        log::info!("Loading installed plugins from: {:?}", self.install_dir);
        
        if !self.install_dir.exists() {
            return;
        }
        
        // TODO: Implement plugin loading from disk
        // This would scan the installation directory and load plugin manifests
    }
    
    /// Start background task processor
    fn start_background_processor(&mut self) {
        // TODO: Implement background task processing
        // This would handle actual plugin installation, updates, etc.
        log::info!("Starting plugin manager background processor");
    }
    
    /// Process completed operations
    fn process_completed_operations(&mut self) {
        // TODO: Check for completed background operations and update plugin states
    }
    
    /// Get marketplace
    pub fn get_marketplace(&self) -> &PluginMarketplace {
        &self.marketplace
    }
    
    /// Get marketplace (mutable)
    pub fn get_marketplace_mut(&mut self) -> &mut PluginMarketplace {
        &mut self.marketplace
    }
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}
