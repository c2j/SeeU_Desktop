//! Simple Embedded WebView with egui
//!
//! This example demonstrates a simple, safe approach to WebView embedding
//! based on the proven multi_window.rs pattern

use eframe::egui;
use web_view::*;

/// Simple application state
struct SimpleWebViewApp {
    /// URL input
    url_input: String,
    /// Status message
    status_message: String,
    /// Single WebView instance
    webview: Option<WebView<'static, ()>>,
    /// Show controls
    show_controls: bool,
}

impl Default for SimpleWebViewApp {
    fn default() -> Self {
        Self {
            url_input: "https://www.rust-lang.org".to_string(),
            status_message: "Ready to create WebView".to_string(),
            webview: None,
            show_controls: true,
        }
    }
}

impl SimpleWebViewApp {
    fn create_webview(&mut self) {
        // Close existing WebView first
        if self.webview.is_some() {
            self.webview = None;
            self.status_message = "Closed previous WebView".to_string();
        }

        if self.url_input.is_empty() {
            self.status_message = "Please enter a URL".to_string();
            return;
        }

        let url = if !self.url_input.starts_with("http://") && !self.url_input.starts_with("https://") {
            format!("https://{}", self.url_input)
        } else {
            self.url_input.clone()
        };

        // Create WebView using the safe build() method (same as multi_window.rs)
        match web_view::builder()
            .title("Simple Embedded WebView")
            .content(Content::Url(&url))
            .size(800, 600)
            .resizable(true)
            .debug(false)
            .user_data(())
            .invoke_handler(|_webview, arg| {
                println!("WebView message: {}", arg);
                Ok(())
            })
            .build()
        {
            Ok(webview) => {
                self.webview = Some(webview);
                self.status_message = format!("Created WebView for: {}", url);
            }
            Err(e) => {
                self.status_message = format!("Error creating WebView: {:?}", e);
            }
        }
    }

    fn step_webview(&mut self) {
        if let Some(webview) = &mut self.webview {
            // Use the same step pattern as multi_window.rs
            match webview.step() {
                Some(Ok(_)) => {
                    // WebView is still running
                }
                Some(Err(e)) => {
                    self.status_message = format!("WebView error: {:?}", e);
                    self.webview = None;
                }
                None => {
                    // WebView was closed
                    self.webview = None;
                    self.status_message = "WebView was closed".to_string();
                }
            }
        }
    }

    fn close_webview(&mut self) {
        if self.webview.is_some() {
            self.webview = None;
            self.status_message = "WebView closed manually".to_string();
        }
    }

    fn eval_js(&mut self, js: &str) {
        if let Some(webview) = &mut self.webview {
            match webview.eval(js) {
                Ok(_) => {
                    self.status_message = format!("Executed JS: {}", js);
                }
                Err(e) => {
                    self.status_message = format!("JS error: {:?}", e);
                }
            }
        }
    }
}

impl eframe::App for SimpleWebViewApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Step WebView using the same pattern as multi_window.rs
        self.step_webview();

        // Main UI
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("🔗 Simple Embedded WebView");
            ui.separator();

            // URL input section
            ui.horizontal(|ui| {
                ui.label("URL:");
                let response = ui.add(
                    egui::TextEdit::singleline(&mut self.url_input)
                        .desired_width(400.0)
                        .hint_text("Enter URL (e.g., https://www.rust-lang.org)")
                );

                if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    self.create_webview();
                }

                if ui.button("🔗 Create WebView").clicked() {
                    self.create_webview();
                }

                if self.webview.is_some() && ui.button("❌ Close").clicked() {
                    self.close_webview();
                }
            });

            ui.add_space(10.0);

            // Quick navigation buttons
            ui.horizontal(|ui| {
                ui.label("Quick load:");
                if ui.button("🦀 Rust").clicked() {
                    self.url_input = "https://www.rust-lang.org".to_string();
                    self.create_webview();
                }
                if ui.button("📖 egui").clicked() {
                    self.url_input = "https://github.com/emilk/egui".to_string();
                    self.create_webview();
                }
                if ui.button("🌍 Wikipedia").clicked() {
                    self.url_input = "https://en.wikipedia.org".to_string();
                    self.create_webview();
                }
            });

            ui.separator();

            // Status section
            ui.horizontal(|ui| {
                ui.label("Status:");
                if self.webview.is_some() {
                    ui.colored_label(egui::Color32::GREEN, "✅ WebView Active");
                } else {
                    ui.colored_label(egui::Color32::GRAY, "⭕ No WebView");
                }
            });

            ui.label(&self.status_message);

            ui.add_space(10.0);

            // Controls
            ui.checkbox(&mut self.show_controls, "Show WebView Controls");

            if self.show_controls && self.webview.is_some() {
                ui.group(|ui| {
                    ui.label("🎛️ WebView Controls");
                    ui.horizontal(|ui| {
                        if ui.button("💬 Alert").clicked() {
                            self.eval_js("alert('Hello from egui!');");
                        }
                        if ui.button("🎨 Blue Background").clicked() {
                            self.eval_js("document.body.style.backgroundColor = '#e6f3ff';");
                        }
                        if ui.button("📄 Change Title").clicked() {
                            self.eval_js("document.title = 'Modified by egui';");
                        }
                        if ui.button("🔄 Reload").clicked() {
                            self.eval_js("location.reload();");
                        }
                    });
                });
            }

            ui.add_space(10.0);

            // Information section
            ui.group(|ui| {
                ui.label("ℹ️ About this simple embedded example:");
                ui.label("• Uses the same safe pattern as multi_window.rs");
                ui.label("• Single WebView instance for simplicity");
                ui.label("• WebView::build() + step() approach");
                ui.label("• JavaScript evaluation support");
                ui.label("• No threading - all operations in main thread");
            });

            ui.add_space(10.0);

            // Technical details
            ui.collapsing("🔧 Technical Details", |ui| {
                ui.label("Safe Architecture (based on multi_window.rs):");
                ui.label("• WebView::build() instead of run()");
                ui.label("• Manual step() calls in egui update loop");
                ui.label("• Single WebView for maximum stability");
                ui.label("• JavaScript evaluation via eval()");
                ui.label("• Proper WebView cleanup");
                
                ui.add_space(5.0);
                ui.label("Safety measures:");
                ui.label("• No thread spawning");
                ui.label("• Proven multi_window.rs pattern");
                ui.label("• Resource cleanup on close");
                ui.label("• Main thread operation only");
            });
        });

        // Request repaint for WebView stepping (same as multi_window.rs loop)
        if self.webview.is_some() {
            ctx.request_repaint_after(std::time::Duration::from_millis(16)); // ~60 FPS
        }
    }
}

fn main() -> Result<(), eframe::Error> {
    env_logger::init();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([900.0, 700.0])
            .with_title("Simple Embedded WebView"),
        ..Default::default()
    };

    eframe::run_native(
        "Simple Embedded WebView Example",
        options,
        Box::new(|_cc| Ok(Box::new(SimpleWebViewApp::default()))),
    )
}
