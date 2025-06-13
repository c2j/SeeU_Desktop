use std::collections::HashMap;
use uuid::Uuid;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use anyhow::Result;
use tokio::sync::mpsc;

/// MCP Protocol version
pub const MCP_VERSION: &str = "2024-11-05";

/// MCP Protocol handler for managing the complete protocol lifecycle
#[derive(Debug)]
pub struct McpProtocolHandler {
    /// Connection ID
    connection_id: Uuid,
    
    /// Protocol state
    state: ProtocolState,
    
    /// Client capabilities
    client_capabilities: ClientCapabilities,
    
    /// Server capabilities (received during initialization)
    server_capabilities: Option<ServerCapabilities>,
    
    /// Pending requests
    pending_requests: HashMap<String, PendingRequest>,
    
    /// Event sender
    event_sender: Option<mpsc::UnboundedSender<ProtocolEvent>>,
}

/// Protocol state machine
#[derive(Debug, Clone, PartialEq)]
pub enum ProtocolState {
    /// Not connected
    Disconnected,
    /// Connecting - transport established but not initialized
    Connecting,
    /// Initializing - sent initialize request
    Initializing,
    /// Connected and ready
    Ready,
    /// Error state
    Error(String),
}

/// Client capabilities that we support
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientCapabilities {
    /// Experimental capabilities
    pub experimental: Option<HashMap<String, Value>>,
    
    /// Sampling capability
    pub sampling: Option<SamplingCapability>,
    
    /// Roots capability
    pub roots: Option<RootsCapability>,
}

/// Sampling capability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamplingCapability {
    /// Whether client supports sampling
    pub enabled: bool,
}

/// Roots capability  
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RootsCapability {
    /// Whether client supports listing roots
    pub list_changed: bool,
}

/// Server capabilities received from server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerCapabilities {
    /// Experimental capabilities
    pub experimental: Option<HashMap<String, Value>>,
    
    /// Logging capability
    pub logging: Option<LoggingCapability>,
    
    /// Prompts capability
    pub prompts: Option<PromptsCapability>,
    
    /// Resources capability
    pub resources: Option<ResourcesCapability>,
    
    /// Tools capability
    pub tools: Option<ToolsCapability>,
}

/// Logging capability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingCapability {
    /// Supported log levels
    pub levels: Option<Vec<String>>,
}

/// Prompts capability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptsCapability {
    /// Whether server supports listing prompts
    pub list_changed: Option<bool>,
}

/// Resources capability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourcesCapability {
    /// Whether server supports subscribing to resource changes
    pub subscribe: Option<bool>,
    
    /// Whether server supports listing resources
    pub list_changed: Option<bool>,
}

/// Tools capability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolsCapability {
    /// Whether server supports listing tools
    pub list_changed: Option<bool>,
}

/// Pending request information
#[derive(Debug, Clone)]
pub struct PendingRequest {
    pub id: String,
    pub method: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Protocol events
#[derive(Debug, Clone)]
pub enum ProtocolEvent {
    /// State changed
    StateChanged(ProtocolState),
    
    /// Server capabilities received
    CapabilitiesReceived(ServerCapabilities),
    
    /// Request completed
    RequestCompleted(String, Value),
    
    /// Request failed
    RequestFailed(String, String),
    
    /// Notification received
    NotificationReceived(String, Value),
    
    /// Protocol error
    ProtocolError(String),
}

/// MCP JSON-RPC message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpMessage {
    pub jsonrpc: String,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Value>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub method: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<McpError>,
}

