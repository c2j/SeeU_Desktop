use eframe::egui;
use std::path::PathBuf;
use crate::db_state::DbINoteState;
use crate::hex_to_color32;
use crate::siyuan_import::SiyuanImporter;

/// Create a highlighted layout job for text editor with search terms
fn create_highlighted_layout_job(
    text: &str,
    search_terms: &[String],
    wrap_width: f32,
    ui: &egui::Ui,
) -> egui::text::LayoutJob {
    use egui::{text::LayoutJob, Color32, FontId, TextFormat};

    if search_terms.is_empty() {
        // No search terms, create normal layout
        let mut layout_job = LayoutJob::default();
        let text_format = TextFormat {
            font_id: FontId::monospace(14.0),
            color: ui.visuals().text_color(),
            ..Default::default()
        };
        layout_job.wrap.max_width = wrap_width;
        layout_job.append(text, 0.0, text_format);
        return layout_job;
    }

    // Convert to character vector for safe indexing
    let chars: Vec<char> = text.chars().collect();
    let text_lower = text.to_lowercase();
    let text_lower_chars: Vec<char> = text_lower.chars().collect();

    let mut highlighted_ranges = Vec::new();

    // Find all matches
    for term in search_terms {
        let term_lower = term.to_lowercase();
        let term_chars: Vec<char> = term_lower.chars().collect();

        if term_chars.is_empty() {
            continue;
        }

        let mut start = 0;
        while start + term_chars.len() <= text_lower_chars.len() {
            // Check if term matches at current position
            let mut matches = true;
            for (i, &term_char) in term_chars.iter().enumerate() {
                if text_lower_chars[start + i] != term_char {
                    matches = false;
                    break;
                }
            }

            if matches {
                // Check if this range overlaps with existing highlights
                let range = (start, start + term_chars.len());
                let overlaps = highlighted_ranges.iter().any(|&(existing_start, existing_end)| {
                    range.0 < existing_end && range.1 > existing_start
                });

                if !overlaps {
                    highlighted_ranges.push(range);
                }

                start += term_chars.len();
            } else {
                start += 1;
            }
        }
    }

    // Sort ranges by start position
    highlighted_ranges.sort_by_key(|&(start, _)| start);

    // Create LayoutJob with highlighting
    let mut layout_job = LayoutJob::default();
    layout_job.wrap.max_width = wrap_width;
    let mut last_end = 0;

    // Default text format
    let normal_format = TextFormat {
        font_id: FontId::monospace(14.0),
        color: ui.visuals().text_color(),
        ..Default::default()
    };

    // Highlighted text format
    let highlight_format = TextFormat {
        font_id: FontId::monospace(14.0),
        color: Color32::BLACK,
        background: Color32::YELLOW,
        ..Default::default()
    };

    for (start, end) in highlighted_ranges {
        // Add normal text before highlight
        if start > last_end {
            let normal_text: String = chars[last_end..start].iter().collect();
            layout_job.append(&normal_text, 0.0, normal_format.clone());
        }

        // Add highlighted text
        let highlighted_text: String = chars[start..end].iter().collect();
        layout_job.append(&highlighted_text, 0.0, highlight_format.clone());

        last_end = end;
    }

    // Add remaining normal text
    if last_end < chars.len() {
        let remaining_text: String = chars[last_end..].iter().collect();
        layout_job.append(&remaining_text, 0.0, normal_format);
    }

    layout_job
}

/// Render the notebook list
// pub fn render_notebook_list(ui: &mut egui::Ui, state: &mut DbINoteState) {
    // ui.horizontal(|ui| {
    //     ui.heading("笔记本");

    //     ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
    //         if ui.button("+ 新建").clicked() {
    //             state.show_create_notebook = true;
    //         }
    //     });
    // });

    // ui.separator();

    // Notebook list
//     egui::ScrollArea::vertical().id_source("db_notebooks_scroll").show(ui, |ui| {
//         let notebooks = state.notebooks.clone();
//         let current_notebook = state.current_notebook;

//         for (i, notebook) in notebooks.iter().enumerate() {
//             let is_selected = current_notebook == Some(i);

//             ui.horizontal(|ui| {
//                 if ui.selectable_label(is_selected, format!("📓 {}", notebook.name)).clicked() {
//                     state.select_notebook(i);
//                 }

//                 if ui.button("🗑").clicked() {
//                     state.delete_notebook(i);
//                 }
//             });
//         }

//         if notebooks.is_empty() {
//             ui.label("没有笔记本");
//         }
//     });
// }

/// Render the tag list
pub fn render_tag_list(ui: &mut egui::Ui, state: &mut DbINoteState) {
    ui.horizontal(|ui| {
        ui.heading("标签");

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button("+ 新建").clicked() {
                state.show_create_tag = true;
            }
        });
    });

    ui.separator();

    // Tag list
    egui::ScrollArea::vertical().id_source("db_tags_scroll").show(ui, |ui| {
        let tags = state.tags.clone();

        for tag in &tags {
            ui.horizontal(|ui| {
                let tag_text = format!("🏷 {}", tag.name);

                // Color rectangle
                let color = hex_to_color32(&tag.color);
                let _rect = ui.painter().add(
                    egui::Shape::rect_filled(
                        egui::Rect::from_min_size(
                            ui.cursor().min + egui::vec2(0.0, 2.0),
                            egui::vec2(10.0, 10.0),
                        ),
                        0.0,
                        color,
                    )
                );

                ui.add_space(15.0);

                // 使标签文本可点击
                let tag_id = tag.id.clone();
                let tag_label = ui.selectable_label(false, tag_text)
                    .on_hover_text("点击查看此标签下的所有笔记");

                if tag_label.clicked() {
                    // 点击标签时，搜索该标签下的所有笔记
                    log::info!("Sidebar tag clicked: {}", tag_id);
                    state.search_notes_by_tag(&tag_id);
                }

                if ui.button("🗑").clicked() {
                    state.delete_tag(&tag_id);
                }
            });
        }

        if tags.is_empty() {
            ui.label("没有标签");
        }
    });
}

