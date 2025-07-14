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
            let old_state = state.ui_state.word_wrap;
            state.ui_state.word_wrap = !state.ui_state.word_wrap;
            log::info!("WORD_WRAP_TOGGLE: {} -> {} (from compact toolbar)", old_state, state.ui_state.word_wrap);
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
    let (file_path, language, read_only, rope_content, _cursor_line, line_count) = {
        if let Some(buffer) = state.editor.get_active_buffer() {
            let line_count = buffer.rope.to_string().lines().count();
            (
                buffer.file_path.clone(),
                buffer.language.clone(),
                buffer.read_only,
                buffer.rope.to_string(),
                buffer.cursor.line,
                line_count,
            )
        } else {
            return;
        }
    };

    // 性能优化：根据文件大小和行数使用不同的渲染策略
    // 大文件判断：内容超过1MB或行数超过阈值
    let content_size_mb = rope_content.len() as f64 / (1024.0 * 1024.0);
    let is_large_file = content_size_mb > 1.0 || line_count > state.settings.virtual_scroll_threshold;

    // if state.settings.large_file_optimization && is_large_file {
        render_large_file_editor(ui, state, &rope_content, &file_path, language.as_deref(), read_only, line_count);
    // } else {
    //     render_standard_file_editor(ui, state, &rope_content, &file_path, language.as_deref(), read_only);
    // }
}

/// 渲染标准文件编辑器（小文件）
fn render_standard_file_editor(
    ui: &mut egui::Ui,
    state: &mut IFileEditorState,
    rope_content: &str,
    file_path: &std::path::PathBuf,
    language: Option<&str>,
    read_only: bool
) {
    // 创建简化的文本缓冲区
    let mut simple_buffer = SimpleTextBuffer::new(rope_content.to_string(), read_only);

    // 确定语法高亮类型
    let syntax = determine_syntax(&file_path, language.as_deref());

    // 确定主题
    let theme = determine_theme(&state.settings.theme);

    // 创建代码编辑器，使用唯一的ID以便控制滚动
    let editor_id = format!("code_editor_{}", file_path.display());
    let line_height = state.ui_state.editor_font_size * state.settings.line_height_factor;

    // 根据折行设置选择不同的编辑器实现
    let available_width = ui.available_width();

    // 根据折行设置选择不同的编辑器实现并处理响应
    if state.ui_state.word_wrap {
        // 折行模式：使用原生 TextEdit，支持真正的文本折行
        log::debug!("WORD_WRAP_DIAGNOSIS: Rendering with TextEdit (wrap enabled)");
        let response = render_wrapped_text_editor(ui, &mut simple_buffer, available_width, &syntax, state.ui_state.editor_font_size, state.ui_state.show_line_numbers, state.settings.line_height_factor);

        // 处理文本变化
        if response.response.changed() {
            if let Some(buffer) = state.editor.get_active_buffer_mut() {
                buffer.rope = simple_buffer.to_rope();
                buffer.modified = true;
            }
        }

        // 处理光标滚动
        handle_editor_response_with_scroll(ui, &response, state, &editor_id);
    } else {
        // 不折行模式：使用自定义的CodeEditor渲染，支持行间距设置
        log::debug!("WORD_WRAP_DIAGNOSIS: Rendering with CodeEditor (wrap disabled)");
        let wide_width = available_width.max(2000.0);

        let response = render_code_editor_with_line_height(
            ui,
            &mut simple_buffer,
            &syntax,
            state.ui_state.editor_font_size,
            line_height,
            state.ui_state.show_line_numbers,
            wide_width,
            &theme,
            &editor_id
        );

        // 处理文本变化
        if response.response.changed() {
            if let Some(buffer) = state.editor.get_active_buffer_mut() {
                buffer.rope = simple_buffer.to_rope();
                buffer.modified = true;
            }
        }

        // 对于标准编辑器，暂时跳过光标滚动处理，因为类型不匹配
    }
}

