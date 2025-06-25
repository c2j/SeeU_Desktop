use crate::terminal::TerminalManager;
use crate::config::TerminalConfig;
use uuid::Uuid;

/// Main state for the iTerminal module
#[derive(Debug)]
pub struct ITerminalState {
    /// Terminal manager
    pub terminal_manager: TerminalManager,
    /// Whether to show command history
    pub show_history: bool,
    /// Search query for history
    pub history_search: String,
    /// Whether the terminal has focus
    pub has_focus: bool,
    /// Whether cursor is visible (for blinking)
    pub cursor_visible: bool,
    /// Font size scale factor
    pub font_scale: f32,
    /// Last update time for cursor blinking
    pub last_cursor_blink: std::time::Instant,
    /// Command input for the interface
    pub command_input: String,
    /// Terminal history for display
    pub terminal_history: Vec<String>,
    /// Current session name
    pub current_session: String,
}

impl ITerminalState {
    /// Create a new terminal state
    pub fn new() -> Self {
        let config = TerminalConfig::load();
        let terminal_manager = TerminalManager::new(config);

        Self {
            terminal_manager,
            show_history: false,
            history_search: String::new(),
            has_focus: false,
            cursor_visible: true,
            font_scale: 1.0,
            last_cursor_blink: std::time::Instant::now(),
            command_input: String::new(),
            terminal_history: Vec::new(),
            current_session: "主会话".to_string(),
        }
    }

    /// Update the terminal state
    pub fn update(&mut self) {
        // Update terminal manager
        self.terminal_manager.update();

        // Handle cursor blinking
        let config = self.terminal_manager.get_config();
        if self.has_focus && config.cursor_blink_interval > 0 {
            let elapsed = self.last_cursor_blink.elapsed();
            if elapsed.as_millis() >= config.cursor_blink_interval as u128 {
                self.cursor_visible = !self.cursor_visible;
                self.last_cursor_blink = std::time::Instant::now();
            }
        } else {
            self.cursor_visible = true;
        }
    }

    /// Create a new terminal session
    pub fn create_session(&mut self, title: Option<String>) -> Uuid {
        let title = title.unwrap_or_else(|| {
            format!("Terminal {}", self.terminal_manager.session_count() + 1)
        });
        self.terminal_manager.create_session(title)
    }

    /// Close a terminal session
    pub fn close_session(&mut self, session_id: Uuid) -> bool {
        self.terminal_manager.close_session(session_id)
    }

    /// Set the active session
    pub fn set_active_session(&mut self, session_id: Uuid) -> bool {
        self.terminal_manager.set_active_session(session_id)
    }

    /// Execute a command in the active session
    pub fn execute_command(&mut self, command: String) -> bool {
        self.terminal_manager.execute_command(command)
    }

    /// Execute command in current session
    pub fn execute_current_input(&mut self) -> bool {
        if let Some(session) = self.terminal_manager.get_active_session_mut() {
            let command = session.execute_current_input();
            if !command.trim().is_empty() {
                let session_id = session.id;
                // Drop the mutable borrow before calling execute_command_in_session
                let _ = session;
                self.terminal_manager.execute_command_in_session(session_id, command);
                return true;
            }
        }
        false
    }

    /// Insert character at cursor position
    pub fn insert_char(&mut self, ch: char) -> bool {
        if let Some(session) = self.terminal_manager.get_active_session_mut() {
            session.insert_char(ch);
            true
        } else {
            false
        }
    }

    /// Delete character before cursor
    pub fn delete_char_before_cursor(&mut self) -> bool {
        if let Some(session) = self.terminal_manager.get_active_session_mut() {
            session.delete_char_before_cursor();
            true
        } else {
            false
        }
    }

    /// Move cursor left
    pub fn move_cursor_left(&mut self) -> bool {
        if let Some(session) = self.terminal_manager.get_active_session_mut() {
            session.move_cursor_left();
            true
        } else {
            false
        }
    }

    /// Move cursor right
    pub fn move_cursor_right(&mut self) -> bool {
        if let Some(session) = self.terminal_manager.get_active_session_mut() {
            session.move_cursor_right();
            true
        } else {
            false
        }
    }

    /// Navigate to previous command in history
    pub fn history_previous(&mut self) -> bool {
        if let Some(session) = self.terminal_manager.get_active_session_mut() {
            if let Some(prev_command) = session.history.get_previous() {
                session.set_input(prev_command);
                return true;
            }
        }
        false
    }

    /// Navigate to next command in history
    pub fn history_next(&mut self) -> bool {
        if let Some(session) = self.terminal_manager.get_active_session_mut() {
            if let Some(next_command) = session.history.get_next() {
                session.set_input(next_command);
                return true;
            }
        }
        false
    }

    /// Toggle history dialog
    pub fn toggle_history(&mut self) {
        self.show_history = !self.show_history;
    }

    /// Save current configuration
    pub fn save_config(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.terminal_manager.get_config().save()
    }

    /// Update configuration
    pub fn update_config(&mut self, config: TerminalConfig) {
        self.terminal_manager.update_config(config);
    }

    /// Get current configuration
    pub fn get_config(&self) -> &TerminalConfig {
        self.terminal_manager.get_config()
    }

    /// Increase font size
    pub fn increase_font_size(&mut self) {
        self.font_scale = (self.font_scale + 0.1).min(3.0);
    }

    /// Decrease font size
    pub fn decrease_font_size(&mut self) {
        self.font_scale = (self.font_scale - 0.1).max(0.5);
    }

    /// Reset font size
    pub fn reset_font_size(&mut self) {
        self.font_scale = 1.0;
    }

    /// Execute command directly
    pub fn execute_command_direct(&mut self, command: String) {
        if let Some(session) = self.terminal_manager.get_active_session_mut() {
            let session_id = session.id;
            session.current_input = command.clone();
            let executed_command = session.execute_current_input();
            if !executed_command.trim().is_empty() {
                let _ = session;
                self.terminal_manager.execute_command_in_session(session_id, executed_command);
            }
        }
    }

    /// Check if there are any running commands
    pub fn has_running_commands(&self) -> bool {
        self.terminal_manager.has_active_commands()
    }

    /// Get the number of sessions
    pub fn session_count(&self) -> usize {
        self.terminal_manager.session_count()
    }
}

impl Default for ITerminalState {
    fn default() -> Self {
        Self::new()
    }
}
