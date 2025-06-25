use eframe::egui;
use uuid::Uuid;
use crate::state::IToolsState;
use crate::plugins::marketplace::MarketplacePlugin;

/// Common UI components for iTools

/// Render a status badge
pub fn status_badge(ui: &mut egui::Ui, text: &str, color: egui::Color32) {
    ui.colored_label(color, format!("● {}", text));
}

/// Render a permission level indicator
pub fn permission_level_indicator(ui: &mut egui::Ui, level: &crate::state::PermissionLevel) {
    let (text, color) = match level {
        crate::state::PermissionLevel::Low => ("低", egui::Color32::from_rgb(100, 255, 100)),
        crate::state::PermissionLevel::Medium => ("中", egui::Color32::from_rgb(255, 200, 100)),
        crate::state::PermissionLevel::High => ("高", egui::Color32::from_rgb(255, 150, 100)),
        crate::state::PermissionLevel::Critical => ("严重", egui::Color32::from_rgb(255, 100, 100)),
    };
    
    ui.colored_label(color, text);
}

/// Render a plugin status indicator
pub fn plugin_status_indicator(ui: &mut egui::Ui, status: &crate::plugins::PluginStatus) {
    let (text, color) = match status {
        crate::plugins::PluginStatus::NotInstalled => ("未安装", egui::Color32::GRAY),
        crate::plugins::PluginStatus::Installing => ("安装中", egui::Color32::from_rgb(255, 200, 100)),
        crate::plugins::PluginStatus::Installed => ("已安装", egui::Color32::from_rgb(150, 150, 255)),
        crate::plugins::PluginStatus::Enabled => ("已启用", egui::Color32::from_rgb(100, 255, 100)),
        crate::plugins::PluginStatus::Disabled => ("已禁用", egui::Color32::GRAY),
        crate::plugins::PluginStatus::Updating => ("更新中", egui::Color32::from_rgb(255, 200, 100)),
        crate::plugins::PluginStatus::Error(_) => ("错误", egui::Color32::from_rgb(255, 100, 100)),
        crate::plugins::PluginStatus::Uninstalling => ("卸载中", egui::Color32::from_rgb(255, 200, 100)),
    };
    
    status_badge(ui, text, color);
}

/// Render a confirmation dialog
pub fn confirmation_dialog(
    ui: &mut egui::Ui,
    title: &str,
    message: &str,
    on_confirm: impl FnOnce(),
    on_cancel: impl FnOnce(),
) {
    egui::Window::new(title)
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .show(ui.ctx(), |ui| {
            ui.label(message);
            
            ui.add_space(10.0);
            
            ui.horizontal(|ui| {
                if ui.button("确认").clicked() {
                    on_confirm();
                }
                
                if ui.button("取消").clicked() {
                    on_cancel();
                }
            });
        });
}

/// Render a loading spinner with text
pub fn loading_spinner(ui: &mut egui::Ui, text: &str) {
    ui.horizontal(|ui| {
        ui.add(egui::Spinner::new());
        ui.label(text);
    });
}

/// Render a progress bar with text
pub fn progress_bar(ui: &mut egui::Ui, progress: f32, text: &str) {
    ui.add(egui::ProgressBar::new(progress).text(text));
}

/// Render an info box
pub fn info_box(ui: &mut egui::Ui, title: &str, content: &str, icon: &str) {
    egui::Frame::none()
        .fill(ui.style().visuals.faint_bg_color)
        .stroke(egui::Stroke::new(1.0, ui.style().visuals.widgets.noninteractive.bg_stroke.color))
        .rounding(egui::Rounding::same(5.0))
        .inner_margin(egui::Margin::same(10.0))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(icon).size(20.0));
                ui.vertical(|ui| {
                    ui.label(egui::RichText::new(title).strong());
                    ui.label(content);
                });
            });
        });
}

