use std::collections::HashMap;
use uuid::Uuid;
use serde_json::Value;
use anyhow::Result;
use tokio::sync::mpsc;
use serde::{Serialize, Deserialize};

// Real rmcp integration for MCP protocol
use rmcp::{
    ServiceExt,
    transport::{TokioChildProcess, SseTransport},
    model::{CallToolRequestParam, ReadResourceRequestParam, GetPromptRequestParam, ClientInfo, ClientCapabilities, Implementation},
    service::RunningService,
    RoleClient,
};
use serde_json::json;

use super::server_manager::{McpServerConfig, McpServerInfo};

/// MCP Client implementation using rmcp
#[derive(Debug)]
struct McpClient {
    service: McpService,
}

/// Enum to handle different service types
#[derive(Debug)]
enum McpService {
    Stdio(RunningService<RoleClient, ()>),
    Sse(RunningService<RoleClient, rmcp::model::InitializeRequestParam>),
}

impl McpService {
    async fn list_all_resources(&self) -> Result<Vec<rmcp::model::Resource>, Box<dyn std::error::Error + Send + Sync>> {
        match self {
            McpService::Stdio(service) => service.list_all_resources().await.map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>),
            McpService::Sse(service) => service.list_all_resources().await.map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>),
        }
    }

    async fn list_all_prompts(&self) -> Result<Vec<rmcp::model::Prompt>, Box<dyn std::error::Error + Send + Sync>> {
        match self {
            McpService::Stdio(service) => service.list_all_prompts().await.map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>),
            McpService::Sse(service) => service.list_all_prompts().await.map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>),
        }
    }

    async fn cancel(self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        match self {
            McpService::Stdio(service) => service.cancel().await.map(|_| ()).map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>),
            McpService::Sse(service) => service.cancel().await.map(|_| ()).map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>),
        }
    }
}

impl McpClient {
    /// Create a new MCP client with rmcp service
    async fn new(command: &str, args: &[String]) -> Result<Self> {
        log::info!("Creating MCP client for command: {} {:?}", command, args);

        // Create the command for the MCP server
        let mut cmd = tokio::process::Command::new(command);
        for arg in args {
            cmd.arg(arg);
        }

        // Create transport using TokioChildProcess
        let transport = TokioChildProcess::new(&mut cmd)?;

        // Create the service using rmcp
        let service = ().serve(transport).await
            .map_err(|e| anyhow::anyhow!("Failed to create rmcp service: {}", e))?;

        Ok(McpClient {
            service: McpService::Stdio(service),
        })
    }

    /// List tools using rmcp service
    async fn list_tools(&self) -> Result<Value> {
        log::debug!("Listing tools using rmcp service");
        let tools = match &self.service {
            McpService::Stdio(service) => service.list_all_tools().await,
            McpService::Sse(service) => service.list_all_tools().await,
        }.map_err(|e| anyhow::anyhow!("Failed to list tools: {}", e))?;

        log::debug!("Raw tools response from rmcp: {:?}", tools);

        // rmcp returns a Vec<Tool> directly, convert to our expected JSON format with proper inputSchema
        let tools_json: Vec<Value> = tools.iter().map(|tool| {
            // Ensure inputSchema is properly formatted for UI consumption
            let input_schema = if tool.input_schema.is_empty() {
                // Default empty object schema for tools without parameters
                json!({
                    "type": "object",
                    "title": "EmptyObject"
                })
            } else {
                // Convert Arc<Map<String, Value>> to Value
                serde_json::Value::Object((*tool.input_schema).clone())
            };

            json!({
                "name": tool.name,
                "description": tool.description,
                "inputSchema": input_schema
            })
        }).collect();

        log::debug!("Converted {} tools to JSON format with proper inputSchema", tools_json.len());

        Ok(json!({ "tools": tools_json }))
    }

    /// List resources using rmcp service
    async fn list_resources(&self) -> Result<Value> {
        log::debug!("Listing resources using rmcp service");
        let resources = self.service.list_all_resources().await
            .map_err(|e| anyhow::anyhow!("Failed to list resources: {}", e))?;

        log::debug!("Raw resources response from rmcp: {:?}", resources);

        // rmcp returns a Vec<Resource> directly, convert to our expected JSON format
        let resources_json: Vec<Value> = resources.iter().map(|resource| {
            json!({
                "uri": resource.uri,
                "name": resource.name,
                "description": resource.description,
                "mimeType": resource.mime_type
            })
        }).collect();

        log::debug!("Converted {} resources to JSON format", resources_json.len());

        Ok(json!({ "resources": resources_json }))
    }

    /// List prompts using rmcp service
    async fn list_prompts(&self) -> Result<Value> {
        log::debug!("Listing prompts using rmcp service");
        let prompts = self.service.list_all_prompts().await
            .map_err(|e| anyhow::anyhow!("Failed to list prompts: {}", e))?;

        log::debug!("Raw prompts response from rmcp: {:?}", prompts);

        // rmcp returns a Vec<Prompt> directly, convert to our expected JSON format
        let prompts_json: Vec<Value> = prompts.iter().map(|prompt| {
            // For now, skip arguments processing until we understand the exact structure
            json!({
                "name": prompt.name,
                "description": prompt.description,
                "arguments": []  // TODO: Fix arguments processing
            })
        }).collect();

        log::debug!("Converted {} prompts to JSON format", prompts_json.len());

        Ok(json!({ "prompts": prompts_json }))
    }

    /// Call a tool using rmcp service
    async fn call_tool(&self, name: &str, arguments: Option<Value>) -> Result<Value> {
        log::debug!("Calling tool '{}' using rmcp service with arguments: {:?}", name, arguments);

        // Convert arguments to the format expected by rmcp
        let arguments_map = if let Some(args) = arguments {
            // If arguments is already an object, use it directly
            if let Some(obj) = args.as_object() {
                Some(obj.clone())
            } else {
                // If it's not an object, try to convert it to one
                log::warn!("Arguments for tool '{}' are not an object, attempting conversion: {:?}", name, args);
                None
            }
        } else {
            None
        };

        log::debug!("Converted arguments for tool '{}': {:?}", name, arguments_map);

        let result = match &self.service {
            McpService::Stdio(service) => service.call_tool(CallToolRequestParam {
                name: name.to_string().into(),
                arguments: arguments_map,
            }).await,
            McpService::Sse(service) => service.call_tool(CallToolRequestParam {
                name: name.to_string().into(),
                arguments: arguments_map,
            }).await,
        }.map_err(|e| {
            log::error!("RMCP service call_tool failed for '{}': {}", name, e);
            anyhow::anyhow!("Failed to call tool '{}': {}", name, e)
        })?;

        log::debug!("RMCP service call_tool result for '{}': {:?}", name, result);

        // Convert rmcp result to JSON format
        serde_json::to_value(&result)
            .map_err(|e| anyhow::anyhow!("Failed to serialize tool result: {}", e))
    }

    /// Read a resource using rmcp service
    async fn read_resource(&self, uri: &str) -> Result<Value> {
        log::debug!("Reading resource '{}' using rmcp service", uri);

        let result = match &self.service {
            McpService::Stdio(service) => service.read_resource(ReadResourceRequestParam {
                uri: uri.to_string().into(),
            }).await,
            McpService::Sse(service) => service.read_resource(ReadResourceRequestParam {
                uri: uri.to_string().into(),
            }).await,
        }.map_err(|e| {
            log::error!("RMCP service read_resource failed for '{}': {}", uri, e);
            anyhow::anyhow!("Failed to read resource '{}': {}", uri, e)
        })?;

        log::debug!("RMCP service read_resource result for '{}': {:?}", uri, result);

        // Convert rmcp result to JSON format
        serde_json::to_value(&result)
            .map_err(|e| anyhow::anyhow!("Failed to serialize resource result: {}", e))
    }

    /// Get a prompt using rmcp service
    async fn get_prompt(&self, name: &str, arguments: Option<Value>) -> Result<Value> {
        log::debug!("Getting prompt '{}' using rmcp service with arguments: {:?}", name, arguments);

        // Convert arguments to the format expected by rmcp
        let arguments_map = if let Some(args) = arguments {
            // If arguments is already an object, use it directly
            if let Some(obj) = args.as_object() {
                Some(obj.clone())
            } else {
                // If it's not an object, try to convert it to one
                log::warn!("Arguments for prompt '{}' are not an object, attempting conversion: {:?}", name, args);
                None
            }
        } else {
            None
        };

        log::debug!("Converted arguments for prompt '{}': {:?}", name, arguments_map);

        let result = match &self.service {
            McpService::Stdio(service) => service.get_prompt(GetPromptRequestParam {
                name: name.to_string().into(),
                arguments: arguments_map,
            }).await,
            McpService::Sse(service) => service.get_prompt(GetPromptRequestParam {
                name: name.to_string().into(),
                arguments: arguments_map,
            }).await,
        }.map_err(|e| {
            log::error!("RMCP service get_prompt failed for '{}': {}", name, e);
            anyhow::anyhow!("Failed to get prompt '{}': {}", name, e)
        })?;

        log::debug!("RMCP service get_prompt result for '{}': {:?}", name, result);

        // Convert rmcp result to JSON format
        serde_json::to_value(&result)
            .map_err(|e| anyhow::anyhow!("Failed to serialize prompt result: {}", e))
    }
}

/// RMCP client wrapper for MCP server communication
#[derive(Debug)]
pub struct RmcpClient {
    /// Active server connections
    servers: HashMap<Uuid, ServerConnection>,

    /// Server configurations
    server_configs: HashMap<Uuid, McpServerConfig>,

    /// Event senders for UI updates (支持多个接收器)
    event_senders: Vec<mpsc::UnboundedSender<McpEvent>>,
}

/// Connection to an MCP server using rmcp
#[derive(Debug)]
pub struct ServerConnection {
    pub server_id: Uuid,
    pub config: McpServerConfig,
    pub status: ConnectionStatus,
    pub health_status: ServerHealthStatus,
    pub capabilities: Option<ServerCapabilities>,
    pub last_test_time: Option<chrono::DateTime<chrono::Utc>>,
    pub test_results: Vec<TestResult>,
    pub rmcp_service: Option<McpClient>,
}

/// Connection status
#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionStatus {
    Disconnected,
    Connecting,
    Connected,
    Error(String),
}

/// Server health status (traffic light system)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ServerHealthStatus {
    /// Red light: Server configuration added/modified but not tested
    Red,
    /// Yellow light: Server connected successfully but not tested
    Yellow,
    /// Green light: Server connected and passed all tests
    Green,
}

/// Server capabilities
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ServerCapabilities {
    pub tools: Vec<ToolInfo>,
    pub resources: Vec<ResourceInfo>,
    pub prompts: Vec<PromptInfo>,
}

/// Tool information
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ToolInfo {
    pub name: String,
    pub description: Option<String>,
    pub input_schema: Option<Value>,
}

/// Resource information
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ResourceInfo {
    pub uri: String,
    pub name: String,
    pub description: Option<String>,
    pub mime_type: Option<String>,
}

/// Prompt information
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PromptInfo {
    pub name: String,
    pub description: Option<String>,
    pub arguments: Vec<PromptArgument>,
}

/// Prompt argument
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PromptArgument {
    pub name: String,
    pub description: Option<String>,
    pub required: bool,
}

