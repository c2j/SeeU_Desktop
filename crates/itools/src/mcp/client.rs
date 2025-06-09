use std::collections::HashMap;
use uuid::Uuid;
use serde_json::Value;
use anyhow::Result;
use tokio::sync::mpsc;

use super::protocol::{
    McpMessage, McpRequest, McpResponse, McpNotification, McpResult,
    InitializeParams, ClientInfo, ClientCapabilities, SamplingCapability,
    methods, error_codes
};
use super::transport::{McpTransport, TransportType};

/// MCP client for communicating with plugin servers
#[derive(Debug)]
pub struct McpClient {
    /// Active connections to plugin servers
    connections: HashMap<Uuid, PluginConnection>,

    /// Message queue for outgoing messages
    outgoing_queue: Vec<(Uuid, McpMessage)>,

    /// Message queue for incoming messages
    incoming_queue: Vec<(Uuid, McpMessage)>,

    /// Request ID counter
    next_request_id: u64,

    /// Pending requests
    pending_requests: HashMap<u64, PendingRequest>,
}

/// Connection to a plugin server
#[derive(Debug)]
pub struct PluginConnection {
    pub plugin_id: Uuid,
    pub connection_id: Uuid,
    pub transport: McpTransport,
    pub status: ConnectionStatus,
    pub capabilities: Option<super::protocol::ServerCapabilities>,
}

/// Connection status
#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionStatus {
    Disconnected,
    Connecting,
    Initializing,
    Connected,
    Error(String),
}

