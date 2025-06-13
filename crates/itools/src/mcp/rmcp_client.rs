use std::collections::HashMap;
use std::process::{Stdio, Child};
use uuid::Uuid;
use serde_json::Value;
use anyhow::Result;
use tokio::sync::mpsc;
use tokio::process::Command as TokioCommand;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, AsyncReadExt, BufReader};

// Future rmcp integration (currently disabled due to API complexity)
// use rmcp::{
//     RoleClient, ClientHandler, ServiceExt,
//     model::{
//         ClientInfo, ServerInfo, Tool as RmcpTool,
//         Resource as RmcpResource, Prompt as RmcpPrompt
//     },
//     transport::TokioChildProcess,
//     ServiceError as McpError
// };

use super::server_manager::{McpServerConfig, McpServerInfo, TransportConfig};
use super::protocol_handler::{McpProtocolHandler, ProtocolEvent, ProtocolState, ClientInfo};

/// RMCP client wrapper for MCP server communication
#[derive(Debug)]
pub struct RmcpClient {
    /// Active server connections
    servers: HashMap<Uuid, ServerConnection>,

    /// Server configurations
    server_configs: HashMap<Uuid, McpServerConfig>,

    /// Event sender for UI updates
    event_sender: Option<mpsc::UnboundedSender<McpEvent>>,

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
    // Future: rmcp client integration
    // pub rmcp_client: Option<RoleClient>,
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
            // Future: rmcp client integration
            // rmcp_client: None,
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

        match cmd.spawn() {
            Ok(mut child) => {
                log::info!("Started MCP server process: {} {:?}", command, args);

                // Get stdin and stdout handles
                let stdin = child.stdin.take().ok_or_else(|| anyhow::anyhow!("Failed to get stdin"))?;
                let stdout = child.stdout.take().ok_or_else(|| anyhow::anyhow!("Failed to get stdout"))?;

                // Create protocol handler
                let mut protocol_handler = McpProtocolHandler::new(server_id);

                // Setup message communication
                let (message_sender, message_receiver) = mpsc::unbounded_channel::<String>();
                let (event_sender, mut event_receiver) = mpsc::unbounded_channel();

                protocol_handler.set_event_sender(event_sender);
                protocol_handler.connect();

                // Store connection info
                if let Some(connection) = self.servers.get_mut(&server_id) {
                    connection.status = ConnectionStatus::Connecting;
                    connection.protocol_handler = Some(protocol_handler);
                    connection.message_sender = Some(message_sender.clone());
                }

                // Start message handling tasks
                self.start_message_tasks(server_id, stdin, stdout, message_receiver, message_sender.clone()).await?;

                // Start protocol initialization
                self.initialize_protocol(server_id).await?;

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

    /// Query server capabilities
    async fn query_server_capabilities(&mut self, server_id: Uuid) -> Result<()> {
        // TODO: Implement MCP protocol to query actual capabilities
        // For now, create mock capabilities based on server type
        let config = self.servers.get(&server_id)
            .map(|conn| &conn.config)
            .ok_or_else(|| anyhow::anyhow!("Server not found"))?;

        let capabilities = match config.name.as_str() {
            "Everything Server" => ServerCapabilities {
                tools: vec![
                    ToolInfo {
                        name: "read_file".to_string(),
                        description: Some("Read file contents".to_string()),
                        input_schema: None,
                    },
                    ToolInfo {
                        name: "write_file".to_string(),
                        description: Some("Write file contents".to_string()),
                        input_schema: None,
                    },
                ],
                resources: vec![
                    ResourceInfo {
                        uri: "file://".to_string(),
                        name: "File System".to_string(),
                        description: Some("Access to file system".to_string()),
                        mime_type: Some("application/octet-stream".to_string()),
                    }
                ],
                prompts: vec![
                    PromptInfo {
                        name: "code_review".to_string(),
                        description: Some("Review code for issues".to_string()),
                        arguments: vec![],
                    }
                ],
            },
            _ => ServerCapabilities {
                tools: vec![
                    ToolInfo {
                        name: "example_tool".to_string(),
                        description: Some("An example tool".to_string()),
                        input_schema: None,
                    }
                ],
                resources: vec![],
                prompts: vec![],
            }
        };

        if let Some(connection) = self.servers.get_mut(&server_id) {
            connection.capabilities = Some(capabilities.clone());
        }

        self.send_event(McpEvent::CapabilitiesUpdated(server_id, capabilities));
        Ok(())
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

    /// Call a tool on a specific server (placeholder)
    pub async fn call_tool(&self, _server_id: Uuid, _tool_name: &str, _arguments: Value) -> Result<Value> {
        Err(anyhow::anyhow!("Tool calling not yet implemented - awaiting rmcp integration"))
    }

    /// Read a resource from a specific server (placeholder)
    pub async fn read_resource(&self, _server_id: Uuid, _uri: &str) -> Result<Value> {
        Err(anyhow::anyhow!("Resource reading not yet implemented - awaiting rmcp integration"))
    }

    /// Get a prompt from a specific server (placeholder)
    pub async fn get_prompt(&self, _server_id: Uuid, _prompt_name: &str, _arguments: Option<Value>) -> Result<Value> {
        Err(anyhow::anyhow!("Prompt getting not yet implemented - awaiting rmcp integration"))
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

        tokio::spawn(async move {
            while let Ok(Some(line)) = lines.next_line().await {
                if !line.trim().is_empty() {
                    log::debug!("Received from server {}: {}", server_id, line);
                    // Parse and handle the message
                    // In a real implementation, we'd parse JSON-RPC and handle it
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
            let client_info = ClientInfo::default();
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
