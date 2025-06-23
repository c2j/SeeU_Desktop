use eframe::egui;
use crate::state::{IToolsState, IToolsView};

/// Render the main iTools interface
pub fn render_main_interface(ui: &mut egui::Ui, state: &mut IToolsState) {
    // Get available height for proper layout
    let available_height = ui.available_height();

    // Create main container
    egui::containers::Frame::none()
        .fill(ui.style().visuals.window_fill)
        .show(ui, |ui| {
            ui.set_min_height(available_height);

            // Top toolbar with role selector and view tabs
            render_top_toolbar(ui, state);

            ui.separator();

            // Main content area based on current view
            match state.ui_state.current_view {
                IToolsView::Dashboard => {
                    crate::ui::dashboard::render_dashboard(ui, state);
                }
                IToolsView::PluginMarket => {
                    crate::ui::marketplace::render_marketplace(ui, state);
                }
                IToolsView::InstalledPlugins => {
                    crate::ui::plugins::render_installed_plugins(ui, state);
                }
                IToolsView::McpSettings => {
                    render_mcp_settings(ui, state);
                }
            }
        });

    // Render plugin details dialog if needed
    crate::ui::components::render_plugin_details_dialog(ui.ctx(), state);
}

/// Render the top toolbar with navigation and controls
fn render_top_toolbar(ui: &mut egui::Ui, state: &mut IToolsState) {
    ui.horizontal(|ui| {
        // Role selector
        render_role_selector(ui, state);

        ui.separator();

        // View tabs
        render_view_tabs(ui, state);

        // ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
        //     // Info button or other controls can be added here
        //     ui.label(egui::RichText::new("💡 在全局设置中配置 iTools").weak());
        // });
    });
}

/// Render role selector dropdown
fn render_role_selector(ui: &mut egui::Ui, state: &mut IToolsState) {
    ui.label("当前角色:");

    egui::ComboBox::from_id_source("role_selector")
        .selected_text(state.current_role.display_name())
        .show_ui(ui, |ui| {
            use crate::roles::UserRole;

            let roles = [
                UserRole::BusinessUser,
                UserRole::Developer,
                UserRole::Operations,
                UserRole::Administrator,
            ];

            for role in &roles {
                let selected = ui.selectable_value(
                    &mut state.current_role,
                    role.clone(),
                    role.display_name(),
                );

                if selected.clicked() {
                    log::info!("Role changed to: {:?}", role);
                    // Update UI components based on new role
                    update_ui_for_role(state, role);
                }
            }
        });
}

/// Render view navigation tabs
fn render_view_tabs(ui: &mut egui::Ui, state: &mut IToolsState) {
    ui.horizontal(|ui| {
        // Get available UI components for current role
        let ui_components = state.current_role.get_ui_components();

        // Dashboard (always available)
        if ui.selectable_label(
            state.ui_state.current_view == IToolsView::Dashboard,
            "📊 仪表板"
        ).clicked() {
            state.ui_state.current_view = IToolsView::Dashboard;
        }

        // Plugin Market
        if ui.selectable_label(
            state.ui_state.current_view == IToolsView::PluginMarket,
            "🛒 插件市场"
        ).clicked() {
            state.ui_state.current_view = IToolsView::PluginMarket;
        }

        // Installed Plugins
        if ui.selectable_label(
            state.ui_state.current_view == IToolsView::InstalledPlugins,
            "🔧 已安装插件"
        ).clicked() {
            state.ui_state.current_view = IToolsView::InstalledPlugins;
        }

        // MCP Settings
        if ui.selectable_label(
            state.ui_state.current_view == IToolsView::McpSettings,
            "⚙️ MCP Hub"
        ).clicked() {
            state.ui_state.current_view = IToolsView::McpSettings;
        }

        // Show role-specific tabs
        use crate::roles::UiComponent;

        if ui_components.contains(&UiComponent::CodeEditor) {
            if ui.selectable_label(false, "💻 代码编辑器").clicked() {
                // TODO: Open code editor
            }
        }

        if ui_components.contains(&UiComponent::SystemMonitor) {
            if ui.selectable_label(false, "📈 系统监控").clicked() {
                // TODO: Open system monitor
            }
        }

        if ui_components.contains(&UiComponent::SecurityAudit) {
            if ui.selectable_label(false, "🔒 安全审计").clicked() {
                // TODO: Open security audit
            }
        }
    });
}

/// Update UI state when role changes
fn update_ui_for_role(state: &mut IToolsState, new_role: &crate::roles::UserRole) {
    log::info!("Updating UI for role: {:?}", new_role);

    // Clear any role-specific state
    state.ui_state.search_query.clear();
    state.ui_state.selected_plugin = None;

    // Reset to dashboard view
    state.ui_state.current_view = IToolsView::Dashboard;

    // Log role change for audit
    state.log_audit(
        format!("Role changed to {}", new_role.display_name()),
        None,
        crate::state::AuditResult::Success,
    );
}

/// Render status indicators
pub fn render_status_indicators(ui: &mut egui::Ui, state: &IToolsState) {
    ui.horizontal(|ui| {
        // Connection status
        let connected_plugins = state.mcp_client.get_connected_plugins().len();
        ui.label(format!("🔗 已连接插件: {}", connected_plugins));

        // Active operations
        let pending_ops = state.plugin_manager.get_pending_operations().len();
        if pending_ops > 0 {
            ui.label(format!("⏳ 进行中操作: {}", pending_ops));
        }

        // Security status
        let high_risk_events = 0; // TODO: Get from audit logger
        if high_risk_events > 0 {
            ui.colored_label(
                egui::Color32::from_rgb(255, 100, 100),
                format!("⚠ 高风险事件: {}", high_risk_events)
            );
        }


    });
}

/// Render loading overlay
pub fn render_loading_overlay(ui: &mut egui::Ui, message: &str) {
    let rect = ui.available_rect_before_wrap();

    // Semi-transparent background
    ui.painter().rect_filled(
        rect,
        egui::Rounding::ZERO,
        egui::Color32::from_black_alpha(128),
    );

    // Loading message in center
    ui.allocate_ui_at_rect(rect, |ui| {
        ui.centered_and_justified(|ui| {
            ui.vertical_centered(|ui| {
                ui.spinner();
                ui.label(message);
            });
        });
    });
}

/// Render error message
pub fn render_error_message(ui: &mut egui::Ui, error: &str) {
    ui.colored_label(egui::Color32::from_rgb(255, 100, 100), format!("❌ {}", error));
}

/// Render success message
pub fn render_success_message(ui: &mut egui::Ui, message: &str) {
    ui.colored_label(egui::Color32::from_rgb(100, 255, 100), format!("✅ {}", message));
}

/// Render warning message
pub fn render_warning_message(ui: &mut egui::Ui, warning: &str) {
    ui.colored_label(egui::Color32::from_rgb(255, 200, 100), format!("⚠ {}", warning));
}

/// Render MCP settings view
fn render_mcp_settings(ui: &mut egui::Ui, state: &mut IToolsState) {
    // Check if we have MCP settings UI
    if state.mcp_settings_ui.is_none() {
        ui.vertical_centered(|ui| {
            ui.label("MCP Hub界面未初始化");
            if ui.button("重新初始化").clicked() {
                state.initialize_mcp_settings_ui();
            }
        });
        return;
    }

    // Use our complete MCP settings UI
    if let Some(mcp_settings_ui) = &mut state.mcp_settings_ui {
        let ctx = ui.ctx().clone();
        mcp_settings_ui.render(&ctx, ui);
    }
}


