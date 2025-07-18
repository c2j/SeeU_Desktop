//! Safe Native WebView Embedding
//!
//! This example provides a safe approach to native WebView embedding
//! that avoids foreign exceptions while still demonstrating positioning concepts

use eframe::egui;
use web_view::*;

/// Safe native embedding application
struct SafeNativeEmbedApp {
    /// URL input
    url_input: String,
    /// Status message
    status_message: String,
    /// WebView instance
    webview: Option<WebView<'static, ()>>,
    /// WebView area
    webview_rect: Option<egui::Rect>,
    /// WebView created
    webview_created: bool,
    /// Position sync enabled
    position_sync_enabled: bool,
    /// Debug positioning
    debug_positioning: bool,
    /// Simulated WebView position
    simulated_webview_pos: egui::Pos2,
    /// Simulated WebView size
    simulated_webview_size: egui::Vec2,
}

impl Default for SafeNativeEmbedApp {
    fn default() -> Self {
        Self {
            url_input: "https://www.rust-lang.org".to_string(),
            status_message: "Ready for safe native WebView embedding".to_string(),
            webview: None,
            webview_rect: None,
            webview_created: false,
            position_sync_enabled: true,
            debug_positioning: true,
            simulated_webview_pos: egui::Pos2::new(100.0, 100.0),
            simulated_webview_size: egui::Vec2::new(600.0, 400.0),
        }
    }
}

