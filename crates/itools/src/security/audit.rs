use std::collections::VecDeque;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::roles::UserRole;

/// Audit logger for security events
#[derive(Debug)]
pub struct AuditLogger {
    /// In-memory audit log (for recent events)
    memory_log: VecDeque<AuditEvent>,
    
    /// Maximum events to keep in memory
    max_memory_events: usize,
    
    /// File path for persistent audit log
    log_file_path: Option<std::path::PathBuf>,
}

/// Audit event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub event_type: AuditEventType,
    pub user_role: UserRole,
    pub plugin_id: Option<Uuid>,
    pub resource: Option<String>,
    pub action: String,
    pub result: AuditResult,
    pub details: AuditDetails,
    pub risk_level: RiskLevel,
}

/// Types of audit events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditEventType {
    PermissionCheck,
    PluginInstall,
    PluginUninstall,
    PluginEnable,
    PluginDisable,
    ToolCall,
    ResourceAccess,
    ConfigurationChange,
    SecurityViolation,
    SystemAccess,
}

/// Audit result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditResult {
    Success,
    Failure { reason: String },
    Denied { reason: String },
    RequiredConfirmation,
    Cancelled,
}

/// Additional audit details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditDetails {
    pub session_id: Uuid,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub request_id: Option<String>,
    pub duration_ms: Option<u64>,
    pub data_size: Option<u64>,
    pub additional_fields: std::collections::HashMap<String, String>,
}

/// Risk level for audit events
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

/// Audit query for filtering events
#[derive(Debug, Clone)]
pub struct AuditQuery {
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub event_types: Option<Vec<AuditEventType>>,
    pub user_roles: Option<Vec<UserRole>>,
    pub plugin_ids: Option<Vec<Uuid>>,
    pub risk_levels: Option<Vec<RiskLevel>>,
    pub results: Option<Vec<AuditResult>>,
    pub limit: Option<usize>,
}

impl AuditLogger {
    /// Create a new audit logger
    pub fn new() -> Self {
        Self {
            memory_log: VecDeque::new(),
            max_memory_events: 1000,
            log_file_path: None,
        }
    }
    
    /// Create audit logger with file persistence
    pub fn with_file_logging(log_file_path: std::path::PathBuf) -> Self {
        Self {
            memory_log: VecDeque::new(),
            max_memory_events: 1000,
            log_file_path: Some(log_file_path),
        }
    }
    
    /// Initialize the audit logger
    pub fn initialize(&mut self) {
        log::info!("Initializing audit logger");
        
        // Create log directory if needed
        if let Some(log_path) = &self.log_file_path {
            if let Some(parent) = log_path.parent() {
                if let Err(e) = std::fs::create_dir_all(parent) {
                    log::error!("Failed to create audit log directory: {}", e);
                }
            }
        }
        
        // Load recent events from file if available
        self.load_recent_events();
    }
    
    /// Log an audit event
    pub fn log_event(&mut self, event: AuditEvent) {
        // Only log high-risk events to reduce noise
        if matches!(event.risk_level, RiskLevel::High | RiskLevel::Critical) {
            log::warn!("High-risk audit event: {:?} - {} - {:?}",
                      event.event_type, event.action, event.result);
        }
        
        // Add to memory log
        self.memory_log.push_back(event.clone());
        
        // Maintain memory limit
        while self.memory_log.len() > self.max_memory_events {
            self.memory_log.pop_front();
        }
        
        // Write to file if configured
        if self.log_file_path.is_some() {
            self.write_event_to_file(&event);
        }
        
        // Check for security violations
        self.check_security_violations(&event);
    }
    
    /// Log a permission check event
    pub fn log_permission_check(
        &mut self,
        user_role: UserRole,
        plugin_id: Option<Uuid>,
        action: String,
        resource: Option<String>,
        result: AuditResult,
        details: AuditDetails,
    ) {
        let event = AuditEvent {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            event_type: AuditEventType::PermissionCheck,
            user_role,
            plugin_id,
            resource,
            action,
            result: result.clone(),
            details,
            risk_level: self.calculate_risk_level(&AuditEventType::PermissionCheck, &result),
        };
        
        self.log_event(event);
    }
    
    /// Log a plugin operation event
    pub fn log_plugin_operation(
        &mut self,
        event_type: AuditEventType,
        user_role: UserRole,
        plugin_id: Uuid,
        action: String,
        result: AuditResult,
        details: AuditDetails,
    ) {
        let event = AuditEvent {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            event_type: event_type.clone(),
            user_role,
            plugin_id: Some(plugin_id),
            resource: None,
            action,
            result: result.clone(),
            details,
            risk_level: self.calculate_risk_level(&event_type, &result),
        };
        
        self.log_event(event);
    }
    
    /// Log a security violation
    pub fn log_security_violation(
        &mut self,
        user_role: UserRole,
        action: String,
        reason: String,
        details: AuditDetails,
    ) {
        let event = AuditEvent {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            event_type: AuditEventType::SecurityViolation,
            user_role,
            plugin_id: None,
            resource: None,
            action,
            result: AuditResult::Denied { reason },
            details,
            risk_level: RiskLevel::High,
        };
        
        self.log_event(event);
    }
    
