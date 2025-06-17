use eframe::egui;
use crate::state::{AIAssistState, MessageRole};

/// Render the AI assistant UI
pub fn render_ai_assist(ui: &mut egui::Ui, state: &mut AIAssistState) {
    // 检查是否有来自异步任务的更新
    state.check_for_updates();

    // 请求重绘以保持流式输出的更新
    if state.is_sending {
        ui.ctx().request_repaint();
    }
    // 获取可用高度，减去状态栏的高度（约30像素）以避免遮挡
    let available_height = ui.available_height() - 30.0;

    // 创建一个垂直布局容器，确保内容撑满高度但不超出
    egui::containers::Frame::NONE
        .fill(ui.style().visuals.window_fill)
        .show(ui, |ui| {
            // 设置最大高度，确保不会遮挡状态栏
            ui.set_max_height(available_height);

            ui.vertical(|ui| {
                // 顶部标题栏
                ui.horizontal(|ui| {
                    ui.heading("SeeU AI助手");

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("设置").clicked() {
                            state.show_ai_settings = true;
                        }
                    });
                });

                ui.separator();

                // 历史会话下拉菜单和新建会话按钮
                ui.horizontal(|ui| {
                    // 历史会话下拉菜单
                    let dropdown_button = egui::Button::new(
                        if state.show_history_dropdown { "历史消息 ▲" } else { "历史消息 ▼" }
                    );

                    if ui.add(dropdown_button).clicked() {
                        state.show_history_dropdown = !state.show_history_dropdown;
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("+ 新消息").clicked() {
                            state.create_new_session();
                        }
                    });
                });

                // 显示历史会话下拉菜单
                if state.show_history_dropdown {
                    egui::Frame::NONE
                        .fill(ui.style().visuals.window_fill)
                        .stroke(egui::Stroke::new(1.0, ui.style().visuals.widgets.noninteractive.bg_stroke.color))
                        .show(ui, |ui| {
                            egui::ScrollArea::vertical().max_height(300.0).show(ui, |ui| {
                                let mut selected_idx = None;
                                let mut delete_idx = None;

                                for (idx, session) in state.chat_sessions.iter().enumerate() {
                                    let is_active = idx == state.active_session_idx;

                                    ui.horizontal(|ui| {
                                        // Session name (clickable)
                                        let session_response = ui.selectable_label(is_active, &session.name);
                                        if session_response.clicked() {
                                            selected_idx = Some(idx);
                                        }

                                        // Show session creation time on hover
                                        if session_response.hovered() {
                                            session_response.on_hover_text(format!(
                                                "创建时间: {}\n消息数量: {}",
                                                session.created_at.format("%Y-%m-%d %H:%M"),
                                                session.messages.len()
                                            ));
                                        }

                                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                            // Delete button (only show if more than one session)
                                            if state.chat_sessions.len() > 1 {
                                                if ui.small_button("🗑").clicked() {
                                                    delete_idx = Some(idx);
                                                }
                                            }
                                        });
                                    });

                                    ui.separator();
                                }

                                // Handle session selection outside the loop to avoid borrowing issues
                                if let Some(idx) = selected_idx {
                                    state.switch_session(idx);
                                    state.show_history_dropdown = false;
                                }

                                // Handle session deletion
                                if let Some(idx) = delete_idx {
                                    state.delete_session(idx);
                                }
                            });
                        });
                }

                ui.separator();

                // 聊天消息区域
                let chat_height = available_height - 120.0; // 减去标题栏、工具栏和输入框的高度
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .stick_to_bottom(true)
                    .max_height(chat_height)
                    .show(ui, |ui| {
                        for message in &state.chat_messages {
                            let is_user = message.role == MessageRole::User;
                            let is_slash_command = message.role == MessageRole::SlashCommand;
                            let is_system = message.role == MessageRole::System;

                            // 创建一个垂直布局，确保消息内容可以自动换行
                            ui.vertical(|ui| {
                                // 获取当前主题的颜色
                                let visuals = &ui.style().visuals;

                                // 消息框的背景色
                                let frame_fill = if is_user {
                                    if visuals.dark_mode {
                                        egui::Color32::from_rgba_premultiplied(45, 85, 135, 200)  // 深蓝色背景（深色主题）
                                    } else {
                                        egui::Color32::from_rgba_premultiplied(240, 240, 255, 200)  // 浅蓝色背景（浅色主题）
                                    }
                                } else if is_slash_command {
                                    if visuals.dark_mode {
                                        egui::Color32::from_rgba_premultiplied(85, 65, 45, 200)  // 深橙色背景（深色主题）
                                    } else {
                                        egui::Color32::from_rgba_premultiplied(255, 245, 230, 200)  // 浅橙色背景（浅色主题）
                                    }
                                } else if is_system {
                                    if visuals.dark_mode {
                                        egui::Color32::from_rgba_premultiplied(65, 75, 65, 200)  // 深绿色背景（深色主题）
                                    } else {
                                        egui::Color32::from_rgba_premultiplied(240, 255, 240, 200)  // 浅绿色背景（浅色主题）
                                    }
                                } else {
                                    if visuals.dark_mode {
                                        egui::Color32::from_rgba_premultiplied(55, 55, 55, 200)  // 深灰色背景（深色主题）
                                    } else {
                                        egui::Color32::from_rgba_premultiplied(240, 255, 240, 200)  // 浅灰色背景（浅色主题）
                                    }
                                };

                                // 创建圆角方框
                                let frame = egui::Frame::NONE
                                    .fill(frame_fill)
                                    .corner_radius(egui::Rounding::same(8))
                                    .inner_margin(egui::Margin::same(10))
                                    .stroke(egui::Stroke::new(1.0, ui.style().visuals.widgets.noninteractive.bg_stroke.color));

                                frame.show(ui, |ui| {
                                    // 获取当前主题的文字颜色
                                    let visuals = &ui.style().visuals;
                                    let text_color = if visuals.dark_mode {
                                        egui::Color32::WHITE
                                    } else {
                                        egui::Color32::BLACK
                                    };

                                    if is_user {
                                        // 用户消息右对齐
                                        ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                                            ui.horizontal(|ui| {
                                                // 时间戳
                                                ui.label(egui::RichText::new(message.format_timestamp())
                                                    .size(10.0)
                                                    .color(egui::Color32::GRAY));

                                                // 用户头像 - 尝试加载图片，失败则显示emoji
                                                if ui.ctx().try_load_texture("file://assets/icons/u.png", Default::default(), egui::SizeHint::default()).is_ok() {
                                                    ui.add(egui::Image::from_uri("file://assets/icons/u.png").max_size(egui::Vec2::splat(16.0)));
                                                } else {
                                                    ui.label("👤");
                                                }

                                                ui.label(egui::RichText::new("用户: ").strong().color(text_color));
                                            });
                                        });

                                        // 用户消息内容
                                        ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                                            let user_text = egui::RichText::new(&message.content)
                                                .strong()
                                                .color(text_color);
                                            ui.add(egui::Label::new(user_text).wrap());
                                        });
                                    } else if is_slash_command {
                                        // Slash指令消息左对齐，使用特殊标识
                                        ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP), |ui| {
                                            ui.label(egui::RichText::new("指令: ").strong().color(text_color));
                                        });

                                        // Slash指令内容
                                        ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP), |ui| {
                                            let command_text = egui::RichText::new(&message.content)
                                                .monospace()  // 使用等宽字体
                                                .color(text_color);
                                            ui.add(egui::Label::new(command_text).wrap());
                                        });
                                    } else if is_system {
                                        // 系统消息居中对齐
                                        ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP), |ui| {
                                            ui.label(egui::RichText::new("系统: ").strong().color(text_color));
                                        });

                                        // 系统消息内容
                                        ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP), |ui| {
                                            let system_text = egui::RichText::new(&message.content)
                                                .color(text_color);
                                            ui.add(egui::Label::new(system_text).wrap());
                                        });
                                    } else {
                                        // 检查是否是正在流式输出的消息
                                        let is_streaming = state.streaming_message_id.map_or(false, |id| id == message.id);

                                        // AI消息左对齐
                                        ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP), |ui| {
                                            ui.horizontal(|ui| {
                                                // AI头像 - 尝试加载图片，失败则显示emoji
                                                if ui.ctx().try_load_texture("file://assets/icons/c-see.png", Default::default(), egui::SizeHint::default()).is_ok() {
                                                    ui.add(egui::Image::from_uri("file://assets/icons/c-see.png").max_size(egui::Vec2::splat(16.0)));
                                                } else {
                                                    ui.label("👁️");
                                                }

                                                ui.label(egui::RichText::new("SeeU AI: ").strong().color(text_color));

                                                // 时间戳
                                                ui.label(egui::RichText::new(message.format_timestamp())
                                                    .size(10.0)
                                                    .color(egui::Color32::GRAY));
                                            });
                                        });

                                        // AI消息内容
                                        ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP), |ui| {
                                            // 设置最大宽度，确保文本自动换行
                                            let available_width = ui.available_width();

                                            // 如果是流式输出中的消息，显示动画效果
                                            if is_streaming && state.is_sending {
                                                let text = &message.content;

                                                // 处理消息内容
                                                render_formatted_message(ui, text, available_width - 10.0, true, text_color);

                                                // 添加闪烁的光标
                                                let cursor = if (ui.input(|i| i.time) * 2.0).sin() > 0.0 { "▋" } else { " " };
                                                ui.label(egui::RichText::new(cursor).color(text_color));
                                            } else {
                                                // 处理消息内容
                                                render_formatted_message(ui, &message.content, available_width - 10.0, false, text_color);
                                            }
                                        });
                                    }
                                });

                                // 添加复制按钮和插入到笔记按钮
                                ui.horizontal(|ui| {
                                    // 检查是否是正在流式输出的消息
                                    let is_streaming = state.streaming_message_id.map_or(false, |id| id == message.id);

                                    if is_user {
                                        // 用户消息的复制按钮放在右下方
                                        ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                                            if ui.button("📋 复制").clicked() {
                                                // 复制消息内容到剪贴板
                                                ui.ctx().copy_text(message.content.clone());

                                                // 可以添加一个提示，表示已复制
                                                log::info!("已复制消息到剪贴板");
                                            }
                                        });
                                    } else if !is_streaming || !state.is_sending {
                                        // AI消息的复制按钮和插入按钮放在左下方，只有在消息完全接收完毕后才显示
                                        ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP), |ui| {
                                            if ui.button("📋 复制").clicked() {
                                                // 复制消息内容到剪贴板
                                                ui.ctx().copy_text(message.content.clone());

                                                // 可以添加一个提示，表示已复制
                                                log::info!("已复制消息到剪贴板");
                                            }

                                            // 如果有插入到笔记的回调函数，且当前处于笔记视图且有打开的笔记，显示插入按钮
                                            if state.insert_to_note_callback.is_some() && state.can_insert_to_note {
                                                ui.add_space(4.0);
                                                if ui.button("📝 插入到笔记").clicked() {
                                                    // 调用回调函数，将消息内容插入到笔记
                                                    if let Some(callback) = &mut state.insert_to_note_callback {
                                                        callback(message.content.clone());
                                                        log::info!("已插入消息到笔记");
                                                    }
                                                }
                                            }
                                        });
                                    }
                                });
                            });

                            ui.add_space(8.0);
                        }
                    });

                ui.separator();

                // 输入区域 - 使用垂直布局确保输入框占满宽度
                ui.vertical(|ui| {
                    // 设置输入框的最大宽度
                    let available_width = ui.available_width();

                    let response = ui.add(
                        egui::TextEdit::multiline(&mut state.chat_input)
                            .hint_text("输入消息...")
                            .desired_width(available_width) // 使用全部可用宽度
                            .desired_rows(2)
                            // 如果正在发送，禁用输入框
                            .interactive(!state.is_sending)
                    );

                    // 如果需要聚焦，则请求焦点
                    if state.should_focus_chat {
                        response.request_focus();
                        state.should_focus_chat = false;
                    }

                    // 获取光标位置并更新指令菜单
                    let cursor_pos = if response.has_focus() {
                        // 尝试获取光标在屏幕上的位置
                        Some(response.rect.left_bottom())
                    } else {
                        None
                    };

                    // 更新指令菜单状态
                    state.update_command_menu(cursor_pos);

                    // 处理键盘输入 - 首先检查菜单是否需要处理
                    let mut menu_handled = false;

                    if state.command_menu.is_visible && response.has_focus() {
                        // 当菜单可见时，优先处理菜单相关的键盘事件
                        ui.input_mut(|i| {
                            if i.key_pressed(egui::Key::ArrowUp) {
                                menu_handled = true;
                                state.handle_command_menu_input(egui::Key::ArrowUp);
                            } else if i.key_pressed(egui::Key::ArrowDown) {
                                menu_handled = true;
                                state.handle_command_menu_input(egui::Key::ArrowDown);
                            } else if i.key_pressed(egui::Key::Enter) && !i.modifiers.alt {
                                // 消费回车键事件，防止被正常发送逻辑处理
                                i.consume_key(egui::Modifiers::NONE, egui::Key::Enter);
                                menu_handled = true;
                                state.handle_command_menu_input(egui::Key::Enter);
                            } else if i.key_pressed(egui::Key::Tab) {
                                // 消费Tab键事件，防止焦点转移
                                i.consume_key(egui::Modifiers::NONE, egui::Key::Tab);
                                menu_handled = true;
                                state.handle_command_menu_input(egui::Key::Tab);
                            } else if i.key_pressed(egui::Key::Escape) {
                                menu_handled = true;
                                state.handle_command_menu_input(egui::Key::Escape);
                            }
                        });
                    }

                    // 只有在菜单没有处理输入时，才处理正常的键盘输入
                    if !menu_handled && response.has_focus() && !state.is_sending {
                        let enter_pressed = ui.input(|i| i.key_pressed(egui::Key::Enter));
                        let alt_pressed = ui.input(|i| i.modifiers.alt);

                        if enter_pressed {
                            if alt_pressed {
                                // Alt+回车：添加换行符
                                state.chat_input.push('\n');
                            } else {
                                // 单独的回车：发送消息
                                if let Some(cmd) = state.send_message() {
                                    // Return the slash command to be handled by the parent
                                    if let Some(callback) = &mut state.slash_command_callback {
                                        callback(cmd);
                                    }
                                }
                            }
                        }
                    }
                });

                // 底部工具栏
                ui.horizontal(|ui| {
                    // 模型名称显示
                    let display_text = if state.ai_settings.model.len() > 15 {
                        format!("{}...", &state.ai_settings.model[..12])
                    } else {
                        state.ai_settings.model.clone()
                    };

                    ui.label(format!("模型: {}", display_text));

                    // 附件按钮
                    if ui.button("📎").clicked() {
                        state.show_attachment_dialog = true;
                    }

                    // @命令按钮
                    if ui.button("@").clicked() {
                        // 切换@命令提示框的显示状态
                        state.show_at_commands = !state.show_at_commands;
                        // 关闭Slash命令提示框
                        state.show_slash_commands = false;
                    }

                    // Slash命令按钮
                    if ui.button("/").clicked() {
                        // 切换Slash命令提示框的显示状态
                        state.show_slash_commands = !state.show_slash_commands;
                        // 关闭@命令提示框
                        state.show_at_commands = false;
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // 如果正在发送，显示加载动画
                        if state.is_sending {
                            ui.spinner();
                        } else {
                            if ui.button("✈发送").clicked() {
                                if let Some(cmd) = state.send_message() {
                                    // Return the slash command to be handled by the parent
                                    if let Some(callback) = &mut state.slash_command_callback {
                                        callback(cmd);
                                    }
                                }
                            }
                        }

                        // MCP服务器选择器
                        ui.separator();
                        ui.label("MCP:");

                        let selected_text = if let Some(server_id) = state.selected_mcp_server {
                            // 尝试从server_names中获取服务器名称
                            if let Some(server_name) = state.server_names.get(&server_id) {
                                format!("🟢 {}", server_name)
                            } else if let Some(_capabilities) = state.mcp_server_capabilities.get(&server_id) {
                                format!("🟢 服务器 {}", server_id.to_string().chars().take(8).collect::<String>())
                            } else {
                                format!("服务器 {}", server_id.to_string().chars().take(8).collect::<String>())
                            }
                        } else {
                            "无".to_string()
                        };

                        egui::ComboBox::from_label("")
                            .selected_text(selected_text)
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut state.selected_mcp_server, None, "无");

                                // 只显示有capabilities的服务器（表示已测试通过的绿灯服务器）
                                for (server_id, capabilities) in &state.mcp_server_capabilities {
                                    let tool_count = capabilities.tools.len();
                                    let resource_count = capabilities.resources.len();
                                    let prompt_count = capabilities.prompts.len();

                                    // 获取服务器名称，如果没有则使用UUID前8位
                                    let server_name = state.server_names.get(server_id)
                                        .cloned()
                                        .unwrap_or_else(|| format!("服务器 {}", server_id.to_string().chars().take(8).collect::<String>()));

                                    let server_display = format!(
                                        "🟢 {} (工具:{} 资源:{} 提示:{})",
                                        server_name,
                                        tool_count,
                                        resource_count,
                                        prompt_count
                                    );

                                    ui.selectable_value(&mut state.selected_mcp_server, Some(*server_id), server_display);
                                }

                                // 如果没有可用的MCP服务器，显示提示
                                if state.mcp_server_capabilities.is_empty() {
                                    ui.colored_label(egui::Color32::GRAY, "💡 请在iTools中配置并测试MCP服务器");
                                }
                            });
                    });
                });
            });
        });

    // 显示设置对话框
    if state.show_ai_settings {
        render_ai_settings(ui.ctx(), state);
    }

    // 显示智能指令菜单
    if state.command_menu.is_visible {
        render_smart_command_menu(ui.ctx(), state);
    }

    // 显示@命令提示框
    if state.show_at_commands {
        render_at_commands(ui.ctx(), state);
    }

    // 显示Slash命令提示框
    if state.show_slash_commands {
        render_slash_commands(ui.ctx(), state);
    }

    // 显示工具调用确认对话框
    if state.show_tool_call_confirmation {
        render_tool_call_confirmation(ui.ctx(), state);
    }

    // 显示MCP状态信息面板
    render_mcp_status_panel(ui, state);
}

