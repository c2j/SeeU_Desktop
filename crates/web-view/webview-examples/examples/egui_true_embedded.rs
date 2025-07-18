//! True Embedded WebView with egui Integration
//!
//! This example demonstrates true WebView embedding within an egui application
//! using the safe step() method approach from multi_window.rs

use eframe::egui;
use std::collections::HashMap;
use web_view::*;

/// Application state for true embedded WebView
struct TrueEmbeddedApp {
    /// URL input
    url_input: String,
    /// Status message
    status_message: String,
    /// Active WebViews (using Box to avoid lifetime issues)
    webviews: HashMap<String, Box<WebView<'static, ()>>>,
    /// Next WebView ID
    next_id: u32,
    /// Show controls
    show_controls: bool,
    /// Auto-step WebViews
    auto_step: bool,
}

impl Default for TrueEmbeddedApp {
    fn default() -> Self {
        Self {
            url_input: "https://www.rust-lang.org".to_string(),
            status_message: "Ready to create embedded WebViews".to_string(),
            webviews: HashMap::new(),
            next_id: 1,
            show_controls: true,
            auto_step: true,
        }
    }
}

impl TrueEmbeddedApp {
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

        let webview_id = format!("webview_{}", self.next_id);
        self.next_id += 1;

        // Create WebView using the safe build() method
        match web_view::builder()
            .title(&format!("Embedded WebView {}", webview_id))
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
                self.webviews.insert(webview_id.clone(), Box::new(webview));
                self.status_message = format!("Created WebView {}: {}", webview_id, url);
            }
            Err(e) => {
                self.status_message = format!("Error creating WebView: {:?}", e);
            }
        }
    }

    fn step_all_webviews(&mut self) {
        if !self.auto_step {
            return;
        }

        let mut to_remove = Vec::new();
        
        for (id, webview) in &mut self.webviews {
            match webview.step() {
                Some(Ok(_)) => {
                    // WebView is still running
                }
                Some(Err(e)) => {
                    println!("WebView {} error: {:?}", id, e);
                    to_remove.push(id.clone());
                }
                None => {
                    // WebView was closed
                    to_remove.push(id.clone());
                }
            }
        }

        // Remove closed WebViews
        for id in to_remove {
            self.webviews.remove(&id);
            self.status_message = format!("WebView {} was closed", id);
        }
    }

    fn close_webview(&mut self, id: &str) {
        if self.webviews.remove(id).is_some() {
            self.status_message = format!("Closed WebView {}", id);
        }
    }

    fn close_all_webviews(&mut self) {
        let count = self.webviews.len();
        self.webviews.clear();
        self.status_message = format!("Closed {} WebView(s)", count);
    }

    fn eval_js_in_webview(&mut self, id: &str, js: &str) {
        if let Some(webview) = self.webviews.get_mut(id) {
            match webview.eval(js) {
                Ok(_) => {
                    self.status_message = format!("Executed JS in {}: {}", id, js);
                }
                Err(e) => {
                    self.status_message = format!("JS error in {}: {:?}", id, e);
                }
            }
        }
    }
}

impl eframe::App for TrueEmbeddedApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Step all WebViews first
        self.step_all_webviews();

        // Top menu bar
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("WebView", |ui| {
                    if ui.button("New WebView").clicked() {
                        self.create_webview();
                        ui.close_menu();
                    }
                    if ui.button("Close All").clicked() {
                        self.close_all_webviews();
                        ui.close_menu();
                    }
                });
                
                ui.menu_button("Controls", |ui| {
                    ui.checkbox(&mut self.show_controls, "Show Controls");
                    ui.checkbox(&mut self.auto_step, "Auto Step WebViews");
                });
            });
        });

        // Main content
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("🔗 True Embedded WebView with egui");
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
            });

            ui.add_space(10.0);

            // Quick navigation buttons
            ui.horizontal(|ui| {
                ui.label("Quick create:");
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
                if ui.button("🔍 GitHub").clicked() {
                    self.url_input = "https://github.com".to_string();
                    self.create_webview();
                }
            });

            ui.separator();

            // Active WebViews section
            ui.group(|ui| {
                ui.label(format!("📱 Active WebViews ({})", self.webviews.len()));
                
                if self.webviews.is_empty() {
                    ui.colored_label(egui::Color32::GRAY, "No active WebViews");
                } else {
                    egui::ScrollArea::vertical()
                        .id_source("webviews_scroll")
                        .max_height(200.0)
                        .show(ui, |ui| {
                            let webview_ids: Vec<String> = self.webviews.keys().cloned().collect();
                            for id in webview_ids {
                                ui.horizontal(|ui| {
                                    ui.label("🌐");
                                    ui.label(&id);
                                    
                                    if self.show_controls {
                                        if ui.small_button("💬 Alert").clicked() {
                                            self.eval_js_in_webview(&id, "alert('Hello from egui!');");
                                        }
                                        if ui.small_button("🎨 Color").clicked() {
                                            self.eval_js_in_webview(&id, "document.body.style.backgroundColor = '#e6f3ff';");
                                        }
                                        if ui.small_button("📄 Title").clicked() {
                                            self.eval_js_in_webview(&id, "document.title = 'Modified by egui';");
                                        }
                                    }
                                    
                                    if ui.small_button("❌").clicked() {
                                        self.close_webview(&id);
                                    }
                                });
                            }
                        });
                }
            });

            ui.add_space(10.0);

            // Status section
            ui.horizontal(|ui| {
                ui.label("Status:");
                if !self.webviews.is_empty() {
                    ui.colored_label(egui::Color32::GREEN, format!("✅ {} WebView(s) Active", self.webviews.len()));
                } else {
                    ui.colored_label(egui::Color32::GRAY, "⭕ No WebViews");
                }
            });

            ui.label(&self.status_message);

            ui.add_space(10.0);

            // Information section
            ui.group(|ui| {
                ui.label("ℹ️ About this true embedded example:");
                ui.label("• Uses WebView::build() + step() for safe operation");
                ui.label("• Multiple WebViews can run simultaneously");
                ui.label("• JavaScript evaluation support");
                ui.label("• Proper WebView lifecycle management");
                ui.label("• No threading issues - all operations in main thread");
            });

            ui.add_space(10.0);

            // Technical details
            ui.collapsing("🔧 Technical Details", |ui| {
                ui.label("Safe Architecture:");
                ui.label("• WebView::build() instead of run()");
                ui.label("• Manual step() calls in egui update loop");
                ui.label("• HashMap for multiple WebView management");
                ui.label("• JavaScript evaluation via eval()");
                ui.label("• Graceful WebView cleanup");
                
                ui.add_space(5.0);
                ui.label("Safety measures:");
                ui.label("• No thread spawning");
                ui.label("• Proper error handling");
                ui.label("• Resource cleanup on close");
                ui.label("• Main thread operation only");
            });
        });

        // Request repaint for WebView stepping
        if !self.webviews.is_empty() && self.auto_step {
            ctx.request_repaint_after(std::time::Duration::from_millis(16)); // ~60 FPS
        }
    }
}

fn main() -> Result<(), eframe::Error> {
    env_logger::init();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1000.0, 800.0])
            .with_title("True Embedded WebView with egui"),
        ..Default::default()
    };

    eframe::run_native(
        "True Embedded WebView Example",
        options,
        Box::new(|_cc| Ok(Box::new(TrueEmbeddedApp::default()))),
    )
}