/// Render a warning box
pub fn warning_box(ui: &mut egui::Ui, title: &str, content: &str) {
    egui::Frame::none()
        .fill(egui::Color32::from_rgba_unmultiplied(255, 200, 100, 30))
        .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(255, 200, 100)))
        .rounding(egui::Rounding::same(5.0))
        .inner_margin(egui::Margin::same(10.0))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("⚠").size(20.0).color(egui::Color32::from_rgb(255, 200, 100)));
                ui.vertical(|ui| {
                    ui.colored_label(egui::Color32::from_rgb(255, 200, 100), egui::RichText::new(title).strong());
                    ui.label(content);
                });
            });
        });
}

/// Render an error box
pub fn error_box(ui: &mut egui::Ui, title: &str, content: &str) {
    egui::Frame::none()
        .fill(egui::Color32::from_rgba_unmultiplied(255, 100, 100, 30))
        .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(255, 100, 100)))
        .rounding(egui::Rounding::same(5.0))
        .inner_margin(egui::Margin::same(10.0))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("❌").size(20.0));
                ui.vertical(|ui| {
                    ui.colored_label(egui::Color32::from_rgb(255, 100, 100), egui::RichText::new(title).strong());
                    ui.label(content);
                });
            });
        });
}

/// Render a success box
pub fn success_box(ui: &mut egui::Ui, title: &str, content: &str) {
    egui::Frame::none()
        .fill(egui::Color32::from_rgba_unmultiplied(100, 255, 100, 30))
        .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(100, 255, 100)))
        .rounding(egui::Rounding::same(5.0))
        .inner_margin(egui::Margin::same(10.0))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("✅").size(20.0));
                ui.vertical(|ui| {
                    ui.colored_label(egui::Color32::from_rgb(100, 255, 100), egui::RichText::new(title).strong());
                    ui.label(content);
                });
            });
        });
}

/// Render a collapsible section
pub fn collapsible_section<R>(
    ui: &mut egui::Ui,
    title: &str,
    default_open: bool,
    add_contents: impl FnOnce(&mut egui::Ui) -> R,
) -> egui::CollapsingResponse<R> {
    egui::CollapsingHeader::new(title)
        .default_open(default_open)
        .show(ui, add_contents)
}

/// Render a card container
pub fn card<R>(
    ui: &mut egui::Ui,
    add_contents: impl FnOnce(&mut egui::Ui) -> R,
) -> egui::InnerResponse<R> {
    egui::Frame::none()
        .fill(ui.style().visuals.faint_bg_color)
        .stroke(egui::Stroke::new(1.0, ui.style().visuals.widgets.noninteractive.bg_stroke.color))
        .rounding(egui::Rounding::same(5.0))
        .inner_margin(egui::Margin::same(15.0))
        .show(ui, add_contents)
}

/// Render a metric display
pub fn metric_display(ui: &mut egui::Ui, label: &str, value: &str, icon: Option<&str>) {
    ui.horizontal(|ui| {
        if let Some(icon) = icon {
            ui.label(egui::RichText::new(icon).size(16.0));
        }
        ui.label(format!("{}: ", label));
        ui.label(egui::RichText::new(value).strong());
    });
}

/// Render a tag
pub fn tag(ui: &mut egui::Ui, text: &str, color: Option<egui::Color32>) {
    let bg_color = color.unwrap_or(ui.style().visuals.selection.bg_fill);

    egui::Frame::none()
        .fill(bg_color)
        .rounding(egui::Rounding::same(3.0))
        .inner_margin(egui::Margin::symmetric(6.0, 2.0))
        .show(ui, |ui| {
            ui.label(egui::RichText::new(text).size(12.0));
        });
}

