//! 文本编辑器UI组件

use egui;
use crate::state::{IFileEditorState, TextBuffer};
// 语法高亮现在由 egui_code_editor 处理
// use crate::core::syntax::{SyntaxHighlighter, utils};

/// 渲染文本编辑器（优化性能）
pub fn render_editor(ui: &mut egui::Ui, state: &mut IFileEditorState) {
    // 检查是否正在加载文件
    let loading_state = state.editor.loading_state.clone();
    match loading_state {
        crate::state::FileLoadingState::Loading { path, progress } => {
            render_loading_message(ui, &path, progress);
        }
        crate::state::FileLoadingState::Failed(error) => {
            render_error_message(ui, &error);
        }
        _ => {
            // 检查是否有活动缓冲区
            let has_buffer = state.editor.get_active_buffer().is_some();
            if has_buffer {
                render_text_editor_optimized(ui, state);
            } else {
                render_no_file_message(ui);
            }
        }
    }
}

/// 渲染文本编辑器内容
fn render_text_editor(ui: &mut egui::Ui, buffer: &TextBuffer, state: &IFileEditorState) {
    // 主编辑区域（移除了编辑器信息栏，状态信息将显示在主状态栏中）
    let _scroll_area_response = egui::ScrollArea::both()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            render_text_content_interactive(ui, buffer, state)
        });

    // 处理编辑器级别的键盘快捷键
    handle_editor_shortcuts(ui, state);
}

/// 渲染文本编辑器内容（完整编辑功能）
fn render_text_editor_optimized(ui: &mut egui::Ui, state: &mut IFileEditorState) {
    // 获取完整的可用区域
    let full_rect = ui.available_rect_before_wrap();
    let mut current_y = full_rect.min.y;

    // 1. 渲染工具栏（固定高度，无额外间距）
    let toolbar_height = 28.0; // 紧凑的工具栏高度
    let toolbar_rect = egui::Rect::from_min_size(
        egui::pos2(full_rect.min.x, current_y),
        egui::vec2(full_rect.width(), toolbar_height)
    );

    ui.allocate_ui_at_rect(toolbar_rect, |ui| {
        render_editor_toolbar_compact(ui, state);
    });
    current_y += toolbar_height;

    // 2. 渲染查找替换面板（如果启用）
    if state.ui_state.show_find_replace {
        let find_height = 32.0; // 查找面板高度
        let find_rect = egui::Rect::from_min_size(
            egui::pos2(full_rect.min.x, current_y),
            egui::vec2(full_rect.width(), find_height)
        );

        ui.allocate_ui_at_rect(find_rect, |ui| {
            render_find_replace_panel(ui, state);
        });
        current_y += find_height;
    }

    // 3. 渲染编辑区域（使用剩余的所有空间）
    let editor_rect = egui::Rect::from_min_size(
        egui::pos2(full_rect.min.x, current_y),
        egui::vec2(full_rect.width(), full_rect.max.y - current_y)
    );

    ui.allocate_ui_at_rect(editor_rect, |ui| {
        render_text_editor_direct(ui, state, editor_rect);
    });

    // 处理编辑器级别的键盘快捷键
    handle_editor_shortcuts(ui, state);
}

/// 渲染可交互的文本内容
fn render_text_content_interactive(ui: &mut egui::Ui, buffer: &TextBuffer, state: &IFileEditorState) -> egui::Response {
    let available_rect = ui.available_rect_before_wrap();
    let line_height = ui.text_style_height(&egui::TextStyle::Monospace);

    // 计算可见行范围
    let visible_start = (buffer.scroll_offset / line_height) as usize;
    let visible_count = (available_rect.height() / line_height) as usize + 2;
    let visible_end = (visible_start + visible_count).min(buffer.line_count());

    // 创建一个可交互的区域
    let (response, painter) = ui.allocate_painter(available_rect.size(), egui::Sense::click_and_drag());

    // 渲染文本内容
    render_text_lines_with_interaction(ui, &painter, buffer, visible_start, visible_end, line_height, &response);

    response
}

/// 处理编辑器快捷键
fn handle_editor_shortcuts(ui: &mut egui::Ui, state: &IFileEditorState) {
    let ctx = ui.ctx();

    // Ctrl+S: 保存文件
    if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::S)) {
        log::info!("Save shortcut pressed");
        // 这里需要通过某种方式触发保存操作
        // 由于我们只有不可变引用，需要通过事件系统或其他方式处理
    }

    // Ctrl+Z: 撤销
    if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::Z)) {
        log::info!("Undo shortcut pressed");
    }

    // Ctrl+Y: 重做
    if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::Y)) {
        log::info!("Redo shortcut pressed");
    }

    // Ctrl+F: 查找
    if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::F)) {
        log::info!("Find shortcut pressed");
    }
}

