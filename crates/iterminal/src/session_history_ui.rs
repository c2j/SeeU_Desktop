use crate::session_history::{SavedSession, SessionHistoryManager};
use uuid::Uuid;

/// UI state for session history management
#[derive(Debug)]
pub struct SessionHistoryUI {
    /// Whether the history dialog is open
    pub is_open: bool,
    /// Whether the save session dialog is open
    pub save_dialog_open: bool,
    /// Session name for saving
    pub save_session_name: String,
    /// Whether this is the first frame of the save dialog (for focus handling)
    pub save_dialog_first_frame: bool,
    /// Search query for filtering sessions
    pub search_query: String,
    /// Selected session for operations
    pub selected_session: Option<Uuid>,
    /// Whether to show session details
    pub show_details: bool,
    /// Edit mode for session metadata
    pub edit_mode: bool,
    /// Temporary storage for editing session data
    pub edit_data: EditSessionData,
    /// Error message to display
    pub error_message: Option<String>,
    /// Success message to display
    pub success_message: Option<String>,
    /// Confirmation dialog state
    pub confirmation_dialog: Option<ConfirmationDialog>,
}

/// Data for editing session metadata
#[derive(Debug, Clone, Default)]
pub struct EditSessionData {
    pub title: String,
    pub notes: String,
    pub tags: String, // Comma-separated tags
}

/// Confirmation dialog state
#[derive(Debug, Clone)]
pub struct ConfirmationDialog {
    pub message: String,
    pub action: ConfirmationAction,
}

/// Actions that require confirmation
#[derive(Debug, Clone)]
pub enum ConfirmationAction {
    DeleteSession(Uuid),
    ClearAllSessions,
}

impl Default for SessionHistoryUI {
    fn default() -> Self {
        Self {
            is_open: false,
            save_dialog_open: false,
            save_session_name: String::new(),
            save_dialog_first_frame: false,
            search_query: String::new(),
            selected_session: None,
            show_details: false,
            edit_mode: false,
            edit_data: EditSessionData::default(),
            error_message: None,
            success_message: None,
            confirmation_dialog: None,
        }
    }
}

impl SessionHistoryUI {
    /// Open the session history dialog
    pub fn open(&mut self) {
        self.is_open = true;
        self.clear_messages();
    }

    /// Close the session history dialog
    pub fn close(&mut self) {
        self.is_open = false;
        self.selected_session = None;
        self.show_details = false;
        self.edit_mode = false;
        self.clear_messages();
        self.confirmation_dialog = None;
    }

    /// Open the save session dialog
    pub fn open_save_dialog(&mut self) {
        self.save_dialog_open = true;
        self.save_dialog_first_frame = true;
        self.save_session_name = format!("会话 {}", chrono::Utc::now().format("%Y-%m-%d %H:%M"));
        self.clear_messages();
    }

    /// Close the save session dialog
    pub fn close_save_dialog(&mut self) {
        self.save_dialog_open = false;
        self.save_dialog_first_frame = false;
        self.save_session_name.clear();
    }

    /// Clear error and success messages
    pub fn clear_messages(&mut self) {
        self.error_message = None;
        self.success_message = None;
    }

    /// Set error message
    pub fn set_error(&mut self, message: String) {
        self.error_message = Some(message);
        self.success_message = None;
    }

    /// Set success message
    pub fn set_success(&mut self, message: String) {
        self.success_message = Some(message);
        self.error_message = None;
    }

    /// Start editing a session
    pub fn start_edit(&mut self, session: &SavedSession) {
        self.edit_mode = true;
        self.edit_data.title = session.title.clone();
        self.edit_data.notes = session.notes.clone();
        self.edit_data.tags = session.tags.join(", ");
    }

    /// Cancel editing
    pub fn cancel_edit(&mut self) {
        self.edit_mode = false;
        self.edit_data = EditSessionData::default();
    }

