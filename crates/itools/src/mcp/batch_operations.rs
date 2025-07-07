use std::collections::HashMap;
use uuid::Uuid;
use serde::{Deserialize, Serialize};
use anyhow::Result;
use tokio::sync::mpsc;

use super::server_manager::{McpServerManager, McpServerConfig};

/// Batch operation types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BatchOperation {
    /// Connect to multiple servers
    Connect(Vec<Uuid>),
    
    /// Disconnect from multiple servers
    Disconnect(Vec<Uuid>),
    
    /// Enable multiple servers
    Enable(Vec<Uuid>),
    
    /// Disable multiple servers
    Disable(Vec<Uuid>),
    
    /// Test multiple servers
    Test(Vec<Uuid>),
    
    /// Update configuration for multiple servers
    UpdateConfig(Vec<Uuid>, ConfigUpdate),
    
    /// Delete multiple servers
    Delete(Vec<Uuid>),
    
    /// Import servers from file
    Import(String), // file path
    
    /// Export servers to file
    Export(Vec<Uuid>, String), // server IDs and file path
    
    /// Restart multiple servers
    Restart(Vec<Uuid>),
    
    /// Apply template to multiple servers
    ApplyTemplate(Vec<Uuid>, String), // server IDs and template ID
}

/// Configuration update for batch operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigUpdate {
    pub auto_start: Option<bool>,
    pub directory: Option<String>,
    pub metadata: Option<HashMap<String, String>>,
}

/// Batch operation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchOperationResult {
    pub operation_id: Uuid,
    pub operation: BatchOperation,
    pub total_servers: usize,
    pub successful: Vec<Uuid>,
    pub failed: Vec<(Uuid, String)>, // server ID and error message
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub status: BatchOperationStatus,
}

/// Batch operation status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BatchOperationStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

/// Batch operation progress
#[derive(Debug, Clone)]
pub struct BatchOperationProgress {
    pub operation_id: Uuid,
    pub completed: usize,
    pub total: usize,
    pub current_server: Option<Uuid>,
    pub message: String,
}

/// Batch operation events
#[derive(Debug, Clone)]
pub enum BatchOperationEvent {
    /// Operation started
    Started(Uuid, BatchOperation),
    
    /// Progress update
    Progress(BatchOperationProgress),
    
    /// Server operation completed
    ServerCompleted(Uuid, Uuid, bool, Option<String>), // operation_id, server_id, success, error
    
    /// Operation completed
    Completed(BatchOperationResult),
    
    /// Operation failed
    Failed(Uuid, String),
    
    /// Operation cancelled
    Cancelled(Uuid),
}

/// Batch operations manager
#[derive(Debug)]
pub struct BatchOperationsManager {
    /// Active operations
    active_operations: HashMap<Uuid, BatchOperationResult>,
    
    /// Operation history
    operation_history: Vec<BatchOperationResult>,
    
    /// Event sender
    event_sender: Option<mpsc::UnboundedSender<BatchOperationEvent>>,
    
    /// Maximum history size
    max_history_size: usize,
}

impl BatchOperationsManager {
    /// Create a new batch operations manager
    pub fn new() -> Self {
        Self {
            active_operations: HashMap::new(),
            operation_history: Vec::new(),
            event_sender: None,
            max_history_size: 100,
        }
    }

    /// Set event sender
    pub fn set_event_sender(&mut self, sender: mpsc::UnboundedSender<BatchOperationEvent>) {
        self.event_sender = Some(sender);
    }