/// Pending request information
#[derive(Debug)]
pub struct PendingRequest {
    pub request_id: u64,
    pub plugin_id: Uuid,
    pub method: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl McpClient {
    /// Create a new MCP client
    pub fn new() -> Self {
        Self {
            connections: HashMap::new(),
            outgoing_queue: Vec::new(),
            incoming_queue: Vec::new(),
            next_request_id: 1,
            pending_requests: HashMap::new(),
        }
    }

    /// Initialize the MCP client
    pub fn initialize(&mut self) {
        log::info!("Initializing MCP client");

        // TODO: Setup transport layers
        // TODO: Initialize connection pools
    }

    /// Process incoming and outgoing messages
    pub fn process_messages(&mut self) {
        // Process incoming messages
        self.process_incoming_messages();

        // Process outgoing messages
        self.process_outgoing_messages();

        // Clean up expired requests
        self.cleanup_expired_requests();
    }

    /// Connect to a plugin server
    pub fn connect_to_plugin(&mut self, plugin_id: Uuid, transport_type: TransportType) -> Result<Uuid> {
        let connection_id = Uuid::new_v4();

        let transport = McpTransport::new(transport_type)?;

        let connection = PluginConnection {
            plugin_id,
            connection_id,
            transport,
            status: ConnectionStatus::Connecting,
            capabilities: None,
        };

        self.connections.insert(connection_id, connection);

        // Send initialize request
        self.send_initialize_request(connection_id)?;

        log::info!("Connecting to plugin {} via connection {}", plugin_id, connection_id);

        Ok(connection_id)
    }

    /// Disconnect from a plugin server
    pub fn disconnect_from_plugin(&mut self, connection_id: Uuid) -> Result<()> {
        if let Some(mut connection) = self.connections.remove(&connection_id) {
            connection.status = ConnectionStatus::Disconnected;
            // TODO: Close transport connection
            log::info!("Disconnected from plugin via connection {}", connection_id);
        }

        Ok(())
    }

    /// Send a request to a plugin
    pub fn send_request(&mut self, plugin_id: Uuid, method: String, params: Option<Value>) -> Result<u64> {
        let connection_id = self.find_connection_for_plugin(plugin_id)
            .ok_or_else(|| anyhow::anyhow!("No connection found for plugin"))?;

        let request_id = self.next_request_id;
        self.next_request_id += 1;

        let request = McpRequest::new(
            Value::Number(request_id.into()),
            method.clone(),
            params,
        );

        // Add to pending requests
        self.pending_requests.insert(request_id, PendingRequest {
            request_id,
            plugin_id,
            method: method.clone(),
            timestamp: chrono::Utc::now(),
        });

        // Queue for sending
        self.outgoing_queue.push((connection_id, McpMessage::Request(request)));

        log::debug!("Queued request {} to plugin {}: {}", request_id, plugin_id, method);

        Ok(request_id)
    }

    /// Send a notification to a plugin
    pub fn send_notification(&mut self, plugin_id: Uuid, method: String, params: Option<Value>) -> Result<()> {
        let connection_id = self.find_connection_for_plugin(plugin_id)
            .ok_or_else(|| anyhow::anyhow!("No connection found for plugin"))?;

        let notification = McpNotification::new(method.clone(), params);

        // Queue for sending
        self.outgoing_queue.push((connection_id, McpMessage::Notification(notification)));

        log::debug!("Queued notification to plugin {}: {}", plugin_id, method);

        Ok(())
    }

    /// Get connection status for a plugin
    pub fn get_plugin_connection_status(&self, plugin_id: Uuid) -> Option<ConnectionStatus> {
        self.connections
            .values()
            .find(|conn| conn.plugin_id == plugin_id)
            .map(|conn| conn.status.clone())
    }

    /// Get all connected plugins
    pub fn get_connected_plugins(&self) -> Vec<Uuid> {
        self.connections
            .values()
            .filter(|conn| conn.status == ConnectionStatus::Connected)
            .map(|conn| conn.plugin_id)
            .collect()
    }

    /// Send initialize request to a connection
    fn send_initialize_request(&mut self, connection_id: Uuid) -> Result<()> {
        let client_info = ClientInfo {
            name: "SeeU iTools".to_string(),
            version: "1.0.0".to_string(),
        };

        let request = McpRequest::initialize(
            Value::Number(self.next_request_id.into()),
            client_info,
        );

        self.next_request_id += 1;

        // Queue for sending
        self.outgoing_queue.push((connection_id, McpMessage::Request(request)));

        // Update connection status
        if let Some(connection) = self.connections.get_mut(&connection_id) {
            connection.status = ConnectionStatus::Initializing;
        }

        Ok(())
    }

    /// Find connection ID for a plugin
    fn find_connection_for_plugin(&self, plugin_id: Uuid) -> Option<Uuid> {
        self.connections
            .iter()
            .find(|(_, conn)| conn.plugin_id == plugin_id && conn.status == ConnectionStatus::Connected)
            .map(|(id, _)| *id)
    }

    /// Process incoming messages from plugins
    fn process_incoming_messages(&mut self) {
        // TODO: Read messages from transport layers
        // TODO: Handle responses, notifications, and errors
        // TODO: Update connection states

        let messages: Vec<_> = self.incoming_queue.drain(..).collect();
        for (connection_id, message) in messages {
            self.handle_incoming_message(connection_id, message);
        }
    }

    /// Process outgoing messages to plugins
    fn process_outgoing_messages(&mut self) {
        // TODO: Send messages via transport layers

        for (connection_id, message) in self.outgoing_queue.drain(..) {
            if let Some(connection) = self.connections.get_mut(&connection_id) {
                // TODO: Actually send the message via transport
                log::debug!("Sending message to connection {}: {:?}", connection_id, message);
            }
        }
    }

    /// Handle an incoming message
    fn handle_incoming_message(&mut self, connection_id: Uuid, message: McpMessage) {
        match message {
            McpMessage::Response(response) => {
                self.handle_response(connection_id, response);
            }
            McpMessage::Notification(notification) => {
                self.handle_notification(connection_id, notification);
            }
            McpMessage::Request(request) => {
                self.handle_request(connection_id, request);
            }
        }
    }

    /// Handle a response message
    fn handle_response(&mut self, connection_id: Uuid, response: McpResponse) {
        if let Value::Number(id_num) = &response.id {
            if let Some(request_id) = id_num.as_u64() {
                if let Some(pending) = self.pending_requests.remove(&request_id) {
                    match &response.result {
                        McpResult::Success { result } => {
                            log::debug!("Received successful response for request {}", request_id);

                            // Handle initialize response
                            if pending.method == methods::INITIALIZE {
                                self.handle_initialize_response(connection_id, result);
                            }
                        }
                        McpResult::Error { error } => {
                            log::warn!("Received error response for request {}: {} - {}",
                                     request_id, error.code, error.message);
                        }
                    }
                }
            }
        }
    }

    /// Handle a notification message
    fn handle_notification(&mut self, connection_id: Uuid, notification: McpNotification) {
        log::debug!("Received notification from connection {}: {}", connection_id, notification.method);

        // TODO: Handle specific notification types
    }

    /// Handle a request message (from server to client)
    fn handle_request(&mut self, connection_id: Uuid, request: McpRequest) {
        log::debug!("Received request from connection {}: {}", connection_id, request.method);

        // TODO: Handle specific request types (like sampling requests)
    }

    /// Handle initialize response
    fn handle_initialize_response(&mut self, connection_id: Uuid, result: &Value) {
        if let Some(connection) = self.connections.get_mut(&connection_id) {
            // TODO: Parse server capabilities from result
            connection.status = ConnectionStatus::Connected;
            log::info!("Successfully initialized connection {} to plugin {}",
                     connection_id, connection.plugin_id);
        }
    }

    /// Clean up expired requests
    fn cleanup_expired_requests(&mut self) {
        let cutoff = chrono::Utc::now() - chrono::Duration::minutes(5);
        self.pending_requests.retain(|_, request| request.timestamp > cutoff);
    }
}

impl Default for McpClient {
    fn default() -> Self {
        Self::new()
    }
}
