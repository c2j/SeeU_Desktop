//! 文件树功能演示
//! 
//! 这个示例展示了基于egui_ltreeview的新文件树实现

use eframe::egui;
use ifile_editor::{IFileEditorState, ui::render_file_tree};
use std::path::PathBuf;

fn main() -> Result<(), eframe::Error> {
    env_logger::init();
    
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0])
            .with_title("iFile Editor - 文件树演示"),
        ..Default::default()
    };
    
    eframe::run_native(
        "iFile Editor Demo",
        options,
        Box::new(|cc| {
            cc.egui_ctx.set_visuals(egui::Visuals::dark());
            Ok(Box::new(FileTreeDemo::new()))
        }),
    )
}

struct FileTreeDemo {
    state: IFileEditorState,
    selected_directory: Option<PathBuf>,
}

impl FileTreeDemo {
    fn new() -> Self {
        let mut state = IFileEditorState::new();
        
        // 尝试设置当前目录作为根目录
        if let Ok(current_dir) = std::env::current_dir() {
            if let Err(e) = state.file_tree.set_root(current_dir) {
                log::error!("Failed to set root directory: {}", e);
            }
        }
        
        Self {
            state,
            selected_directory: None,
        }
    }
}

impl eframe::App for FileTreeDemo {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("🗂️ iFile Editor - 文件树演示");
            ui.separator();
            
            ui.horizontal(|ui| {
                // 左侧：文件树
                ui.vertical(|ui| {
                    ui.set_width(300.0);
                    ui.heading("文件树");
                    ui.separator();
                    
                    // 目录选择按钮
                    if ui.button("📁 选择目录").clicked() {
                        if let Some(path) = rfd::FileDialog::new().pick_folder() {
                            self.selected_directory = Some(path.clone());
                            if let Err(e) = self.state.file_tree.set_root(path) {
                                log::error!("Failed to set root directory: {}", e);
                            }
                        }
                    }
                    
                    ui.separator();
                    
                    // 渲染文件树
                    egui::ScrollArea::vertical()
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            render_file_tree(ui, &mut self.state);
                        });
                });
                
                ui.separator();
                
                // 右侧：信息面板
                ui.vertical(|ui| {
                    ui.heading("信息面板");
                    ui.separator();
                    
                    // 显示当前根目录
                    if let Some(root) = &self.state.file_tree.root_path {
                        ui.label(format!("📂 根目录: {}", root.display()));
                    } else {
                        ui.label("📂 未选择根目录");
                    }
                    
                    ui.separator();
                    
                    // 显示统计信息
                    ui.label(format!("📄 文件数量: {}", self.state.file_tree.file_entries.len()));
                    ui.label(format!("📁 目录数量: {}", 
                        self.state.file_tree.file_entries.values()
                            .filter(|entry| entry.is_dir)
                            .count()
                    ));
                    
                    ui.separator();
                    
                    // 显示选中的文件
                    let selected = self.state.file_tree.tree_view_state.selected();
                    if !selected.is_empty() {
                        ui.label("🎯 选中的项目:");
                        for node_id in selected {
                            let path = &node_id.0;
                            if let Some(entry) = self.state.file_tree.get_file_entry(path) {
                                ui.label(format!("  {} {}", entry.icon, entry.name));
                            }
                        }
                    } else {
                        ui.label("🎯 未选中任何项目");
                    }
                    
                    ui.separator();
                    
                    // 功能说明
                    ui.heading("功能特性");
                    ui.label("✅ 基于 egui_ltreeview 的专业树形视图");
                    ui.label("✅ 支持多选（Ctrl/Cmd + 点击）");
                    ui.label("✅ 键盘导航（方向键）");
                    ui.label("✅ 右键上下文菜单");
                    ui.label("✅ 文件类型图标显示");
                    ui.label("✅ 目录展开/折叠");
                    ui.label("✅ 高性能虚拟化支持");
                    
                    ui.separator();
                    
                    // 操作说明
                    ui.heading("操作说明");
                    ui.label("🖱️ 单击：选择文件/目录");
                    ui.label("🖱️ 双击：激活（打开文件/切换目录）");
                    ui.label("🖱️ 右键：显示上下文菜单");
                    ui.label("⌨️ 方向键：键盘导航");
                    ui.label("⌨️ Ctrl/Cmd + 点击：多选");
                    ui.label("⌨️ Enter：激活选中项");
                });
            });
        });
        
        // 显示错误信息
        let error_message = self.state.last_error.as_ref().map(|e| e.to_string());
        if let Some(error_text) = error_message {
            egui::Window::new("❌ 错误")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label(format!("发生错误: {}", error_text));
                    if ui.button("确定").clicked() {
                        self.state.last_error = None;
                    }
                });
        }
    }
}
