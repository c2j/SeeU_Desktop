use eframe::egui;
use crate::state::ITerminalState;

/// Settings category information for iterminal
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

/// Terminal settings module
pub struct ITerminalSettingsModule<'a> {
    pub state: &'a mut ITerminalState,
    pub category: SettingsCategory,
}

impl<'a> ITerminalSettingsModule<'a> {
    pub fn new(state: &'a mut ITerminalState) -> Self {
        Self {
            state,
            category: SettingsCategory::new(
                "terminal",
                "终端设置",
                "💻",
                "终端外观、行为、颜色等配置"
            ),
        }
    }
}

impl<'a> SettingsModule for ITerminalSettingsModule<'a> {
    fn get_category(&self) -> SettingsCategory {
        self.category.clone()
    }

    fn render_settings(&mut self, ui: &mut egui::Ui) -> bool {
        let mut settings_changed = false;

        ui.heading("💻 终端设置");
        ui.add_space(10.0);

        let mut config = self.state.get_config().clone();
        let mut font_scale = self.state.font_scale;

        // Appearance settings
        ui.group(|ui| {
            ui.vertical(|ui| {
                ui.label(egui::RichText::new("外观设置").strong());
                ui.add_space(5.0);

                // Font size
                ui.horizontal(|ui| {
                    ui.label("字体大小:");
                    if ui.add(egui::Slider::new(&mut config.font_size, 8.0..=24.0).suffix("px")).changed() {
                        settings_changed = true;
                    }
                });

                // Font scale
                ui.horizontal(|ui| {
                    ui.label("字体缩放:");
                    if ui.add(egui::Slider::new(&mut font_scale, 0.5..=3.0).step_by(0.1)).changed() {
                        settings_changed = true;
                    }
                });

                // Scrollback lines
                ui.horizontal(|ui| {
                    ui.label("滚动缓冲行数:");
                    if ui.add(egui::Slider::new(&mut config.scrollback_lines, 100..=50000).suffix("行")).changed() {
                        settings_changed = true;
                    }
                });
            });
        });

        ui.add_space(15.0);

        // Behavior settings
        ui.group(|ui| {
            ui.vertical(|ui| {
                ui.label(egui::RichText::new("行为设置").strong());
                ui.add_space(5.0);

                // Shell command
                ui.horizontal(|ui| {
                    ui.label("默认Shell:");
                    if ui.text_edit_singleline(&mut config.shell_command).changed() {
                        settings_changed = true;
                    }
                });

                // Enable bell
                if ui.checkbox(&mut config.enable_bell, "启用响铃").changed() {
                    settings_changed = true;
                }

                // Cursor blink
                ui.horizontal(|ui| {
                    ui.label("光标闪烁间隔:");
                    if ui.add(egui::Slider::new(&mut config.cursor_blink_interval, 0..=2000).suffix("ms")).changed() {
                        settings_changed = true;
                    }
                    if config.cursor_blink_interval == 0 {
                        ui.label("(0 = 禁用闪烁)");
                    }
                });

                // Tab size
                ui.horizontal(|ui| {
                    ui.label("Tab大小:");
                    if ui.add(egui::Slider::new(&mut config.tab_size, 2..=8).suffix("空格")).changed() {
                        settings_changed = true;
                    }
                });
            });
        });

        ui.add_space(15.0);

        // Color settings
        ui.group(|ui| {
            ui.vertical(|ui| {
                ui.label(egui::RichText::new("颜色设置").strong());
                ui.add_space(5.0);

                ui.horizontal(|ui| {
                    ui.label("背景色:");
                    let mut bg_color = egui::Color32::from_rgba_premultiplied(
                        (config.background_color[0] * 255.0) as u8,
                        (config.background_color[1] * 255.0) as u8,
                        (config.background_color[2] * 255.0) as u8,
                        (config.background_color[3] * 255.0) as u8,
                    );
                    if ui.color_edit_button_srgba(&mut bg_color).changed() {
                        let [r, g, b, a] = bg_color.to_normalized_gamma_f32();
                        config.background_color = [r, g, b, a];
                        settings_changed = true;
                    }
                });

                ui.horizontal(|ui| {
                    ui.label("文本色:");
                    let mut text_color = egui::Color32::from_rgba_premultiplied(
                        (config.text_color[0] * 255.0) as u8,
                        (config.text_color[1] * 255.0) as u8,
                        (config.text_color[2] * 255.0) as u8,
                        (config.text_color[3] * 255.0) as u8,
                    );
                    if ui.color_edit_button_srgba(&mut text_color).changed() {
                        let [r, g, b, a] = text_color.to_normalized_gamma_f32();
                        config.text_color = [r, g, b, a];
                        settings_changed = true;
                    }
                });

                ui.horizontal(|ui| {
                    ui.label("光标色:");
                    let mut cursor_color = egui::Color32::from_rgba_premultiplied(
                        (config.cursor_color[0] * 255.0) as u8,
                        (config.cursor_color[1] * 255.0) as u8,
                        (config.cursor_color[2] * 255.0) as u8,
                        (config.cursor_color[3] * 255.0) as u8,
                    );
                    if ui.color_edit_button_srgba(&mut cursor_color).changed() {
                        let [r, g, b, a] = cursor_color.to_normalized_gamma_f32();
                        config.cursor_color = [r, g, b, a];
                        settings_changed = true;
                    }
                });
            });
        });

        ui.add_space(15.0);

        // Action buttons
        ui.horizontal(|ui| {
            if ui.button("💾 保存设置").clicked() || settings_changed {
                self.state.update_config(config.clone());
                self.state.font_scale = font_scale;
                let _ = self.state.save_config();
            }

            if ui.button("🔄 重置为默认").clicked() {
                let default_config = crate::config::TerminalConfig::default();
                self.state.update_config(default_config);
                self.state.font_scale = 1.0;
                settings_changed = true;
            }

            if ui.button("📋 导出配置").clicked() {
                log::info!("Export terminal config");
                // TODO: Implement config export
            }

            if ui.button("📁 导入配置").clicked() {
                log::info!("Import terminal config");
                // TODO: Implement config import
            }
        });

        // Update the state
        if settings_changed {
            self.state.update_config(config);
            self.state.font_scale = font_scale;
        }

        settings_changed
    }

    fn save_settings(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.state.save_config()
    }

    fn load_settings(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Terminal config is loaded automatically when the state is created
        Ok(())
    }

    fn reset_to_default(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let default_config = crate::config::TerminalConfig::default();
        self.state.update_config(default_config);
        self.state.font_scale = 1.0;
        self.save_settings()?;
        Ok(())
    }

    fn get_settings_summary(&self) -> String {
        let config = self.state.get_config();
        format!("字体: {:.0}px, 缓冲: {} 行", config.font_size, config.scrollback_lines)
    }

    fn validate_settings(&self) -> Result<(), String> {
        let config = self.state.get_config();
        
        if config.font_size < 6.0 || config.font_size > 48.0 {
            return Err("字体大小必须在 6-48px 之间".to_string());
        }

        if config.scrollback_lines < 100 || config.scrollback_lines > 100000 {
            return Err("滚动缓冲行数必须在 100-100000 之间".to_string());
        }

        if config.shell_command.trim().is_empty() {
            return Err("Shell命令不能为空".to_string());
        }

        Ok(())
    }

    fn get_help_text(&self) -> Option<String> {
        Some("在这里配置终端的外观和行为。字体大小和颜色会立即生效，Shell命令在新建终端时使用。".to_string())
    }
}
