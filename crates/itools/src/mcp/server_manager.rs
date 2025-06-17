use std::collections::HashMap;
use std::path::PathBuf;
use uuid::Uuid;
use serde::{Deserialize, Serialize};
use anyhow::Result;
use tokio::sync::mpsc;

use super::rmcp_client::{RmcpClient, ConnectionStatus, ServerCapabilities, McpEvent, ServerHealthStatus, TestResult};

/// MCP Server Manager for managing server configurations and connections
#[derive(Debug)]
pub struct McpServerManager {
    /// RMCP client for server communication
    client: RmcpClient,
    
    /// Server configurations organized by directory
    server_directories: HashMap<String, Vec<McpServerConfig>>,
    
    /// Configuration file path
    config_path: PathBuf,
}

/// MCP Server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerConfig {
    #[serde(default = "Uuid::new_v4")]
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub transport: TransportConfig,
    pub enabled: bool,
    pub auto_start: bool,
    pub directory: String,
    pub metadata: HashMap<String, String>,

    // 静态能力信息（从配置文件中读取）
    #[serde(default)]
    pub capabilities: Option<serde_json::Value>,

    // 持久化状态信息
    #[serde(default)]
    pub last_health_status: Option<ServerHealthStatus>,
    #[serde(default)]
    pub last_test_time: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(default)]
    pub last_test_success: Option<bool>,
}

/// Transport configuration for MCP servers
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum TransportConfig {
    #[serde(rename = "command")]
    Command {
        command: String,
        args: Vec<String>,
        env: HashMap<String, String>,
    },
    #[serde(rename = "tcp")]
    Tcp {
        host: String,
        port: u16,
    },
    #[serde(rename = "unix")]
    Unix {
        socket_path: String,
    },
    #[serde(rename = "websocket")]
    WebSocket {
        url: String,
    },
}

/// MCP Server information for UI display
#[derive(Debug, Clone)]
pub struct McpServerInfo {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub status: ConnectionStatus,
    pub health_status: ServerHealthStatus,
    pub capabilities: Option<ServerCapabilities>,
    pub last_ping: Option<chrono::DateTime<chrono::Utc>>,
    pub last_test_time: Option<chrono::DateTime<chrono::Utc>>,
    pub test_results: Vec<TestResult>,
}

/// Server directory information
#[derive(Debug, Clone)]
pub struct ServerDirectory {
    pub name: String,
    pub path: String,
    pub servers: Vec<McpServerConfig>,
    pub expanded: bool,
}

impl McpServerManager {
    /// Create a new server manager
    pub fn new(config_path: PathBuf) -> Self {
        Self {
            client: RmcpClient::new(),
            server_directories: HashMap::new(),
            config_path,
        }
    }

    /// Initialize the server manager
    pub async fn initialize(&mut self) -> Result<()> {
        // Clear existing data to avoid duplicates
        self.server_directories.clear();

        self.load_configuration().await?;
        self.setup_default_directories();
        Ok(())
    }

    /// Set event sender for MCP events
    pub fn set_event_sender(&mut self, sender: mpsc::UnboundedSender<McpEvent>) {
        self.client.set_event_sender(sender);
    }

    /// Add event sender for MCP events (支持多个接收器)
    pub fn add_event_sender(&mut self, sender: mpsc::UnboundedSender<McpEvent>) {
        self.client.add_event_sender(sender);
    }

    /// Initialize the server manager synchronously (for use in non-async contexts)
    pub fn initialize_sync(&mut self) -> Result<()> {
        // Setup default directories first
        self.setup_default_directories();

        // Load configuration synchronously
        self.load_configuration_sync()?;

        Ok(())
    }

