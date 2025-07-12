use eframe::egui;
use crate::db_state::DbINoteState;
use crate::hex_to_color32;


/// 渲染树状结构的笔记本和笔记
pub fn render_tree_view(ui: &mut egui::Ui, state: &mut DbINoteState) {

    ui.horizontal(|ui| {
        ui.heading("笔记树");

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button("+ 新建笔记本").clicked() {
                state.show_create_notebook = true;
            }

            if ui.button("📥 导入文档").on_hover_text("导入文档到笔记").clicked() {
                state.show_document_import_dialog = true;
            }
        });
    });

    ui.separator();

    // 笔记本和笔记的树状视图 - 移除内部滚动区域，使用外层统一滚动
    let notebooks = state.notebooks.clone();

    if notebooks.is_empty() {
        ui.label("没有笔记本");
        return;
    }

        for (notebook_idx, notebook) in notebooks.iter().enumerate() {
            let is_notebook_selected = state.current_notebook == Some(notebook_idx);

            // 笔记本行
            ui.horizontal(|ui| {
                // 展开/折叠图标
                if ui.button(if notebook.expanded { "▼" } else { "▶" }).clicked() {
                    // 切换展开状态并加载笔记
                    if let Some(nb) = state.notebooks.get_mut(notebook_idx) {
                        nb.toggle_expanded();
                        // 如果展开了，加载该笔记本的笔记
                        if nb.expanded {
                            let notebook_id = nb.id.clone();
                            state.load_notes_for_notebook(&notebook_id);
                        }
                    }
                }

                // 笔记本名称
                if ui.selectable_label(is_notebook_selected && state.current_note.is_none(),
                                      format!("📓 {}", notebook.name)).clicked() {
                    state.select_notebook(notebook_idx);
                    // 选择笔记本时清除当前笔记选择
                    state.current_note = None;
                    // 选择笔记本时自动展开并加载笔记
                    if let Some(nb) = state.notebooks.get_mut(notebook_idx) {
                        if !nb.expanded {
                            nb.expanded = true;
                            let notebook_id = nb.id.clone();
                            state.load_notes_for_notebook(&notebook_id);
                        }
                    }
                }

                // 排序按钮
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // 删除按钮
                    if ui.button("🗑").clicked() {
                        state.delete_notebook(notebook_idx);
                        return;
                    }

                    // 添加笔记按钮
                    if ui.button("+ 笔记").clicked() {
                        // 先选择笔记本，然后创建笔记
                        state.select_notebook(notebook_idx);
                        let note_id = state.create_note("新笔记".to_string(), "".to_string());
                        if let Some(id) = note_id {
                            state.select_note(&id);
                        }
                    }

                    // 下移按钮
                    if ui.add_enabled(notebook_idx < notebooks.len() - 1,
                                     egui::Button::new("↓").small())
                        .on_hover_text("向下移动笔记本")
                        .clicked() {
                        state.move_notebook_down(notebook_idx);
                    }

                    // 上移按钮
                    if ui.add_enabled(notebook_idx > 0,
                                     egui::Button::new("↑").small())
                        .on_hover_text("向上移动笔记本")
                        .clicked() {
                        state.move_notebook_up(notebook_idx);
                    }
                });
            });

            // 如果笔记本展开，显示其中的笔记
            if notebook.expanded {
                // 获取该笔记本的笔记
                let note_ids = notebook.note_ids.clone();

                // 缩进笔记，使用笔记本ID作为唯一标识
                ui.indent(format!("notes_{}", notebook.id), |ui| {
                    if note_ids.is_empty() {
                        ui.label("    没有笔记");
                    } else {
                        for note_id in note_ids {
                            // 先获取需要的数据，避免借用冲突
                            let note_title = if let Some(note) = state.notes.get(&note_id) {
                                note.title.clone()
                            } else {
                                continue;
                            };

                            let is_note_selected = state.current_note.as_ref() == Some(&note_id);
                            let note_id_clone = note_id.clone();

                            // 获取标签信息
                            // 获取标签信息并克隆所需数据，避免借用冲突
                            let note_tags = state.get_note_tags(&note_id);
                            let tag_info: Vec<(String, String, String)> = note_tags.iter()
                                .take(2)
                                .map(|tag| (tag.id.clone(), tag.name.clone(), tag.color.clone()))
                                .collect();
                            let tag_count = note_tags.len();

                            ui.horizontal(|ui| {
                                ui.add_space(20.0); // 额外缩进

                                let truncated_title = crate::truncate_note_title(&note_title);
                                if ui.selectable_label(is_note_selected,
                                                     format!("📝 {}", truncated_title)).clicked() {
                                    state.select_notebook(notebook_idx);
                                    state.select_note(&note_id_clone);
                                }

                                // 显示标签 - 限制宽度并使用省略号
                                ui.horizontal(|ui| {
                                    ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend); // 禁止换行

                                    // 限制标签区域的最大宽度
                                    let available_width = ui.available_width().min(100.0);
                                    let mut remaining_width = available_width;
                                    let mut shown_tags = 0;

                                    for (tag_id, tag_name, tag_color) in tag_info.iter() {
                                        let color = hex_to_color32(tag_color);

                                        // 处理超长标签文本
                                        let original_tag_name = tag_name.clone();

                                        // 截断超长标签名称
                                        let truncated_tag_name = if tag_name.len() > 12 {
                                            format!("{}...", &tag_name[0..9])
                                        } else {
                                            tag_name.clone()
                                        };

                                        // 计算标签文本的宽度
                                        let tag_text = format!("🏷 {}", truncated_tag_name);
                                        let galley = ui.painter().layout_no_wrap(
                                            tag_text.clone(),
                                            egui::FontId::default(),
                                            color
                                        );
                                        let tag_width = galley.rect.width() + 4.0; // 添加一些边距

                                        // 如果没有足够空间显示完整标签，显示省略号并退出循环
                                        if remaining_width < tag_width && shown_tags > 0 {
                                            break;
                                        }

                                        // 显示标签并添加悬停提示，使其可点击
                                        let tag_response = ui.add(egui::Label::new(
                                            egui::RichText::new(tag_text).color(color)
                                        ).truncate())
                                        .on_hover_text(format!("标签: {}\n点击查看此标签下的所有笔记", original_tag_name));

                                        // 点击标签时，搜索该标签下的所有笔记
                                        if tag_response.clicked() {
                                            // 使用已经克隆的标签ID
                                            let tag_id_clone = tag_id.clone();
                                            log::info!("Tree view tag clicked: {}", tag_id_clone);
                                            state.search_notes_by_tag(&tag_id_clone);
                                        }
                                        remaining_width -= tag_width;
                                        shown_tags += 1;

                                        // 最多显示2个标签
                                        if shown_tags >= 2 {
                                            break;
                                        }
                                    }

                                    // 如果有更多标签，显示"+n"
                                    if tag_count > shown_tags {
                                        ui.label(format!("+{}", tag_count - shown_tags));
                                    }
                                });

                                // 删除按钮
                                if ui.button("🗑").clicked() {
                                    state.delete_note(&note_id_clone);
                                }
                            });
                        }
                    }
                });
            }
        }
}

