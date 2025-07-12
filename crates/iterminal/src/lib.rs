

pub mod state;
pub mod config;
pub mod settings_ui;
pub mod egui_terminal;
pub mod export;
pub mod export_ui;
pub mod session_history;
pub mod session_history_ui;
pub mod help_content;
pub mod help_ui;
// 远程服务器管理相关模块
pub mod remote_server;
pub mod remote_server_manager;
pub mod ssh_connection;
pub mod encryption;
pub mod remote_server_ui;

pub use state::ITerminalState;
use export_ui::{ExportDialog};
use egui_terminal::EguiTerminalManager;
use session_history_ui::SessionHistoryAction;

/// Create a settings module for iterminal
pub fn create_settings_module(state: &mut ITerminalState) -> settings_ui::ITerminalSettingsModule {
    settings_ui::ITerminalSettingsModule::new(state)
}

/// Initialize the iTerminal module
pub fn initialize() -> ITerminalState {
    log::info!("Initializing iTerminal module");
    ITerminalState::new()
}

/// Update function for background tasks
pub fn update_iterminal(state: &mut ITerminalState) {
    state.update();
}

/// egui terminal interface using egui_term
pub fn render_iterminal(ui: &mut eframe::egui::Ui, state: &mut ITerminalState) {
    // Update the state
    update_iterminal(state);

    // Get available space
    let available_rect = ui.available_rect_before_wrap();
    let available_width = available_rect.width();
    let available_height = available_rect.height();

    render_egui_terminal(ui, state, available_width, available_height);
}

