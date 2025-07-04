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
                render_code_editor_widget(ui, state);
            } else {
                render_no_file_message(ui);
            }
        }
    }
}

/// 渲染代码编辑器组件
fn render_code_editor_widget(ui: &mut egui::Ui, state: &mut IFileEditorState) {
    // 获取活动缓冲区的信息
    let (file_path, language, read_only, rope_content) = {
        if let Some(buffer) = state.editor.get_active_buffer() {
            (
                buffer.file_path.clone(),
                buffer.language.clone(),
                buffer.read_only,
                buffer.rope.to_string(),
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

    // 创建代码编辑器
    let mut code_editor = CodeEditor::default()
        .id_source(format!("code_editor_{}", file_path.display()))
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

    // 处理编辑器响应
    handle_editor_response(ui, &response, state);
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
            buffer.cursor.line = 0; // TODO: 计算实际行号
            buffer.cursor.column = cursor_range.primary.ccursor.index;
        }
    }

    // 处理键盘快捷键
    handle_editor_shortcuts(ui, state);
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
