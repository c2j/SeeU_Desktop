//! Safe Embedded WebView Example
//!
//! This example demonstrates a safer approach to embedding WebView with egui,
//! using single-threaded WebView operations to avoid trace traps.

use eframe::egui;
use web_view::*;

/// Application state for embedded WebView
struct EmbeddedWebViewApp {
    /// URL input
    url_input: String,
    /// Status message
    status_message: String,
    /// WebView instance (optional)
    webview: Option<WebView<'static, ()>>,
    /// Whether to show WebView controls
    show_controls: bool,
}

impl Default for EmbeddedWebViewApp {
    fn default() -> Self {
        Self {
            url_input: "https://www.rust-lang.org".to_string(),
            status_message: "Ready to create WebView".to_string(),
            webview: None,
            show_controls: true,
        }
    }
}

impl EmbeddedWebViewApp {
    fn create_webview(&mut self) {
        if self.url_input.is_empty() {
            self.status_message = "Please enter a URL".to_string();
            return;
        }

        let url = if !self.url_input.starts_with("http://") && !self.url_input.starts_with("https://") {
            format!("https://{}", self.url_input)
        } else {
            self.url_input.clone()
        };

        // Try to create WebView using the safer build() method instead of run()
        match web_view::builder()
            .title("Embedded WebView")
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
                self.status_message = format!("WebView created for: {}", url);
            }
            Err(e) => {
                self.status_message = format!("Error creating WebView: {:?}", e);
                // Fallback to system browser
                self.open_in_system_browser(&url);
            }
        }
    }

    fn open_in_system_browser(&mut self, url: &str) {
        let result = self.open_url_in_system_browser(url);
        match result {
            Ok(_) => {
                self.status_message = format!("Opened in system browser: {}", url);
            }
            Err(e) => {
                self.status_message = format!("Error opening URL: {}", e);
            }
        }
    }

    fn open_url_in_system_browser(&self, url: &str) -> Result<(), Box<dyn std::error::Error>> {
        #[cfg(target_os = "macos")]
        {
            std::process::Command::new("open").arg(url).spawn()?;
        }

        #[cfg(target_os = "linux")]
        {
            std::process::Command::new("xdg-open").arg(url).spawn()?;
        }

        #[cfg(target_os = "windows")]
        {
            std::process::Command::new("cmd")
                .args(&["/C", "start", url])
                .spawn()?;
        }

        Ok(())
    }

    fn step_webview(&mut self) {
        if let Some(webview) = &mut self.webview {
            // Try to step the WebView safely
            match webview.step() {
                Some(_) => {
                    // WebView is still running
                }
                None => {
                    // WebView was closed
                    self.webview = None;
                    self.status_message = "WebView was closed".to_string();
                }
            }
        }
    }
}

impl eframe::App for EmbeddedWebViewApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Step WebView if it exists
        self.step_webview();

        // Main UI
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("🔗 Safe Embedded WebView Example");
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

                if ui.button("🌐 Open in Browser").clicked() {
                    let url = if !self.url_input.starts_with("http://") && !self.url_input.starts_with("https://") {
                        format!("https://{}", self.url_input)
                    } else {
                        self.url_input.clone()
                    };
                    self.open_in_system_browser(&url);
                }
            });

            ui.add_space(10.0);

            // Quick navigation buttons
            ui.horizontal(|ui| {
                ui.label("Quick links:");
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

            ui.add_space(5.0);
            ui.label(&self.status_message);

            ui.add_space(10.0);

            // Controls
            ui.checkbox(&mut self.show_controls, "Show WebView Controls");

            if self.show_controls && self.webview.is_some() {
                ui.group(|ui| {
                    ui.label("🎛️ WebView Controls");
                    ui.horizontal(|ui| {
                        if ui.button("📄 Eval JS").clicked() {
                            if let Some(webview) = &mut self.webview {
                                let _ = webview.eval("alert('Hello from egui!');");
                            }
                        }
                        if ui.button("🎨 Change Color").clicked() {
                            if let Some(webview) = &mut self.webview {
                                let _ = webview.eval("document.body.style.backgroundColor = '#f0f0f0';");
                            }
                        }
                        if ui.button("❌ Close WebView").clicked() {
                            self.webview = None;
                            self.status_message = "WebView closed manually".to_string();
                        }
                    });
                });
            }

            ui.add_space(10.0);

            // Information section
            ui.group(|ui| {
                ui.label("ℹ️ About this safe embedded example:");
                ui.label("• Attempts to create embedded WebView using build() instead of run()");
                ui.label("• Falls back to system browser if WebView creation fails");
                ui.label("• Uses single-threaded WebView operations");
                ui.label("• Includes WebView step() calls for safe operation");
                ui.label("• Compatible with egui 0.28.1");
            });

            ui.add_space(10.0);

            // Technical details
            ui.collapsing("🔧 Technical Details", |ui| {
                ui.label("Safe Architecture:");
                ui.label("• Uses WebView::build() instead of run()");
                ui.label("• Single-threaded operation with step() calls");
                ui.label("• Graceful fallback to system browser");
                ui.label("• JavaScript evaluation support");
                ui.label("• Manual WebView lifecycle management");
                
                ui.add_space(5.0);
                ui.label("Safety measures:");
                ui.label("• No thread spawning for WebView");
                ui.label("• Proper error handling");
                ui.label("• Fallback mechanisms");
                ui.label("• Resource cleanup");
            });
        });

        // Request repaint for WebView stepping
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
            .with_title("Safe Embedded WebView Example"),
        ..Default::default()
    };

    eframe::run_native(
        "Safe Embedded WebView Example",
        options,
        Box::new(|_cc| Ok(Box::new(EmbeddedWebViewApp::default()))),
    )
}