/// 渲染编辑器信息栏
fn render_editor_info(ui: &mut egui::Ui, buffer: &TextBuffer) {
    ui.horizontal(|ui| {
        // 文件路径
        ui.label(format!("📄 {}", buffer.file_path.display()));
        
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            // 文件统计信息
            ui.label(format!("行: {} | 字符: {} | 字节: {}", 
                buffer.line_count(),
                buffer.char_count(),
                buffer.byte_count()
            ));
            
            ui.separator();
            
            // 编码信息
            ui.label(&buffer.encoding);
            
            ui.separator();
            
            // 语言信息
            if let Some(lang) = &buffer.language {
                ui.label(lang);
            }
            
            ui.separator();
            
            // 修改状态
            if buffer.modified {
                ui.label("●").on_hover_text("已修改");
            } else {
                ui.label("○").on_hover_text("已保存");
            }
        });
    });
}

/// 渲染文本内容
fn render_text_content(ui: &mut egui::Ui, buffer: &TextBuffer, state: &IFileEditorState) {
    // 计算可见区域
    let available_rect = ui.available_rect_before_wrap();
    let line_height = ui.text_style_height(&egui::TextStyle::Monospace);
    
    // 计算可见行范围
    let visible_start = (buffer.scroll_offset / line_height) as usize;
    let visible_count = (available_rect.height() / line_height) as usize + 2; // 多渲染2行避免闪烁
    let visible_end = (visible_start + visible_count).min(buffer.line_count());
    
    ui.horizontal(|ui| {
        // 行号区域
        if state.settings.show_line_numbers {
            render_line_numbers(ui, visible_start, visible_end, line_height);
            ui.separator();
        }
        
        // 文本内容区域
        render_text_lines_simple(ui, buffer, visible_start, visible_end, line_height);
    });
}

/// 渲染行号
fn render_line_numbers(ui: &mut egui::Ui, start: usize, end: usize, line_height: f32) {
    let line_number_width = format!("{}", end).len() as f32 * 8.0 + 10.0; // 估算宽度
    
    ui.allocate_ui_with_layout(
        egui::vec2(line_number_width, ui.available_height()),
        egui::Layout::top_down(egui::Align::RIGHT),
        |ui| {
            for line_num in start..end {
                ui.allocate_ui_with_layout(
                    egui::vec2(line_number_width, line_height),
                    egui::Layout::right_to_left(egui::Align::Center),
                    |ui| {
                        ui.label(
                            egui::RichText::new(format!("{}", line_num + 1))
                                .color(ui.visuals().weak_text_color())
                                .monospace()
                        );
                    }
                );
            }
        }
    );
}

/// 渲染简单的文本行（临时实现）
fn render_text_lines_simple(ui: &mut egui::Ui, buffer: &TextBuffer, start: usize, end: usize, line_height: f32) {
    ui.allocate_ui_with_layout(
        egui::vec2(ui.available_width(), ui.available_height()),
        egui::Layout::top_down(egui::Align::LEFT),
        |ui| {
            for line_num in start..end {
                if line_num < buffer.line_count() {
                    let line_text = get_line_text(buffer, line_num);

                    ui.allocate_ui_with_layout(
                        egui::vec2(ui.available_width(), line_height),
                        egui::Layout::left_to_right(egui::Align::Center),
                        |ui| {
                            ui.label(egui::RichText::new(line_text.trim_end_matches('\n')).monospace());
                        }
                    );
                } else {
                    // 空行
                    ui.allocate_ui_with_layout(
                        egui::vec2(ui.available_width(), line_height),
                        egui::Layout::left_to_right(egui::Align::Center),
                        |ui| {
                            ui.label(" ");
                        }
                    );
                }
            }
        }
    );
}

