use eframe::egui;
use crate::app::SeeUApp;
use super::settings_trait::{SettingsRegistry, ModularSettingsState};
use super::settings_base::{AppSettingsModule, AppearanceSettingsModule, AdvancedSettingsModule};

/// Create settings registry with all modules
pub fn create_settings_registry(_app: &mut SeeUApp) -> SettingsRegistry {
    let mut registry = SettingsRegistry::new();

    // Register base application modules
    registry.register_module(Box::new(AppSettingsModule::new()));
    registry.register_module(Box::new(AppearanceSettingsModule::new()));
    registry.register_module(Box::new(AdvancedSettingsModule::new()));

    // TODO: Register crate-specific modules
    // Note: These need lifetime management and proper trait implementation
    // For now, we'll use the base modules until we resolve the lifetime issues

    // registry.register_module(Box::new(inote::create_settings_module(&mut app.inote_state)));
    // registry.register_module(Box::new(aiAssist::create_settings_module(&mut app.ai_assist_state)));
    // registry.register_module(Box::new(isearch::create_settings_module(&mut app.isearch_state)));
    // registry.register_module(Box::new(iterminal::create_settings_module(&mut app.iterminal_state)));
    // registry.register_module(Box::new(itools::create_settings_module(&mut app.itools_state)));

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

            if let Some(registry) = &app.modular_settings_state.settings_registry {
                let categories = registry.get_categories();
                
                for category in categories {
                    let is_selected = app.modular_settings_state.selected_category == category.id;
                    
                    let response = ui.selectable_label(
                        is_selected,
                        format!("{} {}", category.icon, category.display_name)
                    );
                    
                    if response.clicked() {
                        app.modular_settings_state.selected_category = category.id.clone();
                        if let Some(registry) = &mut app.modular_settings_state.settings_registry {
                            registry.set_current_category(category.id);
                        }
                    }
                    
                    // Show help text on hover
                    if !category.description.is_empty() {
                        response.on_hover_text(&category.description);
                    }
                }
            }
        });

        ui.separator();

        // Content area
        ui.vertical(|ui| {
            ui.set_min_width(400.0);

            // Render current category settings
            let mut settings_changed = false;
            if let Some(registry) = &mut app.modular_settings_state.settings_registry {
                settings_changed = registry.render_current_settings(ui);
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
                    if let Some(registry) = &app.modular_settings_state.settings_registry {
                        match registry.save_all_settings() {
                            Ok(()) => {
                                app.modular_settings_state.has_unsaved_changes = false;
                                app.modular_settings_state.last_save_status = "保存成功".to_string();
                                log::info!("All settings saved successfully");
                            }
                            Err(errors) => {
                                app.modular_settings_state.last_save_status = format!("保存失败: {} 个错误", errors.len());
                                for error in errors {
                                    log::error!("Settings save error: {}", error);
                                }
                            }
                        }
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
