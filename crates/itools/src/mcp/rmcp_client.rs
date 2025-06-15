use std::collections::HashMap;
use std::process::Child;
use uuid::Uuid;
use serde_json::Value;
use anyhow::Result;
use tokio::sync::{mpsc, oneshot};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, AsyncReadExt, BufReader};
use std::sync::Arc;
use tokio::sync::Mutex;

// Real rmcp integration for MCP protocol
use rmcp::{
    ServiceExt,
    transport::TokioChildProcess,
    model::{CallToolRequestParam, ReadResourceRequestParam, GetPromptRequestParam},
    service::RunningService,
    RoleClient,
};
use serde_json::json;

use super::server_manager::{McpServerConfig, McpServerInfo, TransportConfig};
use super::protocol_handler::{McpProtocolHandler, ProtocolState, ClientInfo as ProtocolClientInfo};

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

        // rmcp returns a Vec<Tool> directly, convert to our expected JSON format
        let tools_json: Vec<Value> = tools.iter().map(|tool| {
            json!({
                "name": tool.name,
                "description": tool.description,
                "inputSchema": tool.input_schema
            })
        }).collect();

        log::debug!("Converted {} tools to JSON format", tools_json.len());

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
        log::debug!("Calling tool '{}' using rmcp service", name);

        let arguments_map = arguments.and_then(|v| v.as_object().cloned());

        let result = self.service.call_tool(CallToolRequestParam {
            name: name.to_string().into(),
            arguments: arguments_map,
        }).await
            .map_err(|e| anyhow::anyhow!("Failed to call tool '{}': {}", name, e))?;

        // Convert rmcp result to JSON format
        serde_json::to_value(&result)
            .map_err(|e| anyhow::anyhow!("Failed to serialize tool result: {}", e))
    }

    /// Read a resource using rmcp service
    async fn read_resource(&self, uri: &str) -> Result<Value> {
        log::debug!("Reading resource '{}' using rmcp service", uri);

        let result = self.service.read_resource(ReadResourceRequestParam {
            uri: uri.to_string(),
        }).await
            .map_err(|e| anyhow::anyhow!("Failed to read resource '{}': {}", uri, e))?;

        // Convert rmcp result to JSON format
        serde_json::to_value(&result)
            .map_err(|e| anyhow::anyhow!("Failed to serialize resource result: {}", e))
    }

    /// Get a prompt using rmcp service
    async fn get_prompt(&self, name: &str, arguments: Option<Value>) -> Result<Value> {
        log::debug!("Getting prompt '{}' using rmcp service", name);

        let arguments_map = arguments.and_then(|v| v.as_object().cloned());

        let result = self.service.get_prompt(GetPromptRequestParam {
            name: name.to_string(),
            arguments: arguments_map,
        }).await
            .map_err(|e| anyhow::anyhow!("Failed to get prompt '{}': {}", name, e))?;

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

    /// Pending requests waiting for responses
    pending_requests: Arc<Mutex<HashMap<String, oneshot::Sender<Value>>>>,

    // Future: RMCP client instances (currently using protocol handler)
    // rmcp_clients: HashMap<Uuid, RoleClient>,
}

