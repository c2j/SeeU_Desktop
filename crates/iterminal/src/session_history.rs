use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// Represents a saved terminal session that can be restored
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedSession {
    /// Unique identifier for the session
    pub id: Uuid,
    /// Display title of the session
    pub title: String,
    /// When the session was created
    pub created_at: DateTime<Utc>,
    /// When the session was last active
    pub last_activity: DateTime<Utc>,
    /// When the session was saved
    pub saved_at: DateTime<Utc>,
    /// Working directory when session was saved
    pub working_directory: Option<PathBuf>,
    /// Terminal output content
    pub content: String,
    /// Command history
    pub command_history: Vec<String>,
    /// Environment variables (filtered for security)
    pub environment: HashMap<String, String>,
    /// Session tags for organization
    pub tags: Vec<String>,
    /// User notes about the session
    pub notes: String,
}

impl SavedSession {
    /// Create a new saved session from current session data
    pub fn new(
        id: Uuid,
        title: String,
        created_at: DateTime<Utc>,
        last_activity: DateTime<Utc>,
        content: String,
    ) -> Self {
        Self {
            id,
            title,
            created_at,
            last_activity,
            saved_at: Utc::now(),
            working_directory: std::env::current_dir().ok(),
            content,
            command_history: Vec::new(),
            environment: Self::get_safe_environment(),
            tags: Vec::new(),
            notes: String::new(),
        }
    }

    /// Get safe environment variables (excluding sensitive ones)
    fn get_safe_environment() -> HashMap<String, String> {
        let mut env = HashMap::new();
        let safe_vars = [
            "PATH", "HOME", "USER", "SHELL", "TERM", "LANG", "LC_ALL",
            "PWD", "OLDPWD", "EDITOR", "VISUAL", "PAGER"
        ];
        
        for var in &safe_vars {
            if let Ok(value) = std::env::var(var) {
                env.insert(var.to_string(), value);
            }
        }
        
        env
    }

    /// Add a tag to the session
    pub fn add_tag(&mut self, tag: String) {
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
        }
    }

    /// Remove a tag from the session
    pub fn remove_tag(&mut self, tag: &str) {
        self.tags.retain(|t| t != tag);
    }

    /// Set user notes for the session
    pub fn set_notes(&mut self, notes: String) {
        self.notes = notes;
    }

    /// Get a short description of the session
    pub fn get_description(&self) -> String {
        if !self.notes.is_empty() {
            self.notes.clone()
        } else if !self.command_history.is_empty() {
            format!("Last command: {}", self.command_history.last().unwrap())
        } else {
            format!("Session from {}", self.created_at.format("%Y-%m-%d %H:%M"))
        }
    }
}

/// Manages session history storage and retrieval
#[derive(Debug)]
pub struct SessionHistoryManager {
    /// Storage directory for session files
    storage_dir: PathBuf,
    /// In-memory cache of loaded sessions
    sessions: HashMap<Uuid, SavedSession>,
    /// Maximum number of sessions to keep
    max_sessions: usize,
}

impl SessionHistoryManager {
    /// Create a new session history manager
    pub fn new() -> Result<Self, SessionHistoryError> {
        let storage_dir = Self::get_storage_directory()?;
        std::fs::create_dir_all(&storage_dir)
            .map_err(|e| SessionHistoryError::IoError(format!("Failed to create storage directory: {}", e)))?;

        let mut manager = Self {
            storage_dir,
            sessions: HashMap::new(),
            max_sessions: 100, // Default limit
        };

        // Load existing sessions
        manager.load_all_sessions()?;
        
        Ok(manager)
    }

    /// Get the storage directory for session files
    fn get_storage_directory() -> Result<PathBuf, SessionHistoryError> {
        let mut dir = dirs::config_dir()
            .ok_or_else(|| SessionHistoryError::ConfigError("Cannot determine config directory".to_string()))?;
        dir.push("seeu_desktop");
        dir.push("iterminal");
        dir.push("sessions");
        Ok(dir)
    }

    /// Save a session to storage
    pub fn save_session(&mut self, session: SavedSession) -> Result<(), SessionHistoryError> {
        // Add to memory cache
        self.sessions.insert(session.id, session.clone());

        // Save to file
        let file_path = self.storage_dir.join(format!("{}.json", session.id));
        let json = serde_json::to_string_pretty(&session)
            .map_err(|e| SessionHistoryError::SerializationError(e.to_string()))?;
        
        std::fs::write(&file_path, json)
            .map_err(|e| SessionHistoryError::IoError(format!("Failed to write session file: {}", e)))?;

        // Cleanup old sessions if we exceed the limit
        self.cleanup_old_sessions()?;

        log::info!("Saved session '{}' to {}", session.title, file_path.display());
        Ok(())
    }