/// MCP error object
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpError {
    pub code: i32,
    pub message: String,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

impl McpProtocolHandler {
    /// Create a new protocol handler
    pub fn new(connection_id: Uuid) -> Self {
        Self {
            connection_id,
            state: ProtocolState::Disconnected,
            client_capabilities: ClientCapabilities::default(),
            server_capabilities: None,
            pending_requests: HashMap::new(),
            event_sender: None,
        }
    }

    /// Set event sender
    pub fn set_event_sender(&mut self, sender: mpsc::UnboundedSender<ProtocolEvent>) {
        self.event_sender = Some(sender);
    }

    /// Get current state
    pub fn state(&self) -> &ProtocolState {
        &self.state
    }

    /// Get server capabilities
    pub fn server_capabilities(&self) -> Option<&ServerCapabilities> {
        self.server_capabilities.as_ref()
    }

    /// Start protocol initialization
    pub fn initialize(&mut self, client_info: ClientInfo) -> Result<McpMessage> {
        if self.state != ProtocolState::Connecting {
            return Err(anyhow::anyhow!("Cannot initialize in state: {:?}", self.state));
        }

        self.set_state(ProtocolState::Initializing);

        let request_id = Uuid::new_v4().to_string();
        
        let params = json!({
            "protocolVersion": MCP_VERSION,
            "capabilities": self.client_capabilities,
            "clientInfo": client_info
        });

        let message = McpMessage {
            jsonrpc: "2.0".to_string(),
            id: Some(Value::String(request_id.clone())),
            method: Some("initialize".to_string()),
            params: Some(params),
            result: None,
            error: None,
        };

        self.pending_requests.insert(request_id.clone(), PendingRequest {
            id: request_id.clone(),
            method: "initialize".to_string(),
            timestamp: chrono::Utc::now(),
        });

        Ok(message)
    }

    /// Handle incoming message
    pub fn handle_message(&mut self, message: McpMessage) -> Result<Option<McpMessage>> {
        // Handle response
        if let Some(id) = &message.id {
            if let Some(id_str) = id.as_str() {
                if let Some(pending) = self.pending_requests.remove(id_str) {
                    return self.handle_response(pending, message);
                }
            }
        }

        // Handle request/notification
        if let Some(method) = &message.method {
            return self.handle_request_or_notification(method, &message);
        }

        Err(anyhow::anyhow!("Invalid message format"))
    }

    /// Handle response message
    fn handle_response(&mut self, pending: PendingRequest, message: McpMessage) -> Result<Option<McpMessage>> {
        match pending.method.as_str() {
            "initialize" => {
                if let Some(error) = message.error {
                    let error_msg = error.message.clone();
                    self.set_state(ProtocolState::Error(error.message));
                    self.send_event(ProtocolEvent::RequestFailed(pending.id, error_msg));
                } else if let Some(result) = message.result {
                    self.handle_initialize_response(result)?;
                }
            }
            _ => {
                if let Some(error) = message.error {
                    self.send_event(ProtocolEvent::RequestFailed(pending.id, error.message));
                } else if let Some(result) = message.result {
                    self.send_event(ProtocolEvent::RequestCompleted(pending.id, result));
                }
            }
        }

        Ok(None)
    }

    /// Handle initialize response
    fn handle_initialize_response(&mut self, result: Value) -> Result<()> {
        // Parse server capabilities
        if let Some(capabilities) = result.get("capabilities") {
            let server_caps: ServerCapabilities = serde_json::from_value(capabilities.clone())?;
            self.server_capabilities = Some(server_caps.clone());
            self.send_event(ProtocolEvent::CapabilitiesReceived(server_caps));
        }

        self.set_state(ProtocolState::Ready);
        Ok(())
    }

    /// Handle request or notification
    fn handle_request_or_notification(&mut self, method: &str, message: &McpMessage) -> Result<Option<McpMessage>> {
        match method {
            "notifications/initialized" => {
                // Server confirms initialization is complete
                log::info!("Server initialization confirmed");
                Ok(None)
            }
            "notifications/progress" => {
                // Progress notification
                if let Some(params) = &message.params {
                    log::debug!("Progress: {:?}", params);
                }
                Ok(None)
            }
            _ => {
                // Handle other notifications
                if let Some(params) = &message.params {
                    self.send_event(ProtocolEvent::NotificationReceived(method.to_string(), params.clone()));
                }
                Ok(None)
            }
        }
    }

    /// Create a request message
    pub fn create_request(&mut self, method: &str, params: Option<Value>) -> McpMessage {
        let request_id = Uuid::new_v4().to_string();
        
        self.pending_requests.insert(request_id.clone(), PendingRequest {
            id: request_id.clone(),
            method: method.to_string(),
            timestamp: chrono::Utc::now(),
        });

        McpMessage {
            jsonrpc: "2.0".to_string(),
            id: Some(Value::String(request_id)),
            method: Some(method.to_string()),
            params,
            result: None,
            error: None,
        }
    }

    /// Set protocol state
    fn set_state(&mut self, new_state: ProtocolState) {
        if self.state != new_state {
            self.state = new_state.clone();
            self.send_event(ProtocolEvent::StateChanged(new_state));
        }
    }

    /// Send event
    fn send_event(&self, event: ProtocolEvent) {
        if let Some(sender) = &self.event_sender {
            let _ = sender.send(event);
        }
    }

    /// Connect (set state to connecting)
    pub fn connect(&mut self) {
        self.set_state(ProtocolState::Connecting);
    }

    /// Disconnect
    pub fn disconnect(&mut self) {
        self.pending_requests.clear();
        self.server_capabilities = None;
        self.set_state(ProtocolState::Disconnected);
    }

    /// Check if ready
    pub fn is_ready(&self) -> bool {
        self.state == ProtocolState::Ready
    }
}

/// Client information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientInfo {
    pub name: String,
    pub version: String,
}

impl Default for ClientCapabilities {
    fn default() -> Self {
        Self {
            experimental: None,
            sampling: Some(SamplingCapability { enabled: false }),
            roots: Some(RootsCapability { list_changed: true }),
        }
    }
}

impl Default for ClientInfo {
    fn default() -> Self {
        Self {
            name: "SeeU Desktop".to_string(),
            version: "0.1.0".to_string(),
        }
    }
}
