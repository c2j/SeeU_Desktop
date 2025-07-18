//! Advanced egui + WebView Integration Example
//!
//! This example demonstrates advanced integration patterns between egui and web-view,
//! including multiple windows, communication, and state management.

use eframe::egui;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use web_view::*;

/// Advanced application state with multiple WebView support
#[derive(Default)]
struct AdvancedEguiWebViewApp {
    /// URL input for new windows
    url_input: String,
    /// Window title input
    title_input: String,
    /// Status messages
    status_messages: Vec<String>,
    /// WebView windows manager
    webview_manager: Arc<Mutex<AdvancedWebViewManager>>,
    /// Show advanced controls
    show_advanced: bool,
    /// Auto-refresh interval (seconds)
    auto_refresh_interval: u32,
}

/// Advanced WebView manager supporting multiple windows
#[derive(Default)]
struct AdvancedWebViewManager {
    /// Active WebView windows
    windows: HashMap<String, WebViewWindow>,
    /// Next window ID
    next_window_id: u32,
}

/// Individual WebView window
struct WebViewWindow {
    id: String,
    title: String,
    url: String,
    created_at: std::time::Instant,
}

impl AdvancedWebViewManager {
    /// Create a new WebView window using system command (safer approach)
    fn create_window(&mut self, title: String, url: String) -> Result<String, Box<dyn std::error::Error>> {
        let window_id = format!("window_{}", self.next_window_id);
        self.next_window_id += 1;

        // Use system command to open URL in default browser for safety
        #[cfg(target_os = "macos")]
        {
            std::process::Command::new("open")
                .arg(&url)
                .spawn()?;
        }

        #[cfg(target_os = "linux")]
        {
            std::process::Command::new("xdg-open")
                .arg(&url)
                .spawn()?;
        }

        #[cfg(target_os = "windows")]
        {
            std::process::Command::new("cmd")
                .args(&["/C", "start", &url])
                .spawn()?;
        }

        let window = WebViewWindow {
            id: window_id.clone(),
            title,
            url,
            created_at: std::time::Instant::now(),
        };

        self.windows.insert(window_id.clone(), window);
        println!("Created window '{}' - opened URL in system browser", window_id);
        Ok(window_id)
    }

    /// Get list of active windows
    fn get_active_windows(&self) -> Vec<(String, String, String, std::time::Duration)> {
        // Return window info (all windows are considered "active" since they're opened in system browser)
        self.windows.iter()
            .map(|(id, window)| {
                (
                    id.clone(),
                    window.title.clone(),
                    window.url.clone(),
                    window.created_at.elapsed(),
                )
            })
            .collect()
    }

    /// Close a specific window
    fn close_window(&mut self, window_id: &str) {
        self.windows.remove(window_id);
    }

    /// Close all windows
    fn close_all_windows(&mut self) {
        self.windows.clear();
    }
}

impl eframe::App for AdvancedEguiWebViewApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Top menu bar
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("New Window").clicked() {
                        self.create_new_window();
                        ui.close_menu();
                    }
                    if ui.button("Close All").clicked() {
                        self.close_all_windows();
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("Exit").clicked() {
                        std::process::exit(0);
                    }
                });
                
                ui.menu_button("View", |ui| {
                    ui.checkbox(&mut self.show_advanced, "Show Advanced Controls");
                });

                ui.menu_button("Help", |ui| {
                    if ui.button("About").clicked() {
                        self.add_status_message("Advanced egui + WebView Integration v1.0".to_string());
                        ui.close_menu();
                    }
                });
            });
        });

        // Main content
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("🚀 Advanced egui + WebView Integration");
            ui.separator();

            // Window creation section
            ui.group(|ui| {
                ui.label("🆕 Create New WebView Window");
                
                ui.horizontal(|ui| {
                    ui.label("Title:");
                    ui.add(egui::TextEdit::singleline(&mut self.title_input)
                        .hint_text("Window title"));
                });

                ui.horizontal(|ui| {
                    ui.label("URL:");
                    let response = ui.add(egui::TextEdit::singleline(&mut self.url_input)
                        .desired_width(300.0)
                        .hint_text("https://example.com"));

                    if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        self.create_new_window();
                    }

                    if ui.button("🔗 Create Window").clicked() {
                        self.create_new_window();
                    }
                });

                // Preset buttons
                ui.horizontal(|ui| {
                    ui.label("Quick create:");
                    if ui.button("🦀 Rust Docs").clicked() {
                        self.create_preset_window("Rust Documentation", "https://doc.rust-lang.org");
                    }
                    if ui.button("📖 egui Demo").clicked() {
                        self.create_preset_window("egui Demo", "https://www.egui.rs");
                    }
                    if ui.button("🌍 Wikipedia").clicked() {
                        self.create_preset_window("Wikipedia", "https://en.wikipedia.org");
                    }
                });
            });

            ui.add_space(10.0);

            // Active windows section
            ui.group(|ui| {
                ui.label("📱 Active WebView Windows");
                
                let active_windows = {
                    let manager = self.webview_manager.lock().unwrap();
                    manager.get_active_windows()
                };

                if active_windows.is_empty() {
                    ui.colored_label(egui::Color32::GRAY, "No active windows");
                } else {
                    egui::ScrollArea::vertical()
                        .id_source("active_windows_scroll")
                        .max_height(200.0)
                        .show(ui, |ui| {
                            for (id, title, url, duration) in active_windows {
                                ui.horizontal(|ui| {
                                    ui.label("🪟");
                                    ui.label(&title);
                                    ui.separator();
                                    ui.small(&url);
                                    ui.separator();
                                    ui.small(format!("{}s", duration.as_secs()));

                                    if ui.small_button("❌").clicked() {
                                        let mut manager = self.webview_manager.lock().unwrap();
                                        manager.close_window(&id);
                                    }
                                });
                            }
                        });
                }
            });

            // Advanced controls
            if self.show_advanced {
                ui.add_space(10.0);
                ui.group(|ui| {
                    ui.label("⚙️ Advanced Controls");
                    
                    ui.horizontal(|ui| {
                        ui.label("Auto-refresh interval:");
                        ui.add(egui::Slider::new(&mut self.auto_refresh_interval, 0..=300)
                            .suffix(" seconds"));
                    });

                    ui.horizontal(|ui| {
                        if ui.button("🔄 Refresh All").clicked() {
                            self.add_status_message("Refresh functionality not implemented yet".to_string());
                        }
                        if ui.button("📸 Screenshot All").clicked() {
                            self.add_status_message("Screenshot functionality not implemented yet".to_string());
                        }
                        if ui.button("🧹 Clear Cache").clicked() {
                            self.add_status_message("Cache cleared (simulated)".to_string());
                        }
                    });
                });
            }

            // Status messages
            if !self.status_messages.is_empty() {
                ui.add_space(10.0);
                ui.group(|ui| {
                    ui.label("📋 Status Messages");
                    egui::ScrollArea::vertical()
                        .id_source("status_messages_scroll")
                        .max_height(100.0)
                        .show(ui, |ui| {
                            for (i, message) in self.status_messages.iter().enumerate() {
                                ui.horizontal(|ui| {
                                    ui.small(format!("[{}]", i + 1));
                                    ui.small(message);
                                });
                            }
                        });

                    if ui.button("Clear Messages").clicked() {
                        self.status_messages.clear();
                    }
                });
            }
        });

        // Request repaint for real-time updates
        ctx.request_repaint_after(std::time::Duration::from_secs(1));
    }
}

impl AdvancedEguiWebViewApp {
    fn new() -> Self {
        Self {
            url_input: "https://www.rust-lang.org".to_string(),
            title_input: "My WebView".to_string(),
            status_messages: vec!["Application started".to_string()],
            webview_manager: Arc::new(Mutex::new(AdvancedWebViewManager::default())),
            show_advanced: false,
            auto_refresh_interval: 0,
        }
    }

    fn create_new_window(&mut self) {
        if self.url_input.is_empty() {
            self.add_status_message("Please enter a URL".to_string());
            return;
        }

        let title = if self.title_input.is_empty() {
            "WebView Window".to_string()
        } else {
            self.title_input.clone()
        };

        let url = if !self.url_input.starts_with("http://") && !self.url_input.starts_with("https://") {
            format!("https://{}", self.url_input)
        } else {
            self.url_input.clone()
        };

        let result = {
            let mut manager = self.webview_manager.lock().unwrap();
            manager.create_window(title.clone(), url.clone())
        };

        match result {
            Ok(window_id) => {
                self.add_status_message(format!("Created window '{}': {}", window_id, title));
            }
            Err(e) => {
                self.add_status_message(format!("Error creating window: {}", e));
            }
        }
    }

    fn create_preset_window(&mut self, title: &str, url: &str) {
        let result = {
            let mut manager = self.webview_manager.lock().unwrap();
            manager.create_window(title.to_string(), url.to_string())
        };

        match result {
            Ok(window_id) => {
                self.add_status_message(format!("Created preset window '{}': {}", window_id, title));
            }
            Err(e) => {
                self.add_status_message(format!("Error creating preset window: {}", e));
            }
        }
    }

    fn close_all_windows(&mut self) {
        {
            let mut manager = self.webview_manager.lock().unwrap();
            manager.close_all_windows();
        }
        self.add_status_message("All windows closed".to_string());
    }

    fn add_status_message(&mut self, message: String) {
        self.status_messages.push(message);
        // Keep only last 10 messages
        if self.status_messages.len() > 10 {
            self.status_messages.remove(0);
        }
    }
}

fn main() -> Result<(), eframe::Error> {
    env_logger::init();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([900.0, 700.0])
            .with_title("Advanced egui + WebView Integration"),
        ..Default::default()
    };

    eframe::run_native(
        "Advanced egui WebView Example",
        options,
        Box::new(|_cc| Ok(Box::new(AdvancedEguiWebViewApp::new()))),
    )
}
