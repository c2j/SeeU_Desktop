//! WebView Positioning Demo
//!
//! This example demonstrates WebView positioning concepts without risky operations
//! Shows how positioning would work in a safe, stable environment

use eframe::egui;
use web_view::*;

/// Positioning demo application
struct PositioningDemoApp {
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
    /// Calculated WebView position
    calculated_webview_pos: egui::Pos2,
    /// Calculated WebView size
    calculated_webview_size: egui::Vec2,
    /// Window position (simulated)
    window_pos: egui::Pos2,
}

impl Default for PositioningDemoApp {
    fn default() -> Self {
        Self {
            url_input: "https://www.rust-lang.org".to_string(),
            status_message: "Ready for positioning demo".to_string(),
            webview: None,
            webview_rect: None,
            webview_created: false,
            position_sync_enabled: true,
            debug_positioning: true,
            calculated_webview_pos: egui::Pos2::new(0.0, 0.0),
            calculated_webview_size: egui::Vec2::new(600.0, 400.0),
            window_pos: egui::Pos2::new(100.0, 100.0),
        }
    }
}

impl PositioningDemoApp {
    fn create_demo_webview(&mut self) {
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

        // Create safe WebView for demonstration
        let result = web_view::builder()
            .title("Positioning Demo WebView")
            .content(Content::Url(&url))
            .size(self.calculated_webview_size.x as i32, self.calculated_webview_size.y as i32)
            .resizable(false)
            .debug(false)
            .frameless(true)  // Frameless for better appearance
            .user_data(())
            .invoke_handler(|_webview, arg| {
                println!("Demo WebView: {}", arg);
                Ok(())
            })
            .build();

        match result {
            Ok(webview) => {
                self.webview = Some(webview);
                self.webview_created = true;
                self.status_message = format!("✅ Created demo WebView: {}", url);
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

    fn calculate_positioning(&mut self, egui_rect: egui::Rect) {
        if !self.position_sync_enabled || !self.webview_created {
            return;
        }

        // Calculate where WebView should be positioned
        let target_x = self.window_pos.x + egui_rect.min.x;
        let target_y = self.window_pos.y + egui_rect.min.y;
        
        // Update calculated position
        self.calculated_webview_pos = egui::Pos2::new(target_x, target_y);
        self.calculated_webview_size = egui_rect.size();
        
        if self.debug_positioning {
            self.status_message = format!(
                "Calculated: egui({:.0},{:.0}) + window({:.0},{:.0}) = target({:.0},{:.0})",
                egui_rect.min.x, egui_rect.min.y,
                self.window_pos.x, self.window_pos.y,
                target_x, target_y
            );
        }
    }

    fn adjust_window_position(&mut self, delta: egui::Vec2) {
        self.window_pos += delta;
        if self.debug_positioning {
            println!("Window position adjusted to: ({:.0}, {:.0})", self.window_pos.x, self.window_pos.y);
        }
    }
}

impl eframe::App for PositioningDemoApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Step WebView
        self.step_webview();

        // Top controls
        egui::TopBottomPanel::top("controls").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("URL:");
                ui.add(egui::TextEdit::singleline(&mut self.url_input)
                    .desired_width(400.0));
                
                if ui.button("🔗 Create Demo WebView").clicked() {
                    self.create_demo_webview();
                }
                
                if self.webview.is_some() {
                    if ui.button("❌ Close").clicked() {
                        self.close_webview();
                    }
                    
                    ui.separator();
                    ui.checkbox(&mut self.position_sync_enabled, "🎯 Position Calc");
                    ui.checkbox(&mut self.debug_positioning, "🔍 Debug Info");
                }
            });
            
