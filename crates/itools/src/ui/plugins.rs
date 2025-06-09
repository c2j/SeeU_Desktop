use eframe::egui;
use crate::state::IToolsState;

/// Render the installed plugins view
pub fn render_installed_plugins(ui: &mut egui::Ui, state: &mut IToolsState) {
    ui.vertical(|ui| {
        // Header
        ui.horizontal(|ui| {
            ui.heading("已安装插件");

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("刷新").clicked() {
                    // TODO: Refresh plugin list
                }

                if ui.button("从文件安装").clicked() {
                    // TODO: Show file picker for local plugin installation
                }
            });
        });

        ui.separator();

        // Plugin list
        render_installed_plugin_list(ui, state);
    });
}

/// Render the list of installed plugins
fn render_installed_plugin_list(ui: &mut egui::Ui, state: &mut IToolsState) {
    let installed_plugins: Vec<_> = state.plugin_manager.get_installed_plugins()
        .into_iter()
        .map(|p| p.clone())
        .collect();

    if installed_plugins.is_empty() {
        ui.centered_and_justified(|ui| {
            ui.vertical_centered(|ui| {
                ui.label("还没有安装任何插件");
                ui.add_space(10.0);
                if ui.button("浏览插件市场").clicked() {
                    state.ui_state.current_view = crate::state::IToolsView::PluginMarket;
                }
            });
        });
        return;
    }

    egui::ScrollArea::vertical()
        .auto_shrink([false; 2])
        .show(ui, |ui| {
            for plugin in &installed_plugins {
                render_installed_plugin_card(ui, plugin, state);
                ui.add_space(10.0);
            }
        });
}

/// Render an installed plugin card
fn render_installed_plugin_card(ui: &mut egui::Ui, plugin: &crate::plugins::Plugin, state: &mut IToolsState) {
    egui::Frame::NONE
        .fill(ui.style().visuals.faint_bg_color)
        .stroke(egui::Stroke::new(1.0, ui.style().visuals.widgets.noninteractive.bg_stroke.color))
        .corner_radius(egui::Rounding::same(5))
        .inner_margin(egui::Margin::same(15))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                // Plugin icon and status indicator
                ui.vertical(|ui| {
                    if let Some(icon) = &plugin.metadata.icon {
                        ui.label(egui::RichText::new(icon).size(32.0));
                    } else {
                        ui.label(egui::RichText::new("📦").size(32.0));
                    }

                    // Status indicator
                    let (status_color, status_icon) = match plugin.status {
                        crate::plugins::PluginStatus::Enabled => (egui::Color32::from_rgb(100, 255, 100), "●"),
                        crate::plugins::PluginStatus::Disabled => (egui::Color32::from_rgb(150, 150, 150), "●"),
                        crate::plugins::PluginStatus::Error(_) => (egui::Color32::from_rgb(255, 100, 100), "●"),
                        _ => (egui::Color32::from_rgb(255, 200, 100), "●"),
                    };

                    ui.colored_label(status_color, status_icon);
                });

                ui.vertical(|ui| {
                    // Plugin name and version
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new(&plugin.metadata.display_name).strong().size(16.0));
                        ui.label(egui::RichText::new(&format!("v{}", plugin.metadata.version)).weak());
                    });

                    // Description
                    ui.label(&plugin.metadata.description);

                    // Status and metadata
                    ui.horizontal(|ui| {
                        ui.label(format!("状态: {}", plugin.get_status_text()));
                        ui.separator();
                        ui.label(format!("作者: {}", plugin.metadata.author));

                        if let Some(installed_at) = plugin.installed_at {
                            ui.separator();
                            ui.label(format!("安装时间: {}", installed_at.format("%Y-%m-%d")));
                        }
                    });

                    // Capabilities
                    ui.horizontal(|ui| {
                        ui.label("功能:");

                        if plugin.manifest.capabilities.provides_resources {
                            ui.small_button("资源");
                        }
                        if plugin.manifest.capabilities.provides_tools {
                            ui.small_button("工具");
                        }
                        if plugin.manifest.capabilities.provides_prompts {
                            ui.small_button("提示");
                        }
                        if plugin.manifest.capabilities.supports_sampling {
                            ui.small_button("采样");
                        }
                    });

                    // Permissions
                    if !plugin.manifest.permissions.is_empty() {
                        ui.horizontal(|ui| {
                            ui.label("权限:");
                            for permission in plugin.manifest.permissions.iter().take(3) {
                                let permission_color = match permission.level {
                                    crate::state::PermissionLevel::Low => egui::Color32::from_rgb(100, 255, 100),
                                    crate::state::PermissionLevel::Medium => egui::Color32::from_rgb(255, 200, 100),
                                    crate::state::PermissionLevel::High => egui::Color32::from_rgb(255, 150, 100),
                                    crate::state::PermissionLevel::Critical => egui::Color32::from_rgb(255, 100, 100),
                                };

                                ui.colored_label(permission_color, format!("{:?}", permission.permission_type));
                            }

                            if plugin.manifest.permissions.len() > 3 {
                                ui.label(format!("... +{} 更多", plugin.manifest.permissions.len() - 3));
                            }
                        });
                    }
                });

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // Action buttons
                    ui.vertical(|ui| {
                        match plugin.status {
                            crate::plugins::PluginStatus::Enabled => {
                                if ui.button("禁用").clicked() {
                                    if let Err(e) = state.plugin_manager.disable_plugin(plugin.id) {
                                        log::error!("Failed to disable plugin: {}", e);
                                    }
                                }
                            }
                            crate::plugins::PluginStatus::Disabled | crate::plugins::PluginStatus::Installed => {
                                if ui.button("启用").clicked() {
                                    if let Err(e) = state.plugin_manager.enable_plugin(plugin.id, &state.current_role) {
                                        log::error!("Failed to enable plugin: {}", e);
                                    }
                                }
                            }
                            crate::plugins::PluginStatus::Installing => {
                                ui.add(egui::Spinner::new());
                            }
                            crate::plugins::PluginStatus::Updating => {
                                ui.add(egui::Spinner::new());
                            }
                            crate::plugins::PluginStatus::Uninstalling => {
                                ui.add(egui::Spinner::new());
                            }
                            _ => {}
                        }

                        if plugin.can_be_uninstalled() {
                            if ui.small_button("卸载").clicked() {
                                if let Err(e) = state.plugin_manager.uninstall_plugin(plugin.id) {
                                    log::error!("Failed to uninstall plugin: {}", e);
                                }
                            }
                        }

                        if ui.small_button("详情").clicked() {
                            state.ui_state.selected_plugin = Some(plugin.id);
                            // TODO: Show plugin details dialog
                        }

                        if ui.small_button("设置").clicked() {
                            // TODO: Show plugin settings dialog
                        }
                    });
                });
            });

            // Show operation progress if any
            if let Some(operation) = state.plugin_manager.get_operation_status(&plugin.id) {
                ui.separator();
                ui.horizontal(|ui| {
                    ui.add(egui::ProgressBar::new(operation.progress).text(&operation.status_message));
                });
            }
        });
}