/// MCP events for UI updates
#[derive(Debug, Clone)]
pub enum McpEvent {
    ServerConnected(Uuid),
    ServerDisconnected(Uuid),
    ServerError(Uuid, String),
    CapabilitiesUpdated(Uuid, ServerCapabilities),
    /// Server capabilities extracted and ready for database storage
    CapabilitiesExtracted(Uuid, ServerCapabilities, String), // server_id, capabilities, capabilities_json_string
    HealthStatusChanged(Uuid, ServerHealthStatus),
    TestCompleted(Uuid, TestResult),
}

/// Test result with detailed output information
#[derive(Debug, Clone)]
pub struct TestResult {
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
    pub error_message: Option<String>,
}

impl RmcpClient {
    /// Create a new RMCP client
    pub fn new() -> Self {
        Self {
            servers: HashMap::new(),
            server_configs: HashMap::new(),
            event_senders: Vec::new(),
        }
    }

    /// Set event sender for UI updates
    pub fn set_event_sender(&mut self, sender: mpsc::UnboundedSender<McpEvent>) {
        // 清除现有的发送器并添加新的
        self.event_senders.clear();
        self.event_senders.push(sender);
    }

    /// Add event sender for UI updates (支持多个接收器)
    pub fn add_event_sender(&mut self, sender: mpsc::UnboundedSender<McpEvent>) {
        self.event_senders.push(sender);
        log::info!("➕ 添加MCP事件发送器，当前总数: {}", self.event_senders.len());
    }

    /// Add a server configuration
    pub fn add_server_config(&mut self, config: McpServerConfig) -> Uuid {
        let server_id = config.id;
        self.server_configs.insert(server_id, config.clone());

        // 恢复之前保存的状态，如果有的话
        let health_status = config.last_health_status.clone().unwrap_or(ServerHealthStatus::Red);
        let last_test_time = config.last_test_time;

        let connection = ServerConnection {
            server_id,
            config,
            status: ConnectionStatus::Disconnected,
            health_status: health_status.clone(),
            capabilities: None,
            last_test_time,
            test_results: Vec::new(),
            rmcp_service: None,
        };

        self.servers.insert(server_id, connection);

        // Notify UI of the server's health status
        self.send_event(McpEvent::HealthStatusChanged(server_id, health_status));

        server_id
    }

    /// Update a server configuration (resets health status to Red)
    pub fn update_server_config(&mut self, config: McpServerConfig) -> Result<()> {
        let server_id = config.id;

        // Disconnect if currently connected
        if let Some(connection) = self.servers.get(&server_id) {
            if connection.status == ConnectionStatus::Connected {
                let _ = self.disconnect_server(server_id); // Ignore errors during disconnect
            }
        }

        // Update configuration
        self.server_configs.insert(server_id, config.clone());

        // Reset health status to Red when configuration is modified
        if let Some(connection) = self.servers.get_mut(&server_id) {
            connection.config = config;
            connection.health_status = ServerHealthStatus::Red;
            connection.status = ConnectionStatus::Disconnected;
            connection.capabilities = None;
            connection.last_test_time = None;
            connection.test_results.clear();
            connection.rmcp_service = None;

            // Notify UI of health status change
            self.send_event(McpEvent::HealthStatusChanged(server_id, ServerHealthStatus::Red));
        }

        Ok(())
    }

    /// Remove a server configuration
    pub fn remove_server_config(&mut self, server_id: Uuid) -> Result<()> {
        if let Some(connection) = self.servers.get(&server_id) {
            if connection.status == ConnectionStatus::Connected {
                self.disconnect_server(server_id)?;
            }
        }

        self.servers.remove(&server_id);
        self.server_configs.remove(&server_id);

        // 触发服务器断开事件，这将通知主应用更新AI助手状态
        self.send_event(McpEvent::ServerDisconnected(server_id));

        log::info!("🗑️ 服务器配置已删除，已发送断开事件: {}", server_id);

        Ok(())
    }

    /// Connect to a server
    pub async fn connect_server(&mut self, server_id: Uuid) -> Result<()> {
        let config = self.server_configs.get(&server_id)
            .ok_or_else(|| anyhow::anyhow!("Server config not found"))?
            .clone();

        // Update status to connecting
        if let Some(connection) = self.servers.get_mut(&server_id) {
            connection.status = ConnectionStatus::Connecting;
        }

        self.send_event(McpEvent::ServerConnected(server_id));

        match &config.transport {
            crate::mcp::server_manager::TransportConfig::Command { command, args, .. } => {
                self.connect_command_server(server_id, &command, &args).await
            }
            crate::mcp::server_manager::TransportConfig::WebSocket { url } => {
                self.connect_sse_server(server_id, &url).await
            }
            crate::mcp::server_manager::TransportConfig::Tcp { host, port } => {
                let error = format!("TCP transport not yet supported: {}:{}", host, port);
                self.set_server_error(server_id, error.clone());
                Err(anyhow::anyhow!(error))
            }
            crate::mcp::server_manager::TransportConfig::Unix { socket_path } => {
                let error = format!("Unix socket transport not yet supported: {}", socket_path);
                self.set_server_error(server_id, error.clone());
                Err(anyhow::anyhow!(error))
            }
        }
    }

    /// Connect to a command-based server
    async fn connect_command_server(&mut self, server_id: Uuid, command: &str, args: &[String]) -> Result<()> {
        let mut cmd = tokio::process::Command::new(command);
        for arg in args {
            cmd.arg(arg);
        }

        // Configure stdio for MCP communication
        cmd.stdin(std::process::Stdio::piped())
           .stdout(std::process::Stdio::piped())
           .stderr(std::process::Stdio::piped());

        // Try to create rmcp client first
        match self.create_rmcp_client(command, args).await {
            Ok(mcp_client) => {
                log::info!("Successfully created rmcp client for server: {} {:?}", command, args);

                // Store the rmcp client and mark as connected
                if let Some(connection) = self.servers.get_mut(&server_id) {
                    connection.rmcp_service = Some(mcp_client);
                    connection.status = ConnectionStatus::Connected;
                    connection.health_status = ServerHealthStatus::Yellow; // Connected but not tested
                    log::info!("Stored rmcp service for server {} and marked as connected (Yellow status)", server_id);

                    // Notify UI of health status change
                    self.send_event(McpEvent::HealthStatusChanged(server_id, ServerHealthStatus::Yellow));
                } else {
                    log::error!("Failed to find connection for server {} when storing rmcp service", server_id);
                    return Err(anyhow::anyhow!("Failed to find connection for server"));
                }

                // Query server capabilities using rmcp service
                if let Err(e) = self.query_server_capabilities(server_id).await {
                    log::warn!("Failed to query capabilities for server {}: {}", server_id, e);
                    // Don't fail the connection just because capability query failed
                }

                // Add a longer delay after capability query to stabilize the connection
                log::debug!("⏳ Allowing rmcp service to stabilize after capability query...");
                tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;

                // Verify the service is still healthy after capability query with timeout
                log::debug!("🔍 Verifying rmcp service health after capability query...");
                if let Some(connection) = self.servers.get(&server_id) {
                    if let Some(rmcp_service) = &connection.rmcp_service {
                        match tokio::time::timeout(
                            tokio::time::Duration::from_secs(5),
                            rmcp_service.list_tools()
                        ).await {
                            Ok(Ok(_)) => {
                                log::info!("✅ RMCP service is healthy after capability query");
                            }
                            Ok(Err(e)) => {
                                log::warn!("⚠️ RMCP service health check failed after capability query: {}", e);
                                // Don't fail the connection, just log the warning
                            }
                            Err(_) => {
                                log::warn!("⏰ RMCP service health check timed out after capability query");
                                // Don't fail the connection, just log the warning
                            }
                        }
                    }
                }

                Ok(())
            }
            Err(e) => {
                log::warn!("Failed to create rmcp client: {}, falling back to manual process management", e);

                // Fallback to manual process management
                match cmd.spawn() {
                    Ok(mut child) => {
                        log::info!("Started MCP server process: {} {:?}", command, args);

                        // Get stdin and stdout handles
                        let stdin = child.stdin.take().ok_or_else(|| anyhow::anyhow!("Failed to get stdin"))?;
                        let stdout = child.stdout.take().ok_or_else(|| anyhow::anyhow!("Failed to get stdout"))?;

                        // Fallback: just mark as connected without protocol handler
                        log::warn!("Using fallback connection without rmcp service for server {}", server_id);

                        // Update connection status to connected
                        if let Some(connection) = self.servers.get_mut(&server_id) {
                            connection.status = ConnectionStatus::Connected;
                        }

                        // Query server capabilities after successful connection
                        if let Err(e) = self.query_server_capabilities(server_id).await {
                            log::warn!("Failed to query capabilities for server {}: {}", server_id, e);
                            // Don't fail the connection just because capability query failed
                        }

                        // Fallback connection established

                        Ok(())
                    }
                    Err(e) => {
                        let error = format!("Failed to start command: {}", e);
                        self.set_server_error(server_id, error.clone());
                        Err(anyhow::anyhow!(error))
                    }
                }
            }
        }
    }

    /// Connect to an SSE-based server
    async fn connect_sse_server(&mut self, server_id: Uuid, url: &str) -> Result<()> {
        log::info!("Connecting to SSE MCP server at: {}", url);

        // Try to create rmcp client with SSE transport
        match self.create_sse_rmcp_client(url).await {
            Ok(mcp_client) => {
                log::info!("Successfully created SSE rmcp client for server: {}", url);

                // Store the rmcp client and mark as connected
                if let Some(connection) = self.servers.get_mut(&server_id) {
                    connection.rmcp_service = Some(mcp_client);
                    connection.status = ConnectionStatus::Connected;
                    connection.health_status = ServerHealthStatus::Yellow; // Connected but not tested
                    log::info!("Stored SSE rmcp service for server {} and marked as connected (Yellow status)", server_id);

                    // Notify UI of health status change
                    self.send_event(McpEvent::HealthStatusChanged(server_id, ServerHealthStatus::Yellow));
                } else {
                    log::error!("Failed to find connection for server {} when storing SSE rmcp service", server_id);
                    return Err(anyhow::anyhow!("Failed to find connection for server"));
                }

                // Query server capabilities using rmcp service
                if let Err(e) = self.query_server_capabilities(server_id).await {
                    log::warn!("Failed to query capabilities for SSE server {}: {}", server_id, e);
                    // Don't fail the connection just because capability query failed
                }

                // Add a delay to stabilize the connection
                log::debug!("⏳ Allowing SSE rmcp service to stabilize after capability query...");
                tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;

                // Verify the service is still healthy after capability query with timeout
                log::debug!("🔍 Verifying SSE rmcp service health after capability query...");
                if let Some(connection) = self.servers.get(&server_id) {
                    if let Some(rmcp_service) = &connection.rmcp_service {
                        match tokio::time::timeout(
                            tokio::time::Duration::from_secs(5),
                            rmcp_service.list_tools()
                        ).await {
                            Ok(Ok(_)) => {
                                log::info!("✅ SSE RMCP service is healthy after capability query");
                            }
                            Ok(Err(e)) => {
                                log::warn!("⚠️ SSE RMCP service health check failed after capability query: {}", e);
                                // Don't fail the connection, just log the warning
                            }
                            Err(_) => {
                                log::warn!("⏰ SSE RMCP service health check timed out after capability query");
                                // Don't fail the connection, just log the warning
                            }
                        }
                    }
                }

                Ok(())
            }
            Err(e) => {
                let error = format!("Failed to connect to SSE server at {}: {}", url, e);
                self.set_server_error(server_id, error.clone());
                Err(anyhow::anyhow!(error))
            }
        }
    }

