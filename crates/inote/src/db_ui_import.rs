use eframe::egui;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use crate::db_state::DbINoteState;
use crate::siyuan_import::{SiyuanImporter, ImportStats};

/// 思源笔记导入对话框状态
#[derive(Clone)]
pub struct SiyuanImportState {
    pub show_dialog: bool,
    pub siyuan_path: String,
    pub import_in_progress: bool,
    pub import_completed: bool,
    pub import_error: Option<String>,
    pub import_stats: Option<ImportStats>,
}

impl Default for SiyuanImportState {
    fn default() -> Self {
        Self {
            show_dialog: false,
            siyuan_path: String::new(),
            import_in_progress: false,
            import_completed: false,
            import_error: None,
            import_stats: None,
        }
    }
}

/// 渲染思源笔记导入对话框
pub fn render_siyuan_import_dialog(ui: &mut egui::Ui, state: &mut DbINoteState) {
    if !state.siyuan_import.show_dialog {
        return;
    }

    let mut dialog_open = true;

    egui::Window::new("从思源笔记导入")
        .collapsible(false)
        .resizable(true)
        .min_width(400.0)
        .open(&mut dialog_open)
        .show(ui.ctx(), |ui| {
            ui.vertical(|ui| {
                ui.add_space(10.0);

                // 如果导入正在进行中，显示进度
                if state.siyuan_import.import_in_progress {
                    ui.vertical_centered(|ui| {
                        ui.label("正在导入数据，请稍候...");
                        ui.add_space(10.0);
                        ui.add(egui::ProgressBar::new(0.5).animate(true));
                    });
                    return;
                }

                // 如果导入已完成，显示结果
                if state.siyuan_import.import_completed {
                    ui.vertical_centered(|ui| {
                        ui.heading("导入完成！");
                        ui.add_space(10.0);

                        if let Some(stats) = &state.siyuan_import.import_stats {
                            ui.label(format!("成功导入 {} 个笔记本", stats.notebooks_count));
                            ui.label(format!("成功导入 {} 个笔记", stats.notes_count));
                            ui.label(format!("成功导入 {} 个标签", stats.tags_count));
                        }

                        ui.add_space(10.0);
                        if ui.button("关闭").clicked() {
                            state.siyuan_import.show_dialog = false;
                            state.siyuan_import.import_completed = false;
                            state.siyuan_import.import_stats = None;
                        }
                    });
                    return;
                }

                // 如果有错误，显示错误信息
                let has_error = state.siyuan_import.import_error.is_some();
                if has_error {
                    let error_message = state.siyuan_import.import_error.clone().unwrap_or_default();
                    ui.vertical_centered(|ui| {
                        ui.heading("导入失败");
                        ui.add_space(10.0);
                        ui.label(&error_message);
                        ui.add_space(10.0);
                        if ui.button("关闭").clicked() {
                            state.siyuan_import.show_dialog = false;
                            state.siyuan_import.import_error = None;
                        }
                    });
                    return;
                }

                // 正常显示导入对话框
                ui.heading("从思源笔记导入数据");
                ui.add_space(10.0);

                ui.label("请输入思源笔记的工作空间目录路径:");
                ui.text_edit_singleline(&mut state.siyuan_import.siyuan_path);

                ui.add_space(5.0);
                ui.label("💡 提示:");
                ui.label("• 请选择思源笔记的工作空间根目录");
                ui.label("• 该目录应包含以时间戳格式命名的笔记本文件夹");
                ui.label("• 例如: ~/SiYuan/ 或 /Users/username/Documents/SiYuan/");
                ui.label("• 工具将自动识别笔记本、解析.sy文件并导入附件");

                ui.add_space(20.0);
                ui.separator();
                ui.add_space(10.0);

                ui.horizontal(|ui| {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("开始导入").clicked() {
                            start_import(state);
                        }

                        if ui.button("取消").clicked() {
                            state.siyuan_import.show_dialog = false;
                        }
                    });
                });
            });
        });

    if !dialog_open {
        state.siyuan_import.show_dialog = false;
    }
}

/// 开始导入过程
fn start_import(state: &mut DbINoteState) {
    // 验证路径
    if state.siyuan_import.siyuan_path.trim().is_empty() {
        state.siyuan_import.import_error = Some("请输入有效的思源笔记工作空间目录路径".to_string());
        return;
    }

    let path = PathBuf::from(&state.siyuan_import.siyuan_path);
    if !path.exists() || !path.is_dir() {
        state.siyuan_import.import_error = Some(format!("目录不存在: {}", path.display()));
        return;
    }

    // 重置状态
    state.siyuan_import.import_error = None;
    state.siyuan_import.import_completed = false;
    state.siyuan_import.import_stats = None;

    // 标记导入开始
    state.siyuan_import.import_in_progress = true;

    // 克隆必要的数据用于线程
    let path_clone = path.clone();
    let storage_clone = state.storage.clone();

    // 简化版本：直接在当前线程中执行导入（避免线程同步问题）
    let result: Result<ImportStats, Box<dyn std::error::Error>> = (|| {
        log::info!("开始导入思源笔记，路径: {}", path_clone.display());

        // 获取存储管理器
        let storage = storage_clone.lock().map_err(|e| {
            log::error!("无法获取存储锁: {}", e);
            format!("无法获取存储锁: {}", e)
        })?;

        // 创建导入器
        let mut importer = SiyuanImporter::new(storage.clone(), path_clone);

        // 执行导入
        let stats = importer.import()?;

        log::info!("导入完成: {} 个笔记本, {} 个笔记, {} 个标签",
            stats.notebooks_count, stats.notes_count, stats.tags_count);

        Ok(stats)
    })();

    // 更新状态
    state.siyuan_import.import_in_progress = false;

    match result {
        Ok(stats) => {
            log::info!("导入成功完成");
            state.siyuan_import.import_completed = true;
            state.siyuan_import.import_stats = Some(stats);
            state.siyuan_import.import_error = None;

            // 重新加载数据以显示导入的笔记
            log::info!("重新加载数据以显示导入的笔记...");
            state.force_reload_data();
            log::info!("数据重新加载完成");
        },
        Err(e) => {
            log::error!("导入失败: {}", e);
            state.siyuan_import.import_error = Some(format!("导入失败: {}", e));
            state.siyuan_import.import_completed = false;
        }
    }
}

/// 添加思源笔记导入按钮到工具栏
pub fn add_siyuan_import_button(ui: &mut egui::Ui, state: &mut DbINoteState) {
    if ui.button("📥 从思源笔记导入").clicked() {
        state.siyuan_import.show_dialog = true;
    }
}
