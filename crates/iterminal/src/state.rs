use crate::config::TerminalConfig;
use crate::egui_terminal::EguiTerminalManager;
use crate::export_ui::ExportDialog;
use crate::session_history::SessionHistoryManager;
use crate::session_history_ui::SessionHistoryUI;
use crate::help_ui::TerminalHelpUI;
use eframe::egui;
use uuid::Uuid;

/// Main state for the iTerminal module
#[derive(Debug)]
pub struct ITerminalState {
    /// egui_term based terminal manager
    pub egui_terminal_manager: EguiTerminalManager,
    /// Terminal configuration
    pub config: TerminalConfig,
    /// Export dialog state
    pub export_dialog: ExportDialog,
    /// Session history manager
    pub session_history_manager: Option<SessionHistoryManager>,
    /// Session history UI state
    pub session_history_ui: SessionHistoryUI,
    /// Terminal help UI state
    pub help_ui: TerminalHelpUI,
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
    /// Terminal history for display
    pub terminal_history: Vec<String>,
    /// Current session name
    pub current_session: String,
}

impl ITerminalState {
    /// Create a new terminal state
    pub fn new() -> Self {
        let config = TerminalConfig::load();
        let egui_terminal_manager = EguiTerminalManager::new();

        // Initialize session history manager
        let session_history_manager = match SessionHistoryManager::new() {
            Ok(manager) => {
                log::info!("Session history manager initialized successfully");
                Some(manager)
            }
            Err(e) => {
                log::warn!("Failed to initialize session history manager: {}", e);
                None
            }
        };

        Self {
            egui_terminal_manager,
            config,
            export_dialog: ExportDialog::default(),
            session_history_manager,
            session_history_ui: SessionHistoryUI::default(),
            help_ui: TerminalHelpUI::default(),
            show_history: false,
            history_search: String::new(),
            has_focus: false,
            cursor_visible: true,
            font_scale: 1.0,
            last_cursor_blink: std::time::Instant::now(),
            terminal_history: Vec::new(),
            current_session: "主会话".to_string(),
        }
    }

    /// Update the terminal state
    pub fn update(&mut self) {
        // Update egui terminal manager
        self.egui_terminal_manager.update();

        // Handle cursor blinking
        if self.has_focus && self.config.cursor_blink_interval > 0 {
            let elapsed = self.last_cursor_blink.elapsed();
            if elapsed.as_millis() >= self.config.cursor_blink_interval as u128 {
                self.cursor_visible = !self.cursor_visible;
                self.last_cursor_blink = std::time::Instant::now();
            }
        } else {
            self.cursor_visible = true;
        }
    }

    /// Create a new terminal session
    pub fn create_session(&mut self, title: Option<String>, ctx: Option<&egui::Context>) -> Result<Uuid, String> {
        if let Some(ctx) = ctx {
            self.egui_terminal_manager.create_session(title, ctx)
        } else {
            let error_msg = "Cannot create egui_term session without context";
            log::error!("{}", error_msg);
            Err(error_msg.to_string())
        }
    }

    /// Close a terminal session
    pub fn close_session(&mut self, session_id: Uuid) -> bool {
        self.egui_terminal_manager.close_session(session_id)
    }

    /// Set the active session
    pub fn set_active_session(&mut self, session_id: Uuid) -> bool {
        self.egui_terminal_manager.set_active_session(session_id)
    }

    /// Toggle history dialog
    pub fn toggle_history(&mut self) {
        self.show_history = !self.show_history;
    }

    /// Save current configuration
    pub fn save_config(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.config.save()
    }

    /// Update configuration
    pub fn update_config(&mut self, config: TerminalConfig) {
        self.config = config;
    }

    /// Get current configuration
    pub fn get_config(&self) -> &TerminalConfig {
        &self.config
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

    /// Get the number of sessions
    pub fn session_count(&self) -> usize {
        self.egui_terminal_manager.session_count()
    }

    /// Get the egui terminal manager
    pub fn get_egui_terminal_manager(&self) -> &EguiTerminalManager {
        &self.egui_terminal_manager
    }

    /// Get the egui terminal manager (mutable)
    pub fn get_egui_terminal_manager_mut(&mut self) -> &mut EguiTerminalManager {
        &mut self.egui_terminal_manager
    }


}

impl Default for ITerminalState {
    fn default() -> Self {
        Self::new()
    }
}
