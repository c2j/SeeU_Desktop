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

        // Load installed plugins asynchronously to avoid blocking startup
        let install_dir = self.install_dir.clone();

        std::thread::spawn(move || {
            // Plugin loading logic would go here
            // For now, just do it asynchronously without logging
        });

        // Initialize marketplace asynchronously
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
            if let Err(e) = std::fs::create_dir_all(&self.install_dir) {
                log::error!("Failed to create plugin directory: {}", e);
                return;
            }
        }

        // Scan plugin directory for installed plugins
        match std::fs::read_dir(&self.install_dir) {
            Ok(entries) => {
                for entry in entries.flatten() {
                    if entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false) {
                        self.load_plugin_from_directory(&entry.path());
                    }
                }
            }
            Err(e) => {
                log::error!("Failed to read plugin directory: {}", e);
            }
        }
    }

    /// Load a plugin from a directory
    fn load_plugin_from_directory(&mut self, plugin_dir: &std::path::Path) {
        let manifest_path = plugin_dir.join("manifest.json");
        if !manifest_path.exists() {
            return;
        }

        match std::fs::read_to_string(&manifest_path) {
            Ok(content) => {
                match serde_json::from_str::<super::plugin::PluginManifest>(&content) {
                    Ok(manifest) => {
                        // Try to load metadata
                        let metadata_path = plugin_dir.join("metadata.json");
                        match std::fs::read_to_string(&metadata_path) {
                            Ok(metadata_content) => {
                                match serde_json::from_str::<super::plugin::PluginMetadata>(&metadata_content) {
                                    Ok(metadata) => {
                                        let mut plugin = super::plugin::Plugin::new(metadata, manifest);
                                        plugin.status = super::plugin::PluginStatus::Installed;
                                        plugin.installation_path = Some(plugin_dir.to_path_buf());

                                        // Try to load installation timestamp
                                        if let Ok(metadata) = std::fs::metadata(&manifest_path) {
                                            if let Ok(created) = metadata.created() {
                                                plugin.installed_at = Some(chrono::DateTime::from(created));
                                            }
                                        }

                                        self.plugins.insert(plugin.id, plugin);
                                        log::info!("Loaded plugin from: {:?}", plugin_dir);
                                    }
                                    Err(e) => {
                                        log::error!("Failed to parse plugin metadata from {:?}: {}", metadata_path, e);
                                    }
                                }
                            }
                            Err(e) => {
                                log::error!("Failed to read plugin metadata from {:?}: {}", metadata_path, e);
                            }
                        }
                    }
                    Err(e) => {
                        log::error!("Failed to parse plugin manifest from {:?}: {}", manifest_path, e);
                    }
                }
            }
            Err(e) => {
                log::error!("Failed to read plugin manifest from {:?}: {}", manifest_path, e);
            }
        }
    }
    
    /// Start background task processor
    fn start_background_processor(&mut self) {
        log::info!("Starting plugin manager background processor");

        let (sender, receiver) = mpsc::unbounded_channel();
        self.task_sender = Some(sender);
        self.task_receiver = Some(receiver);

        // Note: In a real implementation, we would spawn a background thread here
        // For now, we'll process tasks synchronously in the update method
    }
    
    /// Process completed operations
    fn process_completed_operations(&mut self) {
        // Collect tasks first to avoid borrowing issues
        let mut tasks = Vec::new();
        if let Some(receiver) = &mut self.task_receiver {
            while let Ok(task) = receiver.try_recv() {
                tasks.push(task);
            }
        }

        // Process collected tasks
        for task in tasks {
            self.process_task(task);
        }
    }

    /// Process a single task
    fn process_task(&mut self, task: PluginTask) {
        match task {
            PluginTask::Install(plugin_id, source_url) => {
                self.process_install_task(plugin_id, source_url);
            }
            PluginTask::Uninstall(plugin_id) => {
                self.process_uninstall_task(plugin_id);
            }
            PluginTask::Enable(plugin_id) => {
                self.process_enable_task(plugin_id);
            }
            PluginTask::Disable(plugin_id) => {
                self.process_disable_task(plugin_id);
            }
            PluginTask::Update(plugin_id) => {
                self.process_update_task(plugin_id);
            }
            PluginTask::RefreshMarketplace => {
                self.marketplace.refresh_marketplace();
            }
        }
    }

    /// Process plugin installation
    fn process_install_task(&mut self, plugin_id: uuid::Uuid, source_url: String) {
        log::info!("Processing install task for plugin: {}", plugin_id);

        // Update progress
        if let Some(operation) = self.pending_operations.get_mut(&plugin_id) {
            operation.progress = 0.1;
            operation.status_message = "下载插件...".to_string();
        }

        // Get plugin from marketplace
        let marketplace_plugin = match self.marketplace.get_plugin(&plugin_id) {
            Some(plugin) => plugin.clone(),
            None => {
                log::error!("Plugin not found in marketplace: {}", plugin_id);
                self.complete_operation_with_error(plugin_id, "插件在市场中未找到".to_string());
                return;
            }
        };

        // Create plugin directory
        let plugin_dir = self.install_dir.join(plugin_id.to_string());
        if let Err(e) = std::fs::create_dir_all(&plugin_dir) {
            log::error!("Failed to create plugin directory: {}", e);
            self.complete_operation_with_error(plugin_id, format!("创建插件目录失败: {}", e));
            return;
        }

        // Update progress
        if let Some(operation) = self.pending_operations.get_mut(&plugin_id) {
            operation.progress = 0.5;
            operation.status_message = "安装插件文件...".to_string();
        }

        // Save plugin manifest
        let manifest_path = plugin_dir.join("manifest.json");
        match serde_json::to_string_pretty(&marketplace_plugin.plugin.manifest) {
            Ok(manifest_json) => {
                if let Err(e) = std::fs::write(&manifest_path, manifest_json) {
                    log::error!("Failed to write plugin manifest: {}", e);
                    self.complete_operation_with_error(plugin_id, format!("写入插件清单失败: {}", e));
                    return;
                }
            }
            Err(e) => {
                log::error!("Failed to serialize plugin manifest: {}", e);
                self.complete_operation_with_error(plugin_id, format!("序列化插件清单失败: {}", e));
                return;
            }
        }

        // Save plugin metadata
        let metadata_path = plugin_dir.join("metadata.json");
        match serde_json::to_string_pretty(&marketplace_plugin.plugin.metadata) {
            Ok(metadata_json) => {
                if let Err(e) = std::fs::write(&metadata_path, metadata_json) {
                    log::error!("Failed to write plugin metadata: {}", e);
                    self.complete_operation_with_error(plugin_id, format!("写入插件元数据失败: {}", e));
                    return;
                }
            }
            Err(e) => {
                log::error!("Failed to serialize plugin metadata: {}", e);
                self.complete_operation_with_error(plugin_id, format!("序列化插件元数据失败: {}", e));
                return;
            }
        }

        // Create plugin instance
        let mut plugin = marketplace_plugin.plugin.clone();
        plugin.status = super::plugin::PluginStatus::Installed;
        plugin.installation_path = Some(plugin_dir);
        plugin.installed_at = Some(chrono::Utc::now());

        // Add to installed plugins
        self.plugins.insert(plugin_id, plugin);

        // Complete operation
        self.pending_operations.remove(&plugin_id);
        log::info!("Plugin installation completed: {}", plugin_id);
    }

    /// Process plugin uninstallation
    fn process_uninstall_task(&mut self, plugin_id: uuid::Uuid) {
        log::info!("Processing uninstall task for plugin: {}", plugin_id);

        if let Some(plugin) = self.plugins.get(&plugin_id) {
            if let Some(install_path) = &plugin.installation_path {
                if let Err(e) = std::fs::remove_dir_all(install_path) {
                    log::error!("Failed to remove plugin directory: {}", e);
                    self.complete_operation_with_error(plugin_id, format!("删除插件目录失败: {}", e));
                    return;
                }
            }
        }

        // Remove from installed plugins
        self.plugins.remove(&plugin_id);

        // Complete operation
        self.pending_operations.remove(&plugin_id);
        log::info!("Plugin uninstallation completed: {}", plugin_id);
    }

    /// Process plugin enabling
    fn process_enable_task(&mut self, plugin_id: uuid::Uuid) {
        log::info!("Processing enable task for plugin: {}", plugin_id);

        if let Some(plugin) = self.plugins.get_mut(&plugin_id) {
            plugin.status = super::plugin::PluginStatus::Enabled;
            plugin.last_updated = Some(chrono::Utc::now());
        }

        // Complete operation
        self.pending_operations.remove(&plugin_id);
        log::info!("Plugin enabling completed: {}", plugin_id);
    }

    /// Process plugin disabling
    fn process_disable_task(&mut self, plugin_id: uuid::Uuid) {
        log::info!("Processing disable task for plugin: {}", plugin_id);

        if let Some(plugin) = self.plugins.get_mut(&plugin_id) {
            plugin.status = super::plugin::PluginStatus::Disabled;
            plugin.last_updated = Some(chrono::Utc::now());
        }

        // Complete operation
        self.pending_operations.remove(&plugin_id);
        log::info!("Plugin disabling completed: {}", plugin_id);
    }

    /// Process plugin update
    fn process_update_task(&mut self, plugin_id: uuid::Uuid) {
        log::info!("Processing update task for plugin: {}", plugin_id);

        // TODO: Implement plugin update logic
        // This would download new version and replace existing files

        // Complete operation
        self.pending_operations.remove(&plugin_id);
        log::info!("Plugin update completed: {}", plugin_id);
    }

    /// Complete operation with error
    fn complete_operation_with_error(&mut self, plugin_id: uuid::Uuid, error_message: String) {
        if let Some(plugin) = self.plugins.get_mut(&plugin_id) {
            plugin.status = super::plugin::PluginStatus::Error(error_message.clone());
        }
        self.pending_operations.remove(&plugin_id);
        log::error!("Plugin operation failed for {}: {}", plugin_id, error_message);
    }
    
    /// Get marketplace
    pub fn get_marketplace(&self) -> &PluginMarketplace {
        &self.marketplace
    }
    
    /// Get marketplace (mutable)
    pub fn get_marketplace_mut(&mut self) -> &mut PluginMarketplace {
        &mut self.marketplace
    }

    /// Get count of installed plugins
    pub fn get_installed_count(&self) -> usize {
        self.plugins.len()
    }

    /// Get count of active plugins
    pub fn get_active_count(&self) -> usize {
        self.plugins.values()
            .filter(|plugin| matches!(plugin.status, PluginStatus::Enabled))
            .count()
    }
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}
