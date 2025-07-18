//! Safe egui + WebView Integration Example
//!
//! This example demonstrates a safe integration between egui and web-view,
//! using system browser commands to avoid threading issues.

use eframe::egui;
use std::collections::HashMap;

/// Safe application state
#[derive(Default)]
struct SafeEguiWebViewApp {
    /// URL input for new windows
    url_input: String,
    /// Window title input
    title_input: String,
    /// Status messages
    status_messages: Vec<String>,
    /// Created windows (for tracking)
    created_windows: HashMap<String, WindowInfo>,
    /// Next window ID
    next_window_id: u32,
}

/// Window information for tracking
struct WindowInfo {
    title: String,
    url: String,
    created_at: std::time::Instant,
}

impl SafeEguiWebViewApp {
    fn new() -> Self {
        Self {
            url_input: "https://www.rust-lang.org".to_string(),
            title_input: "My WebView".to_string(),
            status_messages: vec!["Application started - using safe system browser integration".to_string()],
            created_windows: HashMap::new(),
            next_window_id: 1,
        }
    }

    fn create_new_window(&mut self) {
        if self.url_input.is_empty() {
            self.add_status_message("Please enter a URL".to_string());
            return;
        }

        let title = if self.title_input.is_empty() {
            format!("WebView Window {}", self.next_window_id)
        } else {
            self.title_input.clone()
        };

        let url = if !self.url_input.starts_with("http://") && !self.url_input.starts_with("https://") {
            format!("https://{}", self.url_input)
        } else {
            self.url_input.clone()
        };

        match self.open_url_in_system_browser(&url) {
            Ok(_) => {
                let window_id = format!("window_{}", self.next_window_id);
                self.next_window_id += 1;

                let window_info = WindowInfo {
                    title: title.clone(),
                    url: url.clone(),
                    created_at: std::time::Instant::now(),
                };

                self.created_windows.insert(window_id.clone(), window_info);
                self.add_status_message(format!("Opened '{}' in system browser", title));
            }
            Err(e) => {
                self.add_status_message(format!("Error opening URL: {}", e));
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

    fn create_preset_window(&mut self, title: &str, url: &str) {
        self.title_input = title.to_string();
        self.url_input = url.to_string();
        self.create_new_window();
    }

    fn clear_window_history(&mut self) {
        self.created_windows.clear();
        self.add_status_message("Window history cleared".to_string());
    }

    fn add_status_message(&mut self, message: String) {
        self.status_messages.push(message);
        // Keep only last 10 messages
        if self.status_messages.len() > 10 {
            self.status_messages.remove(0);
        }
    }
}

impl eframe::App for SafeEguiWebViewApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Top menu bar
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("New Window").clicked() {
                        self.create_new_window();
                        ui.close_menu();
                    }
                    if ui.button("Clear History").clicked() {
                        self.clear_window_history();
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("Exit").clicked() {
                        std::process::exit(0);
                    }
                });

                ui.menu_button("Help", |ui| {
                    if ui.button("About").clicked() {
                        self.add_status_message("Safe egui + WebView Integration v1.0".to_string());
                        ui.close_menu();
                    }
                });
            });
        });

        // Main content
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("🛡️ Safe egui + WebView Integration");
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

                    if ui.button("🔗 Open in Browser").clicked() {
                        self.create_new_window();
                    }
                });

                // Preset buttons
                ui.horizontal(|ui| {
                    ui.label("Quick open:");
                    if ui.button("🦀 Rust Docs").clicked() {
                        self.create_preset_window("Rust Documentation", "https://doc.rust-lang.org");
                    }
                    if ui.button("📖 egui Demo").clicked() {
                        self.create_preset_window("egui Demo", "https://www.egui.rs");
                    }
                    if ui.button("🌍 Wikipedia").clicked() {
                        self.create_preset_window("Wikipedia", "https://en.wikipedia.org");
                    }
                    if ui.button("🔍 GitHub").clicked() {
                        self.create_preset_window("GitHub", "https://github.com");
                    }
                });
            });

            ui.add_space(10.0);

            // Window history section
            ui.group(|ui| {
                ui.label("📱 Opened Windows History");
                
                if self.created_windows.is_empty() {
                    ui.colored_label(egui::Color32::GRAY, "No windows opened yet");
                } else {
                    egui::ScrollArea::vertical()
                        .id_source("window_history_scroll")
                        .max_height(200.0)
                        .show(ui, |ui| {
                            for (_id, window_info) in &self.created_windows {
                                ui.horizontal(|ui| {
                                    ui.label("🌐");
                                    ui.label(&window_info.title);
                                    ui.separator();
                                    ui.small(&window_info.url);
                                    ui.separator();
                                    ui.small(format!("{}s ago", window_info.created_at.elapsed().as_secs()));
                                });
                            }
                        });
                }
            });

            ui.add_space(10.0);

            // Information section
            ui.group(|ui| {
                ui.label("ℹ️ About this safe integration:");
                ui.label("• Uses system browser commands instead of embedded WebView");
                ui.label("• Avoids threading and memory safety issues");
                ui.label("• Cross-platform: macOS (open), Linux (xdg-open), Windows (start)");
                ui.label("• Compatible with egui 0.28.1");
                ui.label("• Safer for production use");
            });

            // Status messages
            if !self.status_messages.is_empty() {
                ui.add_space(10.0);
                ui.group(|ui| {
                    ui.label("📋 Status Messages");
                    egui::ScrollArea::vertical()
                        .id_source("status_messages_scroll_safe")
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
    }
}

fn main() -> Result<(), eframe::Error> {
    env_logger::init();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0])
            .with_title("Safe egui + WebView Integration"),
        ..Default::default()
    };

    eframe::run_native(
        "Safe egui WebView Example",
        options,
        Box::new(|_cc| Ok(Box::new(SafeEguiWebViewApp::new()))),
    )
}
