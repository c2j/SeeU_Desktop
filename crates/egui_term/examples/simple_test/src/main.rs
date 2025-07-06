use eframe::egui::{self, Widget};
use std::sync::mpsc;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([800.0, 600.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Simple egui_term Test",
        options,
        Box::new(|cc| Ok(Box::new(SimpleTermApp::new(cc.egui_ctx.clone())))),
    )
}

struct SimpleTermApp {
    terminal_backend: Option<egui_term::TerminalBackend>,
    pty_event_receiver: mpsc::Receiver<(u64, egui_term::PtyEvent)>,
}

impl SimpleTermApp {
    fn new(ctx: egui::Context) -> Self {
        let (pty_event_sender, pty_event_receiver) = mpsc::channel();
        Self {
            terminal_backend: None,
            pty_event_receiver,
        }
    }
}

impl eframe::App for SimpleTermApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 处理 PTY 事件
        while let Ok((id, event)) = self.pty_event_receiver.try_recv() {
            println!("Received PTY event for terminal {}: {:?}", id, event);
            ctx.request_repaint();
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("egui_term Simple Test");

            ui.separator();

            if ui.button("Create Terminal").clicked() {
                let (pty_event_sender, pty_event_receiver) = mpsc::channel();
                self.pty_event_receiver = pty_event_receiver;

                let settings = egui_term::BackendSettings::default();
                match egui_term::TerminalBackend::new(1, ctx.clone(), pty_event_sender, settings) {
                    Ok(backend) => {
                        self.terminal_backend = Some(backend);
                        println!("Terminal created successfully!");
                    }
                    Err(e) => {
                        println!("Failed to create terminal: {}", e);
                    }
                }
            }

            if let Some(ref mut backend) = self.terminal_backend {
                ui.separator();
                ui.label("Terminal Widget:");

                // 显示终端组件
                let available_size = ui.available_size();
                let terminal_size = egui::Vec2::new(available_size.x, available_size.y - 50.0);

                ui.allocate_ui_with_layout(
                    terminal_size,
                    egui::Layout::top_down(egui::Align::LEFT),
                    |ui| {
                        let terminal_view = egui_term::TerminalView::new(ui, backend);
                        terminal_view.ui(ui);
                    },
                );
            } else {
                ui.label("Click 'Create Terminal' to start");
            }
        });
    }
}