    /// Show confirmation dialog
    pub fn show_confirmation(&mut self, message: String, action: ConfirmationAction) {
        self.confirmation_dialog = Some(ConfirmationDialog { message, action });
    }

    /// Render the session history dialog
    pub fn render(
        &mut self,
        ctx: &eframe::egui::Context,
        history_manager: &mut SessionHistoryManager,
    ) -> Option<SessionHistoryAction> {
        let mut action = None;

        // Render save session dialog
        if self.save_dialog_open {
            action = self.render_save_dialog(ctx);
        }

        if !self.is_open {
            return action;
        }

        eframe::egui::Window::new("📚 Session History")
            .default_width(800.0)
            .default_height(600.0)
            .resizable(true)
            .collapsible(false)
            .show(ctx, |ui| {
                action = self.render_content(ui, history_manager);
            });

        // Handle confirmation dialog
        if let Some(confirmation) = &self.confirmation_dialog.clone() {
            let mut should_close = false;
            let mut confirmed = false;

            eframe::egui::Window::new("⚠️ Confirmation")
                .default_width(400.0)
                .resizable(false)
                .collapsible(false)
                .show(ctx, |ui| {
                    ui.vertical(|ui| {
                        ui.label(&confirmation.message);
                        ui.add_space(10.0);
                        
                        ui.horizontal(|ui| {
                            if ui.button("✅ Confirm").clicked() {
                                confirmed = true;
                                should_close = true;
                            }
                            
                            if ui.button("❌ Cancel").clicked() {
                                should_close = true;
                            }
                        });
                    });
                });

            if should_close {
                if confirmed {
                    action = Some(self.handle_confirmation_action(confirmation.action.clone(), history_manager));
                }
                self.confirmation_dialog = None;
            }
        }

        action
    }

    /// Render the main content of the dialog
    fn render_content(
        &mut self,
        ui: &mut eframe::egui::Ui,
        history_manager: &SessionHistoryManager,
    ) -> Option<SessionHistoryAction> {
        let mut action = None;

        ui.vertical(|ui| {
            // Header with search and actions
            ui.horizontal(|ui| {
                ui.label("🔍 Search:");
                ui.add(eframe::egui::TextEdit::singleline(&mut self.search_query)
                    .id(eframe::egui::Id::new("session_search_query")));
                
                ui.separator();
                
                if ui.button("🔄 刷新").clicked() {
                    // Refresh will be handled by the caller
                    action = Some(SessionHistoryAction::Refresh);
                }

                if ui.button("🗑️ 清空全部").clicked() {
                    self.show_confirmation(
                        "确定要删除所有保存的会话吗？此操作无法撤销。".to_string(),
                        ConfirmationAction::ClearAllSessions,
                    );
                }
            });

            ui.separator();

            // Display messages
            if let Some(error) = &self.error_message {
                ui.colored_label(eframe::egui::Color32::RED, format!("❌ {}", error));
            }
            if let Some(success) = &self.success_message {
                ui.colored_label(eframe::egui::Color32::GREEN, format!("✅ {}", success));
            }

            // Session list and details
            ui.horizontal(|ui| {
                // Left panel: Session list
                ui.vertical(|ui| {
                    ui.set_width(350.0);
                    ui.heading("Sessions");
                    
                    eframe::egui::ScrollArea::vertical()
                        .id_source("session_history_list")
                        .max_height(400.0)
                        .show(ui, |ui| {
                            let sessions = if self.search_query.is_empty() {
                                history_manager.get_all_sessions()
                            } else {
                                history_manager.search_sessions(&self.search_query)
                            };

                            for session in &sessions {
                                let is_selected = self.selected_session == Some(session.id);

                                ui.group(|ui| {
                                    ui.vertical(|ui| {
                                        let response = ui.selectable_label(
                                            is_selected,
                                            format!("📄 {}", session.title)
                                        );

                                        // Single click to select
                                        if response.clicked() {
                                            self.selected_session = Some(session.id);
                                            self.show_details = true;
                                            self.edit_mode = false;
                                        }

                                        // Double click to restore directly
                                        if response.double_clicked() {
                                            action = Some(SessionHistoryAction::RestoreSession(session.id));
                                        }

                                        // Show session info
                                        ui.small(format!(
                                            "Created: {} | Last: {}",
                                            session.created_at.format("%Y-%m-%d %H:%M"),
                                            session.last_activity.format("%Y-%m-%d %H:%M")
                                        ));

                                        if !session.tags.is_empty() {
                                            ui.small(format!("Tags: {}", session.tags.join(", ")));
                                        }

                                        // Add a small hint for double-click
                                        ui.small("💡 双击恢复会话");
                                    });
                                });

                                ui.add_space(5.0);
                            }

                            if sessions.is_empty() {
                                ui.label("No sessions found");
                            }
                        });
                });

                ui.separator();

                // Right panel: Session details
                ui.vertical(|ui| {
                    if let Some(session_id) = self.selected_session {
                        // Find the session in the list
                        let sessions = history_manager.get_all_sessions();
                        if let Some(session) = sessions.iter().find(|s| s.id == session_id) {
                            action = self.render_session_details(ui, session);
                        }
                    } else {
                        ui.label("Select a session to view details");
                    }
                });
            });

            ui.separator();

            // Bottom buttons
            ui.horizontal(|ui| {
                if ui.button("❌ 关闭").clicked() {
                    self.close();
                }
            });
        });

        action
    }