/// Render AI settings dialog
fn render_ai_settings(ctx: &egui::Context, state: &mut AIAssistState) {
    let mut open = true;

    // 获取屏幕中心位置
    let screen_rect = ctx.screen_rect();
    let window_size = egui::vec2(400.0, 350.0);

    // 计算窗口位置 - 放在屏幕右上角附近
    let window_pos = egui::pos2(
        screen_rect.right() - window_size.x - 20.0,
        screen_rect.top() + 50.0
    );

    egui::Window::new("AI助手设置")
        .open(&mut open)
        .resizable(false)
        .fixed_size(window_size)
        .current_pos(window_pos)
        .show(ctx, |ui| {
            ui.heading("AI助手设置");
            ui.separator();

            ui.horizontal(|ui| {
                ui.label("Base URL:");
                ui.text_edit_singleline(&mut state.ai_settings.base_url);
            });
            ui.label("提示: 通常以 /v1 结尾，如 http://localhost:11434/v1");

            ui.add_space(10.0);

            ui.horizontal(|ui| {
                ui.label("API Key:");
                ui.text_edit_singleline(&mut state.ai_settings.api_key);
            });
            ui.label("提示: 本地服务可以留空");

            ui.add_space(10.0);

            ui.horizontal(|ui| {
                ui.label("模型名称:");
                ui.text_edit_singleline(&mut state.ai_settings.model);
            });
            ui.label("提示: 如 gpt-3.5-turbo, qwen2.5:7b 等");

            ui.separator();

            ui.horizontal(|ui| {
                ui.label("温度:");
                ui.add(egui::Slider::new(&mut state.ai_settings.temperature, 0.0..=1.0));
            });

            ui.horizontal(|ui| {
                ui.label("最大Token:");
                ui.add(egui::Slider::new(&mut state.ai_settings.max_tokens, 100..=4000));
            });

            ui.checkbox(&mut state.ai_settings.streaming, "启用流式输出");

            ui.separator();

            ui.horizontal(|ui| {
                if ui.button("保存").clicked() {
                    state.show_ai_settings = false;
                }

                if ui.button("取消").clicked() {
                    state.show_ai_settings = false;
                }
            });
        });

    if !open {
        state.show_ai_settings = false;
    }
}

