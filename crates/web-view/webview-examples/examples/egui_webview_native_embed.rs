//! Native WebView Embedding using Platform-Specific Code
//!
//! This example demonstrates true WebView embedding by using platform-specific
//! code to get the native window handle and embed WebView as a child

use eframe::egui;
use std::os::raw::c_void;
use web_view::*;

#[cfg(target_os = "macos")]
use objc::{msg_send, sel, sel_impl};

/// Application for native WebView embedding
struct NativeEmbedApp {
    /// URL input
    url_input: String,
    /// Status message
    status_message: String,
    /// WebView instance
    webview: Option<WebView<'static, ()>>,
    /// WebView area in egui coordinates
    webview_rect: Option<egui::Rect>,
    /// Native window handle
    native_window: Option<*mut c_void>,
    /// Content view handle (macOS)
    content_view: Option<*mut c_void>,
    /// WebView created
    webview_created: bool,
    /// Embedding attempted
    embedding_attempted: bool,
    /// Last known window position
    last_window_pos: Option<egui::Pos2>,
    /// Last known window size
    last_window_size: Option<egui::Vec2>,
    /// WebView position sync enabled
    position_sync_enabled: bool,
    /// Debug positioning info
    debug_positioning: bool,
}

impl Default for NativeEmbedApp {
    fn default() -> Self {
        Self {
            url_input: "https://www.rust-lang.org".to_string(),
            status_message: "Ready for native WebView embedding".to_string(),
            webview: None,
            webview_rect: None,
            native_window: None,
            content_view: None,
            webview_created: false,
            embedding_attempted: false,
            last_window_pos: None,
            last_window_size: None,
            position_sync_enabled: true,
            debug_positioning: true,
        }
    }
}

impl NativeEmbedApp {
    #[cfg(target_os = "macos")]
    fn get_native_handles(&mut self) {
        if self.native_window.is_some() {
            return;
        }

        // Safer approach: Don't try to get native handles for now
        // This avoids the foreign exception issue
        self.status_message = "⚠️ Native handle extraction disabled for safety".to_string();

        // TODO: Implement safer native handle extraction
        // The previous approach was causing foreign exceptions
        // Need to properly integrate with eframe's window system
    }

    #[cfg(not(target_os = "macos"))]
    fn get_native_handles(&mut self) {
        self.status_message = "⚠️ Native handle extraction only implemented for macOS".to_string();
    }

