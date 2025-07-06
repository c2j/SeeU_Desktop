use eframe::egui;
use egui_term::{TerminalView, TerminalBackend, BackendSettings};
use alacritty_terminal::event::Event as PtyEvent;
use std::collections::HashMap;
use std::sync::mpsc;
use uuid::Uuid;
use crate::export::{TerminalExporter, ExportOptions, ExportResult, ExportError};
use crate::session_history::{SavedSession, SessionHistoryManager};

/// Terminal session using egui_term
pub struct EguiTerminalSession {
    pub id: Uuid,
    pub title: String,
    pub backend: Option<TerminalBackend>,
    pub is_active: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_activity: chrono::DateTime<chrono::Utc>,
    pub event_receiver: Option<mpsc::Receiver<(u64, PtyEvent)>>,
}

impl EguiTerminalSession {
    pub fn new(title: String) -> Self {
        let id = Uuid::new_v4();
        let now = chrono::Utc::now();
        
        Self {
            id,
            title,
            backend: None,
            is_active: false,
            created_at: now,
            last_activity: now,
            event_receiver: None,
        }
    }

    pub fn initialize_terminal(&mut self, ctx: &egui::Context) -> Result<(), Box<dyn std::error::Error>> {
        // Create a channel for terminal events
        let (sender, receiver) = mpsc::channel();

        // Generate a unique ID for this terminal
        let terminal_id = self.id.as_u128() as u64;

        match TerminalBackend::new(
            terminal_id,
            ctx.clone(),
            sender,
            BackendSettings::default()
        ) {
            Ok(backend) => {
                self.backend = Some(backend);
                self.event_receiver = Some(receiver);
                self.last_activity = chrono::Utc::now();
                log::info!("Terminal backend initialized for session {}", self.id);
                Ok(())
            }
            Err(e) => {
                log::error!("Failed to initialize terminal backend for session {}: {}", self.id, e);
                Err(Box::new(e))
            }
        }
    }

    pub fn is_terminal_ready(&self) -> bool {
        self.backend.is_some()
    }

    pub fn render(&mut self, ui: &mut egui::Ui) {
        if let Some(ref mut backend) = self.backend {
            // Update last activity
            self.last_activity = chrono::Utc::now();

            // Create and render the terminal view
            let terminal_view = TerminalView::new(ui, backend)
                .set_focus(self.is_active)
                .set_size(ui.available_size());

            ui.add(terminal_view);
        } else {
            // Show initialization message
            ui.vertical_centered(|ui| {
                ui.add_space(20.0);
                ui.label("🔄 Initializing terminal...");

                if ui.button("Retry").clicked() {
                    // We need the context to initialize the terminal
                    // This will be handled by the caller
                    log::info!("Terminal initialization retry requested for session {}", self.id);
                }
            });
        }
    }

    /// Export terminal content with the given options
    pub fn export_content(&self, options: &ExportOptions) -> Result<ExportResult, ExportError> {
        TerminalExporter::export_session(self, options)
    }

    /// Get a quick text export of the terminal content
    pub fn get_text_content(&self) -> Result<String, ExportError> {
        let options = ExportOptions {
            format: crate::export::ExportFormat::PlainText,
            include_metadata: false,
            include_empty_lines: false,
            max_lines: None,
            strip_ansi: true,
            include_line_numbers: false,
        };

        let result = self.export_content(&options)?;
        Ok(result.content)
    }
}

/// Manager for multiple terminal sessions using egui_term
pub struct EguiTerminalManager {
    sessions: HashMap<Uuid, EguiTerminalSession>,
    active_session_id: Option<Uuid>,
    next_session_number: usize,
}