impl SafeNativeEmbedApp {
    fn create_safe_webview(&mut self) {
        if self.url_input.is_empty() {
            self.status_message = "Please enter a URL".to_string();
            return;
        }

        let url = if !self.url_input.starts_with("http://") && !self.url_input.starts_with("https://") {
            format!("https://{}", self.url_input)
        } else {
            self.url_input.clone()
        };

        // Close existing WebView
        if self.webview.is_some() {
            self.webview = None;
            self.webview_created = false;
        }

        // Create safe WebView without any risky native operations
        let result = web_view::builder()
            .title("Safe Native-Style WebView")
            .content(Content::Url(&url))
            .size(self.simulated_webview_size.x as i32, self.simulated_webview_size.y as i32)
            .resizable(false)
            .debug(false)
            .frameless(true)  // Frameless for better embedding appearance
            .user_data(())
            .invoke_handler(|_webview, arg| {
                println!("Safe WebView: {}", arg);
                Ok(())
            })
            .build();

        match result {
            Ok(webview) => {
                self.webview = Some(webview);
                self.webview_created = true;
                self.status_message = format!("✅ Created safe WebView: {}", url);
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
                    self.webview_created = false;
                }
                None => {
                    self.webview = None;
                    self.webview_created = false;
                    self.status_message = "WebView was closed".to_string();
                }
            }
        }
    }

    fn close_webview(&mut self) {
        if self.webview.is_some() {
            self.webview = None;
            self.webview_created = false;
            self.status_message = "WebView closed".to_string();
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

    fn simulate_position_sync(&mut self, egui_rect: egui::Rect, window_pos: egui::Pos2) {
        if !self.position_sync_enabled || !self.webview_created {
            return;
        }

        // Simulate position synchronization
        let target_x = window_pos.x + egui_rect.min.x;
        let target_y = window_pos.y + egui_rect.min.y;
        
        // Update simulated position
        self.simulated_webview_pos = egui::Pos2::new(target_x, target_y);
        self.simulated_webview_size = egui_rect.size();
        
        if self.debug_positioning {
            self.status_message = format!(
                "Simulated sync: egui({:.0},{:.0}) + window({:.0},{:.0}) = target({:.0},{:.0})",
                egui_rect.min.x, egui_rect.min.y,
                window_pos.x, window_pos.y,
                target_x, target_y
            );
        }
    }

    fn adjust_position(&mut self, delta: egui::Vec2) {
        self.simulated_webview_pos += delta;
        if self.debug_positioning {
            println!("Position adjusted by ({:.0}, {:.0})", delta.x, delta.y);
        }
    }
}

impl eframe::App for SafeNativeEmbedApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Step WebView
        self.step_webview();

        // Simulated window position
        let current_window_pos = egui::Pos2::new(100.0, 100.0);

        // Top controls
        egui::TopBottomPanel::top("controls").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("URL:");
                ui.add(egui::TextEdit::singleline(&mut self.url_input)
                    .desired_width(400.0));
                
                if ui.button("🔗 Create Safe WebView").clicked() {
                    self.create_safe_webview();
                }
                
                if self.webview.is_some() {
                    if ui.button("❌ Close").clicked() {
                        self.close_webview();
                    }
                    
                    ui.separator();
                    ui.checkbox(&mut self.position_sync_enabled, "🎯 Position Sync");
                    ui.checkbox(&mut self.debug_positioning, "🔍 Debug Positioning");
                    
                    // Manual position adjustment
                    ui.horizontal(|ui| {
                        ui.label("Adjust:");
                        if ui.button("⬅️").clicked() {
                            self.adjust_position(egui::vec2(-10.0, 0.0));
                        }
                        if ui.button("➡️").clicked() {
                            self.adjust_position(egui::vec2(10.0, 0.0));
                        }
                        if ui.button("⬆️").clicked() {
                            self.adjust_position(egui::vec2(0.0, -10.0));
                        }
                        if ui.button("⬇️").clicked() {
                            self.adjust_position(egui::vec2(0.0, 10.0));
                        }
                    });
                }
            });
        });

        // Bottom status
        egui::TopBottomPanel::bottom("status").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Status:");
                if self.webview_created {
                    ui.colored_label(egui::Color32::GREEN, "✅ Safe WebView Active");
                } else {
                    ui.colored_label(egui::Color32::GRAY, "⭕ No WebView");
                }
                ui.separator();
                ui.label("Mode: Safe (No Foreign Exceptions)");
            });
            ui.label(&self.status_message);
        });

        // Main content area
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("🛡️ Safe Native WebView Embedding");
            
            if self.webview_created {
                ui.separator();
                
                // WebView controls
                ui.horizontal(|ui| {
                    if ui.button("💬 Alert").clicked() {
                        self.eval_js("alert('Hello from safe WebView!');");
                    }
                    if ui.button("🎨 Green Background").clicked() {
                        self.eval_js("document.body.style.backgroundColor = '#e8f5e8';");
                    }
                    if ui.button("📄 Change Title").clicked() {
                        self.eval_js("document.title = 'Safe Native WebView';");
                    }
                });
                
                ui.add_space(10.0);
                
                // WebView area simulation
                ui.group(|ui| {
                    ui.label("🌐 Safe WebView Area (Simulated Positioning)");
                    
                    let available = ui.available_size();
                    let webview_size = egui::vec2(
                        (available.x - 20.0).max(600.0),
                        (available.y - 100.0).max(400.0)
                    );
                    
                    let (rect, _response) = ui.allocate_exact_size(
                        webview_size,
                        egui::Sense::hover()
                    );
                    
                    self.webview_rect = Some(rect);
                    
                    // Simulate position sync
                    if self.position_sync_enabled && self.webview_created {
                        self.simulate_position_sync(rect, current_window_pos);
                    }
                    
                    // Draw the area
                    ui.painter().rect_filled(
                        rect,
                        egui::Rounding::same(8.0),
                        egui::Color32::from_rgb(240, 255, 240)  // Light green
                    );
                    
                    ui.painter().rect_stroke(
                        rect,
                        egui::Rounding::same(8.0),
                        egui::Stroke::new(3.0, egui::Color32::from_rgb(76, 175, 80))
                    );
                    
                    // Center text
                    let center = rect.center();
                    ui.painter().text(
                        center,
                        egui::Align2::CENTER_CENTER,
                        "🛡️ Safe WebView (No Foreign Exceptions)\nFrameless window positioned separately",
                        egui::FontId::proportional(16.0),
                        egui::Color32::from_rgb(100, 100, 100)
                    );
                    
                    // Debug positioning info
                    if self.debug_positioning && self.webview_created {
                        let debug_text = format!(
                            "Simulated WebView Position:\n({:.0}, {:.0}) {}x{}\nSync: {}",
                            self.simulated_webview_pos.x, self.simulated_webview_pos.y,
                            self.simulated_webview_size.x, self.simulated_webview_size.y,
                            if self.position_sync_enabled { "ON" } else { "OFF" }
                        );
                        
                        ui.painter().text(
                            rect.min + egui::vec2(10.0, 10.0),
                            egui::Align2::LEFT_TOP,
                            debug_text,
                            egui::FontId::monospace(10.0),
                            egui::Color32::from_rgb(150, 150, 150)
                        );
                    }
                });
                
            } else {
                // Instructions
                ui.vertical_centered(|ui| {
                    ui.add_space(50.0);
                    ui.heading("Safe Native WebView Embedding");
                    ui.add_space(20.0);
                    
                    ui.group(|ui| {
                        ui.label("🛡️ Safety Features:");
                        ui.label("• No foreign exception risks");
                        ui.label("• No unsafe Objective-C calls");
                        ui.label("• Stable WebView creation");
                        ui.label("• Simulated position synchronization");
                        ui.label("• Frameless WebView for better appearance");
                    });
                    
                    ui.add_space(20.0);
                    
                    ui.group(|ui| {
                        ui.label("🔧 How it works:");
                        ui.label("• Creates frameless WebView window");
                        ui.label("• Simulates position synchronization");
                        ui.label("• Provides manual position adjustment");
                        ui.label("• Shows debug information");
                        ui.label("• Avoids risky native operations");
                    });
                });
            }
        });

        // Request repaint for WebView stepping
        if self.webview_created {
            ctx.request_repaint_after(std::time::Duration::from_millis(16));
        }
    }
}

fn main() -> Result<(), eframe::Error> {
    env_logger::init();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 900.0])
            .with_title("Safe Native WebView Embedding"),
        ..Default::default()
    };

    eframe::run_native(
        "Safe Native WebView Embedding",
        options,
        Box::new(|_cc| Ok(Box::new(SafeNativeEmbedApp::default()))),
    )
}