/// Render the egui_term based terminal
pub fn render_egui_terminal(ui: &mut eframe::egui::Ui, state: &mut ITerminalState, _available_width: f32, _available_height: f32) {
    // Handle export dialog rendering separately to avoid borrowing issues
    if state.export_dialog.is_open {
        let ctx = ui.ctx().clone();
        // We need to split the borrow to avoid conflicts
        let export_dialog = &mut state.export_dialog;
        let terminal_manager = &state.egui_terminal_manager;

        // Render export dialog with limited state access
        render_export_dialog(&ctx, export_dialog, terminal_manager);
    }

    // Handle session history dialog
    if let Some(history_manager) = &mut state.session_history_manager {
        let ctx = ui.ctx().clone();
        if let Some(action) = state.session_history_ui.render(&ctx, history_manager) {
            handle_session_history_action(state, action);
        }
    }

    // Handle help dialog
    let ctx = ui.ctx().clone();
    state.help_ui.render(&ctx);

    // Create vertical layout
    ui.vertical(|ui| {
        // Header with controls
        ui.horizontal(|ui| {
            ui.label("🖥 Terminal");
            ui.separator();

            // Session management buttons
            if ui.button("+ 新建会话").clicked() {
                log::info!("Creating new terminal session");
                match state.create_session(None, Some(ui.ctx())) {
                    Ok(session_id) => {
                        log::info!("Successfully created new terminal session: {}", session_id);
                    }
                    Err(e) => {
                        log::error!("Failed to create new terminal session: {}", e);
                        // TODO: Show error message to user
                    }
                }
            }

            if ui.button("关闭会话").clicked() {
                if let Some(session_id) = state.get_egui_terminal_manager().get_active_session_id() {
                    log::info!("Closing terminal session");
                    state.close_session(session_id);
                }
            }

            ui.separator();

            // Remote server management buttons
            // 远程服务器按钮 - 点击时才初始化
            if ui.button("🌐 远程服务器").clicked() {
                if state.ensure_remote_server_ui() {
                    state.show_remote_servers = !state.show_remote_servers;
                    state.show_compact_remote_panel = true; // 默认显示紧凑面板
                } else {
                    log::error!("Failed to initialize remote server UI");
                }
            }

            // 只有在已初始化的情况下才显示快速连接功能
            if state.has_remote_servers() {
                // Quick connect button for recent servers
                let has_servers = if let Some(remote_ui) = state.get_remote_server_ui() {
                    !remote_ui.manager.list_servers().is_empty()
                } else {
                    false
                };

                if has_servers {
                    // 收集服务器数据以避免借用问题
                    let server_data: Vec<_> = if let Some(remote_ui) = state.get_remote_server_ui() {
                        remote_ui.manager.list_servers().iter()
                            .take(5)
                            .filter(|s| s.enabled)
                            .map(|s| (s.id, s.name.clone(), s.get_connection_string()))
                            .collect()
                    } else {
                        Vec::new()
                    };

                    let mut selected_server_id = None;
                    let mut show_add_dialog = false;
                    let mut show_manage = false;

                    eframe::egui::ComboBox::from_label("快速连接")
                        .selected_text("选择服务器")
                        .show_ui(ui, |ui| {
                            for (server_id, name, connection_string) in server_data {
                                let display_text = format!("{} ({})", name, connection_string);
                                if ui.selectable_label(false, display_text).clicked() {
                                    selected_server_id = Some(server_id);
                                }
                            }

                            ui.separator();
                            if ui.button("➕ 添加新服务器").clicked() {
                                show_add_dialog = true;
                            }

                            if ui.button("⚙️ 管理服务器").clicked() {
                                show_manage = true;
                            }
                        });

                    // 处理选择的操作
                    if let Some(server_id) = selected_server_id {
                        if let Err(e) = state.handle_remote_server_action(
                            crate::remote_server_ui::RemoteServerAction::Connect(server_id),
                            ui.ctx()
                        ) {
                            log::error!("Failed to connect to server: {}", e);
                        }
                    }

                    if show_add_dialog {
                        if let Some(remote_ui) = state.get_remote_server_ui_mut() {
                            remote_ui.show_add_dialog();
                        }
                    }

                    if show_manage {
                        state.show_remote_servers = true;
                    }
                }
            }

            ui.separator();

            // Session history buttons
            if ui.button("💾 保存会话").clicked() {
                // Open save session dialog
                state.session_history_ui.open_save_dialog();
            }

            if ui.button("📚 会话历史").clicked() {
                state.session_history_ui.open();
            }

            ui.separator();

            // Export buttons
            if ui.button("📤 导出").clicked() {
                state.export_dialog.open();
            }

            if ui.button("📋 快速复制").clicked() {
                match crate::export_ui::QuickExport::copy_as_text(state.get_egui_terminal_manager()) {
                    Ok(_) => log::info!("Terminal content copied to clipboard"),
                    Err(e) => log::error!("Failed to copy terminal content: {}", e),
                }
            }

            ui.separator();

            // Help button
            if ui.button("❓ 帮助").clicked() {
                state.help_ui.open();
            }
        });

        ui.separator();

        // Remote server management panel (if shown)
        if state.show_remote_servers {
            ui.group(|ui| {
                ui.vertical(|ui| {
                    if state.show_compact_remote_panel {
                        // 紧凑面板标题栏
                        ui.horizontal(|ui| {
                            ui.label(eframe::egui::RichText::new("🌐 远程服务器").strong());
                            ui.with_layout(eframe::egui::Layout::right_to_left(eframe::egui::Align::Center), |ui| {
                                if ui.small_button("✖").on_hover_text("关闭").clicked() {
                                    state.show_remote_servers = false;
                                }
                                if ui.small_button("⚙️").on_hover_text("完整管理").clicked() {
                                    state.show_compact_remote_panel = false;
                                }
                            });
                        });

                        ui.separator();

                        // 渲染紧凑面板
                        let ctx = ui.ctx().clone();
                        if let Some(remote_ui) = state.get_remote_server_ui_mut_lazy() {
                            if let Some(action) = remote_ui.render_compact_panel(ui, &ctx) {
                                if let Err(e) = state.handle_remote_server_action(action, &ctx) {
                                    log::error!("Failed to handle remote server action: {}", e);
                                }
                            }
                        } else {
                            ui.label("远程服务器功能初始化失败");
                        }
                    } else {
                        // 完整管理界面标题栏
                        ui.horizontal(|ui| {
                            ui.heading("远程服务器管理");
                            ui.with_layout(eframe::egui::Layout::right_to_left(eframe::egui::Align::Center), |ui| {
                                if ui.button("✖ 关闭").clicked() {
                                    state.show_remote_servers = false;
                                }
                                if ui.button("📱 紧凑视图").clicked() {
                                    state.show_compact_remote_panel = true;
                                }
                            });
                        });

                        ui.separator();

                        // 渲染完整管理界面
                        let ctx = ui.ctx().clone();
                        if let Some(remote_ui) = state.get_remote_server_ui_mut_lazy() {
                            if let Some(action) = remote_ui.render(ui, &ctx) {
                                if let Err(e) = state.handle_remote_server_action(action, &ctx) {
                                    log::error!("Failed to handle remote server action: {}", e);
                                }
                            }
                        } else {
                            ui.label("远程服务器功能初始化失败");
                        }
                    }
                });
            });

            ui.separator();
        }

        // Session tabs (if multiple sessions)
        state.get_egui_terminal_manager_mut().render_session_tabs(ui);
        if state.session_count() > 1 {
            ui.separator();
        }

        // Main terminal area
        let terminal_rect = ui.available_rect_before_wrap();
        ui.allocate_ui_with_layout(
            terminal_rect.size(),
            eframe::egui::Layout::top_down(eframe::egui::Align::LEFT),
            |ui| {
                state.get_egui_terminal_manager_mut().render_active_session(ui);
            },
        );
    });
}

