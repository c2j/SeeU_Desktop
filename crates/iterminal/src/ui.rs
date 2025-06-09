use eframe::egui;
use crate::state::ITerminalState;
use crate::session::OutputLine;

/// Render the main terminal interface
pub fn render_terminal_interface(ui: &mut egui::Ui, state: &mut ITerminalState) {
    // Main terminal layout
    ui.vertical(|ui| {
        // Terminal tabs
        render_terminal_tabs(ui, state);

        ui.separator();

        // Terminal content
        render_terminal_content(ui, state);

        // Input area
        render_input_area(ui, state);
    });

    // History dialog
    if state.show_history {
        render_history_dialog(ui, state);
    }
}

/// Render terminal tabs
fn render_terminal_tabs(ui: &mut egui::Ui, state: &mut ITerminalState) {
    ui.horizontal(|ui| {
        // Collect session info to avoid borrowing conflicts
        let session_info: Vec<(uuid::Uuid, String, bool)> = {
            let sessions = state.terminal_manager.get_all_sessions();
            let active_session_id = if let Some(session) = state.terminal_manager.get_active_session() {
                Some(session.id)
            } else {
                None
            };

            sessions.iter().map(|(session_id, session)| {
                let is_active = Some(*session_id) == active_session_id;
                let tab_text = if session.title.len() > 15 {
                    format!("{}...", &session.title[..12])
                } else {
                    session.title.clone()
                };
                (*session_id, tab_text, is_active)
            }).collect()
        };

        let session_count = session_info.len();

        for (session_id, tab_text, is_active) in session_info {
            if ui.selectable_label(is_active, tab_text).clicked() {
                state.set_active_session(session_id);
            }

            // Close button for tab (except if it's the only one)
            if session_count > 1 {
                if ui.small_button("×").clicked() {
                    state.close_session(session_id);
                }
            }
        }

        // Add new tab button
        if ui.small_button("+").clicked() {
            state.create_session(None);
        }

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            // History button
            if ui.small_button("📜").clicked() {
                state.toggle_history();
            }

            // Clear button
            if ui.small_button("🗑").clicked() {
                if let Some(session) = state.terminal_manager.get_active_session_mut() {
                    session.clear_output();
                }
            }
        });
    });
}

/// Render terminal content (output area)
fn render_terminal_content(ui: &mut egui::Ui, state: &mut ITerminalState) {
    if let Some(session) = state.terminal_manager.get_active_session() {
        let config = state.terminal_manager.get_config();
        let available_height = ui.available_height() - 60.0; // Reserve space for input

        egui::ScrollArea::vertical()
            .stick_to_bottom(session.scroll_position == 0)
            .auto_shrink([false; 2])
            .max_height(available_height)
            .show(ui, |ui| {
                // Calculate how many lines can fit
                let line_height = ui.text_style_height(&egui::TextStyle::Monospace);
                let max_visible_lines = (available_height / line_height) as usize;

                let visible_lines = session.get_visible_output(max_visible_lines);

                for line in visible_lines {
                    render_output_line(ui, line, config, state.font_scale);
                }

                // Add some spacing at the bottom
                ui.add_space(10.0);
            });
    } else {
        ui.centered_and_justified(|ui| {
            ui.label("No terminal session available");
        });
    }
}

/// Render a single output line
fn render_output_line(ui: &mut egui::Ui, line: &OutputLine, config: &crate::config::TerminalConfig, font_scale: f32) {
    let mut job = egui::text::LayoutJob::default();

    // Set font
    let font_id = egui::FontId::monospace(config.font_size * font_scale);

    // Determine color based on line type and style
    let color = if line.is_input {
        egui::Color32::from_rgba_premultiplied(
            (config.text_color[0] * 255.0) as u8,
            (config.text_color[1] * 255.0) as u8,
            (config.text_color[2] * 255.0) as u8,
            (config.text_color[3] * 255.0) as u8,
        )
    } else {
        // Use style info if available
        if let Some(style) = &line.style_info {
            if let Some(fg_color) = style.fg_color {
                egui::Color32::from_rgb(fg_color[0], fg_color[1], fg_color[2])
            } else {
                egui::Color32::from_rgba_premultiplied(
                    (config.text_color[0] * 255.0) as u8,
                    (config.text_color[1] * 255.0) as u8,
                    (config.text_color[2] * 255.0) as u8,
                    (config.text_color[3] * 255.0) as u8,
                )
            }
        } else {
            egui::Color32::GRAY
        }
    };

    // Add text with formatting
    job.append(
        &line.content,
        0.0,
        egui::text::TextFormat {
            font_id,
            color,
            ..Default::default()
        },
    );

    ui.label(job);
}

