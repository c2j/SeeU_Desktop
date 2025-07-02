use eframe::egui;
use crate::ISearchState;

/// Settings category information for isearch
#[derive(Debug, Clone, PartialEq)]
pub struct SettingsCategory {
    pub id: String,
    pub display_name: String,
    pub icon: String,
    pub description: String,
}

impl SettingsCategory {
    pub fn new(id: &str, display_name: &str, icon: &str, description: &str) -> Self {
        Self {
            id: id.to_string(),
            display_name: display_name.to_string(),
            icon: icon.to_string(),
            description: description.to_string(),
        }
    }

    /// Get the full display name with icon
    pub fn full_display_name(&self) -> String {
        format!("{} {}", self.icon, self.display_name)
    }
}

/// Trait for settings modules
pub trait SettingsModule {
    /// Get the settings category information
    fn get_category(&self) -> SettingsCategory;

    /// Render the settings UI for this module
    /// Returns true if any settings were changed
    fn render_settings(&mut self, ui: &mut egui::Ui) -> bool;

    /// Save settings to persistent storage
    fn save_settings(&self) -> Result<(), Box<dyn std::error::Error>>;

    /// Load settings from persistent storage
    fn load_settings(&mut self) -> Result<(), Box<dyn std::error::Error>>;

    /// Reset settings to default values
    fn reset_to_default(&mut self) -> Result<(), Box<dyn std::error::Error>>;

    /// Get a brief status or summary of current settings
    fn get_settings_summary(&self) -> String {
        "设置已配置".to_string()
    }

    /// Check if settings are valid
    fn validate_settings(&self) -> Result<(), String> {
        Ok(())
    }

    /// Get help text for this settings module
    fn get_help_text(&self) -> Option<String> {
        None
    }
}

/// Search settings module
pub struct ISearchSettingsModule<'a> {
    pub state: &'a mut ISearchState,
    pub category: SettingsCategory,
}

impl<'a> ISearchSettingsModule<'a> {
    pub fn new(state: &'a mut ISearchState) -> Self {
        Self {
            state,
            category: SettingsCategory::new(
                "search",
                "搜索设置",
                "🔍",
                "文件索引、搜索选项、目录管理等设置"
            ),
        }
    }
}

impl<'a> SettingsModule for ISearchSettingsModule<'a> {
    fn get_category(&self) -> SettingsCategory {
        self.category.clone()
    }

