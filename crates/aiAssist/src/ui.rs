use eframe::egui;
use crate::state::{AIAssistState, MessageRole};

/// Render the AI assistant UI
pub fn render_ai_assist(ui: &mut egui::Ui, state: &mut AIAssistState) {
    // 在每帧开始时重置智能指令菜单的回车键处理标志
    // 注意：这个重置必须在处理任何输入之前进行
    if !state.command_menu.is_visible {
        state.command_menu_just_handled_enter = false;
    }

    // 检查是否有来自异步任务的更新
    state.check_for_updates();

    // 处理延迟的MCP服务器选择变化（避免在ComboBox事件处理过程中触发）
    state.process_pending_mcp_selection_change();

    // 请求重绘以保持流式输出的更新
    if state.is_sending {
        ui.ctx().request_repaint();
    }

    // 如果需要强制布局更新，请求重绘并重置标志
    if state.force_layout_update {
        ui.ctx().request_repaint();
        state.force_layout_update = false;
        log::debug!("🎨 强制UI重新布局以解决消息重叠问题");
    }
    // 获取可用高度，减去状态栏的高度（约30像素）以避免遮挡
    let available_height = ui.available_height() - 30.0;

    // 创建一个垂直布局容器，确保内容撑满高度但不超出
    egui::containers::Frame::none()
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
                    egui::Frame::none()
                        .fill(ui.style().visuals.window_fill)
                        .stroke(egui::Stroke::new(1.0, ui.style().visuals.widgets.noninteractive.bg_stroke.color))
                        .show(ui, |ui| {
                            egui::ScrollArea::vertical()
                                .id_source("chat_history_dropdown")
                                .max_height(300.0)
                                .show(ui, |ui| {
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
                let chat_height = available_height - 175.0; // 减去标题栏、工具栏和输入框的高度
                egui::ScrollArea::vertical()
                    .id_source("main_chat_messages")
                    .auto_shrink([false, false])
                    .stick_to_bottom(true)
                    .max_height(chat_height)
                    .show(ui, |ui| {
                        log::debug!("🎨 UI渲染消息列表，总消息数: {}", state.chat_messages.len());

                        // 收集需要执行的工具调用
                        let mut pending_tool_executions: Vec<ToolCallExecutionRequest> = Vec::new();

                        for (index, message) in state.chat_messages.iter().enumerate() {
                            log::trace!("🎨 渲染消息 {}: ID={}, 角色={:?}, 工具调用={}",
                                index + 1, message.id, message.role, message.tool_calls.is_some());
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
                                let frame = egui::Frame::none()
                                    .fill(frame_fill)
                                    .rounding(egui::Rounding::same(8.0))
                                    .inner_margin(egui::Margin::same(10.0))
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
                                            // 计算用户消息的最大宽度，与AI消息保持一致
                                            let available_width = ui.available_width();
                                            let max_content_width = (available_width - 40.0).max(200.0); // 预留40px边距，最小200px

                                            // 使用与AI消息相同的宽度控制，但不强制限制布局宽度
                                            ui.set_max_width(max_content_width);
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
                                            // 根据AI侧边栏的实际宽度动态调整对话框宽度
                                            // 预留右边距，防止长文本撑开整个界面宽度
                                            let available_width = ui.available_width();
                                            let max_content_width = (available_width * 0.85).max(200.0); // 限制为可用宽度的85%，最小200px

                                            // 设置最大宽度，防止AI消息过宽
                                            ui.set_max_width(max_content_width);

                                            // 如果是流式输出中的消息，显示动画效果
                                            if is_streaming && state.is_sending {
                                                let text = &message.content;

                                                // 处理消息内容
                                                render_formatted_message(ui, text, max_content_width, true, text_color);

                                                // 添加闪烁的光标
                                                let cursor = if (ui.input(|i| i.time) * 2.0).sin() > 0.0 { "▋" } else { " " };
                                                ui.label(egui::RichText::new(cursor).color(text_color));
                                            } else {
                                                // 根据消息内容和工具调用情况，分三种情况显示：
                                                let has_content = !message.content.trim().is_empty();
                                                let has_tool_calls = message.tool_calls.is_some();

                                                log::debug!("🎨 UI渲染分析: 消息ID={}, has_content={}, has_tool_calls={}",
                                                    message.id, has_content, has_tool_calls);
                                                if has_content {
                                                    log::debug!("🎨 Content内容: {}", &message.content[..std::cmp::min(100, message.content.len())]);
                                                }
                                                if has_tool_calls {
                                                    if let Some(tool_calls) = &message.tool_calls {
                                                        log::debug!("🎨 Tool calls数量: {}", tool_calls.len());
                                                    }
                                                }

                                                // 情况1: 仅包含content - 显示content内容
                                                if has_content && !has_tool_calls {
                                                    log::debug!("🎨 UI渲染: 仅包含content，消息ID: {}", message.id);
                                                    ui.group(|ui| {
                                                        render_formatted_message(ui, &message.content, max_content_width, false, text_color);
                                                    });
                                                }
                                                // 情况2: 仅包含tool_calls - 显示工具调用框
                                                else if !has_content && has_tool_calls {
                                                    if let Some(tool_calls) = &message.tool_calls {
                                                        log::debug!("🎨 UI渲染: 仅包含工具调用，消息ID: {}", message.id);
                                                        ui.group(|ui| {
                                                            if let Some(tool_call_request) = render_tool_calls_in_message(ui, tool_calls, max_content_width, message.tool_call_results.as_deref(), message.mcp_server_info.as_ref()) {
                                                                pending_tool_executions.push(tool_call_request);
                                                            }
                                                        });
                                                    }
                                                }
                                                // 情况3: 同时包含content和tool_calls - 先显示content，再显示tool_calls
                                                else if has_content && has_tool_calls {
                                                    log::debug!("🎨 UI渲染: 同时包含content和工具调用，消息ID: {}", message.id);

                                                    // 使用垂直布局确保content和tool_calls不重叠
                                                    ui.vertical(|ui| {
                                                        // 先显示content
                                                        ui.group(|ui| {
                                                            render_formatted_message(ui, &message.content, max_content_width, false, text_color);
                                                        });

                                                        if let Some(tool_calls) = &message.tool_calls {
                                                            // 添加content和工具调用之间的视觉分隔
                                                            ui.add_space(16.0);
                                                            ui.horizontal(|ui| {
                                                                ui.add_space(20.0);
                                                                ui.separator();
                                                                ui.add_space(20.0);
                                                            });
                                                            ui.add_space(12.0);

                                                            // 再显示工具调用框
                                                            ui.group(|ui| {
                                                                if let Some(tool_call_request) = render_tool_calls_in_message(ui, tool_calls, max_content_width, message.tool_call_results.as_deref(), message.mcp_server_info.as_ref()) {
                                                                    pending_tool_executions.push(tool_call_request);
                                                                }
                                                            });
                                                        }
                                                    });
                                                }

                                                // 处理向后兼容：如果没有工具调用但有工具调用结果，单独显示结果
                                                if !has_tool_calls && message.tool_call_results.is_some() {
                                                    if let Some(tool_results) = &message.tool_call_results {
                                                        // 为工具调用结果添加视觉分隔
                                                        if has_content {
                                                            ui.add_space(16.0);
                                                            ui.horizontal(|ui| {
                                                                ui.add_space(10.0);
                                                                ui.separator();
                                                                ui.add_space(10.0);
                                                            });
                                                            ui.add_space(8.0);
                                                        }
                                                        render_tool_call_results_in_message(ui, tool_results, max_content_width);
                                                    }
                                                }
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

                        // 处理待执行的工具调用
                        for tool_call_request in pending_tool_executions {
                            log::info!("🚀 执行工具调用: {}", tool_call_request.tool_call.function.name);
                            if let Some(mcp_info) = &tool_call_request.mcp_server_info {
                                log::info!("📡 使用记录的MCP服务器: {}", mcp_info.server_name);
                            }
                            state.execute_single_tool_call(&tool_call_request.tool_call, tool_call_request.mcp_server_info.as_ref());
                        }
                    });

                ui.separator();

                // 输入区域 - 使用垂直布局确保输入框占满宽度
                ui.vertical(|ui| {
                    // 设置输入框的最大宽度
                    let available_width = ui.available_width();

                    // 计算输入框的动态高度
                    let line_height = ui.text_style_height(&egui::TextStyle::Body);
                    let min_rows = 2;
                    let max_rows = 8;

                    // 计算当前文本的行数
                    let text_lines = state.chat_input.lines().count().max(1);
                    let actual_rows = text_lines.clamp(min_rows, max_rows);

                    // 计算输入框高度（包括内边距）
                    let input_height = (actual_rows as f32 * line_height) + 16.0; // 16.0为内边距

                    // 使用 ScrollArea 来强制限制高度并提供滚动功能
                    let text_edit_response = egui::ScrollArea::vertical()
                        .id_source("chat_input_scroll")
                        .max_height(input_height)
                        .show(ui, |ui| {
                            ui.add_sized(
                                [available_width, input_height],
                                egui::TextEdit::multiline(&mut state.chat_input)
                                    .id(egui::Id::new("main_chat_input"))
                                    .hint_text("输入消息... (点击此处获得焦点)")
                                    .desired_rows(actual_rows) // 使用计算出的行数
                                    // 如果正在发送，禁用输入框
                                    .interactive(!state.is_sending)
                            )
                        }).inner;

                    // 如果需要聚焦，则请求焦点
                    if state.should_focus_chat {
                        log::info!("AI助手请求焦点: should_focus_chat = true");
                        text_edit_response.request_focus();
                        state.should_focus_chat = false;
                        // 强制刷新UI以确保焦点生效
                        ui.ctx().request_repaint();
                    }

                    // 如果用户点击了输入框区域，也请求焦点
                    if text_edit_response.clicked() {
                        log::info!("AI助手输入框被点击，请求焦点");
                        text_edit_response.request_focus();
                        // 强制刷新UI以确保焦点生效
                        ui.ctx().request_repaint();
                    }

                    // 获取光标位置并更新指令菜单
                    let cursor_pos = if text_edit_response.has_focus() {
                        // 尝试获取光标在屏幕上的位置
                        Some(text_edit_response.rect.left_bottom())
                    } else {
                        None
                    };

                    // 更新指令菜单状态
                    state.update_command_menu(cursor_pos);

                    // 处理键盘输入 - 首先检查菜单是否需要处理
                    let mut menu_handled = false;

                    // 检查是否刚刚处理了智能指令菜单的回车键（在菜单可见性检查之前）
                    if state.command_menu_just_handled_enter {
                        menu_handled = true;
                        log::info!("智能指令菜单刚刚处理了回车键，跳过所有键盘处理");
                    } else if state.command_menu.is_visible && text_edit_response.has_focus() {
                        // 当菜单可见时，优先处理菜单相关的键盘事件
                        ui.input_mut(|i| {
                            // 检查是否有IME组合状态，如果有则不处理特殊键
                            let has_ime_composition = i.events.iter().any(|event| {
                                matches!(event, egui::Event::Ime(egui::ImeEvent::Preedit(_)))
                            });

                            if !has_ime_composition {
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
                                    log::info!("智能指令菜单: 处理回车键，当前输入框内容: '{}'", state.chat_input);
                                    state.handle_command_menu_input(egui::Key::Enter);
                                    log::info!("智能指令菜单: 回车键处理完成，输入框内容: '{}'", state.chat_input);
                                } else if i.key_pressed(egui::Key::Tab) {
                                    // 消费Tab键事件，防止焦点转移
                                    i.consume_key(egui::Modifiers::NONE, egui::Key::Tab);
                                    menu_handled = true;
                                    state.handle_command_menu_input(egui::Key::Tab);
                                } else if i.key_pressed(egui::Key::Escape) {
                                    menu_handled = true;
                                    state.handle_command_menu_input(egui::Key::Escape);
                                }
                            }
                        });
                    }

                    // 只有在菜单没有处理输入时，才处理正常的键盘输入
                    if !menu_handled && text_edit_response.has_focus() && !state.is_sending {
                        // 检查IME组合状态
                        let has_ime_composition = ui.input(|i| {
                            i.events.iter().any(|event| {
                                matches!(event, egui::Event::Ime(egui::ImeEvent::Preedit(_)))
                            })
                        });

                        if !has_ime_composition {
                            let enter_pressed = ui.input(|i| i.key_pressed(egui::Key::Enter));
                            let alt_pressed = ui.input(|i| i.modifiers.alt);

                            // 检查是否刚刚处理了智能指令菜单的回车键
                            if state.command_menu_just_handled_enter {
                                log::info!("跳过正常发送逻辑: 智能指令菜单刚刚处理了回车键");
                                // 重置标志，为下一帧做准备
                                state.command_menu_just_handled_enter = false;
                            } else if enter_pressed {
                                if alt_pressed {
                                    // Alt+回车：添加换行符
                                    state.chat_input.push('\n');
                                } else {
                                    // 单独的回车：发送消息
                                    log::info!("正常发送逻辑: 处理回车键，当前输入框内容: '{}'", state.chat_input);
                                    if let Some(cmd) = state.send_message() {
                                        // Return the slash command to be handled by the parent
                                        if let Some(callback) = &mut state.slash_command_callback {
                                            callback(cmd);
                                        }
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
                    let at_button_response = ui.button("@");
                    if at_button_response.clicked() {
                        // 在输入框中插入@字符并激活智能指令菜单
                        if !state.chat_input.ends_with('@') {
                            state.chat_input.push('@');
                        }
                        // 激活智能指令菜单
                        state.command_menu.is_visible = true;
                        state.command_menu.menu_type = crate::state::CommandMenuType::AtCommands;
                        state.command_menu.selected_index = 0;
                        state.command_menu.trigger_position = state.chat_input.len() - 1;
                        state.command_menu.cursor_position = Some(at_button_response.rect.left_bottom());
                        // 请求输入框焦点
                        state.should_focus_chat = true;
                        // 关闭旧的选择框
                        state.show_at_commands = false;
                        state.show_slash_commands = false;
                    }

                    // Slash命令按钮
                    let slash_button_response = ui.button("/");
                    if slash_button_response.clicked() {
                        // 清空输入框并插入/字符，激活智能指令菜单
                        state.chat_input.clear();
                        state.chat_input.push('/');
                        // 激活智能指令菜单
                        state.command_menu.is_visible = true;
                        state.command_menu.menu_type = crate::state::CommandMenuType::SlashCommands;
                        state.command_menu.selected_index = 0;
                        state.command_menu.trigger_position = 0;
                        state.command_menu.cursor_position = Some(slash_button_response.rect.left_bottom());
                        // 请求输入框焦点
                        state.should_focus_chat = true;
                        // 关闭旧的选择框
                        state.show_at_commands = false;
                        state.show_slash_commands = false;
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

                        // 添加调试日志
                        log::debug!("🔍 AI助手UI渲染 - MCP服务器状态:");
                        log::debug!("  - mcp_server_capabilities 数量: {}", state.mcp_server_capabilities.len());
                        log::debug!("  - server_names 数量: {}", state.server_names.len());
                        log::debug!("  - selected_mcp_server: {:?}", state.selected_mcp_server);

                        for (server_id, capabilities) in &state.mcp_server_capabilities {
                            let server_name = state.server_names.get(server_id).cloned().unwrap_or_else(|| "未知".to_string());
                            log::debug!("  - 服务器 {}: {} (工具:{} 资源:{} 提示:{})",
                                server_id.to_string().chars().take(8).collect::<String>(),
                                server_name,
                                capabilities.tools.len(),
                                capabilities.resources.len(),
                                capabilities.prompts.len()
                            );
                        }

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

                        ui.horizontal(|ui| {
                            // 记录选择前的状态
                            let previous_selection = state.selected_mcp_server;

                            let combo_response = egui::ComboBox::from_label("")
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

                            // 延迟检查选择变化，避免在ComboBox事件处理过程中触发
                            // 只有在ComboBox关闭后才检查变化，避免macOS事件处理冲突
                            if combo_response.response.changed() {
                                // 使用ctx.request_repaint()延迟到下一帧处理
                                ui.ctx().request_repaint();
                                // 将选择变化检查存储起来，在下一帧处理
                                state.pending_mcp_selection_change = Some(previous_selection);
                            }

                            // 添加刷新按钮用于调试
                            if ui.small_button("🔄").on_hover_text("刷新MCP服务器列表").clicked() {
                                log::info!("🔄 用户点击刷新MCP服务器列表按钮");
                                // 通过回调来触发主应用的同步
                                if let Some(callback) = &mut state.mcp_refresh_callback {
                                    callback();
                                }
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

    // 旧的选择框已被智能指令菜单替代，不再需要单独渲染

    // 工具调用现在嵌入在消息中显示，不使用外部弹出对话框
    // if state.show_tool_call_confirmation {
    //     log::info!("🎨 显示工具调用确认对话框");
    //     render_tool_call_confirmation(ui.ctx(), state);
    // }

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

                // 获取用于显示的API Key
                let mut display_key = state.get_display_api_key();
                let response = ui.text_edit_singleline(&mut display_key);

                // 如果用户修改了显示的内容，更新实际的API Key
                if response.changed() {
                    // 如果当前是掩码模式且用户修改了内容，切换到非掩码模式
                    if state.show_api_key_masked {
                        state.show_api_key_masked = false;
                    }
                    state.ai_settings.api_key = display_key;
                }

                // 添加显示/隐藏按钮
                let button_text = if state.show_api_key_masked { "👁" } else { "🙈" };
                let button_tooltip = if state.show_api_key_masked { "显示完整API Key" } else { "隐藏API Key" };

                if ui.button(button_text).on_hover_text(button_tooltip).clicked() {
                    state.show_api_key_masked = !state.show_api_key_masked;
                }
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
            ("@term", "引用终端输出内容"),
            ("@note", "引用当前笔记内容"),
            ("@editor", "引用当前文件内容"),
        ],
        CommandMenuType::SlashCommands => vec![
            ("/search", "执行搜索"),
            ("/clear", "清空当前会话"),
            ("/help", "显示帮助信息"),
            ("/new", "创建新会话"),
            ("/term", "终端操作"),
            ("/note", "笔记操作"),
            ("/editor", "文件编辑器操作"),
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

    let window_size = egui::vec2(300.0, 200.0);

    // 计算窗口位置 - 基于@按钮的实际位置
    let window_pos = if let Some(button_rect) = state.at_button_rect {
        let screen_rect = ctx.screen_rect();

        // 计算初始位置：在@按钮的上方
        let mut pos_x = button_rect.left() - 50.0; // 向左偏移50像素
        let mut pos_y = button_rect.top() - window_size.y - 10.0; // 在按钮上方10像素

        // 边界检查：确保不超出屏幕左边界
        if pos_x < screen_rect.left() {
            pos_x = screen_rect.left() + 10.0;
        }

        // 边界检查：确保不超出屏幕右边界
        if pos_x + window_size.x > screen_rect.right() {
            pos_x = screen_rect.right() - window_size.x - 10.0;
        }

        // 边界检查：如果上方空间不够，放在按钮下方
        if pos_y < screen_rect.top() {
            pos_y = button_rect.bottom() + 10.0;
        }

        // 边界检查：确保不超出屏幕下边界
        if pos_y + window_size.y > screen_rect.bottom() {
            pos_y = screen_rect.bottom() - window_size.y - 10.0;
        }

        egui::pos2(pos_x, pos_y)
    } else {
        // 回退到屏幕中央（如果没有按钮位置信息）
        let screen_rect = ctx.screen_rect();
        egui::pos2(
            screen_rect.center().x - window_size.x / 2.0,
            screen_rect.center().y - window_size.y / 2.0
        )
    };

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
            ui.label("@term - 引用终端输出内容");
            ui.label("@note - 引用当前笔记内容");
            ui.label("@editor - 引用当前文件内容");

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

            if ui.button("插入 @term").clicked() {
                state.chat_input.push_str("@term");
                state.show_at_commands = false;
            }

            if ui.button("插入 @note").clicked() {
                state.chat_input.push_str("@note");
                state.show_at_commands = false;
            }

            if ui.button("插入 @editor").clicked() {
                state.chat_input.push_str("@editor");
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

    let window_size = egui::vec2(300.0, 200.0);

    // 计算窗口位置 - 基于/按钮的实际位置
    let window_pos = if let Some(button_rect) = state.slash_button_rect {
        let screen_rect = ctx.screen_rect();

        // 计算初始位置：在/按钮的上方
        let mut pos_x = button_rect.left() - 50.0; // 向左偏移50像素
        let mut pos_y = button_rect.top() - window_size.y - 10.0; // 在按钮上方10像素

        // 边界检查：确保不超出屏幕左边界
        if pos_x < screen_rect.left() {
            pos_x = screen_rect.left() + 10.0;
        }

        // 边界检查：确保不超出屏幕右边界
        if pos_x + window_size.x > screen_rect.right() {
            pos_x = screen_rect.right() - window_size.x - 10.0;
        }

        // 边界检查：如果上方空间不够，放在按钮下方
        if pos_y < screen_rect.top() {
            pos_y = button_rect.bottom() + 10.0;
        }

        // 边界检查：确保不超出屏幕下边界
        if pos_y + window_size.y > screen_rect.bottom() {
            pos_y = screen_rect.bottom() - window_size.y - 10.0;
        }

        egui::pos2(pos_x, pos_y)
    } else {
        // 回退到屏幕中央（如果没有按钮位置信息）
        let screen_rect = ctx.screen_rect();
        egui::pos2(
            screen_rect.center().x - window_size.x / 2.0,
            screen_rect.center().y - window_size.y / 2.0
        )
    };

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
            ui.label("/term [命令] - 终端操作");
            ui.label("/note [操作] - 笔记操作");
            ui.label("/editor [操作] - 文件编辑器操作");

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

            if ui.button("插入 /term").clicked() {
                state.chat_input.push_str("/term ");
                state.show_slash_commands = false;
            }

            if ui.button("插入 /note").clicked() {
                state.chat_input.push_str("/note ");
                state.show_slash_commands = false;
            }

            if ui.button("插入 /editor").clicked() {
                state.chat_input.push_str("/editor ");
                state.show_slash_commands = false;
            }
        });

    if !open {
        state.show_slash_commands = false;
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
                .id_source(format!("mcp_server_status_{}", server_id))
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
                            .id_source(format!("mcp_tools_list_{}", server_id))
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
                            .id_source(format!("mcp_resources_list_{}", server_id))
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
                            .id_source(format!("mcp_prompts_list_{}", server_id))
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
    // 如果内容为空，不显示任何内容
    if content.trim().is_empty() {
        return;
    }

    // 选择文本颜色
    let display_color = if content.contains("<think>") {
        egui::Color32::from_rgb(100, 100, 100)
    } else {
        text_color
    };

    // 使用严格的宽度控制，防止长文本撑开界面
    ui.set_max_width(max_width);

    // 使用 allocate_ui_with_layout 来确保内容在指定宽度内换行
    ui.allocate_ui_with_layout(
        egui::Vec2::new(max_width, ui.available_height()),
        egui::Layout::top_down_justified(egui::Align::LEFT),
        |ui| {
            ui.set_max_width(max_width);
            // 使用 Label 并强制换行，确保文本在指定宽度内换行
            ui.add(
                egui::Label::new(
                    egui::RichText::new(content)
                        .color(display_color)
                ).wrap()
            );
        }
    );
}

/// 工具调用执行请求
#[derive(Clone, Debug)]
pub struct ToolCallExecutionRequest {
    pub tool_call: crate::api::ToolCall,
    pub mcp_server_info: Option<crate::state::McpServerInfo>,
}

/// 渲染工具调用摘要信息（简化版本）
fn render_tool_calls_summary(ui: &mut egui::Ui, tool_calls: &[crate::api::ToolCall], max_width: f32, tool_results: Option<&[crate::state::ToolCallResult]>, mcp_server_info: Option<&crate::state::McpServerInfo>) {
    // 获取当前主题的颜色
    let visuals = &ui.style().visuals;
    let is_dark_mode = visuals.dark_mode;

    let title_color = if is_dark_mode {
        egui::Color32::from_rgb(100, 150, 255)
    } else {
        egui::Color32::from_rgb(0, 80, 160)
    };

    let background_color = if is_dark_mode {
        egui::Color32::from_rgb(40, 45, 55)
    } else {
        egui::Color32::from_rgb(250, 252, 255)
    };

    let border_color = if is_dark_mode {
        egui::Color32::from_rgb(70, 100, 180)
    } else {
        egui::Color32::from_rgb(120, 160, 255)
    };

    ui.set_max_width(max_width);

    // 显示简化的工具调用摘要
    egui::Frame::none()
        .fill(background_color)
        .stroke(egui::Stroke::new(1.0, border_color))
        .inner_margin(egui::Margin::same(12.0))
        .rounding(6.0)
        .show(ui, |ui| {
            ui.vertical(|ui| {
                // 工具调用标题
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("🔧").size(16.0).color(title_color));
                    ui.add_space(6.0);
                    ui.label(egui::RichText::new(format!("AI助手请求调用 {} 个工具", tool_calls.len()))
                        .strong()
                        .color(title_color));
                });

                ui.add_space(8.0);

                // 显示工具列表（简化版）
                for (index, tool_call) in tool_calls.iter().enumerate() {
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new(format!("{}.", index + 1)).color(title_color));
                        ui.add_space(4.0);

                        let tool_display = if let Some(mcp_info) = mcp_server_info {
                            format!("{} ({})", &tool_call.function.name, &mcp_info.server_name)
                        } else {
                            tool_call.function.name.clone()
                        };

                        ui.label(egui::RichText::new(tool_display).color(title_color));

                        // 显示执行状态
                        if let Some(results) = tool_results {
                            if let Some(result) = results.iter().find(|r| r.tool_call_id == tool_call.id) {
                                ui.add_space(8.0);
                                let (icon, color) = if result.success {
                                    ("✅", egui::Color32::DARK_GREEN)
                                } else {
                                    ("❌", egui::Color32::DARK_RED)
                                };
                                ui.label(egui::RichText::new(icon).color(color));
                            }
                        }
                    });

                    if index < tool_calls.len() - 1 {
                        ui.add_space(4.0);
                    }
                }

                // 如果有执行结果，显示结果摘要
                if let Some(results) = tool_results {
                    let related_results: Vec<_> = results.iter()
                        .filter(|result| tool_calls.iter().any(|tc| tc.id == result.tool_call_id))
                        .collect();

                    if !related_results.is_empty() {
                        ui.add_space(8.0);
                        ui.separator();
                        ui.add_space(6.0);

                        ui.label(egui::RichText::new("📋 执行结果摘要").strong().color(title_color));
                        ui.add_space(4.0);

                        for result in related_results {
                            ui.horizontal(|ui| {
                                let (icon, color) = if result.success {
                                    ("✅", egui::Color32::DARK_GREEN)
                                } else {
                                    ("❌", egui::Color32::DARK_RED)
                                };
                                ui.label(egui::RichText::new(icon).color(color));
                                ui.add_space(4.0);

                                let status = if result.success { "成功" } else { "失败" };
                                ui.label(egui::RichText::new(status).color(color));

                                if !result.success && result.error.is_some() {
                                    ui.add_space(4.0);
                                    ui.label(egui::RichText::new("(有错误信息)").small().color(egui::Color32::GRAY));
                                }
                            });
                        }
                    }
                }
            });
        });
}

/// 在消息中渲染工具调用信息，包括相关的执行结果
fn render_tool_calls_in_message(ui: &mut egui::Ui, tool_calls: &[crate::api::ToolCall], max_width: f32, tool_results: Option<&[crate::state::ToolCallResult]>, mcp_server_info: Option<&crate::state::McpServerInfo>) -> Option<ToolCallExecutionRequest> {
    // 减少日志频率，只在第一次渲染时记录
    log::debug!("🎨 开始渲染工具调用信息，工具数量: {}", tool_calls.len());

    // 使用传入的最大宽度，确保与对话框宽度一致
    ui.set_max_width(max_width);
    // 移除这里的间距，因为调用方已经添加了适当的分隔
    // ui.add_space(8.0);

    // 获取当前主题的颜色
    let visuals = &ui.style().visuals;
    let is_dark_mode = visuals.dark_mode;

    // 主题适配的颜色
    let title_color = if is_dark_mode {
        egui::Color32::from_rgb(100, 150, 255)  // 深色主题：亮蓝色
    } else {
        egui::Color32::from_rgb(0, 80, 160)     // 浅色主题：深蓝色
    };

    let background_color = if is_dark_mode {
        egui::Color32::from_rgb(45, 50, 65)     // 深色主题：深蓝灰色
    } else {
        egui::Color32::from_rgb(248, 252, 255)  // 浅色主题：浅蓝色
    };

    let border_color = if is_dark_mode {
        egui::Color32::from_rgb(80, 120, 200)   // 深色主题：中蓝色
    } else {
        egui::Color32::from_rgb(100, 150, 255)  // 浅色主题：亮蓝色
    };

    // 添加工具调用标题


    log::debug!("🎨 工具调用标题已渲染");
    // ui.add_space(6.0);

    let mut tool_call_to_execute = None;

    ui.vertical(|ui|{

    
        // 显示工具调用总标题（只显示一次）- 使用更突出的样式
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("🔧").size(18.0).color(title_color));
            ui.add_space(6.0);
            ui.label(egui::RichText::new(format!("AI助手请求调用 {} 个工具", tool_calls.len()))
                .strong()
                .size(14.0)
                .color(title_color));
        });
        ui.add_space(12.0);

        for (index, tool_call) in tool_calls.iter().enumerate() {
            // 工具调用框架 - 使用主题适配的样式
            egui::Frame::none()
                .fill(background_color)
                .stroke(egui::Stroke::new(1.5, border_color))
                .inner_margin(egui::Margin::same(12.0))
                .rounding(8.0)
                .show(ui, |ui| {
                    ui.set_max_width(max_width - 24.0); // 确保不超出父容器宽度

                    ui.vertical(|ui| {
                        // 工具名称行，包含MCP Server信息
                        ui.horizontal(|ui| {
                            let tool_display = if let Some(mcp_info) = mcp_server_info {
                                format!("{}#工具: {} ({})", index + 1, &tool_call.function.name, &mcp_info.server_name)
                            } else {
                                format!("{}#工具: {}", index + 1, &tool_call.function.name)
                            };

                            ui.label(egui::RichText::new(tool_display)
                                .strong()
                                .color(title_color));
                        });

                        ui.add_space(4.0);

                        // 参数显示
                        let formatted_args = if tool_call.function.arguments.trim().is_empty() {
                            "无参数".to_string()
                        } else {
                            match serde_json::from_str::<serde_json::Value>(&tool_call.function.arguments) {
                                Ok(json) => serde_json::to_string_pretty(&json).unwrap_or_else(|_| tool_call.function.arguments.clone()),
                                Err(_) => tool_call.function.arguments.clone(),
                            }
                        };

                        // 参数框
                        egui::Frame::none()
                            .fill(if is_dark_mode { egui::Color32::from_rgb(35, 40, 50) } else { egui::Color32::from_rgb(250, 250, 250) })
                            .stroke(egui::Stroke::new(1.0, if is_dark_mode { egui::Color32::from_rgb(60, 60, 60) } else { egui::Color32::from_rgb(220, 220, 220) }))
                            .inner_margin(egui::Margin::same(8.0))
                            .rounding(4.0)
                            .show(ui, |ui| {
                                ui.vertical(|ui| {
                                    ui.label(egui::RichText::new("参数:")
                                        .small()
                                        .color(if is_dark_mode { egui::Color32::LIGHT_GRAY } else { egui::Color32::GRAY }));

                                    ui.add(egui::TextEdit::multiline(&mut formatted_args.as_str())
                                        .id(egui::Id::new(format!("tool_args_{}", tool_call.id)))
                                        .desired_rows(if formatted_args.lines().count() > 3 { 3 } else { formatted_args.lines().count().max(1) })
                                        .font(egui::TextStyle::Monospace)
                                        .interactive(false));
                                });
                            });

                        ui.add_space(8.0);

                        // 执行按钮
                        ui.horizontal(|ui| {
                            // 检查是否已经有执行结果来决定按钮文字
                            let has_results = if let Some(results) = tool_results {
                                results.iter().any(|result| result.tool_call_id == tool_call.id)
                            } else {
                                false
                            };

                            let button_text = if has_results {
                                "▶ 再次执行"
                            } else {
                                "▶ 执行"
                            };

                            let button_response = ui.add_sized([100.0, 28.0], egui::Button::new(button_text)
                                .fill(if is_dark_mode { egui::Color32::from_rgb(0, 120, 60) } else { egui::Color32::from_rgb(0, 150, 80) }));

                            if button_response.clicked() {
                                log::debug!("🎯 用户点击执行工具调用: {}", tool_call.function.name);
                                tool_call_to_execute = Some(ToolCallExecutionRequest {
                                    tool_call: tool_call.clone(),
                                    mcp_server_info: mcp_server_info.cloned(),
                                });
                            }

                            ui.add_space(8.0);
                            let hint_text = if has_results {
                                "点击再次执行此工具调用"
                            } else {
                                "点击执行此工具调用"
                            };
                            ui.label(egui::RichText::new(hint_text)
                                .small()
                                .color(if is_dark_mode { egui::Color32::LIGHT_GRAY } else { egui::Color32::GRAY }));
                        });

                        // 在工具调用下方显示相关的执行结果
                        if let Some(results) = tool_results {
                            let related_results: Vec<_> = results.iter()
                                .filter(|result| result.tool_call_id == tool_call.id)
                                .collect();

                            if !related_results.is_empty() {
                                ui.add_space(12.0);
                                ui.separator();
                                ui.add_space(8.0);

                                ui.label(egui::RichText::new("📋 执行结果")
                                    .strong()
                                    .color(title_color));
                                ui.add_space(4.0);

                                // 按时间戳排序结果
                                let mut sorted_results = related_results;
                                sorted_results.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

                                for (result_index, result) in sorted_results.iter().enumerate() {
                                    render_single_tool_result(ui, result, is_dark_mode, result_index);
                                    ui.add_space(6.0);
                                }
                            }
                        }
                    });
                });

            if index < tool_calls.len() - 1 {
                ui.add_space(8.0);
            }
        }
    });

    tool_call_to_execute
}

/// 渲染单个工具调用结果
fn render_single_tool_result(ui: &mut egui::Ui, result: &crate::state::ToolCallResult, is_dark_mode: bool, result_index: usize) {
    // 获取主题适配的颜色
    let (bg_color, border_color, icon) = if result.success {
        if is_dark_mode {
            (egui::Color32::from_rgb(25, 45, 25), egui::Color32::from_rgb(80, 150, 80), "✅")
        } else {
            (egui::Color32::from_rgb(240, 255, 240), egui::Color32::from_rgb(144, 238, 144), "✅")
        }
    } else {
        if is_dark_mode {
            (egui::Color32::from_rgb(45, 25, 25), egui::Color32::from_rgb(150, 80, 80), "❌")
        } else {
            (egui::Color32::from_rgb(255, 240, 240), egui::Color32::from_rgb(255, 182, 193), "❌")
        }
    };

    egui::Frame::none()
        .fill(bg_color)
        .stroke(egui::Stroke::new(1.0, border_color))
        .inner_margin(egui::Margin::same(8.0))
        .rounding(4.0)
        .show(ui, |ui| {
            ui.vertical(|ui| {
                // 结果标题行，包含时间戳
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(icon).size(14.0));

                    let status_text = if result.success { "执行成功" } else { "执行失败" };
                    ui.label(egui::RichText::new(status_text)
                        .strong()
                        .color(if result.success {
                            if is_dark_mode { egui::Color32::from_rgb(100, 200, 100) } else { egui::Color32::DARK_GREEN }
                        } else {
                            if is_dark_mode { egui::Color32::from_rgb(200, 100, 100) } else { egui::Color32::DARK_RED }
                        }));

                    // 精准时间戳显示
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let timestamp_text = format_precise_timestamp(&result.timestamp);
                        ui.label(egui::RichText::new(timestamp_text)
                            .small()
                            .color(if is_dark_mode { egui::Color32::LIGHT_GRAY } else { egui::Color32::GRAY }));
                    });
                });

                // 结果内容
                if !result.result.trim().is_empty() {
                    ui.add_space(4.0);
                    // 使用时间戳和索引创建唯一ID，避免多次执行同一工具时的ID冲突
                    let unique_id = format!("{}_{}_{}_{}", result.tool_call_id, result.timestamp.timestamp_millis(), result_index, result.success);
                    egui::CollapsingHeader::new("结果详情")
                        .id_source(format!("tool_result_details_{}", unique_id))
                        .default_open(true)
                        .show(ui, |ui| {
                            egui::ScrollArea::vertical()
                                .id_source(format!("tool_result_scroll_{}", unique_id))
                                .max_height(120.0)
                                .show(ui, |ui| {
                                    ui.add(egui::TextEdit::multiline(&mut result.result.as_str())
                                        .id(egui::Id::new(format!("tool_result_text_{}", unique_id)))
                                        .desired_rows(3)
                                        .font(egui::TextStyle::Monospace)
                                        .interactive(false));
                                });
                        });
                }

                // 错误信息
                if let Some(error) = &result.error {
                    ui.add_space(4.0);
                    ui.colored_label(
                        if is_dark_mode { egui::Color32::from_rgb(255, 150, 150) } else { egui::Color32::RED },
                        format!("错误: {}", error)
                    );
                }
            });
        });
}

/// 格式化工具结果时间戳（相对时间）
pub fn format_tool_result_timestamp(timestamp: &chrono::DateTime<chrono::Utc>) -> String {
    use chrono::Local;

    // 转换为本地时间
    let local_time = timestamp.with_timezone(&Local);
    let now = Local::now();

    // 计算时间差
    let duration = now.signed_duration_since(local_time);

    if duration.num_seconds() < 60 {
        "刚刚".to_string()
    } else if duration.num_minutes() < 60 {
        format!("{}分钟前", duration.num_minutes())
    } else if duration.num_hours() < 24 {
        format!("{}小时前", duration.num_hours())
    } else {
        // 超过一天显示具体时间
        local_time.format("%m-%d %H:%M").to_string()
    }
}

/// 格式化精准时间戳（yyyy-mm-dd HH:MM:SS）
pub fn format_precise_timestamp(timestamp: &chrono::DateTime<chrono::Utc>) -> String {
    use chrono::Local;

    // 转换为本地时间
    let local_time = timestamp.with_timezone(&Local);

    // 格式化为 yyyy-mm-dd HH:MM:SS
    local_time.format("%Y-%m-%d %H:%M:%S").to_string()
}

/// 在消息中渲染工具调用结果
fn render_tool_call_results_in_message(ui: &mut egui::Ui, tool_results: &[crate::state::ToolCallResult], max_width: f32) {
    // 使用传入的最大宽度，确保与对话框宽度一致
    ui.set_max_width(max_width);
    ui.add_space(8.0);

    for (index, result) in tool_results.iter().enumerate() {
        // 结果框架，根据成功/失败使用不同颜色
        let (bg_color, border_color, icon) = if result.success {
            (egui::Color32::from_rgb(240, 255, 240), egui::Color32::from_rgb(144, 238, 144), "✅")
        } else {
            (egui::Color32::from_rgb(255, 240, 240), egui::Color32::from_rgb(255, 182, 193), "❌")
        };

        egui::Frame::none()
            .fill(bg_color)
            .stroke(egui::Stroke::new(1.0, border_color))
            .inner_margin(egui::Margin::same(8.0))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    // 结果图标
                    ui.label(egui::RichText::new(icon).size(16.0));

                    // 结果标题
                    let status_text = if result.success { "执行成功" } else { "执行失败" };
                    ui.label(egui::RichText::new(format!("工具结果 {}: {}", index + 1, status_text))
                        .strong()
                        .color(if result.success { egui::Color32::DARK_GREEN } else { egui::Color32::DARK_RED }));
                });

                // 结果内容
                if !result.result.trim().is_empty() {
                    egui::CollapsingHeader::new("结果详情")
                        .id_source(format!("legacy_tool_result_details_{}_{}", result.tool_call_id, index))
                        .default_open(true)
                        .show(ui, |ui| {
                            egui::ScrollArea::vertical()
                                .id_source(format!("legacy_tool_result_scroll_{}_{}", result.tool_call_id, index))
                                .max_height(120.0)
                                .show(ui, |ui| {
                                    ui.add(egui::TextEdit::multiline(&mut result.result.as_str())
                                        .id(egui::Id::new(format!("legacy_tool_result_text_{}_{}", result.tool_call_id, index)))
                                        .desired_rows(3)
                                        .font(egui::TextStyle::Monospace)
                                        .interactive(false));
                                });
                        });
                }

                // 错误信息
                if let Some(error) = &result.error {
                    ui.colored_label(egui::Color32::RED, format!("错误: {}", error));
                }
            });

        if index < tool_results.len() - 1 {
            ui.add_space(4.0);
        }
    }
}