    /// Load server configurations from file synchronously
    fn load_configuration_sync(&mut self) -> Result<()> {
        if !self.config_path.exists() {
            log::info!("Configuration file not found: {:?}", self.config_path);
            return Ok(());
        }

        let content = std::fs::read_to_string(&self.config_path)?;
        let configs: Vec<McpServerConfig> = serde_json::from_str(&content)?;

        // Clear existing data to avoid duplicates
        self.server_directories.clear();
        self.setup_default_directories();

        // Organize configs by directory and ensure they have IDs
        for mut config in configs {
            // Ensure config has an ID (for backward compatibility)
            if config.id == Uuid::nil() {
                config.id = Uuid::new_v4();
            }

            let directory = config.directory.clone();
            self.server_directories
                .entry(directory)
                .or_insert_with(Vec::new)
                .push(config.clone());

            // Also add to client
            self.client.add_server_config(config);
        }

        log::info!("Loaded {} server configurations from {:?}", self.get_total_server_count(), self.config_path);
        Ok(())
    }

    /// Load server configurations from file
    async fn load_configuration(&mut self) -> Result<()> {
        if !self.config_path.exists() {
            log::info!("Configuration file not found, creating default configuration");
            self.create_default_configuration().await?;
            return Ok(());
        }

        let content = tokio::fs::read_to_string(&self.config_path).await?;
        let configs: Vec<McpServerConfig> = serde_json::from_str(&content)?;

        // Organize configs by directory and ensure they have IDs
        for mut config in configs {
            // Ensure config has an ID (for backward compatibility)
            if config.id == Uuid::nil() {
                config.id = Uuid::new_v4();
            }

            let directory = config.directory.clone();
            self.server_directories
                .entry(directory)
                .or_insert_with(Vec::new)
                .push(config.clone());

            // Also add to client
            self.client.add_server_config(config);
        }

        log::info!("Loaded {} server configurations", self.get_total_server_count());
        Ok(())
    }

    /// Save server configurations to file
    pub async fn save_configuration(&self) -> Result<()> {
        let mut all_configs = Vec::new();
        for configs in self.server_directories.values() {
            all_configs.extend(configs.iter().cloned());
        }

        let content = serde_json::to_string_pretty(&all_configs)?;
        tokio::fs::write(&self.config_path, content).await?;

        log::info!("Saved {} server configurations", all_configs.len());
        Ok(())
    }

    /// Save configuration after test (with updated status from client)
    async fn save_configuration_after_test(&mut self) -> Result<()> {
        // 从客户端获取更新后的配置
        let updated_configs = self.client.get_all_server_configs();

        // 更新本地配置
        self.server_directories.clear();
        for config in updated_configs {
            let directory = config.directory.clone();
            self.server_directories
                .entry(directory)
                .or_insert_with(Vec::new)
                .push(config);
        }

        // 保存到文件
        self.save_configuration().await
    }

    /// Create default configuration
    async fn create_default_configuration(&mut self) -> Result<()> {
        // Create empty configuration - no default servers
        // Users can add servers through the UI or import configurations

        self.save_configuration().await?;
        Ok(())
    }

    /// Setup default directories
    fn setup_default_directories(&mut self) {
        let default_dirs = vec!["Examples", "Local", "Remote", "Custom"];
        
        for dir in default_dirs {
            self.server_directories.entry(dir.to_string()).or_insert_with(Vec::new);
        }
    }

    /// Add a new server configuration
    pub async fn add_server(&mut self, mut config: McpServerConfig) -> Result<Uuid> {
        // Ensure the config has a unique ID
        if config.id == Uuid::nil() {
            config.id = Uuid::new_v4();
        }

        let directory = config.directory.clone();
        let server_id = config.id;

        // Add to directory
        self.server_directories
            .entry(directory)
            .or_insert_with(Vec::new)
            .push(config.clone());

        // Add to client
        self.client.add_server_config(config);

        // Save configuration
        self.save_configuration().await?;

        log::info!("Added new server: {}", server_id);
        Ok(server_id)
    }

    /// Remove a server configuration
    pub async fn remove_server(&mut self, server_id: Uuid) -> Result<()> {
        // Find and remove from directories
        let mut found = false;
        for configs in self.server_directories.values_mut() {
            if let Some(pos) = configs.iter().position(|c| c.id == server_id) {
                configs.remove(pos);
                found = true;
                break;
            }
        }

        if !found {
            return Err(anyhow::anyhow!("Server configuration not found"));
        }

        // Remove from client
        self.client.remove_server_config(server_id)?;

        // Save configuration
        self.save_configuration().await?;

        log::info!("Removed server: {}", server_id);
        Ok(())
    }