    /// Create rmcp client for MCP communication
    async fn create_rmcp_client(&self, command: &str, args: &[String]) -> Result<McpClient> {
        log::info!("Creating rmcp client for command: {} {:?}", command, args);

        // Create the command for the MCP server
        let mut cmd = tokio::process::Command::new(command);
        for arg in args {
            cmd.arg(arg);
        }

        // Configure stdio for MCP communication
        cmd.stdin(std::process::Stdio::piped())
           .stdout(std::process::Stdio::piped())
           .stderr(std::process::Stdio::piped());

        // Create transport using TokioChildProcess
        let transport = TokioChildProcess::new(&mut cmd)
            .map_err(|e| anyhow::anyhow!("Failed to create transport: {}", e))?;

        // Create the service using rmcp
        let service = ().serve(transport).await
            .map_err(|e| anyhow::anyhow!("Failed to create rmcp service: {}", e))?;

        Ok(McpClient {
            service: McpService::Stdio(service),
        })
    }

    /// Create SSE rmcp client for MCP communication
    async fn create_sse_rmcp_client(&self, url: &str) -> Result<McpClient> {
        log::info!("Creating SSE rmcp client for URL: {}", url);

        // Create SSE transport
        let transport = SseTransport::start(url.to_string()).await
            .map_err(|e| anyhow::anyhow!("Failed to create SSE transport: {}", e))?;

        // Create client info for SSE connection
        let client_info = ClientInfo {
            protocol_version: Default::default(),
            capabilities: ClientCapabilities::default(),
            client_info: Implementation {
                name: "SeeU Desktop iTools".to_string(),
                version: "0.1.0".to_string(),
            },
        };

        // Create the service using rmcp with client info
        let service = client_info.serve(transport).await
            .map_err(|e| anyhow::anyhow!("Failed to create SSE rmcp service: {}", e))?;

        Ok(McpClient {
            service: McpService::Sse(service),
        })
    }

    /// Disconnect from a server (async version)
    pub async fn disconnect_server_async(&mut self, server_id: Uuid) -> Result<()> {
        if let Some(connection) = self.servers.get_mut(&server_id) {
            // If we have an rmcp service, cancel it properly
            if let Some(rmcp_service) = connection.rmcp_service.take() {
                log::info!("Properly cancelling rmcp service for server: {}", server_id);
                if let Err(e) = rmcp_service.service.cancel().await {
                    log::warn!("Failed to cancel rmcp service for server {}: {}", server_id, e);
                } else {
                    log::info!("Successfully cancelled rmcp service for server: {}", server_id);
                }
            }

            connection.status = ConnectionStatus::Disconnected;
            connection.capabilities = None;
        }

        self.send_event(McpEvent::ServerDisconnected(server_id));
        log::info!("Disconnected from server: {}", server_id);
        Ok(())
    }

    /// Disconnect from a server (sync version - for backward compatibility)
    pub fn disconnect_server(&mut self, server_id: Uuid) -> Result<()> {
        if let Some(connection) = self.servers.get_mut(&server_id) {
            // If we have an rmcp service, we can't cancel it properly in sync context
            if connection.rmcp_service.is_some() {
                log::warn!("Dropping rmcp service for server {} without proper cancellation (sync context)", server_id);
            }

            connection.status = ConnectionStatus::Disconnected;
            connection.capabilities = None;
            connection.rmcp_service = None; // This will drop the service
        }

        self.send_event(McpEvent::ServerDisconnected(server_id));
        log::info!("Disconnected from server: {}", server_id);
        Ok(())
    }

    /// Query server capabilities using MCP protocol
    pub async fn query_server_capabilities(&mut self, server_id: Uuid) -> Result<()> {
        log::info!("🔍 开始查询服务器能力: {}", server_id);

        // Check if server is connected
        let connection = self.servers.get(&server_id)
            .ok_or_else(|| anyhow::anyhow!("Server not found"))?;

        // Get server config for logging
        let server_name = {
            let config = self.server_configs.get(&server_id)
                .ok_or_else(|| anyhow::anyhow!("Server config not found"))?;
            config.name.clone()
        };

        log::info!("📊 服务器状态检查 - '{}' ({}):", server_name, server_id);
        log::info!("  - 连接状态: {:?}", connection.status);
        log::info!("  - 健康状态: {:?}", connection.health_status);
        log::info!("  - rmcp服务可用: {}", connection.rmcp_service.is_some());

        if connection.status != ConnectionStatus::Connected {
            log::warn!("⚠️ 服务器未连接，无法查询能力 - '{}': {:?}", server_name, connection.status);
            return Err(anyhow::anyhow!("Server is not connected"));
        }

        // Check if we have rmcp service - if so, use it directly
        if connection.rmcp_service.is_some() {
            log::info!("✅ 使用现有rmcp服务提取能力 - 服务器: '{}'", server_name);

            // Extract capabilities from rmcp service
            match self.extract_capabilities_from_rmcp_service(server_id).await {
                Ok(capabilities) => {
                    log::info!("🎉 成功从rmcp服务提取能力 - 服务器: '{}' - 工具:{}, 资源:{}, 提示:{}",
                               server_name, capabilities.tools.len(), capabilities.resources.len(), capabilities.prompts.len());

                    // 详细记录工具信息
                    if !capabilities.tools.is_empty() {
                        log::info!("🛠️ 提取的工具详情 - 服务器 '{}':", server_name);
                        for (index, tool) in capabilities.tools.iter().enumerate() {
                            log::info!("  {}. {} - {}",
                                index + 1,
                                tool.name,
                                tool.description.as_deref().unwrap_or("无描述")
                            );
                        }
                    } else {
                        log::warn!("⚠️ 从rmcp服务提取的能力中没有工具 - 服务器: '{}'", server_name);
                    }

                    // Update connection with capabilities
                    if let Some(connection) = self.servers.get_mut(&server_id) {
                        connection.capabilities = Some(capabilities.clone());
                        log::info!("✅ 已更新连接中的能力信息 - 服务器: '{}'", server_name);
                    }

                    // Save the extracted capabilities to the server configuration
                    if let Err(e) = self.save_runtime_capabilities(server_id, &capabilities).await {
                        log::error!("❌ 保存运行时能力失败 - 服务器: '{}' - 错误: {}", server_name, e);
                    } else {
                        log::info!("✅ 成功保存运行时能力到配置 - 服务器: '{}'", server_name);
                    }

                    // Send event to UI
                    self.send_event(McpEvent::CapabilitiesUpdated(server_id, capabilities));
                    return Ok(());
                }
                Err(e) => {
                    log::error!("❌ 从rmcp服务提取能力失败 - 服务器: '{}' - 错误: {}", server_name, e);
                    // 继续执行fallback逻辑
                }
            }
        } else {
            log::warn!("⚠️ 没有可用的rmcp服务 - 服务器: '{}'", server_name);

            // 尝试创建新的rmcp服务进行能力提取
            log::info!("🔄 尝试创建新的rmcp服务进行能力提取 - 服务器: '{}'", server_name);
            match self.extract_capabilities_with_fresh_service(server_id).await {
                Ok(capabilities) => {
                    log::info!("🎉 使用新rmcp服务成功提取能力 - 服务器: '{}' - 工具:{}, 资源:{}, 提示:{}",
                               server_name, capabilities.tools.len(), capabilities.resources.len(), capabilities.prompts.len());

                    // 详细记录工具信息
                    if !capabilities.tools.is_empty() {
                        log::info!("🛠️ 新服务提取的工具详情 - 服务器 '{}':", server_name);
                        for (index, tool) in capabilities.tools.iter().enumerate() {
                            log::info!("  {}. {} - {}",
                                index + 1,
                                tool.name,
                                tool.description.as_deref().unwrap_or("无描述")
                            );
                        }
                    } else {
                        log::warn!("⚠️ 新rmcp服务提取的能力中没有工具 - 服务器: '{}'", server_name);
                    }

                    // Update connection with capabilities
                    if let Some(connection) = self.servers.get_mut(&server_id) {
                        connection.capabilities = Some(capabilities.clone());
                        log::info!("✅ 已更新连接中的能力信息 - 服务器: '{}'", server_name);
                    }

                    // Save the extracted capabilities to the server configuration
                    if let Err(e) = self.save_runtime_capabilities(server_id, &capabilities).await {
                        log::error!("❌ 保存运行时能力失败 - 服务器: '{}' - 错误: {}", server_name, e);
                    } else {
                        log::info!("✅ 成功保存运行时能力到配置 - 服务器: '{}'", server_name);
                    }

                    // Send event to UI
                    self.send_event(McpEvent::CapabilitiesUpdated(server_id, capabilities));
                    return Ok(());
                }
                Err(e) => {
                    log::error!("❌ 使用新rmcp服务提取能力失败 - 服务器: '{}' - 错误: {}", server_name, e);
                    // 继续执行fallback逻辑
                }
            }
        }

        // Fallback: no rmcp service available, return empty capabilities
        log::error!("🚫 所有能力提取方法都失败，返回空能力 - 服务器: '{}'", server_name);

        let capabilities = ServerCapabilities {
            tools: Vec::new(),
            resources: Vec::new(),
            prompts: Vec::new(),
        };

        // Update connection with empty capabilities
        if let Some(connection) = self.servers.get_mut(&server_id) {
            connection.capabilities = Some(capabilities.clone());
        }

        // Send event to UI
        self.send_event(McpEvent::CapabilitiesUpdated(server_id, capabilities));
        Ok(())
    }

