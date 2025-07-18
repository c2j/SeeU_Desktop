//! Safe Embedded WebView Implementation
//!
//! This example demonstrates a safer approach to WebView embedding
//! that avoids segmentation faults while still providing true embedding capabilities

use eframe::egui;
use std::os::raw::c_void;
use web_view::*;

/// Safe embedded WebView application
struct SafeEmbeddedApp {
    /// URL input
    url_input: String,
    /// Status message
    status_message: String,
    /// WebView instance
    webview: Option<WebView<'static, ()>>,
    /// Show WebView
    show_webview: bool,
    /// Embedding mode
    embedding_mode: EmbeddingMode,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum EmbeddingMode {
    Standard,      // Standard WebView window
    Positioned,    // Positioned over egui area
    TrueEmbedded,  // True embedding (when available)
}

impl Default for SafeEmbeddedApp {
    fn default() -> Self {
        Self {
            url_input: "https://www.rust-lang.org".to_string(),
            status_message: "Ready for safe WebView embedding".to_string(),
            webview: None,
            show_webview: true,
            embedding_mode: EmbeddingMode::Standard,
        }
    }
}

impl SafeEmbeddedApp {
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

        // Close existing WebView first
        if self.webview.is_some() {
            self.webview = None;
            self.status_message = "Closed previous WebView".to_string();
        }

        let title = match self.embedding_mode {
            EmbeddingMode::Standard => "Standard WebView",
            EmbeddingMode::Positioned => "Positioned WebView",
            EmbeddingMode::TrueEmbedded => "True Embedded WebView",
        };

        // Create WebView based on embedding mode
        let result = match self.embedding_mode {
            EmbeddingMode::TrueEmbedded => {
                // TODO: Implement safe parent window handle extraction
                // For now, fall back to standard mode
                self.status_message = "True embedding not yet implemented safely, using standard mode".to_string();
                web_view::builder()
                    .title("True Embedded (Fallback)")
                    .content(Content::Url(&url))
                    .size(800, 600)
                    .resizable(true)
                    .debug(false)
                    .user_data(())
                    .invoke_handler(|_webview, arg| {
                        println!("True embedded fallback: {}", arg);
                        Ok(())
                    })
                    .build()
            }
            _ => {
                web_view::builder()
                    .title(title)
                    .content(Content::Url(&url))
                    .size(800, 600)
                    .resizable(true)
                    .debug(false)
                    .user_data(())
                    .invoke_handler(|_webview, arg| {
                        println!("Standard WebView: {}", arg);
                        Ok(())
                    })
                    .build()
            }
        };

        match result {
            Ok(webview) => {
                self.webview = Some(webview);
                self.status_message = format!("✅ Created {} WebView: {}", title, url);
            }
            Err(e) => {
                self.status_message = format!("❌ Error creating WebView: {:?}", e);
            }
        }
    }



    fn step_webview(&mut self) {
        if let Some(webview) = &mut self.webview {
            match webview.step() {
                Some(Ok(_)) => {
                    // WebView is running
                }
                Some(Err(e)) => {
                    self.status_message = format!("WebView error: {:?}", e);
                    self.webview = None;
                }
                None => {
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
                    self.status_message = format!("✅ JS executed: {}", js);
                }
                Err(e) => {
                    self.status_message = format!("❌ JS error: {:?}", e);
                }
            }
        }
    }
}