/// 渲染支持文本折行的编辑器（使用原生 TextEdit + 行号）
fn render_wrapped_text_editor(
    ui: &mut egui::Ui,
    simple_buffer: &mut SimpleTextBuffer,
    available_width: f32,
    syntax: &egui_code_editor::Syntax,
    font_size: f32,
    show_line_numbers: bool,
    line_height_factor: f32,
) -> egui::widgets::text_edit::TextEditOutput {


    // 计算行高
    let line_height = font_size * line_height_factor;

    // 计算行号区域宽度 - 使用更精确的计算方法
    let line_count = simple_buffer.text.lines().count().max(1);
    let line_number_width = if show_line_numbers {
        // 根据最大行号计算宽度，使用更精确的字符宽度估算
        let max_line_digits = line_count.to_string().len().max(3); // 至少3位数
        let char_width = font_size * 0.6; // 等宽字体字符宽度约为字体大小的0.6倍
        (max_line_digits as f32 * char_width) + 16.0 // 增加边距以确保对齐
    } else {
        0.0
    };

    // 计算编辑器可用宽度
    let editor_width = available_width - line_number_width - 20.0; // 为滚动条预留空间

    let mut text_edit_output = None;

    // 使用垂直滚动区域包装整个编辑器，确保可以上下滚动
    egui::ScrollArea::vertical()
        .id_source(format!("{}_wrapped_scroll", "wrapped_editor"))
        .auto_shrink([false, false])
        .show(ui, |ui| {
            // 使用一个特殊的布局来同步行号和文本
            render_wrapped_editor_with_synchronized_line_numbers(
                ui,
                simple_buffer,
                syntax,
                font_size,
                line_height,
                show_line_numbers,
                line_number_width,
                editor_width,
                &mut text_edit_output,
            );
        });

    text_edit_output.expect("TextEditOutput should exist")
}

/// 为折行编辑器渲染行号（智能处理自动折行）
fn render_line_numbers_for_wrapped_editor(
    ui: &mut egui::Ui,
    _text: &str,
    width: f32,
    font_size: f32,
    line_height: f32,
    line_count: usize,
) {
    // 简单的行号渲染，只显示逻辑行号
    let max_digits = line_count.to_string().len().max(3);
    let line_numbers = (1..=line_count)
        .map(|i| format!("{:>width$}", i, width = max_digits))
        .collect::<Vec<_>>()
        .join("\n");

    let mut line_number_text = line_numbers;
    let mut layouter = |ui: &egui::Ui, string: &str, _wrap_width: f32| {
        let layout_job = egui::text::LayoutJob::single_section(
            string.to_string(),
            egui::TextFormat {
                font_id: egui::FontId::monospace(font_size),
                color: ui.style().visuals.weak_text_color(),
                line_height: Some(line_height),
                ..Default::default()
            },
        );
        ui.fonts(|f| f.layout_job(layout_job))
    };

    ui.add(
        egui::TextEdit::multiline(&mut line_number_text)
            .id_source("line_numbers_wrapped")
            .interactive(false)
            .frame(false)
            .desired_width(width)
            .layouter(&mut layouter),
    );
}

/// 渲染带有同步行号的折行编辑器
fn render_wrapped_editor_with_synchronized_line_numbers(
    ui: &mut egui::Ui,
    simple_buffer: &mut SimpleTextBuffer,
    syntax: &egui_code_editor::Syntax,
    font_size: f32,
    line_height: f32,
    show_line_numbers: bool,
    line_number_width: f32,
    editor_width: f32,
    text_edit_output: &mut Option<egui::widgets::text_edit::TextEditOutput>,
) {
    use egui_code_editor::highlighting::highlight;

    let text_width = editor_width;

    // 首先计算文本的折行布局
    let mut layout_job = highlight(ui.ctx(), &egui_code_editor::CodeEditor::default().with_syntax(syntax.clone()), &simple_buffer.text);

    // 设置折行参数
    layout_job.wrap.max_width = text_width;
    layout_job.wrap.max_rows = usize::MAX;
    layout_job.wrap.break_anywhere = false;

    // 应用字体和行间距设置
    for section in &mut layout_job.sections {
        section.format.font_id = egui::FontId::monospace(font_size);
        section.format.line_height = Some(line_height);
    }

    // 计算折行后的布局
    let galley = ui.fonts(|f| f.layout_job(layout_job));

    ui.horizontal_top(|ui| {
        // 1. 渲染同步的行号
        if show_line_numbers {
            render_synchronized_line_numbers(ui, &simple_buffer.text, &galley, line_number_width, font_size, line_height);
        }

        // 2. 渲染文本编辑器
        let mut layouter = |ui: &egui::Ui, string: &str, _wrap_width: f32| {
            // 使用相同的布局计算
            let mut layout_job = highlight(ui.ctx(), &egui_code_editor::CodeEditor::default().with_syntax(syntax.clone()), string);

            layout_job.wrap.max_width = text_width;
            layout_job.wrap.max_rows = usize::MAX;
            layout_job.wrap.break_anywhere = false;

            for section in &mut layout_job.sections {
                section.format.font_id = egui::FontId::monospace(font_size);
                section.format.line_height = Some(line_height);
            }

            ui.fonts(|f| f.layout_job(layout_job))
        };

        let text_edit = egui::TextEdit::multiline(&mut simple_buffer.text)
            .id_source("wrapped_text_editor")
            .font(egui::FontId::monospace(font_size))
            .desired_width(editor_width)
            .layouter(&mut layouter);

        *text_edit_output = Some(text_edit.show(ui));
    });
}