/// 渲染标签列表
pub fn render_tag_list(ui: &mut egui::Ui, state: &mut DbINoteState) {
    ui.horizontal(|ui| {
        ui.heading("标签");

        // 添加一个小提示，说明标签可点击
        ui.label("(可点击)")
            .on_hover_text("点击标签可查看该标签下的所有笔记");

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button("+ 新建").clicked() {
                state.show_create_tag = true;
            }
        });
    });

    ui.separator();

    // 标签列表 - 移除内部滚动区域，使用外层统一滚动
    let tags = state.tags.clone();

        for tag in &tags {
            ui.horizontal(|ui| {
                // 设置固定宽度的布局
                ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend); // 禁止换行

                // 颜色矩形
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

                // 计算可用宽度并限制标签文本宽度
                let _available_width = ui.available_width() - 30.0; // 减去删除按钮的宽度

                // 更严格地限制标签名称长度
                let original_name = tag.name.clone();
                let truncated_name = if tag.name.len() > 15 {
                    format!("{}...", &tag.name[0..12])
                } else {
                    tag.name.clone()
                };

                let tag_text = format!("🏷 {}", truncated_name);

                // 使用固定宽度的标签，并使其可点击
                let tag_id_clone = tag.id.clone();

                // 使用 selectable_label 使标签看起来更像可点击的元素
                let tag_response = ui.selectable_label(false, tag_text)
                    .on_hover_text(format!("标签: {}\n点击查看此标签下的所有笔记", original_name)); // 悬停时显示完整名称和提示

                // 点击标签时，搜索该标签下的所有笔记
                if tag_response.clicked() {
                    log::info!("Tag list tag clicked: {}", tag_id_clone);
                    state.search_notes_by_tag(&tag_id_clone);
                }

                // 使用弹性布局，将删除按钮推到最右侧
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let tag_id = tag.id.clone();
                    if ui.button("🗑").clicked() {
                        state.delete_tag(&tag_id);
                    }
                });
            });
        }

        if tags.is_empty() {
            ui.label("没有标签");
        }
}
