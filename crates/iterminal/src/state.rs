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
    /// Last update time for cursor blinking
    pub last_cursor_blink: std::time::Instant,
    /// Whether cursor is visible (for blinking)
    pub cursor_visible: bool,
    /// Font size scale factor
    pub font_scale: f32,
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
            last_cursor_blink: std::time::Instant::now(),
            cursor_visible: true,
            font_scale: 1.0,
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

    /// Handle keyboard input
    pub fn handle_key_input(&mut self, key: egui::Key, modifiers: egui::Modifiers) -> bool {
        // Get session ID first to avoid borrowing conflicts
        let session_id = if let Some(session) = self.terminal_manager.get_active_session() {
            session.id
        } else {
            return false;
        };

        if let Some(session) = self.terminal_manager.get_active_session_mut() {
            match key {
                egui::Key::Enter => {
                    let command = session.execute_current_input();
                    if !command.trim().is_empty() {
                        // Drop the mutable borrow before calling execute_command_in_session
                        let _ = session;
                        self.terminal_manager.execute_command_in_session(session_id, command);
                    }
                    true
                }
                egui::Key::Backspace => {
                    session.delete_char_before_cursor();
                    true
                }
                egui::Key::Delete => {
                    session.delete_char_at_cursor();
                    true
                }
                egui::Key::ArrowLeft => {
                    session.move_cursor_left();
                    true
                }
                egui::Key::ArrowRight => {
                    session.move_cursor_right();
                    true
                }
                egui::Key::ArrowUp => {
                    if let Some(prev_command) = session.history.get_previous() {
                        session.set_input(prev_command);
                    }
                    true
                }
                egui::Key::ArrowDown => {
                    if let Some(next_command) = session.history.get_next() {
                        session.set_input(next_command);
                    }
                    true
                }
                egui::Key::Home => {
                    if modifiers.ctrl {
                        session.scroll_to_top();
                    } else {
                        session.move_cursor_to_start();
                    }
                    true
                }
                egui::Key::End => {
                    if modifiers.ctrl {
                        session.scroll_to_bottom();
                    } else {
                        session.move_cursor_to_end();
                    }
                    true
                }
                egui::Key::PageUp => {
                    session.scroll_up(10);
                    true
                }
                egui::Key::PageDown => {
                    session.scroll_down(10);
                    true
                }
                _ => false,
            }
        } else {
            false
        }
    }

    /// Handle character input
    pub fn handle_char_input(&mut self, ch: char) -> bool {
        if let Some(session) = self.terminal_manager.get_active_session_mut() {
            // Handle special characters
            match ch {
                '\u{08}' => {
                    // Backspace
                    session.delete_char_before_cursor();
                }
                '\u{7f}' => {
                    // Delete
                    session.delete_char_at_cursor();
                }
                '\r' | '\n' => {
                    // Enter - handled in key input
                    return false;
                }
                _ => {
                    // Regular character
                    if ch.is_control() {
                        return false;
                    }
                    session.insert_char(ch);
                }
            }
            true
        } else {
            false
        }
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
