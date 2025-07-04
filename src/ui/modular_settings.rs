use eframe::egui;
use crate::app::SeeUApp;
use super::settings_trait::{SettingsRegistry, SettingsCategory, SettingsModule};
use super::settings_base::{AppSettingsModule, AppearanceSettingsModule, AdvancedSettingsModule};

/// Get all available settings categories
pub fn get_all_settings_categories() -> Vec<SettingsCategory> {
    vec![
        SettingsCategory::new("app", "应用设置", "🔧", "应用程序的基本设置，包括启动、数据管理等"),
        SettingsCategory::new("appearance", "外观设置", "🎨", "主题、字体、界面缩放等设置"),
        SettingsCategory::new("advanced", "高级设置", "⚙️", "性能、调试、开发者选项等高级设置"),
        SettingsCategory::new("notes", "笔记设置", "📝", "笔记编辑、显示、导入导出等相关设置"),
        SettingsCategory::new("file_editor", "文件编辑器", "📄", "文件编辑器外观、行为、语法高亮等设置"),
        SettingsCategory::new("ai_assistant", "AI助手设置", "🤖", "AI模型配置、API设置、对话管理等"),
        SettingsCategory::new("search", "搜索设置", "🔍", "文件索引、搜索选项、目录管理等设置"),
        SettingsCategory::new("terminal", "终端设置", "💻", "终端外观、行为、颜色等配置"),
        SettingsCategory::new("tools", "工具设置", "🔧", "开发工具、实用程序、快捷操作等设置"),
    ]
}

/// Create settings registry with base modules only
/// Crate-specific settings are handled directly in render_modular_settings
pub fn create_settings_registry(_app: &mut SeeUApp) -> SettingsRegistry {
    let mut registry = SettingsRegistry::new();

    // Register base application modules
    registry.register_module(Box::new(AppSettingsModule::new()));
    registry.register_module(Box::new(AppearanceSettingsModule::new()));
    registry.register_module(Box::new(AdvancedSettingsModule::new()));

    // Register file editor settings module
    registry.register_module(Box::new(FileEditorSettingsAdapter::new()));

    registry
}