/// 渲染带交互的文本行
fn render_text_lines_with_interaction(
    ui: &mut egui::Ui,
    painter: &egui::Painter,
    buffer: &TextBuffer,
    start: usize,
    end: usize,
    line_height: f32,
    response: &egui::Response,
) {
    let rect = response.rect;
    let line_number_width = if buffer.line_count() > 0 {
        format!("{}", buffer.line_count()).len() as f32 * 8.0 + 10.0
    } else {
        50.0
    };

    // 渲染背景
    painter.rect_filled(rect, 0.0, ui.visuals().extreme_bg_color);

    // 渲染行号背景
    let line_number_rect = egui::Rect::from_min_size(
        rect.min,
        egui::vec2(line_number_width, rect.height())
    );
    painter.rect_filled(line_number_rect, 0.0, ui.visuals().faint_bg_color);

    // 渲染分隔线
    painter.line_segment(
        [
            egui::pos2(rect.min.x + line_number_width, rect.min.y),
            egui::pos2(rect.min.x + line_number_width, rect.max.y),
        ],
        egui::Stroke::new(1.0, ui.visuals().weak_text_color())
    );

    // 渲染文本行
    for line_num in start..end {
        let y_offset = (line_num - start) as f32 * line_height;
        let line_rect = egui::Rect::from_min_size(
            egui::pos2(rect.min.x, rect.min.y + y_offset),
            egui::vec2(rect.width(), line_height)
        );

        // 渲染行号
        let line_number_text = format!("{}", line_num + 1);
        let line_number_pos = egui::pos2(
            rect.min.x + line_number_width - 5.0,
            line_rect.center().y - line_height * 0.4
        );
        painter.text(
            line_number_pos,
            egui::Align2::RIGHT_CENTER,
            &line_number_text,
            egui::FontId::monospace(12.0),
            ui.visuals().weak_text_color()
        );

        // 渲染文本内容
        if line_num < buffer.line_count() {
            let line_text = get_line_text(buffer, line_num);
            let text_pos = egui::pos2(
                rect.min.x + line_number_width + 5.0,
                line_rect.center().y - line_height * 0.4
            );

            // 高亮当前行（如果光标在此行）
            if buffer.cursor.line == line_num {
                let highlight_rect = egui::Rect::from_min_size(
                    egui::pos2(rect.min.x + line_number_width, line_rect.min.y),
                    egui::vec2(rect.width() - line_number_width, line_height)
                );
                painter.rect_filled(highlight_rect, 0.0, ui.visuals().selection.bg_fill.gamma_multiply(0.3));
            }

            // 渲染文本（简化版，不使用语法高亮以避免复杂性）
            painter.text(
                text_pos,
                egui::Align2::LEFT_CENTER,
                &line_text.trim_end_matches('\n'),
                egui::FontId::monospace(14.0),
                ui.visuals().text_color()
            );

            // 渲染光标
            if buffer.cursor.line == line_num {
                let cursor_x = text_pos.x + (buffer.cursor.column as f32 * 8.0); // 估算字符宽度
                painter.line_segment(
                    [
                        egui::pos2(cursor_x, line_rect.min.y + 2.0),
                        egui::pos2(cursor_x, line_rect.max.y - 2.0),
                    ],
                    egui::Stroke::new(2.0, ui.visuals().text_color())
                );
            }
        }
    }

    // 处理鼠标点击
    if response.clicked() {
        if let Some(click_pos) = response.interact_pointer_pos() {
            handle_text_click(click_pos, rect, line_number_width, line_height, start, buffer);
        }
    }
}

/// 处理文本区域的鼠标点击
fn handle_text_click(
    click_pos: egui::Pos2,
    text_rect: egui::Rect,
    line_number_width: f32,
    line_height: f32,
    visible_start: usize,
    buffer: &TextBuffer,
) {
    // 计算点击的行
    let relative_y = click_pos.y - text_rect.min.y;
    let clicked_line = visible_start + (relative_y / line_height) as usize;

    // 计算点击的列（简化计算）
    let text_x = click_pos.x - text_rect.min.x - line_number_width - 5.0;
    let clicked_column = (text_x / 8.0).max(0.0) as usize; // 估算字符宽度

    log::info!("Text clicked at line: {}, column: {}", clicked_line, clicked_column);

    // 这里需要通过某种方式更新光标位置
    // 由于我们只有不可变引用，需要通过事件系统处理
}

/// 获取指定行的文本
fn get_line_text(buffer: &TextBuffer, line_num: usize) -> String {
    if line_num < buffer.line_count() {
        let line_slice = buffer.rope.line(line_num);
        line_slice.to_string()
    } else {
        String::new()
    }
}

/// 渲染带语法高亮的行 (已弃用 - 现在使用 egui_code_editor)
fn render_line_with_syntax_highlighting(ui: &mut egui::Ui, line_text: &str, _buffer: &TextBuffer) {
    // 语法高亮现在由 egui_code_editor 处理
    // 这里只需要简单渲染文本
    render_plain_text(ui, line_text);
}

/// 使用 syntect 渲染语法高亮 (已弃用 - 现在使用 egui_code_editor)
fn render_syntect_highlighting(_ui: &mut egui::Ui, _line_text: &str, _language: &str) -> Result<(), Box<dyn std::error::Error>> {
    // 这个函数已经被 egui_code_editor 替代
    // 保留函数签名以避免编译错误，但不执行任何操作
    Err("Syntect highlighting is deprecated, use egui_code_editor instead".into())
}

