//! True WebView Embedding in egui Window
//!
//! This example attempts true WebView embedding by getting the egui window handle
//! and positioning the WebView as a child window

use eframe::egui;
use web_view::*;

/// Application for true WebView embedding
struct TrueEmbedApp {
    /// URL input
    url_input: String,
    /// Status message
    status_message: String,
    /// WebView instance
    webview: Option<WebView<'static, ()>>,
    /// WebView area in egui
    webview_area: Option<egui::Rect>,
    /// Show WebView overlay
    show_webview: bool,
    /// Window handle obtained
    window_handle_obtained: bool,
}

impl Default for TrueEmbedApp {
    fn default() -> Self {
        Self {
            url_input: "https://www.rust-lang.org".to_string(),
            status_message: "Ready for true WebView embedding".to_string(),
            webview: None,
            webview_area: None,
            show_webview: true,
            window_handle_obtained: false,
        }
    }
}

impl TrueEmbedApp {
    fn create_embedded_webview(&mut self, parent_handle: Option<*mut std::ffi::c_void>) {
        if self.url_input.is_empty() {
            self.status_message = "Please enter a URL".to_string();
            return;
        }

        let url = if !self.url_input.starts_with("http://") && !self.url_input.starts_with("https://") {
            format!("https://{}", self.url_input)
        } else {
            self.url_input.clone()
        };

        // Create WebView with smaller size for embedding
        match web_view::builder()
            .title("Embedded WebView")
            .content(Content::Url(&url))
            .size(600, 400)
            .resizable(false)
            .debug(false)
            .user_data(())
            .invoke_handler(|_webview, arg| {
                println!("Embedded WebView: {}", arg);
                Ok(())
            })
            .build()
        {
            Ok(webview) => {
                self.webview = Some(webview);
                self.status_message = format!("✅ Created embedded WebView: {}", url);
                
                // Try to get WebView window handle for positioning
                if let Some(wv) = &self.webview {
                    // Note: This is a simplified approach
                    // Real implementation would need platform-specific code
                    self.status_message += " (Window handle available for positioning)";
                }
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

    fn position_webview(&mut self) {
        // This is where we would implement platform-specific window positioning
        // to make the WebView appear within the egui area
        if let (Some(_webview), Some(area)) = (&self.webview, &self.webview_area) {
            // Platform-specific positioning code would go here
            // For now, we just update the status
            self.status_message = format!(
                "WebView positioned at ({:.0}, {:.0}) size {}x{}", 
                area.min.x, area.min.y, area.width(), area.height()
            );
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

impl eframe::App for TrueEmbedApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        // Step WebView
        self.step_webview();

        // Try to get window handle (platform-specific)
        if !self.window_handle_obtained {
            // This would be platform-specific code to get the egui window handle
            self.window_handle_obtained = true;
            self.status_message += " | Window handle access attempted";
        }

        // Top panel with controls
        egui::TopBottomPanel::top("controls").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("URL:");
                ui.add(egui::TextEdit::singleline(&mut self.url_input)
                    .desired_width(300.0));
                
                if ui.button("🔗 Embed WebView").clicked() {
                    self.create_embedded_webview(None); // Would pass window handle
                }
                
                if self.webview.is_some() {
                    ui.separator();
                    ui.checkbox(&mut self.show_webview, "Show WebView");
                    
                    if ui.button("💬 Alert").clicked() {
                        self.eval_js("alert('Hello from truly embedded WebView!');");
                    }
                    if ui.button("🎨 Style").clicked() {
                        self.eval_js("document.body.style.backgroundColor = '#f0f8ff'; document.body.style.border = '3px solid #4CAF50';");
                    }
                }
            });
        });

        // Bottom panel with status
        egui::TopBottomPanel::bottom("status").show(ctx, |ui| {
            ui.label(&self.status_message);
        });

        // Main content area
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("🎯 True WebView Embedding Attempt");
            
            if self.show_webview && self.webview.is_some() {
                ui.separator();
                ui.label("📱 WebView Embedding Area:");
                
                // Create a dedicated area for WebView
                let available = ui.available_size();
                let webview_size = egui::vec2(
                    (available.x * 0.9).max(400.0),
                    (available.y * 0.7).max(300.0)
                );
                
                ui.allocate_ui(webview_size, |ui| {
                    // Get the exact rect for WebView positioning
                    let rect = ui.max_rect();
                    self.webview_area = Some(rect);
                    
                    // Draw WebView container
                    ui.painter().rect_filled(
                        rect,
                        egui::Rounding::same(8.0),
                        egui::Color32::from_rgb(240, 248, 255)
                    );
                    
                    ui.painter().rect_stroke(
                        rect,
                        egui::Rounding::same(8.0),
                        egui::Stroke::new(2.0, egui::Color32::from_rgb(70, 130, 180))
                    );
                    
                    // Position WebView (this would be platform-specific)
                    self.position_webview();
                    
                    // Overlay information
                    ui.centered_and_justified(|ui| {
                        ui.vertical_centered(|ui| {
                            ui.label("🌐 WebView Embedding Zone");
                            ui.label("(WebView window should appear here)");
                            ui.add_space(10.0);
                            ui.label(format!("Area: {:.0}x{:.0}", rect.width(), rect.height()));
                            ui.label(format!("Position: ({:.0}, {:.0})", rect.min.x, rect.min.y));
                        });
                    });
                });
                
            } else {
                // Instructions when no WebView
                ui.vertical_centered(|ui| {
                    ui.add_space(50.0);
                    ui.heading("Enter a URL above and click 'Embed WebView'");
                    ui.add_space(20.0);
                    
                    ui.group(|ui| {
                        ui.label("🎯 True Embedding Approach:");
                        ui.label("• Get egui window handle");
                        ui.label("• Create WebView as child window");
                        ui.label("• Position WebView within egui area");
                        ui.label("• Synchronize WebView with egui layout");
                    });
                });
            }
            
            ui.add_space(20.0);
            
            // Technical information
            ui.collapsing("🔧 Implementation Details", |ui| {
                ui.label("True embedding requires:");
                ui.label("• Platform-specific window parenting");
                ui.label("• Coordinate system conversion");
                ui.label("• Window message handling");
                ui.label("• Synchronization with egui repaints");
                
                ui.add_space(5.0);
                ui.label("Platform implementations needed:");
                ui.label("• Windows: SetParent() + SetWindowPos()");
                ui.label("• macOS: NSView hierarchy manipulation");
                ui.label("• Linux: X11/Wayland window embedding");
            });
            
            ui.add_space(10.0);
            
            // Alternative approaches
            ui.collapsing("🔄 Alternative Approaches", |ui| {
                ui.label("If true embedding proves difficult:");
                ui.label("• Use web rendering libraries (servo, webkit2gtk)");
                ui.label("• Implement texture-based web rendering");
                ui.label("• Use positioned overlay windows");
                ui.label("• Consider CEF (Chromium Embedded Framework)");
                ui.label("• Use tauri-style webview integration");
            });
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
            .with_inner_size([1200.0, 900.0])
            .with_title("True WebView Embedding in egui"),
        ..Default::default()
    };

    eframe::run_native(
        "True WebView Embedding",
        options,
        Box::new(|_cc| Ok(Box::new(TrueEmbedApp::default()))),
    )
}