/// 渲染智能指令菜单
fn render_smart_command_menu(ctx: &egui::Context, state: &mut AIAssistState) {
    use crate::state::{CommandMenuType};

    if !state.command_menu.is_visible {
        return;
    }

    let commands = match state.command_menu.menu_type {
        CommandMenuType::AtCommands => vec![
            ("@search", "引用最近搜索的第一条结果详细内容"),
            ("@date", "插入当前日期"),
            ("@time", "插入当前时间"),
            ("@user", "引用当前用户"),
        ],
        CommandMenuType::SlashCommands => vec![
            ("/search", "执行搜索"),
            ("/clear", "清空当前会话"),
            ("/help", "显示帮助信息"),
            ("/new", "创建新会话"),
        ],
        CommandMenuType::None => return,
    };

    // 计算菜单位置
    let menu_pos = if let Some(cursor_pos) = state.command_menu.cursor_position {
        egui::pos2(cursor_pos.x, cursor_pos.y + 20.0) // 在光标下方显示
    } else {
        // 回退到屏幕中央
        let screen_rect = ctx.screen_rect();
        egui::pos2(screen_rect.center().x - 150.0, screen_rect.center().y)
    };

    let menu_size = egui::vec2(300.0, (commands.len() as f32 * 25.0 + 40.0).min(200.0));

    egui::Window::new("指令菜单")
        .title_bar(false)
        .resizable(false)
        .fixed_size(menu_size)
        .current_pos(menu_pos)
        .frame(egui::Frame::popup(&ctx.style()))
        .show(ctx, |ui| {
            ui.vertical(|ui| {
                for (i, (command, description)) in commands.iter().enumerate() {
                    let is_selected = i == state.command_menu.selected_index;

                    // 使用不同的样式来突出显示选中项
                    let response = if is_selected {
                        ui.add(
                            egui::Label::new(
                                egui::RichText::new(format!("{} - {}", command, description))
                                    .background_color(egui::Color32::from_rgb(100, 150, 255))
                                    .color(egui::Color32::WHITE)
                            )
                            .sense(egui::Sense::click())
                        )
                    } else {
                        ui.add(
                            egui::Label::new(format!("{} - {}", command, description))
                                .sense(egui::Sense::click())
                        )
                    };

                    if response.clicked() {
                        state.command_menu.selected_index = i;
                        state.apply_selected_command_to_input();
                    }

                    // 如果是选中项，确保它在视图中可见
                    if is_selected {
                        response.scroll_to_me(Some(egui::Align::Center));
                    }
                }
            });
        });
}