/// Render plugin details dialog
pub fn render_plugin_details_dialog(ctx: &egui::Context, state: &mut IToolsState) {
    if let Some(plugin_id) = state.ui_state.selected_plugin {
        let mut show_dialog = true;

        egui::Window::new("插件详情")
            .open(&mut show_dialog)
            .default_width(600.0)
            .default_height(500.0)
            .resizable(true)
            .show(ctx, |ui| {
                // Try to find plugin in marketplace first
                if let Some(marketplace_plugin) = state.plugin_manager.get_marketplace().get_plugin(&plugin_id).cloned() {
                    render_marketplace_plugin_details(ui, &marketplace_plugin, state);
                } else if let Some(installed_plugin) = state.plugin_manager.get_plugin(&plugin_id).cloned() {
                    render_installed_plugin_details(ui, &installed_plugin, state);
                } else {
                    ui.label("插件未找到");
                }
            });

        if !show_dialog {
            state.ui_state.selected_plugin = None;
        }
    }
}

/// Render marketplace plugin details
fn render_marketplace_plugin_details(ui: &mut egui::Ui, plugin: &MarketplacePlugin, state: &mut IToolsState) {
    egui::ScrollArea::vertical().show(ui, |ui| {
        // Header with icon and basic info
        ui.horizontal(|ui| {
            // Plugin icon placeholder
            ui.add_sized([64.0, 64.0], egui::Button::new("📦").sense(egui::Sense::hover()));

            ui.vertical(|ui| {
                ui.label(egui::RichText::new(&plugin.plugin.metadata.display_name).heading().strong());
                ui.label(format!("版本: {}", plugin.plugin.metadata.version));
                ui.label(format!("作者: {}", plugin.plugin.metadata.author));

                ui.horizontal(|ui| {
                    if plugin.verified {
                        ui.label(egui::RichText::new("✓ 已验证").color(egui::Color32::from_rgb(100, 255, 100)));
                    }
                    if plugin.featured {
                        ui.label(egui::RichText::new("⭐ 精选").color(egui::Color32::from_rgb(255, 200, 100)));
                    }
                });
            });
        });

        ui.separator();

        // Description
        ui.group(|ui| {
            ui.label(egui::RichText::new("描述").strong());
            ui.label(&plugin.plugin.metadata.description);
        });

        ui.add_space(10.0);

        // Statistics
        ui.group(|ui| {
            ui.label(egui::RichText::new("统计信息").strong());
            ui.horizontal(|ui| {
                ui.label(format!("评分: {:.1}/5.0 ({} 评价)", plugin.rating, plugin.review_count));
                ui.separator();
                ui.label(format!("下载量: {}", plugin.download_count));
                ui.separator();
                ui.label(format!("大小: {:.1} MB", plugin.size_mb));
            });
        });

        ui.add_space(10.0);

        // Capabilities
        ui.group(|ui| {
            ui.label(egui::RichText::new("功能特性").strong());
            ui.horizontal_wrapped(|ui| {
                if plugin.plugin.manifest.capabilities.provides_resources {
                    ui.label("📁 资源");
                }
                if plugin.plugin.manifest.capabilities.provides_tools {
                    ui.label("🔧 工具");
                }
                if plugin.plugin.manifest.capabilities.provides_prompts {
                    ui.label("💬 提示");
                }
                if plugin.plugin.manifest.capabilities.supports_sampling {
                    ui.label("🎯 采样");
                }
                if plugin.plugin.manifest.capabilities.supports_notifications {
                    ui.label("🔔 通知");
                }
                if plugin.plugin.manifest.capabilities.supports_progress {
                    ui.label("📊 进度");
                }
            });
        });

        ui.add_space(10.0);

        // Permissions
        if !plugin.plugin.manifest.permissions.is_empty() {
            ui.group(|ui| {
                ui.label(egui::RichText::new("所需权限").strong());
                for permission in &plugin.plugin.manifest.permissions {
                    ui.horizontal(|ui| {
                        let level_color = match permission.level {
                            crate::state::PermissionLevel::Low => egui::Color32::from_rgb(100, 255, 100),
                            crate::state::PermissionLevel::Medium => egui::Color32::from_rgb(255, 200, 100),
                            crate::state::PermissionLevel::High => egui::Color32::from_rgb(255, 100, 100),
                            crate::state::PermissionLevel::Critical => egui::Color32::from_rgb(255, 50, 50),
                        };
                        ui.label(egui::RichText::new(&permission.resource).color(level_color));
                        ui.label(&permission.description);
                    });
                }
            });

            ui.add_space(10.0);
        }

        // Tools
        if !plugin.plugin.manifest.tools.is_empty() {
            ui.group(|ui| {
                ui.label(egui::RichText::new("提供的工具").strong());
                for tool in &plugin.plugin.manifest.tools {
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new(&tool.name).strong());
                        ui.label(&tool.description);
                    });
                }
            });

            ui.add_space(10.0);
        }

        // Installation section
        ui.separator();
        ui.horizontal(|ui| {
            // Check if plugin is already installed
            if let Some(installed) = state.plugin_manager.get_plugin(&plugin.plugin.id) {
                match installed.status {
                    crate::plugins::PluginStatus::NotInstalled => {
                        if ui.button("安装插件").clicked() {
                            if let Err(e) = state.plugin_manager.install_plugin(
                                plugin.plugin.id,
                                plugin.download_url.clone()
                            ) {
                                log::error!("Failed to install plugin: {}", e);
                            }
                        }
                    }
                    crate::plugins::PluginStatus::Installing => {
                        ui.add(egui::Spinner::new());
                        ui.label("安装中...");
                    }
                    crate::plugins::PluginStatus::Installed => {
                        if ui.button("启用插件").clicked() {
                            if let Err(e) = state.plugin_manager.enable_plugin(plugin.plugin.id, &state.current_role) {
                                log::error!("Failed to enable plugin: {}", e);
                            }
                        }
                        if ui.button("卸载插件").clicked() {
                            if let Err(e) = state.plugin_manager.uninstall_plugin(plugin.plugin.id) {
                                log::error!("Failed to uninstall plugin: {}", e);
                            }
                        }
                    }
                    crate::plugins::PluginStatus::Enabled => {
                        ui.label("✓ 已启用");
                        if ui.button("禁用插件").clicked() {
                            if let Err(e) = state.plugin_manager.disable_plugin(plugin.plugin.id) {
                                log::error!("Failed to disable plugin: {}", e);
                            }
                        }
                        if ui.button("卸载插件").clicked() {
                            if let Err(e) = state.plugin_manager.uninstall_plugin(plugin.plugin.id) {
                                log::error!("Failed to uninstall plugin: {}", e);
                            }
                        }
                    }
                    _ => {
                        ui.label(installed.get_status_text());
                    }
                }
            } else {
                if ui.button("安装插件").clicked() {
                    if let Err(e) = state.plugin_manager.install_plugin(
                        plugin.plugin.id,
                        plugin.download_url.clone()
                    ) {
                        log::error!("Failed to install plugin: {}", e);
                    }
                }
            }

            if ui.button("关闭").clicked() {
                state.ui_state.selected_plugin = None;
            }
        });
    });
}

