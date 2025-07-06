use eframe::egui;
use crate::state::{IToolsState, MarketFilters};
use crate::plugins::marketplace::{MarketplaceFilters, SortBy};

/// Render the plugin marketplace view
pub fn render_marketplace(ui: &mut egui::Ui, state: &mut IToolsState) {
    egui::SidePanel::left("marketplace_filters")
        .resizable(true)
        .default_width(250.0)
        .show_inside(ui, |ui| {
            render_marketplace_filters(ui, state);
        });

    egui::CentralPanel::default().show_inside(ui, |ui| {
        render_marketplace_content(ui, state);
    });
}

/// Render marketplace filters sidebar
fn render_marketplace_filters(ui: &mut egui::Ui, state: &mut IToolsState) {
    ui.heading("筛选条件");

    ui.separator();

    // Search box
    ui.label("搜索:");
    ui.text_edit_singleline(&mut state.ui_state.search_query);

    ui.add_space(10.0);

    // Category filter
    ui.label("分类:");
    let categories = state.plugin_manager.get_marketplace().get_categories();

    egui::ComboBox::from_id_source("category_filter")
        .selected_text(
            state.ui_state.filters.category_filter
                .as_ref()
                .map(|c| c.as_str())
                .unwrap_or("全部")
        )
        .show_ui(ui, |ui| {
            if ui.selectable_value(&mut state.ui_state.filters.category_filter, None, "全部").clicked() {
                // Category changed
            }

            for category in categories {
                let selected = ui.selectable_value(
                    &mut state.ui_state.filters.category_filter,
                    Some(category.id.clone()),
                    &format!("{} {}", category.icon.as_deref().unwrap_or(""), category.name)
                );

                if selected.clicked() {
                    // Category changed
                }
            }
        });

    ui.add_space(10.0);

    // Role filter
    ui.label("目标角色:");
    egui::ComboBox::from_id_source("role_filter")
        .selected_text(
            state.ui_state.filters.role_filter
                .as_ref()
                .map(|r| r.display_name())
                .unwrap_or("全部")
        )
        .show_ui(ui, |ui| {
            use crate::roles::UserRole;

            if ui.selectable_value(&mut state.ui_state.filters.role_filter, None, "全部").clicked() {
                // Role filter changed
            }

            let roles = [
                UserRole::BusinessUser,
                UserRole::Developer,
                UserRole::Operations,
                UserRole::Administrator,
            ];

            for role in &roles {
                if ui.selectable_value(
                    &mut state.ui_state.filters.role_filter,
                    Some(role.clone()),
                    role.display_name()
                ).clicked() {
                    // Role filter changed
                }
            }
        });

    ui.add_space(10.0);

    // Permission level filter
    ui.label("权限级别:");
    egui::ComboBox::from_id_source("permission_filter")
        .selected_text(
            state.ui_state.filters.permission_level
                .as_ref()
                .map(|p| format!("{:?}", p))
                .unwrap_or_else(|| "全部".to_string())
        )
        .show_ui(ui, |ui| {
            use crate::state::PermissionLevel;

            if ui.selectable_value(&mut state.ui_state.filters.permission_level, None, "全部").clicked() {
                // Permission filter changed
            }

            let levels = [
                PermissionLevel::Low,
                PermissionLevel::Medium,
                PermissionLevel::High,
                PermissionLevel::Critical,
            ];

            for level in &levels {
                if ui.selectable_value(
                    &mut state.ui_state.filters.permission_level,
                    Some(level.clone()),
                    format!("{:?}", level)
                ).clicked() {
                    // Permission filter changed
                }
            }
        });

    ui.add_space(20.0);

    // Filter options
    ui.checkbox(&mut state.ui_state.filters.verified_only, "仅显示已验证插件");
    ui.checkbox(&mut state.ui_state.filters.featured_only, "仅显示推荐插件");

    ui.add_space(20.0);

    // Clear filters button
    if ui.button("清除筛选条件").clicked() {
        state.ui_state.filters = MarketFilters::default();
        state.ui_state.search_query.clear();
    }
}

/// Render marketplace content area
fn render_marketplace_content(ui: &mut egui::Ui, state: &mut IToolsState) {
    ui.vertical(|ui| {
        // Header with sort options and local install button
        render_marketplace_header(ui, state);

        ui.separator();

        // Plugin list
        render_plugin_list(ui, state);
    });
}

/// Render marketplace header
fn render_marketplace_header(ui: &mut egui::Ui, state: &mut IToolsState) {
    ui.horizontal(|ui| {
        ui.heading("插件市场");

        ui.add_space(20.0);

        // Local install button
        if ui.button("📁 从本地安装").clicked() {
            // Open file dialog
            if let Some(file_path) = rfd::FileDialog::new()
                .add_filter("插件包", &["itpkg", "zip", "tar.gz"])
                .set_title("选择插件包文件")
                .pick_file()
            {
                log::info!("Selected plugin file: {:?}", file_path);

                // Install plugin from file
                match state.plugin_manager.install_plugin_from_file(file_path.clone()) {
                    Ok(plugin_id) => {
                        log::info!("Started installation of plugin {} from file: {:?}", plugin_id, file_path);
                    }
                    Err(e) => {
                        log::error!("Failed to start plugin installation from file: {}", e);
                    }
                }
            }
        }

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            // Refresh button
            if ui.button("🔄 刷新市场").clicked() {
                // Refresh marketplace
                state.plugin_manager.get_marketplace_mut().refresh_marketplace();
            }

            ui.add_space(10.0);

            // Sort options
            ui.label("排序:");
            egui::ComboBox::from_id_source("sort_by")
                .selected_text("相关性") // TODO: Use actual sort value
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut SortBy::Relevance, SortBy::Relevance, "相关性");
                    ui.selectable_value(&mut SortBy::Name, SortBy::Name, "名称");
                    ui.selectable_value(&mut SortBy::Rating, SortBy::Rating, "评分");
                    ui.selectable_value(&mut SortBy::Downloads, SortBy::Downloads, "下载量");
                    ui.selectable_value(&mut SortBy::LastUpdated, SortBy::LastUpdated, "更新时间");
                });
        });
    });
}

