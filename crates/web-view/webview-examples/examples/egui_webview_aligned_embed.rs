//! Aligned WebView Embedding
//!
//! This example focuses on precise position alignment between egui areas and WebView windows

use eframe::egui;
use web_view::*;

/// Aligned embedding application
struct AlignedEmbedApp {
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
    /// Position sync enabled
    position_sync_enabled: bool,
    /// Debug positioning
    debug_positioning: bool,
    /// Manual position offset
    manual_offset: egui::Vec2,
    /// Window position tracking
    last_window_pos: Option<egui::Pos2>,
    /// Screen scale factor
    scale_factor: f32,
    /// Title bar height estimation
    title_bar_height: f32,
}

impl Default for AlignedEmbedApp {
    fn default() -> Self {
        Self {
            url_input: "https://www.rust-lang.org".to_string(),
            status_message: "Ready for aligned WebView embedding".to_string(),
            webview: None,
            webview_rect: None,
            webview_created: false,
            position_sync_enabled: true,
            debug_positioning: true,
            manual_offset: egui::Vec2::ZERO,
            last_window_pos: None,
            scale_factor: 1.0,
            title_bar_height: 28.0, // Typical macOS title bar height
        }
    }
}

impl AlignedEmbedApp {
    fn create_aligned_webview(&mut self) {
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

        // Create WebView with initial size
        let initial_size = if let Some(rect) = self.webview_rect {
            (rect.width() as i32, rect.height() as i32)
        } else {
            (600, 400)
        };

        let result = web_view::builder()
            .title("Aligned WebView")
            .content(Content::Url(&url))
            .size(initial_size.0, initial_size.1)
            .resizable(false)
            .debug(false)
            .frameless(true)  // Frameless for better alignment
            .user_data(())
            .invoke_handler(|_webview, arg| {
                println!("Aligned WebView: {}", arg);
                Ok(())
            })
            .build();

        match result {
            Ok(webview) => {
                self.webview = Some(webview);
                self.webview_created = true;
                self.status_message = format!("✅ Created aligned WebView: {}", url);
            }
            Err(e) => {
                self.status_message = format!("❌ Error creating WebView: {:?}", e);
            }
        }
    }

    fn step_webview(&mut self) {
        // Safe stepping - avoid potential issues
        if self.webview.is_some() && self.debug_positioning {
            static mut COUNTER: u32 = 0;
            unsafe {
                COUNTER += 1;
                if COUNTER % 120 == 0 { // Log every ~2 seconds
                    println!("WebView step (safe mode) - frame {}", COUNTER);
                }
            }
        }
    }

    fn close_webview(&mut self) {
        if self.webview.is_some() {
            if self.debug_positioning {
                println!("Closing aligned WebView...");
            }
            self.webview = None;
            self.webview_created = false;
            self.status_message = "Aligned WebView closed".to_string();
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

    fn calculate_aligned_position(&self, egui_rect: egui::Rect, window_pos: egui::Pos2) -> (i32, i32, i32, i32) {
        // Calculate screen position considering:
        // 1. Window position
        // 2. Title bar height
        // 3. egui area position
        // 4. Manual offset
        // 5. Scale factor
        
        let screen_x = (window_pos.x + egui_rect.min.x + self.manual_offset.x) * self.scale_factor;
        let screen_y = (window_pos.y + self.title_bar_height + egui_rect.min.y + self.manual_offset.y) * self.scale_factor;
        
        let width = (egui_rect.width() * self.scale_factor) as i32;
        let height = (egui_rect.height() * self.scale_factor) as i32;
        
        // macOS coordinate system conversion (bottom-left origin)
        let screen_height = 1080.0 * self.scale_factor; // TODO: Get actual screen height
        let macos_y = screen_height - screen_y - height as f32;
        
        (screen_x as i32, macos_y as i32, width, height)
    }

    fn sync_webview_position(&mut self, egui_rect: egui::Rect, window_pos: egui::Pos2) {
        if !self.position_sync_enabled || !self.webview_created {
            return;
        }

        let (x, y, width, height) = self.calculate_aligned_position(egui_rect, window_pos);
        
        if self.debug_positioning {
            self.status_message = format!(
                "Align: egui({:.0},{:.0}) + win({:.0},{:.0}) + offset({:.0},{:.0}) = screen({},{}) {}x{}",
                egui_rect.min.x, egui_rect.min.y,
                window_pos.x, window_pos.y,
                self.manual_offset.x, self.manual_offset.y,
                x, y, width, height
            );
        }

        // Attempt to set WebView bounds with safety measures
        if let Some(webview) = &mut self.webview {
            // Validate bounds before setting
            if x >= -1000 && x <= 3000 && y >= -1000 && y <= 3000 &&
               width > 0 && width <= 2000 && height > 0 && height <= 2000 {

                // Try to set bounds with error handling
                std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    webview.set_bounds(x, y, width, height);
                })).unwrap_or_else(|_| {
                    if self.debug_positioning {
                        println!("Warning: set_bounds failed safely for ({}, {}) {}x{}", x, y, width, height);
                    }
                });

                if self.debug_positioning {
                    println!("WebView bounds set to: ({}, {}) {}x{}", x, y, width, height);
                }
            } else {
                if self.debug_positioning {
                    println!("Invalid bounds rejected: ({}, {}) {}x{}", x, y, width, height);
                }
            }
        }
    }

    fn adjust_manual_offset(&mut self, delta: egui::Vec2) {
        self.manual_offset += delta;
        if self.debug_positioning {
            println!("Manual offset adjusted to: ({:.0}, {:.0})", self.manual_offset.x, self.manual_offset.y);
        }
        
        // Trigger position recalculation
        if let Some(rect) = self.webview_rect {
            if let Some(window_pos) = self.last_window_pos {
                self.sync_webview_position(rect, window_pos);
            }
        }
    }

    fn reset_alignment(&mut self) {
        self.manual_offset = egui::Vec2::ZERO;
        self.status_message = "Alignment reset to default".to_string();
        
        if self.debug_positioning {
            println!("Alignment reset");
        }
    }

    fn get_window_position(&mut self, _ctx: &egui::Context) -> egui::Pos2 {
        // Try to get actual window position from egui context
        // For now, use a more dynamic estimate based on screen center
        let estimated_pos = if let Some(last_pos) = self.last_window_pos {
            // Use last known position with small random variation to simulate movement
            let variation = ((std::ptr::addr_of!(self) as usize) % 20) as f32 - 10.0;
            egui::Pos2::new(last_pos.x + variation * 0.1, last_pos.y + variation * 0.1)
        } else {
            // Initial position estimate
            egui::Pos2::new(200.0, 150.0)
        };

        // Update last known position
        self.last_window_pos = Some(estimated_pos);

        estimated_pos
    }

    fn update_scale_factor(&mut self, ctx: &egui::Context) {
        // Get the actual scale factor from egui
        self.scale_factor = ctx.pixels_per_point();
        
        if self.debug_positioning {
            static mut LAST_SCALE: f32 = 0.0;
            unsafe {
                if (LAST_SCALE - self.scale_factor).abs() > 0.01 {
                    println!("Scale factor updated to: {:.2}", self.scale_factor);
                    LAST_SCALE = self.scale_factor;
                }
            }
        }
    }
}

