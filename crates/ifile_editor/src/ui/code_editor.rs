//! 基于 egui_code_editor 的代码编辑器组件

use egui;
use egui_code_editor::{CodeEditor, ColorTheme, Syntax};
use crate::state::IFileEditorState;
use crate::core::text_buffer_adapter::SimpleTextBuffer;

/// 渲染代码编辑器
pub fn render_code_editor(ui: &mut egui::Ui, state: &mut IFileEditorState) {
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
                render_code_editor_with_toolbar(ui, state);
            } else {
                render_no_file_message(ui);
            }
        }
    }
}

/// 渲染代码编辑器（工具栏已在主UI中渲染）
fn render_code_editor_with_toolbar(ui: &mut egui::Ui, state: &mut IFileEditorState) {
    // 获取完整的可用区域
    let full_rect = ui.available_rect_before_wrap();
    let mut current_y = full_rect.min.y;

    // 1. 渲染查找替换面板（如果启用）
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

    // 2. 渲染编辑器内容区域
    let editor_rect = egui::Rect::from_min_size(
        egui::pos2(full_rect.min.x, current_y),
        egui::vec2(full_rect.width(), full_rect.height() - (current_y - full_rect.min.y))
    );

    ui.allocate_ui_at_rect(editor_rect, |ui| {
        render_code_editor_widget(ui, state);
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
        if ui.small_button("💾").clicked() {
            if let Err(e) = state.editor.save_active_file() {
                log::error!("Failed to save file: {}", e);
                state.last_error = Some(e);
            } else {
                log::info!("File saved successfully");
            }
        }
        if ui.small_button("↶").clicked() {
            // TODO: 实现撤销功能
            log::info!("Undo");
        }
        if ui.small_button("↷").clicked() {
            // TODO: 实现重做功能
            log::info!("Redo");
        }

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

/// 渲染查找替换面板
fn render_find_replace_panel(ui: &mut egui::Ui, state: &mut IFileEditorState) {
    ui.horizontal(|ui| {
        ui.label("查找:");
        ui.text_edit_singleline(&mut state.ui_state.find_query);

        if ui.button("下一个").clicked() {
            // TODO: 实现查找下一个
            log::info!("Find next: {}", state.ui_state.find_query);
        }

        ui.separator();

        ui.label("替换:");
        ui.text_edit_singleline(&mut state.ui_state.replace_query);

        if ui.button("替换").clicked() {
            // TODO: 实现替换
            log::info!("Replace: {} -> {}", state.ui_state.find_query, state.ui_state.replace_query);
        }

        if ui.button("全部替换").clicked() {
            // TODO: 实现全部替换
            log::info!("Replace all: {} -> {}", state.ui_state.find_query, state.ui_state.replace_query);
        }

        if ui.button("关闭").clicked() {
            state.ui_state.show_find_replace = false;
        }
    });
}

/// 渲染代码编辑器组件
fn render_code_editor_widget(ui: &mut egui::Ui, state: &mut IFileEditorState) {
    // 获取活动缓冲区的信息
    let (file_path, language, read_only, rope_content, cursor_line) = {
        if let Some(buffer) = state.editor.get_active_buffer() {
            (
                buffer.file_path.clone(),
                buffer.language.clone(),
                buffer.read_only,
                buffer.rope.to_string(),
                buffer.cursor.line,
            )
        } else {
            return;
        }
    };

    // 创建简化的文本缓冲区
    let mut simple_buffer = SimpleTextBuffer::new(rope_content, read_only);

    // 确定语法高亮类型
    let syntax = determine_syntax(&file_path, language.as_deref());

    // 确定主题
    let theme = determine_theme(&state.settings.theme);

    // 创建代码编辑器，使用唯一的ID以便控制滚动
    let editor_id = format!("code_editor_{}", file_path.display());
    let mut code_editor = CodeEditor::default()
        .id_source(&editor_id)
        .with_fontsize(state.ui_state.editor_font_size)
        .with_theme(theme)
        .with_syntax(syntax)
        .with_numlines(state.ui_state.show_line_numbers)
        .vscroll(true)
        .auto_shrink(false);

    // 如果启用了自动折行，设置合适的宽度
    if state.ui_state.word_wrap {
        let available_width = ui.available_width();
        code_editor = code_editor.desired_width(available_width);
    } else {
        code_editor = code_editor.desired_width(f32::INFINITY);
    }

    // 显示编辑器
    let response = code_editor.show(ui, &mut simple_buffer);

    // 处理文本变化
    if response.response.changed() {
        // 更新 ROPE 内容
        if let Some(buffer) = state.editor.get_active_buffer_mut() {
            buffer.rope = simple_buffer.to_rope();
            buffer.modified = true;
            log::info!("Text content changed, new length: {}", simple_buffer.text.len());
        }
    }

    // 处理编辑器响应和光标滚动
    handle_editor_response_with_scroll(ui, &response, state, &editor_id);
}

/// 确定语法高亮类型
fn determine_syntax(file_path: &std::path::Path, language: Option<&str>) -> Syntax {
    // 优先使用已检测的语言
    if let Some(lang) = language {
        match lang.to_lowercase().as_str() {
            "rust" => Syntax::rust(),
            "python" => Syntax::python(),
            "lua" => Syntax::lua(),
            "sql" => Syntax::sql(),
            "bash" | "sh" | "shell" => Syntax::shell(),
            "asm" | "assembly" => Syntax::asm(),
            _ => Syntax::default(),
        }
    } else {
        // 根据文件扩展名推断
        match file_path.extension().and_then(|ext| ext.to_str()) {
            Some("rs") => Syntax::rust(),
            Some("py") => Syntax::python(),
            Some("lua") => Syntax::lua(),
            Some("sql") => Syntax::sql(),
            Some("sh" | "bash") => Syntax::shell(),
            Some("asm" | "s") => Syntax::asm(),
            _ => Syntax::default(),
        }
    }
}

/// 确定编辑器主题
fn determine_theme(theme_name: &str) -> ColorTheme {
    match theme_name.to_lowercase().as_str() {
        "dark" | "gruvbox" => ColorTheme::GRUVBOX,
        "light" | "github" => ColorTheme::GITHUB_LIGHT,
        "github_dark" => ColorTheme::GITHUB_DARK,
        "ayu" => ColorTheme::AYU,
        "ayu_dark" => ColorTheme::AYU_DARK,
        "ayu_mirage" => ColorTheme::AYU_MIRAGE,
        "sonokai" => ColorTheme::SONOKAI,
        _ => ColorTheme::GRUVBOX, // 默认主题
    }
}

/// 处理编辑器响应
fn handle_editor_response(
    ui: &mut egui::Ui,
    response: &egui::widgets::text_edit::TextEditOutput,
    state: &mut IFileEditorState,
) {
    // 处理光标位置变化
    if let Some(cursor_range) = response.cursor_range {
        // 更新缓冲区光标位置
        if let Some(buffer) = state.editor.get_active_buffer_mut() {
            let cursor_pos = cursor_range.primary.ccursor.index;
            buffer.cursor.byte_offset = cursor_pos;

            // 计算实际行号和列号
            update_cursor_line_column_from_byte_offset(buffer, cursor_pos);

            // 确保光标在可见区域内（自动滚动）
            // ensure_cursor_visible(ui, response, buffer); // 暂时注释掉，需要实现
        }
    }

    // 处理键盘快捷键
    handle_editor_shortcuts(ui, state);
}

/// 根据字节偏移更新光标的行列位置
fn update_cursor_line_column_from_byte_offset(buffer: &mut crate::state::TextBuffer, byte_offset: usize) {
    let text = buffer.rope.to_string();
    let mut line = 0;
    let mut column = 0;
    let mut current_offset = 0;

    for (i, ch) in text.char_indices() {
        if i >= byte_offset {
            break;
        }

        if ch == '\n' {
            line += 1;
            column = 0;
        } else {
            column += 1;
        }
        current_offset = i + ch.len_utf8();
    }

    buffer.cursor.line = line;
    buffer.cursor.column = column;
    buffer.cursor.byte_offset = byte_offset;
}

/// 处理编辑器响应和光标滚动
fn handle_editor_response_with_scroll(
    ui: &mut egui::Ui,
    response: &egui::widgets::text_edit::TextEditOutput,
    state: &mut IFileEditorState,
    editor_id: &str,
) {
    // 处理光标位置变化
    if let Some(cursor_range) = response.cursor_range {
        // 更新缓冲区光标位置
        if let Some(buffer) = state.editor.get_active_buffer_mut() {
            let cursor_pos = cursor_range.primary.ccursor.index;
            buffer.cursor.byte_offset = cursor_pos;

            // 计算实际行号和列号
            update_cursor_line_column_from_byte_offset(buffer, cursor_pos);

            // 确保光标在可见区域内（自动滚动）
            ensure_cursor_visible_with_scroll_area(ui, response, buffer, editor_id);
        }
    }

    // 处理键盘快捷键
    handle_editor_shortcuts(ui, state);
}

/// 确保光标在可见区域内，使用ScrollArea API控制滚动
fn ensure_cursor_visible_with_scroll_area(
    ui: &mut egui::Ui,
    response: &egui::widgets::text_edit::TextEditOutput,
    buffer: &mut crate::state::TextBuffer,
    editor_id: &str,
) {
    // 获取编辑器的滚动区域ID
    let scroll_area_id = egui::Id::new(format!("{}_outer_scroll", editor_id));

    // 计算光标应该在的位置
    let line_height = buffer.get_line_height_estimate();
    let cursor_line = buffer.cursor.line;
    let target_y = cursor_line as f32 * line_height;

    // 获取当前滚动状态
    if let Some(scroll_area_state) = egui::scroll_area::State::load(ui.ctx(), scroll_area_id) {
        let visible_rect = response.response.rect;
        let visible_height = visible_rect.height();
        let current_scroll_y = scroll_area_state.offset.y;

        // 计算可见区域的范围
        let visible_top = current_scroll_y;
        let visible_bottom = current_scroll_y + visible_height;

        // 设置边距，确保光标不会太靠近边缘
        let margin = line_height * 3.0;

        // 检查是否需要滚动
        let mut new_scroll_y = current_scroll_y;

        // 如果光标在可见区域上方
        if target_y < visible_top + margin {
            new_scroll_y = (target_y - margin).max(0.0);
        }
        // 如果光标在可见区域下方
        else if target_y > visible_bottom - margin {
            new_scroll_y = target_y - visible_height + margin;
        }

        // 如果需要滚动，更新滚动位置
        if (new_scroll_y - current_scroll_y).abs() > 1.0 {
            let mut new_state = scroll_area_state;
            new_state.offset.y = new_scroll_y;
            new_state.store(ui.ctx(), scroll_area_id);
            ui.ctx().request_repaint();

            log::debug!("Scrolling to line {}, target_y: {}, new_scroll_y: {}",
                       cursor_line, target_y, new_scroll_y);
        }
    }
}

/// 处理编辑器快捷键
fn handle_editor_shortcuts(ui: &mut egui::Ui, state: &mut IFileEditorState) {
    let ctx = ui.ctx();
    
    // Ctrl+S 保存
    if ctx.input_mut(|i| i.consume_key(egui::Modifiers::CTRL, egui::Key::S)) {
        log::info!("Save shortcut triggered");
        // TODO: 实现保存功能
    }
    
    // Ctrl+Z 撤销
    if ctx.input_mut(|i| i.consume_key(egui::Modifiers::CTRL, egui::Key::Z)) {
        log::info!("Undo shortcut triggered");
        // TODO: 实现撤销功能
    }
    
    // Ctrl+Y 重做
    if ctx.input_mut(|i| i.consume_key(egui::Modifiers::CTRL, egui::Key::Y)) {
        log::info!("Redo shortcut triggered");
        // TODO: 实现重做功能
    }
}

/// 渲染加载消息
fn render_loading_message(ui: &mut egui::Ui, path: &std::path::Path, progress: f32) {
    ui.vertical_centered(|ui| {
        ui.add_space(50.0);
        ui.label(format!("正在加载文件: {}", path.display()));
        ui.add(egui::ProgressBar::new(progress).show_percentage());
    });
}

/// 渲染错误消息
fn render_error_message(ui: &mut egui::Ui, error: &str) {
    ui.vertical_centered(|ui| {
        ui.add_space(50.0);
        ui.colored_label(egui::Color32::RED, format!("错误: {}", error));
    });
}

/// 渲染无文件消息
fn render_no_file_message(ui: &mut egui::Ui) {
    ui.vertical_centered(|ui| {
        ui.add_space(50.0);
        ui.label("请选择一个文件进行编辑");
        ui.label("在左侧文件树中点击文件名即可打开");
    });
}
