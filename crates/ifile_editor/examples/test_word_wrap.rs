//! 测试文件编辑器的折行功能和行号显示正确性

use eframe::egui;
use ifile_editor::{IFileEditorState, settings::EditorSettings};
use std::path::PathBuf;

fn main() -> Result<(), eframe::Error> {
    // 设置日志级别为 debug 以查看诊断信息
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Debug)
        .init();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1200.0, 800.0]),
        ..Default::default()
    };

    eframe::run_native(
        "文件编辑器折行和行号测试",
        options,
        Box::new(|_cc| Ok(Box::new(TestApp::new()))),
    )
}

struct TestApp {
    file_editor_state: IFileEditorState,
}

impl TestApp {
    fn new() -> Self {
        let mut state = IFileEditorState::new();

        // 启用行号显示
        state.ui_state.show_line_numbers = true;

        // 创建一个专门测试行号显示的文件内容
        let test_content = "第1行：这是一行很长很长很长很长很长很长很长很长很长很长很长很长很长很长很长很长很长很长很长很长很长很长很长很长很长很长很长很长很长很长很长很长很长很长很长很长很长很长很长很长的文本，用来测试折行功能是否正常工作。
第2行：短行
第3行：这是第三行，也是一行很长很长很长很长很长很长很长很长很长很长很长很长很长很长很长很长很长很长很长很长很长很长很长很长很长很长很长很长很长很长很长很长很长很长很长很长很长很长很长很长的文本。
第4行：空行下面

第6行：包含代码示例的行：
第7行：fn main() {
第8行：    println!(\"Hello, world! This is a very long line that should wrap when word wrap is enabled but should not wrap when word wrap is disabled and should show horizontal scrolling instead.\");
第9行：    let very_long_variable_name_that_exceeds_normal_line_length = \"This is a very long string that demonstrates horizontal scrolling when word wrap is disabled\";
第10行：    println!(\"{}\", very_long_variable_name_that_exceeds_normal_line_length);
第11行：}
第12行：
第13行：这是最后一行，用来测试折行功能在不同模式下的表现，特别是行号的显示是否正确。当启用自动折行时，只有真正的换行符才应该显示新的行号，而自动折行的续行应该显示空白的行号区域。";

        // 创建一个临时文件路径
        let test_file_path = PathBuf::from("test_wrap.txt");

        // 尝试打开文件，如果失败就创建一个临时文件
        if let Err(e) = state.editor.open_file(test_file_path.clone(), &state.settings, None) {
            log::warn!("Failed to open test file: {}, creating temporary file", e);

            // 创建临时文件
            if let Err(write_err) = std::fs::write(&test_file_path, test_content) {
                log::error!("Failed to create temporary test file: {}", write_err);
            } else {
                // 再次尝试打开
                if let Err(open_err) = state.editor.open_file(test_file_path, &state.settings, None) {
                    log::error!("Failed to open temporary test file: {}", open_err);
                }
            }
        }

        Self {
            file_editor_state: state,
        }
    }
}

impl eframe::App for TestApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("文件编辑器折行和行号显示测试");

            ui.horizontal(|ui| {
                ui.label("当前折行状态:");
                if self.file_editor_state.ui_state.word_wrap {
                    ui.colored_label(egui::Color32::GREEN, "✓ 已启用");
                } else {
                    ui.colored_label(egui::Color32::RED, "✗ 已禁用");
                }

                if ui.button("切换折行").clicked() {
                    self.file_editor_state.ui_state.word_wrap = !self.file_editor_state.ui_state.word_wrap;
                    log::info!("Word wrap toggled to: {}", self.file_editor_state.ui_state.word_wrap);
                }

                ui.separator();

                ui.label("行号显示:");
                if self.file_editor_state.ui_state.show_line_numbers {
                    ui.colored_label(egui::Color32::GREEN, "✓ 已启用");
                } else {
                    ui.colored_label(egui::Color32::RED, "✗ 已禁用");
                }

                if ui.button("切换行号").clicked() {
                    self.file_editor_state.ui_state.show_line_numbers = !self.file_editor_state.ui_state.show_line_numbers;
                    log::info!("Line numbers toggled to: {}", self.file_editor_state.ui_state.show_line_numbers);
                }
            });

            ui.separator();

            ui.label("测试说明：");
            ui.label("• 启用折行时，长文本应该在窗口边界处自动换行，只有真正的换行符才显示新行号");
            ui.label("• 禁用折行时，长文本应该显示水平滚动条，每个逻辑行都显示行号");
            ui.label("• 自动折行的续行应该在行号区域显示空白，而不是重复的行号");
            ui.label("• 点击上方按钮可以切换折行模式和行号显示");

            ui.separator();

            // 渲染文件编辑器
            ifile_editor::ui::main_ui::render_file_editor(ui, &mut self.file_editor_state);
        });
    }
}