/// 渲染工具调用确认对话框
fn render_tool_call_confirmation(ctx: &egui::Context, state: &mut AIAssistState) {
    // 克隆批次数据以避免借用冲突
    let batch_clone = state.current_tool_call_batch.clone();

    if let Some(batch) = batch_clone {
        let mut should_approve = false;
        let mut should_reject = false;

        egui::Window::new("🔧 工具调用确认")
            .collapsible(false)
            .resizable(true)
            .default_width(750.0)
            .default_height(550.0)
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    // 标题区域
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("🔧").size(24.0));
                        ui.add_space(8.0);
                        ui.label(egui::RichText::new("AI助手请求执行工具调用")
                            .size(18.0)
                            .strong());
                    });

                    ui.add_space(12.0);
                    ui.separator();
                    ui.add_space(12.0);

                    // 统计信息
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new(format!("📊 工具调用总数: {}", batch.tool_calls.len()))
                            .strong());

                        // 显示服务器信息（如果有的话）
                        if !batch.tool_calls.is_empty() {
                            ui.add_space(20.0);
                            ui.label(egui::RichText::new(format!("🖥️ 目标服务器: {}",
                                &batch.tool_calls[0].server_name))
                                .strong());
                        }
                    });

                    ui.add_space(16.0);

                    // 工具调用列表
                    egui::ScrollArea::vertical()
                        .max_height(300.0)
                        .show(ui, |ui| {
                            for (index, pending_call) in batch.tool_calls.iter().enumerate() {
                                render_tool_call_card(ui, pending_call, index);

                                if index < batch.tool_calls.len() - 1 {
                                    ui.add_space(12.0);
                                }
                            }
                        });

                    ui.add_space(16.0);

                    // 安全提醒
                    egui::Frame::none()
                        .fill(egui::Color32::from_rgb(255, 248, 220))
                        .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(255, 193, 7)))
                        .inner_margin(egui::Margin::same(12.0))
                        .rounding(6.0)
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new("⚠️").size(16.0));
                                ui.add_space(8.0);
                                ui.label(egui::RichText::new("安全提醒：请仔细检查工具调用参数，确认无误后再执行。")
                                    .strong()
                                    .color(egui::Color32::from_rgb(133, 77, 14)));
                            });
                        });

                    ui.add_space(20.0);

                    // 操作按钮
                    ui.horizontal(|ui| {
                        // 确认按钮
                        let confirm_button = ui.add_sized([120.0, 40.0],
                            egui::Button::new(egui::RichText::new("✅ 确认执行").strong())
                                .fill(egui::Color32::from_rgb(40, 167, 69)));

                        if confirm_button.clicked() {
                            log::info!("用户确认执行工具调用");
                            should_approve = true;
                        }

                        ui.add_space(12.0);

                        // 拒绝按钮
                        let reject_button = ui.add_sized([120.0, 40.0],
                            egui::Button::new(egui::RichText::new("❌ 拒绝执行").strong())
                                .fill(egui::Color32::from_rgb(220, 53, 69)));

                        if reject_button.clicked() {
                            log::info!("用户拒绝执行工具调用");
                            should_reject = true;
                        }
                    });
                });
            });

        // 处理按钮点击
        if should_approve {
            state.approve_tool_calls();
        } else if should_reject {
            state.reject_tool_calls();
        }
    }
}