/// 渲染纯文本
fn render_plain_text(ui: &mut egui::Ui, text: &str) {
    ui.label(egui::RichText::new(text).monospace());
}

/// 简单的Rust语法高亮
fn render_rust_syntax(ui: &mut egui::Ui, text: &str) {
    let keywords = ["fn", "let", "mut", "pub", "struct", "enum", "impl", "trait", "use", "mod"];
    
    let mut current_text = String::new();
    let mut in_string = false;
    let mut in_comment = false;
    
    for word in text.split_whitespace() {
        if word.starts_with("//") {
            in_comment = true;
        }
        
        if word.starts_with('"') || word.ends_with('"') {
            in_string = !in_string;
        }
        
        if in_comment {
            current_text.push_str(&format!("{} ", word));
        } else if in_string {
            current_text.push_str(&format!("{} ", word));
        } else if keywords.contains(&word) {
            if !current_text.is_empty() {
                ui.label(egui::RichText::new(&current_text).monospace());
                current_text.clear();
            }
            ui.label(egui::RichText::new(word).color(egui::Color32::from_rgb(86, 156, 214)).monospace());
            ui.label(" ");
        } else {
            current_text.push_str(&format!("{} ", word));
        }
    }
    
    if !current_text.is_empty() {
        if in_comment {
            ui.label(egui::RichText::new(&current_text).color(egui::Color32::from_rgb(106, 153, 85)).monospace());
        } else if in_string {
            ui.label(egui::RichText::new(&current_text).color(egui::Color32::from_rgb(206, 145, 120)).monospace());
        } else {
            ui.label(egui::RichText::new(&current_text).monospace());
        }
    }
}

/// 简单的Python语法高亮
fn render_python_syntax(ui: &mut egui::Ui, text: &str) {
    let keywords = ["def", "class", "import", "from", "if", "else", "elif", "for", "while", "try", "except"];
    
    // 简化实现，类似Rust
    render_simple_syntax(ui, text, &keywords);
}

/// 简单的JavaScript语法高亮
fn render_javascript_syntax(ui: &mut egui::Ui, text: &str) {
    let keywords = ["function", "var", "let", "const", "if", "else", "for", "while", "return", "class"];
    
    render_simple_syntax(ui, text, &keywords);
}

/// 简单的Markdown语法高亮
fn render_markdown_syntax(ui: &mut egui::Ui, text: &str) {
    if text.starts_with('#') {
        // 标题
        ui.label(egui::RichText::new(text).color(egui::Color32::from_rgb(86, 156, 214)).monospace().strong());
    } else if text.starts_with("```") {
        // 代码块
        ui.label(egui::RichText::new(text).color(egui::Color32::from_rgb(206, 145, 120)).monospace());
    } else {
        render_plain_text(ui, text);
    }
}

/// 简单的语法高亮通用实现
fn render_simple_syntax(ui: &mut egui::Ui, text: &str, keywords: &[&str]) {
    let mut current_text = String::new();
    
    for word in text.split_whitespace() {
        if keywords.contains(&word) {
            if !current_text.is_empty() {
                ui.label(egui::RichText::new(&current_text).monospace());
                current_text.clear();
            }
            ui.label(egui::RichText::new(word).color(egui::Color32::from_rgb(86, 156, 214)).monospace());
            ui.label(" ");
        } else {
            current_text.push_str(&format!("{} ", word));
        }
    }
    
    if !current_text.is_empty() {
        ui.label(egui::RichText::new(&current_text).monospace());
    }
}

/// 渲染无文件消息
fn render_no_file_message(ui: &mut egui::Ui) {
    ui.centered_and_justified(|ui| {
        ui.label("📄 选择一个文件开始编辑");
    });
}

/// 渲染文件加载消息
fn render_loading_message(ui: &mut egui::Ui, path: &std::path::PathBuf, progress: f32) {
    ui.centered_and_justified(|ui| {
        ui.vertical_centered(|ui| {
            ui.label("⏳ 正在加载文件...");
            ui.label(format!("📁 {}", path.file_name().unwrap_or_default().to_string_lossy()));

            // 显示进度条
            let progress_bar = egui::ProgressBar::new(progress)
                .desired_width(200.0)
                .text(format!("{:.1}%", progress * 100.0));
            ui.add(progress_bar);

            ui.label("💡 大文件加载中，请稍候...");
        });
    });
}

