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
                        .show(ui, |ui| {
                            egui::ScrollArea::vertical().max_height(200.0).show(ui, |ui| {
                                let mut selected_idx = None;

                                for (idx, session) in state.chat_sessions.iter().enumerate() {
                                    let is_active = idx == state.active_session_idx;
                                    if ui.selectable_label(is_active, &session.name).clicked() {
                                        selected_idx = Some(idx);
                                    }
                                }

                                // Handle session selection outside the loop to avoid borrowing issues
                                if let Some(idx) = selected_idx {
                                    state.switch_session(idx);
                                    state.show_history_dropdown = false;
                                }
                            });
                        });
                }

                ui.separator();

                // 聊天消息区域
                let chat_height = available_height - 150.0; // 减去标题栏、工具栏和输入框的高度
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .stick_to_bottom(true)
                    .max_height(chat_height)
                    .show(ui, |ui| {
                        for message in &state.chat_messages {
                            let is_user = message.role == MessageRole::User;

                            // 创建一个垂直布局，确保消息内容可以自动换行
                            ui.vertical(|ui| {
                                // 消息框的背景色
                                let frame_fill = if is_user {
                                    egui::Color32::from_rgba_premultiplied(240, 240, 255, 200)
                                } else {
                                    egui::Color32::from_rgba_premultiplied(240, 255, 240, 200)
                                };

                                // 创建圆角方框
                                let frame = egui::Frame::NONE
                                    .fill(frame_fill)
                                    .corner_radius(egui::Rounding::same(8))
                                    .inner_margin(egui::Margin::same(10))
                                    .stroke(egui::Stroke::new(1.0, ui.style().visuals.widgets.noninteractive.bg_stroke.color));

                                frame.show(ui, |ui| {
                                    if is_user {
                                        // 用户消息右对齐
                                        ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                                            ui.label(egui::RichText::new("用户: ").strong());
                                        });

                                        // 用户消息内容
                                        ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                                            // 使用 RichText 和 Label 设置自动换行，黑色文字
                                            let user_text = egui::RichText::new(&message.content)
                                                .strong()
                                                .color(egui::Color32::BLACK);
                                            ui.add(egui::Label::new(user_text).wrap());
                                        });
                                    } else {
                                        // 检查是否是正在流式输出的消息
                                        let is_streaming = state.streaming_message_id.map_or(false, |id| id == message.id);

                                        // AI消息左对齐
                                        // ui.horizontal(|ui| {
                                        //     ui.label(egui::RichText::new("AI: ").strong());
                                        // });
                                        ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP), |ui| {
                                            ui.label(egui::RichText::new("SeeU AI: ").strong());
                                        });

                                        // AI消息内容
                                        ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP), |ui| {
                                            // 设置最大宽度，确保文本自动换行
                                            let available_width = ui.available_width();

                                            // 如果是流式输出中的消息，显示动画效果
                                            if is_streaming && state.is_sending {
                                                let text = &message.content;

                                                // 处理消息内容
                                                render_formatted_message(ui, text, available_width - 10.0, true);

                                                // 添加闪烁的光标
                                                let cursor = if (ui.input(|i| i.time) * 2.0).sin() > 0.0 { "▋" } else { " " };
                                                ui.label(egui::RichText::new(cursor).color(egui::Color32::BLACK));
                                            } else {
                                                // 处理消息内容
                                                render_formatted_message(ui, &message.content, available_width - 10.0 , false);
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

                    let _response = ui.add(
                        egui::TextEdit::multiline(&mut state.chat_input)
                            .hint_text("输入消息...")
                            .desired_width(available_width) // 使用全部可用宽度
                            .desired_rows(2)
                            .lock_focus(state.should_focus_chat)
                            // 如果正在发送，禁用输入框
                            .interactive(!state.is_sending)
                    );

                    // 处理键盘输入
                    let enter_pressed = ui.input(|i| i.key_pressed(egui::Key::Enter));
                    let alt_pressed = ui.input(|i| i.modifiers.alt);

                    // Alt+回车表示换行，单独的回车表示发送
                    if !state.is_sending && enter_pressed {
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
                    });
                });
            });
        });

    // 显示设置对话框
    if state.show_ai_settings {
        render_ai_settings(ui.ctx(), state);
    }

    // 显示@命令提示框
    if state.show_at_commands {
        render_at_commands(ui.ctx(), state);
    }

    // 显示Slash命令提示框
    if state.show_slash_commands {
        render_slash_commands(ui.ctx(), state);
    }
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

            ui.label("@search - 引用最近的搜索结果");
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

/// 渲染消息内容，简单处理<think>标签
fn render_formatted_message(ui: &mut egui::Ui, content: &str, max_width: f32, _is_streaming: bool) {
    // 设置最大宽度
    ui.set_max_width(max_width);

    // 直接将所有内容作为普通文本处理
    // 如果内容包含<think>标签，则将其中的文本显示为灰色，否则显示为黑色
    if content.contains("<think>") {
        ui.add(egui::Label::new(
            egui::RichText::new(content)
                .color(egui::Color32::from_rgb(100, 100, 100))
        ).wrap());
    } else {
        ui.add(egui::Label::new(
            egui::RichText::new(content)
                .color(egui::Color32::BLACK)
        ).wrap());
    }
}