/// Render input area
fn render_input_area(ui: &mut egui::Ui, state: &mut ITerminalState) {
    // Get current input first to avoid borrowing conflicts
    let current_input = if let Some(session) = state.terminal_manager.get_active_session() {
        session.current_input.clone()
    } else {
        return;
    };

    ui.horizontal(|ui| {
        // Prompt
        ui.label("$");

        // Input field
        let mut input_text = current_input;
        let response = ui.add(
            egui::TextEdit::singleline(&mut input_text)
                .font(egui::TextStyle::Monospace)
                .desired_width(ui.available_width() - 20.0)
                .hint_text("输入命令...")
        );

        // Update session input if changed
        if let Some(session_mut) = state.terminal_manager.get_active_session_mut() {
            if input_text != session_mut.current_input {
                session_mut.current_input = input_text;
                session_mut.cursor_position = session_mut.current_input.len();
            }
        }

        // Handle Enter key for command execution
        if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
            if let Some(session_mut) = state.terminal_manager.get_active_session_mut() {
                let command = session_mut.execute_current_input();
                if !command.trim().is_empty() {
                    let session_id = session_mut.id;
                    // Drop the mutable borrow before calling execute_command_in_session
                    let _ = session_mut;
                    state.terminal_manager.execute_command_in_session(session_id, command);
                }
            }
        }

        // Handle focus
        if response.gained_focus() {
            state.has_focus = true;
        }
        if response.lost_focus() {
            state.has_focus = false;
        }

        // Auto-focus the input
        if !state.has_focus {
            response.request_focus();
        }
    });
}




/// Render history dialog
fn render_history_dialog(ui: &mut egui::Ui, state: &mut ITerminalState) {
    let mut show_history = state.show_history;
    let mut history_search = state.history_search.clone();
    let mut command_to_execute: Option<String> = None;
    let mut command_to_copy: Option<String> = None;

    // Collect history entries to avoid borrowing conflicts
    let history_entries = if let Some(session) = state.terminal_manager.get_active_session() {
        if history_search.is_empty() {
            session.history.get_entries().iter().cloned().collect::<Vec<_>>()
        } else {
            session.history.search(&history_search).into_iter().cloned().collect::<Vec<_>>()
        }
    } else {
        Vec::new()
    };

    let window_response = egui::Window::new("Command History")
        .open(&mut show_history)
        .default_width(500.0)
        .default_height(400.0)
        .show(ui.ctx(), |ui| {
            // Search box
            ui.horizontal(|ui| {
                ui.label("Search:");
                ui.text_edit_singleline(&mut history_search);
            });

            ui.separator();

            // History list
            egui::ScrollArea::vertical().show(ui, |ui| {
                for entry in history_entries.iter().rev() {
                    ui.horizontal(|ui| {
                        if ui.small_button("📋").clicked() {
                            command_to_copy = Some(entry.command.clone());
                        }

                        if ui.small_button("▶").clicked() {
                            command_to_execute = Some(entry.command.clone());
                        }

                        ui.label(&entry.command);

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(entry.timestamp.format("%H:%M:%S").to_string());
                            if let Some(exit_code) = entry.exit_code {
                                let color = if exit_code == 0 {
                                    egui::Color32::GREEN
                                } else {
                                    egui::Color32::RED
                                };
                                ui.colored_label(color, format!("({})", exit_code));
                            }
                        });
                    });
                    ui.separator();
                }

                if history_entries.is_empty() {
                    ui.label("No commands in history");
                }
            });
        });

    // Update state after the window
    if let Some(_) = window_response {
        state.show_history = show_history;
        state.history_search = history_search;

        // Handle actions
        if let Some(command) = command_to_execute {
            state.execute_command(command);
            state.show_history = false;
        }

        if let Some(command) = command_to_copy {
            ui.ctx().copy_text(command);
        }
    }
}