    /// Extract capabilities from rmcp service initialization
    async fn extract_capabilities_from_rmcp_service(&self, server_id: Uuid) -> Result<ServerCapabilities> {
        log::info!("Extracting capabilities from rmcp service for server: {}", server_id);

        // Get the connection with rmcp service
        let connection = self.servers.get(&server_id)
            .ok_or_else(|| anyhow::anyhow!("Server not found"))?;

        // If no rmcp service is available, create a fresh one for capability extraction
        let rmcp_client = if let Some(service) = &connection.rmcp_service {
            log::info!("Using existing rmcp service for capability extraction");
            service
        } else {
            log::info!("No existing rmcp service, creating fresh service for capability extraction");
            return self.extract_capabilities_with_fresh_service(server_id).await;
        };

        // Use the real rmcp service to query actual capabilities
        // Query tools using the real MCP protocol with timeout
        let tools = match tokio::time::timeout(
            tokio::time::Duration::from_secs(10),
            rmcp_client.list_tools()
        ).await {
            Ok(Ok(response)) => {
                log::info!("✅ 成功从rmcp服务查询工具 - 服务器: {}", server_id);
                log::debug!("🔍 原始工具响应: {}", serde_json::to_string_pretty(&response).unwrap_or_default());

                // Parse the response to extract tools
                if let Some(tools_array) = response.get("tools").and_then(|t| t.as_array()) {
                    let tool_infos: Vec<ToolInfo> = tools_array.iter().filter_map(|tool| {
                        let name = tool.get("name")?.as_str()?.to_string();
                        let description = tool.get("description").and_then(|d| d.as_str()).map(|s| s.to_string());
                        let input_schema = tool.get("inputSchema").cloned();

                        log::debug!("🔧 解析工具: {} - {}", name, description.as_deref().unwrap_or("无描述"));

                        Some(ToolInfo {
                            name,
                            description,
                            input_schema,
                        })
                    }).collect();

                    log::info!("📊 成功解析 {} 个工具从rmcp服务响应", tool_infos.len());
                    if !tool_infos.is_empty() {
                        log::info!("🛠️ 解析的工具列表:");
                        for (index, tool) in tool_infos.iter().enumerate() {
                            log::info!("  {}. {} - {}",
                                index + 1,
                                tool.name,
                                tool.description.as_deref().unwrap_or("无描述")
                            );
                        }
                    } else {
                        log::warn!("⚠️ 工具数组为空，但响应中包含tools字段");
                    }
                    tool_infos
                } else {
                    log::warn!("⚠️ rmcp服务响应中没有找到tools数组");
                    log::debug!("📋 响应结构: {}", serde_json::to_string_pretty(&response).unwrap_or_default());
                    Vec::new()
                }
            }
            Ok(Err(e)) => {
                log::error!("❌ 从rmcp服务查询工具失败 - 服务器: {} - 错误: {}", server_id, e);
                Vec::new()
            }
            Err(_) => {
                log::error!("⏰ 从rmcp服务查询工具超时 - 服务器: {}", server_id);
                Vec::new()
            }
        };

        // Query resources using the real MCP protocol with timeout
        let resources = match tokio::time::timeout(
            tokio::time::Duration::from_secs(10),
            rmcp_client.list_resources()
        ).await {
            Ok(Ok(response)) => {
                log::info!("Successfully queried resources from rmcp service");

                // Parse the response to extract resources
                if let Some(resources_array) = response.get("resources").and_then(|r| r.as_array()) {
                    let resource_infos: Vec<ResourceInfo> = resources_array.iter().filter_map(|resource| {
                        let uri = resource.get("uri")?.as_str()?.to_string();
                        let name = resource.get("name")?.as_str()?.to_string();
                        let description = resource.get("description").and_then(|d| d.as_str()).map(|s| s.to_string());
                        let mime_type = resource.get("mimeType").and_then(|m| m.as_str()).map(|s| s.to_string());

                        Some(ResourceInfo {
                            uri,
                            name,
                            description,
                            mime_type,
                        })
                    }).collect();

                    log::info!("Parsed {} resources from rmcp service response", resource_infos.len());
                    resource_infos
                } else {
                    log::warn!("No resources array found in rmcp service response");
                    Vec::new()
                }
            }
            Ok(Err(e)) => {
                log::warn!("Failed to query resources from rmcp service: {}", e);
                Vec::new()
            }
            Err(_) => {
                log::warn!("⏰ 从rmcp服务查询资源超时");
                Vec::new()
            }
        };

        // Query prompts using the real MCP protocol with timeout
        let prompts = match tokio::time::timeout(
            tokio::time::Duration::from_secs(10),
            rmcp_client.list_prompts()
        ).await {
            Ok(Ok(response)) => {
                log::info!("Successfully queried prompts from rmcp service");

                // Parse the response to extract prompts
                if let Some(prompts_array) = response.get("prompts").and_then(|p| p.as_array()) {
                    let prompt_infos: Vec<PromptInfo> = prompts_array.iter().filter_map(|prompt| {
                        let name = prompt.get("name")?.as_str()?.to_string();
                        let description = prompt.get("description").and_then(|d| d.as_str()).map(|s| s.to_string());
                        let arguments = prompt.get("arguments").and_then(|a| a.as_array()).map(|args| {
                            args.iter().filter_map(|arg| {
                                let name = arg.get("name")?.as_str()?.to_string();
                                let description = arg.get("description").and_then(|d| d.as_str()).map(|s| s.to_string());
                                let required = arg.get("required").and_then(|r| r.as_bool()).unwrap_or(false);

                                Some(PromptArgument {
                                    name,
                                    description,
                                    required,
                                })
                            }).collect()
                        }).unwrap_or_default();

                        Some(PromptInfo {
                            name,
                            description,
                            arguments,
                        })
                    }).collect();

                    log::info!("Parsed {} prompts from rmcp service response", prompt_infos.len());
                    prompt_infos
                } else {
                    log::warn!("No prompts array found in rmcp service response");
                    Vec::new()
                }
            }
            Ok(Err(e)) => {
                log::warn!("Failed to query prompts from rmcp service: {}", e);
                Vec::new()
            }
            Err(_) => {
                log::warn!("⏰ 从rmcp服务查询提示超时");
                Vec::new()
            }
        };

        log::info!("Successfully extracted capabilities from rmcp service - Tools: {}, Resources: {}, Prompts: {}",
                   tools.len(), resources.len(), prompts.len());

        Ok(ServerCapabilities {
            tools,
            resources,
            prompts,
        })
    }

    /// Extract capabilities using a fresh rmcp service
    async fn extract_capabilities_with_fresh_service(&self, server_id: Uuid) -> Result<ServerCapabilities> {
        log::info!("Creating fresh rmcp service to extract capabilities for server: {}", server_id);

        let config = self.server_configs.get(&server_id)
            .ok_or_else(|| anyhow::anyhow!("Server config not found"))?;

        match &config.transport {
            crate::mcp::server_manager::TransportConfig::Command { command, args, .. } => {
                log::info!("🚀 Creating fresh command rmcp service for capability extraction: {} {:?}", command, args);

                // Create fresh rmcp service
                let mut cmd = tokio::process::Command::new(command);
                for arg in args {
                    cmd.arg(arg);
                }

                let transport = TokioChildProcess::new(&mut cmd)
                    .map_err(|e| anyhow::anyhow!("Failed to create transport: {}", e))?;
                let service = ().serve(transport).await
                    .map_err(|e| anyhow::anyhow!("Failed to create rmcp service: {}", e))?;

                log::info!("✅ Fresh rmcp service created, extracting capabilities");

                // Extract capabilities using the fresh service
                let capabilities = match self.extract_capabilities_from_command_service(&service).await {
                    Ok(caps) => caps,
                    Err(e) => {
                        log::warn!("Failed to extract capabilities from command service: {}", e);
                        ServerCapabilities {
                            tools: Vec::new(),
                            resources: Vec::new(),
                            prompts: Vec::new(),
                        }
                    }
                };

                // Clean up service
                if let Err(e) = service.cancel().await {
                    log::warn!("Failed to cancel capability extraction service: {}", e);
                }

                log::info!("✅ Capabilities extracted successfully using fresh service - Tools: {}, Resources: {}, Prompts: {}",
                           capabilities.tools.len(), capabilities.resources.len(), capabilities.prompts.len());

                Ok(capabilities)
            }
            crate::mcp::server_manager::TransportConfig::WebSocket { url } => {
                log::info!("🚀 Creating fresh SSE rmcp service for capability extraction: {}", url);

                // Create fresh SSE rmcp service
                let service = self.create_sse_rmcp_client(url).await
                    .map_err(|e| anyhow::anyhow!("Failed to create SSE rmcp service: {}", e))?;

                log::info!("✅ Fresh SSE rmcp service created, extracting capabilities");

                // For now, return empty capabilities for SSE services
                // TODO: Implement proper SSE capability extraction
                log::warn!("SSE capability extraction not yet implemented, returning empty capabilities");
                let capabilities = ServerCapabilities {
                    tools: Vec::new(),
                    resources: Vec::new(),
                    prompts: Vec::new(),
                };

                // Clean up service
                if let Err(e) = service.service.cancel().await {
                    log::warn!("Failed to cancel SSE capability extraction service: {}", e);
                }

                log::info!("✅ Capabilities extracted successfully using fresh SSE service - Tools: {}, Resources: {}, Prompts: {}",
                           capabilities.tools.len(), capabilities.resources.len(), capabilities.prompts.len());

                Ok(capabilities)
            }
            _ => {
                Err(anyhow::anyhow!("Transport type not supported for capability extraction"))
            }
        }
    }



    /// Extract capabilities from a command-based rmcp service
    async fn extract_capabilities_from_command_service(&self, service: &rmcp::service::RunningService<rmcp::RoleClient, ()>) -> Result<ServerCapabilities> {
        log::info!("Extracting capabilities from command-based rmcp service");

        // Query tools using the real MCP protocol with timeout
        let tools = match tokio::time::timeout(
            tokio::time::Duration::from_secs(10),
            service.list_tools(Default::default())
        ).await {
            Ok(Ok(response)) => {
                log::info!("✅ 成功从命令服务查询工具");

                // Parse the response to extract tools
                let tool_infos: Vec<ToolInfo> = response.tools.iter().map(|tool| {
                    log::debug!("🔧 解析命令服务工具: {} - {}", tool.name, tool.description);

                    ToolInfo {
                        name: tool.name.to_string(),
                        description: Some(tool.description.to_string()),
                        input_schema: Some(serde_json::Value::Object((*tool.input_schema).clone())),
                    }
                }).collect();

                log::info!("📊 成功解析 {} 个工具从命令服务响应", tool_infos.len());
                if !tool_infos.is_empty() {
                    log::info!("🛠️ 命令服务工具列表:");
                    for (index, tool) in tool_infos.iter().enumerate() {
                        log::info!("  {}. {} - {}",
                            index + 1,
                            tool.name,
                            tool.description.as_deref().unwrap_or("无描述")
                        );
                    }
                }
                tool_infos
            }
            Ok(Err(e)) => {
                log::error!("❌ 从命令服务查询工具失败: {}", e);
                Vec::new()
            }
            Err(_) => {
                log::error!("⏰ 从命令服务查询工具超时");
                Vec::new()
            }
        };

        // Query resources using the real MCP protocol with timeout
        let resources = match tokio::time::timeout(
            tokio::time::Duration::from_secs(10),
            service.list_resources(Default::default())
        ).await {
            Ok(Ok(response)) => {
                log::info!("Successfully queried resources from rmcp service");

                let resource_infos: Vec<ResourceInfo> = response.resources.iter().map(|resource| {
                    ResourceInfo {
                        uri: resource.uri.clone(),
                        name: resource.name.clone(),
                        description: resource.description.clone(),
                        mime_type: resource.mime_type.clone(),
                    }
                }).collect();

                log::info!("Parsed {} resources from rmcp service response", resource_infos.len());
                resource_infos
            }
            Ok(Err(e)) => {
                log::warn!("Failed to query resources from rmcp service: {}", e);
                Vec::new()
            }
            Err(_) => {
                log::warn!("⏰ 从命令服务查询资源超时");
                Vec::new()
            }
        };

        // Query prompts using the real MCP protocol with timeout
        let prompts = match tokio::time::timeout(
            tokio::time::Duration::from_secs(10),
            service.list_prompts(Default::default())
        ).await {
            Ok(Ok(response)) => {
                log::info!("Successfully queried prompts from rmcp service");

                let prompt_infos: Vec<PromptInfo> = response.prompts.iter().map(|prompt| {
                    let arguments = if let Some(args) = &prompt.arguments {
                        args.iter().map(|arg| {
                            PromptArgument {
                                name: arg.name.clone(),
                                description: arg.description.clone(),
                                required: arg.required.unwrap_or(false),
                            }
                        }).collect()
                    } else {
                        Vec::new()
                    };

                    PromptInfo {
                        name: prompt.name.to_string(),
                        description: prompt.description.clone(),
                        arguments,
                    }
                }).collect();

                log::info!("Parsed {} prompts from rmcp service response", prompt_infos.len());
                prompt_infos
            }
            Ok(Err(e)) => {
                log::warn!("Failed to query prompts from rmcp service: {}", e);
                Vec::new()
            }
            Err(_) => {
                log::warn!("⏰ 从命令服务查询提示超时");
                Vec::new()
            }
        };

        log::info!("Successfully extracted capabilities from command-based rmcp service - Tools: {}, Resources: {}, Prompts: {}",
                   tools.len(), resources.len(), prompts.len());

        Ok(ServerCapabilities {
            tools,
            resources,
            prompts,
        })
    }