    fn render_settings(&mut self, ui: &mut egui::Ui) -> bool {
        let mut settings_changed = false;

        // Check for completed indexing operations to update UI (now non-blocking)
        self.state.check_reindex_results();

        ui.heading("🔍 搜索设置");
        ui.add_space(10.0);

        // Process directory dialog in settings
        self.state.process_directory_dialog();

        // Indexed directories management
        ui.group(|ui| {
            ui.vertical(|ui| {
                ui.label(egui::RichText::new("索引目录管理").strong());
                ui.add_space(5.0);

                // Directory management buttons
                ui.horizontal(|ui| {
                    if ui.button("+ 添加目录").clicked() {
                        self.state.open_directory_dialog();
                    }

                    // Remove directory button (only enabled if a directory is selected)
                    if let Some(selected) = self.state.selected_directory {
                        if ui.button("- 移除目录").clicked() {
                            self.state.remove_directory(selected);
                            settings_changed = true;
                        }
                    } else {
                        ui.add_enabled(false, egui::Button::new("- 移除目录"));
                    }

                    // Update all indexes button
                    if ui.button("🔄 重新索引全部").on_hover_text("重新索引所有目录，应用最新功能改进").clicked() {
                        self.state.reindex_all_directories();
                    }
                });

                ui.add_space(8.0);

                // Directory list - full width
                ui.label("已索引的目录：");
                egui::ScrollArea::vertical()
                    .max_height(200.0)
                    .show(ui, |ui| {
                        if self.state.indexed_directories.is_empty() {
                            ui.label("暂无索引目录");
                            ui.add_space(5.0);
                            ui.label(egui::RichText::new("提示：添加目录后才能进行文件搜索").weak());
                        } else {
                            // Clone the directories to avoid borrowing issues
                            let directories = self.state.indexed_directories.clone();
                            let selected_directory = self.state.selected_directory;

                            for (i, directory) in directories.iter().enumerate() {
                                let is_selected = selected_directory == Some(i);

                                // Full width group for each directory
                                ui.allocate_ui_with_layout(
                                    egui::Vec2::new(ui.available_width(), 0.0),
                                    egui::Layout::top_down(egui::Align::LEFT),
                                    |ui| {
                                        ui.group(|ui| {
                                            ui.set_min_width(ui.available_width());
                                            ui.vertical(|ui| {
                                                // Directory path with wrapping
                                                let path_text = format!("📁 {}", directory.path);
                                                ui.allocate_ui_with_layout(
                                                    egui::Vec2::new(ui.available_width(), 0.0),
                                                    egui::Layout::top_down(egui::Align::LEFT),
                                                    |ui| {
                                                        if ui.selectable_label(is_selected, &path_text).clicked() {
                                                            self.state.selected_directory = Some(i);
                                                        }
                                                    }
                                                );

                                                // Directory stats in a horizontal layout
                                                ui.horizontal(|ui| {
                                                    ui.label(egui::RichText::new(format!("📄 {} 个文件", directory.file_count)).small().weak());
                                                    ui.label(egui::RichText::new(format!("💾 {:.1} MB", directory.total_size_bytes as f64 / (1024.0 * 1024.0))).small().weak());
                                                });

                                                // Last indexed time
                                                if let Some(last_indexed) = directory.last_indexed {
                                                    ui.label(egui::RichText::new(format!("🕒 最后索引: {}", last_indexed.format("%m-%d %H:%M"))).small().weak());
                                                } else {
                                                    ui.label(egui::RichText::new("🕒 未索引").small().weak());
                                                }

                                                // Update button for this directory
                                                ui.horizontal(|ui| {
                                                    if ui.small_button("🔄 更新此目录").on_hover_text("重新索引此目录").clicked() {
                                                        self.state.update_directory_index(i);
                                                    }
                                                });
                                            });
                                        });
                                    }
                                );

                                ui.add_space(4.0);
                            }
                        }
                    });

                ui.add_space(8.0);

                // Directory status info
                ui.horizontal(|ui| {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(egui::RichText::new("选择目录后可移除或重新索引").weak());
                    });
                });
            });
        });

        ui.add_space(15.0);

        // Index statistics
        ui.group(|ui| {
            ui.vertical(|ui| {
                ui.label(egui::RichText::new("索引统计").strong());
                ui.add_space(5.0);

                ui.horizontal(|ui| {
                    ui.label("已索引文件数:");
                    ui.label(format!("{}", self.state.index_stats.total_files));
                });

                ui.horizontal(|ui| {
                    ui.label("索引大小:");
                    ui.label(format!("{:.1} MB", self.state.index_stats.total_size_bytes as f64 / (1024.0 * 1024.0)));
                });

                if let Some(last_updated) = self.state.index_stats.last_updated {
                    ui.horizontal(|ui| {
                        ui.label("最后更新:");
                        ui.label(format!("{}", last_updated.format("%Y-%m-%d %H:%M")));
                    });
                }

                if self.state.is_indexing {
                    ui.add_space(5.0);
                    ui.horizontal(|ui| {
                        ui.spinner();
                        ui.label("正在索引...");
                    });
                }
            });
        });

        ui.add_space(15.0);

        // Search options
        ui.group(|ui| {
            ui.vertical(|ui| {
                ui.label(egui::RichText::new("搜索选项").strong());
                ui.add_space(5.0);

                if ui.checkbox(&mut self.state.enable_content_preview, "启用内容预览").changed() {
                    settings_changed = true;
                }
                if ui.checkbox(&mut self.state.enable_file_type_filter, "启用文件类型筛选").changed() {
                    settings_changed = true;
                }
                if ui.checkbox(&mut self.state.search_hidden_files, "搜索隐藏文件").changed() {
                    settings_changed = true;
                }
                if ui.checkbox(&mut self.state.enable_file_monitoring, "实时文件监控").changed() {
                    settings_changed = true;
                }
                if ui.checkbox(&mut self.state.search_on_typing, "输入时触发搜索").on_hover_text("启用后每次输入都会触发搜索，禁用后需按回车键触发").changed() {
                    settings_changed = true;
                }

                // Auto-save search options when changed
                if settings_changed {
                    self.state.save_search_options();
                }
            });
        });

        // Directory input dialog (替代文件对话框)
        if self.state.show_directory_input_dialog {
            egui::Window::new("添加索引目录")
                .collapsible(false)
                .resizable(false)
                .default_width(400.0)
                .show(ui.ctx(), |ui| {
                    ui.vertical(|ui| {
                        ui.label("请输入要索引的目录路径：");
                        ui.add_space(5.0);

                        let response = ui.add(
                            egui::TextEdit::singleline(&mut self.state.directory_input)
                                .hint_text("例如：/home/user/Documents")
                                .desired_width(ui.available_width())
                        );

                        // 自动聚焦输入框
                        if self.state.show_directory_input_dialog {
                            response.request_focus();
                        }

                        ui.add_space(10.0);

                        // 常用目录快捷按钮
                        ui.label("常用目录：");
                        ui.horizontal_wrapped(|ui| {
                            if let Some(home_dir) = dirs::home_dir() {
                                if ui.small_button("🏠 主目录").clicked() {
                                    self.state.directory_input = home_dir.to_string_lossy().to_string();
                                }
                            }

                            if let Some(documents_dir) = dirs::document_dir() {
                                if ui.small_button("📄 文档").clicked() {
                                    self.state.directory_input = documents_dir.to_string_lossy().to_string();
                                }
                            }

                            if let Some(downloads_dir) = dirs::download_dir() {
                                if ui.small_button("📥 下载").clicked() {
                                    self.state.directory_input = downloads_dir.to_string_lossy().to_string();
                                }
                            }

                            if let Some(desktop_dir) = dirs::desktop_dir() {
                                if ui.small_button("🖥 桌面").clicked() {
                                    self.state.directory_input = desktop_dir.to_string_lossy().to_string();
                                }
                            }
                        });

                        ui.add_space(10.0);
                        ui.separator();
                        ui.add_space(5.0);

                        // 按钮
                        ui.horizontal(|ui| {
                            if ui.button("添加").clicked() ||
                               (response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter))) {
                                self.state.add_directory_from_input();
                                settings_changed = true;
                            }

                            if ui.button("取消").clicked() {
                                self.state.show_directory_input_dialog = false;
                                self.state.directory_input.clear();
                            }
                        });
                    });
                });
        }

        settings_changed
    }

    fn save_settings(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.state.save_search_options();
        Ok(())
    }

    fn load_settings(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Search options are loaded automatically when the state is created
        Ok(())
    }

    fn reset_to_default(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Reset search options to default
        self.state.enable_content_preview = true;
        self.state.enable_file_type_filter = true;
        self.state.search_hidden_files = false;
        self.state.enable_file_monitoring = true;
        self.state.search_on_typing = true;
        
        // Save the reset settings
        self.save_settings()?;
        Ok(())
    }

    fn get_settings_summary(&self) -> String {
        let dir_count = self.state.indexed_directories.len();
        let file_count = self.state.index_stats.total_files;
        format!("目录: {}, 文件: {}", dir_count, file_count)
    }

    fn validate_settings(&self) -> Result<(), String> {
        if self.state.indexed_directories.is_empty() {
            return Err("至少需要添加一个索引目录".to_string());
        }
        Ok(())
    }

    fn get_help_text(&self) -> Option<String> {
        Some("在这里管理搜索索引目录和搜索选项。添加目录后系统会自动建立索引，支持实时文件监控和内容预览。".to_string())
    }
}