/// Render export dialog with limited state access to avoid borrowing conflicts
fn render_export_dialog(
    ctx: &eframe::egui::Context,
    export_dialog: &mut ExportDialog,
    terminal_manager: &EguiTerminalManager,
) {
    if !export_dialog.is_open {
        return;
    }

    eframe::egui::Window::new("📤 Export Terminal Output")
        .default_width(500.0)
        .default_height(400.0)
        .resizable(true)
        .collapsible(false)
        .show(ctx, |ui| {
            render_export_dialog_content(ui, export_dialog, terminal_manager);
        });
}

/// Render the export dialog content
fn render_export_dialog_content(
    ui: &mut eframe::egui::Ui,
    export_dialog: &mut ExportDialog,
    terminal_manager: &EguiTerminalManager,
) {
    ui.vertical(|ui| {
        // Export format selection
        ui.group(|ui| {
            ui.label("📋 Export Format:");
            ui.horizontal(|ui| {
                ui.radio_value(&mut export_dialog.options.format, crate::export::ExportFormat::PlainText, "Plain Text");
                ui.radio_value(&mut export_dialog.options.format, crate::export::ExportFormat::Markdown, "Markdown");
                ui.radio_value(&mut export_dialog.options.format, crate::export::ExportFormat::Html, "HTML");
            });
        });

        ui.add_space(10.0);

        // Export options
        ui.group(|ui| {
            ui.label("⚙️ Export Options:");
            ui.checkbox(&mut export_dialog.options.include_metadata, "Include session metadata");
            ui.checkbox(&mut export_dialog.options.include_empty_lines, "Include empty lines");
            ui.checkbox(&mut export_dialog.options.strip_ansi, "Strip ANSI escape sequences");
            ui.checkbox(&mut export_dialog.options.include_line_numbers, "Include line numbers");

            ui.horizontal(|ui| {
                ui.label("Max lines:");
                let mut has_limit = export_dialog.options.max_lines.is_some();
                ui.checkbox(&mut has_limit, "Limit");

                if has_limit {
                    let mut max_lines = export_dialog.options.max_lines.unwrap_or(1000);
                    ui.add(eframe::egui::DragValue::new(&mut max_lines).range(1..=10000));
                    export_dialog.options.max_lines = Some(max_lines);
                } else {
                    export_dialog.options.max_lines = None;
                }
            });
        });

        ui.add_space(10.0);

        // Action buttons
        ui.horizontal(|ui| {
            if ui.button("📋 Copy to Clipboard").clicked() {
                handle_copy_to_clipboard(export_dialog, terminal_manager);
            }

            if ui.button("💾 Save to File").clicked() {
                handle_save_to_file(export_dialog, terminal_manager);
            }

            if ui.button("📝 Export to Note").clicked() {
                handle_export_to_note(export_dialog, terminal_manager);
            }

            ui.with_layout(eframe::egui::Layout::right_to_left(eframe::egui::Align::Center), |ui| {
                if ui.button("❌ Close").clicked() {
                    export_dialog.close();
                }

                if ui.button("👁 Preview").clicked() {
                    handle_preview_export(export_dialog, terminal_manager);
                }
            });
        });

        ui.add_space(10.0);

        // Show error message if any
        if let Some(ref error) = export_dialog.error_message {
            ui.colored_label(eframe::egui::Color32::RED, format!("❌ Error: {}", error));
        }

        // Show export result if requested
        if export_dialog.show_result {
            if let Some(ref result) = export_dialog.last_result {
                ui.separator();
                ui.label("📄 Export Preview:");

                eframe::egui::ScrollArea::vertical()
                    .max_height(200.0)
                    .show(ui, |ui| {
                        ui.add(
                            eframe::egui::TextEdit::multiline(&mut result.content.as_str())
                                .font(eframe::egui::TextStyle::Monospace)
                                .desired_width(f32::INFINITY)
                                .desired_rows(10)
                        );
                    });
            }
        }
    });
}