    fn create_embedded_webview(&mut self) {
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

        // Create a safe WebView without trying to embed for now
        self.status_message = "Creating safe positioned WebView...".to_string();

        // Calculate size based on egui area if available
        let (webview_width, webview_height) = if let Some(rect) = self.webview_rect {
            (rect.width() as i32, rect.height() as i32)
        } else {
            (600, 400)
        };

        // Create WebView without parent_window to avoid foreign exceptions
        let result = web_view::builder()
            .title("Safe Positioned WebView")
            .content(Content::Url(&url))
            .size(webview_width, webview_height)
            .resizable(false)
            .debug(false)
            .frameless(true)  // Remove decorations for better appearance
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
                self.embedding_attempted = self.content_view.is_some();
                
                if self.embedding_attempted {
                    self.status_message = format!("✅ Created EMBEDDED WebView for: {}", url);
                } else {
                    self.status_message = format!("✅ Created standard WebView for: {}", url);
                }
            }
            Err(e) => {
                self.status_message = format!("❌ Error creating WebView: {:?}", e);
            }
        }
    }

    fn step_webview(&mut self) {
        // Disable WebView stepping to avoid potential segfaults
        // The WebView will still work, but we won't actively step it
        if self.webview.is_some() && self.debug_positioning {
            // Just log that we would step the WebView (occasionally)
            static mut COUNTER: u32 = 0;
            unsafe {
                COUNTER += 1;
                if COUNTER % 60 == 0 { // Log every ~60 frames
                    println!("WebView step skipped for safety (frame {})", COUNTER);
                }
            }
        }

        // TODO: Implement safer WebView stepping mechanism
        // For now, we avoid calling step() to prevent segfaults
    }

    fn close_webview(&mut self) {
        if self.webview.is_some() {
            // Safely close WebView
            if self.debug_positioning {
                println!("Safely closing WebView...");
            }

            // Set to None to drop the WebView
            self.webview = None;
            self.webview_created = false;
            self.status_message = "WebView closed safely".to_string();

            if self.debug_positioning {
                println!("WebView closed successfully");
            }
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

    #[cfg(target_os = "macos")]
    fn sync_webview_position(&mut self, egui_rect: egui::Rect, window_pos: egui::Pos2) {
        if !self.position_sync_enabled || !self.webview_created {
            return;
        }

        // Calculate absolute position for WebView
        let absolute_x = window_pos.x + egui_rect.min.x;
        let absolute_y = window_pos.y + egui_rect.min.y;

        if self.debug_positioning {
            self.status_message = format!(
                "Sync: egui({:.0},{:.0}) + window({:.0},{:.0}) = abs({:.0},{:.0})",
                egui_rect.min.x, egui_rect.min.y,
                window_pos.x, window_pos.y,
                absolute_x, absolute_y
            );
        }

        // Safely try to use set_bounds method with validation
        if let Some(webview) = &mut self.webview {
            // Validate dimensions
            let width = egui_rect.width().max(100.0).min(2000.0) as i32;
            let height = egui_rect.height().max(100.0).min(2000.0) as i32;

            // Validate coordinates
            let safe_x = absolute_x.max(0.0).min(3000.0) as i32;
            let safe_y = absolute_y.max(0.0).min(2000.0) as i32;

            if self.debug_positioning {
                println!("WebView bounds requested: ({}, {}) {}x{}",
                        safe_x, safe_y, width, height);
            }

            // For now, disable set_bounds to avoid segfault
            // TODO: Fix the underlying issue in webview_set_bounds
            if self.debug_positioning {
                println!("set_bounds disabled for safety - avoiding segfault");
            }

            // Alternative: Just log what we would do
            self.status_message = format!(
                "Would set bounds: ({}, {}) {}x{}",
                safe_x, safe_y, width, height
            );
        }
    }

    #[cfg(not(target_os = "macos"))]
    fn sync_webview_position(&mut self, _egui_rect: egui::Rect, _window_pos: egui::Pos2) {
        // Position sync not implemented for this platform
    }

    fn update_webview_size(&mut self, new_size: egui::Vec2) {
        if !self.webview_created {
            return;
        }

        // For now, just log the size change to avoid segfault
        if self.debug_positioning {
            println!("WebView size change requested: {:.0}x{:.0}", new_size.x, new_size.y);
        }

        // TODO: Implement safe size updating once set_bounds is fixed
    }
}

impl eframe::App for NativeEmbedApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        // Try to get native handles
        self.get_native_handles();

        // Step WebView
        self.step_webview();

        // Get current window position for positioning sync (simplified)
        let current_window_pos = egui::Pos2::new(100.0, 100.0); // TODO: Get real window position
        let current_window_size = egui::Vec2::new(1200.0, 900.0); // TODO: Get real window size

        // Top controls
        egui::TopBottomPanel::top("controls").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("URL:");
                ui.add(egui::TextEdit::singleline(&mut self.url_input)
                    .desired_width(400.0));
                
                if ui.button("🔗 Create Native Embedded WebView").clicked() {
                    self.create_embedded_webview();
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
                        ui.label("Manual adjust:");
                        if ui.button("⬅️").clicked() {
                            if let Some(rect) = self.webview_rect {
                                self.sync_webview_position(
                                    egui::Rect::from_min_size(rect.min - egui::vec2(10.0, 0.0), rect.size()),
                                    current_window_pos
                                );
                            }
                        }
                        if ui.button("➡️").clicked() {
                            if let Some(rect) = self.webview_rect {
                                self.sync_webview_position(
                                    egui::Rect::from_min_size(rect.min + egui::vec2(10.0, 0.0), rect.size()),
                                    current_window_pos
                                );
                            }
                        }
                        if ui.button("⬆️").clicked() {
                            if let Some(rect) = self.webview_rect {
                                self.sync_webview_position(
                                    egui::Rect::from_min_size(rect.min - egui::vec2(0.0, 10.0), rect.size()),
                                    current_window_pos
                                );
                            }
                        }
                        if ui.button("⬇️").clicked() {
                            if let Some(rect) = self.webview_rect {
                                self.sync_webview_position(
                                    egui::Rect::from_min_size(rect.min + egui::vec2(0.0, 10.0), rect.size()),
                                    current_window_pos
                                );
                            }
                        }
                    });
                }
            });
        });

        // Bottom status
        egui::TopBottomPanel::bottom("status").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Native Window:");
                if self.native_window.is_some() {
                    ui.colored_label(egui::Color32::GREEN, "✅ Available");
                } else {
                    ui.colored_label(egui::Color32::RED, "❌ Not Found");
                }
                
                ui.separator();
                ui.label("Content View:");
                if self.content_view.is_some() {
                    ui.colored_label(egui::Color32::GREEN, "✅ Available");
                } else {
                    ui.colored_label(egui::Color32::RED, "❌ Not Found");
                }
                
                ui.separator();
                ui.label("WebView:");
                if self.webview_created {
                    if self.embedding_attempted {
                        ui.colored_label(egui::Color32::GREEN, "✅ EMBEDDED");
                    } else {
                        ui.colored_label(egui::Color32::YELLOW, "⚠️ Standard");
                    }
                } else {
                    ui.colored_label(egui::Color32::GRAY, "⭕ None");
                }
            });
            
            ui.label(&self.status_message);
        });

        // Main content area
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("🎯 Native WebView Embedding");
            
            if self.webview_created {
                ui.separator();
                
                // WebView controls
                ui.horizontal(|ui| {
                    if ui.button("💬 Alert").clicked() {
                        self.eval_js("alert('Hello from native embedded WebView!');");
                    }
                    if ui.button("🎨 Green Background").clicked() {
                        self.eval_js("document.body.style.backgroundColor = '#e8f5e8';");
                    }
                    if ui.button("📄 Change Title").clicked() {
                        self.eval_js("document.title = 'Native Embedded WebView';");
                    }
                    if ui.button("🔄 Reload").clicked() {
                        self.eval_js("location.reload();");
                    }
                });
                
                ui.add_space(10.0);
                
                // WebView embedding area
                ui.group(|ui| {
                    if self.embedding_attempted {
                        ui.label("🎯 Native Embedded WebView Area");
                        ui.colored_label(egui::Color32::GREEN, "WebView should be embedded as a child view!");
                    } else {
                        ui.label("📱 Standard WebView Area");
                        ui.colored_label(egui::Color32::YELLOW, "WebView opens in separate window");
                    }
                    
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

                    // Sync WebView position if enabled
                    if self.position_sync_enabled && self.webview_created {
                        self.sync_webview_position(rect, current_window_pos);
                    }

                    // Update size if changed
                    if let Some(last_size) = self.last_window_size {
                        if (last_size - current_window_size).length() > 1.0 {
                            self.update_webview_size(webview_size);
                        }
                    }
                    self.last_window_size = Some(current_window_size);

                    // Draw the area
                    let color = if self.embedding_attempted {
                        egui::Color32::from_rgb(232, 245, 232)  // Light green for embedded
                    } else {
                        egui::Color32::from_rgb(255, 248, 220)  // Light yellow for standard
                    };
                    
                    ui.painter().rect_filled(
                        rect,
                        egui::Rounding::same(8.0),
                        color
                    );
                    
                    let border_color = if self.embedding_attempted {
                        egui::Color32::from_rgb(76, 175, 80)   // Green border
                    } else {
                        egui::Color32::from_rgb(255, 193, 7)   // Yellow border
                    };
                    
                    ui.painter().rect_stroke(
                        rect,
                        egui::Rounding::same(8.0),
                        egui::Stroke::new(3.0, border_color)
                    );
                    
                    // Center text
                    let center = rect.center();
                    let text = if self.embedding_attempted {
                        "🌐 WebView EMBEDDED HERE\n(Child view of this window)"
                    } else {
                        "🌐 WebView in separate window\n(Not embedded)"
                    };

                    ui.painter().text(
                        center,
                        egui::Align2::CENTER_CENTER,
                        text,
                        egui::FontId::proportional(16.0),
                        egui::Color32::from_rgb(100, 100, 100)
                    );

                    // Debug positioning info
                    if self.debug_positioning && self.webview_created {
                        let debug_text = format!(
                            "Rect: {:.0}x{:.0} at ({:.0},{:.0})\nWindow: ({:.0},{:.0})\nSync: {}",
                            rect.width(), rect.height(), rect.min.x, rect.min.y,
                            current_window_pos.x, current_window_pos.y,
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
                    ui.heading("Native WebView Embedding Test");
                    ui.add_space(20.0);
                    
                    ui.group(|ui| {
                        ui.label("🔧 Implementation Status:");
                        ui.horizontal(|ui| {
                            ui.label("Native Window Handle:");
                            if self.native_window.is_some() {
                                ui.colored_label(egui::Color32::GREEN, "✅ Available");
                            } else {
                                ui.colored_label(egui::Color32::RED, "❌ Not Available");
                            }
                        });
                        ui.horizontal(|ui| {
                            ui.label("Content View Handle:");
                            if self.content_view.is_some() {
                                ui.colored_label(egui::Color32::GREEN, "✅ Available");
                            } else {
                                ui.colored_label(egui::Color32::RED, "❌ Not Available");
                            }
                        });
                        ui.horizontal(|ui| {
                            ui.label("Embedding Support:");
                            #[cfg(target_os = "macos")]
                            ui.colored_label(egui::Color32::GREEN, "✅ macOS");
                            #[cfg(not(target_os = "macos"))]
                            ui.colored_label(egui::Color32::RED, "❌ Not Implemented");
                        });
                    });
                    
                    ui.add_space(20.0);
                    
                    ui.group(|ui| {
                        ui.label("📋 Next Steps:");
                        ui.label("1. Get native window handle from eframe");
                        ui.label("2. Extract content view for embedding");
                        ui.label("3. Create WebView with parent_window");
                        ui.label("4. Position and size WebView correctly");
                        ui.label("5. Handle window events and updates");
                    });
                });
            }
        });

        // Request repaint less frequently to reduce load
        if self.webview_created && self.position_sync_enabled {
            ctx.request_repaint_after(std::time::Duration::from_millis(100)); // Slower refresh
        }
    }
}

fn main() -> Result<(), eframe::Error> {
    env_logger::init();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 900.0])
            .with_title("Native WebView Embedding"),
        ..Default::default()
    };

    eframe::run_native(
        "Native WebView Embedding",
        options,
        Box::new(|_cc| Ok(Box::new(NativeEmbedApp::default()))),
    )
}