/// 渲染错误消息
fn render_error_message(ui: &mut egui::Ui, error: &str) {
    ui.centered_and_justified(|ui| {
        ui.vertical_centered(|ui| {
            ui.label("❌ 文件加载失败");
            ui.label(format!("错误: {}", error));
            ui.label("💡 请检查文件是否存在或权限是否正确");
        });
    });
}

/// 渲染编辑器工具栏
fn render_editor_toolbar(ui: &mut egui::Ui, state: &mut IFileEditorState) {
    // 使用紧凑的水平布局
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing.x = 4.0; // 减少按钮间距

        // 文件操作按钮
        if ui.small_button("💾 保存").clicked() {
            log::info!("Save button clicked");
        }

        ui.separator();

        // 编辑操作按钮
        if ui.small_button("↶ 撤销").clicked() {
            log::info!("Undo button clicked");
        }

        if ui.small_button("↷ 重做").clicked() {
            log::info!("Redo button clicked");
        }

        ui.separator();

        // 编辑功能按钮
        if ui.small_button("📋 复制").clicked() {
            log::info!("Copy button clicked");
        }

        if ui.small_button("📄 粘贴").clicked() {
            log::info!("Paste button clicked");
        }

        ui.separator();

        // 视图选项按钮（可切换状态）
        let wrap_text = if state.ui_state.word_wrap { "📏 折行 ✓" } else { "📏 折行" };
        if ui.small_button(wrap_text).clicked() {
            state.ui_state.word_wrap = !state.ui_state.word_wrap;
            log::info!("Word wrap toggled: {}", state.ui_state.word_wrap);
        }

        let line_numbers_text = if state.ui_state.show_line_numbers { "🔢 行号 ✓" } else { "🔢 行号" };
        if ui.small_button(line_numbers_text).clicked() {
            state.ui_state.show_line_numbers = !state.ui_state.show_line_numbers;
            log::info!("Line numbers toggled: {}", state.ui_state.show_line_numbers);
        }

        let auto_save_text = if state.ui_state.auto_save { "🔄 自动保存 ✓" } else { "🔄 自动保存" };
        if ui.small_button(auto_save_text).clicked() {
            state.ui_state.auto_save = !state.ui_state.auto_save;
            log::info!("Auto save toggled: {}", state.ui_state.auto_save);
        }

        ui.separator();

        // 搜索功能按钮
        if ui.small_button("🔍 查找").clicked() {
            state.ui_state.show_find_replace = !state.ui_state.show_find_replace;
            log::info!("Find/Replace panel toggled: {}", state.ui_state.show_find_replace);
        }

        // 字体大小调整
        ui.separator();
        ui.label("字体:");
        if ui.small_button("🔍+").clicked() {
            state.ui_state.editor_font_size = (state.ui_state.editor_font_size + 1.0).min(24.0);
            log::info!("Font size increased to: {}", state.ui_state.editor_font_size);
        }
        if ui.small_button("🔍-").clicked() {
            state.ui_state.editor_font_size = (state.ui_state.editor_font_size - 1.0).max(8.0);
            log::info!("Font size decreased to: {}", state.ui_state.editor_font_size);
        }
    });
}

/// 渲染紧凑的编辑器工具栏
fn render_editor_toolbar_compact(ui: &mut egui::Ui, state: &mut IFileEditorState) {
    // 设置紧凑的样式
    ui.spacing_mut().item_spacing.x = 2.0;
    ui.spacing_mut().item_spacing.y = 2.0;
    ui.spacing_mut().button_padding = egui::vec2(4.0, 2.0);

    ui.horizontal(|ui| {
        // 核心按钮（更紧凑）
        if ui.small_button("💾").clicked() { log::info!("Save"); }
        if ui.small_button("↶").clicked() { log::info!("Undo"); }
        if ui.small_button("↷").clicked() { log::info!("Redo"); }

        ui.separator();

        // 视图切换
        let wrap_icon = if state.ui_state.word_wrap { "📏✓" } else { "📏" };
        if ui.small_button(wrap_icon).clicked() {
            state.ui_state.word_wrap = !state.ui_state.word_wrap;
        }

        let line_icon = if state.ui_state.show_line_numbers { "🔢✓" } else { "🔢" };
        if ui.small_button(line_icon).clicked() {
            state.ui_state.show_line_numbers = !state.ui_state.show_line_numbers;
        }

        ui.separator();

        if ui.small_button("🔍").clicked() {
            state.ui_state.show_find_replace = !state.ui_state.show_find_replace;
        }

        if ui.small_button("+").clicked() {
            state.ui_state.editor_font_size = (state.ui_state.editor_font_size + 1.0).min(24.0);
        }
        if ui.small_button("-").clicked() {
            state.ui_state.editor_font_size = (state.ui_state.editor_font_size - 1.0).max(8.0);
        }
    });
}