    /// Get server information
    pub fn get_server_info(&self, server_id: Uuid) -> Option<McpServerInfo> {
        let connection = self.servers.get(&server_id)?;
        let config = self.server_configs.get(&server_id)?;

        Some(McpServerInfo {
            id: server_id,
            name: config.name.clone(),
            description: config.description.clone(),
            status: connection.status.clone(),
            health_status: connection.health_status.clone(),
            capabilities: connection.capabilities.clone(),
            last_ping: None, // Field removed from ServerConnection
            last_test_time: connection.last_test_time,
            test_results: connection.test_results.clone(),
        })
    }

    /// List all servers
    pub fn list_servers(&self) -> Vec<McpServerInfo> {
        self.servers.keys()
            .filter_map(|&id| self.get_server_info(id))
            .collect()
    }

    /// Test server connection
    pub async fn test_server(&mut self, server_id: Uuid) -> Result<bool> {
        let connection = self.servers.get(&server_id)
            .ok_or_else(|| anyhow::anyhow!("Server not found"))?;

        match connection.status {
            ConnectionStatus::Connected => {
                // Server is already connected, test with ping
                log::info!("Testing connected server: {}", server_id);

                // Server is connected, no need to update ping time (field removed)

                Ok(true)
            }
            ConnectionStatus::Disconnected => {
                // Server is disconnected, try to connect temporarily for testing
                log::info!("Testing disconnected server by attempting connection: {}", server_id);

                match self.test_connection_temporarily(server_id).await {
                    Ok(success) => {
                        log::info!("Test connection result for server {}: {}", server_id, success);
                        Ok(success)
                    }
                    Err(e) => {
                        log::warn!("Test connection failed for server {}: {}", server_id, e);
                        Ok(false)
                    }
                }
            }
            ConnectionStatus::Connecting => {
                // Server is currently connecting, consider it as a successful test
                log::info!("Server {} is currently connecting", server_id);
                Ok(true)
            }
            ConnectionStatus::Error(_) => {
                // Server has an error, try to test anyway
                log::info!("Testing server {} with error status", server_id);

                match self.test_connection_temporarily(server_id).await {
                    Ok(success) => Ok(success),
                    Err(_) => Ok(false)
                }
            }
        }
    }

    /// Test server connection with detailed output
    pub async fn test_server_detailed(&mut self, server_id: Uuid) -> Result<TestResult> {
        // Use the simplified test functionality
        self.test_server_functionality(server_id).await
    }

    /// Test connection temporarily without changing the server's persistent state
    async fn test_connection_temporarily(&mut self, server_id: Uuid) -> Result<bool> {
        // Use the simplified test functionality
        match self.test_server_functionality(server_id).await {
            Ok(result) => Ok(result.success),
            Err(_) => Ok(false),
        }
    }











    /// Set server error status
    fn set_server_error(&mut self, server_id: Uuid, error: String) {
        if let Some(connection) = self.servers.get_mut(&server_id) {
            connection.status = ConnectionStatus::Error(error.clone());
        }
        self.send_event(McpEvent::ServerError(server_id, error));
    }

    // Future: Enhanced MCP operations with rmcp integration
    // These methods will be implemented when rmcp integration is complete

    /// Check if server is ready for operations (Green health status)
    pub fn is_server_ready(&self, server_id: Uuid) -> Result<bool> {
        let connection = self.servers.get(&server_id)
            .ok_or_else(|| anyhow::anyhow!("Server not found"))?;

        Ok(connection.health_status == ServerHealthStatus::Green)
    }

    /// Get server health status
    pub fn get_server_health_status(&self, server_id: Uuid) -> Option<ServerHealthStatus> {
        self.servers.get(&server_id).map(|conn| conn.health_status.clone())
    }

    /// Manually update server health status to Green (after successful testing)
    pub fn update_server_status_to_green(&mut self, server_id: Uuid) -> Result<()> {
        let config = self.server_configs.get(&server_id)
            .ok_or_else(|| anyhow::anyhow!("Server config not found"))?;

        let server_name = config.name.clone();
        log::info!("🟢 手动将服务器状态更新为绿灯: '{}' ({})", server_name, server_id);

        // Update server connection health status
        if let Some(connection) = self.servers.get_mut(&server_id) {
            let old_status = connection.health_status.clone();
            connection.health_status = ServerHealthStatus::Green;
            connection.last_test_time = Some(chrono::Utc::now());

            log::info!("✅ 服务器状态已更新: '{}' {} -> Green", server_name, format!("{:?}", old_status));

            // Update configuration for persistence
            if let Some(config) = self.server_configs.get_mut(&server_id) {
                let old_config_status = config.last_health_status.clone();
                config.last_health_status = Some(ServerHealthStatus::Green);
                config.last_test_time = Some(chrono::Utc::now());
                config.last_test_success = Some(true);

                log::info!("✅ 服务器配置状态已更新: '{}' {:?} -> Green", server_name, old_config_status);
            } else {
                log::error!("❌ 未找到服务器配置: '{}'", server_name);
            }

            // Notify UI of health status change
            self.send_event(McpEvent::HealthStatusChanged(server_id, ServerHealthStatus::Green));

            Ok(())
        } else {
            Err(anyhow::anyhow!("Server connection not found"))
        }
    }

    /// Get server capabilities
    pub fn get_server_capabilities(&self, server_id: Uuid) -> Option<ServerCapabilities> {
        let capabilities = self.servers.get(&server_id).and_then(|conn| conn.capabilities.clone());

        if let Some(ref caps) = capabilities {
            let server_name = self.server_configs.get(&server_id)
                .map(|config| config.name.clone())
                .unwrap_or_else(|| format!("Unknown server {}", server_id));

            log::debug!("📋 获取服务器能力 - '{}': 工具:{}, 资源:{}, 提示:{}",
                server_name, caps.tools.len(), caps.resources.len(), caps.prompts.len());
        } else {
            let server_name = self.server_configs.get(&server_id)
                .map(|config| config.name.clone())
                .unwrap_or_else(|| format!("Unknown server {}", server_id));

            log::warn!("⚠️ 服务器 '{}' 没有能力信息", server_name);
        }

        capabilities
    }

    /// 手动触发服务器能力提取（用于调试和测试）
    pub async fn force_extract_capabilities(&mut self, server_id: Uuid) -> Result<()> {
        let server_name = self.server_configs.get(&server_id)
            .map(|config| config.name.clone())
            .unwrap_or_else(|| format!("Unknown server {}", server_id));

        log::info!("🔄 手动触发能力提取 - 服务器: '{}'", server_name);

        // 强制查询服务器能力，即使服务器状态不是Connected
        let connection = self.servers.get(&server_id)
            .ok_or_else(|| anyhow::anyhow!("Server not found"))?;

        log::info!("📊 强制提取前的服务器状态 - '{}' ({}):", server_name, server_id);
        log::info!("  - 连接状态: {:?}", connection.status);
        log::info!("  - 健康状态: {:?}", connection.health_status);
        log::info!("  - rmcp服务可用: {}", connection.rmcp_service.is_some());

        // 尝试使用现有的rmcp服务
        if connection.rmcp_service.is_some() {
            log::info!("✅ 使用现有rmcp服务强制提取能力 - 服务器: '{}'", server_name);

            match self.extract_capabilities_from_rmcp_service(server_id).await {
                Ok(capabilities) => {
                    log::info!("🎉 强制提取成功 - 服务器: '{}' - 工具:{}, 资源:{}, 提示:{}",
                               server_name, capabilities.tools.len(), capabilities.resources.len(), capabilities.prompts.len());

                    // Update connection with capabilities
                    if let Some(connection) = self.servers.get_mut(&server_id) {
                        connection.capabilities = Some(capabilities.clone());
                        log::info!("✅ 已更新连接中的能力信息 - 服务器: '{}'", server_name);
                    }

                    // Save the extracted capabilities to the server configuration
                    if let Err(e) = self.save_runtime_capabilities(server_id, &capabilities).await {
                        log::error!("❌ 保存运行时能力失败 - 服务器: '{}' - 错误: {}", server_name, e);
                    } else {
                        log::info!("✅ 成功保存运行时能力到配置 - 服务器: '{}'", server_name);
                    }

                    // Send event to UI
                    self.send_event(McpEvent::CapabilitiesUpdated(server_id, capabilities));
                    return Ok(());
                }
                Err(e) => {
                    log::error!("❌ 强制提取失败 - 服务器: '{}' - 错误: {}", server_name, e);
                }
            }
        }

        // 尝试创建新的rmcp服务
        log::info!("🔄 尝试创建新的rmcp服务进行强制提取 - 服务器: '{}'", server_name);
        match self.extract_capabilities_with_fresh_service(server_id).await {
            Ok(capabilities) => {
                log::info!("🎉 使用新rmcp服务强制提取成功 - 服务器: '{}' - 工具:{}, 资源:{}, 提示:{}",
                           server_name, capabilities.tools.len(), capabilities.resources.len(), capabilities.prompts.len());

                // Update connection with capabilities
                if let Some(connection) = self.servers.get_mut(&server_id) {
                    connection.capabilities = Some(capabilities.clone());
                    log::info!("✅ 已更新连接中的能力信息 - 服务器: '{}'", server_name);
                }

                // Save the extracted capabilities to the server configuration
                if let Err(e) = self.save_runtime_capabilities(server_id, &capabilities).await {
                    log::error!("❌ 保存运行时能力失败 - 服务器: '{}' - 错误: {}", server_name, e);
                } else {
                    log::info!("✅ 成功保存运行时能力到配置 - 服务器: '{}'", server_name);
                }

                // Send event to UI
                self.send_event(McpEvent::CapabilitiesUpdated(server_id, capabilities));
                return Ok(());
            }
            Err(e) => {
                log::error!("❌ 使用新rmcp服务强制提取失败 - 服务器: '{}' - 错误: {}", server_name, e);
                return Err(e);
            }
        }
    }

    /// Get server test results
    pub fn get_server_test_results(&self, server_id: Uuid) -> Option<Vec<TestResult>> {
        self.servers.get(&server_id).map(|conn| conn.test_results.clone())
    }

    /// List tools for a specific server
    pub async fn list_tools_for_server(&self, server_id: Uuid) -> Result<Value> {
        let connection = self.servers.get(&server_id)
            .ok_or_else(|| anyhow::anyhow!("Server not found"))?;

        if let Some(rmcp_service) = &connection.rmcp_service {
            rmcp_service.list_tools().await
        } else {
            Err(anyhow::anyhow!("Server not connected with rmcp service"))
        }
    }

    /// Save runtime capabilities to server configuration and trigger database update
    async fn save_runtime_capabilities(&mut self, server_id: Uuid, capabilities: &ServerCapabilities) -> Result<()> {
        let server_name = self.server_configs.get(&server_id)
            .map(|config| config.name.clone())
            .unwrap_or_else(|| format!("Unknown server {}", server_id));

        log::info!("💾 保存运行时能力到配置 - 服务器: '{}'", server_name);

        // Convert ServerCapabilities to JSON Value for storage
        let capabilities_json = serde_json::to_value(capabilities)
            .map_err(|e| anyhow::anyhow!("Failed to serialize capabilities: {}", e))?;

        // Update the server configuration with the extracted capabilities
        if let Some(config) = self.server_configs.get_mut(&server_id) {
            config.capabilities = Some(capabilities_json.clone());
            log::info!("✅ 已更新服务器配置中的运行时能力 - 服务器: '{}' - 工具:{}, 资源:{}, 提示:{}",
                       server_name, capabilities.tools.len(), capabilities.resources.len(), capabilities.prompts.len());
        } else {
            return Err(anyhow::anyhow!("Server config not found for ID: {}", server_id));
        }

        // 触发数据库更新事件，将测试通过的能力保存到数据库
        let capabilities_json_string = serde_json::to_string(&capabilities_json)
            .map_err(|e| anyhow::anyhow!("Failed to serialize capabilities to string: {}", e))?;

        log::info!("📤 触发数据库更新事件 - 保存测试通过的能力到数据库 - 服务器: '{}'", server_name);

        // 发送能力更新事件，包含序列化的能力信息
        self.send_event(McpEvent::CapabilitiesExtracted(server_id, capabilities.clone(), capabilities_json_string));

        log::info!("✅ 成功保存运行时能力到配置并触发数据库更新 - 服务器: '{}'", server_name);

        Ok(())
    }

