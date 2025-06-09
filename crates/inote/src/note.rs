use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// Note structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Note {
    pub id: String,
    pub title: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub tag_ids: Vec<String>,
    pub attachments: Vec<Attachment>,
}

/// Attachment structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attachment {
    pub id: String,
    pub name: String,
    pub file_path: String,
    pub file_type: String,
    pub created_at: DateTime<Utc>,
}

impl Note {
    /// Create a new note
    pub fn new(title: String, content: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            title,
            content,
            created_at: now,
            updated_at: now,
            tag_ids: Vec::new(),
            attachments: Vec::new(),
        }
    }

    /// Add a tag to the note
    pub fn add_tag(&mut self, tag_id: String) {
        if !self.tag_ids.contains(&tag_id) {
            self.tag_ids.push(tag_id);
            self.updated_at = Utc::now();
        }
    }

    /// Remove a tag from the note
    pub fn remove_tag(&mut self, tag_id: &str) {
        self.tag_ids.retain(|id| id != tag_id);
        self.updated_at = Utc::now();
    }

    /// Add an attachment to the note
    pub fn add_attachment(&mut self, name: String, file_path: String, file_type: String) {
        let attachment = Attachment {
            id: Uuid::new_v4().to_string(),
            name,
            file_path,
            file_type,
            created_at: Utc::now(),
        };

        self.attachments.push(attachment);
        self.updated_at = Utc::now();
    }

    /// Remove an attachment from the note
    pub fn remove_attachment(&mut self, attachment_id: &str) {
        self.attachments.retain(|attachment| attachment.id != attachment_id);
        self.updated_at = Utc::now();
    }
}