/// Render the note list
pub fn render_note_list(ui: &mut egui::Ui, state: &mut DbINoteState) {
    // If we're searching, show search results instead of notebook notes
    if state.is_searching {
        render_search_results(ui, state);
        return;
    }

    if let Some(notebook_idx) = state.current_notebook {
        if notebook_idx < state.notebooks.len() {
            let notebook_name = state.notebooks[notebook_idx].name.clone();

            ui.horizontal(|ui| {
                ui.heading(format!("笔记 - {}", notebook_name));

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("+ 新建笔记").clicked() {
                        let note_id = state.create_note("新笔记".to_string(), "".to_string());

                        if let Some(id) = note_id {
                            state.select_note(&id);
                        }
                    }
                });
            });

            ui.separator();

            // Notes list
            // Clone the data we need to avoid borrowing issues
            let notes = state.get_current_notebook_notes();
            let current_note = state.current_note.clone();

            // Create a list of note IDs and titles for rendering
            let note_data: Vec<(String, String)> = notes;

            let has_notes = !note_data.is_empty();

            egui::ScrollArea::vertical().id_source("db_notes_scroll").show(ui, |ui| {
                for (note_id, title) in note_data {
                    let is_selected = current_note.as_ref().map_or(false, |id| id == &note_id);
                    let note_id_clone = note_id.clone();

                    ui.horizontal(|ui| {
                        let truncated_title = crate::truncate_note_title(&title);
                        if ui.selectable_label(is_selected, format!("📝 {}", truncated_title)).clicked() {
                            state.select_note(&note_id);
                        }

                        if ui.button("🗑").clicked() {
                            state.delete_note(&note_id_clone);
                        }
                    });
                }

                if !has_notes {
                    ui.label("没有笔记");
                }
            });
        }
    } else {
        ui.centered_and_justified(|ui| {
            ui.label("选择或创建一个笔记本");
        });
    }
}

/// Render search results
pub fn render_search_results(ui: &mut egui::Ui, state: &mut DbINoteState) {
    ui.horizontal(|ui| {
        // 检查是否是标签搜索（搜索查询以"标签:"开头）
        let is_tag_search = state.search_query.starts_with("标签:");

        if is_tag_search {
            ui.heading(&state.search_query); // 已经包含"标签: xxx"格式
        } else {
            ui.heading(format!("搜索结果: \"{}\"", state.search_query));
        }

        // 显示结果数量
        let result_count = state.search_results.len();
        ui.label(format!("(找到 {} 条结果)", result_count));


    });

    // 添加一个明显的返回按钮
    if ui.button("◀ 返回笔记列表").clicked() {
        state.is_searching = false;
        state.search_query.clear();
        state.search_results.clear();
    }

    ui.separator();

    // Search results list
    let current_note = state.current_note.clone();

    // Create a list of note IDs, titles, and notebook names for rendering
    let result_data: Vec<(String, String, String)> = state.search_results.iter()
        .filter_map(|note_id| {
            state.notes.get(note_id).map(|note| {
                // Find which notebook this note belongs to
                let notebook_name = state.notebooks.iter()
                    .find(|nb| nb.note_ids.contains(&note.id))
                    .map(|nb| nb.name.clone())
                    .unwrap_or_else(|| "未知笔记本".to_string());

                (note.id.clone(), note.title.clone(), notebook_name)
            })
        })
        .collect();

    let has_results = !result_data.is_empty();

    egui::ScrollArea::vertical().id_source("db_search_results_scroll").show(ui, |ui| {
        for (note_id, title, notebook_name) in result_data {
            let is_selected = current_note.as_ref().map_or(false, |id| id == &note_id);

            ui.horizontal(|ui| {
                let truncated_title = crate::truncate_note_title(&title);
                if ui.selectable_label(is_selected, format!("📝 {}", truncated_title)).clicked() {
                    state.select_note(&note_id);
                }

                ui.label(format!("({})", notebook_name));
            });
        }

        if !has_results {
            ui.label("没有找到匹配的笔记");
        }
    });
}