/// Handle copy to clipboard action
fn handle_copy_to_clipboard(
    export_dialog: &mut ExportDialog,
    terminal_manager: &EguiTerminalManager,
) {
    match terminal_manager.export_active_session(&export_dialog.options) {
        Ok(result) => {
            // Try to copy to clipboard
            match export_dialog.copy_to_clipboard(&result.content) {
                Ok(_) => {
                    export_dialog.last_result = Some(result);
                    export_dialog.error_message = Some("✅ Content copied to clipboard successfully!".to_string());
                    export_dialog.show_result = true;
                    log::info!("Terminal content copied to clipboard successfully");
                }
                Err(e) => {
                    export_dialog.last_result = Some(result);
                    export_dialog.error_message = Some(format!("❌ Failed to copy to clipboard: {}", e));
                    export_dialog.show_result = true;
                    log::error!("Failed to copy to clipboard: {}", e);
                }
            }
        }
        Err(e) => {
            export_dialog.error_message = Some(format!("❌ Export failed: {}", e));
            log::error!("Export failed: {}", e);
        }
    }
}

/// Handle save to file action
fn handle_save_to_file(
    export_dialog: &mut ExportDialog,
    terminal_manager: &EguiTerminalManager,
) {
    match terminal_manager.export_active_session(&export_dialog.options) {
        Ok(result) => {
            // Generate default filename
            let extension = match export_dialog.options.format {
                crate::export::ExportFormat::PlainText => "txt",
                crate::export::ExportFormat::Markdown => "md",
                crate::export::ExportFormat::Html => "html",
            };

            let filename = format!(
                "terminal_export_{}_{}.{}",
                result.metadata.session_title.replace(" ", "_"),
                result.metadata.exported_at.format("%Y%m%d_%H%M%S"),
                extension
            );

            export_dialog.last_result = Some(result);
            export_dialog.error_message = Some(format!("Save as: {}", filename));
            log::info!("Export ready for saving as: {}", filename);
        }
        Err(e) => {
            export_dialog.error_message = Some(e.to_string());
        }
    }
}

/// Handle export to note action
fn handle_export_to_note(
    export_dialog: &mut ExportDialog,
    terminal_manager: &EguiTerminalManager,
) {
    match terminal_manager.export_active_session(&export_dialog.options) {
        Ok(result) => {
            // Create note title
            let note_title = format!(
                "Terminal Export - {} ({})",
                result.metadata.session_title,
                result.metadata.exported_at.format("%Y-%m-%d %H:%M")
            );

            export_dialog.last_result = Some(result);
            export_dialog.error_message = Some(format!("Note would be created: '{}'", note_title));
            log::info!("Export ready for note creation: {}", note_title);
        }
        Err(e) => {
            export_dialog.error_message = Some(e.to_string());
        }
    }
}

/// Handle preview export action
fn handle_preview_export(
    export_dialog: &mut ExportDialog,
    terminal_manager: &EguiTerminalManager,
) {
    match terminal_manager.export_active_session(&export_dialog.options) {
        Ok(result) => {
            export_dialog.last_result = Some(result);
            export_dialog.show_result = true;
            export_dialog.error_message = None;
        }
        Err(e) => {
            export_dialog.error_message = Some(e.to_string());
            export_dialog.show_result = false;
        }
    }
}

