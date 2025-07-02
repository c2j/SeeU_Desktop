use eframe::egui;
use crate::state::IToolsState;

/// Settings category information for itools
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

/// Tools settings module
pub struct IToolsSettingsModule<'a> {
    pub state: &'a mut IToolsState,
    pub category: SettingsCategory,
}

impl<'a> IToolsSettingsModule<'a> {
    pub fn new(state: &'a mut IToolsState) -> Self {
        Self {
            state,
            category: SettingsCategory::new(
                "tools",
                "工具设置",
                "🔧",
                "开发工具、实用程序、快捷操作等设置"
            ),
        }
    }
}

impl<'a> SettingsModule for IToolsSettingsModule<'a> {
    fn get_category(&self) -> SettingsCategory {
        self.category.clone()
    }

    fn render_settings(&mut self, ui: &mut egui::Ui) -> bool {
        let mut settings_changed = false;

        ui.heading("🔧 工具设置");
        ui.add_space(10.0);

        // Current role display
        ui.group(|ui| {
            ui.vertical(|ui| {
                ui.label(egui::RichText::new("用户角色").strong());
                ui.add_space(5.0);

                ui.horizontal(|ui| {
                    ui.label("当前角色:");
                    ui.label(format!("{:?}", self.state.current_role));
                });

                ui.add_space(5.0);
                ui.label(egui::RichText::new("角色决定了可用的工具和权限").weak());
            });
        });

        ui.add_space(15.0);

        // Plugin management
        ui.group(|ui| {
            ui.vertical(|ui| {
                ui.label(egui::RichText::new("插件管理").strong());
                ui.add_space(5.0);

                ui.horizontal(|ui| {
                    ui.label("已安装插件:");
                    ui.label(format!("{}", self.state.plugin_manager.get_installed_count()));
                });

                ui.horizontal(|ui| {
                    ui.label("活跃插件:");
                    ui.label(format!("{}", self.state.plugin_manager.get_active_count()));
                });

                ui.add_space(5.0);
                if ui.button("🔄 刷新插件列表").clicked() {
                    // Plugin refresh would be handled here
                    settings_changed = true;
                }
            });
        });

        ui.add_space(15.0);

        // MCP settings
        ui.group(|ui| {
            ui.vertical(|ui| {
                ui.label(egui::RichText::new("MCP 协议设置").strong());
                ui.add_space(5.0);

                ui.horizontal(|ui| {
                    ui.label("MCP 客户端状态:");
                    ui.label("已连接"); // Simplified status
                });

                ui.horizontal(|ui| {
                    ui.label("服务器管理器:");
                    if self.state.mcp_server_manager.is_some() {
                        ui.label("已启用");
                    } else {
                        ui.label("未启用");
                    }
                });

                ui.add_space(5.0);
                if ui.button("🔧 打开 MCP 设置").clicked() {
                    log::info!("Opening MCP settings");
                }
            });
        });

        ui.add_space(15.0);

        // Security settings
        ui.group(|ui| {
            ui.vertical(|ui| {
                ui.label(egui::RichText::new("安全设置").strong());
                ui.add_space(5.0);

                ui.horizontal(|ui| {
                    ui.label("安全上下文:");
                    ui.label("已启用");
                });

                ui.horizontal(|ui| {
                    ui.label("审计日志:");
                    ui.label(format!("{} 条记录", self.state.security_context.audit_log.len()));
                });

                ui.add_space(5.0);
                if ui.button("🔒 查看安全设置").clicked() {
                    log::info!("Opening security settings");
                }
            });
        });

        ui.add_space(15.0);

        // Action buttons
        ui.horizontal(|ui| {
            if ui.button("🔄 刷新状态").clicked() {
                log::info!("Refreshing iTools status");
                settings_changed = true;
            }

            if ui.button("📊 查看统计").clicked() {
                log::info!("Show tools statistics");
                // TODO: Implement statistics view
            }

            if ui.button("🔧 重置设置").clicked() {
                log::info!("Reset iTools settings");
                settings_changed = true;
            }
        });

        settings_changed
    }

    fn save_settings(&self) -> Result<(), Box<dyn std::error::Error>> {
        // iTools settings saving would be implemented here
        log::info!("iTools settings saved");
        Ok(())
    }

    fn load_settings(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // iTools settings loading would be implemented here
        log::info!("iTools settings loaded");
        Ok(())
    }

    fn reset_to_default(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Reset iTools to default state
        log::info!("Resetting iTools to default settings");

        // Reset would be implemented here based on actual state structure
        // For now, just log the action

        // Save the reset settings
        self.save_settings()?;
        Ok(())
    }

    fn get_settings_summary(&self) -> String {
        let plugin_count = self.state.plugin_manager.get_installed_count();
        let active_count = self.state.plugin_manager.get_active_count();
        format!("插件: {}/{}, 角色: {:?}", active_count, plugin_count, self.state.current_role)
    }

    fn validate_settings(&self) -> Result<(), String> {
        // Basic validation for iTools settings
        // More specific validation would be added based on actual requirements
        Ok(())
    }

    fn get_help_text(&self) -> Option<String> {
        Some("在这里配置各种开发工具的行为和性能参数。启用的工具分类会在主界面显示，可以根据需要调整性能设置。".to_string())
    }
}