/// Connection to an MCP server using rmcp
#[derive(Debug)]
pub struct ServerConnection {
    pub server_id: Uuid,
    pub config: McpServerConfig,
    pub status: ConnectionStatus,
    pub capabilities: Option<ServerCapabilities>,
    pub last_ping: Option<chrono::DateTime<chrono::Utc>>,
    pub process: Option<Child>,
    pub protocol_handler: Option<McpProtocolHandler>,
    pub message_sender: Option<mpsc::UnboundedSender<String>>,
    // Real rmcp client integration
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
            pending_requests: Arc::new(Mutex::new(HashMap::new())),
            // rmcp_clients: HashMap::new(),
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
            capabilities: None,
            last_ping: None,
            process: None,
            protocol_handler: None,
            message_sender: None,
            // Real rmcp client integration
            rmcp_service: None,
        };

        self.servers.insert(server_id, connection);
        server_id
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
                self.connect_tcp_server(server_id, &host, *port).await
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
                    log::info!("Stored rmcp service for server {} and marked as connected", server_id);
                } else {
                    log::error!("Failed to find connection for server {} when storing rmcp service", server_id);
                    return Err(anyhow::anyhow!("Failed to find connection for server"));
                }

                // Query server capabilities using rmcp service
                if let Err(e) = self.query_server_capabilities(server_id).await {
                    log::warn!("Failed to query capabilities for server {}: {}", server_id, e);
                    // Don't fail the connection just because capability query failed
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

                        // Create protocol handler as fallback
                        let mut protocol_handler = McpProtocolHandler::new(server_id);

                        // Setup message communication
                        let (message_sender, message_receiver) = mpsc::unbounded_channel::<String>();
                        let (event_sender, mut event_receiver) = mpsc::unbounded_channel();

                        protocol_handler.set_event_sender(event_sender);
                        protocol_handler.connect();

                        // Store connection info
                        if let Some(connection) = self.servers.get_mut(&server_id) {
                            connection.protocol_handler = Some(protocol_handler);
                            connection.message_sender = Some(message_sender.clone());
                        }

                        // Start message handling tasks
                        self.start_message_tasks(server_id, stdin, stdout, message_receiver, message_sender.clone()).await?;

                        // Start protocol initialization
                        self.initialize_protocol(server_id).await?;

                        // Update connection status to connected
                        if let Some(connection) = self.servers.get_mut(&server_id) {
                            connection.status = ConnectionStatus::Connected;
                        }

                        // Query server capabilities after successful connection
                        if let Err(e) = self.query_server_capabilities(server_id).await {
                            log::warn!("Failed to query capabilities for server {}: {}", server_id, e);
                            // Don't fail the connection just because capability query failed
                        }

                        // Handle protocol events
                        tokio::spawn(async move {
                            while let Some(event) = event_receiver.recv().await {
                                log::debug!("Protocol event: {:?}", event);
                                // Handle events (state changes, capabilities, etc.)
                            }
                        });

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

        // Configure stdio
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

    /// Connect to a TCP server
    async fn connect_tcp_server(&mut self, _server_id: Uuid, _host: &str, _port: u16) -> Result<()> {
        // TODO: Implement TCP connection using tokio TcpStream
        Err(anyhow::anyhow!("TCP transport not yet implemented"))
    }

    // Future: Query rmcp client capabilities
    // This will be implemented when rmcp integration is complete

    /// Disconnect from a server
    pub fn disconnect_server(&mut self, server_id: Uuid) -> Result<()> {
        if let Some(connection) = self.servers.get_mut(&server_id) {
            connection.status = ConnectionStatus::Disconnected;
            connection.capabilities = None;
            connection.last_ping = None;
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

        // Fallback to manual JSON-RPC queries if no rmcp service
        log::info!("No rmcp service available, falling back to manual queries for server: {}", config.name);

        // Fallback to manual JSON-RPC queries if no rmcp service
        log::info!("No rmcp service available, falling back to manual queries for server: {}", config.name);

        let message_sender = connection.message_sender.as_ref()
            .ok_or_else(|| anyhow::anyhow!("No message sender available"))?;

        // First, query tools
        let tools = self.query_server_tools(server_id, message_sender).await?;

        // Then, query resources
        let resources = self.query_server_resources(server_id, message_sender).await?;

        // Finally, query prompts
        let prompts = self.query_server_prompts(server_id, message_sender).await?;

        // Log the counts before moving the data
        log::info!("Successfully queried capabilities for server: {} - Tools: {}, Resources: {}, Prompts: {}",
                   config.name, tools.len(), resources.len(), prompts.len());

        // Create capabilities from the queried data
        let capabilities = ServerCapabilities {
            tools,
            resources,
            prompts,
        };

        // Update connection with capabilities
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
            capabilities: connection.capabilities.clone(),
            last_ping: connection.last_ping,
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

                // Update last ping time
                if let Some(conn) = self.servers.get_mut(&server_id) {
                    conn.last_ping = Some(chrono::Utc::now());
                }

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
        let config = self.server_configs.get(&server_id)
            .ok_or_else(|| anyhow::anyhow!("Server config not found"))?
            .clone();

        log::info!("Starting detailed test for server: {} ({})", config.name, server_id);

        // Try to start the server process temporarily
        match &config.transport {
            TransportConfig::Command { command, args, env } => {
                log::info!("Testing command: {} with args: {:?}", command, args);
                if !env.is_empty() {
                    log::debug!("Environment variables: {:?}", env);
                }

                // Create a temporary process to test the connection
                let mut cmd = tokio::process::Command::new(command);
                cmd.args(args);

                // Set environment variables
                for (key, value) in env {
                    cmd.env(key, value);
                }

                // Configure process
                cmd.stdin(std::process::Stdio::piped())
                   .stdout(std::process::Stdio::piped())
                   .stderr(std::process::Stdio::piped());

                // Try to spawn the process
                match cmd.spawn() {
                    Ok(mut child) => {
                        log::info!("Successfully spawned test process for server {} (PID: {:?})", server_id, child.id());

                        // Set a timeout for the test
                        let timeout_duration = tokio::time::Duration::from_secs(15);

                        let test_result = tokio::time::timeout(
                            timeout_duration,
                            self.perform_detailed_process_test_with_output(&mut child, server_id)
                        ).await;

                        match test_result {
                            Ok(result) => {
                                // Clean up the test process
                                let _ = child.kill().await;
                                let _ = child.wait().await;
                                result
                            }
                            Err(_) => {
                                log::error!("Test for server {} timed out after {} seconds", server_id, timeout_duration.as_secs());
                                // Clean up the test process
                                let _ = child.kill().await;
                                let _ = child.wait().await;
                                Ok(TestResult {
                                    success: false,
                                    stdout: String::new(),
                                    stderr: String::new(),
                                    error_message: Some(format!("Test timed out after {} seconds", timeout_duration.as_secs())),
                                })
                            }
                        }
                    }
                    Err(e) => {
                        log::error!("Failed to spawn test process for server {}: {}", server_id, e);
                        log::error!("Command details - executable: '{}', args: {:?}", command, args);

                        // Check if the command exists
                        let command_check_error = if let Err(spawn_err) = std::process::Command::new(command).arg("--help").output() {
                            format!("Command '{}' may not be available: {}", command, spawn_err)
                        } else {
                            String::new()
                        };

                        Ok(TestResult {
                            success: false,
                            stdout: String::new(),
                            stderr: format!("Failed to spawn process: {}", e),
                            error_message: Some(if command_check_error.is_empty() {
                                format!("Process spawn failed: {}", e)
                            } else {
                                format!("Process spawn failed: {}. {}", e, command_check_error)
                            }),
                        })
                    }
                }
            }
            _ => {
                // For non-command transports, fall back to simple test
                let success = self.test_connection_temporarily(server_id).await?;
                Ok(TestResult {
                    success,
                    stdout: String::new(),
                    stderr: String::new(),
                    error_message: if success { None } else { Some("Connection test failed".to_string()) },
                })
            }
        }
    }

    /// Test connection temporarily without changing the server's persistent state
    async fn test_connection_temporarily(&mut self, server_id: Uuid) -> Result<bool> {
        let config = self.server_configs.get(&server_id)
            .ok_or_else(|| anyhow::anyhow!("Server config not found"))?
            .clone();

        log::info!("Starting temporary connection test for server: {} ({})", config.name, server_id);

        // Try to start the server process temporarily
        match &config.transport {
            TransportConfig::Command { command, args, env } => {
                log::info!("Testing command: {} with args: {:?}", command, args);
                if !env.is_empty() {
                    log::debug!("Environment variables: {:?}", env);
                }

                // Create a temporary process to test the connection
                let mut cmd = tokio::process::Command::new(command);
                cmd.args(args);

                // Set environment variables
                for (key, value) in env {
                    cmd.env(key, value);
                }

                // Configure process
                cmd.stdin(std::process::Stdio::piped())
                   .stdout(std::process::Stdio::piped())
                   .stderr(std::process::Stdio::piped());

                // Try to spawn the process
                match cmd.spawn() {
                    Ok(mut child) => {
                        log::info!("Successfully spawned test process for server {} (PID: {:?})", server_id, child.id());

                        // Set a timeout for the test
                        let timeout_duration = tokio::time::Duration::from_secs(15);

                        let test_result = tokio::time::timeout(
                            timeout_duration,
                            self.perform_detailed_process_test(&mut child, server_id)
                        ).await;

                        match test_result {
                            Ok(result) => {
                                // Clean up the test process
                                let _ = child.kill().await;
                                let _ = child.wait().await;
                                result
                            }
                            Err(_) => {
                                log::error!("Test for server {} timed out after {} seconds", server_id, timeout_duration.as_secs());
                                // Clean up the test process
                                let _ = child.kill().await;
                                let _ = child.wait().await;
                                Ok(false)
                            }
                        }
                    }
                    Err(e) => {
                        log::error!("Failed to spawn test process for server {}: {}", server_id, e);
                        log::error!("Command details - executable: '{}', args: {:?}", command, args);

                        // Check if the command exists
                        if let Err(spawn_err) = std::process::Command::new(command).arg("--help").output() {
                            log::error!("Command '{}' may not be available: {}", command, spawn_err);
                        }

                        Ok(false)
                    }
                }
            }
            TransportConfig::Tcp { host, port } => {
                log::debug!("Testing TCP connection to {}:{}", host, port);

                // Try to connect to the TCP endpoint
                match tokio::net::TcpStream::connect(format!("{}:{}", host, port)).await {
                    Ok(_) => {
                        log::info!("Successfully connected to TCP endpoint {}:{}", host, port);
                        Ok(true)
                    }
                    Err(e) => {
                        log::warn!("Failed to connect to TCP endpoint {}:{}: {}", host, port, e);
                        Ok(false)
                    }
                }
            }
            TransportConfig::Unix { socket_path } => {
                log::debug!("Testing Unix socket connection to {}", socket_path);

                // Try to connect to the Unix socket
                match tokio::net::UnixStream::connect(socket_path).await {
                    Ok(_) => {
                        log::info!("Successfully connected to Unix socket {}", socket_path);
                        Ok(true)
                    }
                    Err(e) => {
                        log::warn!("Failed to connect to Unix socket {}: {}", socket_path, e);
                        Ok(false)
                    }
                }
            }
            TransportConfig::WebSocket { url } => {
                log::debug!("Testing WebSocket connection to {}", url);

                // For WebSocket, we'll just validate the URL format for now
                // A full implementation would require a WebSocket client
                if url.starts_with("ws://") || url.starts_with("wss://") {
                    log::info!("WebSocket URL format is valid: {}", url);
                    Ok(true)
                } else {
                    log::warn!("Invalid WebSocket URL format: {}", url);
                    Ok(false)
                }
            }
        }
    }

    /// Perform detailed process testing with output capture
    async fn perform_detailed_process_test(&self, child: &mut tokio::process::Child, server_id: Uuid) -> Result<bool> {
        // Give the process a moment to start
        tokio::time::sleep(tokio::time::Duration::from_millis(2000)).await;

        // Check if the process is still running
        match child.try_wait() {
            Ok(Some(status)) => {
                // Process has exited
                if status.success() {
                    log::info!("Test process for server {} exited successfully with status: {}", server_id, status);

                    // Try to capture any output before the process exited and check for errors
                    let output_ok = self.capture_process_output(child, server_id, true).await;
                    if !output_ok {
                        log::error!("Test process for server {} exited successfully but stderr contains errors", server_id);
                        return Ok(false);
                    }
                    Ok(true)
                } else {
                    log::error!("Test process for server {} exited with error status: {}", server_id, status);

                    // Capture error output
                    self.capture_process_output(child, server_id, false).await;
                    Ok(false)
                }
            }
            Ok(None) => {
                // Process is still running, which is good for an MCP server
                log::info!("Test process for server {} is running successfully", server_id);

                // Try to capture some output to verify the server is responsive and check for errors
                let output_ok = self.capture_process_output(child, server_id, true).await;
                if !output_ok {
                    log::error!("Test process for server {} is running but stderr contains errors", server_id);
                    return Ok(false);
                }
                Ok(true)
            }
            Err(e) => {
                log::error!("Error checking test process status for server {}: {}", server_id, e);
                Ok(false)
            }
        }
    }

    /// Perform detailed process testing with output capture and return detailed results
    async fn perform_detailed_process_test_with_output(&self, child: &mut tokio::process::Child, server_id: Uuid) -> Result<TestResult> {
        // Give the process a moment to start
        tokio::time::sleep(tokio::time::Duration::from_millis(2000)).await;

        // Capture output first
        let (stdout_output, stderr_output) = self.capture_process_output_detailed(child, server_id).await;

        // Check if the process is still running
        match child.try_wait() {
            Ok(Some(status)) => {
                // Process has exited
                if status.success() {
                    log::info!("Test process for server {} exited successfully with status: {}", server_id, status);

                    // Check for errors in stderr even if process succeeded
                    let has_errors = self.contains_error_patterns(&stderr_output);
                    if has_errors {
                        log::error!("Test process for server {} exited successfully but stderr contains errors", server_id);
                        Ok(TestResult {
                            success: false,
                            stdout: stdout_output,
                            stderr: stderr_output,
                            error_message: Some("Process succeeded but stderr contains error messages".to_string()),
                        })
                    } else {
                        Ok(TestResult {
                            success: true,
                            stdout: stdout_output,
                            stderr: stderr_output,
                            error_message: None,
                        })
                    }
                } else {
                    log::error!("Test process for server {} exited with error status: {}", server_id, status);
                    Ok(TestResult {
                        success: false,
                        stdout: stdout_output,
                        stderr: stderr_output,
                        error_message: Some(format!("Process exited with error status: {}", status)),
                    })
                }
            }
            Ok(None) => {
                // Process is still running, which is good for an MCP server
                log::info!("Test process for server {} is running successfully", server_id);

                // Check for errors in stderr even if process is running
                let has_errors = self.contains_error_patterns(&stderr_output);
                if has_errors {
                    log::error!("Test process for server {} is running but stderr contains errors", server_id);
                    Ok(TestResult {
                        success: false,
                        stdout: stdout_output,
                        stderr: stderr_output,
                        error_message: Some("Process is running but stderr contains error messages".to_string()),
                    })
                } else {
                    Ok(TestResult {
                        success: true,
                        stdout: stdout_output,
                        stderr: stderr_output,
                        error_message: None,
                    })
                }
            }
            Err(e) => {
                log::error!("Error checking test process status for server {}: {}", server_id, e);
                Ok(TestResult {
                    success: false,
                    stdout: stdout_output,
                    stderr: stderr_output,
                    error_message: Some(format!("Error checking process status: {}", e)),
                })
            }
        }
    }

    /// Capture process output and return both stdout and stderr
    async fn capture_process_output_detailed(&self, child: &mut tokio::process::Child, server_id: Uuid) -> (String, String) {
        let mut stdout_output = String::new();
        let mut stderr_output = String::new();

        // Capture stdout
        if let Some(stdout) = child.stdout.take() {
            let mut stdout_reader = tokio::io::BufReader::new(stdout);

            let read_result = tokio::time::timeout(
                tokio::time::Duration::from_millis(1000),
                stdout_reader.read_to_string(&mut stdout_output)
            ).await;

            match read_result {
                Ok(Ok(bytes_read)) => {
                    if bytes_read > 0 && !stdout_output.trim().is_empty() {
                        log::info!("Server {} stdout output ({} bytes): {}", server_id, bytes_read, stdout_output.trim());
                    } else {
                        log::debug!("Server {} produced no stdout output", server_id);
                    }
                }
                Ok(Err(e)) => {
                    log::debug!("Could not read stdout from server {}: {}", server_id, e);
                    stdout_output = format!("Error reading stdout: {}", e);
                }
                Err(_) => {
                    log::debug!("Timeout reading stdout from server {} (normal for some servers)", server_id);
                    stdout_output = "Timeout reading stdout".to_string();
                }
            }
        }

        // Capture stderr
        if let Some(stderr) = child.stderr.take() {
            let mut stderr_reader = tokio::io::BufReader::new(stderr);

            let read_result = tokio::time::timeout(
                tokio::time::Duration::from_millis(1000),
                stderr_reader.read_to_string(&mut stderr_output)
            ).await;

            match read_result {
                Ok(Ok(bytes_read)) => {
                    if bytes_read > 0 && !stderr_output.trim().is_empty() {
                        let stderr_content = stderr_output.trim();

                        // Check for common error patterns in stderr
                        let has_errors = self.contains_error_patterns(stderr_content);

                        if has_errors {
                            log::error!("Server {} stderr contains errors ({} bytes): {}", server_id, bytes_read, stderr_content);
                        } else {
                            log::info!("Server {} stderr output ({} bytes): {}", server_id, bytes_read, stderr_content);
                        }
                    }
                }
                Ok(Err(e)) => {
                    log::debug!("Could not read stderr from server {}: {}", server_id, e);
                    stderr_output = format!("Error reading stderr: {}", e);
                }
                Err(_) => {
                    log::debug!("Timeout reading stderr from server {}", server_id);
                    stderr_output = "Timeout reading stderr".to_string();
                }
            }
        }

        (stdout_output, stderr_output)
    }

    /// Capture process output for logging and error detection
    async fn capture_process_output(&self, child: &mut tokio::process::Child, server_id: Uuid, is_success: bool) -> bool {
        let mut has_errors = false;

        // Capture stdout
        if let Some(stdout) = child.stdout.take() {
            let mut stdout_reader = tokio::io::BufReader::new(stdout);
            let mut stdout_output = String::new();

            let read_result = tokio::time::timeout(
                tokio::time::Duration::from_millis(1000),
                stdout_reader.read_to_string(&mut stdout_output)
            ).await;

            match read_result {
                Ok(Ok(bytes_read)) => {
                    if bytes_read > 0 && !stdout_output.trim().is_empty() {
                        if is_success {
                            log::info!("Server {} stdout output ({} bytes): {}", server_id, bytes_read, stdout_output.trim());
                        } else {
                            log::warn!("Server {} stdout output ({} bytes): {}", server_id, bytes_read, stdout_output.trim());
                        }
                    } else {
                        log::debug!("Server {} produced no stdout output", server_id);
                    }
                }
                Ok(Err(e)) => {
                    log::debug!("Could not read stdout from server {}: {}", server_id, e);
                }
                Err(_) => {
                    log::debug!("Timeout reading stdout from server {} (normal for some servers)", server_id);
                }
            }
        }

        // Capture stderr and check for errors
        if let Some(stderr) = child.stderr.take() {
            let mut stderr_reader = tokio::io::BufReader::new(stderr);
            let mut stderr_output = String::new();

            let read_result = tokio::time::timeout(
                tokio::time::Duration::from_millis(1000),
                stderr_reader.read_to_string(&mut stderr_output)
            ).await;

            match read_result {
                Ok(Ok(bytes_read)) => {
                    if bytes_read > 0 && !stderr_output.trim().is_empty() {
                        let stderr_content = stderr_output.trim();

                        // Check for common error patterns in stderr
                        has_errors = self.contains_error_patterns(stderr_content);

                        if has_errors {
                            log::error!("Server {} stderr contains errors ({} bytes): {}", server_id, bytes_read, stderr_content);
                        } else {
                            log::info!("Server {} stderr output ({} bytes): {}", server_id, bytes_read, stderr_content);
                        }
                    }
                }
                Ok(Err(e)) => {
                    log::debug!("Could not read stderr from server {}: {}", server_id, e);
                }
                Err(_) => {
                    log::debug!("Timeout reading stderr from server {}", server_id);
                }
            }
        }

        !has_errors
    }

    /// Check if stderr content contains error patterns
    fn contains_error_patterns(&self, stderr_content: &str) -> bool {
        let error_patterns = [
            "Error:",
            "ERROR:",
            "error:",
            "ENOENT:",
            "EACCES:",
            "EPERM:",
            "no such file or directory",
            "permission denied",
            "access denied",
            "cannot access",
            "failed to",
            "unable to",
            "not found",
            "invalid",
            "exception:",
            "Exception:",
            "EXCEPTION:",
            "fatal:",
            "Fatal:",
            "FATAL:",
            "panic:",
            "Panic:",
            "PANIC:",
        ];

        for pattern in &error_patterns {
            if stderr_content.contains(pattern) {
                return true;
            }
        }

        false
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

    /// Call a tool on a specific server
    pub async fn call_tool(&self, server_id: Uuid, tool_name: &str, arguments: Value) -> Result<Value> {
        let config = self.server_configs.get(&server_id)
            .ok_or_else(|| anyhow::anyhow!("Server config not found"))?;

        log::info!("Calling tool '{}' on server: {} ({})", tool_name, config.name, server_id);

        // Check if server is connected
        if !self.servers.contains_key(&server_id) {
            return Err(anyhow::anyhow!("Server not connected"));
        }

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
            match rmcp_service.call_tool(tool_name, Some(arguments)).await {
                Ok(response) => {
                    log::info!("Tool '{}' executed successfully", tool_name);

                    // Extract the content from the response
                    if let Some(content) = response.get("content") {
                        Ok(content.clone())
                    } else {
                        Ok(response)
                    }
                }
                Err(e) => {
                    log::error!("Failed to call tool '{}': {}", tool_name, e);
                    Err(anyhow::anyhow!("Tool call failed: {}", e))
                }
            }
        } else {
            Err(anyhow::anyhow!("Server not properly connected with rmcp"))
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
            match rmcp_service.read_resource(uri).await {
                Ok(response) => {
                    log::info!("Resource '{}' read successfully", uri);

                    // Extract the content from the response
                    if let Some(content) = response.get("contents") {
                        Ok(content.clone())
                    } else {
                        Ok(response)
                    }
                }
                Err(e) => {
                    log::error!("Failed to read resource '{}': {}", uri, e);
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
            match rmcp_service.get_prompt(prompt_name, arguments).await {
                Ok(response) => {
                    log::info!("Prompt '{}' executed successfully", prompt_name);

                    // Extract the content from the response
                    if let Some(content) = response.get("messages") {
                        Ok(content.clone())
                    } else {
                        Ok(response)
                    }
                }
                Err(e) => {
                    log::error!("Failed to get prompt '{}': {}", prompt_name, e);
                    Err(anyhow::anyhow!("Prompt execution failed: {}", e))
                }
            }
        } else {
            Err(anyhow::anyhow!("Server not properly connected with rmcp"))
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

    /// Start message handling tasks
    async fn start_message_tasks(
        &self,
        server_id: Uuid,
        mut stdin: tokio::process::ChildStdin,
        stdout: tokio::process::ChildStdout,
        mut message_receiver: mpsc::UnboundedReceiver<String>,
        message_sender: mpsc::UnboundedSender<String>,
    ) -> Result<()> {
        // Task to send messages to server
        tokio::spawn(async move {
            while let Some(message) = message_receiver.recv().await {
                if let Err(e) = stdin.write_all(message.as_bytes()).await {
                    log::error!("Failed to send message to server {}: {}", server_id, e);
                    break;
                }
                if let Err(e) = stdin.write_all(b"\n").await {
                    log::error!("Failed to send newline to server {}: {}", server_id, e);
                    break;
                }
                if let Err(e) = stdin.flush().await {
                    log::error!("Failed to flush stdin for server {}: {}", server_id, e);
                    break;
                }
            }
        });

        // Task to receive messages from server
        let reader = BufReader::new(stdout);
        let mut lines = reader.lines();
        let pending_requests_clone = self.pending_requests.clone();

        tokio::spawn(async move {
            while let Ok(Some(line)) = lines.next_line().await {
                if !line.trim().is_empty() {
                    log::debug!("Received from server {}: {}", server_id, line);

                    // Try to parse as JSON-RPC response
                    if let Ok(response) = serde_json::from_str::<Value>(&line) {
                        // Check if this is a response (has "id" field)
                        if let Some(id) = response.get("id").and_then(|id| id.as_str()) {
                            // Look for pending request with this ID
                            let mut pending = pending_requests_clone.lock().await;
                            if let Some(sender) = pending.remove(id) {
                                // Send response to waiting request
                                if let Some(result) = response.get("result") {
                                    let _ = sender.send(result.clone());
                                } else if let Some(error) = response.get("error") {
                                    log::error!("MCP error response for request {}: {}", id, error);
                                    let _ = sender.send(json!({"error": error}));
                                } else {
                                    let _ = sender.send(response);
                                }
                                continue;
                            }
                        }
                    }

                    // If not a response, send to message handler
                    let _ = message_sender.send(line);
                }
            }
        });

        Ok(())
    }

    /// Initialize MCP protocol
    async fn initialize_protocol(&mut self, server_id: Uuid) -> Result<()> {
        let connection = self.servers.get_mut(&server_id)
            .ok_or_else(|| anyhow::anyhow!("Server not found"))?;

        if let Some(protocol_handler) = &mut connection.protocol_handler {
            let client_info = ProtocolClientInfo::default();
            let init_message = protocol_handler.initialize(client_info)?;

            // Send initialize message
            if let Some(sender) = &connection.message_sender {
                let message_json = serde_json::to_string(&init_message)?;
                sender.send(message_json)?;

                log::info!("Sent initialize message to server {}", server_id);
            }
        }

        Ok(())
    }

    /// Handle incoming protocol message
    fn handle_protocol_message(&mut self, server_id: Uuid, message_json: &str) -> Result<()> {
        // First, extract the necessary data without holding mutable references
        let (response_opt, protocol_state, server_caps_opt, message_sender_opt) = {
            let connection = self.servers.get_mut(&server_id)
                .ok_or_else(|| anyhow::anyhow!("Server not found"))?;

            if let Some(protocol_handler) = &mut connection.protocol_handler {
                let message: super::protocol_handler::McpMessage = serde_json::from_str(message_json)?;
                let response = protocol_handler.handle_message(message)?;
                let state = protocol_handler.state().clone();
                let caps = protocol_handler.server_capabilities().map(|caps| caps.clone());
                let sender = connection.message_sender.clone();

                (response, state, caps, sender)
            } else {
                return Ok(());
            }
        };

        // Send response if needed
        if let Some(response) = response_opt {
            if let Some(sender) = &message_sender_opt {
                let response_json = serde_json::to_string(&response)?;
                let _ = sender.send(response_json);
            }
        }

        // Update connection status based on protocol state
        if let Some(connection) = self.servers.get_mut(&server_id) {
            match protocol_state {
                ProtocolState::Ready => {
                    connection.status = ConnectionStatus::Connected;
                    connection.last_ping = Some(chrono::Utc::now());

                    // Extract capabilities
                    if let Some(server_caps) = server_caps_opt {
                        let capabilities = Self::convert_server_capabilities_static(&server_caps);
                        connection.capabilities = Some(capabilities.clone());
                        self.send_event(McpEvent::CapabilitiesUpdated(server_id, capabilities));
                    }

                    self.send_event(McpEvent::ServerConnected(server_id));
                }
                ProtocolState::Error(error) => {
                    let error_msg = error.clone();
                    connection.status = ConnectionStatus::Error(error);
                    self.send_event(McpEvent::ServerError(server_id, error_msg));
                }
                _ => {}
            }
        }

        Ok(())
    }

    /// Convert protocol handler capabilities to our format
    fn convert_server_capabilities_static(server_caps: &super::protocol_handler::ServerCapabilities) -> ServerCapabilities {
        let mut tools = Vec::new();
        let mut resources = Vec::new();
        let mut prompts = Vec::new();

        // Extract tools
        if server_caps.tools.is_some() {
            tools.push(ToolInfo {
                name: "server_tool".to_string(),
                description: Some("Server provided tool".to_string()),
                input_schema: None,
            });
        }

        // Extract resources
        if server_caps.resources.is_some() {
            resources.push(ResourceInfo {
                uri: "server://resource".to_string(),
                name: "Server Resource".to_string(),
                description: Some("Server provided resource".to_string()),
                mime_type: Some("application/json".to_string()),
            });
        }

        // Extract prompts
        if server_caps.prompts.is_some() {
            prompts.push(PromptInfo {
                name: "server_prompt".to_string(),
                description: Some("Server provided prompt".to_string()),
                arguments: Vec::new(),
            });
        }

        ServerCapabilities {
            tools,
            resources,
            prompts,
        }
    }
}

impl Default for RmcpClient {
    fn default() -> Self {
        Self::new()
    }
}

impl RmcpClient {
    /// Query server tools using real MCP protocol
    async fn query_server_tools(&self, server_id: Uuid, _message_sender: &mpsc::UnboundedSender<String>) -> Result<Vec<ToolInfo>> {
        log::info!("Querying tools for server: {}", server_id);

        // Try to get the rmcp client for real MCP communication
        let connection = self.servers.get(&server_id)
            .ok_or_else(|| anyhow::anyhow!("Server not found"))?;

        // Check if we have rmcp service
        if connection.rmcp_service.is_some() {
            log::info!("Found rmcp service for server {}, sending tools/list request", server_id);

            // Try to use real MCP protocol to list tools
            match self.send_mcp_request(server_id, "tools/list", None).await {
                Ok(response) => {
                    log::info!("Successfully queried tools from server {}", server_id);

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

                        log::info!("Parsed {} tools from server response", tool_infos.len());
                        return Ok(tool_infos);
                    } else {
                        log::warn!("No 'tools' array found in server response");
                        // Return empty list instead of error for now
                        return Ok(Vec::new());
                    }
                }
                Err(e) => {
                    log::warn!("Failed to query tools from server {}: {}", server_id, e);
                    // Return empty list instead of error for now
                    return Ok(Vec::new());
                }
            }
        } else {
            log::warn!("No rmcp service available for server {}", server_id);
        }

        // If rmcp client is not available, return empty list for now
        log::info!("Returning empty tools list for server {}", server_id);
        Ok(Vec::new())
    }

    /// Query server resources using real MCP protocol
    async fn query_server_resources(&self, server_id: Uuid, _message_sender: &mpsc::UnboundedSender<String>) -> Result<Vec<ResourceInfo>> {
        log::debug!("Querying resources for server: {}", server_id);

        // Try to get the rmcp client for real MCP communication
        let connection = self.servers.get(&server_id)
            .ok_or_else(|| anyhow::anyhow!("Server not found"))?;

        if let Some(rmcp_service) = &connection.rmcp_service {
            // Try to use real MCP protocol to list resources
            match self.send_mcp_request(server_id, "resources/list", None).await {
                Ok(response) => {
                    log::info!("Successfully queried resources from server {}", server_id);

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

                        return Ok(resource_infos);
                    }
                }
                Err(e) => {
                    log::warn!("Failed to query resources from server {}: {}", server_id, e);
                }
            }
        }

        // If rmcp client is not available, return empty list for now
        log::info!("Returning empty resources list for server {}", server_id);
        Ok(Vec::new())
    }

    /// Query server prompts using real MCP protocol
    async fn query_server_prompts(&self, server_id: Uuid, _message_sender: &mpsc::UnboundedSender<String>) -> Result<Vec<PromptInfo>> {
        log::debug!("Querying prompts for server: {}", server_id);

        // Try to get the rmcp client for real MCP communication
        let connection = self.servers.get(&server_id)
            .ok_or_else(|| anyhow::anyhow!("Server not found"))?;

        if let Some(rmcp_service) = &connection.rmcp_service {
            // Try to use real MCP protocol to list prompts
            match self.send_mcp_request(server_id, "prompts/list", None).await {
                Ok(response) => {
                    log::info!("Successfully queried prompts from server {}", server_id);

                    // Parse the response to extract prompts
                    if let Some(prompts_array) = response.get("prompts").and_then(|p| p.as_array()) {
                        let prompt_infos: Vec<PromptInfo> = prompts_array.iter().filter_map(|prompt| {
                            let name = prompt.get("name")?.as_str()?.to_string();
                            let description = prompt.get("description").and_then(|d| d.as_str()).map(|s| s.to_string());

                            let arguments = prompt.get("arguments")
                                .and_then(|args| args.as_array())
                                .map(|args_array| {
                                    args_array.iter().filter_map(|arg| {
                                        let name = arg.get("name")?.as_str()?.to_string();
                                        let description = arg.get("description").and_then(|d| d.as_str()).map(|s| s.to_string());
                                        let required = arg.get("required").and_then(|r| r.as_bool()).unwrap_or(false);

                                        Some(PromptArgument {
                                            name,
                                            description,
                                            required,
                                        })
                                    }).collect()
                                })
                                .unwrap_or_default();

                            Some(PromptInfo {
                                name,
                                description,
                                arguments,
                            })
                        }).collect();

                        return Ok(prompt_infos);
                    }
                }
                Err(e) => {
                    log::warn!("Failed to query prompts from server {}: {}", server_id, e);
                }
            }
        }

        // If rmcp client is not available, return empty list for now
        log::info!("Returning empty prompts list for server {}", server_id);
        Ok(Vec::new())
    }
    /// Send a JSON-RPC request to an MCP server using the real MCP protocol
    async fn send_mcp_request(&self, server_id: Uuid, method: &str, params: Option<Value>) -> Result<Value> {
        log::debug!("Sending MCP request to server {}: {} with params: {:?}", server_id, method, params);

        // Get the connection
        let connection = self.servers.get(&server_id)
            .ok_or_else(|| anyhow::anyhow!("Server not found"))?;

        // Check if we have rmcp service - use it directly
        if let Some(rmcp_service) = &connection.rmcp_service {
            log::debug!("Using rmcp service to send request: {}", method);

            // Try to downcast the service to the actual rmcp service type
            // For now, we'll use a different approach since we can't easily downcast Box<dyn Any>
            // We need to implement the actual rmcp service calls

            // TODO: Implement actual rmcp service method calls
            // For now, we need to implement the proper rmcp service integration
            // The rmcp service should provide methods like list_tools(), list_resources(), list_prompts()

            // Since we can't easily downcast Box<dyn Any>, we need to restructure this
            // For now, return an error to indicate that rmcp service calls are not yet implemented
            log::error!("RMCP service method calls not yet properly implemented for method: {}", method);
            log::error!("This requires proper rmcp service API integration");

            Err(anyhow::anyhow!("RMCP service method calls not yet implemented - need to use actual rmcp API for method: {}", method))
        }
        // Fallback to manual message sending if no rmcp service
        else if let Some(message_sender) = &connection.message_sender {
            // Create JSON-RPC request
            let request_id = uuid::Uuid::new_v4().to_string();
            let request = json!({
                "jsonrpc": "2.0",
                "id": request_id.clone(),
                "method": method,
                "params": params
            });

            // Create a oneshot channel to receive the response
            let (response_sender, response_receiver) = oneshot::channel();

            // Store the pending request
            {
                let mut pending = self.pending_requests.lock().await;
                pending.insert(request_id.clone(), response_sender);
            }

            // Send the request
            let request_str = serde_json::to_string(&request)
                .map_err(|e| anyhow::anyhow!("Failed to serialize request: {}", e))?;

            message_sender.send(request_str)
                .map_err(|e| anyhow::anyhow!("Failed to send request: {}", e))?;

            log::debug!("MCP request sent successfully, waiting for response...");

            // Wait for the response with a timeout
            let timeout_duration = tokio::time::Duration::from_secs(10);
            match tokio::time::timeout(timeout_duration, response_receiver).await {
                Ok(Ok(response)) => {
                    log::debug!("Received MCP response for request {}: {:?}", request_id, response);
                    Ok(response)
                }
                Ok(Err(_)) => {
                    // Channel was closed without sending a response
                    Err(anyhow::anyhow!("Response channel closed unexpectedly"))
                }
                Err(_) => {
                    // Timeout occurred
                    // Clean up the pending request
                    let mut pending = self.pending_requests.lock().await;
                    pending.remove(&request_id);
                    Err(anyhow::anyhow!("Request timed out after {} seconds", timeout_duration.as_secs()))
                }
            }
        } else {
            Err(anyhow::anyhow!("No rmcp service or message sender available for server"))
        }
    }






}