/// Handle session history actions
fn handle_session_history_action(state: &mut ITerminalState, action: SessionHistoryAction) {
    match action {
        SessionHistoryAction::SaveSession(session_name) => {
            if let Some(history_manager) = &mut state.session_history_manager {
                // Create a custom saved session with the provided name
                if let Some(session) = state.egui_terminal_manager.get_active_session() {
                    // Get session content
                    let content = match state.egui_terminal_manager.get_active_session_text() {
                        Ok(content) => content,
                        Err(e) => {
                            log::warn!("Failed to get session content for saving: {}", e);
                            format!("Session content unavailable: {}", e)
                        }
                    };

                    // Create saved session with custom name
                    let saved_session = crate::session_history::SavedSession::new(
                        session.id,
                        session_name,
                        session.created_at,
                        session.last_activity,
                        content,
                    );

                    // Save to history
                    match history_manager.save_session(saved_session.clone()) {
                        Ok(_) => {
                            state.session_history_ui.set_success("会话保存成功！".to_string());
                            log::info!("Saved session with custom name: {}", saved_session.title);
                        }
                        Err(e) => {
                            state.session_history_ui.set_error(format!("保存会话失败: {}", e));
                            log::error!("Failed to save session: {}", e);
                        }
                    }
                } else {
                    state.session_history_ui.set_error("没有活动会话可保存".to_string());
                }
            } else {
                state.session_history_ui.set_error("会话历史不可用".to_string());
            }
        }
        SessionHistoryAction::RestoreSession(session_id) => {
            if let Some(history_manager) = &state.session_history_manager {
                if let Some(saved_session) = history_manager.get_session(&session_id) {
                    let saved_session = saved_session.clone();
                    // We need a context for restoration, but we can't get it here easily
                    // For now, we'll create a placeholder context
                    let ctx = eframe::egui::Context::default();
                    match state.egui_terminal_manager.restore_session_from_history(&saved_session, &ctx) {
                        Ok(_) => {
                            // Close the session history dialog after successful restoration
                            state.session_history_ui.close();
                            log::info!("Restored session: {} and closed history dialog", saved_session.title);
                        }
                        Err(e) => {
                            state.session_history_ui.set_error(format!("恢复会话失败: {}", e));
                            log::error!("Failed to restore session: {}", e);
                        }
                    }
                }
            }
        }
        SessionHistoryAction::DeleteSession(session_id) => {
            if let Some(history_manager) = &mut state.session_history_manager {
                match history_manager.delete_session(session_id) {
                    Ok(true) => {
                        state.session_history_ui.set_success("Session deleted successfully!".to_string());
                        log::info!("Deleted session: {}", session_id);
                    }
                    Ok(false) => {
                        state.session_history_ui.set_error("Session not found".to_string());
                    }
                    Err(e) => {
                        state.session_history_ui.set_error(format!("Failed to delete session: {}", e));
                        log::error!("Failed to delete session: {}", e);
                    }
                }
            }
        }
        SessionHistoryAction::UpdateSession { id, title, notes, tags } => {
            if let Some(history_manager) = &mut state.session_history_manager {
                if let Some(session) = history_manager.get_session_mut(&id) {
                    session.title = title;
                    session.notes = notes;
                    session.tags = tags;

                    // Save the updated session
                    let session_clone = session.clone();
                    match history_manager.save_session(session_clone) {
                        Ok(_) => {
                            state.session_history_ui.set_success("Session updated successfully!".to_string());
                            log::info!("Updated session: {}", id);
                        }
                        Err(e) => {
                            state.session_history_ui.set_error(format!("Failed to update session: {}", e));
                            log::error!("Failed to update session: {}", e);
                        }
                    }
                }
            }
        }
        SessionHistoryAction::ClearAllSessions => {
            if let Some(history_manager) = &mut state.session_history_manager {
                let session_ids: Vec<_> = history_manager.get_all_sessions().iter().map(|s| s.id).collect();
                let mut deleted_count = 0;

                for session_id in session_ids {
                    if let Ok(true) = history_manager.delete_session(session_id) {
                        deleted_count += 1;
                    }
                }

                state.session_history_ui.set_success(format!("Deleted {} sessions", deleted_count));
                log::info!("Cleared all sessions: {} deleted", deleted_count);
            }
        }
        SessionHistoryAction::Refresh => {
            if let Some(history_manager) = &mut state.session_history_manager {
                match history_manager.load_all_sessions() {
                    Ok(_) => {
                        state.session_history_ui.set_success("Session list refreshed".to_string());
                        log::info!("Refreshed session history");
                    }
                    Err(e) => {
                        state.session_history_ui.set_error(format!("Failed to refresh: {}", e));
                        log::error!("Failed to refresh session history: {}", e);
                    }
                }
            }
        }
    }
}