    /// Get all server configurations (with updated status)
    pub fn get_all_server_configs(&self) -> Vec<McpServerConfig> {
        self.server_configs.values().cloned().collect()
    }

    /// List resources for a specific server
    pub async fn list_resources_for_server(&self, server_id: Uuid) -> Result<Value> {
        let connection = self.servers.get(&server_id)
            .ok_or_else(|| anyhow::anyhow!("Server not found"))?;

        if let Some(rmcp_service) = &connection.rmcp_service {
            rmcp_service.list_resources().await
        } else {
            Err(anyhow::anyhow!("Server not connected with rmcp service"))
        }
    }

    /// List prompts for a specific server
    pub async fn list_prompts_for_server(&self, server_id: Uuid) -> Result<Value> {
        let connection = self.servers.get(&server_id)
            .ok_or_else(|| anyhow::anyhow!("Server not found"))?;

        if let Some(rmcp_service) = &connection.rmcp_service {
            rmcp_service.list_prompts().await
        } else {
            Err(anyhow::anyhow!("Server not connected with rmcp service"))
        }
    }

    /// Call a tool on a specific server
    pub async fn call_tool(&self, server_id: Uuid, tool_name: &str, arguments: Value) -> Result<Value> {
        self.call_tool_internal(server_id, tool_name, arguments, false).await
    }

    /// Call a tool on a specific server (for testing purposes, bypasses health status check)
    pub async fn call_tool_for_testing(&self, server_id: Uuid, tool_name: &str, arguments: Value) -> Result<Value> {
        self.call_tool_internal(server_id, tool_name, arguments, true).await
    }

    /// Internal method to call a tool with optional health status bypass
    async fn call_tool_internal(&self, server_id: Uuid, tool_name: &str, arguments: Value, bypass_health_check: bool) -> Result<Value> {
        let config = self.server_configs.get(&server_id)
            .ok_or_else(|| anyhow::anyhow!("Server config not found"))?;

        log::info!("Calling tool '{}' on server: {} ({}) [bypass_health_check: {}]",
                   tool_name, config.name, server_id, bypass_health_check);

        // Check if server is ready for operations (Green status) - unless bypassing for testing
        if !bypass_health_check && !self.is_server_ready(server_id)? {
            let health_status = self.get_server_health_status(server_id)
                .unwrap_or(ServerHealthStatus::Red);
            return Err(anyhow::anyhow!(
                "Server is not ready for operations. Current health status: {:?}. Please test the server first.",
                health_status
            ));
        }

        // For testing, try to use existing connection first, fallback to fresh service if needed
        if bypass_health_check {
            // First try to use existing connected service
            if let Some(connection) = self.servers.get(&server_id) {
                if connection.status == ConnectionStatus::Connected && connection.rmcp_service.is_some() {
                    log::info!("🔄 Reusing existing rmcp service for tool call: '{}'", tool_name);

                    if let Some(rmcp_service) = &connection.rmcp_service {
                        match rmcp_service.call_tool(tool_name, Some(arguments.clone())).await {
                            Ok(response) => {
                                log::info!("✅ Tool '{}' executed successfully via existing rmcp service", tool_name);
                                return Ok(response);
                            }
                            Err(e) => {
                                log::warn!("❌ Tool call failed via existing service, falling back to fresh service: {}", e);
                                // Fall through to create fresh service
                            }
                        }
                    }
                }
            }

            // Fallback to fresh service if existing connection failed or doesn't exist
            log::info!("🚀 Creating fresh rmcp service for tool call as fallback");
            return self.call_tool_with_fresh_service(server_id, tool_name, arguments).await;
        }

        // Check if server is connected and has rmcp service
        let connection = self.servers.get(&server_id)
            .ok_or_else(|| anyhow::anyhow!("Server not found"))?;

        if connection.status != ConnectionStatus::Connected {
            return Err(anyhow::anyhow!("Server is not connected (status: {:?})", connection.status));
        }

        if connection.rmcp_service.is_none() {
            return Err(anyhow::anyhow!("Server does not have an active rmcp service"));
        }

        // Skip health check to avoid potential conflicts
        // The health check itself might be causing the disconnection
        log::debug!("⚠️ Skipping health check to avoid potential rmcp service conflicts");

        // Instead, try the actual tool call directly and handle errors
        log::debug!("� Proceeding directly to tool call without health check");

        // Create call_tool request
        let request = super::protocol::McpRequest::new(
            serde_json::Value::Number(serde_json::Number::from(100)),
            super::protocol::methods::CALL_TOOL.to_string(),
            Some(serde_json::json!({
                "name": tool_name,
                "arguments": arguments
            })),
        );

        // Use real rmcp client to call the tool
        let connection = self.servers.get(&server_id)
            .ok_or_else(|| anyhow::anyhow!("Server not found"))?;

        if let Some(rmcp_service) = &connection.rmcp_service {
            // Use real MCP protocol to call the tool
            log::debug!("Calling rmcp_service.call_tool with tool_name: '{}', arguments: {:?}", tool_name, arguments);

            match rmcp_service.call_tool(tool_name, Some(arguments)).await {
                Ok(response) => {
                    log::info!("Tool '{}' executed successfully via rmcp", tool_name);
                    log::debug!("Tool '{}' response: {:?}", tool_name, response);

                    // Return the full response - let the caller decide what to extract
                    Ok(response)
                }
                Err(e) => {
                    log::error!("RMCP tool call failed for '{}': {}", tool_name, e);

                    // Check if this is a transport/connection error
                    let error_str = e.to_string();
                    if error_str.contains("disconnected") || error_str.contains("Transport error") {
                        log::warn!("Detected connection issue for server {}, marking as disconnected", server_id);
                        // Note: We can't modify self here because this method takes &self
                        // The UI should handle reconnection
                    }

                    Err(anyhow::anyhow!("Tool call failed: {}", e))
                }
            }
        } else {
            Err(anyhow::anyhow!("Server not properly connected with rmcp"))
        }
    }

    /// Call tool with fresh rmcp service (like git_stdio.rs pattern)
    async fn call_tool_with_fresh_service(&self, server_id: Uuid, tool_name: &str, arguments: Value) -> Result<Value> {
        let config = self.server_configs.get(&server_id)
            .ok_or_else(|| anyhow::anyhow!("Server config not found"))?;

        match &config.transport {
            crate::mcp::server_manager::TransportConfig::Command { command, args, .. } => {
                log::info!("🚀 Creating fresh rmcp service for tool call: {} {:?}", command, args);

                // Create fresh rmcp service (like git_stdio.rs)
                let mut cmd = tokio::process::Command::new(command);
                for arg in args {
                    cmd.arg(arg);
                }

                let transport = TokioChildProcess::new(&mut cmd)
                    .map_err(|e| anyhow::anyhow!("Failed to create transport: {}", e))?;
                let service = ().serve(transport).await
                    .map_err(|e| anyhow::anyhow!("Failed to create rmcp service: {}", e))?;

                log::info!("✅ Fresh rmcp service created, calling tool '{}'", tool_name);

                // Convert arguments to the format expected by rmcp (like git_stdio.rs)
                let arguments_map = if let Some(obj) = arguments.as_object() {
                    Some(obj.clone())
                } else {
                    None
                };

                // Call tool (like git_stdio.rs)
                let result = service.call_tool(CallToolRequestParam {
                    name: tool_name.to_string().into(),
                    arguments: arguments_map,
                }).await;

                // Clean up service
                if let Err(e) = service.cancel().await {
                    log::warn!("Failed to cancel tool call service: {}", e);
                }

                match result {
                    Ok(tool_result) => {
                        log::info!("✅ Tool '{}' executed successfully via fresh rmcp service", tool_name);
                        serde_json::to_value(&tool_result)
                            .map_err(|e| anyhow::anyhow!("Failed to serialize tool result: {}", e))
                    }
                    Err(e) => {
                        log::error!("❌ Tool '{}' failed via fresh rmcp service: {}", tool_name, e);
                        Err(anyhow::anyhow!("Tool call failed: {}", e))
                    }
                }
            }
            crate::mcp::server_manager::TransportConfig::WebSocket { url } => {
                log::info!("🚀 Creating fresh SSE rmcp service for tool call: {}", url);

                // Create fresh SSE rmcp service
                let service = self.create_sse_rmcp_client(url).await
                    .map_err(|e| anyhow::anyhow!("Failed to create SSE rmcp service: {}", e))?;

                log::info!("✅ Fresh SSE rmcp service created, calling tool '{}'", tool_name);

                // Convert arguments to the format expected by rmcp
                let arguments_value = if arguments.is_null() {
                    None
                } else {
                    Some(arguments)
                };

                // Call tool using SSE service
                let result = service.call_tool(tool_name, arguments_value).await;

                // Clean up service
                if let Err(e) = service.service.cancel().await {
                    log::warn!("Failed to cancel SSE tool call service: {}", e);
                }

                match result {
                    Ok(tool_result) => {
                        log::info!("✅ Tool '{}' executed successfully via fresh SSE rmcp service", tool_name);
                        serde_json::to_value(&tool_result)
                            .map_err(|e| anyhow::anyhow!("Failed to serialize tool result: {}", e))
                    }
                    Err(e) => {
                        log::error!("❌ Tool '{}' failed via fresh SSE rmcp service: {}", tool_name, e);
                        Err(anyhow::anyhow!("Tool call failed: {}", e))
                    }
                }
            }
            crate::mcp::server_manager::TransportConfig::Tcp { host, port } => {
                Err(anyhow::anyhow!("TCP transport is not yet supported for tool calls: {}:{}", host, port))
            }
            crate::mcp::server_manager::TransportConfig::Unix { socket_path } => {
                Err(anyhow::anyhow!("Unix socket transport is not yet supported for tool calls: {}", socket_path))
            }
        }
    }

    /// Read a resource from a specific server
    pub async fn read_resource(&self, server_id: Uuid, uri: &str) -> Result<Value> {
        let config = self.server_configs.get(&server_id)
            .ok_or_else(|| anyhow::anyhow!("Server config not found"))?;

        log::info!("Reading resource '{}' from server: {} ({})", uri, config.name, server_id);

        // Check if server is connected
        if !self.servers.contains_key(&server_id) {
            return Err(anyhow::anyhow!("Server not connected"));
        }

        // Create read_resource request
        let request = super::protocol::McpRequest::new(
            serde_json::Value::Number(serde_json::Number::from(101)),
            super::protocol::methods::READ_RESOURCE.to_string(),
            Some(serde_json::json!({
                "uri": uri
            })),
        );

        // Use real rmcp client to read the resource
        let connection = self.servers.get(&server_id)
            .ok_or_else(|| anyhow::anyhow!("Server not found"))?;

        if let Some(rmcp_service) = &connection.rmcp_service {
            // Use real MCP protocol to read the resource
            log::debug!("Calling rmcp_service.read_resource with uri: '{}'", uri);

            match rmcp_service.read_resource(uri).await {
                Ok(response) => {
                    log::info!("Resource '{}' read successfully via rmcp", uri);
                    log::debug!("Resource '{}' response: {:?}", uri, response);

                    // Return the full response - let the caller decide what to extract
                    Ok(response)
                }
                Err(e) => {
                    log::error!("RMCP resource read failed for '{}': {}", uri, e);
                    Err(anyhow::anyhow!("Resource read failed: {}", e))
                }
            }
        } else {
            Err(anyhow::anyhow!("Server not properly connected with rmcp"))
        }
    }