/// Render installed plugin details
fn render_installed_plugin_details(ui: &mut egui::Ui, plugin: &crate::plugins::Plugin, state: &mut IToolsState) {
    egui::ScrollArea::vertical().show(ui, |ui| {
        // Header with icon and basic info
        ui.horizontal(|ui| {
            // Plugin icon placeholder
            ui.add_sized([64.0, 64.0], egui::Button::new("📦").sense(egui::Sense::hover()));

            ui.vertical(|ui| {
                ui.label(egui::RichText::new(&plugin.metadata.display_name).heading().strong());
                ui.label(format!("版本: {}", plugin.metadata.version));
                ui.label(format!("作者: {}", plugin.metadata.author));

                // Status
                plugin_status_indicator(ui, &plugin.status);
            });
        });

        ui.separator();

        // Description
        ui.group(|ui| {
            ui.label(egui::RichText::new("描述").strong());
            ui.label(&plugin.metadata.description);
        });

        ui.add_space(10.0);

        // Installation info
        ui.group(|ui| {
            ui.label(egui::RichText::new("安装信息").strong());
            if let Some(installed_at) = plugin.installed_at {
                ui.label(format!("安装时间: {}", installed_at.format("%Y-%m-%d %H:%M:%S")));
            }
            if let Some(last_updated) = plugin.last_updated {
                ui.label(format!("最后更新: {}", last_updated.format("%Y-%m-%d %H:%M:%S")));
            }
            if let Some(path) = &plugin.installation_path {
                ui.label(format!("安装路径: {}", path.display()));
            }
        });

        ui.add_space(10.0);

        // Capabilities
        ui.group(|ui| {
            ui.label(egui::RichText::new("功能特性").strong());
            ui.horizontal_wrapped(|ui| {
                if plugin.manifest.capabilities.provides_resources {
                    ui.label("📁 资源");
                }
                if plugin.manifest.capabilities.provides_tools {
                    ui.label("🔧 工具");
                }
                if plugin.manifest.capabilities.provides_prompts {
                    ui.label("💬 提示");
                }
                if plugin.manifest.capabilities.supports_sampling {
                    ui.label("🎯 采样");
                }
                if plugin.manifest.capabilities.supports_notifications {
                    ui.label("🔔 通知");
                }
                if plugin.manifest.capabilities.supports_progress {
                    ui.label("📊 进度");
                }
            });
        });

        ui.add_space(10.0);

        // Permissions
        if !plugin.manifest.permissions.is_empty() {
            ui.group(|ui| {
                ui.label(egui::RichText::new("所需权限").strong());
                for permission in &plugin.manifest.permissions {
                    ui.horizontal(|ui| {
                        let level_color = match permission.level {
                            crate::state::PermissionLevel::Low => egui::Color32::from_rgb(100, 255, 100),
                            crate::state::PermissionLevel::Medium => egui::Color32::from_rgb(255, 200, 100),
                            crate::state::PermissionLevel::High => egui::Color32::from_rgb(255, 100, 100),
                            crate::state::PermissionLevel::Critical => egui::Color32::from_rgb(255, 50, 50),
                        };
                        ui.label(egui::RichText::new(&permission.resource).color(level_color));
                        ui.label(&permission.description);
                    });
                }
            });

            ui.add_space(10.0);
        }

        // Tools
        if !plugin.manifest.tools.is_empty() {
            ui.group(|ui| {
                ui.label(egui::RichText::new("提供的工具").strong());
                for tool in &plugin.manifest.tools {
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new(&tool.name).strong());
                        ui.label(&tool.description);
                    });
                }
            });

            ui.add_space(10.0);
        }

        // Resources
        if !plugin.manifest.resources.is_empty() {
            ui.group(|ui| {
                ui.label(egui::RichText::new("提供的资源").strong());
                for resource in &plugin.manifest.resources {
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new(&resource.name).strong());
                        ui.label(&resource.description);
                    });
                }
            });

            ui.add_space(10.0);
        }

        // Control buttons
        ui.separator();
        ui.horizontal(|ui| {
            match plugin.status {
                crate::plugins::PluginStatus::Installed => {
                    if ui.button("启用插件").clicked() {
                        if let Err(e) = state.plugin_manager.enable_plugin(plugin.id, &state.current_role) {
                            log::error!("Failed to enable plugin: {}", e);
                        }
                    }
                }
                crate::plugins::PluginStatus::Enabled => {
                    if ui.button("禁用插件").clicked() {
                        if let Err(e) = state.plugin_manager.disable_plugin(plugin.id) {
                            log::error!("Failed to disable plugin: {}", e);
                        }
                    }
                }
                _ => {}
            }

            if plugin.can_be_uninstalled() {
                if ui.button("卸载插件").clicked() {
                    if let Err(e) = state.plugin_manager.uninstall_plugin(plugin.id) {
                        log::error!("Failed to uninstall plugin: {}", e);
                    }
                }
            }

            if ui.button("关闭").clicked() {
                state.ui_state.selected_plugin = None;
            }
        });
    });
}