/// 渲染可编辑文本区域（简化版本）
fn render_editable_text_area_simple(ui: &mut egui::Ui, state: &mut IFileEditorState, available_rect: egui::Rect) {
    // 获取当前活动缓冲区的文本内容
    if let Some(buffer) = state.editor.get_active_buffer() {
        let mut text = buffer.rope.to_string();
        let line_count = buffer.line_count();
        let word_wrap = state.ui_state.word_wrap;
        let show_line_numbers = state.ui_state.show_line_numbers;
        let font_size = state.ui_state.editor_font_size;

        // 使用滚动区域包装整个编辑器
        egui::ScrollArea::both()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                // 如果显示行号，使用水平布局
                if show_line_numbers {
                    ui.horizontal(|ui| {
                        // 行号区域
                        let line_number_width = (line_count.to_string().len() as f32 * 8.0).max(40.0);

                        ui.allocate_ui_with_layout(
                            egui::vec2(line_number_width, available_rect.height()),
                            egui::Layout::top_down(egui::Align::RIGHT),
                            |ui| {
                                ui.style_mut().visuals.extreme_bg_color = egui::Color32::from_gray(240);
                                // 计算可见行数
                                let line_height = ui.text_style_height(&egui::TextStyle::Monospace);
                                let visible_lines = (available_rect.height() / line_height) as usize + 5;

                                for line_num in 1..=line_count.min(visible_lines) {
                                    ui.label(
                                        egui::RichText::new(format!("{:4}", line_num))
                                            .monospace()
                                            .size(font_size)
                                            .color(egui::Color32::from_gray(128))
                                    );
                                }
                            }
                        );

                        ui.separator();

                        // 文本编辑区域
                        render_text_edit_widget_simple(ui, &mut text, word_wrap, available_rect.width() - line_number_width - 10.0, available_rect.height());
                    });
                } else {
                    // 无行号的文本编辑区域
                    render_text_edit_widget_simple(ui, &mut text, word_wrap, available_rect.width(), available_rect.height());
                }
            });
    }
}

/// 渲染文本编辑组件
fn render_text_edit_widget(ui: &mut egui::Ui, text: &mut String, word_wrap: bool, width: f32, height: f32) {
    // 根据设置创建可编辑的文本区域
    let mut text_edit = egui::TextEdit::multiline(text)
        .font(egui::TextStyle::Monospace)
        .lock_focus(false)  // 允许失去焦点
        .cursor_at_end(false);  // 不自动移动到末尾

    // 应用自动折行设置
    if word_wrap {
        text_edit = text_edit.desired_width(width);
    } else {
        text_edit = text_edit.desired_width(f32::INFINITY);
    }

    // 使用完整的可用空间，不设置固定行数
    let response = ui.add_sized([width, height], text_edit);

    // 处理文本编辑响应
    if response.changed() {
        log::info!("Text content changed, new length: {}", text.len());
        // TODO: 更新buffer内容
    }

    // 处理键盘快捷键
    if response.has_focus() {
        let ctx = response.ctx;
        if ctx.input(|i| i.key_pressed(egui::Key::S) && i.modifiers.ctrl) {
            log::info!("Ctrl+S pressed - Save shortcut");
        }
        if ctx.input(|i| i.key_pressed(egui::Key::Z) && i.modifiers.ctrl) {
            log::info!("Ctrl+Z pressed - Undo shortcut");
        }
        if ctx.input(|i| i.key_pressed(egui::Key::Y) && i.modifiers.ctrl) {
            log::info!("Ctrl+Y pressed - Redo shortcut");
        }
    }
}

/// 简化的文本编辑组件
fn render_text_edit_widget_simple(ui: &mut egui::Ui, text: &mut String, word_wrap: bool, width: f32, height: f32) {
    // 根据设置创建可编辑的文本区域
    let mut text_edit = egui::TextEdit::multiline(text)
        .font(egui::TextStyle::Monospace)
        .lock_focus(false)  // 允许失去焦点
        .cursor_at_end(false);  // 不自动移动到末尾

    // 应用自动折行设置
    if word_wrap {
        text_edit = text_edit.desired_width(width);
    } else {
        text_edit = text_edit.desired_width(f32::INFINITY);
    }

    // 直接添加文本编辑器，使用完整空间
    let response = ui.add_sized([width, height], text_edit);

    // 处理文本编辑响应
    if response.changed() {
        log::info!("Text content changed, new length: {}", text.len());
        // TODO: 更新buffer内容
    }

    // 处理键盘快捷键
    if response.has_focus() {
        let ctx = response.ctx;
        if ctx.input(|i| i.key_pressed(egui::Key::S) && i.modifiers.ctrl) {
            log::info!("Ctrl+S pressed - Save shortcut");
        }
        if ctx.input(|i| i.key_pressed(egui::Key::Z) && i.modifiers.ctrl) {
            log::info!("Ctrl+Z pressed - Undo shortcut");
        }
        if ctx.input(|i| i.key_pressed(egui::Key::Y) && i.modifiers.ctrl) {
            log::info!("Ctrl+Y pressed - Redo shortcut");
        }
    }
}