/// 渲染与文本折行同步的行号
fn render_synchronized_line_numbers(
    ui: &mut egui::Ui,
    text: &str,
    galley: &egui::Galley,
    width: f32,
    font_size: f32,
    line_height: f32,
) {
    let line_count = text.lines().count().max(1);
    let max_digits = line_count.to_string().len().max(3);

    // 将文本转换为字符向量，便于精确定位
    let text_chars: Vec<char> = text.chars().collect();

    // 创建逻辑行的起始字符位置映射
    let mut line_start_positions = Vec::new();
    line_start_positions.push(0); // 第一行从位置0开始

    for (char_idx, &ch) in text_chars.iter().enumerate() {
        if ch == '\n' {
            line_start_positions.push(char_idx + 1); // 下一行从换行符后开始
        }
    }

    // 为每个视觉行确定是否显示行号
    let mut line_number_content = String::new();
    let mut char_offset = 0;

    for (visual_row_index, row) in galley.rows.iter().enumerate() {
        if visual_row_index > 0 {
            line_number_content.push('\n');
        }

        // 计算当前视觉行对应的逻辑行号
        let mut logical_line = 1;
        for (line_idx, &start_pos) in line_start_positions.iter().enumerate() {
            if char_offset >= start_pos {
                logical_line = line_idx + 1;
            } else {
                break;
            }
        }

        // 检查这是否是逻辑行的第一个视觉行
        let is_first_visual_row_of_logical_line = if logical_line <= line_start_positions.len() {
            let line_start = line_start_positions.get(logical_line - 1).copied().unwrap_or(0);
            char_offset == line_start
        } else {
            false
        };

        // 只在逻辑行的第一个视觉行显示行号
        if is_first_visual_row_of_logical_line {
            line_number_content.push_str(&format!("{:>width$}", logical_line, width = max_digits));
        } else {
            // 自动折行的续行，显示空白
            line_number_content.push_str(&" ".repeat(max_digits));
        }

        // 更新字符偏移量 - 使用 glyphs 计算字符数
        let row_char_count = row.glyphs.len();
        char_offset += row_char_count;

        // 如果这一行以换行符结束，需要额外计算换行符
        if row.ends_with_newline {
            char_offset += 1;
        }
    }

    // 如果没有内容，显示行号1
    if line_number_content.is_empty() {
        line_number_content = format!("{:>width$}", 1, width = max_digits);
    }

    // 渲染行号
    let mut line_number_text = line_number_content;
    let mut layouter = |ui: &egui::Ui, string: &str, _wrap_width: f32| {
        let layout_job = egui::text::LayoutJob::single_section(
            string.to_string(),
            egui::TextFormat {
                font_id: egui::FontId::monospace(font_size),
                color: ui.style().visuals.weak_text_color(),
                line_height: Some(line_height),
                ..Default::default()
            },
        );
        ui.fonts(|f| f.layout_job(layout_job))
    };

    ui.add(
        egui::TextEdit::multiline(&mut line_number_text)
            .id_source("line_numbers_synchronized")
            .interactive(false)
            .frame(false)
            .desired_width(width)
            .layouter(&mut layouter),
    );
}