/// 渲染@命令提示框
fn render_at_commands(ctx: &egui::Context, state: &mut AIAssistState) {
    let mut open = true;

    // 获取屏幕信息
    let screen_rect = ctx.screen_rect();
    let window_size = egui::vec2(300.0, 200.0);

    // 计算窗口位置 - 放在底部工具栏上方，@按钮附近
    let window_pos = egui::pos2(
        screen_rect.right() - 250.0, // 大约在@按钮上方
        screen_rect.bottom() - window_size.y - 80.0 // 在底部工具栏上方
    );

    egui::Window::new("@命令列表")
        .open(&mut open)
        .resizable(false)
        .fixed_size(window_size)
        .current_pos(window_pos)
        .show(ctx, |ui| {
            ui.heading("支持的@命令");
            ui.separator();

            ui.label("@search - 引用最近搜索的第一条结果详细内容");
            ui.label("@date - 插入当前日期");
            ui.label("@time - 插入当前时间");
            ui.label("@user - 引用当前用户");

            ui.separator();
            ui.label("点击命令将其插入到输入框");

            // 点击命令将其插入到输入框
            if ui.button("插入 @search").clicked() {
                state.chat_input.push_str("@search");
                state.show_at_commands = false;
            }

            if ui.button("插入 @date").clicked() {
                state.chat_input.push_str("@date");
                state.show_at_commands = false;
            }

            if ui.button("插入 @time").clicked() {
                state.chat_input.push_str("@time");
                state.show_at_commands = false;
            }

            if ui.button("插入 @user").clicked() {
                state.chat_input.push_str("@user");
                state.show_at_commands = false;
            }
        });

    if !open {
        state.show_at_commands = false;
    }
}

