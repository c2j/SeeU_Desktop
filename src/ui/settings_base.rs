use eframe::egui;
use super::settings_trait::{SettingsCategory, SettingsModule};

/// Base settings module for application-level settings
pub struct AppSettingsModule {
    pub category: SettingsCategory,
    // Temporary settings for editing
    pub temp_auto_startup: bool,
    pub temp_restore_session: bool,
    pub temp_auto_save: bool,
    pub temp_periodic_backup: bool,
}

impl AppSettingsModule {
    pub fn new() -> Self {
        Self {
            category: SettingsCategory::new(
                "app",
                "应用设置",
                "🔧",
                "应用程序的基本设置，包括启动、数据管理等"
            ),
            temp_auto_startup: false,
            temp_restore_session: true,
            temp_auto_save: true,
            temp_periodic_backup: false,
        }
    }

    /// Initialize temporary settings from app settings
    pub fn init_from_app_settings(&mut self, app_settings: &crate::app::AppSettings) {
        self.temp_auto_startup = app_settings.auto_startup;
        self.temp_restore_session = app_settings.restore_session;
        self.temp_auto_save = app_settings.auto_save;
        self.temp_periodic_backup = app_settings.periodic_backup;
    }

    /// Apply temporary settings to app settings
    pub fn apply_to_app_settings(&self, app_settings: &mut crate::app::AppSettings) {
        app_settings.auto_startup = self.temp_auto_startup;
        app_settings.restore_session = self.temp_restore_session;
        app_settings.auto_save = self.temp_auto_save;
        app_settings.periodic_backup = self.temp_periodic_backup;
    }
}

impl SettingsModule for AppSettingsModule {
    fn get_category(&self) -> SettingsCategory {
        self.category.clone()
    }

    fn render_settings(&mut self, ui: &mut egui::Ui) -> bool {
        let mut settings_changed = false;

        ui.heading("🔧 应用设置");
        ui.add_space(10.0);

        ui.group(|ui| {
            ui.vertical(|ui| {
                ui.label(egui::RichText::new("启动设置").strong());
                ui.add_space(5.0);

                if ui.checkbox(&mut self.temp_auto_startup, "自动启动")
                    .on_hover_text("应用程序随系统启动")
                    .changed() {
                    settings_changed = true;
                }

                if ui.checkbox(&mut self.temp_restore_session, "会话恢复")
                    .on_hover_text("启动时恢复上次的工作状态")
                    .changed() {
                    settings_changed = true;
                }
            });
        });

        ui.add_space(15.0);

        ui.group(|ui| {
            ui.vertical(|ui| {
                ui.label(egui::RichText::new("数据管理").strong());
                ui.add_space(5.0);

                if ui.checkbox(&mut self.temp_auto_save, "自动保存")
                    .on_hover_text("自动保存工作内容")
                    .changed() {
                    settings_changed = true;
                }

                if ui.checkbox(&mut self.temp_periodic_backup, "定期备份")
                    .on_hover_text("定期备份重要数据")
                    .changed() {
                    settings_changed = true;
                }
            });
        });

        ui.add_space(15.0);

        if settings_changed {
            ui.group(|ui| {
                ui.vertical(|ui| {
                    ui.label(egui::RichText::new("提示").color(egui::Color32::YELLOW));
                    ui.label("设置已修改，请点击保存按钮应用更改。");
                });
            });
        }

        settings_changed
    }

    fn save_settings(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Application settings are saved through the main app
        Ok(())
    }

    fn load_settings(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Application settings are loaded through the main app
        Ok(())
    }

    fn reset_to_default(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Application settings reset is handled through the main app
        Ok(())
    }

    fn get_settings_summary(&self) -> String {
        "应用程序基本设置".to_string()
    }
}

/// Appearance settings module
pub struct AppearanceSettingsModule {
    pub category: SettingsCategory,
    // Temporary settings for editing
    pub temp_theme: crate::ui::theme::Theme,
    pub temp_font_size: f32,
    pub temp_font_family: String,
    pub temp_ui_scale: f32,
}

impl AppearanceSettingsModule {
    pub fn new() -> Self {
        Self {
            category: SettingsCategory::new(
                "appearance",
                "外观设置",
                "🎨",
                "主题、字体、界面缩放等外观相关设置"
            ),
            temp_theme: crate::ui::theme::Theme::DarkModern,
            temp_font_size: 14.0,
            temp_font_family: "Default".to_string(),
            temp_ui_scale: 1.0,
        }
    }