/// Render plugin list
fn render_plugin_list(ui: &mut egui::Ui, state: &mut IToolsState) {
    // Convert UI filters to marketplace filters
    let marketplace_filters = MarketplaceFilters {
        query: state.ui_state.search_query.clone(),
        category: state.ui_state.filters.category_filter.clone(),
        role: state.ui_state.filters.role_filter.clone(),
        permission_level: state.ui_state.filters.permission_level.clone(),
        verified_only: state.ui_state.filters.verified_only,
        featured_only: state.ui_state.filters.featured_only,
        sort_by: SortBy::Relevance, // TODO: Use actual sort value
    };

    // Get filtered plugins
    let plugins: Vec<_> = state.plugin_manager.get_marketplace().search_plugins(&marketplace_filters)
        .into_iter()
        .cloned()
        .collect();

    if plugins.is_empty() {
        ui.centered_and_justified(|ui| {
            ui.label("没有找到符合条件的插件");
        });
        return;
    }

    egui::ScrollArea::vertical()
        .auto_shrink([false; 2])
        .show(ui, |ui| {
            for plugin in &plugins {
                render_plugin_card(ui, plugin, state);
                ui.add_space(10.0);
            }
        });
}

/// Render a plugin card
fn render_plugin_card(ui: &mut egui::Ui, plugin: &crate::plugins::marketplace::MarketplacePlugin, state: &mut IToolsState) {
    egui::Frame::none()
        .fill(ui.style().visuals.faint_bg_color)
        .stroke(egui::Stroke::new(1.0, ui.style().visuals.widgets.noninteractive.bg_stroke.color))
        .rounding(egui::Rounding::same(5.0))
        .inner_margin(egui::Margin::same(15.0))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                // Plugin icon
                if let Some(icon) = &plugin.plugin.metadata.icon {
                    ui.label(egui::RichText::new(icon).size(32.0));
                } else {
                    ui.label(egui::RichText::new("📦").size(32.0));
                }

                ui.vertical(|ui| {
                    // Plugin name and version
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new(&plugin.plugin.metadata.display_name).strong().size(16.0));
                        ui.label(egui::RichText::new(&format!("v{}", plugin.plugin.metadata.version)).weak());

                        if plugin.verified {
                            ui.label(egui::RichText::new("✓").color(egui::Color32::from_rgb(100, 255, 100)));
                        }

                        if plugin.featured {
                            ui.label(egui::RichText::new("⭐").color(egui::Color32::from_rgb(255, 200, 100)));
                        }
                    });

                    // Description
                    ui.label(&plugin.plugin.metadata.description);

                    // Metadata
                    ui.horizontal(|ui| {
                        ui.label(format!("作者: {}", plugin.plugin.metadata.author));
                        ui.separator();
                        ui.label(format!("评分: {:.1}/5.0", plugin.rating));
                        ui.separator();
                        ui.label(format!("下载: {}", plugin.download_count));
                    });

                    // Categories and permissions
                    ui.horizontal(|ui| {
                        ui.label("分类:");
                        for category in &plugin.plugin.metadata.categories {
                            ui.small_button(category);
                        }

                        ui.separator();

                        let max_permission = plugin.plugin.get_max_permission_level();
                        let permission_color = match max_permission {
                            crate::state::PermissionLevel::Low => egui::Color32::from_rgb(100, 255, 100),
                            crate::state::PermissionLevel::Medium => egui::Color32::from_rgb(255, 200, 100),
                            crate::state::PermissionLevel::High => egui::Color32::from_rgb(255, 150, 100),
                            crate::state::PermissionLevel::Critical => egui::Color32::from_rgb(255, 100, 100),
                        };

                        ui.colored_label(permission_color, format!("权限: {:?}", max_permission));
                    });
                });

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // Install/Manage button
                    let installed_plugin = state.plugin_manager.get_plugin(&plugin.plugin.id);

                    if let Some(installed) = installed_plugin {
                        match installed.status {
                            crate::plugins::PluginStatus::NotInstalled => {
                                if ui.button("安装").clicked() {
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
                                if ui.button("启用").clicked() {
                                    if let Err(e) = state.plugin_manager.enable_plugin(plugin.plugin.id, &state.current_role) {
                                        log::error!("Failed to enable plugin: {}", e);
                                    }
                                }
                            }
                            crate::plugins::PluginStatus::Enabled => {
                                ui.label("✓ 已启用");
                            }
                            _ => {
                                ui.label(installed.get_status_text());
                            }
                        }
                    } else {
                        if ui.button("安装").clicked() {
                            if let Err(e) = state.plugin_manager.install_plugin(
                                plugin.plugin.id,
                                plugin.download_url.clone()
                            ) {
                                log::error!("Failed to install plugin: {}", e);
                            }
                        }
                    }

                    // Details button
                    if ui.small_button("详情").clicked() {
                        state.ui_state.selected_plugin = Some(plugin.plugin.id);
                        // TODO: Show plugin details dialog
                    }
                });
            });
        });
}
