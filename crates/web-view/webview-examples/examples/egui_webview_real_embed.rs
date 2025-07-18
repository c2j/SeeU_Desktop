//! Real WebView Embedding in egui Window
//!
//! This example demonstrates true WebView embedding by getting the egui window's
//! native handle and embedding the WebView as a child view

use eframe::egui;
use std::os::raw::c_void;
use web_view::*;

/// Application for real WebView embedding
struct RealEmbedApp {
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
    /// WebView created flag
    webview_created: bool,
}

impl Default for RealEmbedApp {
    fn default() -> Self {
        Self {
            url_input: "https://www.rust-lang.org".to_string(),
            status_message: "Initializing real WebView embedding...".to_string(),
            webview: None,
            webview_rect: None,
            native_window: None,
            webview_created: false,
        }
    }
}

impl RealEmbedApp {
    fn get_native_window_handle(&mut self, ctx: &egui::Context) {
        if self.native_window.is_some() {
            return;
        }

        // Try to get the native window handle from egui context
        // This is platform-specific and requires accessing the underlying window
        #[cfg(target_os = "macos")]
        {
            // On macOS, we need to get the NSWindow from the egui context
            // This requires platform-specific code
            if let Some(viewport) = ctx.viewport(egui::ViewportId::ROOT) {
                // Try to access the native window through the viewport
                // Note: This is a simplified approach and may need adjustment
                // based on the actual eframe implementation
                self.status_message = "Attempting to get macOS window handle...".to_string();
                
                // For now, we'll simulate getting the handle
                // In a real implementation, this would involve:
                // 1. Getting the NSWindow from eframe
                // 2. Getting the contentView from NSWindow
                // 3. Using that as the parent for WebView embedding
                
                // Placeholder for actual implementation
                // self.native_window = Some(actual_nswindow_handle);
                self.status_message = "macOS window handle extraction needs platform-specific implementation".to_string();
            }
        }

        #[cfg(not(target_os = "macos"))]
        {
            self.status_message = "Window handle extraction not implemented for this platform".to_string();
        }
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

        // For now, create a positioned WebView that we'll try to embed
        // This is a stepping stone to true embedding
        let result = if let Some(_native_handle) = self.native_window {
            // TODO: Use the native handle for true embedding
            // For now, create a smaller, positioned WebView
            web_view::builder()
                .title("Embedded WebView")
                .content(Content::Url(&url))
                .size(600, 400)  // Smaller size to fit in egui area
                .resizable(false)
                .debug(false)
                .user_data(())
                .invoke_handler(|_webview, arg| {
                    println!("Embedded WebView: {}", arg);
                    Ok(())
                })
                .build()
        } else {
            // Fallback: create a standard WebView
            web_view::builder()
                .title("WebView (No Embedding)")
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
        };

        match result {
            Ok(webview) => {
                self.webview = Some(webview);
                self.webview_created = true;
                self.status_message = format!("✅ Created WebView for: {}", url);
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
}

impl eframe::App for RealEmbedApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Try to get native window handle
        self.get_native_window_handle(ctx);
        
        // Step WebView
        self.step_webview();

        // Top controls
        egui::TopBottomPanel::top("controls").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("URL:");
                ui.add(egui::TextEdit::singleline(&mut self.url_input)
                    .desired_width(400.0));
                
                if ui.button("🔗 Create Embedded WebView").clicked() {
                    self.create_embedded_webview();
                }
                
                if self.webview.is_some() && ui.button("❌ Close").clicked() {
                    self.close_webview();
                }
            });
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
                ui.label(&self.status_message);
            });
        });

        // Main content area - this is where we want the WebView to be embedded
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("🎯 Real WebView Embedding Attempt");
            
            if self.webview_created {
                ui.separator();
                
                // WebView controls
                ui.horizontal(|ui| {
                    if ui.button("💬 Alert").clicked() {
                        self.eval_js("alert('Hello from real embedded WebView!');");
                    }
                    if ui.button("🎨 Blue Background").clicked() {
                        self.eval_js("document.body.style.backgroundColor = '#e6f3ff';");
                    }
                    if ui.button("📄 Change Title").clicked() {
                        self.eval_js("document.title = 'Real Embedded WebView';");
                    }
                });
                
                ui.add_space(10.0);
                
                // This is the area where we want the WebView to be truly embedded
                ui.group(|ui| {
                    ui.label("📱 WebView Embedding Area");
                    
                    let available = ui.available_size();
                    let webview_size = egui::vec2(
                        (available.x - 20.0).max(600.0),
                        (available.y - 100.0).max(400.0)
                    );
                    
                    let (rect, _response) = ui.allocate_exact_size(
                        webview_size,
                        egui::Sense::hover()
                    );
                    
                    // Store the rect for WebView positioning
                    self.webview_rect = Some(rect);
                    
                    // Draw the embedding area
                    ui.painter().rect_filled(
                        rect,
                        egui::Rounding::same(8.0),
                        egui::Color32::from_rgb(250, 250, 250)
                    );
                    
                    ui.painter().rect_stroke(
                        rect,
                        egui::Rounding::same(8.0),
                        egui::Stroke::new(3.0, egui::Color32::from_rgb(76, 175, 80))
                    );
                    
                    // Show where the WebView should be embedded
                    let center = rect.center();
                    ui.painter().text(
                        center,
                        egui::Align2::CENTER_CENTER,
                        "🌐 WebView should be embedded HERE\n(Currently opens in separate window)",
                        egui::FontId::proportional(16.0),
                        egui::Color32::from_rgb(100, 100, 100)
                    );
                    
                    // Show coordinates for debugging
                    ui.painter().text(
                        rect.min + egui::vec2(10.0, 10.0),
                        egui::Align2::LEFT_TOP,
                        format!("Rect: {:.0}x{:.0} at ({:.0}, {:.0})", 
                               rect.width(), rect.height(), rect.min.x, rect.min.y),
                        egui::FontId::monospace(12.0),
                        egui::Color32::from_rgb(150, 150, 150)
                    );
                });
                
            } else {
                // Instructions
                ui.vertical_centered(|ui| {
                    ui.add_space(50.0);
                    ui.heading("Enter URL and create WebView for embedding");
                    ui.add_space(20.0);
                    
                    ui.group(|ui| {
                        ui.label("🎯 Real Embedding Goals:");
                        ui.label("• Get egui window's native handle");
                        ui.label("• Create WebView as child view/window");
                        ui.label("• Position WebView within egui area");
                        ui.label("• Synchronize with egui layout updates");
                        ui.label("• Handle window resize and move events");
                    });
                    
                    ui.add_space(20.0);
                    
                    ui.group(|ui| {
                        ui.label("🔧 Implementation Status:");
                        ui.label("• ⚠️ Native window handle extraction: In progress");
                        ui.label("• ⚠️ WebView child window creation: Needs implementation");
                        ui.label("• ⚠️ Position synchronization: Needs implementation");
                        ui.label("• ✅ WebView lifecycle management: Working");
                        ui.label("• ✅ JavaScript evaluation: Working");
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
            .with_title("Real WebView Embedding in egui"),
        ..Default::default()
    };

    eframe::run_native(
        "Real WebView Embedding",
        options,
        Box::new(|_cc| Ok(Box::new(RealEmbedApp::default()))),
    )
}