    /// Initialize temporary settings from app settings
    pub fn init_from_app(&mut self, theme: crate::ui::theme::Theme, app_settings: &crate::app::AppSettings) {
        self.temp_theme = theme;
        self.temp_font_size = app_settings.font_size;
        self.temp_font_family = app_settings.font_family.clone();
        self.temp_ui_scale = app_settings.ui_scale;
    }

    /// Apply temporary settings to app
    pub fn apply_to_app(&self, app: &mut crate::app::SeeUApp, ctx: &egui::Context) {
        // Apply theme if changed
        if app.theme != self.temp_theme {
            app.set_theme(ctx, self.temp_theme);
        }

        // Apply font settings if changed
        if app.app_settings.font_size != self.temp_font_size {
            app.set_font_size(ctx, self.temp_font_size);
        }

        if app.app_settings.font_family != self.temp_font_family {
            app.app_settings.font_family = self.temp_font_family.clone();
            app.update_fonts(ctx);
        }

        // Apply UI scale if changed
        if app.app_settings.ui_scale != self.temp_ui_scale {
            app.set_ui_scale(ctx, self.temp_ui_scale);
        }
    }
}

impl SettingsModule for AppearanceSettingsModule {
    fn get_category(&self) -> SettingsCategory {
        self.category.clone()
    }

    fn render_settings(&mut self, ui: &mut egui::Ui) -> bool {
        let mut settings_changed = false;

        ui.heading("🎨 外观设置");
        ui.add_space(10.0);

        // Theme selection
        ui.group(|ui| {
            ui.vertical(|ui| {
                ui.label(egui::RichText::new("主题设置").strong());
                ui.add_space(5.0);

                ui.horizontal(|ui| {
                    ui.label("主题:");
                    egui::ComboBox::from_id_source("theme_selector")
                        .selected_text(self.temp_theme.display_name())
                        .show_ui(ui, |ui| {
                            let themes = [
                                crate::ui::theme::Theme::DarkModern,
                                crate::ui::theme::Theme::Dark,
                                crate::ui::theme::Theme::LightModern,
                                crate::ui::theme::Theme::Light,
                            ];
                            for theme in themes {
                                if ui.selectable_value(&mut self.temp_theme, theme, theme.display_name()).clicked() {
                                    settings_changed = true;
                                }
                            }
                        });
                });
            });
        });

        ui.add_space(15.0);

        // Font settings
        ui.group(|ui| {
            ui.vertical(|ui| {
                ui.label(egui::RichText::new("字体设置").strong());
                ui.add_space(5.0);

                ui.horizontal(|ui| {
                    ui.label("字体族:");
                    egui::ComboBox::from_id_source("font_family_selector")
                        .selected_text(&self.temp_font_family)
                        .show_ui(ui, |ui| {
                            let fonts = ["Default", "Consolas", "Monaco", "Menlo", "Source Code Pro"];
                            for font in fonts {
                                if ui.selectable_value(&mut self.temp_font_family, font.to_string(), font).clicked() {
                                    settings_changed = true;
                                }
                            }
                        });
                });

                ui.horizontal(|ui| {
                    ui.label("字体大小:");
                    if ui.add(egui::Slider::new(&mut self.temp_font_size, 8.0..=32.0)
                        .suffix("px")
                        .step_by(1.0))
                        .changed() {
                        settings_changed = true;
                    }
                });
            });
        });

        ui.add_space(15.0);

        // UI scale settings
        ui.group(|ui| {
            ui.vertical(|ui| {
                ui.label(egui::RichText::new("界面缩放").strong());
                ui.add_space(5.0);

                ui.horizontal(|ui| {
                    ui.label("缩放比例:");
                    if ui.add(egui::Slider::new(&mut self.temp_ui_scale, 0.5..=3.0)
                        .suffix("x")
                        .step_by(0.1))
                        .changed() {
                        settings_changed = true;
                    }
                });

                ui.label("提示: 调整界面元素的大小");
            });
        });

        ui.add_space(15.0);

        if settings_changed {
            ui.group(|ui| {
                ui.vertical(|ui| {
                    ui.label(egui::RichText::new("提示").color(egui::Color32::YELLOW));
                    ui.label("设置已修改，请点击保存按钮应用更改。");
                });
            });
        }

        settings_changed
    }

    fn save_settings(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Appearance settings are saved through the main app
        Ok(())
    }

    fn load_settings(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Appearance settings are loaded through the main app
        Ok(())
    }

    fn reset_to_default(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Appearance settings reset is handled through the main app
        Ok(())
    }

    fn get_settings_summary(&self) -> String {
        "外观和主题设置".to_string()
    }

    fn get_help_text(&self) -> Option<String> {
        Some("在这里可以调整应用程序的外观，包括主题颜色、字体大小和界面缩放比例。".to_string())
    }
}

