//! 测试文件编辑器字体、行间距、行号显示一致性

use eframe::egui;
use ifile_editor::{IFileEditorState, settings::EditorSettings};
use std::path::PathBuf;

fn main() -> Result<(), eframe::Error> {
    // 设置日志级别为 debug 以查看诊断信息
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Debug)
        .init();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1000.0, 700.0]),
        ..Default::default()
    };

    eframe::run_native(
        "文件编辑器字体一致性测试",
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

        // 创建一个测试文件内容，包含多种情况
        let test_content = r#"// 这是一个测试文件，用于验证字体、行间距、行号显示的一致性
// 第2行：包含中文和英文混合内容
fn main() {
    println!("Hello, world! 你好世界！");
    let very_long_line_that_should_wrap_when_word_wrap_is_enabled_and_should_show_horizontal_scroll_when_disabled = "This is a very long string to test wrapping behavior";
    
    // 第7行：测试不同字符类型
    let numbers = 1234567890;
    let symbols = !@#$%^&*()_+-=[]{}|;':\",./<>?;
    let chinese = "这是一行包含中文字符的文本，用来测试中文字符的显示效果";
    
    // 第12行：测试代码缩进
    if true {
        if true {
            if true {
                println!("深层嵌套的代码块");
            }
        }
    }
    
    // 第20行：测试空行和注释
    
    /* 多行注释
       第二行注释
       第三行注释 */
    
    // 第26行：测试各种语法元素
    struct TestStruct {
        field1: String,
        field2: i32,
        field3: Vec<String>,
    }
    
    impl TestStruct {
        fn new() -> Self {
            Self {
                field1: "test".to_string(),
                field2: 42,
                field3: vec!["a".to_string(), "b".to_string()],
            }
        }
    }
}

// 第42行：文件结束
"#;

        // 创建一个临时文件路径
        let test_file_path = PathBuf::from("test_font_consistency.rs");

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
            ui.heading("文件编辑器字体、行间距、行号显示一致性测试");
            
            ui.horizontal(|ui| {
                ui.label("当前设置:");
                ui.label(format!("字体大小: {:.1}px", self.file_editor_state.ui_state.editor_font_size));
                ui.label(format!("行高因子: {:.1}", self.file_editor_state.settings.line_height_factor));
                ui.label(format!("计算行高: {:.1}px", 
                    self.file_editor_state.ui_state.editor_font_size * self.file_editor_state.settings.line_height_factor));
            });
            
            ui.horizontal(|ui| {
                ui.label("折行状态:");
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
            
            ui.horizontal(|ui| {
                ui.label("字体大小:");
                if ui.button("-").clicked() {
                    self.file_editor_state.ui_state.editor_font_size = (self.file_editor_state.ui_state.editor_font_size - 1.0).max(8.0);
                }
                ui.label(format!("{:.0}", self.file_editor_state.ui_state.editor_font_size));
                if ui.button("+").clicked() {
                    self.file_editor_state.ui_state.editor_font_size = (self.file_editor_state.ui_state.editor_font_size + 1.0).min(24.0);
                }
                
                ui.separator();
                
                ui.label("行高因子:");
                if ui.button("↓").clicked() {
                    self.file_editor_state.settings.line_height_factor = (self.file_editor_state.settings.line_height_factor - 0.1).max(0.8);
                }
                ui.label(format!("{:.1}", self.file_editor_state.settings.line_height_factor));
                if ui.button("↑").clicked() {
                    self.file_editor_state.settings.line_height_factor = (self.file_editor_state.settings.line_height_factor + 0.1).min(3.0);
                }
            });
            
            ui.separator();
            
            ui.label("测试要点：");
            ui.label("• 切换折行模式时，字体大小应保持一致");
            ui.label("• 切换折行模式时，行间距应保持一致");
            ui.label("• 切换折行模式时，行号显示样式应保持一致");
            ui.label("• 行号宽度应根据总行数自动调整");
            ui.label("• 调整字体大小和行高因子时，两种模式应同步变化");
            
            ui.separator();
            
            // 渲染文件编辑器
            ifile_editor::ui::main_ui::render_file_editor(ui, &mut self.file_editor_state);
        });
    }
}