/// Render the note editor
pub fn render_note_editor(ui: &mut egui::Ui, state: &mut DbINoteState, font_family: Option<&str>) {
    if let Some(note_id) = state.current_note.clone() {
        // 显示加载状态
        if state.is_loading_note {
            ui.horizontal(|ui| {
                ui.spinner();
                ui.label("正在加载大文件，请稍候...");
            });
            ui.separator();
        }
        // 使用垂直布局，让内容区域自动填充剩余空间
        ui.vertical(|ui| {
            // 顶部工具栏 - 固定高度
            ui.horizontal(|ui| {
                // 只有在非全窗口最大化模式下才显示最大化按钮
                // 因为全窗口最大化模式下，返回按钮在外层UI中
                if !state.editor_maximized {
                    // 全窗口最大化按钮
                    if ui.button("🔍 全屏").clicked() {
                        state.toggle_editor_maximized();
                    }
                }

                // 编辑/预览切换按钮 - 添加背景色区分
                let edit_button_color = if state.markdown_preview {
                    // 预览模式下，编辑按钮使用默认色
                    ui.style().visuals.widgets.inactive.bg_fill
                } else {
                    // 编辑模式下，编辑按钮使用高亮色
                    ui.style().visuals.selection.bg_fill
                };

                let preview_button_color = if state.markdown_preview {
                    // 预览模式下，预览按钮使用高亮色
                    ui.style().visuals.selection.bg_fill
                } else {
                    // 编辑模式下，预览按钮使用默认色
                    ui.style().visuals.widgets.inactive.bg_fill
                };

                // 编辑按钮
                let edit_button = egui::Button::new("📝 编辑")
                    .fill(edit_button_color);
                if ui.add(edit_button).clicked() && state.markdown_preview {
                    state.auto_save_if_modified();
                    state.markdown_preview = false;
                }

                // 预览按钮
                let preview_button = egui::Button::new("👁 预览")
                    .fill(preview_button_color);
                if ui.add(preview_button).clicked() && !state.markdown_preview {
                    state.auto_save_if_modified();
                    state.markdown_preview = true;
                }

                // 幻灯片播放按钮
                ui.separator();
                let is_slideshow = state.is_current_note_slideshow();

                ui.horizontal(|ui| {
                    let slide_button = ui.add_enabled(is_slideshow, egui::Button::new("🎬 播放幻灯片"))
                        .on_hover_text(if is_slideshow {
                            "以幻灯片模式播放当前笔记\n\n支持的幻灯片格式：\n• 使用 --- 或 --slide 分隔幻灯片\n• 包含 <style> CSS样式块\n• 包含幻灯片配置标记"
                        } else {
                            "当前笔记不支持幻灯片模式\n\n要启用幻灯片功能，请添加：\n• 幻灯片分隔符：--- 或 --slide\n• CSS样式块：<style>...</style>\n• 配置标记：slide-config: 或 slideshow:"
                        });

                    if slide_button.clicked() && is_slideshow {
                        match state.start_slideshow() {
                            Ok(()) => {
                                log::info!("Started slideshow successfully");
                            }
                            Err(e) => {
                                log::error!("Failed to start slideshow: {}", e);
                            }
                        }
                    }

                    // 样式选择下拉菜单
                    if is_slideshow {
                        egui::ComboBox::from_id_source("slide_style_selector")
                            .selected_text(format!("样式: {}", state.slide_style_manager.get_selected_template().name))
                            .show_ui(ui, |ui| {
                                let all_templates = state.slide_style_manager.get_all_templates();
                                for template in all_templates {
                                    let is_selected = template.id == state.slide_style_manager.selected_template_id;
                                    if ui.selectable_label(is_selected, &template.name).clicked() {
                                        state.slide_style_manager.set_selected_template(template.id.clone());
                                        // 保存样式配置
                                        if let Err(e) = state.save_slide_style_config() {
                                            log::error!("Failed to save slide style config: {}", e);
                                        }
                                        log::info!("Selected slide template: {}", template.name);
                                    }
                                }
                            });
                    }
                });

                // 富文本粘贴按钮（仅在编辑模式下显示）
                if !state.markdown_preview {
                    ui.separator();

                    // 检查剪贴板是否有富文本内容
                    let has_rich_content = state.clipboard_has_rich_content();

                    let paste_button = ui.button("📋 粘贴富文本")
                        .on_hover_text("从剪贴板粘贴富文本内容并自动转换为Markdown格式\n支持：标题、段落、列表、表格、图片等");

                    if paste_button.clicked() {
                        match state.paste_rich_text() {
                            Ok(markdown_text) => {
                                state.append_to_note_content(&markdown_text);
                                log::info!("Rich text pasted successfully");
                            }
                            Err(e) => {
                                log::error!("Failed to paste rich text: {}", e);
                            }
                        }
                    }

                    // 如果剪贴板有富文本内容，显示提示
                    if has_rich_content {
                        ui.label("💡 检测到富文本内容");
                    }
                }

                // Markdown 帮助按钮
                ui.separator();
                let help_button = ui.button("❓")
                    .on_hover_text("显示 Markdown 格式指引");

                if help_button.clicked() {
                    state.show_markdown_help = true;
                }
            });

            ui.separator();

            // 标题区域 - 固定高度
            ui.horizontal(|ui| {
                ui.label("标题:");

                // 使用与编辑器相同的等宽字体配置
                let mut title_edit = egui::TextEdit::singleline(&mut state.note_title)
                    .desired_width(ui.available_width() - 10.0)
                    .font(egui::FontId::monospace(16.0))  // 稍大的字体，更像标题
                    .hint_text("输入笔记标题...");

                // 使用简单的布局器处理文本
                let mut title_layouter = move |ui: &egui::Ui, text: &str, wrap_width: f32| {
                    // 创建一个简单的布局作业
                    let mut layout_job = egui::text::LayoutJob::default();

                    // 设置基本字体格式 - 使用粗体
                    let text_format = egui::TextFormat {
                        font_id: egui::FontId::monospace(16.0),
                        color: ui.visuals().text_color(),
                        ..Default::default()
                    };

                    // 设置换行属性
                    layout_job.wrap.max_width = wrap_width;

                    // 直接添加整个文本，让字体系统处理字符宽度
                    layout_job.append(text, 0.0, text_format);

                    ui.fonts(|f| f.layout_job(layout_job))
                };

                title_edit = title_edit.layouter(&mut title_layouter);
                let title_response = ui.add(title_edit);

                // Check for title changes and mark as modified
                if title_response.changed() {
                    state.check_note_modified();

                    // Immediate auto-save on title change
                    if state.save_status == crate::db_state::SaveStatus::Modified {
                        state.auto_save_if_modified();
                    }
                }
            });

            // 内容区域 - 自动填充剩余空间
            ui.allocate_ui_with_layout(
                egui::Vec2::new(ui.available_width(), ui.available_height()),
                egui::Layout::top_down(egui::Align::LEFT),
                |ui| {
                    // Note content - either editor or preview
                    if state.markdown_preview {
                        // Markdown preview with search highlighting
                        let search_terms = state.get_search_terms();
                        let available_height = ui.available_height();
                        egui::ScrollArea::vertical()
                            .id_source("markdown_preview_scroll")
                            .auto_shrink([false, false])  // 不自动收缩，占满可用空间
                            .max_height(available_height)  // 设置最大高度为可用高度
                            .min_scrolled_height(available_height)  // 设置最小高度，确保填充可用空间
                            .show(ui, |ui| {
                                ui.add_space(10.0);
                                crate::markdown::render_markdown_with_highlight(ui, &state.note_content, &search_terms, font_family);
                                ui.add_space(10.0);
                            });
                    } else {
                        // Editor mode - 使用 ScrollArea 包装编辑器
                        let available_height = ui.available_height();
                        egui::ScrollArea::vertical()
                            .id_source("editor_scroll")
                            .auto_shrink([false, false])  // 不自动收缩，占满可用空间
                            .max_height(available_height)  // 设置最大高度为可用高度
                            .min_scrolled_height(available_height)  // 设置最小高度，确保填充可用空间
                            .show(ui, |ui| {
                    // 获取搜索关键字（在创建 layouter 之前）
                    let search_terms = state.get_search_terms();

                    // 设置等宽字体，确保中英文宽度比例为1:2
                    let mut text_edit = egui::TextEdit::multiline(&mut state.note_content)
                        .desired_width(f32::INFINITY)
                        .font(egui::FontId::monospace(14.0))  // 使用等宽字体
                        .hint_text("输入笔记内容...\n支持 Markdown 格式");

                    // 设置编辑器样式 - 使用代码编辑器样式，确保等宽字体
                    text_edit = text_edit.code_editor();

                    // 自定义编辑器样式
                    let style = ui.style_mut();
                    style.text_styles.get_mut(&egui::TextStyle::Monospace).map(|font_id| {
                        font_id.size = 14.0; // 设置等宽字体大小
                    });

                    // 自定义字体渲染 - 支持搜索高亮
                    let mut layouter = move |ui: &egui::Ui, text: &str, wrap_width: f32| {
                        // 如果有搜索关键字，创建高亮布局
                        if !search_terms.is_empty() {
                            let layout_job = create_highlighted_layout_job(text, &search_terms, wrap_width, ui);
                            ui.fonts(|f| f.layout_job(layout_job))
                        } else {
                            // 创建一个简单的布局作业
                            let mut layout_job = egui::text::LayoutJob::default();

                            // 设置基本字体格式
                            let text_format = egui::TextFormat {
                                font_id: egui::FontId::monospace(14.0),
                                color: ui.visuals().text_color(),
                                ..Default::default()
                            };

                            // 设置换行属性
                            layout_job.wrap.max_width = wrap_width;

                            // 直接添加整个文本，让字体系统处理字符宽度
                            layout_job.append(text, 0.0, text_format);

                            ui.fonts(|f| f.layout_job(layout_job))
                        }
                    };

                    text_edit = text_edit.layouter(&mut layouter);

                    // 使用allocate_ui强制TextEdit占用全部可用高度
                    let response = ui.allocate_ui_with_layout(
                        egui::Vec2::new(ui.available_width(), available_height),
                        egui::Layout::top_down(egui::Align::LEFT),
                        |ui| {
                            ui.add_sized([ui.available_width(), ui.available_height()], text_edit)
                        }
                    ).inner;

                    // Check for keyboard shortcuts - 改进IME支持
                    ui.input(|i| {
                        // 检查是否有IME组合状态
                        let has_ime_composition = i.events.iter().any(|event| {
                            matches!(event, egui::Event::Ime(egui::ImeEvent::Preedit(_)))
                        });

                        // 只有在没有IME组合状态时才处理快捷键
                        if !has_ime_composition {
                            // Check for Ctrl+V (Cmd+V on Mac) for rich text paste
                            if i.modifiers.command && i.key_pressed(egui::Key::V) {
                                // Prevent default paste behavior and handle rich text paste
                                match state.paste_rich_text() {
                                    Ok(markdown_text) => {
                                        state.append_to_note_content(&markdown_text);
                                        log::info!("Rich text pasted via keyboard shortcut");
                                    }
                                    Err(e) => {
                                        log::error!("Failed to paste rich text via keyboard shortcut: {}", e);
                                    }
                                }
                            }
                        }
                    });

                    // Check for content changes and mark as modified
                    if response.changed() {
                        state.check_note_modified();

                        // Immediate auto-save on content change
                        // This ensures data is saved as soon as user types
                        if state.save_status == crate::db_state::SaveStatus::Modified {
                            state.auto_save_if_modified();
                        }
                    }

                        // Also auto-save when focus is lost
                        if response.lost_focus() && state.save_status == crate::db_state::SaveStatus::Modified {
                            state.auto_save_if_modified();
                        }
                    });
                }
            }
        );

        // 底部区域 - 标签和帮助信息
        ui.separator();

        // Tags for this note
        ui.horizontal(|ui| {
            ui.label("标签:");

            // Clone the data we need to avoid borrowing issues
            let note_tags = state.get_note_tags(&note_id);

            // Create a list of tag data for rendering
            let tag_data: Vec<(String, String, String)> = note_tags.iter()
                .map(|tag| (tag.id.clone(), tag.name.clone(), tag.color.clone()))
                .collect();

            let note_tag_ids: Vec<String> = tag_data.iter().map(|(id, _, _)| id.clone()).collect();

            for (tag_id, tag_name, tag_color) in &tag_data {
                let color = hex_to_color32(tag_color);

                let tag_button = egui::Button::new(format!("🏷 {}", tag_name))
                    .fill(color);

                let note_id_clone = note_id.clone();
                let tag_id_clone = tag_id.clone();

                // 添加悬停提示，说明点击可以查看该标签下的所有笔记
                let response = ui.add(tag_button)
                    .on_hover_text("点击查看此标签下的所有笔记\n右键点击可从当前笔记中移除此标签");

                if response.clicked() {
                    // 左键点击：搜索该标签下的所有笔记
                    log::info!("Note editor tag clicked: {}", tag_id_clone);
                    state.search_notes_by_tag(&tag_id_clone);
                } else if response.secondary_clicked() {
                    // 右键点击：从当前笔记中移除标签
                    state.remove_tag_from_note(&note_id_clone, &tag_id_clone);
                }
            }

            // Add tag dropdown
            egui::ComboBox::from_id_source("add_tag_dropdown")
                .selected_text("+ 添加标签")
                .show_ui(ui, |ui| {
                    // Clone all tags to avoid borrowing issues
                    let all_tags = state.tags.clone();
                    let note_id_clone = note_id.clone();

                    for tag in all_tags {
                        if !note_tag_ids.contains(&tag.id) {
                            let tag_id = tag.id.clone();
                            if ui.selectable_label(false, format!("🏷 {}", tag.name)).clicked() {
                                state.add_tag_to_note(&note_id_clone, &tag_id);
                            }
                        }
                    }
                });
        });

        // Markdown help
        ui.horizontal(|ui| {
            // 创建一个带有超链接样式的文本
            let markdown_help_text = if state.markdown_preview {
                "提示: 支持 Markdown 格式，包括标题、列表、链接、代码块等"
            } else {
                "提示: 支持 Markdown 格式，点击「👁 预览」查看渲染效果"
            };

            // 添加普通文本部分
            ui.label(markdown_help_text);

            // 添加超链接部分
            let link_text = "查看格式指引";
            let link_response = ui.hyperlink_to(link_text, "#");

            // 当超链接被点击时，显示Markdown帮助弹窗
            if link_response.clicked() {
                state.show_markdown_help = true;
            }

            // 移除了编辑器模式切换按钮
        });

        // 显示Markdown帮助弹窗
        if state.show_markdown_help {
            // 创建一个标志，用于在窗口关闭时设置状态
            let mut show_window = state.show_markdown_help;

            egui::Window::new("Markdown 格式指引")
                .collapsible(false)
                .resizable(true)
                .default_size([500.0, 400.0])
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .open(&mut show_window)
                .show(ui.ctx(), |ui| {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        ui.heading("Markdown 格式指引");
                        ui.separator();

                        // 富文本粘贴说明
                        ui.add_space(10.0);
                        ui.strong("💡 富文本粘贴功能");
                        ui.label("支持从其他应用程序复制富文本内容并自动转换为Markdown格式：");
                        ui.label("• 从网页、Word文档、邮件等复制内容");
                        ui.label("• 自动保留标题、段落、列表、表格、图片等格式");
                        ui.label("• 使用 Ctrl+V (Mac: Cmd+V) 或点击「📋 粘贴富文本」按钮");
                        ui.label("• 支持HTML内容自动转换为标准Markdown语法");
                        ui.separator();

                        // 使用三列布局：左侧是Markdown语法，中间是渲染效果，右侧是复制按钮
                        ui.add_space(10.0);
                        ui.columns(3, |columns| {
                            // 左列：语法
                            columns[0].strong("语法");
                            // 中列：效果
                            columns[1].strong("渲染效果");
                            // 右列：操作
                            columns[2].strong("操作");
                        });
                        ui.separator();

                        // 标题
                        ui.add_space(10.0);
                        ui.strong("标题");
                        ui.columns(3, |columns| {
                            // 示例1
                            let heading1_code = "# 一级标题";
                            columns[0].label(heading1_code);
                            columns[1].heading("一级标题");
                            if columns[2].button("插入").clicked() {
                                // 直接插入内容到笔记
                                state.append_to_note_content(heading1_code);
                                state.show_markdown_help = false;
                                log::info!("已插入内容到笔记: {}", heading1_code);
                            }

                            // 示例2
                            let heading2_code = "## 二级标题";
                            columns[0].label(heading2_code);
                            columns[1].heading("二级标题");
                            if columns[2].button("插入").clicked() {
                                state.append_to_note_content(heading2_code);
                                state.show_markdown_help = false;
                                log::info!("已插入内容到笔记: {}", heading2_code);
                            }

                            // 示例3
                            let heading3_code = "### 三级标题";
                            columns[0].label(heading3_code);
                            columns[1].strong("三级标题");
                            if columns[2].button("插入").clicked() {
                                state.append_to_note_content(heading3_code);
                                state.show_markdown_help = false;
                                log::info!("已插入内容到笔记: {}", heading3_code);
                            }
                        });

                        // 文本格式
                        ui.add_space(10.0);
                        ui.strong("文本格式");
                        ui.columns(3, |columns| {
                            // 粗体
                            let bold_code = "**粗体文本**";
                            columns[0].label(bold_code);
                            columns[1].strong("粗体文本");
                            if columns[2].button("插入").clicked() {
                                state.append_to_note_content(bold_code);
                                state.show_markdown_help = false;
                                log::info!("已插入内容到笔记: {}", bold_code);
                            }

                            // 斜体
                            let italic_code = "*斜体文本*";
                            columns[0].label(italic_code);
                            columns[1].label(egui::RichText::new("斜体文本").italics());
                            if columns[2].button("插入").clicked() {
                                state.append_to_note_content(italic_code);
                                state.show_markdown_help = false;
                                log::info!("已插入内容到笔记: {}", italic_code);
                            }

                            // 代码
                            let code_code = "`代码`";
                            columns[0].label(code_code);
                            columns[1].monospace("代码");
                            if columns[2].button("插入").clicked() {
                                state.append_to_note_content(code_code);
                                state.show_markdown_help = false;
                                log::info!("已插入内容到笔记: {}", code_code);
                            }

                            // 删除线
                            let strike_code = "~~删除线~~";
                            columns[0].label(strike_code);
                            columns[1].label(egui::RichText::new("删除线").strikethrough());
                            if columns[2].button("插入").clicked() {
                                state.append_to_note_content(strike_code);
                                state.show_markdown_help = false;
                                log::info!("已插入内容到笔记: {}", strike_code);
                            }
                        });

                        // 列表
                        ui.add_space(10.0);
                        ui.strong("列表");
                        ui.columns(3, |columns| {
                            // 无序列表示例
                            let list_code = "- 无序列表项\n- 另一个无序列表项\n  - 嵌套列表项";

                            // 语法
                            columns[0].label("- 无序列表项");
                            columns[0].label("- 另一个无序列表项");
                            columns[0].label("  - 嵌套列表项");

                            // 效果
                            columns[1].vertical(|ui| {
                                ui.label("• 无序列表项");
                                ui.label("• 另一个无序列表项");
                                ui.add_space(5.0);
                                ui.label("    • 嵌套列表项");
                            });

                            // 复制按钮
                            if columns[2].button("插入").clicked() {
                                state.append_to_note_content(list_code);
                                state.show_markdown_help = false;
                                log::info!("已插入内容到笔记: {}", list_code);
                            }

                            // 有序列表示例
                            let ordered_list_code = "1. 有序列表项\n2. 另一个有序列表项";

                            // 语法
                            columns[0].label("1. 有序列表项");
                            columns[0].label("2. 另一个有序列表项");

                            // 效果
                            columns[1].vertical(|ui| {
                                ui.label("1. 有序列表项");
                                ui.label("2. 另一个有序列表项");
                            });

                            // 复制按钮
                            if columns[2].button("插入").clicked() {
                                state.append_to_note_content(ordered_list_code);
                                state.show_markdown_help = false;
                                log::info!("已插入内容到笔记: {}", ordered_list_code);
                            }
                        });

                        // 链接和图片
                        ui.add_space(10.0);
                        ui.strong("链接和图片");
                        ui.columns(3, |columns| {
                            // 链接示例
                            let link_code = "[链接文本](https://example.com)";
                            columns[0].label(link_code);
                            columns[1].hyperlink_to("链接文本", "https://example.com");
                            if columns[2].button("插入").clicked() {
                                state.append_to_note_content(link_code);
                                state.show_markdown_help = false;
                                log::info!("已插入内容到笔记: {}", link_code);
                            }

                            // 图片示例
                            let image_code = "![图片描述](图片URL)";
                            columns[0].label(image_code);
                            columns[1].label("图片将在此显示");
                            if columns[2].button("插入").clicked() {
                                state.append_to_note_content(image_code);
                                state.show_markdown_help = false;
                                log::info!("已插入内容到笔记: {}", image_code);
                            }
                        });

                        // 引用
                        ui.add_space(10.0);
                        ui.strong("引用");
                        ui.columns(3, |columns| {
                            // 引用示例
                            let quote_code = "> 这是一段引用文本\n> 这是引用的第二行";

                            // 语法
                            columns[0].label("> 这是一段引用文本");
                            columns[0].label("> 这是引用的第二行");

                            // 效果
                            columns[1].group(|ui| {
                                ui.label(egui::RichText::new("这是一段引用文本").italics());
                                ui.label(egui::RichText::new("这是引用的第二行").italics());
                            });

                            // 复制按钮
                            if columns[2].button("插入").clicked() {
                                state.append_to_note_content(quote_code);
                                state.show_markdown_help = false;
                                log::info!("已插入内容到笔记: {}", quote_code);
                            }
                        });

                        // 代码块
                        ui.add_space(10.0);
                        ui.strong("代码块");
                        ui.columns(3, |columns| {
                            // 代码块示例
                            let code_block = "```\nfunction example() {\n  return 'Hello, world!';\n}\n```";

                            // 语法
                            columns[0].vertical(|ui| {
                                ui.label("```");
                                ui.label("function example() {");
                                ui.label("  return 'Hello, world!';");
                                ui.label("}");
                                ui.label("```");
                            });

                            // 效果
                            columns[1].code("function example() {\n  return 'Hello, world!';\n}");

                            // 复制按钮
                            if columns[2].button("插入").clicked() {
                                state.append_to_note_content(code_block);
                                state.show_markdown_help = false;
                                log::info!("已插入内容到笔记: {}", code_block);
                            }
                        });

                        // 表格
                        ui.add_space(10.0);
                        ui.strong("表格");
                        ui.columns(3, |columns| {
                            // 表格示例
                            let table_code = "| 表头1 | 表头2 |\n| ----- | ----- |\n| 单元格1 | 单元格2 |\n| 单元格3 | 单元格4 |";

                            // 语法
                            columns[0].vertical(|ui| {
                                ui.label("| 表头1 | 表头2 |");
                                ui.label("| ----- | ----- |");
                                ui.label("| 单元格1 | 单元格2 |");
                                ui.label("| 单元格3 | 单元格4 |");
                            });

                            // 效果
                            columns[1].group(|ui| {
                                egui::Grid::new("markdown_table_example")
                                    .striped(true)
                                    .show(ui, |ui| {
                                        ui.strong("表头1"); ui.strong("表头2"); ui.end_row();
                                        ui.label("单元格1"); ui.label("单元格2"); ui.end_row();
                                        ui.label("单元格3"); ui.label("单元格4"); ui.end_row();
                                    });
                            });

                            // 复制按钮
                            if columns[2].button("插入").clicked() {
                                state.append_to_note_content(table_code);
                                state.show_markdown_help = false;
                                log::info!("已插入内容到笔记: {}", table_code);
                            }
                        });

                        // 任务列表
                        ui.add_space(10.0);
                        ui.strong("任务列表");
                        ui.columns(3, |columns| {
                            // 任务列表示例
                            let task_code = "- [ ] 未完成任务\n- [x] 已完成任务";

                            // 语法
                            columns[0].vertical(|ui| {
                                ui.label("- [ ] 未完成任务");
                                ui.label("- [x] 已完成任务");
                            });

                            // 效果
                            columns[1].vertical(|ui| {
                                ui.checkbox(&mut false, "未完成任务");
                                ui.checkbox(&mut true, "已完成任务");
                            });

                            // 复制按钮
                            if columns[2].button("插入").clicked() {
                                state.append_to_note_content(task_code);
                                state.show_markdown_help = false;
                                log::info!("已插入内容到笔记: {}", task_code);
                            }
                        });

                        // 水平线
                        ui.add_space(10.0);
                        ui.strong("水平线");
                        ui.columns(3, |columns| {
                            // 水平线示例
                            let hr_code = "---";

                            // 语法
                            columns[0].label(hr_code);

                            // 效果
                            columns[1].separator();

                            // 复制按钮
                            if columns[2].button("插入").clicked() {
                                state.append_to_note_content(hr_code);
                                state.show_markdown_help = false;
                                log::info!("已插入内容到笔记: {}", hr_code);
                            }
                        });

                        // Mermaid 图形
                        ui.add_space(10.0);
                        ui.strong("Mermaid 图形");
                        ui.label("支持多种图形类型，使用 ```mermaid 代码块包围：");

                        // 流程图
                        ui.add_space(5.0);
                        ui.label(egui::RichText::new("• 流程图").strong());
                        ui.columns(3, |columns| {
                            let flowchart_code = "```mermaid\ngraph TD\n    A[开始] --> B{判断}\n    B -->|是| C[执行]\n    B -->|否| D[结束]\n```";

                            columns[0].vertical(|ui| {
                                ui.label("```mermaid");
                                ui.label("graph TD");
                                ui.label("    A[开始] --> B{判断}");
                                ui.label("    B -->|是| C[执行]");
                                ui.label("    B -->|否| D[结束]");
                                ui.label("```");
                            });

                            columns[1].label("🔄 流程图预览");

                            if columns[2].button("插入").clicked() {
                                state.append_to_note_content(flowchart_code);
                                state.show_markdown_help = false;
                                log::info!("已插入内容到笔记: {}", flowchart_code);
                            }
                        });

                        // 时序图
                        ui.add_space(5.0);
                        ui.label(egui::RichText::new("• 时序图").strong());
                        ui.columns(3, |columns| {
                            let sequence_code = "```mermaid\nsequenceDiagram\n    participant A as 用户\n    participant B as 系统\n    A->>B: 请求\n    B-->>A: 响应\n```";

                            columns[0].vertical(|ui| {
                                ui.label("```mermaid");
                                ui.label("sequenceDiagram");
                                ui.label("    participant A as 用户");
                                ui.label("    participant B as 系统");
                                ui.label("    A->>B: 请求");
                                ui.label("    B-->>A: 响应");
                                ui.label("```");
                            });

                            columns[1].label("📋 时序图预览");

                            if columns[2].button("插入").clicked() {
                                state.append_to_note_content(sequence_code);
                                state.show_markdown_help = false;
                                log::info!("已插入内容到笔记: {}", sequence_code);
                            }
                        });

                        // 类图
                        ui.add_space(5.0);
                        ui.label(egui::RichText::new("• 类图").strong());
                        ui.columns(3, |columns| {
                            let class_code = "```mermaid\nclassDiagram\n    class User {\n        +String name\n        +login()\n    }\n    User --> Role\n```";

                            columns[0].vertical(|ui| {
                                ui.label("```mermaid");
                                ui.label("classDiagram");
                                ui.label("    class User {");
                                ui.label("        +String name");
                                ui.label("        +login()");
                                ui.label("    }");
                                ui.label("    User --> Role");
                                ui.label("```");
                            });

                            columns[1].label("🏗️ 类图预览");

                            if columns[2].button("插入").clicked() {
                                state.append_to_note_content(class_code);
                                state.show_markdown_help = false;
                                log::info!("已插入内容到笔记: {}", class_code);
                            }
                        });

                        // 状态图
                        ui.add_space(5.0);
                        ui.label(egui::RichText::new("• 状态图").strong());
                        ui.columns(3, |columns| {
                            let state_code = "```mermaid\nstateDiagram-v2\n    [*] --> 待处理\n    待处理 --> 处理中\n    处理中 --> 完成\n    完成 --> [*]\n```";

                            columns[0].vertical(|ui| {
                                ui.label("```mermaid");
                                ui.label("stateDiagram-v2");
                                ui.label("    [*] --> 待处理");
                                ui.label("    待处理 --> 处理中");
                                ui.label("    处理中 --> 完成");
                                ui.label("    完成 --> [*]");
                                ui.label("```");
                            });

                            columns[1].label("🔄 状态图预览");

                            if columns[2].button("插入").clicked() {
                                state.append_to_note_content(state_code);
                                state.show_markdown_help = false;
                                log::info!("已插入内容到笔记: {}", state_code);
                            }
                        });

                        // 甘特图
                        ui.add_space(5.0);
                        ui.label(egui::RichText::new("• 甘特图").strong());
                        ui.columns(3, |columns| {
                            let gantt_code = "```mermaid\ngantt\n    title 项目计划\n    section 开发\n    任务1 :2024-01-01, 3d\n    任务2 :2024-01-04, 2d\n```";

                            columns[0].vertical(|ui| {
                                ui.label("```mermaid");
                                ui.label("gantt");
                                ui.label("    title 项目计划");
                                ui.label("    section 开发");
                                ui.label("    任务1 :2024-01-01, 3d");
                                ui.label("    任务2 :2024-01-04, 2d");
                                ui.label("```");
                            });

                            columns[1].label("📅 甘特图预览");

                            if columns[2].button("插入").clicked() {
                                state.append_to_note_content(gantt_code);
                                state.show_markdown_help = false;
                                log::info!("已插入内容到笔记: {}", gantt_code);
                            }
                        });

                        // 饼图
                        ui.add_space(5.0);
                        ui.label(egui::RichText::new("• 饼图").strong());
                        ui.columns(3, |columns| {
                            let pie_code = "```mermaid\npie title 数据分布\n    \"类型A\" : 45\n    \"类型B\" : 30\n    \"类型C\" : 25\n```";

                            columns[0].vertical(|ui| {
                                ui.label("```mermaid");
                                ui.label("pie title 数据分布");
                                ui.label("    \"类型A\" : 45");
                                ui.label("    \"类型B\" : 30");
                                ui.label("    \"类型C\" : 25");
                                ui.label("```");
                            });

                            columns[1].label("🥧 饼图预览");

                            if columns[2].button("插入").clicked() {
                                state.append_to_note_content(pie_code);
                                state.show_markdown_help = false;
                                log::info!("已插入内容到笔记: {}", pie_code);
                            }
                        });

                        // 幻灯片格式扩展
                        ui.add_space(10.0);
                        ui.strong("幻灯片格式扩展");
                        ui.label("将笔记转换为幻灯片演示，支持多种分隔符和样式配置：");

                        // 基本幻灯片分隔
                        ui.add_space(5.0);
                        ui.label(egui::RichText::new("• 幻灯片分隔符").strong());
                        ui.columns(3, |columns| {
                            let slide_separator_code = "# 第一张幻灯片\n内容...\n\n---\n\n# 第二张幻灯片\n内容...\n\n--slide\n\n# 第三张幻灯片\n内容...";

                            columns[0].vertical(|ui| {
                                ui.label("# 第一张幻灯片");
                                ui.label("内容...");
                                ui.label("");
                                ui.label("---");
                                ui.label("");
                                ui.label("# 第二张幻灯片");
                                ui.label("内容...");
                                ui.label("");
                                ui.label("--slide");
                                ui.label("");
                                ui.label("# 第三张幻灯片");
                                ui.label("内容...");
                            });

                            columns[1].label("🎬 幻灯片分隔");

                            if columns[2].button("插入").clicked() {
                                state.append_to_note_content(slide_separator_code);
                                state.show_markdown_help = false;
                                log::info!("已插入内容到笔记: {}", slide_separator_code);
                            }
                        });

                        // 幻灯片样式配置
                        ui.add_space(5.0);
                        ui.label(egui::RichText::new("• 样式配置").strong());
                        ui.columns(3, |columns| {
                            let slide_config_code = "<!-- config: background=#1a1a1a text=#ffffff -->\n\n# 深色主题幻灯片\n\n这是一个使用深色背景的幻灯片";

                            columns[0].vertical(|ui| {
                                ui.label("<!-- config: background=#1a1a1a");
                                ui.label("     text=#ffffff -->");
                                ui.label("");
                                ui.label("# 深色主题幻灯片");
                                ui.label("");
                                ui.label("这是一个使用深色背景的幻灯片");
                            });

                            columns[1].label("🎨 样式配置");

                            if columns[2].button("插入").clicked() {
                                state.append_to_note_content(slide_config_code);
                                state.show_markdown_help = false;
                                log::info!("已插入内容到笔记: {}", slide_config_code);
                            }
                        });

                        // CSS 样式块
                        ui.add_space(5.0);
                        ui.label(egui::RichText::new("• CSS 样式块").strong());
                        ui.columns(3, |columns| {
                            let slide_css_code = "```css\n.slide {\n    background: linear-gradient(45deg, #667eea, #764ba2);\n    color: white;\n    font-size: 18px;\n}\n```\n\n# 渐变背景幻灯片\n\n使用自定义CSS样式";

                            columns[0].vertical(|ui| {
                                ui.label("```css");
                                ui.label(".slide {");
                                ui.label("    background: linear-gradient(");
                                ui.label("        45deg, #667eea, #764ba2);");
                                ui.label("    color: white;");
                                ui.label("    font-size: 18px;");
                                ui.label("}");
                                ui.label("```");
                                ui.label("");
                                ui.label("# 渐变背景幻灯片");
                                ui.label("使用自定义CSS样式");
                            });

                            columns[1].label("🎨 CSS样式");

                            if columns[2].button("插入").clicked() {
                                state.append_to_note_content(slide_css_code);
                                state.show_markdown_help = false;
                                log::info!("已插入内容到笔记: {}", slide_css_code);
                            }
                        });

                        // 完整幻灯片示例
                        ui.add_space(5.0);
                        ui.label(egui::RichText::new("• 完整示例").strong());
                        ui.columns(3, |columns| {
                            let complete_slide_code = "```css\n.slide {\n    background: #f8f9fa;\n    padding: 40px;\n}\n```\n\n# 演示标题\n\n欢迎来到我的演示\n\n---\n\n<!-- config: background=#2c3e50 text=#ecf0f1 -->\n\n# 深色主题页面\n\n- 要点一\n- 要点二\n- 要点三\n\n--slide\n\n# 总结\n\n谢谢观看！";

                            columns[0].vertical(|ui| {
                                ui.label("```css");
                                ui.label(".slide { background: #f8f9fa; }");
                                ui.label("```");
                                ui.label("");
                                ui.label("# 演示标题");
                                ui.label("欢迎来到我的演示");
                                ui.label("");
                                ui.label("---");
                                ui.label("");
                                ui.label("<!-- config: background=#2c3e50");
                                ui.label("     text=#ecf0f1 -->");
                                ui.label("# 深色主题页面");
                                ui.label("- 要点一");
                                ui.label("...");
                            });

                            columns[1].label("🎬 完整演示");

                            if columns[2].button("插入").clicked() {
                                state.append_to_note_content(complete_slide_code);
                                state.show_markdown_help = false;
                                log::info!("已插入内容到笔记: {}", complete_slide_code);
                            }
                        });

                        // 幻灯片功能说明
                        ui.add_space(10.0);
                        ui.label(egui::RichText::new("幻灯片功能特性：").strong());
                        ui.label("• 支持 --- 和 --slide 两种分隔符");
                        ui.label("• 内置多种样式模板（默认、深色、蓝色等）");
                        ui.label("• 支持自定义CSS样式");
                        ui.label("• 支持配置注释设置背景色、文字色等");
                        ui.label("• 全屏播放模式和窗口播放模式");
                        ui.label("• 键盘导航（方向键、空格键、ESC键）");
                        ui.label("• 自动检测幻灯片格式并显示播放按钮");
                    });
                });

            // 处理窗口关闭
            if !show_window {
                state.show_markdown_help = false;
            }
        }
        });  // 关闭垂直布局
    } else {
        ui.centered_and_justified(|ui| {
            ui.label("选择或创建一个笔记");
        });
    }

    // Periodically check for auto-save
    // This ensures content is saved even if the user doesn't trigger any events
    if ui.input(|i| i.time) % 3.0 < 0.1 { // Check roughly every 3 seconds
        if state.current_note.is_some() && state.save_status == crate::db_state::SaveStatus::Modified {
            state.auto_save_if_modified();
        }
    }
}