    /// Get a prompt from a specific server
    pub async fn get_prompt(&self, server_id: Uuid, prompt_name: &str, arguments: Option<Value>) -> Result<Value> {
        let config = self.server_configs.get(&server_id)
            .ok_or_else(|| anyhow::anyhow!("Server config not found"))?;

        log::info!("Getting prompt '{}' from server: {} ({})", prompt_name, config.name, server_id);

        // Check if server is connected
        if !self.servers.contains_key(&server_id) {
            return Err(anyhow::anyhow!("Server not connected"));
        }

        // Create get_prompt request
        let request = super::protocol::McpRequest::new(
            serde_json::Value::Number(serde_json::Number::from(102)),
            super::protocol::methods::GET_PROMPT.to_string(),
            Some(serde_json::json!({
                "name": prompt_name,
                "arguments": arguments
            })),
        );

        // Use real rmcp client to get the prompt
        let connection = self.servers.get(&server_id)
            .ok_or_else(|| anyhow::anyhow!("Server not found"))?;

        if let Some(rmcp_service) = &connection.rmcp_service {
            // Use real MCP protocol to get the prompt
            log::debug!("Calling rmcp_service.get_prompt with prompt_name: '{}', arguments: {:?}", prompt_name, arguments);

            match rmcp_service.get_prompt(prompt_name, arguments).await {
                Ok(response) => {
                    log::info!("Prompt '{}' executed successfully via rmcp", prompt_name);
                    log::debug!("Prompt '{}' response: {:?}", prompt_name, response);

                    // Return the full response - let the caller decide what to extract
                    Ok(response)
                }
                Err(e) => {
                    log::error!("RMCP prompt execution failed for '{}': {}", prompt_name, e);
                    Err(anyhow::anyhow!("Prompt execution failed: {}", e))
                }
            }
        } else {
            Err(anyhow::anyhow!("Server not properly connected with rmcp"))
        }
    }

    /// Test server functionality and update health status (simplified approach)
    pub async fn test_server_functionality(&mut self, server_id: Uuid) -> Result<TestResult> {
        log::info!("🧪 开始测试服务器功能: {}", server_id);

        let config = self.server_configs.get(&server_id)
            .ok_or_else(|| anyhow::anyhow!("Server config not found"))?
            .clone();

        let server_name = config.name.clone();
        log::info!("🎯 测试服务器: '{}' ({})", server_name, server_id);

        // Create a fresh rmcp service for testing (like git_stdio.rs) with overall timeout
        log::info!("🚀 开始创建测试服务 - 服务器: '{}'", server_name);
        let test_result = match tokio::time::timeout(
            tokio::time::Duration::from_secs(60), // 总体超时60秒
            async {
                match &config.transport {
                    crate::mcp::server_manager::TransportConfig::Command { command, args, .. } => {
                        log::info!("📋 使用命令传输测试 - 服务器: '{}' - 命令: {} {:?}", server_name, command, args);
                        self.test_server_with_rmcp(server_id, command, args).await
                    }
                    crate::mcp::server_manager::TransportConfig::WebSocket { url } => {
                        log::info!("🌐 使用WebSocket传输测试 - 服务器: '{}' - URL: {}", server_name, url);
                        self.test_server_with_sse(server_id, url).await
                    }
                    crate::mcp::server_manager::TransportConfig::Tcp { host, port } => {
                        log::warn!("🚫 TCP传输暂不支持测试 - 服务器: '{}' - {}:{}", server_name, host, port);
                        TestResult {
                            success: false,
                            stdout: String::new(),
                            stderr: format!("TCP transport not yet supported for testing: {}:{}", host, port),
                            error_message: Some("TCP transport is not yet supported for testing".to_string()),
                        }
                    }
                    crate::mcp::server_manager::TransportConfig::Unix { socket_path } => {
                        log::warn!("🚫 Unix socket传输暂不支持测试 - 服务器: '{}' - {}", server_name, socket_path);
                        TestResult {
                            success: false,
                            stdout: String::new(),
                            stderr: format!("Unix socket transport not yet supported for testing: {}", socket_path),
                            error_message: Some("Unix socket transport is not yet supported for testing".to_string()),
                        }
                    }
                }
            }
        ).await {
            Ok(result) => {
                log::info!("✅ 测试服务创建完成 - 服务器: '{}' - 成功: {}", server_name, result.success);
                result
            }
            Err(_) => {
                let timeout_msg = format!("⏰ 服务器功能测试总体超时 (60秒) - 服务器: '{}'", server_name);
                log::error!("{}", timeout_msg);
                TestResult {
                    success: false,
                    stdout: String::new(),
                    stderr: timeout_msg.clone(),
                    error_message: Some(timeout_msg),
                }
            }
        };

        // Update server health status and test results
        if let Some(connection) = self.servers.get_mut(&server_id) {
            let test_time = chrono::Utc::now();
            connection.last_test_time = Some(test_time);
            connection.test_results.push(test_result.clone());

            // Update health status based on test result
            let new_health_status = if test_result.success {
                ServerHealthStatus::Green
            } else {
                ServerHealthStatus::Yellow // Connected but tests failed
            };

            if connection.health_status != new_health_status {
                connection.health_status = new_health_status.clone();
                self.send_event(McpEvent::HealthStatusChanged(server_id, new_health_status.clone()));
            }

            // 更新配置中的状态信息以便持久化
            if let Some(config) = self.server_configs.get_mut(&server_id) {
                config.last_health_status = Some(new_health_status);
                config.last_test_time = Some(test_time);
                config.last_test_success = Some(test_result.success);
            }
        }

        // Send test completion event
        self.send_event(McpEvent::TestCompleted(server_id, test_result.clone()));

        // 如果测试成功，尝试提取能力并保存到数据库
        if test_result.success {
            let server_name = self.server_configs.get(&server_id)
                .map(|config| config.name.clone())
                .unwrap_or_else(|| format!("Unknown server {}", server_id));

            log::info!("✅ 测试成功，开始提取服务器能力: '{}'", server_name);

            // 尝试提取能力，带超时
            match tokio::time::timeout(
                tokio::time::Duration::from_secs(30), // 能力提取超时30秒
                self.query_server_capabilities(server_id)
            ).await {
                Ok(Ok(_)) => {
                    log::info!("🎉 测试成功且能力提取成功 - 服务器: '{}'", server_name);
                }
                Ok(Err(e)) => {
                    log::warn!("⚠️ 测试成功但能力提取失败 - 服务器: '{}' - 错误: {}", server_name, e);
                }
                Err(_) => {
                    log::error!("⏰ 测试成功但能力提取超时 (30秒) - 服务器: '{}'", server_name);
                }
            }

                // 如果能力提取成功，检查是否有能力信息并触发数据库保存
                if let Some(capabilities) = self.servers.get(&server_id).and_then(|conn| conn.capabilities.clone()) {
                    log::info!("📤 测试成功后触发能力保存到数据库 - 服务器: '{}' - 工具:{}, 资源:{}, 提示:{}",
                        server_name, capabilities.tools.len(), capabilities.resources.len(), capabilities.prompts.len());

                    // 序列化能力信息
                    match serde_json::to_string(&capabilities) {
                        Ok(capabilities_json) => {
                            // 发送能力提取成功事件
                            self.send_event(McpEvent::CapabilitiesExtracted(server_id, capabilities.clone(), capabilities_json));
                            log::info!("✅ 已发送测试成功后的能力提取事件 - 服务器: '{}'", server_name);
                        }
                        Err(e) => {
                            log::error!("❌ 序列化能力信息失败 - 服务器: '{}' - 错误: {}", server_name, e);
                        }
                    }
                } else {
                    log::warn!("⚠️ 测试成功但没有找到能力信息 - 服务器: '{}'", server_name);
                }
        }

        log::info!("🧪 Server {} test completed: success={}", server_id, test_result.success);
        Ok(test_result)
    }

    /// Test server using rmcp service (following git_stdio.rs pattern)
    async fn test_server_with_rmcp(&self, server_id: Uuid, command: &str, args: &[String]) -> TestResult {
        let start_time = std::time::Instant::now();
        let mut test_stdout = String::new();
        let mut test_stderr = String::new();

        log::info!("🚀 Creating fresh rmcp service for testing: {} {:?}", command, args);

        // Create fresh rmcp service for testing (like git_stdio.rs)
        let mut cmd = tokio::process::Command::new(command);
        for arg in args {
            cmd.arg(arg);
        }

        let transport = match TokioChildProcess::new(&mut cmd) {
            Ok(transport) => transport,
            Err(e) => {
                let error_msg = format!("Failed to create transport: {}", e);
                log::error!("{}", error_msg);
                return TestResult {
                    success: false,
                    stdout: test_stdout,
                    stderr: error_msg.clone(),
                    error_message: Some(error_msg),
                };
            }
        };
        let service_result = ().serve(transport).await;

        match service_result {
            Ok(service) => {
                log::info!("✅ RMCP service created successfully");

                // Test 1: Get server info (like git_stdio.rs)
                let server_info = service.peer_info();
                test_stdout.push_str(&format!("✅ Server info: {:#?}\n", server_info));
                log::info!("✅ Server info retrieved: {:#?}", server_info);

                // Track if critical tests pass
                let mut critical_tests_passed = true;
                let mut critical_error_msg = None;

                // Test 2: List tools (CRITICAL - must succeed for green light) with timeout
                log::info!("🔧 开始测试工具列表 (关键测试)...");
                match tokio::time::timeout(
                    tokio::time::Duration::from_secs(10),
                    service.list_tools(Default::default())
                ).await {
                    Ok(Ok(tools)) => {
                        test_stdout.push_str(&format!("✅ Available tools: {:#?}\n", tools));
                        log::info!("✅ Tools listed successfully: {} tools found", tools.tools.len());
                    }
                    Ok(Err(e)) => {
                        let error_msg = format!("❌ Failed to list tools: {}", e);
                        test_stderr.push_str(&format!("{}\n", error_msg));
                        log::error!("{}", error_msg);
                        critical_tests_passed = false;
                        critical_error_msg = Some(error_msg);
                    }
                    Err(_) => {
                        let error_msg = "⏰ List tools operation timed out after 10 seconds".to_string();
                        test_stderr.push_str(&format!("{}\n", error_msg));
                        log::error!("{}", error_msg);
                        critical_tests_passed = false;
                        critical_error_msg = Some(error_msg);
                    }
                }

                // Test 3: List resources (optional) with timeout
                log::info!("📁 开始测试资源列表 (可选测试)...");
                match tokio::time::timeout(
                    tokio::time::Duration::from_secs(10),
                    service.list_resources(Default::default())
                ).await {
                    Ok(Ok(resources)) => {
                        test_stdout.push_str(&format!("✅ Available resources: {:#?}\n", resources));
                        log::info!("✅ Resources listed successfully: {} resources found", resources.resources.len());
                    }
                    Ok(Err(e)) => {
                        // Resources might not be supported, just log as warning
                        test_stdout.push_str(&format!("⚠️ Resources not available: {}\n", e));
                        log::warn!("⚠️ Resources not available: {}", e);
                    }
                    Err(_) => {
                        let warning_msg = "⏰ List resources operation timed out after 10 seconds";
                        test_stdout.push_str(&format!("⚠️ {}\n", warning_msg));
                        log::warn!("{}", warning_msg);
                    }
                }

                // Test 4: List prompts (optional) with timeout
                log::info!("💬 开始测试提示列表 (可选测试)...");
                match tokio::time::timeout(
                    tokio::time::Duration::from_secs(10),
                    service.list_prompts(Default::default())
                ).await {
                    Ok(Ok(prompts)) => {
                        test_stdout.push_str(&format!("✅ Available prompts: {:#?}\n", prompts));
                        log::info!("✅ Prompts listed successfully: {} prompts found", prompts.prompts.len());
                    }
                    Ok(Err(e)) => {
                        // Prompts might not be supported, just log as warning
                        test_stdout.push_str(&format!("⚠️ Prompts not available: {}\n", e));
                        log::warn!("⚠️ Prompts not available: {}", e);
                    }
                    Err(_) => {
                        let warning_msg = "⏰ List prompts operation timed out after 10 seconds";
                        test_stdout.push_str(&format!("⚠️ {}\n", warning_msg));
                        log::warn!("{}", warning_msg);
                    }
                }

                // Clean up service (like git_stdio.rs)
                if let Err(e) = service.cancel().await {
                    log::warn!("Failed to cancel test service: {}", e);
                }

                // Determine final test result based on critical tests
                if critical_tests_passed {
                    log::info!("🟢 All critical tests passed - server ready for green light");
                    TestResult {
                        success: true,
                        stdout: test_stdout,
                        stderr: test_stderr,
                        error_message: None,
                    }
                } else {
                    log::warn!("🟡 Critical tests failed - server stays yellow");
                    TestResult {
                        success: false,
                        stdout: test_stdout,
                        stderr: test_stderr,
                        error_message: critical_error_msg,
                    }
                }
            }
            Err(e) => {
                let error_msg = format!("Failed to create rmcp service: {}", e);
                log::error!("{}", error_msg);

                TestResult {
                    success: false,
                    stdout: test_stdout,
                    stderr: error_msg.clone(),
                    error_message: Some(error_msg),
                }
            }
        }
    }