    /// Query audit events
    pub fn query_events(&self, query: &AuditQuery) -> Vec<&AuditEvent> {
        let mut results: Vec<&AuditEvent> = self.memory_log
            .iter()
            .filter(|event| self.matches_query(event, query))
            .collect();
        
        // Sort by timestamp (newest first)
        results.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        
        // Apply limit
        if let Some(limit) = query.limit {
            results.truncate(limit);
        }
        
        results
    }
    
    /// Get recent high-risk events
    pub fn get_high_risk_events(&self, limit: usize) -> Vec<&AuditEvent> {
        let mut events: Vec<&AuditEvent> = self.memory_log
            .iter()
            .filter(|event| matches!(event.risk_level, RiskLevel::High | RiskLevel::Critical))
            .collect();
        
        events.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        events.truncate(limit);
        
        events
    }
    
    /// Get event statistics
    pub fn get_statistics(&self) -> AuditStatistics {
        let total_events = self.memory_log.len();
        let mut by_type = std::collections::HashMap::new();
        let mut by_risk = std::collections::HashMap::new();
        let mut by_result = std::collections::HashMap::new();
        
        for event in &self.memory_log {
            *by_type.entry(format!("{:?}", event.event_type)).or_insert(0) += 1;
            *by_risk.entry(format!("{:?}", event.risk_level)).or_insert(0) += 1;
            *by_result.entry(format!("{:?}", event.result)).or_insert(0) += 1;
        }
        
        AuditStatistics {
            total_events,
            events_by_type: by_type,
            events_by_risk_level: by_risk,
            events_by_result: by_result,
        }
    }
    
    /// Calculate risk level for an event
    fn calculate_risk_level(&self, event_type: &AuditEventType, result: &AuditResult) -> RiskLevel {
        match (event_type, result) {
            (AuditEventType::SecurityViolation, _) => RiskLevel::Critical,
            (AuditEventType::PluginInstall, AuditResult::Success) => RiskLevel::Medium,
            (AuditEventType::PluginUninstall, AuditResult::Success) => RiskLevel::Medium,
            (AuditEventType::ConfigurationChange, _) => RiskLevel::Medium,
            (AuditEventType::SystemAccess, AuditResult::Failure { .. }) => RiskLevel::High,
            (_, AuditResult::Denied { .. }) => RiskLevel::Medium,
            (_, AuditResult::Failure { .. }) => RiskLevel::Low,
            _ => RiskLevel::Low,
        }
    }
    
    /// Check if event matches query criteria
    fn matches_query(&self, event: &AuditEvent, query: &AuditQuery) -> bool {
        if let Some(start) = query.start_time {
            if event.timestamp < start {
                return false;
            }
        }
        
        if let Some(end) = query.end_time {
            if event.timestamp > end {
                return false;
            }
        }
        
        if let Some(types) = &query.event_types {
            if !types.iter().any(|t| std::mem::discriminant(t) == std::mem::discriminant(&event.event_type)) {
                return false;
            }
        }
        
        if let Some(roles) = &query.user_roles {
            if !roles.contains(&event.user_role) {
                return false;
            }
        }
        
        if let Some(plugin_ids) = &query.plugin_ids {
            if let Some(plugin_id) = event.plugin_id {
                if !plugin_ids.contains(&plugin_id) {
                    return false;
                }
            } else {
                return false;
            }
        }
        
        if let Some(risk_levels) = &query.risk_levels {
            if !risk_levels.contains(&event.risk_level) {
                return false;
            }
        }
        
        true
    }
    
    /// Write event to file
    fn write_event_to_file(&self, event: &AuditEvent) {
        if let Some(log_path) = &self.log_file_path {
            if let Ok(json) = serde_json::to_string(event) {
                if let Err(e) = std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(log_path)
                    .and_then(|mut file| {
                        use std::io::Write;
                        writeln!(file, "{}", json)
                    })
                {
                    log::error!("Failed to write audit event to file: {}", e);
                }
            }
        }
    }
    
    /// Load recent events from file
    fn load_recent_events(&mut self) {
        // TODO: Implement loading recent events from audit log file
        // Removed debug log to reduce noise
    }
    
    /// Check for security violations in the event
    fn check_security_violations(&self, event: &AuditEvent) {
        match &event.event_type {
            AuditEventType::SecurityViolation => {
                log::warn!("Security violation detected: {} by {:?}", 
                          event.action, event.user_role);
            }
            _ if event.risk_level == RiskLevel::Critical => {
                log::warn!("Critical risk event detected: {:?}", event);
            }
            _ => {}
        }
    }
}

/// Audit statistics
#[derive(Debug)]
pub struct AuditStatistics {
    pub total_events: usize,
    pub events_by_type: std::collections::HashMap<String, usize>,
    pub events_by_risk_level: std::collections::HashMap<String, usize>,
    pub events_by_result: std::collections::HashMap<String, usize>,
}

impl Default for AuditLogger {
    fn default() -> Self {
        Self::new()
    }
}
