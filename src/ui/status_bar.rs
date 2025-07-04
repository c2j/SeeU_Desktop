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
    ifile_status: Option<ifile_editor::FileStatusInfo>,
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
                Module::FileEditor => {
                    if let Some(file_status) = &ifile_status {
                        // 显示文件名
                        if let Some(file_name) = file_status.file_path.file_name() {
                            ui.label(format!("📄 {}", file_name.to_string_lossy()));
                        } else {
                            ui.label("📄 未知文件");
                        }

                        ui.separator();

                        // 显示光标位置
                        ui.label(format!("行: {} 列: {}", file_status.cursor_line, file_status.cursor_column));

                        ui.separator();

                        // 显示编码
                        ui.label(&file_status.encoding);

                        ui.separator();

                        // 显示语言
                        if let Some(lang) = &file_status.language {
                            ui.label(lang);
                            ui.separator();
                        }

                        // 显示修改状态
                        if file_status.modified {
                            ui.label(egui::RichText::new("● 已修改").color(egui::Color32::from_rgb(255, 180, 0)));
                        } else {
                            ui.label(egui::RichText::new("○ 已保存").color(egui::Color32::from_rgb(0, 180, 0)));
                        }

                        ui.separator();

                        // 显示只读状态
                        if file_status.read_only {
                            ui.label(egui::RichText::new("🔒 只读").color(egui::Color32::from_rgb(255, 100, 100)));
                            ui.separator();
                        }

                        // 显示文件统计
                        ui.label(format!("行: {} 字符: {}", file_status.line_count, file_status.char_count));
                    } else {
                        ui.label("文件编辑器模式");
                    }
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