    /// Execute a batch operation
    pub async fn execute_operation(
        &mut self,
        operation: BatchOperation,
        server_manager: &mut McpServerManager,
    ) -> Result<Uuid> {
        let operation_id = Uuid::new_v4();
        
        let server_count = match &operation {
            BatchOperation::Connect(servers) |
            BatchOperation::Disconnect(servers) |
            BatchOperation::Enable(servers) |
            BatchOperation::Disable(servers) |
            BatchOperation::Test(servers) |
            BatchOperation::UpdateConfig(servers, _) |
            BatchOperation::Delete(servers) |
            BatchOperation::Export(servers, _) |
            BatchOperation::Restart(servers) |
            BatchOperation::ApplyTemplate(servers, _) => servers.len(),
            BatchOperation::Import(_) => 0, // Unknown until we read the file
        };

        let result = BatchOperationResult {
            operation_id,
            operation: operation.clone(),
            total_servers: server_count,
            successful: Vec::new(),
            failed: Vec::new(),
            started_at: chrono::Utc::now(),
            completed_at: None,
            status: BatchOperationStatus::Running,
        };

        self.active_operations.insert(operation_id, result.clone());
        self.send_event(BatchOperationEvent::Started(operation_id, operation.clone()));

        // Execute the operation
        match operation {
            BatchOperation::Connect(server_ids) => {
                self.execute_connect_operation(operation_id, server_ids, server_manager).await?;
            }
            BatchOperation::Disconnect(server_ids) => {
                self.execute_disconnect_operation(operation_id, server_ids, server_manager).await?;
            }
            BatchOperation::Enable(server_ids) => {
                self.execute_enable_operation(operation_id, server_ids, server_manager).await?;
            }
            BatchOperation::Disable(server_ids) => {
                self.execute_disable_operation(operation_id, server_ids, server_manager).await?;
            }
            BatchOperation::Test(server_ids) => {
                self.execute_test_operation(operation_id, server_ids, server_manager).await?;
            }
            BatchOperation::UpdateConfig(server_ids, config_update) => {
                self.execute_update_config_operation(operation_id, server_ids, config_update, server_manager).await?;
            }
            BatchOperation::Delete(server_ids) => {
                self.execute_delete_operation(operation_id, server_ids, server_manager).await?;
            }
            BatchOperation::Import(file_path) => {
                self.execute_import_operation(operation_id, file_path, server_manager).await?;
            }
            BatchOperation::Export(server_ids, file_path) => {
                self.execute_export_operation(operation_id, server_ids, file_path, server_manager).await?;
            }
            BatchOperation::Restart(server_ids) => {
                self.execute_restart_operation(operation_id, server_ids, server_manager).await?;
            }
            BatchOperation::ApplyTemplate(server_ids, template_id) => {
                self.execute_apply_template_operation(operation_id, server_ids, template_id, server_manager).await?;
            }
        }

        Ok(operation_id)
    }

    /// Execute connect operation
    async fn execute_connect_operation(
        &mut self,
        operation_id: Uuid,
        server_ids: Vec<Uuid>,
        server_manager: &mut McpServerManager,
    ) -> Result<()> {
        for (index, server_id) in server_ids.iter().enumerate() {
            self.send_progress(operation_id, index, server_ids.len(), Some(*server_id), "Connecting...".to_string());

            match server_manager.connect_server(*server_id).await {
                Ok(_) => {
                    self.record_success(operation_id, *server_id);
                    self.send_event(BatchOperationEvent::ServerCompleted(operation_id, *server_id, true, None));
                }
                Err(e) => {
                    let error_msg = e.to_string();
                    self.record_failure(operation_id, *server_id, error_msg.clone());
                    self.send_event(BatchOperationEvent::ServerCompleted(operation_id, *server_id, false, Some(error_msg)));
                }
            }
        }

        self.complete_operation(operation_id);
        Ok(())
    }

    /// Execute disconnect operation
    async fn execute_disconnect_operation(
        &mut self,
        operation_id: Uuid,
        server_ids: Vec<Uuid>,
        server_manager: &mut McpServerManager,
    ) -> Result<()> {
        for (index, server_id) in server_ids.iter().enumerate() {
            self.send_progress(operation_id, index, server_ids.len(), Some(*server_id), "Disconnecting...".to_string());

            match server_manager.disconnect_server(*server_id).await {
                Ok(_) => {
                    self.record_success(operation_id, *server_id);
                    self.send_event(BatchOperationEvent::ServerCompleted(operation_id, *server_id, true, None));
                }
                Err(e) => {
                    let error_msg = e.to_string();
                    self.record_failure(operation_id, *server_id, error_msg.clone());
                    self.send_event(BatchOperationEvent::ServerCompleted(operation_id, *server_id, false, Some(error_msg)));
                }
            }
        }

        self.complete_operation(operation_id);
        Ok(())
    }

    /// Execute enable operation
    async fn execute_enable_operation(
        &mut self,
        operation_id: Uuid,
        server_ids: Vec<Uuid>,
        _server_manager: &mut McpServerManager,
    ) -> Result<()> {
        for (index, server_id) in server_ids.iter().enumerate() {
            self.send_progress(operation_id, index, server_ids.len(), Some(*server_id), "Enabling...".to_string());

            // TODO: Implement enable functionality in server manager
            self.record_success(operation_id, *server_id);
            self.send_event(BatchOperationEvent::ServerCompleted(operation_id, *server_id, true, None));
        }

        self.complete_operation(operation_id);
        Ok(())
    }

    /// Execute disable operation
    async fn execute_disable_operation(
        &mut self,
        operation_id: Uuid,
        server_ids: Vec<Uuid>,
        _server_manager: &mut McpServerManager,
    ) -> Result<()> {
        for (index, server_id) in server_ids.iter().enumerate() {
            self.send_progress(operation_id, index, server_ids.len(), Some(*server_id), "Disabling...".to_string());

            // TODO: Implement disable functionality in server manager
            self.record_success(operation_id, *server_id);
            self.send_event(BatchOperationEvent::ServerCompleted(operation_id, *server_id, true, None));
        }

        self.complete_operation(operation_id);
        Ok(())
    }