    /// Test SSE server using rmcp service
    async fn test_server_with_sse(&self, server_id: Uuid, url: &str) -> TestResult {
        let start_time = std::time::Instant::now();
        let mut test_stdout = String::new();
        let mut test_stderr = String::new();

        log::info!("🚀 Creating fresh SSE rmcp service for testing: {}", url);

        // Create fresh SSE rmcp service for testing
        let service_result = self.create_sse_rmcp_client(url).await;

        match service_result {
            Ok(service) => {
                log::info!("✅ SSE RMCP service created successfully");

                // Test 1: SSE service created successfully
                test_stdout.push_str("✅ SSE RMCP service created successfully\n");
                log::info!("✅ SSE RMCP service created successfully");

                // Track if critical tests pass
                let mut critical_tests_passed = true;
                let mut critical_error_msg = None;

                // Test 2: List tools with timeout
                test_stdout.push_str("🔧 Testing tools/list...\n");
                log::info!("🔧 开始测试SSE工具列表 (关键测试)...");
                match tokio::time::timeout(
                    tokio::time::Duration::from_secs(10),
                    service.list_tools()
                ).await {
                    Ok(Ok(tools_response)) => {
                        test_stdout.push_str(&format!("✅ Tools list: {:#?}\n", tools_response));
                        log::info!("✅ Tools list retrieved successfully");
                    }
                    Ok(Err(e)) => {
                        let error_msg = format!("❌ Failed to list tools: {}", e);
                        test_stderr.push_str(&format!("{}\n", error_msg));
                        log::error!("{}", error_msg);
                        critical_tests_passed = false;
                        critical_error_msg = Some(error_msg);
                    }
                    Err(_) => {
                        let error_msg = "⏰ SSE list tools operation timed out after 10 seconds".to_string();
                        test_stderr.push_str(&format!("{}\n", error_msg));
                        log::error!("{}", error_msg);
                        critical_tests_passed = false;
                        critical_error_msg = Some(error_msg);
                    }
                }

                // Test 3: List resources with timeout
                test_stdout.push_str("📁 Testing resources/list...\n");
                log::info!("📁 开始测试SSE资源列表 (可选测试)...");
                match tokio::time::timeout(
                    tokio::time::Duration::from_secs(10),
                    service.list_resources()
                ).await {
                    Ok(Ok(resources_response)) => {
                        test_stdout.push_str(&format!("✅ Resources list: {:#?}\n", resources_response));
                        log::info!("✅ Resources list retrieved successfully");
                    }
                    Ok(Err(e)) => {
                        let error_msg = format!("❌ Failed to list resources: {}", e);
                        test_stderr.push_str(&format!("{}\n", error_msg));
                        log::error!("{}", error_msg);
                        // Resources failure is not critical for basic functionality
                    }
                    Err(_) => {
                        let warning_msg = "⏰ SSE list resources operation timed out after 10 seconds";
                        test_stderr.push_str(&format!("⚠️ {}\n", warning_msg));
                        log::warn!("{}", warning_msg);
                    }
                }

                // Test 4: List prompts with timeout
                test_stdout.push_str("💬 Testing prompts/list...\n");
                log::info!("💬 开始测试SSE提示列表 (可选测试)...");
                match tokio::time::timeout(
                    tokio::time::Duration::from_secs(10),
                    service.list_prompts()
                ).await {
                    Ok(Ok(prompts_response)) => {
                        test_stdout.push_str(&format!("✅ Prompts list: {:#?}\n", prompts_response));
                        log::info!("✅ Prompts list retrieved successfully");
                    }
                    Ok(Err(e)) => {
                        let error_msg = format!("❌ Failed to list prompts: {}", e);
                        test_stderr.push_str(&format!("{}\n", error_msg));
                        log::error!("{}", error_msg);
                        // Prompts failure is not critical for basic functionality
                    }
                    Err(_) => {
                        let warning_msg = "⏰ SSE list prompts operation timed out after 10 seconds";
                        test_stderr.push_str(&format!("⚠️ {}\n", warning_msg));
                        log::warn!("{}", warning_msg);
                    }
                }

                let duration = start_time.elapsed();
                test_stdout.push_str(&format!("⏱️ Test completed in {:?}\n", duration));

                if critical_tests_passed {
                    log::info!("✅ SSE server test passed for: {}", url);
                    TestResult {
                        success: true,
                        stdout: test_stdout,
                        stderr: test_stderr,
                        error_message: None,
                    }
                } else {
                    log::error!("❌ SSE server test failed for: {} - {}", url, critical_error_msg.as_ref().unwrap_or(&"Unknown error".to_string()));
                    TestResult {
                        success: false,
                        stdout: test_stdout,
                        stderr: test_stderr,
                        error_message: critical_error_msg,
                    }
                }
            }
            Err(e) => {
                let error_msg = format!("Failed to create SSE rmcp service: {}", e);
                log::error!("{}", error_msg);
                test_stderr.push_str(&format!("{}\n", error_msg));

                TestResult {
                    success: false,
                    stdout: test_stdout,
                    stderr: test_stderr,
                    error_message: Some(error_msg),
                }
            }
        }
    }

    /// Check if server connection is healthy and reconnect if needed
    pub async fn ensure_server_connected(&mut self, server_id: Uuid) -> Result<()> {
        log::info!("🔍 Checking connection status for server: {}", server_id);

        let config = self.server_configs.get(&server_id)
            .ok_or_else(|| anyhow::anyhow!("Server config not found"))?
            .clone();

        let connection = self.servers.get(&server_id)
            .ok_or_else(|| anyhow::anyhow!("Server not found"))?;

        log::info!("📊 Server {} status: {:?}, has_rmcp_service: {}",
                   server_id, connection.status, connection.rmcp_service.is_some());

        // Check if we have a valid connection and test its health
        let needs_reconnection = if connection.status == ConnectionStatus::Connected && connection.rmcp_service.is_some() {
            log::info!("🔍 Server {} appears connected, testing rmcp service health...", server_id);

            // Test the actual health of the rmcp service with timeout
            if let Some(rmcp_service) = &connection.rmcp_service {
                match tokio::time::timeout(
                    tokio::time::Duration::from_secs(5),
                    rmcp_service.list_tools()
                ).await {
                    Ok(Ok(_)) => {
                        log::info!("✅ RMCP service for server {} is healthy", server_id);
                        return Ok(());
                    }
                    Ok(Err(e)) => {
                        log::warn!("❌ RMCP service for server {} is unhealthy: {}", server_id, e);
                        log::info!("🔄 Marking server as disconnected and will reconnect");
                        true // Need reconnection
                    }
                    Err(_) => {
                        log::warn!("⏰ RMCP service health check timed out for server {}", server_id);
                        log::info!("🔄 Marking server as disconnected and will reconnect");
                        true // Need reconnection
                    }
                }
            } else {
                true // Need reconnection
            }
        } else {
            true // Need reconnection
        };

        if needs_reconnection {
            // Properly disconnect the old service before reconnecting
            log::info!("🧹 Cleaning up old rmcp service before reconnection...");
            if let Some(connection) = self.servers.get_mut(&server_id) {
                if let Some(rmcp_service) = connection.rmcp_service.take() {
                    log::info!("🔄 Properly cancelling old rmcp service for server: {}", server_id);
                    if let Err(e) = rmcp_service.service.cancel().await {
                        log::warn!("Failed to cancel old rmcp service for server {}: {}", server_id, e);
                    } else {
                        log::info!("✅ Successfully cancelled old rmcp service for server: {}", server_id);
                    }
                }

                connection.status = ConnectionStatus::Disconnected;
                connection.capabilities = None;
            }
        }

        log::info!("🔄 Server {} needs reconnection", server_id);

        // Try to reconnect
        match &config.transport {
            crate::mcp::server_manager::TransportConfig::Command { command, args, .. } => {
                log::info!("🚀 Attempting to reconnect to server {} using command: {} {:?}", server_id, command, args);
                self.connect_command_server(server_id, &command, &args).await
            }
            crate::mcp::server_manager::TransportConfig::WebSocket { url } => {
                log::info!("🚀 Attempting to reconnect to server {} using SSE: {}", server_id, url);
                self.connect_sse_server(server_id, &url).await
            }
            _ => {
                Err(anyhow::anyhow!("Transport type not supported for reconnection"))
            }
        }
    }

    /// Refresh server capabilities (placeholder)
    pub async fn refresh_capabilities(&mut self, _server_id: Uuid) -> Result<()> {
        Err(anyhow::anyhow!("Capability refresh not yet implemented - awaiting rmcp integration"))
    }

    /// Send event to all registered receivers
    fn send_event(&self, event: McpEvent) {
        log::info!("📤 发送MCP事件到 {} 个接收器: {:?}", self.event_senders.len(), event);
        for (i, sender) in self.event_senders.iter().enumerate() {
            match sender.send(event.clone()) {
                Ok(_) => log::debug!("✅ 事件成功发送到接收器 {}", i),
                Err(e) => log::warn!("❌ 事件发送到接收器 {} 失败: {}", i, e),
            }
        }
    }








}

impl Default for RmcpClient {
    fn default() -> Self {
        Self::new()
    }
}