/// 渲染Slash命令提示框
fn render_slash_commands(ctx: &egui::Context, state: &mut AIAssistState) {
    let mut open = true;

    // 获取屏幕信息
    let screen_rect = ctx.screen_rect();
    let window_size = egui::vec2(300.0, 200.0);

    // 计算窗口位置 - 放在底部工具栏上方，/按钮附近
    let window_pos = egui::pos2(
        screen_rect.right() - 200.0, // 大约在/按钮上方
        screen_rect.bottom() - window_size.y - 80.0 // 在底部工具栏上方
    );

    egui::Window::new("Slash命令列表")
        .open(&mut open)
        .resizable(false)
        .fixed_size(window_size)
        .current_pos(window_pos)
        .show(ctx, |ui| {
            ui.heading("支持的Slash命令");
            ui.separator();

            ui.label("/search [查询] - 执行搜索");
            ui.label("/clear - 清空当前会话");
            ui.label("/help - 显示帮助信息");
            ui.label("/new - 创建新会话");

            ui.separator();
            ui.label("点击命令将其插入到输入框");

            // 点击命令将其插入到输入框
            if ui.button("插入 /search").clicked() {
                state.chat_input.push_str("/search ");
                state.show_slash_commands = false;
            }

            if ui.button("插入 /clear").clicked() {
                state.chat_input.push_str("/clear");
                state.show_slash_commands = false;
            }

            if ui.button("插入 /help").clicked() {
                state.chat_input.push_str("/help");
                state.show_slash_commands = false;
            }

            if ui.button("插入 /new").clicked() {
                state.chat_input.push_str("/new");
                state.show_slash_commands = false;
            }
        });

    if !open {
        state.show_slash_commands = false;
    }
}