/// 直接渲染文本编辑器（精确矩形分配，同步滚动）
fn render_text_editor_direct(ui: &mut egui::Ui, state: &mut IFileEditorState, editor_rect: egui::Rect) {
    if let Some(buffer) = state.editor.get_active_buffer() {
        let mut text = buffer.rope.to_string();
        let word_wrap = state.ui_state.word_wrap;
        let show_line_numbers = state.ui_state.show_line_numbers;
        let font_size = state.ui_state.editor_font_size;
        let line_count = buffer.line_count();

        // 如果显示行号，使用精确的矩形分配和同步滚动
        if show_line_numbers {
            let line_number_width = (line_count.to_string().len() as f32 * 8.0).max(40.0);

            // 行号区域矩形
            let line_numbers_rect = egui::Rect::from_min_size(
                editor_rect.min,
                egui::vec2(line_number_width, editor_rect.height())
            );

            // 文本编辑区域矩形
            let text_edit_rect = egui::Rect::from_min_size(
                egui::pos2(editor_rect.min.x + line_number_width + 2.0, editor_rect.min.y),
                egui::vec2(editor_rect.width() - line_number_width - 2.0, editor_rect.height())
            );

            // 直接在编辑器矩形内渲染，避免ScrollArea引入的空白
            ui.allocate_ui_at_rect(editor_rect, |ui| {
                // 设置无边距的样式
                ui.style_mut().spacing.item_spacing = egui::vec2(5.0, 5.0);
                ui.style_mut().spacing.indent = 0.0;

                egui::ScrollArea::both()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        // 设置无边距的样式
                        ui.style_mut().spacing.item_spacing = egui::vec2(5.0, 5.0);
                        ui.style_mut().spacing.indent = 0.0;

                        ui.horizontal(|ui| {
                            // 行号区域
                            ui.allocate_ui_with_layout(
                                egui::vec2(line_number_width, editor_rect.height()),
                                egui::Layout::top_down(egui::Align::RIGHT),
                                |ui| {
                                    ui.style_mut().visuals.extreme_bg_color = egui::Color32::from_gray(240);
                                    ui.style_mut().spacing.item_spacing = egui::vec2(0.0, 0.0);

                                    // 渲染所有行号，使用与文本编辑器完全相同的字体
                                    let font_id = egui::FontId::monospace(font_size);
                                    for line_num in 1..=line_count {
                                        ui.label(
                                            egui::RichText::new(format!("{:4}", line_num))
                                                .font(font_id.clone())
                                                .color(egui::Color32::from_gray(128))
                                        );
                                    }
                                }
                            );

                            ui.separator();

                            // 文本编辑区域（无独立滚动）
                            render_text_edit_widget_no_scroll(ui, &mut text, word_wrap, text_edit_rect.width(), font_size);
                        });
                    });
            });
        } else {
            // 无行号，使用精确矩形分配
            ui.allocate_ui_at_rect(editor_rect, |ui| {
                render_text_edit_at_rect(ui, &mut text, word_wrap, editor_rect, font_size);
            });
        }
    }
}

/// 在指定矩形区域渲染文本编辑器
fn render_text_edit_at_rect(ui: &mut egui::Ui, text: &mut String, word_wrap: bool, rect: egui::Rect, font_size: f32) {
    // 使用滚动区域包装文本编辑器
    egui::ScrollArea::both()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            // 创建自定义字体ID，确保字体大小一致
            let font_id = egui::FontId::monospace(font_size);

            // 创建文本编辑器
            let mut text_edit = egui::TextEdit::multiline(text)
                .font(font_id)
                .lock_focus(false)
                .cursor_at_end(false);

            // 应用自动折行设置
            if word_wrap {
                text_edit = text_edit.desired_width(rect.width() - 20.0); // 为滚动条留出空间
            } else {
                text_edit = text_edit.desired_width(f32::INFINITY);
            }

            let response = ui.add(text_edit);

            // 处理响应
            if response.changed() {
                log::info!("Text changed, length: {}", text.len());
            }

            if response.has_focus() {
                let ctx = response.ctx;
                if ctx.input(|i| i.key_pressed(egui::Key::S) && i.modifiers.ctrl) {
                    log::info!("Ctrl+S - Save");
                }
            }
        });
}

