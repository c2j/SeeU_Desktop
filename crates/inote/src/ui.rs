use eframe::egui;
use crate::INoteState;

/// Render the notebook list
pub fn render_notebook_list(ui: &mut egui::Ui, state: &mut INoteState) {
    ui.horizontal(|ui| {
        ui.heading("笔记本");

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button("+ 新建").clicked() {
                state.show_create_notebook = true;
            }
        });
    });

    ui.separator();

    // Notebook list
    egui::ScrollArea::vertical().show(ui, |ui| {
        let notebooks = state.notebooks.clone();
        let current_notebook = state.current_notebook;

        for (i, notebook) in notebooks.iter().enumerate() {
            let is_selected = current_notebook == Some(i);

            ui.horizontal(|ui| {
                if ui.selectable_label(is_selected, format!("📓 {}", notebook.name)).clicked() {
                    state.select_notebook(i);
                }

                if ui.button("🗑").clicked() {
                    state.delete_notebook(i);
                }
            });
        }

        if notebooks.is_empty() {
            ui.label("没有笔记本");
        }
    });
}

/// Render the tag list
pub fn render_tag_list(ui: &mut egui::Ui, state: &mut INoteState) {
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
    egui::ScrollArea::vertical().show(ui, |ui| {
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
                ui.label(tag_text);

                let tag_id = tag.id.clone();
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
pub fn render_note_list(ui: &mut egui::Ui, state: &mut INoteState) {
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
            let note_data: Vec<(String, String)> = notes.iter()
                .map(|note| (note.id.clone(), note.title.clone()))
                .collect();

            let has_notes = !note_data.is_empty();

            egui::ScrollArea::vertical().show(ui, |ui| {
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
pub fn render_search_results(ui: &mut egui::Ui, state: &mut INoteState) {
    ui.horizontal(|ui| {
        ui.heading(format!("搜索结果: \"{}\"", state.search_query));

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button("✖ 关闭").clicked() {
                state.is_searching = false;
                state.search_query.clear();
                state.search_results.clear();
            }
        });
    });

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

    egui::ScrollArea::vertical().show(ui, |ui| {
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
pub fn render_note_editor(ui: &mut egui::Ui, state: &mut INoteState) {
    if let Some(note_id) = state.current_note.clone() {
        // Note title
        ui.horizontal(|ui| {
            ui.label("标题:");
            ui.add(
                egui::TextEdit::singleline(&mut state.note_title)
                    .desired_width(ui.available_width())
            );
        });

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

                if ui.add(tag_button).clicked() {
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

        ui.separator();

        // Note content
        let text_edit = egui::TextEdit::multiline(&mut state.note_content)
            .desired_width(f32::INFINITY)
            .desired_rows(20)
            .hint_text("输入笔记内容...");

        let response = ui.add(text_edit);

        // Save note when focus is lost
        let note_id_clone = note_id.clone();
        if response.lost_focus() {
            state.update_note(&note_id_clone, state.note_title.clone(), state.note_content.clone());
        }

        // Save button
        let note_id_clone = note_id.clone();
        if ui.button("保存").clicked() {
            state.update_note(&note_id_clone, state.note_title.clone(), state.note_content.clone());
            ui.label("已保存");
        }
    } else {
        ui.centered_and_justified(|ui| {
            ui.label("选择或创建一个笔记");
        });
    }
}

/// Convert hex color string to egui::Color32
fn hex_to_color32(hex: &str) -> egui::Color32 {
    let hex = hex.trim_start_matches('#');

    if hex.len() == 6 {
        // Parse RGB
        if let (Ok(r), Ok(g), Ok(b)) = (
            u8::from_str_radix(&hex[0..2], 16),
            u8::from_str_radix(&hex[2..4], 16),
            u8::from_str_radix(&hex[4..6], 16),
        ) {
            return egui::Color32::from_rgb(r, g, b);
        }
    }

    // Default color if parsing fails
    egui::Color32::BLUE
}