    /// Render session details panel
    fn render_session_details(
        &mut self,
        ui: &mut eframe::egui::Ui,
        session: &SavedSession,
    ) -> Option<SessionHistoryAction> {
        let mut action = None;

        ui.heading("Session Details");

        if self.edit_mode {
            // Edit mode
            ui.horizontal(|ui| {
                ui.label("Title:");
                ui.add(eframe::egui::TextEdit::singleline(&mut self.edit_data.title)
                    .id(eframe::egui::Id::new("session_edit_title")));
            });

            ui.horizontal(|ui| {
                ui.label("Tags:");
                ui.add(eframe::egui::TextEdit::singleline(&mut self.edit_data.tags)
                    .id(eframe::egui::Id::new("session_edit_tags")));
            });

            ui.label("Notes:");
            ui.add(eframe::egui::TextEdit::multiline(&mut self.edit_data.notes)
                .id(eframe::egui::Id::new("session_edit_notes")));

            ui.horizontal(|ui| {
                if ui.button("💾 Save").clicked() {
                    action = Some(SessionHistoryAction::UpdateSession {
                        id: session.id,
                        title: self.edit_data.title.clone(),
                        notes: self.edit_data.notes.clone(),
                        tags: self.edit_data.tags
                            .split(',')
                            .map(|s| s.trim().to_string())
                            .filter(|s| !s.is_empty())
                            .collect(),
                    });
                    self.edit_mode = false;
                }

                if ui.button("❌ Cancel").clicked() {
                    self.cancel_edit();
                }
            });
        } else {
            // View mode
            ui.label(format!("Title: {}", session.title));
            ui.label(format!("Created: {}", session.created_at.format("%Y-%m-%d %H:%M:%S")));
            ui.label(format!("Last Activity: {}", session.last_activity.format("%Y-%m-%d %H:%M:%S")));
            ui.label(format!("Saved: {}", session.saved_at.format("%Y-%m-%d %H:%M:%S")));

            if let Some(wd) = &session.working_directory {
                ui.label(format!("Working Directory: {}", wd.display()));
            }

            if !session.tags.is_empty() {
                ui.label(format!("Tags: {}", session.tags.join(", ")));
            }

            if !session.notes.is_empty() {
                ui.label("Notes:");
                ui.label(&session.notes);
            }

            // Content preview
            ui.label("Content Preview:");
            eframe::egui::ScrollArea::vertical()
                .id_source("session_content_preview")
                .max_height(200.0)
                .show(ui, |ui| {
                    ui.code(&session.content);
                });

            // Action buttons
            ui.horizontal(|ui| {
                if ui.button("🔄 恢复会话").clicked() {
                    // Direct restore without confirmation
                    action = Some(SessionHistoryAction::RestoreSession(session.id));
                }

                if ui.button("✏️ 编辑").clicked() {
                    self.start_edit(session);
                }

                if ui.button("🗑️ 删除").clicked() {
                    self.show_confirmation(
                        format!("删除会话 '{}'？此操作无法撤销。", session.title),
                        ConfirmationAction::DeleteSession(session.id),
                    );
                }
            });
        }

        action
    }