    /// Update a server configuration
    pub async fn update_server(&mut self, server_id: Uuid, updated_config: McpServerConfig) -> Result<()> {
        // Validate the updated configuration
        self.validate_server_config(&updated_config)?;

        // Find and update in directories
        let mut found = false;
        let mut old_directory = String::new();

        for (directory, configs) in self.server_directories.iter_mut() {
            if let Some(pos) = configs.iter().position(|c| c.id == server_id) {
                old_directory = directory.clone();
                configs.remove(pos);
                found = true;
                break;
            }
        }

        if !found {
            return Err(anyhow::anyhow!("Server configuration not found"));
        }

        // Add to new directory (might be the same)
        let new_directory = updated_config.directory.clone();
        self.server_directories
            .entry(new_directory.clone())
            .or_insert_with(Vec::new)
            .push(updated_config.clone());

        // Update in client (this will reset health status to Red)
        self.client.update_server_config(updated_config)?;

        // Save configuration
        self.save_configuration().await?;

        log::info!("Updated server: {} (moved from '{}' to '{}')", server_id, old_directory, new_directory);
        Ok(())
    }

    /// Get server directories for UI display
    pub fn get_server_directories(&self) -> Vec<ServerDirectory> {
        let mut directories: Vec<_> = self.server_directories
            .iter()
            .map(|(name, servers)| ServerDirectory {
                name: name.clone(),
                path: name.clone(),
                servers: servers.clone(),
                expanded: false,
            })
            .collect();

        // Sort directories by name for consistent ordering
        directories.sort_by(|a, b| a.name.cmp(&b.name));

        directories
    }

    /// Connect to a server
    pub async fn connect_server(&mut self, server_id: Uuid) -> Result<()> {
        self.client.connect_server(server_id).await
    }

    /// Disconnect from a server
    pub async fn disconnect_server(&mut self, server_id: Uuid) -> Result<()> {
        self.client.disconnect_server(server_id)
    }

    /// Test server connection
    pub async fn test_server(&mut self, server_id: Uuid) -> Result<bool> {
        let result = self.client.test_server(server_id).await?;

        // 测试完成后自动保存配置以持久化状态
        if let Err(e) = self.save_configuration_after_test().await {
            log::warn!("Failed to save configuration after test: {}", e);
        }

        Ok(result)
    }

    /// Test server connection with detailed output
    pub async fn test_server_detailed(&mut self, server_id: Uuid) -> Result<crate::mcp::rmcp_client::TestResult> {
        let result = self.client.test_server_detailed(server_id).await?;

        // 测试完成后自动保存配置以持久化状态
        if let Err(e) = self.save_configuration_after_test().await {
            log::warn!("Failed to save configuration after test: {}", e);
        }

        Ok(result)
    }

    /// Get server information
    pub fn get_server_info(&self, server_id: Uuid) -> Option<McpServerInfo> {
        self.client.get_server_info(server_id)
    }

    /// List all servers
    pub fn list_servers(&self) -> Vec<McpServerInfo> {
        self.client.list_servers()
    }

    /// Get total server count
    pub fn get_total_server_count(&self) -> usize {
        self.server_directories.values().map(|v| v.len()).sum()
    }

    /// Call a tool on a specific server
    pub async fn call_tool(&mut self, server_id: Uuid, tool_name: &str, arguments: serde_json::Value) -> Result<serde_json::Value> {
        self.client.call_tool(server_id, tool_name, arguments).await
    }

    /// Call a tool on a specific server for testing purposes (bypasses health status check)
    pub async fn call_tool_for_testing(&mut self, server_id: Uuid, tool_name: &str, arguments: serde_json::Value) -> Result<serde_json::Value> {
        self.client.call_tool_for_testing(server_id, tool_name, arguments).await
    }