/// Render the modular settings interface
pub fn render_modular_settings(ui: &mut egui::Ui, app: &mut SeeUApp) {
    ui.heading("⚙️ 设置");
    ui.add_space(10.0);

    // Initialize settings registry if not already done
    if app.modular_settings_state.settings_registry.is_none() {
        app.modular_settings_state.settings_registry = Some(create_settings_registry(app));
        app.modular_settings_state.selected_category = "app".to_string();
    }

    // Create a horizontal layout with sidebar and content
    ui.horizontal(|ui| {
        // Sidebar for categories
        ui.vertical(|ui| {
            ui.set_min_width(180.0);
            ui.set_max_width(180.0);

            ui.label(egui::RichText::new("设置分类").strong());
            ui.separator();
            ui.add_space(5.0);

            let all_categories = get_all_settings_categories();

            for category in all_categories {
                let is_selected = app.modular_settings_state.selected_category == category.id;

                let response = ui.selectable_label(
                    is_selected,
                    format!("{} {}", category.icon, category.display_name)
                );

                if response.clicked() {
                    app.modular_settings_state.selected_category = category.id.clone();
                }

                // Show help text on hover
                if !category.description.is_empty() {
                    response.on_hover_text(&category.description);
                }
            }
        });

        ui.separator();

        // Content area
        ui.vertical(|ui| {
            ui.set_min_width(400.0);

            // Render current category settings
            let mut settings_changed = false;

            match app.modular_settings_state.selected_category.as_str() {
                "app" | "appearance" | "advanced" => {
                    // Use registry for base modules
                    if let Some(registry) = &mut app.modular_settings_state.settings_registry {
                        registry.set_current_category(app.modular_settings_state.selected_category.clone());
                        settings_changed = registry.render_current_settings(ui);
                    }
                }
                "notes" => {
                    // Render iNote settings directly
                    settings_changed = render_inote_settings(ui, &mut app.inote_state);
                }
                "file_editor" => {
                    // Render file editor settings directly
                    settings_changed = render_file_editor_settings(ui, &mut app.ifile_editor_state);
                }
                "ai_assistant" => {
                    // Render AI Assistant settings directly
                    settings_changed = render_ai_assistant_settings(ui, &mut app.ai_assist_state);
                }
                "search" => {
                    // Render iSearch settings directly
                    settings_changed = render_isearch_settings(ui, &mut app.isearch_state);
                }
                "terminal" => {
                    // Render iTerminal settings directly
                    settings_changed = render_iterminal_settings(ui, &mut app.iterminal_state);
                }
                "tools" => {
                    // Render iTools settings directly
                    settings_changed = render_itools_settings(ui, &mut app.itools_state);
                }
                _ => {
                    ui.label("未知的设置类别");
                }
            }

            // Update state if settings changed
            if settings_changed {
                app.modular_settings_state.has_unsaved_changes = true;
                app.modular_settings_state.last_save_status = "有未保存的更改".to_string();
            }

            ui.add_space(20.0);

            // Action buttons
            ui.horizontal(|ui| {
                // Save button
                let save_enabled = app.modular_settings_state.has_unsaved_changes;
                if ui.add_enabled(save_enabled, egui::Button::new("💾 保存设置")).clicked() {
                    let mut save_success = true;
                    let mut error_count = 0;

                    // Save base module settings
                    if let Some(registry) = &app.modular_settings_state.settings_registry {
                        if let Err(errors) = registry.save_all_settings() {
                            save_success = false;
                            error_count += errors.len();
                            for error in errors {
                                log::error!("Base settings save error: {}", error);
                            }
                        }
                    }

                    // Save crate-specific settings
                    if let Err(err) = inote::save_settings(&app.inote_state) {
                        save_success = false;
                        error_count += 1;
                        log::error!("iNote settings save error: {}", err);
                    }

                    // Save file editor settings
                    app.ifile_editor_state.settings_manager.save_settings();

                    if let Err(err) = aiAssist::save_settings(&app.ai_assist_state) {
                        save_success = false;
                        error_count += 1;
                        log::error!("AI Assistant settings save error: {}", err);
                    }

                    if let Err(err) = isearch::save_settings(&app.isearch_state) {
                        save_success = false;
                        error_count += 1;
                        log::error!("iSearch settings save error: {}", err);
                    }

                    if let Err(err) = app.iterminal_state.save_config() {
                        save_success = false;
                        error_count += 1;
                        log::error!("iTerminal settings save error: {}", err);
                    }

                    if let Err(err) = itools::save_settings(&app.itools_state) {
                        save_success = false;
                        error_count += 1;
                        log::error!("iTools settings save error: {}", err);
                    }

                    // Save application-level settings
                    if let Err(err) = app.save_app_settings() {
                        save_success = false;
                        error_count += 1;
                        log::error!("App settings save error: {}", err);
                    }

                    // Update status
                    if save_success {
                        app.modular_settings_state.has_unsaved_changes = false;
                        app.modular_settings_state.last_save_status = "保存成功".to_string();
                        log::info!("All settings saved successfully");
                    } else {
                        app.modular_settings_state.last_save_status = format!("保存失败: {} 个错误", error_count);
                    }
                }

                // Reset button
                if ui.button("🔄 重置为默认").clicked() {
                    app.modular_settings_state.show_reset_confirmation = true;
                }

                // Validate button
                if ui.button("✅ 验证设置").clicked() {
                    if let Some(registry) = &app.modular_settings_state.settings_registry {
                        match registry.validate_all_settings() {
                            Ok(()) => {
                                app.modular_settings_state.validation_status = "验证通过".to_string();
                                app.modular_settings_state.validation_errors.clear();
                            }
                            Err(errors) => {
                                app.modular_settings_state.validation_status = format!("验证失败: {} 个错误", errors.len());
                                app.modular_settings_state.validation_errors = errors;
                            }
                        }
                    }
                }
            });

            // Status display
            ui.add_space(10.0);
            ui.horizontal(|ui| {
                ui.label("状态:");
                ui.label(&app.modular_settings_state.last_save_status);
                ui.separator();
                ui.label("验证:");
                ui.label(&app.modular_settings_state.validation_status);
            });

            // Show validation errors if any
            if !app.modular_settings_state.validation_errors.is_empty() {
                ui.add_space(5.0);
                ui.group(|ui| {
                    ui.vertical(|ui| {
                        ui.label(egui::RichText::new("验证错误:").color(egui::Color32::RED));
                        for error in &app.modular_settings_state.validation_errors {
                            ui.label(egui::RichText::new(format!("• {}", error)).color(egui::Color32::RED));
                        }
                    });
                });
            }
        });
    });

    // Reset confirmation dialog
    if app.modular_settings_state.show_reset_confirmation {
        egui::Window::new("确认重置")
            .collapsible(false)
            .resizable(false)
            .show(ui.ctx(), |ui| {
                ui.label("确定要将所有设置重置为默认值吗？");
                ui.label(egui::RichText::new("此操作不可撤销！").color(egui::Color32::RED));
                
                ui.add_space(10.0);
                ui.horizontal(|ui| {
                    if ui.button("确认重置").clicked() {
                        if let Some(registry) = &mut app.modular_settings_state.settings_registry {
                            match registry.reset_all_to_default() {
                                Ok(()) => {
                                    app.modular_settings_state.has_unsaved_changes = true;
                                    app.modular_settings_state.last_save_status = "已重置，需要保存".to_string();
                                    log::info!("All settings reset to default");
                                }
                                Err(errors) => {
                                    app.modular_settings_state.last_save_status = format!("重置失败: {} 个错误", errors.len());
                                    for error in errors {
                                        log::error!("Settings reset error: {}", error);
                                    }
                                }
                            }
                        }
                        app.modular_settings_state.show_reset_confirmation = false;
                    }
                    
                    if ui.button("取消").clicked() {
                        app.modular_settings_state.show_reset_confirmation = false;
                    }
                });
            });
    }
}