/// 渲染工具调用确认对话框
fn render_tool_call_confirmation(ctx: &egui::Context, state: &mut AIAssistState) {
    let mut open = true;

    let response = egui::Window::new("MCP 工具调用确认")
        .open(&mut open)
        .resizable(true)
        .default_width(600.0)
        .default_height(400.0)
        .show(ctx, |ui| {
            ui.heading("AI 助手请求执行以下工具:");
            ui.separator();

            if let Some(batch) = &state.current_tool_call_batch {
                ui.label(format!("共 {} 个工具调用请求", batch.tool_calls.len()));
                ui.add_space(10.0);

                egui::ScrollArea::vertical().show(ui, |ui| {
                    for (index, pending_call) in batch.tool_calls.iter().enumerate() {
                        ui.group(|ui| {
                            ui.vertical(|ui| {
                                ui.horizontal(|ui| {
                                    ui.label(egui::RichText::new(format!("{}. 工具:", index + 1)).strong());
                                    ui.label(egui::RichText::new(&pending_call.tool_call.function.name).color(egui::Color32::BLUE));
                                });

                                ui.horizontal(|ui| {
                                    ui.label("服务器:");
                                    ui.label(&pending_call.server_name);
                                });

                                ui.horizontal(|ui| {
                                    ui.label("参数:");
                                    ui.label(&pending_call.tool_call.function.arguments);
                                });

                                ui.horizontal(|ui| {
                                    ui.label("类型:");
                                    let call_type = match pending_call.mcp_info.call_type {
                                        crate::mcp_tools::McpCallType::CallTool => "工具调用",
                                        crate::mcp_tools::McpCallType::ReadResource => "读取资源",
                                        crate::mcp_tools::McpCallType::GetPrompt => "获取提示",
                                    };
                                    ui.label(call_type);
                                });
                            });
                        });
                        ui.add_space(5.0);
                    }
                });

                ui.separator();
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("⚠️ 警告:").color(egui::Color32::YELLOW));
                    ui.label("执行这些工具可能会修改系统状态或访问敏感信息。请仔细检查后再确认。");
                });

                ui.add_space(10.0);
                ui.horizontal(|ui| {
                    if ui.button(egui::RichText::new("✅ 确认执行").color(egui::Color32::GREEN)).clicked() {
                        state.approve_tool_calls();
                    }

                    if ui.button(egui::RichText::new("❌ 拒绝执行").color(egui::Color32::RED)).clicked() {
                        state.reject_tool_calls();
                    }

                    ui.separator();
                    if ui.button("取消").clicked() {
                        state.show_tool_call_confirmation = false;
                    }
                });
            } else {
                ui.label("没有待处理的工具调用");
            }
        });

    if !open {
        state.show_tool_call_confirmation = false;
    }
}