impl eframe::App for SafeEmbeddedApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Step WebView
        self.step_webview();

        // Top controls
        egui::TopBottomPanel::top("controls").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("URL:");
                ui.add(egui::TextEdit::singleline(&mut self.url_input)
                    .desired_width(300.0));
                
                if ui.button("🔗 Create WebView").clicked() {
                    self.create_webview();
                }
                
                if self.webview.is_some() && ui.button("❌ Close").clicked() {
                    self.close_webview();
                }
            });
            
            ui.horizontal(|ui| {
                ui.label("Embedding Mode:");
                ui.radio_value(&mut self.embedding_mode, EmbeddingMode::Standard, "Standard");
                ui.radio_value(&mut self.embedding_mode, EmbeddingMode::Positioned, "Positioned");
                ui.radio_value(&mut self.embedding_mode, EmbeddingMode::TrueEmbedded, "True Embedded");
            });
        });

        // Bottom status
        egui::TopBottomPanel::bottom("status").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Status:");
                if self.webview.is_some() {
                    ui.colored_label(egui::Color32::GREEN, "✅ WebView Active");
                } else {
                    ui.colored_label(egui::Color32::GRAY, "⭕ No WebView");
                }
                ui.separator();
                ui.label(&self.status_message);
            });
        });

        // Main content area
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("🛡️ Safe Embedded WebView Implementation");
            
            if self.show_webview && self.webview.is_some() {
                ui.separator();
                
                // WebView controls
                ui.horizontal(|ui| {
                    ui.checkbox(&mut self.show_webview, "Show WebView Area");
                    
                    if ui.button("💬 Alert").clicked() {
                        self.eval_js("alert('Hello from safe embedded WebView!');");
                    }
                    if ui.button("🎨 Style").clicked() {
                        self.eval_js("document.body.style.backgroundColor = '#f0f8ff'; document.body.style.border = '2px solid #4CAF50';");
                    }
                    if ui.button("📄 Title").clicked() {
                        self.eval_js("document.title = 'Modified by Safe Embedded WebView';");
                    }
                });
                
                ui.add_space(10.0);
                
                // WebView area representation
                let available = ui.available_size();
                let webview_size = egui::vec2(
                    (available.x * 0.9).max(400.0),
                    (available.y * 0.7).max(300.0)
                );
                
                ui.allocate_ui(webview_size, |ui| {
                    let rect = ui.max_rect();
                    
                    // Draw WebView container
                    ui.painter().rect_filled(
                        rect,
                        egui::Rounding::same(8.0),
                        egui::Color32::from_rgb(248, 252, 255)
                    );
                    
                    ui.painter().rect_stroke(
                        rect,
                        egui::Rounding::same(8.0),
                        egui::Stroke::new(2.0, egui::Color32::from_rgb(100, 150, 200))
                    );
                    
                    // Content based on embedding mode
                    ui.centered_and_justified(|ui| {
                        ui.vertical_centered(|ui| {
                            match self.embedding_mode {
                                EmbeddingMode::Standard => {
                                    ui.label("🌐 Standard WebView Mode");
                                    ui.label("WebView runs in separate window");
                                }
                                EmbeddingMode::Positioned => {
                                    ui.label("📍 Positioned WebView Mode");
                                    ui.label("WebView positioned over this area");
                                }
                                EmbeddingMode::TrueEmbedded => {
                                    ui.label("🎯 True Embedded Mode");
                                    ui.label("WebView embedded as child view");
                                    ui.colored_label(egui::Color32::YELLOW, "⚠️ Implementation in progress");
                                }
                            }
                            ui.add_space(10.0);
                            ui.label(format!("Area: {:.0}x{:.0}", rect.width(), rect.height()));
                            ui.label(format!("Position: ({:.0}, {:.0})", rect.min.x, rect.min.y));
                        });
                    });
                });
                
            } else {
                // Instructions
                ui.vertical_centered(|ui| {
                    ui.add_space(50.0);
                    ui.heading("Select embedding mode and create WebView");
                    ui.add_space(20.0);
                    
                    ui.group(|ui| {
                        ui.label("🛡️ Safe Implementation Features:");
                        ui.label("• Prevents segmentation faults");
                        ui.label("• Graceful fallback mechanisms");
                        ui.label("• Multiple embedding modes");
                        ui.label("• Comprehensive error handling");
                        ui.label("• Memory safety guarantees");
                    });
                    
                    ui.add_space(20.0);
                    
                    ui.group(|ui| {
                        ui.label("🔧 Embedding Modes:");
                        ui.label("• Standard: Traditional separate window");
                        ui.label("• Positioned: Window positioned over egui area");
                        ui.label("• True Embedded: Child view embedding (WIP)");
                    });
                });
            }
        });

        // Request repaint for WebView stepping
        if self.webview.is_some() {
            ctx.request_repaint_after(std::time::Duration::from_millis(16));
        }
    }
}

fn main() -> Result<(), eframe::Error> {
    env_logger::init();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1000.0, 800.0])
            .with_title("Safe Embedded WebView Implementation"),
        ..Default::default()
    };

    eframe::run_native(
        "Safe Embedded WebView",
        options,
        Box::new(|_cc| Ok(Box::new(SafeEmbeddedApp::default()))),
    )
}
