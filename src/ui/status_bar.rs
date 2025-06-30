use eframe::egui;
use crate::app::Module;
use inote::db_state::SaveStatus;

/// 状态类型
#[derive(Debug, Clone, PartialEq)]
pub enum StatusType {
    /// 普通信息
    Info,
    /// 成功信息
    Success,
    /// 警告信息
    Warning,
    /// 错误信息
    Error,
    /// 进行中
    Progress,
}

/// Render the status bar
pub fn render_status_bar(
    ui: &mut egui::Ui,
    system_service: &crate::services::system_service::SystemService,
    show_right_sidebar: &mut bool,
    active_module: Module,
    save_status: SaveStatus,
) {
    ui.horizontal(|ui| {
        // Left side - system resource monitoring
        ui.horizontal(|ui| {
            let cpu = system_service.get_cpu_usage();
            let memory = system_service.get_memory_usage();

            // 确保CPU和内存值不是NaN
            let cpu_display = if cpu.is_nan() { 0.0 } else { cpu };
            let memory_display = if memory.is_nan() { 0.0 } else { memory };

            // 使用固定宽度的标签显示CPU使用率，避免闪烁，左对齐
            ui.with_layout(egui::Layout::left_to_right(egui::Align::Min), |ui| {
                let cpu_label = egui::Label::new(format!("CPU: {:.1}%", cpu_display))
                    .wrap();
                ui.add_sized([80.0, 0.0], cpu_label);
            });

            ui.separator();

            // 使用固定宽度的标签显示内存使用率，避免闪烁，左对齐
            ui.with_layout(egui::Layout::left_to_right(egui::Align::Min), |ui| {
                let memory_label = egui::Label::new(format!("内存: {:.1}%", memory_display))
                    .wrap();
                ui.add_sized([100.0, 0.0], memory_label);
            });
            ui.separator();

            // 显示当前模块的状态信息
            match active_module {
                Module::Home => {
                    ui.label("主页");
                },
                Module::Note => {
                    // 显示笔记保存状态
                    match save_status {
                        SaveStatus::Saved => {
                            ui.label(egui::RichText::new("✓ 已保存").color(egui::Color32::from_rgb(0, 180, 0)));
                        },
                        SaveStatus::Saving => {
                            ui.label(egui::RichText::new("⋯ 保存中").color(egui::Color32::from_rgb(100, 100, 255)));
                        },
                        SaveStatus::Modified => {
                            ui.label(egui::RichText::new("✎ 未保存").color(egui::Color32::from_rgb(255, 180, 0)));
                        },
                        SaveStatus::Error(err) => {
                            ui.label(egui::RichText::new(format!("❌ 保存失败: {}", err)).color(egui::Color32::from_rgb(255, 0, 0)));
                        }
                    }
                },
                Module::Search => {
                    ui.label("搜索模式");
                },
                Module::Terminal => {
                    ui.label("终端模式");
                },
                Module::Files => {
                    ui.label("文件管理模式");
                },
                Module::DataAnalysis => {
                    ui.label("数据分析模式");
                },
                Module::ITools => {
                    ui.label("iTools 模式");
                },
                Module::Settings => {
                    ui.label("设置模式");
                },
            }
        });

        // Right side - tools
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            // 右侧边栏切换按钮 (AI助手)
            if ui.add(egui::Button::new("🤖 AI助手")
                .selected(*show_right_sidebar))
                .clicked() {
                *show_right_sidebar = !*show_right_sidebar;
                log::info!("右侧边栏状态: {}", *show_right_sidebar);
            }

            ui.add_space(10.0);

            // Version info
            ui.label(format!("SeeU Desktop v{}", env!("CARGO_PKG_VERSION")));
        });
    });
}