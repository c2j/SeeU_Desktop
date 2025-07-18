//! Manual WebView Alignment
//!
//! This example provides manual alignment controls to solve the positioning issue
//! by allowing users to manually position the WebView to match the egui target area

use eframe::egui;
use web_view::*;

/// Manual alignment application
struct ManualAlignApp {
    /// URL input
    url_input: String,
    /// Status message
    status_message: String,
    /// WebView instance
    webview: Option<WebView<'static, ()>>,
    /// WebView area in egui coordinates
    webview_rect: Option<egui::Rect>,
    /// WebView created
    webview_created: bool,
    /// Debug positioning
    debug_positioning: bool,
    /// Manual WebView position (screen coordinates)
    webview_screen_x: i32,
    webview_screen_y: i32,
    /// Manual WebView size
    webview_width: i32,
    webview_height: i32,
    /// Scale factor
    scale_factor: f32,
    /// Auto-apply changes
    auto_apply: bool,
}

impl Default for ManualAlignApp {
    fn default() -> Self {
        Self {
            url_input: "https://www.rust-lang.org".to_string(),
            status_message: "Ready for manual WebView alignment".to_string(),
            webview: None,
            webview_rect: None,
            webview_created: false,
            debug_positioning: true,
            webview_screen_x: 300,
            webview_screen_y: 200,
            webview_width: 800,
            webview_height: 600,
            scale_factor: 1.0,
            auto_apply: false,
        }
    }
}

impl ManualAlignApp {
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

        // Close existing WebView
        if self.webview.is_some() {
            self.webview = None;
            self.webview_created = false;
        }

        let result = web_view::builder()
            .title("Manual Aligned WebView")
            .content(Content::Url(&url))
            .size(self.webview_width, self.webview_height)
            .resizable(false)
            .debug(false)
            .frameless(true)  // Frameless for precise alignment
            .user_data(())
            .invoke_handler(|_webview, arg| {
                println!("Manual aligned WebView: {}", arg);
                Ok(())
            })
            .build();