/// 检查字符是否为 CJK 字符（中文、日文、韩文）或全角字符
/// 这些字符通常需要双倍宽度显示
fn is_cjk_char(c: char) -> bool {
    // CJK 统一表意文字范围
    (c >= '\u{4E00}' && c <= '\u{9FFF}') ||  // CJK 统一表意文字
    (c >= '\u{3400}' && c <= '\u{4DBF}') ||  // CJK 统一表意文字扩展 A
    (c >= '\u{20000}' && c <= '\u{2A6DF}') || // CJK 统一表意文字扩展 B
    (c >= '\u{2A700}' && c <= '\u{2B73F}') || // CJK 统一表意文字扩展 C
    (c >= '\u{2B740}' && c <= '\u{2B81F}') || // CJK 统一表意文字扩展 D
    (c >= '\u{2B820}' && c <= '\u{2CEAF}') || // CJK 统一表意文字扩展 E
    (c >= '\u{F900}' && c <= '\u{FAFF}') ||  // CJK 兼容表意文字
    (c >= '\u{2F800}' && c <= '\u{2FA1F}') || // CJK 兼容表意文字补充

    // 日文和韩文字符
    (c >= '\u{3040}' && c <= '\u{309F}') ||  // 平假名
    (c >= '\u{30A0}' && c <= '\u{30FF}') ||  // 片假名
    (c >= '\u{AC00}' && c <= '\u{D7AF}') ||  // 韩文音节
    (c >= '\u{1100}' && c <= '\u{11FF}') ||  // 韩文字母

    // CJK 符号和标点
    (c >= '\u{3000}' && c <= '\u{303F}') ||  // CJK 符号和标点
    (c >= '\u{3300}' && c <= '\u{33FF}') ||  // CJK 兼容字符
    (c >= '\u{FE30}' && c <= '\u{FE4F}') ||  // CJK 兼容形式

    // 全角字符
    (c >= '\u{FF00}' && c <= '\u{FFEF}') ||  // 全角 ASCII、全角标点

    // 其他需要双倍宽度的字符
    (c == '\u{2329}') || (c == '\u{232A}') || // 尖括号
    (c >= '\u{2E80}' && c <= '\u{2EFF}') ||  // CJK 部首补充
    (c >= '\u{2F00}' && c <= '\u{2FDF}') ||  // 康熙部首
    (c >= '\u{2FF0}' && c <= '\u{2FFF}') ||  // 表意文字描述符
    (c >= '\u{3200}' && c <= '\u{32FF}') ||  // 带圈 CJK 字母和月份
    (c >= '\u{FE10}' && c <= '\u{FE1F}')     // 竖排形式
}

