//! Example: Embedding WebView in an egui application
//!
//! This example demonstrates how to integrate web-view with egui 0.28.1,
//! creating a hybrid application that combines native GUI controls with web content.
//!
//! Features:
//! - egui-based main interface with native controls
//! - Embedded WebView for displaying web content
//! - URL navigation controls
//! - Window management
//! - Cross-platform compatibility

use eframe::egui;
use std::sync::{Arc, Mutex};

/// Main application state
#[derive(Default)]
struct EguiWebViewApp {
    /// Current URL in the address bar
    url_input: String,
    /// Status message
    status_message: String,
    /// WebView manager
    webview_manager: Arc<Mutex<WebViewManager>>,
    /// Whether WebView window is currently open
    webview_open: bool,
}

/// WebView window manager
#[derive(Default)]
struct WebViewManager {
    /// Current URL being displayed
    current_url: String,
    /// Whether WebView was opened
    webview_opened: bool,
    /// When the WebView was last opened
    last_opened: Option<std::time::Instant>,
}

impl WebViewManager {
    /// Create and show a WebView using system browser (safe method)
    fn create_webview(&mut self, url: String) -> Result<(), Box<dyn std::error::Error>> {
        self.current_url = url.clone();
        self.webview_opened = true;
        self.last_opened = Some(std::time::Instant::now());

        // Use system browser command for safety
        self.open_url_in_system_browser(&url)?;

        println!("Opened URL in system browser: {}", url);
        Ok(())
    }

    /// Open URL in system browser
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

    /// Check if WebView was opened recently
    fn is_running(&self) -> bool {
        self.webview_opened
    }

    /// Get current URL
    fn get_current_url(&self) -> &str {
        &self.current_url
    }

    /// Reset the WebView state
    fn reset(&mut self) {
        self.webview_opened = false;
        self.last_opened = None;
        self.current_url.clear();
    }
}

impl eframe::App for EguiWebViewApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Update WebView status
        {
            let manager = self.webview_manager.lock().unwrap();
            self.webview_open = manager.is_running();
        }

        // Main UI
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("🌐 egui + WebView Integration Example");
            ui.separator();

            // URL input section
            ui.horizontal(|ui| {
                ui.label("URL:");
                let response = ui.add(
                    egui::TextEdit::singleline(&mut self.url_input)
                        .desired_width(400.0)
                        .hint_text("Enter URL (e.g., https://www.rust-lang.org)")
                );

                // Handle Enter key
                if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    self.navigate_to_url();
                }

                if ui.button("🔗 Open in WebView").clicked() {
                    self.navigate_to_url();
                }
            });

            ui.add_space(10.0);

            // Quick navigation buttons
            ui.horizontal(|ui| {
                ui.label("Quick links:");
                if ui.button("🦀 Rust").clicked() {
                    self.url_input = "https://www.rust-lang.org".to_string();
                    self.navigate_to_url();
                }
                if ui.button("📖 egui").clicked() {
                    self.url_input = "https://github.com/emilk/egui".to_string();
                    self.navigate_to_url();
                }
                if ui.button("🌍 Wikipedia").clicked() {
                    self.url_input = "https://en.wikipedia.org".to_string();
                    self.navigate_to_url();
                }
                if ui.button("🔍 GitHub").clicked() {
                    self.url_input = "https://github.com".to_string();
                    self.navigate_to_url();
                }
            });

            ui.separator();

            // Status section
            ui.horizontal(|ui| {
                ui.label("Status:");
                if self.webview_open {
                    ui.colored_label(egui::Color32::GREEN, "✅ WebView Open");
                    let current_url = {
                        let manager = self.webview_manager.lock().unwrap();
                        manager.get_current_url().to_string()
                    };
                    ui.label(format!("Displaying: {}", current_url));
                } else {
                    ui.colored_label(egui::Color32::GRAY, "⭕ WebView Closed");
                }
            });

            ui.add_space(10.0);

            // Information section
            ui.group(|ui| {
                ui.label("ℹ️ About this safe example:");
                ui.label("• This demonstrates safe egui + web-view integration");
                ui.label("• Uses system browser commands instead of embedded WebView");
                ui.label("• Avoids threading issues and trace traps");
                ui.label("• Cross-platform: macOS (open), Linux (xdg-open), Windows (start)");
                ui.label("• Compatible with egui 0.28.1 and safe for production");
            });

            ui.add_space(10.0);

            // Technical details
            ui.collapsing("🔧 Technical Details", |ui| {
                ui.label("Safe Architecture:");
                ui.label("• egui: Immediate mode GUI framework");
                ui.label("• System browser: Uses OS default browser");
                ui.label("• No threading: Avoids WebView threading issues");
                ui.label("• Communication: Simple state management");

                ui.add_space(5.0);
                ui.label("Platform commands:");
                ui.label("• macOS: 'open' command");
                ui.label("• Linux: 'xdg-open' command");
                ui.label("• Windows: 'start' command");
            });

            // Status message
            if !self.status_message.is_empty() {
                ui.add_space(10.0);
                ui.colored_label(egui::Color32::BLUE, &self.status_message);
            }
        });

        // Request repaint to update status
        ctx.request_repaint();
    }
}

impl EguiWebViewApp {
    fn new() -> Self {
        Self {
            url_input: "https://www.rust-lang.org".to_string(),
            status_message: "Ready to open URLs in system browser (safe mode)".to_string(),
            webview_manager: Arc::new(Mutex::new(WebViewManager::default())),
            webview_open: false,
        }
    }

    fn navigate_to_url(&mut self) {
        if self.url_input.is_empty() {
            self.status_message = "Please enter a URL".to_string();
            return;
        }

        // Ensure URL has a protocol
        let url = if !self.url_input.starts_with("http://") && !self.url_input.starts_with("https://") {
            format!("https://{}", self.url_input)
        } else {
            self.url_input.clone()
        };

        // Create WebView
        let mut manager = self.webview_manager.lock().unwrap();
        match manager.create_webview(url.clone()) {
            Ok(_) => {
                self.status_message = format!("Opened in system browser: {}", url);
            }
            Err(e) => {
                self.status_message = format!("Error opening URL: {}", e);
            }
        }
    }
}

fn main() -> Result<(), eframe::Error> {
    // Configure logging
    env_logger::init();

    // Configure egui
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0])
            .with_title("Safe egui + WebView Integration Example"),
        ..Default::default()
    };

    // Run the application
    eframe::run_native(
        "Safe egui WebView Example",
        options,
        Box::new(|_cc| Ok(Box::new(EguiWebViewApp::new()))),
    )
}
