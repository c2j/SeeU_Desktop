// use eframe::egui;

/// Terminal module state
pub struct TerminalState {
    input: String,
    output: Vec<String>,
}

impl Default for TerminalState {
    fn default() -> Self {
        Self {
            input: String::new(),
            output: vec!["Welcome to SeeU Terminal".to_string()],
        }
    }
}

// /// Render the terminal module
// pub fn render_terminal(ui: &mut egui::Ui) {
//     // 获取可用高度
//     let available_height = ui.available_height();

//     // 创建一个垂直布局容器，确保内容撑满高度
//     egui::containers::Frame::none()
//         .fill(ui.style().visuals.window_fill)
//         .show(ui, |ui| {
//             // 设置最小高度，确保撑满可用空间
//             ui.set_min_height(available_height);

//             ui.vertical(|ui| {
//                 ui.label("终端模块尚未完全实现");

//                 // 计算输出区域高度（减去标题、标签和输入框的高度）
//                 let output_height = available_height - 80.0;

//                 // Output area
//                 egui::ScrollArea::vertical()
//                     .stick_to_bottom(true)
//                     .auto_shrink([false; 2])
//                     .max_height(output_height)
//                     .show(ui, |ui| {
//                         ui.add(egui::TextEdit::multiline(&mut "$ ls -la\ndrwxr-xr-x  4 user  staff  128 May 18 21:18 .\ndrwxr-xr-x  8 user  staff  256 May 18 20:05 ..\ndrwxr-xr-x  9 user  staff  288 May 18 21:18 .git\n-rw-r--r--  1 user  staff    8 May 18 21:20 .gitignore\n-rw-r--r--  1 user  staff   86 May 18 21:20 Cargo.toml\ndrwxr-xr-x  3 user  staff   96 May 18 20:30 requirement\ndrwxr-xr-x  3 user  staff   96 May 18 21:20 src".to_string())
//                             .desired_width(f32::INFINITY)
//                             .font(egui::TextStyle::Monospace)
//                             .interactive(false));
//                     });

//                 // Input area
//                 ui.horizontal(|ui| {
//                     ui.label("$");
//                     let mut input = String::new();
//                     let response = ui.add(
//                         egui::TextEdit::singleline(&mut input)
//                             .hint_text("输入命令...")
//                             .desired_width(ui.available_width())
//                     );

//                     if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
//                         // TODO: Execute command
//                         log::info!("Command: {}", input);
//                     }
//                 });
//             });
//         });
// }