/// 渲染MCP状态面板
fn render_mcp_status_panel(ui: &mut egui::Ui, state: &AIAssistState) {
    if let Some(server_id) = state.selected_mcp_server {
        if let Some(capabilities) = state.mcp_server_capabilities.get(&server_id) {
            ui.separator();

            // 获取服务器名称
            let server_name = state.server_names.get(&server_id)
                .cloned()
                .unwrap_or_else(|| format!("服务器 {}", server_id.to_string().chars().take(8).collect::<String>()));

            egui::CollapsingHeader::new(format!("🟢 MCP服务器状态 - {}", server_name))
                .default_open(false)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("服务器名称:");
                        ui.colored_label(egui::Color32::BLUE, &server_name);
                    });

                    ui.horizontal(|ui| {
                        ui.label("服务器ID:");
                        ui.monospace(server_id.to_string().chars().take(8).collect::<String>());
                    });

                    ui.horizontal(|ui| {
                        ui.label("状态:");
                        ui.colored_label(egui::Color32::GREEN, "🟢 已测试通过，可用于工具调用");
                    });

                    ui.separator();

                    // 工具统计
                    ui.horizontal(|ui| {
                        ui.label("可用工具:");
                        ui.colored_label(egui::Color32::BLUE, format!("{} 个", capabilities.tools.len()));

                        ui.separator();

                        ui.label("资源:");
                        ui.colored_label(egui::Color32::BLUE, format!("{} 个", capabilities.resources.len()));

                        ui.separator();

                        ui.label("提示:");
                        ui.colored_label(egui::Color32::BLUE, format!("{} 个", capabilities.prompts.len()));
                    });

                    // 工具详情
                    if !capabilities.tools.is_empty() {
                        ui.separator();
                        ui.label(egui::RichText::new("工具列表:").strong());

                        egui::ScrollArea::vertical()
                            .max_height(100.0)
                            .show(ui, |ui| {
                                for (index, tool) in capabilities.tools.iter().enumerate() {
                                    ui.horizontal(|ui| {
                                        ui.label(format!("{}.", index + 1));
                                        ui.label(egui::RichText::new(&tool.name).color(egui::Color32::BLUE));
                                        if let Some(desc) = &tool.description {
                                            ui.label(format!("- {}", desc));
                                        }
                                    });
                                }
                            });
                    }

                    // 资源详情
                    if !capabilities.resources.is_empty() {
                        ui.separator();
                        ui.label(egui::RichText::new("资源列表:").strong());

                        egui::ScrollArea::vertical()
                            .max_height(100.0)
                            .show(ui, |ui| {
                                for (index, resource) in capabilities.resources.iter().enumerate() {
                                    ui.horizontal(|ui| {
                                        ui.label(format!("{}.", index + 1));
                                        ui.label(egui::RichText::new(&resource.name).color(egui::Color32::BLUE));
                                        if let Some(desc) = &resource.description {
                                            ui.label(format!("- {}", desc));
                                        }
                                    });
                                }
                            });
                    }

                    // 提示详情
                    if !capabilities.prompts.is_empty() {
                        ui.separator();
                        ui.label(egui::RichText::new("提示列表:").strong());

                        egui::ScrollArea::vertical()
                            .max_height(100.0)
                            .show(ui, |ui| {
                                for (index, prompt) in capabilities.prompts.iter().enumerate() {
                                    ui.horizontal(|ui| {
                                        ui.label(format!("{}.", index + 1));
                                        ui.label(egui::RichText::new(&prompt.name).color(egui::Color32::BLUE));
                                        if let Some(desc) = &prompt.description {
                                            ui.label(format!("- {}", desc));
                                        }
                                    });
                                }
                            });
                    }

                    ui.separator();
                    ui.colored_label(
                        egui::Color32::GRAY,
                        "💡 提示: 发送消息时，AI助手可以调用上述工具来帮助您完成任务。"
                    );
                });
        }
    } else if !state.mcp_server_capabilities.is_empty() {
        ui.separator();
        ui.colored_label(
            egui::Color32::YELLOW,
            "⚠️ 有可用的MCP服务器，请在上方选择一个以启用工具调用功能。"
        );
    }
}

/// 渲染消息内容，简单处理<think>标签
fn render_formatted_message(ui: &mut egui::Ui, content: &str, max_width: f32, _is_streaming: bool, text_color: egui::Color32) {
    // 设置最大宽度
    ui.set_max_width(max_width);

    // 直接将所有内容作为普通文本处理
    // 如果内容包含<think>标签，则将其中的文本显示为灰色，否则使用传入的颜色
    if content.contains("<think>") {
        ui.add(egui::Label::new(
            egui::RichText::new(content)
                .color(egui::Color32::from_rgb(100, 100, 100))
        ).wrap());
    } else {
        ui.add(egui::Label::new(
            egui::RichText::new(content)
                .color(text_color)
        ).wrap());
    }
}