/// Render iNote settings
fn render_inote_settings(ui: &mut egui::Ui, state: &mut inote::db_state::DbINoteState) -> bool {
    use inote::settings_ui::SettingsModule;
    let mut module = inote::create_settings_module(state);
    module.render_settings(ui)
}

/// Render file editor settings
fn render_file_editor_settings(ui: &mut egui::Ui, state: &mut ifile_editor::IFileEditorState) -> bool {
    state.settings_manager.render_settings(ui)
}

/// Render AI Assistant settings
fn render_ai_assistant_settings(ui: &mut egui::Ui, state: &mut aiAssist::state::AIAssistState) -> bool {
    use aiAssist::settings_ui::SettingsModule;
    let mut module = aiAssist::create_settings_module(state);
    module.render_settings(ui)
}

/// Render iSearch settings
fn render_isearch_settings(ui: &mut egui::Ui, state: &mut isearch::ISearchState) -> bool {
    use isearch::settings_ui::SettingsModule;
    let mut module = isearch::create_settings_module(state);
    module.render_settings(ui)
}

/// Render iTerminal settings
fn render_iterminal_settings(ui: &mut egui::Ui, state: &mut iterminal::ITerminalState) -> bool {
    use iterminal::settings_ui::SettingsModule;
    let mut module = iterminal::create_settings_module(state);
    module.render_settings(ui)
}

/// Render iTools settings
fn render_itools_settings(ui: &mut egui::Ui, state: &mut itools::IToolsState) -> bool {
    use itools::settings_ui::SettingsModule;
    let mut module = itools::create_settings_module(state);
    module.render_settings(ui)
}

/// 文件编辑器设置适配器
pub struct FileEditorSettingsAdapter {
    module: ifile_editor::FileEditorSettingsModule,
}

impl FileEditorSettingsAdapter {
    pub fn new() -> Self {
        Self {
            module: ifile_editor::FileEditorSettingsModule::new(),
        }
    }
}

impl SettingsModule for FileEditorSettingsAdapter {
    fn get_category(&self) -> SettingsCategory {
        SettingsCategory {
            id: "file_editor".to_string(),
            display_name: "文件编辑器".to_string(),
            icon: "📄".to_string(),
            description: "文件编辑器相关设置".to_string(),
        }
    }

    fn render_settings(&mut self, ui: &mut egui::Ui) -> bool {
        self.module.render_settings(ui)
    }

    fn save_settings(&self) -> Result<(), Box<dyn std::error::Error>> {
        // TODO: 实现设置保存
        Ok(())
    }

    fn load_settings(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // TODO: 实现设置加载
        Ok(())
    }

    fn reset_to_default(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.module = ifile_editor::FileEditorSettingsModule::new();
        Ok(())
    }

    fn get_settings_summary(&self) -> String {
        let settings = self.module.get_settings();
        format!("字体: {} {}px, 主题: {}",
            settings.font_family,
            settings.font_size,
            settings.theme
        )
    }
}
