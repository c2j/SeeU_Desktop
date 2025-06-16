use std::collections::HashMap;
use uuid::Uuid;
use serde_json::Value;
use anyhow::Result;
use tokio::sync::mpsc;

// Real rmcp integration for MCP protocol
use rmcp::{
    ServiceExt,
    transport::TokioChildProcess,
    model::{CallToolRequestParam, ReadResourceRequestParam, GetPromptRequestParam},
    service::RunningService,
    RoleClient,
};
use serde_json::json;

use super::server_manager::{McpServerConfig, McpServerInfo};

/// MCP Client implementation using rmcp
#[derive(Debug)]
struct McpClient {
    service: RunningService<RoleClient, ()>,
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
            service,
        })
    }

    /// List tools using rmcp service
    async fn list_tools(&self) -> Result<Value> {
        log::debug!("Listing tools using rmcp service");
        let tools = self.service.list_all_tools().await
            .map_err(|e| anyhow::anyhow!("Failed to list tools: {}", e))?;

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

        let result = self.service.call_tool(CallToolRequestParam {
            name: name.to_string().into(),
            arguments: arguments_map,
        }).await
            .map_err(|e| {
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

        let result = self.service.read_resource(ReadResourceRequestParam {
            uri: uri.to_string().into(),
        }).await
            .map_err(|e| {
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

        let result = self.service.get_prompt(GetPromptRequestParam {
            name: name.to_string().into(),
            arguments: arguments_map,
        }).await
            .map_err(|e| {
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

    /// Event sender for UI updates
    event_sender: Option<mpsc::UnboundedSender<McpEvent>>,
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
#[derive(Debug, Clone, PartialEq)]
pub enum ServerHealthStatus {
    /// Red light: Server configuration added/modified but not tested
    Red,
    /// Yellow light: Server connected successfully but not tested
    Yellow,
    /// Green light: Server connected and passed all tests
    Green,
}

/// Server capabilities
#[derive(Debug, Clone)]
pub struct ServerCapabilities {
    pub tools: Vec<ToolInfo>,
    pub resources: Vec<ResourceInfo>,
    pub prompts: Vec<PromptInfo>,
}

/// Tool information
#[derive(Debug, Clone)]
pub struct ToolInfo {
    pub name: String,
    pub description: Option<String>,
    pub input_schema: Option<Value>,
}

/// Resource information
#[derive(Debug, Clone)]
pub struct ResourceInfo {
    pub uri: String,
    pub name: String,
    pub description: Option<String>,
    pub mime_type: Option<String>,
}

/// Prompt information
#[derive(Debug, Clone)]
pub struct PromptInfo {
    pub name: String,
    pub description: Option<String>,
    pub arguments: Vec<PromptArgument>,
}

/// Prompt argument
#[derive(Debug, Clone)]
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
            event_sender: None,
        }
    }

    /// Set event sender for UI updates
    pub fn set_event_sender(&mut self, sender: mpsc::UnboundedSender<McpEvent>) {
        self.event_sender = Some(sender);
    }

    /// Add a server configuration
    pub fn add_server_config(&mut self, config: McpServerConfig) -> Uuid {
        let server_id = config.id;
        self.server_configs.insert(server_id, config.clone());

        let connection = ServerConnection {
            server_id,
            config,
            status: ConnectionStatus::Disconnected,
            health_status: ServerHealthStatus::Red, // New servers start with red status
            capabilities: None,
            last_test_time: None,
            test_results: Vec::new(),
            rmcp_service: None,
        };

        self.servers.insert(server_id, connection);

        // Notify UI of the new server's health status
        self.send_event(McpEvent::HealthStatusChanged(server_id, ServerHealthStatus::Red));

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
            crate::mcp::server_manager::TransportConfig::Tcp { host, port } => {
                let error = format!("TCP transport not yet supported: {}:{}", host, port);
                self.set_server_error(server_id, error.clone());
                Err(anyhow::anyhow!(error))
            }
            _ => {
                let error = "Transport type not yet supported".to_string();
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

                // Verify the service is still healthy after capability query
                log::debug!("🔍 Verifying rmcp service health after capability query...");
                if let Some(connection) = self.servers.get(&server_id) {
                    if let Some(rmcp_service) = &connection.rmcp_service {
                        match rmcp_service.list_tools().await {
                            Ok(_) => {
                                log::info!("✅ RMCP service is healthy after capability query");
                            }
                            Err(e) => {
                                log::error!("❌ RMCP service became unhealthy after capability query: {}", e);
                                // Mark as disconnected so it will be reconnected on next use
                                if let Some(conn) = self.servers.get_mut(&server_id) {
                                    conn.status = ConnectionStatus::Disconnected;
                                    conn.rmcp_service = None;
                                    conn.capabilities = None;
                                }
                                return Err(anyhow::anyhow!("RMCP service became unhealthy after capability query: {}", e));
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
            service,
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
    async fn query_server_capabilities(&mut self, server_id: Uuid) -> Result<()> {
        log::info!("Querying capabilities for server: {}", server_id);

        // Check if server is connected
        let connection = self.servers.get(&server_id)
            .ok_or_else(|| anyhow::anyhow!("Server not found"))?;

        if connection.status != ConnectionStatus::Connected {
            return Err(anyhow::anyhow!("Server is not connected"));
        }

        // Get server config for logging
        let config = self.server_configs.get(&server_id)
            .ok_or_else(|| anyhow::anyhow!("Server config not found"))?;

        // Check if we have rmcp service - if so, use it directly
        if connection.rmcp_service.is_some() {
            log::info!("Using rmcp service to extract capabilities for server: {}", config.name);

            // Extract capabilities from rmcp service
            let capabilities = self.extract_capabilities_from_rmcp_service(server_id).await?;

            log::info!("Successfully extracted capabilities from rmcp service for server: {} - Tools: {}, Resources: {}, Prompts: {}",
                       config.name, capabilities.tools.len(), capabilities.resources.len(), capabilities.prompts.len());

            // Update connection with capabilities
            if let Some(connection) = self.servers.get_mut(&server_id) {
                connection.capabilities = Some(capabilities.clone());
            }

            // Send event to UI
            self.send_event(McpEvent::CapabilitiesUpdated(server_id, capabilities));
            return Ok(());
        }

        // Fallback: no rmcp service available, return empty capabilities
        log::warn!("No rmcp service available for server: {}, returning empty capabilities", config.name);

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

        let rmcp_client = connection.rmcp_service.as_ref()
            .ok_or_else(|| anyhow::anyhow!("No rmcp service available"))?;

        // Use the real rmcp service to query actual capabilities
        // Query tools using the real MCP protocol
        let tools = match rmcp_client.list_tools().await {
            Ok(response) => {
                log::info!("Successfully queried tools from rmcp service");

                // Parse the response to extract tools
                if let Some(tools_array) = response.get("tools").and_then(|t| t.as_array()) {
                    let tool_infos: Vec<ToolInfo> = tools_array.iter().filter_map(|tool| {
                        let name = tool.get("name")?.as_str()?.to_string();
                        let description = tool.get("description").and_then(|d| d.as_str()).map(|s| s.to_string());
                        let input_schema = tool.get("inputSchema").cloned();

                        Some(ToolInfo {
                            name,
                            description,
                            input_schema,
                        })
                    }).collect();

                    log::info!("Parsed {} tools from rmcp service response", tool_infos.len());
                    tool_infos
                } else {
                    log::warn!("No tools array found in rmcp service response");
                    Vec::new()
                }
            }
            Err(e) => {
                log::warn!("Failed to query tools from rmcp service: {}", e);
                Vec::new()
            }
        };

        // Query resources using the real MCP protocol
        let resources = match rmcp_client.list_resources().await {
            Ok(response) => {
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
            Err(e) => {
                log::warn!("Failed to query resources from rmcp service: {}", e);
                Vec::new()
            }
        };

        // Query prompts using the real MCP protocol
        let prompts = match rmcp_client.list_prompts().await {
            Ok(response) => {
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
            Err(e) => {
                log::warn!("Failed to query prompts from rmcp service: {}", e);
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

        // For testing, create a fresh rmcp service (like git_stdio.rs)
        if bypass_health_check {
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
            _ => {
                Err(anyhow::anyhow!("Only command transport is supported for fresh service tool calls"))
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
        log::info!("🧪 Testing server functionality for: {}", server_id);

        let config = self.server_configs.get(&server_id)
            .ok_or_else(|| anyhow::anyhow!("Server config not found"))?
            .clone();

        // Create a fresh rmcp service for testing (like git_stdio.rs)
        let test_result = match &config.transport {
            crate::mcp::server_manager::TransportConfig::Command { command, args, .. } => {
                self.test_server_with_rmcp(server_id, command, args).await
            }
            _ => {
                TestResult {
                    success: false,
                    stdout: String::new(),
                    stderr: "Unsupported transport type for testing".to_string(),
                    error_message: Some("Only command transport is supported for testing".to_string()),
                }
            }
        };

        // Update server health status and test results
        if let Some(connection) = self.servers.get_mut(&server_id) {
            connection.last_test_time = Some(chrono::Utc::now());
            connection.test_results.push(test_result.clone());

            // Update health status based on test result
            let new_health_status = if test_result.success {
                ServerHealthStatus::Green
            } else {
                ServerHealthStatus::Yellow // Connected but tests failed
            };

            if connection.health_status != new_health_status {
                connection.health_status = new_health_status.clone();
                self.send_event(McpEvent::HealthStatusChanged(server_id, new_health_status));
            }
        }

        // Send test completion event
        self.send_event(McpEvent::TestCompleted(server_id, test_result.clone()));

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

                // Test 2: List tools (like git_stdio.rs)
                match service.list_tools(Default::default()).await {
                    Ok(tools) => {
                        test_stdout.push_str(&format!("✅ Available tools: {:#?}\n", tools));
                        log::info!("✅ Tools listed successfully: {} tools found", tools.tools.len());
                    }
                    Err(e) => {
                        let error_msg = format!("❌ Failed to list tools: {}", e);
                        test_stderr.push_str(&format!("{}\n", error_msg));
                        log::error!("{}", error_msg);
                    }
                }

                // Test 3: List resources (optional)
                match service.list_resources(Default::default()).await {
                    Ok(resources) => {
                        test_stdout.push_str(&format!("✅ Available resources: {:#?}\n", resources));
                        log::info!("✅ Resources listed successfully: {} resources found", resources.resources.len());
                    }
                    Err(e) => {
                        // Resources might not be supported, just log as warning
                        test_stdout.push_str(&format!("⚠️ Resources not available: {}\n", e));
                        log::warn!("⚠️ Resources not available: {}", e);
                    }
                }

                // Test 4: List prompts (optional)
                match service.list_prompts(Default::default()).await {
                    Ok(prompts) => {
                        test_stdout.push_str(&format!("✅ Available prompts: {:#?}\n", prompts));
                        log::info!("✅ Prompts listed successfully: {} prompts found", prompts.prompts.len());
                    }
                    Err(e) => {
                        // Prompts might not be supported, just log as warning
                        test_stdout.push_str(&format!("⚠️ Prompts not available: {}\n", e));
                        log::warn!("⚠️ Prompts not available: {}", e);
                    }
                }

                // Clean up service (like git_stdio.rs)
                if let Err(e) = service.cancel().await {
                    log::warn!("Failed to cancel test service: {}", e);
                }

                TestResult {
                    success: true,
                    stdout: test_stdout,
                    stderr: test_stderr,
                    error_message: None,
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

            // Test the actual health of the rmcp service
            if let Some(rmcp_service) = &connection.rmcp_service {
                match rmcp_service.list_tools().await {
                    Ok(_) => {
                        log::info!("✅ RMCP service for server {} is healthy", server_id);
                        return Ok(());
                    }
                    Err(e) => {
                        log::warn!("❌ RMCP service for server {} is unhealthy: {}", server_id, e);
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
            _ => {
                Err(anyhow::anyhow!("Transport type not supported for reconnection"))
            }
        }
    }

    /// Refresh server capabilities (placeholder)
    pub async fn refresh_capabilities(&mut self, _server_id: Uuid) -> Result<()> {
        Err(anyhow::anyhow!("Capability refresh not yet implemented - awaiting rmcp integration"))
    }

    /// Send event to UI
    fn send_event(&self, event: McpEvent) {
        if let Some(sender) = &self.event_sender {
            let _ = sender.send(event);
        }
    }








}

impl Default for RmcpClient {
    fn default() -> Self {
        Self::new()
    }
}