    /// Execute test operation
    async fn execute_test_operation(
        &mut self,
        operation_id: Uuid,
        server_ids: Vec<Uuid>,
        server_manager: &mut McpServerManager,
    ) -> Result<()> {
        for (index, server_id) in server_ids.iter().enumerate() {
            self.send_progress(operation_id, index, server_ids.len(), Some(*server_id), "Testing...".to_string());

            match server_manager.test_server(*server_id).await {
                Ok(success) => {
                    if success {
                        self.record_success(operation_id, *server_id);
                        self.send_event(BatchOperationEvent::ServerCompleted(operation_id, *server_id, true, None));
                    } else {
                        let error_msg = "Test failed".to_string();
                        self.record_failure(operation_id, *server_id, error_msg.clone());
                        self.send_event(BatchOperationEvent::ServerCompleted(operation_id, *server_id, false, Some(error_msg)));
                    }
                }
                Err(e) => {
                    let error_msg = e.to_string();
                    self.record_failure(operation_id, *server_id, error_msg.clone());
                    self.send_event(BatchOperationEvent::ServerCompleted(operation_id, *server_id, false, Some(error_msg)));
                }
            }
        }

        self.complete_operation(operation_id);
        Ok(())
    }

    /// Execute update config operation
    async fn execute_update_config_operation(
        &mut self,
        operation_id: Uuid,
        server_ids: Vec<Uuid>,
        _config_update: ConfigUpdate,
        _server_manager: &mut McpServerManager,
    ) -> Result<()> {
        for (index, server_id) in server_ids.iter().enumerate() {
            self.send_progress(operation_id, index, server_ids.len(), Some(*server_id), "Updating config...".to_string());

            // TODO: Implement config update functionality
            self.record_success(operation_id, *server_id);
            self.send_event(BatchOperationEvent::ServerCompleted(operation_id, *server_id, true, None));
        }

        self.complete_operation(operation_id);
        Ok(())
    }

    /// Execute delete operation
    async fn execute_delete_operation(
        &mut self,
        operation_id: Uuid,
        server_ids: Vec<Uuid>,
        server_manager: &mut McpServerManager,
    ) -> Result<()> {
        for (index, server_id) in server_ids.iter().enumerate() {
            self.send_progress(operation_id, index, server_ids.len(), Some(*server_id), "Deleting...".to_string());

            match server_manager.remove_server(*server_id).await {
                Ok(_) => {
                    self.record_success(operation_id, *server_id);
                    self.send_event(BatchOperationEvent::ServerCompleted(operation_id, *server_id, true, None));
                }
                Err(e) => {
                    let error_msg = e.to_string();
                    self.record_failure(operation_id, *server_id, error_msg.clone());
                    self.send_event(BatchOperationEvent::ServerCompleted(operation_id, *server_id, false, Some(error_msg)));
                }
            }
        }

        self.complete_operation(operation_id);
        Ok(())
    }

    /// Execute import operation
    async fn execute_import_operation(
        &mut self,
        operation_id: Uuid,
        file_path: String,
        server_manager: &mut McpServerManager,
    ) -> Result<()> {
        self.send_progress(operation_id, 0, 1, None, "Importing servers...".to_string());

        match server_manager.import_server_config(file_path.into()).await {
            Ok(server_ids) => {
                for server_id in server_ids {
                    self.record_success(operation_id, server_id);
                }
                self.send_event(BatchOperationEvent::ServerCompleted(operation_id, Uuid::new_v4(), true, None));
            }
            Err(e) => {
                let error_msg = e.to_string();
                self.record_failure(operation_id, Uuid::new_v4(), error_msg.clone());
                self.send_event(BatchOperationEvent::ServerCompleted(operation_id, Uuid::new_v4(), false, Some(error_msg)));
            }
        }

        self.complete_operation(operation_id);
        Ok(())
    }

    /// Execute export operation
    async fn execute_export_operation(
        &mut self,
        operation_id: Uuid,
        server_ids: Vec<Uuid>,
        file_path: String,
        server_manager: &mut McpServerManager,
    ) -> Result<()> {
        self.send_progress(operation_id, 0, 1, None, "Exporting servers...".to_string());

        // TODO: Implement selective export based on server IDs
        match server_manager.export_server_configs(file_path.into(), None).await {
            Ok(_) => {
                for server_id in server_ids {
                    self.record_success(operation_id, server_id);
                }
                self.send_event(BatchOperationEvent::ServerCompleted(operation_id, Uuid::new_v4(), true, None));
            }
            Err(e) => {
                let error_msg = e.to_string();
                self.record_failure(operation_id, Uuid::new_v4(), error_msg.clone());
                self.send_event(BatchOperationEvent::ServerCompleted(operation_id, Uuid::new_v4(), false, Some(error_msg)));
            }
        }

        self.complete_operation(operation_id);
        Ok(())
    }