impl EguiTerminalManager {
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
            active_session_id: None,
            next_session_number: 1,
        }
    }

    pub fn create_session(&mut self, title: Option<String>, ctx: &egui::Context) -> Result<Uuid, String> {
        let title = title.unwrap_or_else(|| {
            let title = format!("Terminal {}", self.next_session_number);
            self.next_session_number += 1;
            title
        });

        let mut session = EguiTerminalSession::new(title.clone());
        let session_id = session.id;

        // Initialize the terminal - this is critical for keyboard input
        match session.initialize_terminal(ctx) {
            Ok(_) => {
                log::info!("Terminal backend initialized successfully for new session");
            }
            Err(e) => {
                let error_msg = format!("Failed to initialize terminal backend for new session '{}': {}", title, e);
                log::error!("{}", error_msg);
                return Err(error_msg);
            }
        }

        // Verify the terminal is ready
        if !session.is_terminal_ready() {
            let error_msg = format!("Terminal backend is not ready after initialization for session '{}'", title);
            log::error!("{}", error_msg);
            return Err(error_msg);
        }

        // Set as active if it's the first session
        if self.sessions.is_empty() {
            self.active_session_id = Some(session_id);
            session.is_active = true;
        }

        self.sessions.insert(session_id, session);
        log::info!("Created new terminal session: {} ({})", title, session_id);

        Ok(session_id)
    }

    pub fn close_session(&mut self, session_id: Uuid) -> bool {
        if let Some(_session) = self.sessions.remove(&session_id) {
            log::info!("Closed terminal session: {}", session_id);

            // If this was the active session, switch to another one
            if self.active_session_id == Some(session_id) {
                self.active_session_id = self.sessions.keys().next().copied();
                
                // Update active status
                for session in self.sessions.values_mut() {
                    session.is_active = Some(session.id) == self.active_session_id;
                }
            }

            true
        } else {
            false
        }
    }

    pub fn set_active_session(&mut self, session_id: Uuid) -> bool {
        if self.sessions.contains_key(&session_id) {
            // Update active status for all sessions
            for session in self.sessions.values_mut() {
                session.is_active = session.id == session_id;
            }
            
            self.active_session_id = Some(session_id);
            log::info!("Set active session: {}", session_id);
            true
        } else {
            false
        }
    }

    pub fn get_active_session(&self) -> Option<&EguiTerminalSession> {
        self.active_session_id.and_then(|id| self.sessions.get(&id))
    }

    pub fn get_active_session_mut(&mut self) -> Option<&mut EguiTerminalSession> {
        self.active_session_id.and_then(|id| self.sessions.get_mut(&id))
    }

    pub fn get_active_session_id(&self) -> Option<Uuid> {
        self.active_session_id
    }

    pub fn get_sessions(&self) -> &HashMap<Uuid, EguiTerminalSession> {
        &self.sessions
    }

    pub fn session_count(&self) -> usize {
        self.sessions.len()
    }

    pub fn render_active_session(&mut self, ui: &mut egui::Ui) {
        if let Some(active_id) = self.active_session_id {
            if let Some(session) = self.sessions.get_mut(&active_id) {
                session.render(ui);
            } else {
                ui.vertical_centered(|ui| {
                    ui.add_space(50.0);
                    ui.label("❌ Active session not found");
                    if ui.button("Create New Session").clicked() {
                        match self.create_session(None, ui.ctx()) {
                            Ok(session_id) => {
                                log::info!("Created replacement session: {}", session_id);
                            }
                            Err(e) => {
                                log::error!("Failed to create replacement session: {}", e);
                            }
                        }
                    }
                });
            }
        } else {
            ui.vertical_centered(|ui| {
                ui.add_space(50.0);
                ui.label("📱 No terminal sessions");
                ui.add_space(10.0);
                ui.label("Create a new session to get started");
                if ui.button("+ Create New Session").clicked() {
                    match self.create_session(None, ui.ctx()) {
                        Ok(session_id) => {
                            log::info!("Created initial session: {}", session_id);
                        }
                        Err(e) => {
                            log::error!("Failed to create initial session: {}", e);
                        }
                    }
                }
            });
        }
    }

    pub fn render_session_tabs(&mut self, ui: &mut egui::Ui) {
        if self.sessions.len() <= 1 {
            return;
        }

        ui.horizontal(|ui| {
            let session_ids: Vec<_> = self.sessions.keys().copied().collect();
            
            for session_id in session_ids {
                if let Some(session) = self.sessions.get(&session_id) {
                    let is_active = self.active_session_id == Some(session_id);
                    let button_text = &session.title;

                    if ui.selectable_label(is_active, button_text).clicked() && !is_active {
                        self.set_active_session(session_id);
                    }
                }
            }
        });
    }

    pub fn update(&mut self) {
        // Update all sessions
        for _session in self.sessions.values_mut() {
            // Here we could add periodic updates if needed
            // For now, egui_term handles its own updates
        }
    }

    /// Export the active session content
    pub fn export_active_session(&self, options: &ExportOptions) -> Result<ExportResult, ExportError> {
        if let Some(session) = self.get_active_session() {
            session.export_content(options)
        } else {
            Err(ExportError::FormatError("No active session".to_string()))
        }
    }

    /// Export a specific session by ID
    pub fn export_session(&self, session_id: Uuid, options: &ExportOptions) -> Result<ExportResult, ExportError> {
        if let Some(session) = self.sessions.get(&session_id) {
            session.export_content(options)
        } else {
            Err(ExportError::FormatError("Session not found".to_string()))
        }
    }

    /// Get quick text content from active session
    pub fn get_active_session_text(&self) -> Result<String, ExportError> {
        if let Some(session) = self.get_active_session() {
            session.get_text_content()
        } else {
            Err(ExportError::FormatError("No active session".to_string()))
        }
    }

    /// Save the current active session to history
    pub fn save_active_session_to_history(&self, history_manager: &mut SessionHistoryManager) -> Result<Uuid, String> {
        if let Some(session) = self.get_active_session() {
            // Get session content
            let content = match self.get_active_session_text() {
                Ok(content) => content,
                Err(e) => {
                    log::warn!("Failed to get session content for saving: {}", e);
                    format!("Session content unavailable: {}", e)
                }
            };

            // Create saved session
            let saved_session = SavedSession::new(
                session.id,
                session.title.clone(),
                session.created_at,
                session.last_activity,
                content,
            );

            // Save to history
            match history_manager.save_session(saved_session.clone()) {
                Ok(_) => {
                    log::info!("Saved session '{}' to history", session.title);
                    Ok(saved_session.id)
                }
                Err(e) => {
                    let error_msg = format!("Failed to save session to history: {}", e);
                    log::error!("{}", error_msg);
                    Err(error_msg)
                }
            }
        } else {
            Err("No active session to save".to_string())
        }
    }

    /// Save a specific session to history
    pub fn save_session_to_history(&self, session_id: Uuid, history_manager: &mut SessionHistoryManager) -> Result<Uuid, String> {
        if let Some(session) = self.sessions.get(&session_id) {
            // Get session content
            let content = match self.export_session(session_id, &ExportOptions::default()) {
                Ok(result) => result.content,
                Err(e) => {
                    log::warn!("Failed to get session content for saving: {}", e);
                    format!("Session content unavailable: {}", e)
                }
            };

            // Create saved session
            let saved_session = SavedSession::new(
                session.id,
                session.title.clone(),
                session.created_at,
                session.last_activity,
                content,
            );

            // Save to history
            match history_manager.save_session(saved_session.clone()) {
                Ok(_) => {
                    log::info!("Saved session '{}' to history", session.title);
                    Ok(saved_session.id)
                }
                Err(e) => {
                    let error_msg = format!("Failed to save session to history: {}", e);
                    log::error!("{}", error_msg);
                    Err(error_msg)
                }
            }
        } else {
            Err("Session not found".to_string())
        }
    }

    /// Restore a session from history
    pub fn restore_session_from_history(&mut self, saved_session: &SavedSession, ctx: &eframe::egui::Context) -> Result<Uuid, String> {
        // Set working directory first if available
        if let Some(wd) = &saved_session.working_directory {
            if wd.exists() {
                if let Err(e) = std::env::set_current_dir(wd) {
                    log::warn!("Failed to set working directory to {}: {}", wd.display(), e);
                }
            }
        }

        // Create a new session with the saved data
        let title = format!("{} (Restored)", saved_session.title);
        let mut session = EguiTerminalSession::new(title);
        let session_id = session.id;

        // Initialize the terminal - this is critical for keyboard input
        match session.initialize_terminal(ctx) {
            Ok(_) => {
                log::info!("Terminal backend initialized successfully for restored session");
            }
            Err(e) => {
                let error_msg = format!("Failed to initialize terminal backend for restored session: {}", e);
                log::error!("{}", error_msg);
                return Err(error_msg);
            }
        }

        // Verify the terminal is ready
        if !session.is_terminal_ready() {
            let error_msg = "Terminal backend is not ready after initialization";
            log::error!("{}", error_msg);
            return Err(error_msg.to_string());
        }

        // TODO: Set the terminal content to the saved content
        // This would require backend-specific implementation to restore content
        // For now, we'll start with a fresh terminal in the correct working directory

        // Add session to manager
        self.sessions.insert(session_id, session);
        self.active_session_id = Some(session_id);

        log::info!("Successfully restored session '{}' from history with working terminal", saved_session.title);
        Ok(session_id)
    }
}

impl Default for EguiTerminalManager {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for EguiTerminalManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EguiTerminalManager")
            .field("sessions_count", &self.sessions.len())
            .field("active_session_id", &self.active_session_id)
            .field("next_session_number", &self.next_session_number)
            .finish()
    }
}

impl std::fmt::Debug for EguiTerminalSession {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EguiTerminalSession")
            .field("id", &self.id)
            .field("title", &self.title)
            .field("is_active", &self.is_active)
            .field("created_at", &self.created_at)
            .field("last_activity", &self.last_activity)
            .field("has_backend", &self.backend.is_some())
            .field("has_event_receiver", &self.event_receiver.is_some())
            .finish()
    }
}