    /// Read a resource from a specific server
    pub async fn read_resource(&mut self, server_id: Uuid, uri: &str) -> Result<serde_json::Value> {
        self.client.read_resource(server_id, uri).await
    }

    /// Get a prompt from a specific server
    pub async fn get_prompt(&mut self, server_id: Uuid, prompt_name: &str, arguments: Option<serde_json::Value>) -> Result<serde_json::Value> {
        self.client.get_prompt(server_id, prompt_name, arguments).await
    }

    /// Test server functionality and update health status
    pub async fn test_server_functionality(&mut self, server_id: Uuid) -> Result<TestResult> {
        let result = self.client.test_server_functionality(server_id).await?;

        // 测试完成后自动保存配置以持久化状态
        if let Err(e) = self.save_configuration_after_test().await {
            log::warn!("Failed to save configuration after test: {}", e);
        }

        Ok(result)
    }

    /// Check if server is ready for operations (Green health status)
    pub fn is_server_ready(&self, server_id: Uuid) -> Result<bool> {
        self.client.is_server_ready(server_id)
    }

    /// Get server health status
    pub fn get_server_health_status(&self, server_id: Uuid) -> Option<ServerHealthStatus> {
        self.client.get_server_health_status(server_id)
    }

    /// Get server capabilities
    pub fn get_server_capabilities(&self, server_id: Uuid) -> Option<super::rmcp_client::ServerCapabilities> {
        self.client.get_server_capabilities(server_id)
    }

    /// Get server test results
    pub fn get_server_test_results(&self, server_id: Uuid) -> Option<Vec<TestResult>> {
        self.client.get_server_test_results(server_id)
    }

    /// Ensure server is connected and ready for operations
    pub async fn ensure_server_connected(&mut self, server_id: Uuid) -> Result<()> {
        self.client.ensure_server_connected(server_id).await
    }

    /// Import server configuration from file
    pub async fn import_server_config(&mut self, file_path: PathBuf) -> Result<Vec<Uuid>> {
        let content = tokio::fs::read_to_string(file_path).await?;
        let configs: Vec<McpServerConfig> = serde_json::from_str(&content)?;
        
        let mut server_ids = Vec::new();
        for config in configs {
            let server_id = self.add_server(config).await?;
            server_ids.push(server_id);
        }
        
        log::info!("Imported {} server configurations", server_ids.len());
        Ok(server_ids)
    }

    /// Export server configurations to file
    pub async fn export_server_configs(&self, file_path: PathBuf, directory: Option<String>) -> Result<()> {
        let configs = if let Some(dir) = directory {
            self.server_directories.get(&dir).cloned().unwrap_or_default()
        } else {
            let mut all_configs = Vec::new();
            for configs in self.server_directories.values() {
                all_configs.extend(configs.iter().cloned());
            }
            all_configs
        };

        let content = serde_json::to_string_pretty(&configs)?;
        tokio::fs::write(file_path, content).await?;
        
        log::info!("Exported {} server configurations", configs.len());
        Ok(())
    }

    /// Validate server configuration
    pub fn validate_server_config(&self, config: &McpServerConfig) -> Result<()> {
        if config.name.trim().is_empty() {
            return Err(anyhow::anyhow!("Server name cannot be empty"));
        }

        match &config.transport {
            TransportConfig::Command { command, .. } => {
                if command.trim().is_empty() {
                    return Err(anyhow::anyhow!("Command cannot be empty"));
                }
            }
            TransportConfig::Tcp { host, port } => {
                if host.trim().is_empty() {
                    return Err(anyhow::anyhow!("Host cannot be empty"));
                }
                if *port == 0 {
                    return Err(anyhow::anyhow!("Port must be greater than 0"));
                }
            }
            TransportConfig::Unix { socket_path } => {
                if socket_path.trim().is_empty() {
                    return Err(anyhow::anyhow!("Socket path cannot be empty"));
                }
            }
            TransportConfig::WebSocket { url } => {
                if url.trim().is_empty() {
                    return Err(anyhow::anyhow!("WebSocket URL cannot be empty"));
                }
            }
        }

        Ok(())
    }
}