        match result {
            Ok(webview) => {
                self.webview = Some(webview);
                self.webview_created = true;
                self.status_message = format!("✅ Created WebView: {}", url);
                
                // Apply initial positioning
                self.apply_manual_positioning();
            }
            Err(e) => {
                self.status_message = format!("❌ Error creating WebView: {:?}", e);
            }
        }
    }

    fn apply_manual_positioning(&mut self) {
        if !self.webview_created {
            return;
        }

        // Extract values to avoid borrowing issues
        let x = self.webview_screen_x;
        let y = self.webview_screen_y;
        let w = self.webview_width;
        let h = self.webview_height;

        if let Some(webview) = &mut self.webview {
            // Validate bounds
            if x >= -1000 && x <= 3000 && y >= -1000 && y <= 3000 &&
               w > 0 && w <= 2000 && h > 0 && h <= 2000 {

                // Apply positioning with error handling
                let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    webview.set_bounds(x, y, w, h);
                }));
                
                match result {
                    Ok(_) => {
                        self.status_message = format!(
                            "✅ WebView positioned at ({}, {}) {}x{}",
                            x, y, w, h
                        );

                        if self.debug_positioning {
                            println!("WebView bounds applied: ({}, {}) {}x{}", x, y, w, h);
                        }
                    }
                    Err(_) => {
                        self.status_message = "⚠️ set_bounds failed safely".to_string();
                        if self.debug_positioning {
                            println!("Warning: set_bounds failed for ({}, {}) {}x{}", x, y, w, h);
                        }
                    }
                }
            } else {
                self.status_message = "❌ Invalid bounds parameters".to_string();
            }
        }
    }

    fn step_webview(&mut self) {
        // Minimal stepping to avoid issues
        if self.webview.is_some() {
            // Keep WebView alive without calling step()
        }
    }

    fn close_webview(&mut self) {
        if self.webview.is_some() {
            if self.debug_positioning {
                println!("Closing manual aligned WebView...");
            }
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

    fn adjust_position(&mut self, dx: i32, dy: i32) {
        self.webview_screen_x += dx;
        self.webview_screen_y += dy;
        
        if self.auto_apply {
            self.apply_manual_positioning();
        }
        
        if self.debug_positioning {
            println!("Position adjusted to: ({}, {})", self.webview_screen_x, self.webview_screen_y);
        }
    }

    fn adjust_size(&mut self, dw: i32, dh: i32) {
        self.webview_width = (self.webview_width + dw).max(100).min(2000);
        self.webview_height = (self.webview_height + dh).max(100).min(2000);
        
        if self.auto_apply {
            self.apply_manual_positioning();
        }
        
        if self.debug_positioning {
            println!("Size adjusted to: {}x{}", self.webview_width, self.webview_height);
        }
    }

    fn match_target_area(&mut self) {
        if let Some(rect) = self.webview_rect {
            // Try to match the target area size
            self.webview_width = (rect.width() * self.scale_factor) as i32;
            self.webview_height = (rect.height() * self.scale_factor) as i32;
            
            // Estimate position based on target area
            // This is a rough estimate - user will need to fine-tune
            let estimated_window_x = 200.0; // Rough estimate
            let estimated_window_y = 100.0; // Rough estimate
            let title_bar_height = 28.0;
            
            self.webview_screen_x = ((estimated_window_x + rect.min.x) * self.scale_factor) as i32;
            self.webview_screen_y = ((estimated_window_y + title_bar_height + rect.min.y) * self.scale_factor) as i32;
            
            self.apply_manual_positioning();
            
            self.status_message = format!(
                "📐 Matched target area: {}x{} at estimated position ({}, {})",
                self.webview_width, self.webview_height, self.webview_screen_x, self.webview_screen_y
            );
        }
    }
}

impl eframe::App for ManualAlignApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Update scale factor
        self.scale_factor = ctx.pixels_per_point();
        
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
            
            // Manual positioning controls
            if self.webview_created {
                ui.separator();
                
                ui.horizontal(|ui| {
                    ui.label("Position:");
                    ui.add(egui::DragValue::new(&mut self.webview_screen_x).prefix("X: "));
                    ui.add(egui::DragValue::new(&mut self.webview_screen_y).prefix("Y: "));
                    
                    ui.separator();
                    ui.label("Size:");
                    ui.add(egui::DragValue::new(&mut self.webview_width).prefix("W: "));
                    ui.add(egui::DragValue::new(&mut self.webview_height).prefix("H: "));
                    
                    ui.separator();
                    ui.checkbox(&mut self.auto_apply, "Auto Apply");
                    
                    if ui.button("📍 Apply Position").clicked() {
                        self.apply_manual_positioning();
                    }
                });
                
                ui.horizontal(|ui| {
                    ui.label("Quick adjust:");
                    if ui.button("⬅️ 10px").clicked() {
                        self.adjust_position(-10, 0);
                    }
                    if ui.button("➡️ 10px").clicked() {
                        self.adjust_position(10, 0);
                    }
                    if ui.button("⬆️ 10px").clicked() {
                        self.adjust_position(0, -10);
                    }
                    if ui.button("⬇️ 10px").clicked() {
                        self.adjust_position(0, 10);
                    }
                    
                    ui.separator();
                    if ui.button("🔍 Smaller").clicked() {
                        self.adjust_size(-50, -30);
                    }
                    if ui.button("🔍 Larger").clicked() {
                        self.adjust_size(50, 30);
                    }
                    
                    ui.separator();
                    if ui.button("📐 Match Target").clicked() {
                        self.match_target_area();
                    }
                });
                
                ui.horizontal(|ui| {
                    ui.label("Fine adjust:");
                    if ui.button("⬅️ 1px").clicked() {
                        self.adjust_position(-1, 0);
                    }
                    if ui.button("➡️ 1px").clicked() {
                        self.adjust_position(1, 0);
                    }
                    if ui.button("⬆️ 1px").clicked() {
                        self.adjust_position(0, -1);
                    }
                    if ui.button("⬇️ 1px").clicked() {
                        self.adjust_position(0, 1);
                    }
                    
                    ui.separator();
                    ui.checkbox(&mut self.debug_positioning, "🔍 Debug");
                });
            }
        });

        // Bottom status
        egui::TopBottomPanel::bottom("status").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Status:");
                if self.webview_created {
                    ui.colored_label(egui::Color32::GREEN, "✅ WebView Active");
                } else {
                    ui.colored_label(egui::Color32::GRAY, "⭕ No WebView");
                }
                ui.separator();
                ui.label(format!("Scale: {:.2}x", self.scale_factor));
                ui.separator();
                ui.label(format!("WebView: ({}, {}) {}x{}", 
                    self.webview_screen_x, self.webview_screen_y, self.webview_width, self.webview_height));
            });
            ui.label(&self.status_message);
        });

        // Main content area
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("🎯 Manual WebView Alignment");
            
            if self.webview_created {
                ui.separator();
                
                // WebView controls
                ui.horizontal(|ui| {
                    if ui.button("💬 Alert").clicked() {
                        self.eval_js("alert('Hello from manually aligned WebView!');");
                    }
                    if ui.button("🎨 Blue Background").clicked() {
                        self.eval_js("document.body.style.backgroundColor = '#e3f2fd';");
                    }
                    if ui.button("📄 Change Title").clicked() {
                        self.eval_js("document.title = 'Manually Aligned WebView';");
                    }
                });
                
                ui.add_space(10.0);
                
                // Target alignment area
                ui.group(|ui| {
                    ui.label("🌐 Target Alignment Area - Align WebView to match this area");
                    
                    let available = ui.available_size();
                    let target_size = egui::vec2(
                        (available.x - 20.0).max(600.0),
                        (available.y - 100.0).max(400.0)
                    );
                    
                    let (rect, _response) = ui.allocate_exact_size(
                        target_size,
                        egui::Sense::hover()
                    );
                    
                    self.webview_rect = Some(rect);
                    
                    // Draw target area with very strong visual indicators
                    ui.painter().rect_filled(
                        rect,
                        egui::Rounding::same(8.0),
                        egui::Color32::from_rgba_premultiplied(0, 150, 255, 80)  // Blue with transparency
                    );
                    
                    // Very strong border
                    ui.painter().rect_stroke(
                        rect,
                        egui::Rounding::same(8.0),
                        egui::Stroke::new(5.0, egui::Color32::from_rgb(0, 100, 255))
                    );
                    
                    // Large corner markers
                    let marker_size = 20.0;
                    for corner in [rect.min, egui::pos2(rect.max.x, rect.min.y), rect.max, egui::pos2(rect.min.x, rect.max.y)] {
                        ui.painter().rect_filled(
                            egui::Rect::from_center_size(corner, egui::vec2(marker_size, marker_size)),
                            egui::Rounding::same(4.0),
                            egui::Color32::from_rgb(255, 0, 0)
                        );
                    }
                    
                    // Strong crosshairs
                    let center = rect.center();
                    ui.painter().line_segment(
                        [egui::pos2(rect.min.x, center.y), egui::pos2(rect.max.x, center.y)],
                        egui::Stroke::new(3.0, egui::Color32::from_rgb(255, 0, 0))
                    );
                    ui.painter().line_segment(
                        [egui::pos2(center.x, rect.min.y), egui::pos2(center.x, rect.max.y)],
                        egui::Stroke::new(3.0, egui::Color32::from_rgb(255, 0, 0))
                    );
                    
                    // Center text
                    ui.painter().text(
                        center,
                        egui::Align2::CENTER_CENTER,
                        "🎯 WEBVIEW TARGET AREA\nManually position WebView to align with this blue area\nUse the controls above to adjust position and size\nRed markers show exact boundaries",
                        egui::FontId::proportional(18.0),
                        egui::Color32::from_rgb(50, 50, 50)
                    );
                    
                    // Debug info
                    if self.debug_positioning && self.webview_created {
                        let debug_text = format!(
                            "Target area: ({:.0}, {:.0}) {}x{} (egui coords)\nScaled target: ({:.0}, {:.0}) {}x{} (screen coords)\nWebView position: ({}, {}) {}x{} (screen coords)\nScale factor: {:.2}x",
                            rect.min.x, rect.min.y, rect.width(), rect.height(),
                            rect.min.x * self.scale_factor, rect.min.y * self.scale_factor,
                            rect.width() * self.scale_factor, rect.height() * self.scale_factor,
                            self.webview_screen_x, self.webview_screen_y, self.webview_width, self.webview_height,
                            self.scale_factor
                        );
                        
                        ui.painter().text(
                            rect.min + egui::vec2(10.0, 10.0),
                            egui::Align2::LEFT_TOP,
                            debug_text,
                            egui::FontId::monospace(10.0),
                            egui::Color32::from_rgb(255, 255, 255)
                        );
                    }
                });
                
            } else {
                // Instructions
                ui.vertical_centered(|ui| {
                    ui.add_space(50.0);
                    ui.heading("Manual WebView Alignment System");
                    ui.add_space(20.0);
                    
                    ui.group(|ui| {
                        ui.label("🎯 Manual Alignment Features:");
                        ui.label("• Direct position and size controls");
                        ui.label("• Real-time WebView positioning");
                        ui.label("• 1px precision fine-tuning");
                        ui.label("• Target area matching");
                        ui.label("• Auto-apply option");
                        ui.label("• Strong visual target indicators");
                    });
                    
                    ui.add_space(20.0);
                    
                    ui.group(|ui| {
                        ui.label("🔧 How to achieve perfect alignment:");
                        ui.label("1. Create WebView with your target URL");
                        ui.label("2. Click 'Match Target' to get close");
                        ui.label("3. Use 10px buttons for coarse adjustment");
                        ui.label("4. Use 1px buttons for fine-tuning");
                        ui.label("5. Enable 'Auto Apply' for real-time updates");
                        ui.label("6. WebView should align with blue target area");
                    });
                });
            }
        });

        // No automatic repaint to avoid issues
    }
}

fn main() -> Result<(), eframe::Error> {
    env_logger::init();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 900.0])
            .with_title("Manual WebView Alignment"),
        ..Default::default()
    };

    eframe::run_native(
        "Manual WebView Alignment",
        options,
        Box::new(|_cc| Ok(Box::new(ManualAlignApp::default()))),
    )
}
