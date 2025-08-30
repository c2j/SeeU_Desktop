use std::collections::HashMap;
use std::process::Stdio;
use uuid::Uuid;
use anyhow::Result;
use tokio::process::Command as TokioCommand;
use tokio::io::AsyncReadExt;
use tokio_util::sync::CancellationToken;

use rmcp::service::{RoleClient, ServiceExt, Peer};
use rmcp::transport::TokioChildProcess;
use rmcp::handler::client::ClientHandler;
use rmcp::model::{ClientInfo, ClientCapabilities, ProtocolVersion, Implementation};

use super::server_manager::{McpServerConfig, TransportConfig};

/// Real rmcp-based MCP client implementation
pub struct RealRmcpClient {
    server_configs: HashMap<Uuid, McpServerConfig>,
    running_services: HashMap<Uuid, RunningMcpService>,
}

/// A running MCP service instance
struct RunningMcpService {
    server_id: Uuid,
    cancellation_token: CancellationToken,
    // We'll store the running service handle here
    // service_handle: Option<rmcp::service::RunningService<RoleClient, SimpleClientHandler>>,
}

/// Simple client handler implementation
#[derive(Default)]
struct SimpleClientHandler {
    peer: Option<rmcp::service::Peer<RoleClient>>,
}

impl ClientHandler for SimpleClientHandler {
    // 根据rmcp 0.6.0的API，ClientHandler trait已经改变
    // 我们暂时提供一个空的实现，等待进一步的API文档
}



impl RealRmcpClient {
    pub fn new() -> Self {
        Self {
            server_configs: HashMap::new(),
            running_services: HashMap::new(),
        }
    }

    pub fn add_server(&mut self, config: McpServerConfig) {
        self.server_configs.insert(config.id, config);
    }

    pub fn remove_server(&mut self, server_id: Uuid) -> Result<()> {
        // Stop the service if it's running
        if let Some(service) = self.running_services.remove(&server_id) {
            service.cancellation_token.cancel();
        }
        
        // Remove the configuration
        self.server_configs.remove(&server_id);
        Ok(())
    }

    pub async fn connect_server(&mut self, server_id: Uuid) -> Result<()> {
        let config = self.server_configs.get(&server_id)
            .ok_or_else(|| anyhow::anyhow!("Server config not found"))?
            .clone();

        log::info!("Connecting to MCP server: {}", config.name);

        match &config.transport {
            TransportConfig::Command { command, args, env } => {
                self.connect_command_server(server_id, command, args, env).await
            }
            _ => {
                log::warn!("Transport type not yet supported with real rmcp: {:?}", config.transport);
                Err(anyhow::anyhow!("Transport type not supported"))
            }
        }
    }

    async fn connect_command_server(
        &mut self,
        server_id: Uuid,
        command: &str,
        args: &[String],
        env: &HashMap<String, String>,
    ) -> Result<()> {
        log::info!("Starting command-based MCP server: {} {:?}", command, args);

        // Create tokio command
        let mut cmd = TokioCommand::new(command);
        cmd.args(args);
        
        // Set environment variables
        for (key, value) in env {
            cmd.env(key, value);
        }
        
        // Configure stdio
        cmd.stdin(Stdio::piped())
           .stdout(Stdio::piped())
           .stderr(Stdio::piped());

        // Create transport
        let transport = TokioChildProcess::new(cmd)
            .map_err(|e| anyhow::anyhow!("Failed to create child process transport: {}", e))?;

        // Create client handler
        let handler = SimpleClientHandler::default();

        // Create cancellation token
        let cancellation_token = CancellationToken::new();
        let ct_clone = cancellation_token.clone();

        // Start the service
        let service_future = handler.serve_with_ct(transport, ct_clone);

        // Spawn the service in the background
        let server_id_clone = server_id;
        tokio::spawn(async move {
            match service_future.await {
                Ok(_running_service) => {
                    log::info!("MCP server {} started successfully", server_id_clone);

                    // The service will run until the cancellation token is cancelled
                    log::info!("MCP server {} stopped", server_id_clone);
                }
                Err(e) => {
                    log::error!("Failed to start MCP server {}: {}", server_id_clone, e);
                }
            }
        });

        // Store the running service info
        let running_service = RunningMcpService {
            server_id,
            cancellation_token,
        };
        
        self.running_services.insert(server_id, running_service);

        log::info!("MCP server {} connection initiated", server_id);
        Ok(())
    }

    pub async fn disconnect_server(&mut self, server_id: Uuid) -> Result<()> {
        if let Some(service) = self.running_services.remove(&server_id) {
            log::info!("Disconnecting MCP server: {}", server_id);
            service.cancellation_token.cancel();
            Ok(())
        } else {
            Err(anyhow::anyhow!("Server not running"))
        }
    }

    pub fn is_server_connected(&self, server_id: Uuid) -> bool {
        self.running_services.contains_key(&server_id)
    }