    /// Execute restart operation
    async fn execute_restart_operation(
        &mut self,
        operation_id: Uuid,
        server_ids: Vec<Uuid>,
        server_manager: &mut McpServerManager,
    ) -> Result<()> {
        for (index, server_id) in server_ids.iter().enumerate() {
            self.send_progress(operation_id, index, server_ids.len(), Some(*server_id), "Restarting...".to_string());

            // Disconnect then reconnect
            let disconnect_result = server_manager.disconnect_server(*server_id).await;
            let connect_result = server_manager.connect_server(*server_id).await;

            if disconnect_result.is_ok() && connect_result.is_ok() {
                self.record_success(operation_id, *server_id);
                self.send_event(BatchOperationEvent::ServerCompleted(operation_id, *server_id, true, None));
            } else {
                let error_msg = "Restart failed".to_string();
                self.record_failure(operation_id, *server_id, error_msg.clone());
                self.send_event(BatchOperationEvent::ServerCompleted(operation_id, *server_id, false, Some(error_msg)));
            }
        }

        self.complete_operation(operation_id);
        Ok(())
    }

    /// Execute apply template operation
    async fn execute_apply_template_operation(
        &mut self,
        operation_id: Uuid,
        server_ids: Vec<Uuid>,
        _template_id: String,
        _server_manager: &mut McpServerManager,
    ) -> Result<()> {
        for (index, server_id) in server_ids.iter().enumerate() {
            self.send_progress(operation_id, index, server_ids.len(), Some(*server_id), "Applying template...".to_string());

            // TODO: Implement template application
            self.record_success(operation_id, *server_id);
            self.send_event(BatchOperationEvent::ServerCompleted(operation_id, *server_id, true, None));
        }

        self.complete_operation(operation_id);
        Ok(())
    }

    /// Record successful operation for a server
    fn record_success(&mut self, operation_id: Uuid, server_id: Uuid) {
        if let Some(result) = self.active_operations.get_mut(&operation_id) {
            result.successful.push(server_id);
        }
    }

    /// Record failed operation for a server
    fn record_failure(&mut self, operation_id: Uuid, server_id: Uuid, error: String) {
        if let Some(result) = self.active_operations.get_mut(&operation_id) {
            result.failed.push((server_id, error));
        }
    }

    /// Send progress update
    fn send_progress(&self, operation_id: Uuid, completed: usize, total: usize, current_server: Option<Uuid>, message: String) {
        let progress = BatchOperationProgress {
            operation_id,
            completed,
            total,
            current_server,
            message,
        };
        self.send_event(BatchOperationEvent::Progress(progress));
    }

    /// Complete an operation
    fn complete_operation(&mut self, operation_id: Uuid) {
        if let Some(mut result) = self.active_operations.remove(&operation_id) {
            result.completed_at = Some(chrono::Utc::now());
            result.status = if result.failed.is_empty() {
                BatchOperationStatus::Completed
            } else {
                BatchOperationStatus::Failed
            };

            self.send_event(BatchOperationEvent::Completed(result.clone()));
            
            // Add to history
            self.operation_history.push(result);
            
            // Limit history size
            while self.operation_history.len() > self.max_history_size {
                self.operation_history.remove(0);
            }
        }
    }

    /// Cancel an operation
    pub fn cancel_operation(&mut self, operation_id: Uuid) -> Result<()> {
        if let Some(mut result) = self.active_operations.remove(&operation_id) {
            result.status = BatchOperationStatus::Cancelled;
            result.completed_at = Some(chrono::Utc::now());
            
            self.send_event(BatchOperationEvent::Cancelled(operation_id));
            self.operation_history.push(result);
        }
        
        Ok(())
    }

    /// Get active operations
    pub fn get_active_operations(&self) -> Vec<&BatchOperationResult> {
        self.active_operations.values().collect()
    }

    /// Get operation history
    pub fn get_operation_history(&self) -> &[BatchOperationResult] {
        &self.operation_history
    }

    /// Send event
    fn send_event(&self, event: BatchOperationEvent) {
        if let Some(sender) = &self.event_sender {
            let _ = sender.send(event);
        }
    }
}

impl Default for BatchOperationsManager {
    fn default() -> Self {
        Self::new()
    }
}
