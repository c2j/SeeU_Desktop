use eframe::egui;
use crate::ISearchState;

/// Render the search bar
pub fn render_search_bar(ui: &mut egui::Ui, state: &mut ISearchState) {
    ui.horizontal(|ui| {
        ui.label("🔍");
        ui.add(
            egui::TextEdit::singleline(&mut state.search_query)
                .hint_text("搜索文件...")
                .desired_width(ui.available_width() - 100.0)
        );

        if ui.button("搜索").clicked() {
            // TODO: Implement search
            log::info!("Search files: {}", state.search_query);
        }
    });
}

/// Render the directory list
pub fn render_directory_list(ui: &mut egui::Ui, _state: &mut ISearchState) {
    ui.heading("索引目录");

    if ui.button("+ 添加目录").clicked() {
        // TODO: Add directory
        log::info!("Add directory");
    }

    ui.separator();

    // Directory list
    egui::ScrollArea::vertical().show(ui, |ui| {
        // Sample directories
        if ui.selectable_label(false, "📁 /home/user/Documents").clicked() {
            // TODO: Select directory
            log::info!("Select directory: /home/user/Documents");
        }

        if ui.selectable_label(false, "📁 /home/user/Downloads").clicked() {
            // TODO: Select directory
            log::info!("Select directory: /home/user/Downloads");
        }
    });
}

/// Render the search results
pub fn render_search_results(ui: &mut egui::Ui, state: &mut ISearchState) {
    ui.heading("搜索结果");

    if state.search_query.is_empty() {
        ui.centered_and_justified(|ui| {
            ui.label("请输入搜索关键词...");
        });
    } else if state.search_results.is_empty() && !state.is_searching {
        ui.centered_and_justified(|ui| {
            ui.label("未找到匹配结果");
        });
    } else if state.is_searching {
        ui.centered_and_justified(|ui| {
            ui.spinner();
            ui.label("正在搜索...");
        });
    } else {
        // Search results
        egui::ScrollArea::vertical().show(ui, |ui| {
            for result in &state.search_results {
                ui.push_id(&result.id, |ui| {
                    ui.add_space(4.0);

                    // File name and icon
                    ui.horizontal(|ui| {
                        // File type icon
                        let icon = match result.file_type.as_str() {
                            "pdf" => "📄",
                            "doc" | "docx" => "📝",
                            "xls" | "xlsx" => "📊",
                            "ppt" | "pptx" => "📽",
                            "txt" | "md" => "📃",
                            "rs" | "js" | "py" | "cpp" => "💻",
                            "jpg" | "png" | "gif" => "🖼",
                            _ => "📁",
                        };
                        ui.label(icon);

                        // File name
                        ui.heading(&result.filename);

                        // File size and date
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(format!("{}", result.modified.format("%Y-%m-%d %H:%M")));
                            ui.label(format!("{:.1} KB", result.size_bytes as f64 / 1024.0));
                        });
                    });

                    // File path
                    ui.horizontal(|ui| {
                        ui.label("📂");
                        ui.label(&result.path);
                    });

                    // Content preview
                    if !result.content_preview.is_empty() {
                        ui.add_space(4.0);
                        ui.add(egui::Label::new(&result.content_preview).wrap());
                    }

                    // Open file button
                    if ui.button("打开文件").clicked() {
                        // TODO: Open file
                        log::info!("Open file: {}", result.path);
                    }

                    ui.add_space(4.0);
                    ui.separator();
                });
            }
        });
    }
}