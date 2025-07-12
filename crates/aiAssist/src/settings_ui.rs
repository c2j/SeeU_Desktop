use eframe::egui;
use crate::state::AIAssistState;

/// Settings category information for aiAssist
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

/// AI Assistant settings module
pub struct AIAssistSettingsModule<'a> {
    pub state: &'a mut AIAssistState,
    pub category: SettingsCategory,
}

impl<'a> AIAssistSettingsModule<'a> {
    pub fn new(state: &'a mut AIAssistState) -> Self {
        Self {
            state,
            category: SettingsCategory::new(
                "ai_assistant",
                "AI助手设置",
                "🤖",
                "AI模型配置、API设置、对话管理等"
            ),
        }
    }

    /// Test API connection
    fn test_api_connection(&mut self) {
        let base_url = &self.state.ai_settings.base_url;
        let api_key = &self.state.ai_settings.api_key;
        let model = &self.state.ai_settings.model;

        if base_url.is_empty() {
            log::warn!("Base URL is empty, cannot test connection");
            return;
        }

        log::info!("Testing connection to: {} with model: {}", base_url, model);

        // Simple connection test - try to reach the endpoint
        // In a real implementation, you might want to make an actual API call
        match std::process::Command::new("curl")
            .arg("-s")
            .arg("-o")
            .arg("/dev/null")
            .arg("-w")
            .arg("%{http_code}")
            .arg("--connect-timeout")
            .arg("5")
            .arg(base_url)
            .output() {
            Ok(output) => {
                let status_code = String::from_utf8_lossy(&output.stdout);
                if status_code.trim() == "200" || status_code.trim() == "404" {
                    log::info!("✅ Connection test successful (HTTP {})", status_code.trim());
                } else {
                    log::warn!("⚠️ Connection test returned HTTP {}", status_code.trim());
                }
            }
            Err(err) => {
                log::error!("❌ Connection test failed: {}", err);
            }
        }
    }
}

impl<'a> SettingsModule for AIAssistSettingsModule<'a> {
    fn get_category(&self) -> SettingsCategory {
        self.category.clone()
    }

