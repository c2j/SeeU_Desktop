use eframe::egui;
use egui_term::{TerminalView, TerminalBackend, BackendSettings, PtyEvent, FontSettings, TerminalFont};
use std::sync::mpsc;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0])
            .with_title("egui_term 中文字符测试"),
        ..Default::default()
    };

    eframe::run_native(
        "中文终端测试",
        options,
        Box::new(|cc| Ok(Box::new(ChineseTestApp::new(cc)))),
    )
}

struct ChineseTestApp {
    terminal_backend: TerminalBackend,
    pty_receiver: mpsc::Receiver<(u64, PtyEvent)>,
}

impl ChineseTestApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let system_shell = std::env::var("SHELL")
            .unwrap_or_else(|_| "/bin/bash".to_string());

        let (pty_sender, pty_receiver) = mpsc::channel();
        let terminal_backend = TerminalBackend::new(
            0,
            cc.egui_ctx.clone(),
            pty_sender,
            BackendSettings {
                shell: system_shell,
                ..Default::default()
            },
        )
        .expect("Failed to create terminal backend");

        Self {
            terminal_backend,
            pty_receiver,
        }
    }
}

impl eframe::App for ChineseTestApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 处理 PTY 事件
        if let Ok((_, PtyEvent::Exit)) = self.pty_receiver.try_recv() {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }

        // 顶部面板 - 显示测试说明
        egui::TopBottomPanel::top("instructions").show(ctx, |ui| {
            ui.heading("🇨🇳 egui_term 中文字符支持测试");
            ui.separator();
            ui.horizontal(|ui| {
                ui.label("测试项目：");
                ui.label("• 输入中文字符");
                ui.label("• 显示中文文本");
                ui.label("• 中英文混合");
            });
            ui.horizontal(|ui| {
                ui.label("测试命令：");
                ui.code("echo '你好世界！Hello World!'");
                ui.code("ls -la");
                ui.code("cat /etc/passwd");
            });
        });

        // 中央面板 - 终端显示
        egui::CentralPanel::default().show(ctx, |ui| {
            let terminal = TerminalView::new(ui, &mut self.terminal_backend)
                .set_focus(true)
                .set_font(TerminalFont::new(FontSettings {
                    font_type: egui::FontId::monospace(14.0),
                }))
                .set_size(egui::Vec2::new(
                    ui.available_width(),
                    ui.available_height(),
                ));

            ui.add(terminal);
        });
    }
}
