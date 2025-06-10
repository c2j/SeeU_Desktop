use eframe::egui;
use std::path::PathBuf;
use crate::db_state::DbINoteState;
use crate::hex_to_color32;
use crate::siyuan_import::SiyuanImporter;

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
    egui::ScrollArea::vertical().id_salt("db_tags_scroll").show(ui, |ui| {
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
        log::info!("Rendering search results. Query: '{}', Results: {}",
            state.search_query, state.search_results.len());
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
            let note_data: Vec<(String, String)> = notes.iter()
                .map(|note| (note.id.clone(), note.title.clone()))
                .collect();

            let has_notes = !note_data.is_empty();

            egui::ScrollArea::vertical().id_salt("db_notes_scroll").show(ui, |ui| {
                for (note_id, title) in note_data {
                    let is_selected = current_note.as_ref().map_or(false, |id| id == &note_id);
                    let note_id_clone = note_id.clone();

                    ui.horizontal(|ui| {
                        if ui.selectable_label(is_selected, format!("📝 {}", title)).clicked() {
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
    let search_results = state.get_search_result_notes();
    let current_note = state.current_note.clone();

    // Create a list of note IDs, titles, and notebook names for rendering
    let result_data: Vec<(String, String, String)> = search_results.iter()
        .map(|note| {
            // Find which notebook this note belongs to
            let notebook_name = state.notebooks.iter()
                .find(|nb| nb.note_ids.contains(&note.id))
                .map(|nb| nb.name.clone())
                .unwrap_or_else(|| "未知笔记本".to_string());

            (note.id.clone(), note.title.clone(), notebook_name)
        })
        .collect();

    let has_results = !result_data.is_empty();

    egui::ScrollArea::vertical().id_salt("db_search_results_scroll").show(ui, |ui| {
        for (note_id, title, notebook_name) in result_data {
            let is_selected = current_note.as_ref().map_or(false, |id| id == &note_id);

            ui.horizontal(|ui| {
                if ui.selectable_label(is_selected, format!("📝 {}", title)).clicked() {
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
pub fn render_note_editor(ui: &mut egui::Ui, state: &mut DbINoteState) {
    if let Some(note_id) = state.current_note.clone() {
        // 顶部工具栏 - 固定不随内容滚动
        ui.horizontal(|ui| {
            // 只有在非全窗口最大化模式下才显示最大化按钮
            // 因为全窗口最大化模式下，返回按钮在外层UI中
            if !state.editor_maximized {
                // 全窗口最大化按钮
                if ui.button("🔍 全屏").clicked() {
                    state.toggle_editor_maximized();
                }
            }

            // 编辑/预览切换按钮
            if ui.button(if state.markdown_preview { "📝 编辑" } else { "👁 预览" }).clicked() {
                // Auto-save before switching modes
                state.auto_save_if_modified();
                state.markdown_preview = !state.markdown_preview;
            }

            // 在工具栏中显示标题输入框
            // ui.add_space(10.0);
            // ui.label("标题:");
            // // 使用与编辑器相同的等宽字体配置
            // let mut title_edit = egui::TextEdit::singleline(&mut state.note_title)
            //     .desired_width(ui.available_width() - 100.0)
            //     .font(egui::FontId::monospace(14.0))
            //     .hint_text("输入笔记标题...");

            // 在工具栏中显示标题输入框
            ui.add_space(10.0);
            // ui.label("标题:");

            // // 使用简单的布局器处理文本
            // let mut title_layouter = move |ui: &egui::Ui, text: &str, wrap_width: f32| {
            //     // 创建一个简单的布局作业
            //     let mut layout_job = egui::text::LayoutJob::default();

            //     // 设置基本字体格式
            //     let text_format = egui::TextFormat {
            //         font_id: egui::FontId::monospace(14.0),
            //         color: ui.visuals().text_color(),
            //         ..Default::default()
            //     };

            //     // 设置换行属性
            //     layout_job.wrap.max_width = wrap_width;

            //     // 直接添加整个文本，让字体系统处理字符宽度
            //     layout_job.append(text, 0.0, text_format);

            //     ui.fonts(|f| f.layout_job(layout_job))
            // };

            // // 使用与编辑器相同的等宽字体配置
            // let title_edit = egui::TextEdit::singleline(&mut state.note_title)
            //     .desired_width(ui.available_width() - 100.0)
            //     .font(egui::FontId::monospace(14.0))
            //     .hint_text("输入笔记标题...")
            //     .layouter(&mut title_layouter);

            // let title_response = ui.add(title_edit);

            // // Check for title changes and mark as modified
            // if title_response.changed() {
            //     state.check_note_modified();
            // }
        });

        ui.separator();

        // 计算编辑器高度
        let available_height = ui.available_height();
        let editor_height = if state.editor_maximized {
            // 最大化模式，几乎占满整个窗口
            available_height - 80.0
        } else {
            // 正常模式，占据大部分空间但留出空间给标签和提示
            available_height - 120.0
        };

        // 始终显示可编辑的标题 - 固定在顶部不随内容滚动
        ui.horizontal(|ui| {
            ui.label("标题:");

            // 使用与编辑器相同的等宽字体配置
            let mut title_edit = egui::TextEdit::singleline(&mut state.note_title)
                .desired_width(ui.available_width() - 50.0)
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
        ui.separator();

        // Note content - either editor or preview
        if state.markdown_preview {
            // Markdown preview
            egui::ScrollArea::vertical()
                .id_salt("markdown_preview_scroll")
                .max_height(editor_height)
                .show(ui, |ui| {
                    ui.add_space(10.0);
                    crate::markdown::render_markdown(ui, &state.note_content);
                    ui.add_space(10.0);
                });
        } else {
            // Editor mode - 使用 ScrollArea 包装编辑器
            egui::ScrollArea::vertical()
                .id_salt("editor_scroll")
                .max_height(editor_height)
                .show(ui, |ui| {
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

                    // 自定义字体渲染 - 使用简单的方法处理文本
                    let mut layouter = move |ui: &egui::Ui, text: &str, wrap_width: f32| {
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
                    };

                    text_edit = text_edit.layouter(&mut layouter);

                    let response = ui.add(text_edit);

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

        // 底部区域
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
            egui::ComboBox::from_id_salt("add_tag_dropdown")
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

            // 创建一个标志，用于在点击"复制"按钮时记录要复制的内容
            let mut copy_content = None;

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
                            if columns[2].button("复制").clicked() {
                                // 记录要复制的内容
                                copy_content = Some(heading1_code.to_string());
                            }

                            // 示例2
                            let heading2_code = "## 二级标题";
                            columns[0].label(heading2_code);
                            columns[1].heading("二级标题");
                            if columns[2].button("复制").clicked() {
                                copy_content = Some(heading2_code.to_string());
                            }

                            // 示例3
                            let heading3_code = "### 三级标题";
                            columns[0].label(heading3_code);
                            columns[1].strong("三级标题");
                            if columns[2].button("复制").clicked() {
                                copy_content = Some(heading3_code.to_string());
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
                            if columns[2].button("复制").clicked() {
                                copy_content = Some(bold_code.to_string());
                            }

                            // 斜体
                            let italic_code = "*斜体文本*";
                            columns[0].label(italic_code);
                            columns[1].label(egui::RichText::new("斜体文本").italics());
                            if columns[2].button("复制").clicked() {
                                copy_content = Some(italic_code.to_string());
                            }

                            // 代码
                            let code_code = "`代码`";
                            columns[0].label(code_code);
                            columns[1].monospace("代码");
                            if columns[2].button("复制").clicked() {
                                copy_content = Some(code_code.to_string());
                            }

                            // 删除线
                            let strike_code = "~~删除线~~";
                            columns[0].label(strike_code);
                            columns[1].label(egui::RichText::new("删除线").strikethrough());
                            if columns[2].button("复制").clicked() {
                                copy_content = Some(strike_code.to_string());
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
                            if columns[2].button("复制").clicked() {
                                copy_content = Some(list_code.to_string());
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
                            if columns[2].button("复制").clicked() {
                                copy_content = Some(ordered_list_code.to_string());
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
                            if columns[2].button("复制").clicked() {
                                copy_content = Some(link_code.to_string());
                            }

                            // 图片示例
                            let image_code = "![图片描述](图片URL)";
                            columns[0].label(image_code);
                            columns[1].label("图片将在此显示");
                            if columns[2].button("复制").clicked() {
                                copy_content = Some(image_code.to_string());
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
                            if columns[2].button("复制").clicked() {
                                copy_content = Some(quote_code.to_string());
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
                            if columns[2].button("复制").clicked() {
                                copy_content = Some(code_block.to_string());
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
                            if columns[2].button("复制").clicked() {
                                copy_content = Some(table_code.to_string());
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
                            if columns[2].button("复制").clicked() {
                                copy_content = Some(task_code.to_string());
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
                            if columns[2].button("复制").clicked() {
                                copy_content = Some(hr_code.to_string());
                            }
                        });
                    });
                });

            // 处理窗口关闭
            if !show_window {
                state.show_markdown_help = false;
            }

            // 处理复制内容
            if let Some(content) = copy_content {
                state.append_to_note_content(&content);
                state.show_markdown_help = false;
            }
        }
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