    /// Load a specific session by ID
    pub fn load_session(&mut self, session_id: Uuid) -> Result<Option<SavedSession>, SessionHistoryError> {
        // Check memory cache first
        if let Some(session) = self.sessions.get(&session_id) {
            return Ok(Some(session.clone()));
        }

        // Try to load from file
        let file_path = self.storage_dir.join(format!("{}.json", session_id));
        if !file_path.exists() {
            return Ok(None);
        }

        let json = std::fs::read_to_string(&file_path)
            .map_err(|e| SessionHistoryError::IoError(format!("Failed to read session file: {}", e)))?;
        
        let session: SavedSession = serde_json::from_str(&json)
            .map_err(|e| SessionHistoryError::SerializationError(e.to_string()))?;

        // Add to cache
        self.sessions.insert(session.id, session.clone());
        
        Ok(Some(session))
    }

    /// Load all sessions from storage
    pub fn load_all_sessions(&mut self) -> Result<(), SessionHistoryError> {
        if !self.storage_dir.exists() {
            return Ok(());
        }

        let entries = std::fs::read_dir(&self.storage_dir)
            .map_err(|e| SessionHistoryError::IoError(format!("Failed to read storage directory: {}", e)))?;

        for entry in entries {
            let entry = entry.map_err(|e| SessionHistoryError::IoError(e.to_string()))?;
            let path = entry.path();
            
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                match self.load_session_from_file(&path) {
                    Ok(session) => {
                        self.sessions.insert(session.id, session);
                    }
                    Err(e) => {
                        log::warn!("Failed to load session from {}: {}", path.display(), e);
                    }
                }
            }
        }

        log::info!("Loaded {} sessions from storage", self.sessions.len());
        Ok(())
    }

    /// Load a session from a specific file path
    fn load_session_from_file(&self, path: &PathBuf) -> Result<SavedSession, SessionHistoryError> {
        let json = std::fs::read_to_string(path)
            .map_err(|e| SessionHistoryError::IoError(format!("Failed to read file: {}", e)))?;
        
        serde_json::from_str(&json)
            .map_err(|e| SessionHistoryError::SerializationError(e.to_string()))
    }

    /// Get all saved sessions, sorted by last activity
    pub fn get_all_sessions(&self) -> Vec<&SavedSession> {
        let mut sessions: Vec<&SavedSession> = self.sessions.values().collect();
        sessions.sort_by(|a, b| b.last_activity.cmp(&a.last_activity));
        sessions
    }

    /// Search sessions by title, tags, or content
    pub fn search_sessions(&self, query: &str) -> Vec<&SavedSession> {
        let query_lower = query.to_lowercase();
        self.sessions
            .values()
            .filter(|session| {
                session.title.to_lowercase().contains(&query_lower)
                    || session.tags.iter().any(|tag| tag.to_lowercase().contains(&query_lower))
                    || session.notes.to_lowercase().contains(&query_lower)
                    || session.content.to_lowercase().contains(&query_lower)
            })
            .collect()
    }

    /// Delete a session
    pub fn delete_session(&mut self, session_id: Uuid) -> Result<bool, SessionHistoryError> {
        // Remove from memory
        let removed = self.sessions.remove(&session_id).is_some();
        
        // Remove file
        let file_path = self.storage_dir.join(format!("{}.json", session_id));
        if file_path.exists() {
            std::fs::remove_file(&file_path)
                .map_err(|e| SessionHistoryError::IoError(format!("Failed to delete session file: {}", e)))?;
        }

        if removed {
            log::info!("Deleted session {}", session_id);
        }
        
        Ok(removed)
    }

    /// Cleanup old sessions to maintain the limit
    fn cleanup_old_sessions(&mut self) -> Result<(), SessionHistoryError> {
        if self.sessions.len() <= self.max_sessions {
            return Ok(());
        }

        // Sort by last activity and remove oldest
        let mut sessions: Vec<_> = self.sessions.values().cloned().collect();
        sessions.sort_by(|a, b| a.last_activity.cmp(&b.last_activity));

        let to_remove = sessions.len() - self.max_sessions;
        for session in sessions.iter().take(to_remove) {
            self.delete_session(session.id)?;
        }

        log::info!("Cleaned up {} old sessions", to_remove);
        Ok(())
    }

    /// Set the maximum number of sessions to keep
    pub fn set_max_sessions(&mut self, max: usize) {
        self.max_sessions = max;
    }

    /// Get session count
    pub fn session_count(&self) -> usize {
        self.sessions.len()
    }

    /// Get a specific session by ID
    pub fn get_session(&self, session_id: &Uuid) -> Option<&SavedSession> {
        self.sessions.get(session_id)
    }

    /// Get mutable access to a specific session by ID
    pub fn get_session_mut(&mut self, session_id: &Uuid) -> Option<&mut SavedSession> {
        self.sessions.get_mut(session_id)
    }
}

/// Errors that can occur during session history operations
#[derive(Debug, thiserror::Error)]
pub enum SessionHistoryError {
    #[error("IO error: {0}")]
    IoError(String),
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    #[error("Configuration error: {0}")]
    ConfigError(String),
}