    /// Handle confirmation dialog actions
    fn handle_confirmation_action(
        &mut self,
        action: ConfirmationAction,
        _history_manager: &mut SessionHistoryManager,
    ) -> SessionHistoryAction {
        match action {
            ConfirmationAction::DeleteSession(id) => {
                SessionHistoryAction::DeleteSession(id)
            }
            ConfirmationAction::ClearAllSessions => {
                SessionHistoryAction::ClearAllSessions
            }
        }
    }

    /// Render the save session dialog
    fn render_save_dialog(&mut self, ctx: &eframe::egui::Context) -> Option<SessionHistoryAction> {
        let mut action = None;
        let mut should_close = false;

        eframe::egui::Window::new("💾 保存会话")
            .default_width(400.0)
            .default_height(200.0)
            .resizable(false)
            .collapsible(false)
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    ui.add_space(10.0);

                    ui.label("请为当前会话输入一个名称：");
                    ui.add_space(5.0);

                    let response = ui.add(
                        eframe::egui::TextEdit::singleline(&mut self.save_session_name)
                            .desired_width(ui.available_width())
                            .hint_text("输入会话名称...")
                    );

                    // Auto-focus the text input on first frame
                    if self.save_dialog_first_frame {
                        response.request_focus();
                        self.save_dialog_first_frame = false;
                    }

                    ui.add_space(10.0);

                    // Display messages
                    if let Some(error) = &self.error_message {
                        ui.colored_label(eframe::egui::Color32::RED, format!("❌ {}", error));
                    }
                    if let Some(success) = &self.success_message {
                        ui.colored_label(eframe::egui::Color32::GREEN, format!("✅ {}", success));
                    }

                    ui.add_space(10.0);

                    ui.horizontal(|ui| {
                        if ui.button("💾 保存").clicked() {
                            if self.save_session_name.trim().is_empty() {
                                self.set_error("会话名称不能为空".to_string());
                            } else {
                                action = Some(SessionHistoryAction::SaveSession(self.save_session_name.clone()));
                                should_close = true;
                            }
                        }

                        if ui.button("❌ 取消").clicked() {
                            should_close = true;
                        }
                    });

                    // Handle Enter key
                    if response.lost_focus() && ui.input(|i| i.key_pressed(eframe::egui::Key::Enter)) {
                        if !self.save_session_name.trim().is_empty() {
                            action = Some(SessionHistoryAction::SaveSession(self.save_session_name.clone()));
                            should_close = true;
                        }
                    }
                });
            });

        if should_close {
            self.close_save_dialog();
        }

        action
    }
}

/// Actions that can be triggered from the session history UI
#[derive(Debug, Clone)]
pub enum SessionHistoryAction {
    /// Save current session with a custom name
    SaveSession(String),
    /// Restore a saved session
    RestoreSession(Uuid),
    /// Delete a saved session
    DeleteSession(Uuid),
    /// Update session metadata
    UpdateSession {
        id: Uuid,
        title: String,
        notes: String,
        tags: Vec<String>,
    },
    /// Clear all saved sessions
    ClearAllSessions,
    /// Refresh the session list
    Refresh,
}
