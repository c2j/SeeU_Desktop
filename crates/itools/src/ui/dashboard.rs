use eframe::egui;
use crate::state::IToolsState;
use crate::roles::UiComponent;

/// Render the dashboard view
pub fn render_dashboard(ui: &mut egui::Ui, state: &mut IToolsState) {
    // Update background tasks
    crate::update_itools(state);

    egui::ScrollArea::vertical()
        .auto_shrink([false; 2])
        .show(ui, |ui| {
            // Welcome section
            render_welcome_section(ui, state);

            ui.separator();

            // Role-specific dashboard components
            render_role_specific_components(ui, state);

            ui.separator();

            // Quick actions
            render_quick_actions(ui, state);

            ui.separator();

            // System status
            render_system_status(ui, state);
        });
}

/// Render welcome section
fn render_welcome_section(ui: &mut egui::Ui, state: &IToolsState) {
    ui.vertical(|ui| {
        ui.heading("欢迎使用 iTools");
        ui.label(format!("当前角色: {}", state.current_role.display_name()));

        ui.add_space(10.0);

        // Role description
        let role_def = state.current_role.get_role_definition();
        ui.label(format!("角色描述: {}", role_def.description));
    });
}

/// Render role-specific dashboard components
fn render_role_specific_components(ui: &mut egui::Ui, state: &IToolsState) {
    let ui_components = state.current_role.get_ui_components();

    ui.heading("角色专用功能");

    // Create a grid layout for components
    egui::Grid::new("role_components")
        .num_columns(2)
        .spacing([20.0, 10.0])
        .show(ui, |ui| {
            for component in &ui_components {
                render_component_card(ui, component, state);

                // Start new row after every 2 components
                if ui_components.iter().position(|c| c == component).unwrap() % 2 == 1 {
                    ui.end_row();
                }
            }
        });
}

/// Render a component card
fn render_component_card(ui: &mut egui::Ui, component: &UiComponent, _state: &IToolsState) {
    let (title, description, icon) = match component {
        UiComponent::DataDashboard => ("数据看板", "查看和分析业务数据", "📊"),
        UiComponent::DocumentTemplates => ("文档模板", "使用预定义的文档模板", "📄"),
        UiComponent::AnalysisButtons => ("一键分析", "快速执行数据分析任务", "🔍"),
        UiComponent::VisualizationCharts => ("可视化图表", "创建和查看数据图表", "📈"),
        UiComponent::CodeEditor => ("代码编辑器", "编辑和管理代码文件", "💻"),
        UiComponent::ApiDebugger => ("API 调试器", "测试和调试 API 接口", "🔧"),
        UiComponent::PluginDevTools => ("插件开发工具", "开发和测试插件", "🛠️"),
        UiComponent::LogViewer => ("日志查看器", "查看系统和应用日志", "📋"),
        UiComponent::SystemMonitor => ("系统监控", "监控系统性能和资源", "📈"),
        UiComponent::SecurityAudit => ("安全审计", "查看安全事件和审计日志", "🔒"),
        UiComponent::PluginStatusBoard => ("插件状态板", "监控插件运行状态", "🔌"),
        UiComponent::RolePermissionManager => ("角色权限管理", "管理用户角色和权限", "👥"),
        UiComponent::PluginMarketReview => ("插件市场审核", "审核插件市场提交", "🛒"),
        UiComponent::PolicyConfiguration => ("策略配置", "配置安全和访问策略", "⚙️"),
    };

    egui::Frame::none()
        .fill(ui.style().visuals.faint_bg_color)
        .rounding(egui::Rounding::same(5.0))
        .inner_margin(egui::Margin::same(10.0))
        .show(ui, |ui| {
            ui.set_min_size(egui::Vec2::new(200.0, 80.0));

            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(icon).size(24.0));
                    ui.label(egui::RichText::new(title).strong());
                });

                ui.label(description);

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                    if ui.small_button("打开").clicked() {
                        // TODO: Open the specific component
                        log::info!("Opening component: {:?}", component);
                    }
                });
            });
        });
}

/// Render quick actions section
fn render_quick_actions(ui: &mut egui::Ui, state: &mut IToolsState) {
    ui.heading("快速操作");

    ui.horizontal(|ui| {
        if ui.button("🛒 浏览插件市场").clicked() {
            state.ui_state.current_view = crate::state::IToolsView::PluginMarket;
        }

        if ui.button("🔧 管理已安装插件").clicked() {
            state.ui_state.current_view = crate::state::IToolsView::InstalledPlugins;
        }

        if ui.button("⚙️ 设置").clicked() {
            // Note: Settings are now in global settings
            log::info!("iTools settings moved to global settings");
        }
    });
}

/// Render system status section
fn render_system_status(ui: &mut egui::Ui, state: &IToolsState) {
    ui.heading("系统状态");

    egui::Grid::new("system_status")
        .num_columns(2)
        .spacing([20.0, 5.0])
        .show(ui, |ui| {
            // Plugin statistics
            let installed_plugins = state.plugin_manager.get_installed_plugins().len();
            let connected_plugins = state.mcp_client.get_connected_plugins().len();

            ui.label("已安装插件:");
            ui.label(format!("{}", installed_plugins));
            ui.end_row();

            ui.label("已连接插件:");
            ui.label(format!("{}", connected_plugins));
            ui.end_row();

            // Pending operations
            let pending_ops = state.plugin_manager.get_pending_operations().len();
            ui.label("进行中操作:");
            if pending_ops > 0 {
                ui.colored_label(egui::Color32::from_rgb(255, 200, 100), format!("{}", pending_ops));
            } else {
                ui.label("0");
            }
            ui.end_row();

            // Security status
            ui.label("安全状态:");
            ui.colored_label(egui::Color32::from_rgb(100, 255, 100), "正常");
            ui.end_row();

            // Session info
            ui.label("会话 ID:");
            ui.label(format!("{}", state.security_context.session_id.to_string().chars().take(8).collect::<String>()));
            ui.end_row();
        });

    // Show recent audit events if user has permission
    if state.current_role.has_permission("read", "/audit/*") {
        ui.add_space(10.0);
        ui.label("最近活动:");

        // Show last few audit entries
        for (_i, entry) in state.security_context.audit_log.iter().rev().take(3).enumerate() {
            ui.horizontal(|ui| {
                let time_str = entry.timestamp.format("%H:%M:%S").to_string();
                ui.label(format!("[{}]", time_str));
                ui.label(&entry.action);

                match &entry.result {
                    crate::state::AuditResult::Success => {
                        ui.colored_label(egui::Color32::from_rgb(100, 255, 100), "✓");
                    }
                    crate::state::AuditResult::Denied(_) => {
                        ui.colored_label(egui::Color32::from_rgb(255, 100, 100), "✗");
                    }
                    crate::state::AuditResult::Error(_) => {
                        ui.colored_label(egui::Color32::from_rgb(255, 200, 100), "⚠");
                    }
                }
            });
        }
    }
}