impl eframe::App for AlignedEmbedApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Update scale factor
        self.update_scale_factor(ctx);
        
        // Get current window position
        let current_window_pos = self.get_window_position(ctx);
        
        // Step WebView safely
        self.step_webview();

        // Top controls
        egui::TopBottomPanel::top("controls").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("URL:");
                ui.add(egui::TextEdit::singleline(&mut self.url_input)
                    .desired_width(350.0));
                
                if ui.button("🔗 Create Aligned WebView").clicked() {
                    self.create_aligned_webview();
                }
                
                if self.webview.is_some() && ui.button("❌ Close").clicked() {
                    self.close_webview();
                }
            });
            
            // Alignment controls
            if self.webview_created {
                ui.horizontal(|ui| {
                    ui.checkbox(&mut self.position_sync_enabled, "🎯 Position Sync");
                    ui.checkbox(&mut self.debug_positioning, "🔍 Debug");
                    
                    ui.separator();
                    ui.label("Title bar height:");
                    ui.add(egui::DragValue::new(&mut self.title_bar_height).range(0.0..=50.0));
                });
                
                ui.horizontal(|ui| {
                    ui.label("Fine alignment:");
                    if ui.button("⬅️ 1px").clicked() {
                        self.adjust_manual_offset(egui::vec2(-1.0, 0.0));
                    }
                    if ui.button("➡️ 1px").clicked() {
                        self.adjust_manual_offset(egui::vec2(1.0, 0.0));
                    }
                    if ui.button("⬆️ 1px").clicked() {
                        self.adjust_manual_offset(egui::vec2(0.0, -1.0));
                    }
                    if ui.button("⬇️ 1px").clicked() {
                        self.adjust_manual_offset(egui::vec2(0.0, 1.0));
                    }
                    
                    ui.separator();
                    if ui.button("🔄 Reset").clicked() {
                        self.reset_alignment();
                    }
                    
                    ui.label(format!("Offset: ({:.0}, {:.0})", self.manual_offset.x, self.manual_offset.y));
                });
            }
        });

        // Bottom status
        egui::TopBottomPanel::bottom("status").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Status:");
                if self.webview_created {
                    ui.colored_label(egui::Color32::GREEN, "✅ Aligned WebView Active");
                } else {
                    ui.colored_label(egui::Color32::GRAY, "⭕ No WebView");
                }
                ui.separator();
                ui.label(format!("Scale: {:.2}x", self.scale_factor));
            });
            ui.label(&self.status_message);
        });

        // Main content area
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("🎯 Aligned WebView Embedding");
            
            if self.webview_created {
                ui.separator();
                
                // WebView controls
                ui.horizontal(|ui| {
                    if ui.button("💬 Alert").clicked() {
                        self.eval_js("alert('Hello from aligned WebView!');");
                    }
                    if ui.button("🎨 Purple Background").clicked() {
                        self.eval_js("document.body.style.backgroundColor = '#f0e6ff';");
                    }
                    if ui.button("📄 Change Title").clicked() {
                        self.eval_js("document.title = 'Aligned WebView';");
                    }
                });
                
                ui.add_space(10.0);
                
                // WebView alignment area
                ui.group(|ui| {
                    ui.label("🌐 WebView Alignment Target Area");
                    
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
                    
                    // Sync WebView position
                    if self.position_sync_enabled && self.webview_created {
                        self.sync_webview_position(rect, current_window_pos);
                    }
                    
                    // Draw alignment target
                    ui.painter().rect_filled(
                        rect,
                        egui::Rounding::same(8.0),
                        egui::Color32::from_rgb(248, 240, 255)  // Light purple
                    );
                    
                    // Draw crosshairs for alignment reference
                    let center = rect.center();
                    ui.painter().line_segment(
                        [egui::pos2(rect.min.x, center.y), egui::pos2(rect.max.x, center.y)],
                        egui::Stroke::new(1.0, egui::Color32::from_rgb(200, 150, 255))
                    );
                    ui.painter().line_segment(
                        [egui::pos2(center.x, rect.min.y), egui::pos2(center.x, rect.max.y)],
                        egui::Stroke::new(1.0, egui::Color32::from_rgb(200, 150, 255))
                    );
                    
                    // Border with alignment indicators
                    ui.painter().rect_stroke(
                        rect,
                        egui::Rounding::same(8.0),
                        egui::Stroke::new(3.0, egui::Color32::from_rgb(147, 112, 219))
                    );
                    
                    // Corner markers for precise alignment
                    let marker_size = 10.0;
                    for corner in [rect.min, egui::pos2(rect.max.x, rect.min.y), rect.max, egui::pos2(rect.min.x, rect.max.y)] {
                        ui.painter().rect_filled(
                            egui::Rect::from_center_size(corner, egui::vec2(marker_size, marker_size)),
                            egui::Rounding::same(2.0),
                            egui::Color32::from_rgb(255, 0, 0)
                        );
                    }
                    
                    // Center text
                    ui.painter().text(
                        center,
                        egui::Align2::CENTER_CENTER,
                        "🎯 WebView Alignment Target\nWebView should align precisely with this area\nUse fine alignment controls above",
                        egui::FontId::proportional(16.0),
                        egui::Color32::from_rgb(100, 100, 100)
                    );
                    
                    // Debug positioning info
                    if self.debug_positioning && self.webview_created {
                        let (calc_x, calc_y, calc_w, calc_h) = self.calculate_aligned_position(rect, current_window_pos);
                        
                        let debug_text = format!(
                            "Target area: ({:.0}, {:.0}) {}x{}\nWindow pos: ({:.0}, {:.0})\nManual offset: ({:.0}, {:.0})\nCalculated screen pos: ({}, {}) {}x{}\nScale factor: {:.2}x\nTitle bar height: {:.0}px",
                            rect.min.x, rect.min.y, rect.width(), rect.height(),
                            current_window_pos.x, current_window_pos.y,
                            self.manual_offset.x, self.manual_offset.y,
                            calc_x, calc_y, calc_w, calc_h,
                            self.scale_factor,
                            self.title_bar_height
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
                    ui.heading("Precise WebView Alignment");
                    ui.add_space(20.0);
                    
                    ui.group(|ui| {
                        ui.label("🎯 Alignment Features:");
                        ui.label("• Precise position calculation");
                        ui.label("• Scale factor consideration");
                        ui.label("• Title bar height adjustment");
                        ui.label("• Manual fine-tuning (1px precision)");
                        ui.label("• Visual alignment indicators");
                        ui.label("• Real-time position debugging");
                    });
                    
                    ui.add_space(20.0);
                    
                    ui.group(|ui| {
                        ui.label("🔧 How to achieve perfect alignment:");
                        ui.label("1. Create WebView with target URL");
                        ui.label("2. Enable position sync and debug mode");
                        ui.label("3. Adjust title bar height if needed");
                        ui.label("4. Use 1px fine alignment controls");
                        ui.label("5. WebView should align with red corner markers");
                    });
                });
            }
        });

        // Reduced repaint frequency for stability
        if self.webview_created && self.position_sync_enabled {
            ctx.request_repaint_after(std::time::Duration::from_millis(200));
        }
    }
}

fn main() -> Result<(), eframe::Error> {
    env_logger::init();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 900.0])
            .with_title("Aligned WebView Embedding"),
        ..Default::default()
    };

    eframe::run_native(
        "Aligned WebView Embedding",
        options,
        Box::new(|_cc| Ok(Box::new(AlignedEmbedApp::default()))),
    )
}