/// 渲染单个工具调用卡片
fn render_tool_call_card(ui: &mut egui::Ui, pending_call: &crate::state::PendingToolCall, index: usize) {
    let visuals = &ui.style().visuals;
    let is_dark_mode = visuals.dark_mode;

    let background_color = if is_dark_mode {
        egui::Color32::from_rgb(45, 50, 65)
    } else {
        egui::Color32::from_rgb(248, 252, 255)
    };

    let border_color = if is_dark_mode {
        egui::Color32::from_rgb(80, 120, 200)
    } else {
        egui::Color32::from_rgb(100, 150, 255)
    };

    let title_color = if is_dark_mode {
        egui::Color32::from_rgb(100, 150, 255)
    } else {
        egui::Color32::from_rgb(0, 80, 160)
    };

    egui::Frame::none()
        .fill(background_color)
        .stroke(egui::Stroke::new(1.5, border_color))
        .inner_margin(egui::Margin::same(16.0))
        .rounding(8.0)
        .show(ui, |ui| {
            ui.vertical(|ui| {
                // 工具标题
                ui.horizontal(|ui| {
                    // 工具类型图标
                    let icon = match pending_call.mcp_info.call_type {
                        crate::mcp_tools::McpCallType::CallTool => "🔧",
                        crate::mcp_tools::McpCallType::ReadResource => "📄",
                        crate::mcp_tools::McpCallType::GetPrompt => "💬",
                    };

                    ui.label(egui::RichText::new(icon).size(18.0));
                    ui.add_space(8.0);

                    ui.label(egui::RichText::new(format!("{}. {}", index + 1, &pending_call.tool_call.function.name))
                        .size(16.0)
                        .strong()
                        .color(title_color));

                    ui.add_space(12.0);

                    // 服务器信息
                    ui.label(egui::RichText::new(format!("({})", &pending_call.server_name))
                        .small()
                        .color(egui::Color32::GRAY));
                });

                ui.add_space(8.0);

                // 调用类型
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("类型:").strong());
                    ui.add_space(4.0);
                    let type_text = match pending_call.mcp_info.call_type {
                        crate::mcp_tools::McpCallType::CallTool => "工具调用",
                        crate::mcp_tools::McpCallType::ReadResource => "读取资源",
                        crate::mcp_tools::McpCallType::GetPrompt => "获取提示",
                    };
                    ui.label(type_text);
                });

                ui.add_space(8.0);

                // 参数显示
                ui.label(egui::RichText::new("参数:").strong());
                ui.add_space(4.0);

                let formatted_args = if pending_call.tool_call.function.arguments.trim().is_empty() {
                    "无参数".to_string()
                } else {
                    match serde_json::from_str::<serde_json::Value>(&pending_call.tool_call.function.arguments) {
                        Ok(json) => serde_json::to_string_pretty(&json).unwrap_or_else(|_| pending_call.tool_call.function.arguments.clone()),
                        Err(_) => pending_call.tool_call.function.arguments.clone(),
                    }
                };

                egui::ScrollArea::vertical()
                    .max_height(120.0)
                    .show(ui, |ui| {
                        ui.add(egui::TextEdit::multiline(&mut formatted_args.as_str())
                            .desired_rows(3)
                            .font(egui::TextStyle::Monospace)
                            .interactive(false));
                    });
            });
        });
}
