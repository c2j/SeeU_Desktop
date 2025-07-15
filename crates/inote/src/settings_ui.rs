use eframe::egui;
use crate::db_state::DbINoteState;

/// Settings category information for inote
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

/// Notes settings module
pub struct INoteSettingsModule<'a> {
    pub state: &'a mut DbINoteState,
    pub category: SettingsCategory,
}

impl<'a> INoteSettingsModule<'a> {
    pub fn new(state: &'a mut DbINoteState) -> Self {
        Self {
            state,
            category: SettingsCategory::new(
                "notes",
                "笔记设置",
                "📝",
                "笔记编辑、显示、导入导出等相关设置"
            ),
        }
    }
}

impl<'a> SettingsModule for INoteSettingsModule<'a> {
    fn get_category(&self) -> SettingsCategory {
        self.category.clone()
    }

    fn render_settings(&mut self, ui: &mut egui::Ui) -> bool {
        let mut settings_changed = false;

        ui.heading("📝 笔记设置");
        ui.add_space(10.0);

        // 数据管理设置
        ui.group(|ui| {
            ui.vertical(|ui| {
                ui.label(egui::RichText::new("数据管理").strong());
                ui.add_space(5.0);

                // 思源笔记导入
                ui.horizontal(|ui| {
                    if ui.button("📥 从思源笔记导入").clicked() {
                        self.state.siyuan_import.show_dialog = true;
                    }
                    ui.label(egui::RichText::new("导入思源笔记的数据").weak());
                });

                ui.add_space(5.0);

                // 显示导入状态
                if self.state.siyuan_import.import_in_progress {
                    ui.horizontal(|ui| {
                        ui.spinner();
                        ui.label("正在导入...");
                    });
                } else if self.state.siyuan_import.import_completed {
                    if let Some(stats) = &self.state.siyuan_import.import_stats {
                        ui.label(egui::RichText::new(format!(
                            "✅ 导入完成: {} 个笔记本, {} 个笔记",
                            stats.notebooks_count,
                            stats.notes_count
                        )).color(egui::Color32::from_rgb(0, 150, 0)));
                    }
                }

                if let Some(error) = &self.state.siyuan_import.import_error {
                    ui.label(egui::RichText::new(format!("❌ {}", error))
                        .color(egui::Color32::from_rgb(200, 0, 0)));
                }

                ui.add_space(10.0);

                // 缓存管理
                ui.horizontal(|ui| {
                    if ui.button("🧹 清理笔记缓存").clicked() {
                        self.state.clear_note_cache();
                        settings_changed = true;
                    }
                    ui.label(egui::RichText::new(format!("当前缓存: {} 个笔记", self.state.get_cache_size())).weak());
                });
            });
        });

        ui.add_space(15.0);

        // 显示设置
        ui.group(|ui| {
            ui.vertical(|ui| {
                ui.label(egui::RichText::new("显示设置").strong());
                ui.add_space(5.0);

                if ui.checkbox(&mut self.state.settings_default_collapse_notebooks, "默认折叠笔记本").changed() {
                    settings_changed = true;
                    // 立即应用设置：折叠所有笔记本
                    if self.state.settings_default_collapse_notebooks {
                        for notebook in &mut self.state.notebooks {
                            notebook.expanded = false;
                        }
                    }
                }

                if ui.checkbox(&mut self.state.settings_enable_markdown_preview, "启用Markdown预览").changed() {
                    settings_changed = true;
                }

                if ui.checkbox(&mut self.state.settings_show_note_stats, "显示笔记统计信息").changed() {
                    settings_changed = true;
                }
            });
        });

        ui.add_space(15.0);

        // 编辑器设置
        ui.group(|ui| {
            ui.vertical(|ui| {
                ui.label(egui::RichText::new("编辑器设置").strong());
                ui.add_space(5.0);

                if ui.checkbox(&mut self.state.settings_auto_save, "自动保存").changed() {
                    settings_changed = true;
                }

                if ui.checkbox(&mut self.state.settings_syntax_highlight, "语法高亮").changed() {
                    settings_changed = true;
                }

                if ui.checkbox(&mut self.state.settings_show_line_numbers, "显示行号").changed() {
                    settings_changed = true;
                }

                ui.add_space(5.0);

                // 性能设置
                ui.horizontal(|ui| {
                    ui.label("大文件阈值:");
                    let mut threshold_mb = (self.state.large_note_threshold as f32) / (1024.0 * 1024.0);
                    if ui.add(egui::Slider::new(&mut threshold_mb, 0.1..=10.0).suffix("MB")).changed() {
                        self.state.large_note_threshold = (threshold_mb * 1024.0 * 1024.0) as usize;
                        settings_changed = true;
                    }
                });
            });
        });

        ui.add_space(15.0);

        // 幻灯片设置
        ui.group(|ui| {
            ui.vertical(|ui| {
                ui.label(egui::RichText::new("幻灯片设置").strong());
                ui.add_space(5.0);

                // 当前选中的样式模板
                let current_template = self.state.slide_style_manager.get_selected_template();
                ui.horizontal(|ui| {
                    ui.label("当前样式模板:");
                    ui.label(egui::RichText::new(&current_template.name).strong());
                });

                ui.add_space(5.0);

                // 样式模板管理
                ui.horizontal(|ui| {
                    if ui.button("📝 管理样式模板").clicked() {
                        log::info!("Opening slide template manager");
                    }

                    if ui.button("🎨 创建新模板").clicked() {
                        log::info!("Creating new slide template");
                    }
                });

                ui.add_space(5.0);

                // 快速样式选择
                ui.label("快速选择样式:");
                ui.horizontal_wrapped(|ui| {
                    let all_templates = self.state.slide_style_manager.get_all_templates();
                    for template in all_templates {
                        let is_selected = template.id == self.state.slide_style_manager.selected_template_id;
                        let button_text = if is_selected {
                            format!("● {}", template.name)
                        } else {
                            template.name.clone()
                        };

                        let button = egui::Button::new(button_text)
                            .fill(if is_selected {
                                ui.style().visuals.selection.bg_fill
                            } else {
                                ui.style().visuals.widgets.inactive.bg_fill
                            });

                        if ui.add(button).clicked() && !is_selected {
                            self.state.slide_style_manager.set_selected_template(template.id.clone());
                            settings_changed = true;
                            log::info!("Selected slide template: {}", template.name);
                        }
                    }
                });
            });
        });

        ui.add_space(15.0);

        // 导入导出设置
        ui.group(|ui| {
            ui.vertical(|ui| {
                ui.label(egui::RichText::new("导入导出设置").strong());
                ui.add_space(5.0);

                ui.horizontal(|ui| {
                    if ui.button("📤 导出所有笔记").clicked() {
                        log::info!("Exporting all notes");
                    }
                    ui.label(egui::RichText::new("导出为Markdown格式").weak());
                });

                ui.horizontal(|ui| {
                    if ui.button("📥 导入笔记").clicked() {
                        log::info!("Importing notes");
                    }
                    ui.label(egui::RichText::new("支持Markdown和文本格式").weak());
                });

                ui.add_space(5.0);

                ui.horizontal(|ui| {
                    if ui.button("🔄 同步设置").clicked() {
                        log::info!("Opening sync settings");
                    }
                    ui.label(egui::RichText::new("配置云端同步").weak());
                });
            });
        });

        ui.add_space(15.0);

        // 高级功能设置
        ui.group(|ui| {
            ui.vertical(|ui| {
                ui.label(egui::RichText::new("高级功能").strong());
                ui.add_space(5.0);

                if ui.checkbox(&mut self.state.settings_enable_plugin_system, "启用插件系统")
                    .on_hover_text("允许加载第三方插件扩展功能")
                    .changed() {
                    settings_changed = true;
                }

                if ui.checkbox(&mut self.state.settings_enable_ai_integration, "启用AI集成")
                    .on_hover_text("允许AI助手访问笔记内容")
                    .changed() {
                    settings_changed = true;
                }

                if ui.checkbox(&mut self.state.settings_enable_collaboration, "启用协作功能")
                    .on_hover_text("允许多人协作编辑笔记")
                    .changed() {
                    settings_changed = true;
                }

                ui.add_space(5.0);

                ui.horizontal(|ui| {
                    ui.label("备份频率:");
                    egui::ComboBox::from_id_source("backup_frequency")
                        .selected_text("每日")
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut "daily", "daily", "每日");
                            ui.selectable_value(&mut "weekly", "weekly", "每周");
                            ui.selectable_value(&mut "monthly", "monthly", "每月");
                        });
                });
            });
        });

        settings_changed
    }

    fn save_settings(&self) -> Result<(), Box<dyn std::error::Error>> {
        crate::save_settings(self.state)?;
        // Note: slide style config saving is handled internally
        log::info!("iNote settings saved successfully");
        Ok(())
    }

    fn load_settings(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        crate::load_settings(self.state)?;
        // Note: slide style config loading is handled internally
        log::info!("iNote settings loaded successfully");
        Ok(())
    }

    fn reset_to_default(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Reset to default values
        self.state.settings_default_collapse_notebooks = true;
        self.state.settings_enable_markdown_preview = true;
        self.state.settings_show_note_stats = false;
        self.state.settings_auto_save = true;
        self.state.settings_syntax_highlight = true;
        self.state.settings_show_line_numbers = false;
        self.state.large_note_threshold = 1024 * 1024; // 1MB

        // Reset advanced features
        self.state.settings_enable_plugin_system = false;
        self.state.settings_enable_ai_integration = true;
        self.state.settings_enable_collaboration = false;

        // Clear cache
        self.state.clear_note_cache();

        // Save the reset settings
        self.save_settings()?;
        Ok(())
    }

    fn get_settings_summary(&self) -> String {
        let cache_size = self.state.get_cache_size();
        let threshold_mb = (self.state.large_note_threshold as f32) / (1024.0 * 1024.0);
        format!("缓存: {} 个笔记, 大文件阈值: {:.1}MB", cache_size, threshold_mb)
    }

    fn validate_settings(&self) -> Result<(), String> {
        if self.state.large_note_threshold < 1024 {
            return Err("大文件阈值不能小于1KB".to_string());
        }
        Ok(())
    }

    fn get_help_text(&self) -> Option<String> {
        Some("在这里可以配置笔记的显示方式、编辑器行为、幻灯片样式等。修改设置后会自动保存。".to_string())
    }
}
