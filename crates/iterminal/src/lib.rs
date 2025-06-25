pub mod state;
pub mod terminal;
pub mod command;
pub mod history;
pub mod config;
pub mod session;

pub use state::ITerminalState;

/// Initialize the iTerminal module
pub fn initialize() -> ITerminalState {
    log::info!("Initializing iTerminal module");
    ITerminalState::new()
}

/// Update function for background tasks
pub fn update_iterminal(state: &mut ITerminalState) {
    // Handle background tasks like command execution, output processing, etc.
    state.update();
}

/// egui terminal interface
pub fn render_iterminal(ui: &mut eframe::egui::Ui, state: &mut ITerminalState) {
    ui.heading("🖥️ Terminal");
    ui.separator();
    ui.add_space(10.0);

    // Show terminal status
    ui.horizontal(|ui| {
        ui.label("状态:");
        ui.label(eframe::egui::RichText::new("运行中").color(eframe::egui::Color32::GREEN));
    });

    ui.add_space(10.0);

    // Show active sessions
    ui.group(|ui| {
        ui.vertical(|ui| {
            ui.label(eframe::egui::RichText::new("活动会话").strong());
            ui.add_space(5.0);

            let session_count = state.terminal_manager.get_all_sessions().len();
            ui.horizontal(|ui| {
                ui.label("会话数量:");
                ui.label(format!("{}", session_count));
            });

            if let Some(active_session) = state.terminal_manager.get_active_session() {
                ui.horizontal(|ui| {
                    ui.label("当前会话:");
                    ui.label(&active_session.title);
                });
            }
        });
    });

    ui.add_space(15.0);

    // Terminal interface
    ui.group(|ui| {
        ui.vertical(|ui| {
            ui.label(eframe::egui::RichText::new("终端界面").strong());
            ui.add_space(5.0);

            // 终端工具栏
            ui.horizontal(|ui| {
                if ui.button("📁 新建会话").clicked() {
                    log::info!("🆕 创建新的终端会话");
                    state.terminal_manager.create_session("新会话".to_string());
                }

                if ui.button("📋 清空").clicked() {
                    log::info!("🧹 清空终端输出");
                    if let Some(session) = state.terminal_manager.get_active_session_mut() {
                        session.clear_output();
                    }
                }

                if ui.button("⚙️ 设置").clicked() {
                    log::info!("⚙️ 打开终端设置");
                }
            });

            ui.add_space(8.0);

            // 主终端区域
            let available_height = ui.available_height() - 80.0; // 为输入区域留出空间

            ui.group(|ui| {
                ui.vertical(|ui| {
                    // 终端头部
                    ui.horizontal(|ui| {
                        ui.colored_label(eframe::egui::Color32::from_rgb(255, 95, 87), "●");
                        ui.colored_label(eframe::egui::Color32::from_rgb(255, 189, 46), "●");
                        ui.colored_label(eframe::egui::Color32::from_rgb(39, 201, 63), "●");
                        ui.add_space(10.0);
                        ui.label(eframe::egui::RichText::new("SeeU Terminal").size(12.0));

                        ui.with_layout(eframe::egui::Layout::right_to_left(eframe::egui::Align::Center), |ui| {
                            ui.label(eframe::egui::RichText::new("🟢 已连接").size(10.0).color(eframe::egui::Color32::GREEN));
                        });
                    });
                    ui.separator();

                    // 终端输出区域
                    eframe::egui::ScrollArea::vertical()
                        .max_height(available_height)
                        .auto_shrink([false, false])
                        .stick_to_bottom(true)
                        .show(ui, |ui| {
                            ui.style_mut().override_text_style = Some(eframe::egui::TextStyle::Monospace);
                            ui.style_mut().visuals.extreme_bg_color = eframe::egui::Color32::from_rgb(30, 30, 30);

                            // 显示实际的终端输出
                            if let Some(session) = state.terminal_manager.get_active_session() {
                                for line in &session.output_buffer {
                                    ui.label(&line.content);
                                }
                            } else {
                                ui.colored_label(eframe::egui::Color32::from_rgb(100, 149, 237), "Welcome to SeeU Terminal!");
                                ui.colored_label(eframe::egui::Color32::GRAY, "Type 'help' for available commands.");
                            }
                        });
                });
            });

            ui.add_space(8.0);

            // 命令输入区域
            ui.horizontal(|ui| {
                ui.label("💬");
                let response = ui.add_sized(
                    [ui.available_width() - 80.0, 25.0],
                    eframe::egui::TextEdit::singleline(&mut state.command_input)
                        .hint_text("输入命令...")
                        .font(eframe::egui::TextStyle::Monospace)
                );

                if ui.button("执行").clicked() || (response.lost_focus() && ui.input(|i| i.key_pressed(eframe::egui::Key::Enter))) {
                    if !state.command_input.trim().is_empty() {
                        log::info!("🚀 执行命令: {}", state.command_input);
                        state.terminal_manager.execute_command(state.command_input.clone());
                        state.command_input.clear();
                    }
                }

                if response.gained_focus() {
                    response.request_focus();
                }
            });
        });
    });

    // Update the state
    state.update();
}


