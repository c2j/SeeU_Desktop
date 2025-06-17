use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// MCP Server record for database storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerRecord {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub transport_type: String,
    pub transport_config: String, // JSON string
    pub directory: String,
    pub capabilities: Option<String>, // JSON string
    pub health_status: String,
    pub last_test_time: Option<DateTime<Utc>>,
    pub last_test_success: bool,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl McpServerRecord {
    /// Create a new MCP server record
    pub fn new(
        id: Uuid,
        name: String,
        description: Option<String>,
        transport_type: String,
        transport_config: String,
        directory: String,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: id.to_string(),
            name,
            description,
            transport_type,
            transport_config,
            directory,
            capabilities: None,
            health_status: "Red".to_string(),
            last_test_time: None,
            last_test_success: false,
            enabled: true,
            created_at: now,
            updated_at: now,
        }
    }

    /// Update server status after testing
    pub fn update_test_result(&mut self, success: bool, capabilities: Option<String>) {
        self.last_test_time = Some(Utc::now());
        self.last_test_success = success;
        self.health_status = if success { "Green".to_string() } else { "Yellow".to_string() };
        if let Some(caps) = capabilities {
            self.capabilities = Some(caps);
        }
        self.updated_at = Utc::now();
    }

    /// Mark server as modified (Red status)
    pub fn mark_as_modified(&mut self) {
        self.health_status = "Red".to_string();
        self.updated_at = Utc::now();
    }

    /// Check if server is ready for AI assistant (Green status)
    pub fn is_ready_for_ai(&self) -> bool {
        self.enabled && self.health_status == "Green"
    }

    /// Get UUID from string ID
    pub fn get_uuid(&self) -> Result<Uuid, uuid::Error> {
        Uuid::parse_str(&self.id)
    }
}
