use uuid::Uuid;
use std::collections::VecDeque;
use chrono::{DateTime, Utc};
use crate::history::CommandHistory;
use crate::config::TerminalConfig;

/// Terminal output line
#[derive(Debug, Clone)]
pub struct OutputLine {
    /// The text content
    pub content: String,
    /// Timestamp when the line was added
    pub timestamp: DateTime<Utc>,
    /// Whether this line is from command output or user input
    pub is_input: bool,
    /// ANSI color/style information
    pub style_info: Option<StyleInfo>,
}

/// ANSI style information
#[derive(Debug, Clone)]
pub struct StyleInfo {
    /// Foreground color (RGB)
    pub fg_color: Option<[u8; 3]>,
    /// Background color (RGB)
    pub bg_color: Option<[u8; 3]>,
    /// Bold text
    pub bold: bool,
    /// Italic text
    pub italic: bool,
    /// Underlined text
    pub underline: bool,
}

impl Default for StyleInfo {
    fn default() -> Self {
        Self {
            fg_color: None,
            bg_color: None,
            bold: false,
            italic: false,
            underline: false,
        }
    }
}

/// Terminal session state
#[derive(Debug)]
pub struct TerminalSession {
    /// Unique session ID
    pub id: Uuid,
    /// Session title
    pub title: String,
    /// Output buffer
    pub output_buffer: VecDeque<OutputLine>,
    /// Current input line
    pub current_input: String,
    /// Cursor position in current input
    pub cursor_position: usize,
    /// Command history for this session
    pub history: CommandHistory,
    /// Current working directory
    pub working_directory: String,
    /// Whether the session is active (has running process)
    pub is_active: bool,
    /// Process ID if running
    pub process_id: Option<u32>,
    /// Session creation time
    pub created_at: DateTime<Utc>,
    /// Last activity time
    pub last_activity: DateTime<Utc>,
    /// Maximum lines in output buffer
    pub max_output_lines: usize,
    /// Scroll position (0 = bottom, positive = scrolled up)
    pub scroll_position: usize,
}

impl TerminalSession {
    /// Create a new terminal session
    pub fn new(title: String, working_directory: String, config: &TerminalConfig) -> Self {
        Self {
            id: Uuid::new_v4(),
            title,
            output_buffer: VecDeque::new(),
            current_input: String::new(),
            cursor_position: 0,
            history: CommandHistory::new(1000),
            working_directory,
            is_active: false,
            process_id: None,
            created_at: Utc::now(),
            last_activity: Utc::now(),
            max_output_lines: config.scrollback_lines,
            scroll_position: 0,
        }
    }

    /// Add output line to the session
    pub fn add_output(&mut self, content: String, is_input: bool) {
        let line = OutputLine {
            content,
            timestamp: Utc::now(),
            is_input,
            style_info: Some(StyleInfo::default()),
        };

        self.output_buffer.push_back(line);
        self.last_activity = Utc::now();

        // Remove old lines if buffer is too large
        while self.output_buffer.len() > self.max_output_lines {
            self.output_buffer.pop_front();
        }

        // Reset scroll position to bottom when new content is added
        self.scroll_position = 0;
    }

    /// Add styled output line
    pub fn add_styled_output(&mut self, content: String, style: StyleInfo, is_input: bool) {
        let line = OutputLine {
            content,
            timestamp: Utc::now(),
            is_input,
            style_info: Some(style),
        };

        self.output_buffer.push_back(line);
        self.last_activity = Utc::now();

        // Remove old lines if buffer is too large
        while self.output_buffer.len() > self.max_output_lines {
            self.output_buffer.pop_front();
        }

        // Reset scroll position to bottom when new content is added
        self.scroll_position = 0;
    }

    /// Clear the output buffer
    pub fn clear_output(&mut self) {
        self.output_buffer.clear();
        self.scroll_position = 0;
    }

    /// Get visible output lines (considering scroll position)
    pub fn get_visible_output(&self, max_lines: usize) -> Vec<&OutputLine> {
        let total_lines = self.output_buffer.len();
        if total_lines == 0 {
            return Vec::new();
        }

        let start_index = if self.scroll_position >= total_lines {
            0
        } else {
            total_lines - self.scroll_position - std::cmp::min(max_lines, total_lines - self.scroll_position)
        };

        let end_index = if self.scroll_position == 0 {
            total_lines
        } else {
            total_lines - self.scroll_position
        };

        self.output_buffer
            .range(start_index..end_index)
            .collect()
    }

    /// Scroll up in the output
    pub fn scroll_up(&mut self, lines: usize) {
        let max_scroll = self.output_buffer.len().saturating_sub(1);
        self.scroll_position = std::cmp::min(self.scroll_position + lines, max_scroll);
    }

    /// Scroll down in the output
    pub fn scroll_down(&mut self, lines: usize) {
        self.scroll_position = self.scroll_position.saturating_sub(lines);
    }

    /// Scroll to bottom
    pub fn scroll_to_bottom(&mut self) {
        self.scroll_position = 0;
    }

    /// Scroll to top
    pub fn scroll_to_top(&mut self) {
        self.scroll_position = self.output_buffer.len().saturating_sub(1);
    }

    /// Insert character at cursor position
    pub fn insert_char(&mut self, ch: char) {
        if self.cursor_position <= self.current_input.len() {
            self.current_input.insert(self.cursor_position, ch);
            self.cursor_position += 1;
        }
    }

    /// Delete character before cursor
    pub fn delete_char_before_cursor(&mut self) {
        if self.cursor_position > 0 {
            self.current_input.remove(self.cursor_position - 1);
            self.cursor_position -= 1;
        }
    }

    /// Delete character at cursor
    pub fn delete_char_at_cursor(&mut self) {
        if self.cursor_position < self.current_input.len() {
            self.current_input.remove(self.cursor_position);
        }
    }

    /// Move cursor left
    pub fn move_cursor_left(&mut self) {
        if self.cursor_position > 0 {
            self.cursor_position -= 1;
        }
    }

    /// Move cursor right
    pub fn move_cursor_right(&mut self) {
        if self.cursor_position < self.current_input.len() {
            self.cursor_position += 1;
        }
    }

    /// Move cursor to beginning of line
    pub fn move_cursor_to_start(&mut self) {
        self.cursor_position = 0;
    }

    /// Move cursor to end of line
    pub fn move_cursor_to_end(&mut self) {
        self.cursor_position = self.current_input.len();
    }

    /// Clear current input
    pub fn clear_input(&mut self) {
        self.current_input.clear();
        self.cursor_position = 0;
    }

    /// Set current input (used for history navigation)
    pub fn set_input(&mut self, input: String) {
        self.current_input = input;
        self.cursor_position = self.current_input.len();
    }

    /// Execute current input as command
    pub fn execute_current_input(&mut self) -> String {
        let command = self.current_input.clone();

        // Add command to output as input line
        if !command.trim().is_empty() {
            self.add_output(format!("$ {}", command), true);

            // Add to history
            self.history.add_command(command.clone(), self.working_directory.clone());
        }

        // Clear current input
        self.clear_input();

        command
    }

    /// Navigate to previous command in history
    pub fn history_previous(&mut self) -> Option<String> {
        if let Some(command) = self.history.previous() {
            Some(command)
        } else {
            None
        }
    }

    /// Navigate to next command in history
    pub fn history_next(&mut self) -> Option<String> {
        self.history.next()
    }

    /// Get output as string vector for compatibility
    pub fn output(&self) -> Vec<String> {
        self.output_buffer
            .iter()
            .map(|line| line.content.clone())
            .collect()
    }
}