/// 渲染文本编辑器（无滚动，用于同步滚动场景）
fn render_text_edit_widget_no_scroll(ui: &mut egui::Ui, text: &mut String, word_wrap: bool, width: f32, font_size: f32) {
    // 设置无边距的样式
    ui.style_mut().spacing.item_spacing = egui::vec2(5.0, 5.0);
    ui.style_mut().spacing.indent = 0.0;

    // 创建自定义字体ID，确保与行号字体大小一致
    let font_id = egui::FontId::monospace(font_size);

    // 创建文本编辑器（无滚动包装）
    let mut text_edit = egui::TextEdit::multiline(text)
        .font(font_id)
        .lock_focus(false)
        .cursor_at_end(false);

    // 应用自动折行设置
    if word_wrap {
        text_edit = text_edit.desired_width(width);
    } else {
        text_edit = text_edit.desired_width(f32::INFINITY);
    }

    let response = ui.add(text_edit);

    // 处理响应
    if response.changed() {
        log::info!("Text changed, length: {}", text.len());
    }

    if response.has_focus() {
        let ctx = response.ctx;
        if ctx.input(|i| i.key_pressed(egui::Key::S) && i.modifiers.ctrl) {
            log::info!("Ctrl+S - Save");
        }
        if ctx.input(|i| i.key_pressed(egui::Key::Z) && i.modifiers.ctrl) {
            log::info!("Ctrl+Z - Undo");
        }
        if ctx.input(|i| i.key_pressed(egui::Key::Y) && i.modifiers.ctrl) {
            log::info!("Ctrl+Y - Redo");
        }
    }
}



/// 简化的文本渲染（小文件）- 保留作为备用
fn render_text_simple(ui: &mut egui::Ui, buffer: &TextBuffer) {
    egui::ScrollArea::both()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            // 直接显示所有文本，使用只读模式以提高性能
            let text = buffer.rope.to_string();

            // 使用Label而不是TextEdit来显示只读内容，性能更好
            ui.add(
                egui::Label::new(
                    egui::RichText::new(text)
                        .monospace()
                        .size(12.0)
                )
                .selectable(true)
            );
        });
}



/// 渲染查找替换面板
fn render_find_replace_panel(ui: &mut egui::Ui, state: &mut IFileEditorState) {
    ui.separator();

    egui::Frame::none()
        .fill(egui::Color32::from_gray(245))
        .inner_margin(egui::Margin::same(8.0))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label("🔍 查找:");
                ui.text_edit_singleline(&mut state.ui_state.find_query);

                if ui.button("下一个").clicked() {
                    log::info!("Find next: {}", state.ui_state.find_query);
                }

                if ui.button("上一个").clicked() {
                    log::info!("Find previous: {}", state.ui_state.find_query);
                }

                ui.separator();

                ui.label("🔄 替换:");
                ui.text_edit_singleline(&mut state.ui_state.replace_query);

                if ui.button("替换").clicked() {
                    log::info!("Replace: {} -> {}", state.ui_state.find_query, state.ui_state.replace_query);
                }

                if ui.button("全部替换").clicked() {
                    log::info!("Replace all: {} -> {}", state.ui_state.find_query, state.ui_state.replace_query);
                }

                ui.separator();

                if ui.button("❌ 关闭").clicked() {
                    state.ui_state.show_find_replace = false;
                }
            });
        });
}

/// 虚拟滚动文本渲染（大文件）- 保留作为备用
fn render_text_virtual_scroll(ui: &mut egui::Ui, buffer: &TextBuffer, available_rect: egui::Rect) {
    let line_height = ui.text_style_height(&egui::TextStyle::Monospace);
    let visible_lines = (available_rect.height() / line_height) as usize + 2;

    egui::ScrollArea::both()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            // 只渲染可见区域的文本
            let start_line = 0; // 简化：从第一行开始显示
            let end_line = visible_lines.min(buffer.line_count());

            // 渲染可见行
            for line_idx in start_line..end_line {
                if let Some(line) = buffer.rope.lines().nth(line_idx) {
                    ui.label(
                        egui::RichText::new(format!("{:4} | {}", line_idx + 1, line.to_string()))
                            .monospace()
                            .size(12.0)
                    );
                }
            }

            // 如果还有更多行，显示提示
            if buffer.line_count() > end_line {
                ui.label(
                    egui::RichText::new(format!("... 还有 {} 行", buffer.line_count() - end_line))
                        .italics()
                        .color(egui::Color32::GRAY)
                );
            }
        });
}