            // Window position controls
            if self.webview_created {
                ui.horizontal(|ui| {
                    ui.label("Simulate window move:");
                    if ui.button("⬅️").clicked() {
                        self.adjust_window_position(egui::vec2(-20.0, 0.0));
                    }
                    if ui.button("➡️").clicked() {
                        self.adjust_window_position(egui::vec2(20.0, 0.0));
                    }
                    if ui.button("⬆️").clicked() {
                        self.adjust_window_position(egui::vec2(0.0, -20.0));
                    }
                    if ui.button("⬇️").clicked() {
                        self.adjust_window_position(egui::vec2(0.0, 20.0));
                    }
                    ui.label(format!("Window: ({:.0}, {:.0})", self.window_pos.x, self.window_pos.y));
                });
            }
        });

        // Bottom status
        egui::TopBottomPanel::bottom("status").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Status:");
                if self.webview_created {
                    ui.colored_label(egui::Color32::GREEN, "✅ Demo WebView Active");
                } else {
                    ui.colored_label(egui::Color32::GRAY, "⭕ No WebView");
                }
                ui.separator();
                ui.label("Mode: Safe Positioning Demo");
            });
            ui.label(&self.status_message);
        });

        // Main content area
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("📐 WebView Positioning Demo");
            
            if self.webview_created {
                ui.separator();
                
                // WebView controls
                ui.horizontal(|ui| {
                    if ui.button("💬 Alert").clicked() {
                        self.eval_js("alert('Hello from positioning demo!');");
                    }
                    if ui.button("🎨 Blue Background").clicked() {
                        self.eval_js("document.body.style.backgroundColor = '#e6f3ff';");
                    }
                    if ui.button("📄 Change Title").clicked() {
                        self.eval_js("document.title = 'Positioning Demo WebView';");
                    }
                });
                
                ui.add_space(10.0);
                
                // WebView area with positioning calculation
                ui.group(|ui| {
                    ui.label("🌐 WebView Positioning Area");
                    
                    let available = ui.available_size();
                    let webview_size = egui::vec2(
                        (available.x - 20.0).max(600.0),
                        (available.y - 120.0).max(400.0)
                    );
                    
                    let (rect, _response) = ui.allocate_exact_size(
                        webview_size,
                        egui::Sense::hover()
                    );
                    
                    self.webview_rect = Some(rect);
                    
                    // Calculate positioning
                    if self.position_sync_enabled && self.webview_created {
                        self.calculate_positioning(rect);
                    }
                    
                    // Draw the area
                    ui.painter().rect_filled(
                        rect,
                        egui::Rounding::same(8.0),
                        egui::Color32::from_rgb(245, 250, 255)
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
                        "📐 Positioning Calculation Demo\nWebView runs in separate window\nPositioning logic demonstrated here",
                        egui::FontId::proportional(16.0),
                        egui::Color32::from_rgb(100, 100, 100)
                    );
                    
                    // Debug positioning info
                    if self.debug_positioning && self.webview_created {
                        let debug_text = format!(
                            "egui area: ({:.0}, {:.0}) {}x{}\nWindow pos: ({:.0}, {:.0})\nCalculated WebView pos: ({:.0}, {:.0})\nCalculated WebView size: {}x{}",
                            rect.min.x, rect.min.y, rect.width(), rect.height(),
                            self.window_pos.x, self.window_pos.y,
                            self.calculated_webview_pos.x, self.calculated_webview_pos.y,
                            self.calculated_webview_size.x, self.calculated_webview_size.y
                        );
                        
                        ui.painter().text(
                            rect.min + egui::vec2(10.0, 10.0),
                            egui::Align2::LEFT_TOP,
                            debug_text,
                            egui::FontId::monospace(9.0),
                            egui::Color32::from_rgb(150, 150, 150)
                        );
                    }
                });
                
            } else {
                // Instructions
                ui.vertical_centered(|ui| {
                    ui.add_space(50.0);
                    ui.heading("WebView Positioning Demonstration");
                    ui.add_space(20.0);
                    
                    ui.group(|ui| {
                        ui.label("📐 Demo Features:");
                        ui.label("• Safe WebView creation (no segfaults)");
                        ui.label("• Position calculation demonstration");
                        ui.label("• Window movement simulation");
                        ui.label("• Debug information display");
                        ui.label("• Frameless WebView for better appearance");
                    });
                    
                    ui.add_space(20.0);
                    
                    ui.group(|ui| {
                        ui.label("🔧 How it works:");
                        ui.label("• Creates frameless WebView in separate window");
                        ui.label("• Calculates where WebView should be positioned");
                        ui.label("• Shows positioning logic without risky operations");
                        ui.label("• Demonstrates coordinate transformations");
                        ui.label("• Provides safe foundation for future embedding");
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
            .with_title("WebView Positioning Demo"),
        ..Default::default()
    };

    eframe::run_native(
        "WebView Positioning Demo",
        options,
        Box::new(|_cc| Ok(Box::new(PositioningDemoApp::default()))),
    )
}
