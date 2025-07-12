//! 测试清理后的日志输出

use eframe::egui;
use ifile_editor::IFileEditorState;
use std::path::PathBuf;

fn main() -> Result<(), eframe::Error> {
    // 设置日志级别为 info，这样可以看到重要的日志但不会有太多调试信息
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([800.0, 600.0]),
        ..Default::default()
    };

    eframe::run_native(
        "文件编辑器日志清理测试",
        options,
        Box::new(|_cc| Ok(Box::new(TestApp::new()))),
    )
}

struct TestApp {
    file_editor_state: IFileEditorState,
    toggle_count: u32,
}

impl TestApp {
    fn new() -> Self {
        let mut state = IFileEditorState::new();
        
        // 创建一个测试文件内容
        let test_content = "这是一行很长很长很长很长很长很长很长很长很长很长很长很长很长很长很长很长很长很长很长很长很长很长很长很长很长很长很长很长很长很长很长很长很长很长很长很长很长很长很长很长的文本，用来测试折行功能是否正常工作。\n\n这是第二行，也是一行很长很长很长很长很长很长很长很长很长很长很长很长很长很长很长很长很长很长很长很长很长很长很长很长很长很长很长很长很长很长很长很长很长很长很长很长很长很长很长很长的文本。\n\n这是第三行，包含一些代码示例：\nfn main() {\n    println!(\"Hello, world! This is a very long line that should wrap when word wrap is enabled but should not wrap when word wrap is disabled.\");\n    let very_long_variable_name_that_exceeds_normal_line_length = \"This is a very long string that demonstrates horizontal scrolling\";\n    println!(\"{}\", very_long_variable_name_that_exceeds_normal_line_length);\n}\n\n这是最后一行，用来测试折行功能在不同模式下的表现。";
        
        // 创建一个临时文件路径
        let test_file_path = PathBuf::from("test_clean_logs.txt");
        
        // 尝试打开文件，如果失败就创建一个临时文件
        if let Err(_) = state.editor.open_file(test_file_path.clone(), &state.settings, None) {
            // 创建临时文件
            if let Ok(_) = std::fs::write(&test_file_path, test_content) {
                // 再次尝试打开
                let _ = state.editor.open_file(test_file_path, &state.settings, None);
            }
        }
        
        Self {
            file_editor_state: state,
            toggle_count: 0,
        }
    }
}

impl eframe::App for TestApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("文件编辑器日志清理测试");
            
            ui.horizontal(|ui| {
                ui.label("当前折行状态:");
                if self.file_editor_state.ui_state.word_wrap {
                    ui.colored_label(egui::Color32::GREEN, "✓ 已启用");
                } else {
                    ui.colored_label(egui::Color32::RED, "✗ 已禁用");
                }
                
                if ui.button("切换折行").clicked() {
                    self.file_editor_state.ui_state.word_wrap = !self.file_editor_state.ui_state.word_wrap;
                    self.toggle_count += 1;
                    println!("手动切换折行状态 #{}: {} -> {}", 
                            self.toggle_count, 
                            !self.file_editor_state.ui_state.word_wrap, 
                            self.file_editor_state.ui_state.word_wrap);
                }
                
                ui.label(format!("切换次数: {}", self.toggle_count));
            });
            
            ui.separator();
            
            ui.label("说明：");
            ui.label("• 现在应该只看到重要的日志信息");
            ui.label("• 不再有每帧重复的渲染日志");
            ui.label("• 只有在切换折行状态时才会有相关日志");
            ui.label("• 查看控制台输出验证日志清理效果");
            
            ui.separator();
            
            // 渲染文件编辑器
            ifile_editor::ui::main_ui::render_file_editor(ui, &mut self.file_editor_state);
        });
    }
}