/// 渲染支持行间距的CodeEditor（非折行模式）
fn render_code_editor_with_line_height(
    ui: &mut egui::Ui,
    simple_buffer: &mut SimpleTextBuffer,
    syntax: &egui_code_editor::Syntax,
    font_size: f32,
    line_height: f32,
    show_line_numbers: bool,
    desired_width: f32,
    theme: &egui_code_editor::ColorTheme,
    editor_id: &str,
) -> egui::widgets::text_edit::TextEditOutput {
    use egui_code_editor::highlighting::highlight;

    let mut text_edit_output = None;

    // 使用垂直滚动区域包装整个编辑器，确保可以上下滚动
    egui::ScrollArea::vertical()
        .id_source(format!("{}_outer_scroll", editor_id))
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.horizontal_top(|ui| {
                // 1. 渲染行号（如果启用）
                if show_line_numbers {
                    let line_count = simple_buffer.text.lines().count().max(1);
                    let max_digits = line_count.to_string().len().max(3);
                    let char_width = font_size * 0.6;
                    let line_number_width = (max_digits as f32 * char_width) + 16.0;

                    render_line_numbers_for_code_editor(ui, &simple_buffer.text, line_number_width, font_size, line_height, line_count, theme);
                }

                // 2. 渲染代码编辑器
                egui::ScrollArea::horizontal()
                    .id_source(format!("{}_inner_scroll", editor_id))
                    .show(ui, |ui| {
                        // 创建自定义layouter，应用行间距和语法高亮
                        let mut layouter = |ui: &egui::Ui, string: &str, _wrap_width: f32| {
                            let mut layout_job = highlight(ui.ctx(), &egui_code_editor::CodeEditor::default().with_syntax(syntax.clone()), string);

                            // 应用统一的字体和行间距设置到所有文本段
                            for section in &mut layout_job.sections {
                                section.format.font_id = egui::FontId::monospace(font_size); // 确保字体一致
                                section.format.line_height = Some(line_height);
                            }

                            // 设置不折行
                            layout_job.wrap.max_width = f32::INFINITY;
                            layout_job.wrap.max_rows = 1;

                            ui.fonts(|f| f.layout_job(layout_job))
                        };

                        let output = egui::TextEdit::multiline(&mut simple_buffer.text)
                            .id_source(editor_id)
                            .font(egui::FontId::monospace(font_size))
                            .lock_focus(true)
                            .frame(true)
                            .desired_width(desired_width)
                            .layouter(&mut layouter)
                            .show(ui);

                        text_edit_output = Some(output);
                    });
            });
        });

    text_edit_output.expect("TextEditOutput should exist")
}

/// 为CodeEditor渲染行号
fn render_line_numbers_for_code_editor(
    ui: &mut egui::Ui,
    _text: &str,
    width: f32,
    font_size: f32,
    line_height: f32,
    line_count: usize,
    theme: &egui_code_editor::ColorTheme,
) {
    // 生成行号文本
    let max_digits = line_count.to_string().len().max(3);
    let line_numbers = (1..=line_count)
        .map(|i| format!("{:>width$}", i, width = max_digits))
        .collect::<Vec<_>>()
        .join("\n");

    // 创建行号的 layouter
    let mut line_number_text = line_numbers;
    let mut layouter = |ui: &egui::Ui, string: &str, _wrap_width: f32| {
        let layout_job = egui::text::LayoutJob::single_section(
            string.to_string(),
            egui::TextFormat {
                font_id: egui::FontId::monospace(font_size),
                color: theme.type_color(egui_code_editor::TokenType::Comment(true)),
                line_height: Some(line_height),
                ..Default::default()
            },
        );
        ui.fonts(|f| f.layout_job(layout_job))
    };

    ui.add(
        egui::TextEdit::multiline(&mut line_number_text)
            .id_source(format!("{}_numlines", "code_editor"))
            .interactive(false)
            .frame(false)
            .desired_width(width)
            .layouter(&mut layouter),
    );
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
    let mut _current_offset = 0;

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
        _current_offset = i + ch.len_utf8();
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
        }
    }
}

