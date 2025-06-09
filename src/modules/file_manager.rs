use eframe::egui;

/// File manager module state
pub struct FileManagerState {
    current_path: String,
    selected_file: Option<String>,
}

impl Default for FileManagerState {
    fn default() -> Self {
        Self {
            current_path: std::env::current_dir()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string(),
            selected_file: None,
        }
    }
}

/// Render the file manager module
pub fn render_file_manager(ui: &mut egui::Ui) {
    // 获取可用高度
    let available_height = ui.available_height();

    // 创建一个垂直布局容器，确保内容撑满高度
    egui::containers::Frame::none()
        .fill(ui.style().visuals.window_fill)
        .show(ui, |ui| {
            // 设置最小高度，确保撑满可用空间
            ui.set_min_height(available_height);

            ui.vertical(|ui| {
                ui.label("文件管理模块尚未完全实现");

                // Path bar
                ui.horizontal(|ui| {
                    ui.label("路径:");
                    let mut path = std::env::current_dir()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string();

                    ui.add(
                        egui::TextEdit::singleline(&mut path)
                            .desired_width(ui.available_width() - 50.0)
                    );

                    if ui.button("转到").clicked() {
                        // TODO: Navigate to path
                        log::info!("Navigate to: {}", path);
                    }
                });

                ui.separator();

                // 计算文件列表区域高度（减去标题、路径栏和分隔符的高度）
                let file_list_height = available_height - 80.0;

                // File list
                egui::ScrollArea::vertical()
                    .max_height(file_list_height)
                    .show(ui, |ui| {
                        // Parent directory
                        if ui.selectable_label(false, "📁 ..").clicked() {
                            // TODO: Navigate to parent directory
                            log::info!("Navigate to parent directory");
                        }

                        // Sample files
                        if ui.selectable_label(false, "📁 Documents").clicked() {
                            // TODO: Navigate to directory
                            log::info!("Navigate to Documents");
                        }

                        if ui.selectable_label(false, "📁 Downloads").clicked() {
                            // TODO: Navigate to directory
                            log::info!("Navigate to Downloads");
                        }

                        if ui.selectable_label(false, "📄 README.md").clicked() {
                            // TODO: Select file
                            log::info!("Select README.md");
                        }

                        if ui.selectable_label(false, "📄 Cargo.toml").clicked() {
                            // TODO: Select file
                            log::info!("Select Cargo.toml");
                        }

                        // 添加更多示例文件，确保滚动区域有足够的内容
                        for i in 1..20 {
                            if ui.selectable_label(false, format!("📄 File_{}.txt", i)).clicked() {
                                // TODO: Select file
                                log::info!("Select File_{}.txt", i);
                            }
                        }
                    });
            });
        });
}