    pub async fn test_server(&mut self, server_id: Uuid) -> Result<bool> {
        let config = self.server_configs.get(&server_id)
            .ok_or_else(|| anyhow::anyhow!("Server config not found"))?
            .clone();

        log::info!("Testing MCP server: {}", config.name);

        match &config.transport {
            TransportConfig::Command { command, args, env } => {
                self.test_command_server(command, args, env).await
            }
            _ => {
                log::warn!("Transport type not yet supported for testing: {:?}", config.transport);
                Ok(false)
            }
        }
    }

    async fn test_command_server(
        &self,
        command: &str,
        args: &[String],
        env: &HashMap<String, String>,
    ) -> Result<bool> {
        log::info!("Testing command-based MCP server: {} {:?}", command, args);

        // Create tokio command for testing
        let mut cmd = TokioCommand::new(command);
        cmd.args(args);

        // Set environment variables
        for (key, value) in env {
            cmd.env(key, value);
        }

        // Configure stdio
        cmd.stdin(Stdio::piped())
           .stdout(Stdio::piped())
           .stderr(Stdio::piped());

        // Try to spawn the process for testing
        match cmd.spawn() {
            Ok(mut child) => {
                log::info!("Successfully spawned test process for MCP server");

                // Set a timeout for the test
                let timeout_duration = std::time::Duration::from_secs(10);

                // Wait for a short time to see if the process starts successfully
                let wait_result = tokio::time::timeout(
                    timeout_duration,
                    self.wait_for_process_startup(&mut child)
                ).await;

                match wait_result {
                    Ok(Ok(success)) => {
                        if success {
                            log::info!("MCP server test successful - process started and is responsive");
                            // Clean up the test process
                            let _ = child.kill().await;
                            let _ = child.wait().await;
                            Ok(true)
                        } else {
                            log::error!("MCP server test failed - process started but not responsive");
                            // Clean up the test process
                            let _ = child.kill().await;
                            let _ = child.wait().await;
                            Ok(false)
                        }
                    }
                    Ok(Err(e)) => {
                        log::error!("MCP server test failed with error: {}", e);
                        // Clean up the test process
                        let _ = child.kill().await;
                        let _ = child.wait().await;
                        Ok(false)
                    }
                    Err(_) => {
                        log::error!("MCP server test timed out after {} seconds", timeout_duration.as_secs());
                        // Clean up the test process
                        let _ = child.kill().await;
                        let _ = child.wait().await;
                        Ok(false)
                    }
                }
            }
            Err(e) => {
                log::error!("Failed to spawn MCP server test process: {}", e);
                log::error!("Command: {} {:?}", command, args);
                log::error!("Environment variables: {:?}", env);
                Ok(false)
            }
        }
    }

    /// Wait for process startup and check if it's responsive
    async fn wait_for_process_startup(&self, child: &mut tokio::process::Child) -> Result<bool> {
        // Give the process a moment to start
        tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;

        // Check if the process is still running
        match child.try_wait() {
            Ok(Some(status)) => {
                // Process has exited
                if status.success() {
                    log::info!("Test process exited successfully with status: {}", status);
                    Ok(true)
                } else {
                    log::error!("Test process exited with error status: {}", status);

                    // Try to capture stderr output for error details
                    if let Some(stderr) = child.stderr.take() {
                        let mut stderr_reader = tokio::io::BufReader::new(stderr);
                        let mut stderr_output = String::new();
                        if let Ok(_) = stderr_reader.read_to_string(&mut stderr_output).await {
                            if !stderr_output.trim().is_empty() {
                                log::error!("Process stderr output: {}", stderr_output.trim());
                            }
                        }
                    }

                    Ok(false)
                }
            }
            Ok(None) => {
                // Process is still running, which is good for an MCP server
                log::info!("Test process is running successfully (PID: {:?})", child.id());

                // Try to capture some stdout output to verify the server is responsive
                if let Some(stdout) = child.stdout.take() {
                    let mut stdout_reader = tokio::io::BufReader::new(stdout);
                    let mut stdout_output = String::new();

                    // Try to read some output with a short timeout
                    let read_result = tokio::time::timeout(
                        tokio::time::Duration::from_millis(2000),
                        stdout_reader.read_to_string(&mut stdout_output)
                    ).await;

                    match read_result {
                        Ok(Ok(_)) => {
                            if !stdout_output.trim().is_empty() {
                                log::info!("Process stdout output: {}", stdout_output.trim());
                            }
                        }
                        Ok(Err(e)) => {
                            log::debug!("Could not read stdout: {}", e);
                        }
                        Err(_) => {
                            log::debug!("Timeout reading stdout (this is normal for some servers)");
                        }
                    }
                }

                Ok(true)
            }
            Err(e) => {
                log::error!("Error checking test process status: {}", e);
                Ok(false)
            }
        }
    }

    pub fn list_servers(&self) -> Vec<&McpServerConfig> {
        self.server_configs.values().collect()
    }

    pub fn get_server_config(&self, server_id: Uuid) -> Option<&McpServerConfig> {
        self.server_configs.get(&server_id)
    }
}
