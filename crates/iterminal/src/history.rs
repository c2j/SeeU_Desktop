use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use chrono::{DateTime, Utc};

/// Command history entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    /// Command that was executed
    pub command: String,
    /// Timestamp when command was executed
    pub timestamp: DateTime<Utc>,
    /// Exit code of the command
    pub exit_code: Option<i32>,
    /// Working directory when command was executed
    pub working_directory: String,
}

/// Command history manager
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandHistory {
    /// History entries
    entries: VecDeque<HistoryEntry>,
    /// Maximum number of entries to keep
    max_entries: usize,
    /// Current position in history (for navigation)
    current_position: Option<usize>,
}

impl Default for CommandHistory {
    fn default() -> Self {
        Self {
            entries: VecDeque::new(),
            max_entries: 1000,
            current_position: None,
        }
    }
}

impl CommandHistory {
    /// Create a new command history
    pub fn new(max_entries: usize) -> Self {
        Self {
            entries: VecDeque::new(),
            max_entries,
            current_position: None,
        }
    }

    /// Add a command to history
    pub fn add_command(&mut self, command: String, working_directory: String) {
        // Don't add empty commands or duplicates of the last command
        if command.trim().is_empty() {
            return;
        }

        if let Some(last_entry) = self.entries.back() {
            if last_entry.command == command {
                return;
            }
        }

        let entry = HistoryEntry {
            command,
            timestamp: Utc::now(),
            exit_code: None,
            working_directory,
        };

        self.entries.push_back(entry);

        // Remove old entries if we exceed the limit
        while self.entries.len() > self.max_entries {
            self.entries.pop_front();
        }

        // Reset position
        self.current_position = None;
    }

    /// Update the exit code of the last command
    pub fn update_last_exit_code(&mut self, exit_code: i32) {
        if let Some(last_entry) = self.entries.back_mut() {
            last_entry.exit_code = Some(exit_code);
        }
    }

    /// Get previous command in history
    pub fn get_previous(&mut self) -> Option<String> {
        if self.entries.is_empty() {
            return None;
        }

        let new_position = match self.current_position {
            None => self.entries.len() - 1,
            Some(pos) => {
                if pos > 0 {
                    pos - 1
                } else {
                    return None;
                }
            }
        };

        self.current_position = Some(new_position);
        self.entries.get(new_position).map(|entry| entry.command.clone())
    }

    /// Get next command in history
    pub fn get_next(&mut self) -> Option<String> {
        match self.current_position {
            None => None,
            Some(pos) => {
                if pos < self.entries.len() - 1 {
                    let new_position = pos + 1;
                    self.current_position = Some(new_position);
                    self.entries.get(new_position).map(|entry| entry.command.clone())
                } else {
                    self.current_position = None;
                    Some(String::new()) // Return empty string when at the end
                }
            }
        }
    }

    /// Reset history position
    pub fn reset_position(&mut self) {
        self.current_position = None;
    }

    /// Get all history entries
    pub fn get_entries(&self) -> &VecDeque<HistoryEntry> {
        &self.entries
    }

    /// Search history for commands containing the given text
    pub fn search(&self, query: &str) -> Vec<&HistoryEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.command.contains(query))
            .collect()
    }

    /// Clear all history
    pub fn clear(&mut self) {
        self.entries.clear();
        self.current_position = None;
    }

    /// Get the number of entries
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if history is empty
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Alias for get_previous for compatibility
    pub fn previous(&mut self) -> Option<String> {
        self.get_previous()
    }

    /// Alias for get_next for compatibility
    pub fn next(&mut self) -> Option<String> {
        self.get_next()
    }
}
