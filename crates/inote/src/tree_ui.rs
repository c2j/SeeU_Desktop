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

    // 视图模式切换控件
    ui.horizontal(|ui| {
        ui.label("视图：");

        let current_mode = state.get_note_view_mode().clone();

        // 树状视图按钮
        let tree_button_text = if current_mode == crate::db_state::NoteViewMode::TreeView {
            "🌳 树状 ✓"
        } else {
            "🌳 树状"
        };

        if ui.button(tree_button_text).clicked() {
            state.set_note_view_mode(crate::db_state::NoteViewMode::TreeView);
        }

        // 时间视图按钮
        let time_button_text = if current_mode == crate::db_state::NoteViewMode::TimeView {
            "⏰ 时间 ✓"
        } else {
            "⏰ 时间"
        };

        if ui.button(time_button_text).clicked() {
            state.set_note_view_mode(crate::db_state::NoteViewMode::TimeView);
        }

        // 在时间视图模式下显示排序控件
        if current_mode == crate::db_state::NoteViewMode::TimeView {
            ui.separator();
            ui.label("排序：");

            let current_sort = state.get_note_sort_by().clone();

            // 按添加顺序排序按钮
            let created_button_text = if current_sort == crate::db_state::NoteSortBy::CreatedTime {
                "📅 添加 ✓"
            } else {
                "📅 添加"
            };

            if ui.small_button(created_button_text).clicked() {
                state.set_note_sort_by(crate::db_state::NoteSortBy::CreatedTime);
                state.load_global_notes(); // 重新加载全局笔记
            }

            // 按更新时间排序按钮
            let updated_button_text = if current_sort == crate::db_state::NoteSortBy::UpdatedTime {
                "🕒 更新 ✓"
            } else {
                "🕒 更新"
            };

            if ui.small_button(updated_button_text).clicked() {
                state.set_note_sort_by(crate::db_state::NoteSortBy::UpdatedTime);
                state.load_global_notes(); // 重新加载全局笔记
            }
        }
    });

    ui.separator();

    // 根据视图模式渲染不同内容
    let current_mode = state.get_note_view_mode().clone();

    match current_mode {
        crate::db_state::NoteViewMode::TreeView => {
            render_tree_mode(ui, state);
        },
        crate::db_state::NoteViewMode::TimeView => {
            render_time_mode(ui, state);
        }
    }
}

/// 渲染树状视图模式
fn render_tree_mode(ui: &mut egui::Ui, state: &mut DbINoteState) {
    let notebooks = state.notebooks.clone();

    if notebooks.is_empty() {
        ui.label("没有笔记本");
        return;
    }

        for (notebook_idx, notebook) in notebooks.iter().enumerate() {
            let is_notebook_selected = state.current_notebook == Some(notebook_idx);

            // 笔记本行
            let notebook_response = ui.horizontal(|ui| {
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
                let name_response = ui.selectable_label(is_notebook_selected && state.current_note.is_none(),
                                      format!("📓 {}", notebook.name));
                if name_response.clicked() {
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

                // 检测鼠标是否悬停在笔记本行上
                let is_hovered = name_response.hovered();

                // 排序按钮 - 只在悬停时显示
                if is_hovered {
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
                }
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

/// 渲染时间视图模式（全局笔记按时间排序）
fn render_time_mode(ui: &mut egui::Ui, state: &mut DbINoteState) {
    // 确保全局笔记列表已加载
    if state.global_notes.is_empty() {
        state.load_global_notes();
    }

    let global_notes = state.global_notes.clone();

    if global_notes.is_empty() {
        ui.label("没有笔记");
        return;
    }

    ui.label(format!("共 {} 个笔记", global_notes.len()));
    ui.add_space(5.0);

    for note_id in global_notes {
        // 获取笔记信息
        let note_info = if let Some(note) = state.notes.get(&note_id) {
            (note.title.clone(), note.created_at.clone(), note.updated_at.clone())
        } else {
            continue;
        };

        let is_note_selected = state.current_note.as_ref() == Some(&note_id);
        let note_id_clone = note_id.clone();

        // 获取笔记所属的笔记本名称
        let notebook_name = state.notebooks.iter()
            .find(|nb| nb.note_ids.contains(&note_id))
            .map(|nb| nb.name.clone())
            .unwrap_or_else(|| "未知笔记本".to_string());

        // 获取标签信息
        let note_tags = state.get_note_tags(&note_id);
        let tag_info: Vec<(String, String, String)> = note_tags.iter()
            .take(2)
            .map(|tag| (tag.id.clone(), tag.name.clone(), tag.color.clone()))
            .collect();
        let tag_count = note_tags.len();

        ui.horizontal(|ui| {
            // 笔记标题和选择
            let truncated_title = crate::truncate_note_title(&note_info.0);
            if ui.selectable_label(is_note_selected, format!("📝 {}", truncated_title)).clicked() {
                // 找到笔记所属的笔记本并选择
                if let Some((notebook_idx, _)) = state.notebooks.iter().enumerate()
                    .find(|(_, nb)| nb.note_ids.contains(&note_id)) {
                    state.select_notebook(notebook_idx);
                }
                state.select_note(&note_id_clone);
            }

            // 显示笔记本名称
            ui.label(format!("📓 {}", notebook_name));

            // 显示标签
            ui.horizontal(|ui| {
                for (tag_id, tag_name, tag_color) in tag_info {
                    let color = hex_to_color32(&tag_color);
                    ui.colored_label(color, format!("#{}", tag_name));
                }

                if tag_count > 2 {
                    ui.label(format!("+{}", tag_count - 2));
                }
            });

            // 显示时间信息
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let current_sort = state.get_note_sort_by();
                let time_text = match current_sort {
                    crate::db_state::NoteSortBy::CreatedTime => {
                        format!("创建: {}", note_info.1)
                    },
                    crate::db_state::NoteSortBy::UpdatedTime => {
                        format!("更新: {}", note_info.2)
                    }
                };
                ui.label(egui::RichText::new(time_text).small());
            });
        });

        ui.add_space(2.0);
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