    fn render_settings(&mut self, ui: &mut egui::Ui) -> bool {
        let mut settings_changed = false;

        ui.heading("🤖 AI助手设置");
        ui.add_space(10.0);

        // API Configuration
        ui.group(|ui| {
            ui.vertical(|ui| {
                ui.label(egui::RichText::new("API 配置").strong());
                ui.add_space(5.0);

                ui.horizontal(|ui| {
                    ui.label("Base URL:");
                    if ui.text_edit_singleline(&mut self.state.ai_settings.base_url).changed() {
                        settings_changed = true;
                    }
                });

                ui.horizontal(|ui| {
                    ui.label("API Key:");

                    // 获取用于显示的API Key
                    let mut display_key = self.state.get_display_api_key();
                    let response = ui.text_edit_singleline(&mut display_key);

                    // 如果用户修改了显示的内容，更新实际的API Key
                    if response.changed() {
                        // 如果当前是掩码模式且用户修改了内容，切换到非掩码模式
                        if self.state.show_api_key_masked {
                            self.state.show_api_key_masked = false;
                        }
                        self.state.ai_settings.api_key = display_key;
                        settings_changed = true;
                    }

                    // 添加显示/隐藏按钮
                    let button_text = if self.state.show_api_key_masked { "👁" } else { "🙈" };
                    let button_tooltip = if self.state.show_api_key_masked { "显示完整API Key" } else { "隐藏API Key" };

                    if ui.button(button_text).on_hover_text(button_tooltip).clicked() {
                        self.state.show_api_key_masked = !self.state.show_api_key_masked;
                    }
                });

                ui.horizontal(|ui| {
                    ui.label("模型名称:");
                    if ui.text_edit_singleline(&mut self.state.ai_settings.model).changed() {
                        settings_changed = true;
                    }
                });
            });
        });

        ui.add_space(15.0);

        // Model Parameters
        ui.group(|ui| {
            ui.vertical(|ui| {
                ui.label(egui::RichText::new("模型参数").strong());
                ui.add_space(5.0);

                ui.horizontal(|ui| {
                    ui.label("Temperature:");
                    if ui.add(egui::Slider::new(&mut self.state.ai_settings.temperature, 0.0..=2.0)
                        .step_by(0.1)
                        .text("创造性")).changed() {
                        settings_changed = true;
                    }
                });

                ui.horizontal(|ui| {
                    ui.label("Max Tokens:");
                    if ui.add(egui::Slider::new(&mut self.state.ai_settings.max_tokens, 100..=8000)
                        .step_by(100.0)
                        .text("最大长度")).changed() {
                        settings_changed = true;
                    }
                });

                if ui.checkbox(&mut self.state.ai_settings.streaming, "启用流式输出").changed() {
                    settings_changed = true;
                }
            });
        });

        ui.add_space(15.0);

        // Chat Settings
        ui.group(|ui| {
            ui.vertical(|ui| {
                ui.label(egui::RichText::new("对话设置").strong());
                ui.add_space(5.0);

                ui.horizontal(|ui| {
                    ui.label("当前会话数:");
                    ui.label(format!("{}", self.state.chat_sessions.len()));
                });

                ui.horizontal(|ui| {
                    ui.label("活跃会话:");
                    if self.state.active_session_idx < self.state.chat_sessions.len() {
                        let session = &self.state.chat_sessions[self.state.active_session_idx];
                        ui.label(format!("会话 {} ({} 条消息)",
                            self.state.active_session_idx + 1,
                            session.messages.len()));
                    } else {
                        ui.label("无");
                    }
                });

                ui.add_space(10.0);

                ui.horizontal(|ui| {
                    if ui.button("清除所有对话历史").clicked() {
                        self.state.chat_sessions.clear();
                        self.state.create_new_session();
                        self.state.active_session_idx = 0;
                        settings_changed = true;
                    }

                    if ui.button("重置为默认设置").clicked() {
                        self.state.ai_settings = Default::default();
                        settings_changed = true;
                    }
                });
            });
        });

        ui.add_space(15.0);

        // Advanced Settings
        ui.group(|ui| {
            ui.vertical(|ui| {
                ui.label(egui::RichText::new("高级设置").strong());
                ui.add_space(5.0);

                ui.horizontal(|ui| {
                    ui.label("连接超时:");
                    ui.label("30秒"); // TODO: Make this configurable
                });

                ui.horizontal(|ui| {
                    ui.label("重试次数:");
                    ui.label("3次"); // TODO: Make this configurable
                });

                ui.add_space(5.0);

                if ui.button("🔗 测试连接").clicked() {
                    log::info!("Testing AI API connection");
                    self.test_api_connection();
                }
            });
        });

        settings_changed
    }

    fn save_settings(&self) -> Result<(), Box<dyn std::error::Error>> {
        crate::save_settings(self.state)
    }

    fn load_settings(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        crate::load_settings(self.state)
    }

    fn reset_to_default(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Reset AI settings to default
        self.state.ai_settings = Default::default();
        self.state.show_api_key_masked = true;
        
        // Clear chat sessions and create a new one
        self.state.chat_sessions.clear();
        self.state.create_new_session();
        self.state.active_session_idx = 0;
        
        // Save the reset settings
        self.save_settings()?;
        Ok(())
    }

    fn get_settings_summary(&self) -> String {
        let model = if self.state.ai_settings.model.is_empty() {
            "未设置"
        } else {
            &self.state.ai_settings.model
        };
        
        let api_status = if self.state.ai_settings.api_key.is_empty() {
            "未配置"
        } else {
            "已配置"
        };

        format!("模型: {}, API: {}, 会话: {}", 
            model, api_status, self.state.chat_sessions.len())
    }

    fn validate_settings(&self) -> Result<(), String> {
        if self.state.ai_settings.base_url.is_empty() {
            return Err("Base URL 不能为空".to_string());
        }

        if self.state.ai_settings.api_key.is_empty() {
            return Err("API Key 不能为空".to_string());
        }

        if self.state.ai_settings.model.is_empty() {
            return Err("模型名称不能为空".to_string());
        }

        if self.state.ai_settings.temperature < 0.0 || self.state.ai_settings.temperature > 2.0 {
            return Err("Temperature 必须在 0.0 到 2.0 之间".to_string());
        }

        if self.state.ai_settings.max_tokens < 1 || self.state.ai_settings.max_tokens > 32000 {
            return Err("Max Tokens 必须在 1 到 32000 之间".to_string());
        }

        Ok(())
    }

    fn get_help_text(&self) -> Option<String> {
        Some("配置AI助手的API连接和模型参数。确保API Key正确且有足够的配额。Temperature控制回答的创造性，Max Tokens控制回答的最大长度。".to_string())
    }
}
