use egui;
use crate::notebook::Notebook;

/// Import action type
#[derive(Debug, Clone)]
pub enum ImportAction {
    Import(String),      // Just import to notebook with given ID
    ImportAndEdit(String), // Import and open for editing
}

/// Notebook selector dialog state
#[derive(Debug, Clone)]
pub struct NotebookSelectorState {
    pub show_dialog: bool,
    pub selected_notebook_id: Option<String>,
    pub file_path: String,
    pub file_name: String,
    pub import_in_progress: bool,
    pub import_error: Option<String>,
    pub import_success: bool,
}

impl Default for NotebookSelectorState {
    fn default() -> Self {
        Self {
            show_dialog: false,
            selected_notebook_id: None,
            file_path: String::new(),
            file_name: String::new(),
            import_in_progress: false,
            import_error: None,
            import_success: false,
        }
    }
}

impl NotebookSelectorState {
    /// Show the dialog for importing a specific file
    pub fn show_for_file(&mut self, file_path: String, file_name: String) {
        self.file_path = file_path;
        self.file_name = file_name;
        self.show_dialog = true;
        self.selected_notebook_id = None;
        self.import_in_progress = false;
        self.import_error = None;
        self.import_success = false;
    }

    /// Reset the dialog state
    pub fn reset(&mut self) {
        self.show_dialog = false;
        self.selected_notebook_id = None;
        self.file_path.clear();
        self.file_name.clear();
        self.import_in_progress = false;
        self.import_error = None;
        self.import_success = false;
    }
}

/// Render the notebook selector dialog
pub fn render_notebook_selector_dialog(
    ctx: &egui::Context,
    state: &mut NotebookSelectorState,
    notebooks: &[Notebook],
) -> Option<ImportAction> {
    if !state.show_dialog {
        return None;
    }

    let mut dialog_open = true;
    let mut import_action = None;

    egui::Window::new("导入文档到笔记")
        .open(&mut dialog_open)
        .resizable(false)
        .collapsible(false)
        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .show(ctx, |ui| {
            ui.set_min_width(400.0);
            ui.set_max_width(500.0);

            // Show import progress or error
            if state.import_in_progress {
                ui.horizontal(|ui| {
                    ui.spinner();
                    ui.label("正在导入文档...");
                });
                return;
            }

            if state.import_success {
                ui.colored_label(egui::Color32::GREEN, "✓ 文档导入成功！");
                ui.add_space(10.0);
                
                ui.horizontal(|ui| {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("关闭").clicked() {
                            state.reset();
                        }
                    });
                });
                return;
            }

            if let Some(error) = &state.import_error {
                ui.colored_label(egui::Color32::RED, format!("❌ 导入失败: {}", error));
                ui.add_space(10.0);
            }

            // File information
            ui.heading("文档信息");
            ui.add_space(5.0);
            
            ui.horizontal(|ui| {
                ui.label("文件名:");
                ui.label(&state.file_name);
            });
            
            ui.horizontal(|ui| {
                ui.label("路径:");
                ui.label(egui::RichText::new(&state.file_path).weak());
            });

            ui.add_space(15.0);
            ui.separator();
            ui.add_space(15.0);

            // Notebook selection
            ui.heading("选择目标笔记本");
            ui.add_space(10.0);

            if notebooks.is_empty() {
                ui.colored_label(egui::Color32::YELLOW, "⚠ 没有可用的笔记本，请先创建一个笔记本");
                ui.add_space(10.0);
                
                ui.horizontal(|ui| {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("取消").clicked() {
                            state.reset();
                        }
                    });
                });
                return;
            }

            // Notebook list
            egui::ScrollArea::vertical()
                .max_height(200.0)
                .show(ui, |ui| {
                    for notebook in notebooks {
                        let is_selected = state.selected_notebook_id.as_ref() == Some(&notebook.id);
                        
                        if ui.selectable_label(is_selected, &notebook.name).clicked() {
                            state.selected_notebook_id = Some(notebook.id.clone());
                        }
                        
                        if !notebook.description.is_empty() {
                            ui.label(egui::RichText::new(&notebook.description).weak().small());
                        }
                        
                        ui.add_space(5.0);
                    }
                });

            ui.add_space(15.0);
            ui.separator();
            ui.add_space(10.0);

            // Action buttons
            ui.horizontal(|ui| {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let can_import = state.selected_notebook_id.is_some() && !state.import_in_progress;

                    // Import and Edit button
                    if ui.add_enabled(can_import, egui::Button::new("导入并编辑")).clicked() {
                        if let Some(notebook_id) = &state.selected_notebook_id {
                            import_action = Some(ImportAction::ImportAndEdit(notebook_id.clone()));
                        }
                    }

                    // Import button
                    if ui.add_enabled(can_import, egui::Button::new("导入")).clicked() {
                        if let Some(notebook_id) = &state.selected_notebook_id {
                            import_action = Some(ImportAction::Import(notebook_id.clone()));
                        }
                    }

                    // Cancel button
                    if ui.button("取消").clicked() {
                        state.reset();
                    }
                });
            });

            // Help text
            ui.add_space(10.0);
            ui.separator();
            ui.add_space(5.0);
            ui.label(egui::RichText::new("💡 提示: 文档将被转换为Markdown格式并保存为新笔记").weak().small());
        });

    if !dialog_open {
        state.reset();
    }

    import_action
}

/// Render a simple import button for search results
pub fn render_import_button(
    ui: &mut egui::Ui,
    file_path: &str,
    file_name: &str,
    on_click: impl FnOnce(),
) -> bool {
    // Check if the file format is supported
    if !crate::document_converter::DocumentConverter::is_supported_format(file_path) {
        return false;
    }

    // Show import button
    if ui.button("📥 导入到笔记").clicked() {
        on_click();
        return true;
    }

    false
}

/// Get file type icon for supported document formats
pub fn get_file_type_icon(file_path: &str) -> &'static str {
    if let Some(extension) = std::path::Path::new(file_path)
        .extension()
        .and_then(|ext| ext.to_str())
    {
        match extension.to_lowercase().as_str() {
            "docx" => "📝",
            "pptx" => "📽",
            "pdf" => "📄",
            "txt" => "📃",
            "md" => "📃",
            _ => "📁",
        }
    } else {
        "📁"
    }
}