/// Advanced settings module
pub struct AdvancedSettingsModule {
    pub category: SettingsCategory,
    // Temporary settings for editing
    pub temp_hardware_acceleration: bool,
    pub temp_debug_mode: bool,
    pub show_reset_confirmation: bool,
}

impl AdvancedSettingsModule {
    pub fn new() -> Self {
        Self {
            category: SettingsCategory::new(
                "advanced",
                "高级设置",
                "⚙️",
                "性能、调试、开发者选项等高级设置"
            ),
            temp_hardware_acceleration: true,
            temp_debug_mode: false,
            show_reset_confirmation: false,
        }
    }

    /// Open log directory
    fn open_log_directory(&self) -> Result<(), Box<dyn std::error::Error>> {
        let log_dir = dirs::config_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join("seeu_desktop")
            .join("logs");

        // Create log directory if it doesn't exist
        std::fs::create_dir_all(&log_dir)?;

        // Open directory in system file manager
        #[cfg(target_os = "windows")]
        {
            std::process::Command::new("explorer")
                .arg(&log_dir)
                .spawn()?;
        }

        #[cfg(target_os = "macos")]
        {
            std::process::Command::new("open")
                .arg(&log_dir)
                .spawn()?;
        }

        #[cfg(target_os = "linux")]
        {
            std::process::Command::new("xdg-open")
                .arg(&log_dir)
                .spawn()?;
        }

        log::info!("Opened log directory: {:?}", log_dir);
        Ok(())
    }
}

impl SettingsModule for AdvancedSettingsModule {
    fn get_category(&self) -> SettingsCategory {
        self.category.clone()
    }

    fn render_settings(&mut self, ui: &mut egui::Ui) -> bool {
        let mut settings_changed = false;

        ui.heading("⚙️ 高级设置");
        ui.add_space(10.0);

        // Performance settings
        ui.group(|ui| {
            ui.vertical(|ui| {
                ui.label(egui::RichText::new("性能设置").strong());
                ui.add_space(5.0);

                if ui.checkbox(&mut self.temp_hardware_acceleration, "启用硬件加速")
                    .on_hover_text("使用GPU加速渲染，提升界面性能")
                    .changed() {
                    settings_changed = true;
                }

                if ui.checkbox(&mut self.temp_debug_mode, "调试模式")
                    .on_hover_text("启用详细日志记录和调试信息")
                    .changed() {
                    settings_changed = true;
                }
            });
        });

        ui.add_space(15.0);

        // Developer options
        ui.group(|ui| {
            ui.vertical(|ui| {
                ui.label(egui::RichText::new("开发者选项").strong());
                ui.add_space(5.0);

                if ui.button("📁 打开日志目录").clicked() {
                    if let Err(err) = self.open_log_directory() {
                        log::error!("Failed to open log directory: {}", err);
                    }
                }

                ui.add_space(5.0);

                if ui.button("🔄 重置所有设置").clicked() {
                    self.show_reset_confirmation = true;
                }
            });
        });

        // Reset confirmation dialog
        if self.show_reset_confirmation {
            egui::Window::new("确认重置")
                .collapsible(false)
                .resizable(false)
                .show(ui.ctx(), |ui| {
                    ui.vertical(|ui| {
                        ui.label("⚠️ 警告");
                        ui.add_space(5.0);
                        ui.label("这将重置所有设置到默认值，");
                        ui.label("包括主题、字体、窗口布局等。");
                        ui.label("此操作无法撤销！");
                        ui.add_space(10.0);

                        ui.horizontal(|ui| {
                            if ui.button("确认重置").clicked() {
                                log::info!("User confirmed settings reset");
                                // TODO: Implement actual reset functionality
                                self.show_reset_confirmation = false;
                                settings_changed = true;
                            }

                            if ui.button("取消").clicked() {
                                self.show_reset_confirmation = false;
                            }
                        });
                    });
                });
        }

        ui.add_space(15.0);

        if settings_changed {
            ui.group(|ui| {
                ui.vertical(|ui| {
                    ui.label(egui::RichText::new("提示").color(egui::Color32::YELLOW));
                    ui.label("设置已修改，请点击保存按钮应用更改。");
                });
            });
        }

        settings_changed
    }

    fn save_settings(&self) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }

    fn load_settings(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }

    fn reset_to_default(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }

    fn get_settings_summary(&self) -> String {
        "高级功能和开发者选项".to_string()
    }

    fn get_help_text(&self) -> Option<String> {
        Some("高级设置包含性能优化选项和开发者工具。请谨慎修改这些设置。".to_string())
    }
}