/// 处理编辑器快捷键
fn handle_editor_shortcuts(ui: &mut egui::Ui, _state: &mut IFileEditorState) {
    let ctx = ui.ctx();

    // Ctrl+S 保存
    if ctx.input_mut(|i| i.consume_key(egui::Modifiers::CTRL, egui::Key::S)) {
        // TODO: 实现保存功能
    }

    // Ctrl+Z 撤销
    if ctx.input_mut(|i| i.consume_key(egui::Modifiers::CTRL, egui::Key::Z)) {
        // TODO: 实现撤销功能
    }

    // Ctrl+Y 重做
    if ctx.input_mut(|i| i.consume_key(egui::Modifiers::CTRL, egui::Key::Y)) {
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

/// 安全地截断字符串，确保不会在UTF-8字符中间截断
fn safe_string_truncate(s: &str, max_bytes: usize) -> &str {
    if s.len() <= max_bytes {
        return s;
    }

    // 从目标位置向前查找，找到安全的字符边界
    let mut truncate_pos = max_bytes;
    while truncate_pos > 0 && !s.is_char_boundary(truncate_pos) {
        truncate_pos -= 1;
    }

    &s[..truncate_pos]
}

/// 渲染大文件编辑器（性能优化版本）
fn render_large_file_editor(
    ui: &mut egui::Ui,
    state: &mut IFileEditorState,
    rope_content: &str,
    file_path: &std::path::PathBuf,
    language: Option<&str>,
    read_only: bool,
    line_count: usize
) {
    // 显示大文件警告和文件大小信息
    let content_size_mb = rope_content.len() as f64 / (1024.0 * 1024.0);
    ui.horizontal(|ui| {
        ui.label("⚠️");
        // if content_size_mb > 1.0 {
        //     ui.label(format!("大文件模式 ({:.1} MB, {} 行)", content_size_mb, line_count));
        // } else {
        //     ui.label(format!("大文件模式 ({} 行)", line_count));
        // }
        if state.settings.syntax_highlighting {
            ui.label(" 语法高亮已禁用以提高性能");
        }
    });
    ui.separator();

    // 对于大文件，禁用语法高亮以提高性能
    let syntax = if state.settings.syntax_highlighting && line_count < state.settings.max_rendered_lines * 2 {
        determine_syntax(file_path, language)
    } else {
        Syntax::default() // 纯文本模式
    };

    // 创建简化的文本缓冲区，但限制内容大小
    let optimized_content = if content_size_mb > 2.0 { // 超过2MB截断
        // 对于超大文件，只显示前面部分，确保字符边界安全
        let truncate_size = 1024 * 1024 * 2; // 截断到2MB
        let safe_truncated = safe_string_truncate(rope_content, truncate_size);
        format!("{}\n\n[文件内容已截断以提高性能，显示前 2MB 内容...]", safe_truncated)
    } else {
        rope_content.to_string()
    };

    let mut simple_buffer = SimpleTextBuffer::new(optimized_content, read_only);

    // 确定主题
    let theme = determine_theme(&state.settings.theme);

    // 创建代码编辑器，使用性能优化设置
    let editor_id = format!("large_code_editor_{}", file_path.display());
    let mut code_editor = CodeEditor::default()
        .id_source(&editor_id)
        .with_fontsize(state.ui_state.editor_font_size)
        .with_theme(theme)
        .with_syntax(syntax.clone())
        .with_numlines(state.ui_state.show_line_numbers)
        .vscroll(true)
        .auto_shrink(false);

    // 对于大文件，根据用户设置决定是否折行
    let available_width = ui.available_width();
    let response = if state.ui_state.word_wrap {
        // 折行模式：使用混合方案（TextEdit + 行号），支持真正的文本折行
        log::debug!("WORD_WRAP_DIAGNOSIS: Large file - wrap enabled using TextEdit hybrid");
        render_wrapped_text_editor(ui, &mut simple_buffer, available_width, &syntax, state.ui_state.editor_font_size, state.ui_state.show_line_numbers, state.settings.line_height_factor)
    } else {
        // 非折行模式：使用 CodeEditor，设置较大宽度以启用水平滚动
        log::debug!("WORD_WRAP_DIAGNOSIS: Large file - wrap disabled using CodeEditor");
        let wide_width = available_width.max(2000.0);
        code_editor = code_editor.desired_width(wide_width);
        code_editor.show(ui, &mut simple_buffer)
    };

    // 处理文本变化（大文件模式下限制编辑功能）
    if response.response.changed() && !read_only {
        // 对于大文件，显示警告
        if content_size_mb > 1.0 {
            log::warn!("Large file editing detected ({:.1} MB), changes may impact performance", content_size_mb);
        }

        // 更新 ROPE 内容
        if let Some(buffer) = state.editor.get_active_buffer_mut() {
            buffer.rope = simple_buffer.to_rope();
            buffer.modified = true;
        }
    }

    // 显示性能提示
    ui.separator();
    ui.horizontal(|ui| {
        ui.label("💡 提示:");
        if content_size_mb > 2.0 {
            ui.label("超大文件模式：内容已截断，编辑功能受限");
        } else {
            // ui.label("大文件模式：部分功能已优化以确保流畅体验");
        }
    });
}
