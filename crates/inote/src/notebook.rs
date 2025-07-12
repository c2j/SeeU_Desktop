use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// Notebook structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notebook {
    pub id: String,
    pub name: String,
    pub description: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub note_ids: Vec<String>,
    #[serde(default)]
    pub expanded: bool,
    #[serde(default)]
    pub sort_order: i32,
}

impl Notebook {
    /// Create a new notebook
    pub fn new(name: String, description: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            description,
            created_at: now,
            updated_at: now,
            note_ids: Vec::new(),
            expanded: false, // 默认折叠，节省加载时间
            sort_order: 0, // 默认排序值，创建时会被设置为合适的值
        }
    }

    /// 切换展开/折叠状态
    pub fn toggle_expanded(&mut self) {
        self.expanded = !self.expanded;
    }

    /// Add a note to the notebook (inserts at the beginning)
    pub fn add_note(&mut self, note_id: String) {
        self.note_ids.insert(0, note_id);
        self.updated_at = Utc::now();
    }

    /// Add a note to the end of the notebook (for backward compatibility)
    pub fn append_note(&mut self, note_id: String) {
        self.note_ids.push(note_id);
        self.updated_at = Utc::now();
    }

    /// Remove a note from the notebook
    pub fn remove_note(&mut self, note_id: &str) {
        self.note_ids.retain(|id| id != note_id);
        self.updated_at = Utc::now();
    }
}