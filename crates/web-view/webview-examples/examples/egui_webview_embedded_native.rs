//! Native Embedded WebView in egui Window
//!
//! This example attempts to embed WebView directly into the egui window
//! using native window handles and positioning

use eframe::egui;
use std::collections::HashMap;
use web_view::*;

/// Application for native embedded WebView
struct NativeEmbeddedApp {
    /// URL input
    url_input: String,
    /// Status message
    status_message: String,
    /// WebView instance
    webview: Option<WebView<'static, ()>>,
    /// WebView area position and size
    webview_rect: egui::Rect,
    /// Show WebView
    show_webview: bool,
    /// WebView created
    webview_created: bool,
}

impl Default for NativeEmbeddedApp {
    fn default() -> Self {
        Self {
            url_input: "https://www.rust-lang.org".to_string(),
            status_message: "Ready to embed WebView in egui window".to_string(),
            webview: None,
            webview_rect: egui::Rect::NOTHING,
            show_webview: true,
            webview_created: false,
        }
    }
}

impl NativeEmbeddedApp {
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

        // Create a smaller WebView for embedding
        match web_view::builder()
            .title("Embedded WebView")
            .content(Content::Url(&url))
            .size(600, 400)  // Smaller size for embedding
            .resizable(false) // Disable resize for better control
            .debug(false)
            .user_data(())
            .invoke_handler(|_webview, arg| {
                println!("Embedded WebView message: {}", arg);
                Ok(())
            })
            .build()
        {
            Ok(webview) => {
                self.webview = Some(webview);
                self.webview_created = true;
                self.status_message = format!("✅ Created embedded WebView: {}", url);
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
                    // WebView was closed
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
                    self.status_message = format!("✅ Executed JS: {}", js);
                }
                Err(e) => {
                    self.status_message = format!("❌ JS error: {:?}", e);
                }
            }
        }
    }
}

impl eframe::App for NativeEmbeddedApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        // Step WebView
        self.step_webview();

        // Main UI
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("🔗 Native Embedded WebView in egui");
            ui.separator();

            // Controls section
            ui.horizontal(|ui| {
                ui.label("URL:");
                ui.add(egui::TextEdit::singleline(&mut self.url_input)
                    .desired_width(300.0));
                
                if ui.button("🔗 Create Embedded WebView").clicked() {
                    self.create_webview();
                }
                
                if self.webview.is_some() && ui.button("❌ Close").clicked() {
                    self.close_webview();
                }
            });

            ui.add_space(10.0);

            // WebView controls
            if self.webview.is_some() {
                ui.horizontal(|ui| {
                    ui.checkbox(&mut self.show_webview, "Show WebView");
                    
                    if ui.button("💬 Alert").clicked() {
                        self.eval_js("alert('Hello from embedded WebView!');");
                    }
                    if ui.button("🎨 Blue BG").clicked() {
                        self.eval_js("document.body.style.backgroundColor = '#e6f3ff';");
                    }
                    if ui.button("🔄 Reload").clicked() {
                        self.eval_js("location.reload();");
                    }
                });
            }

            ui.add_space(10.0);

            // Status
            ui.label(&self.status_message);

            ui.add_space(10.0);

            // WebView embedding area
            if self.show_webview && self.webview_created {
                ui.group(|ui| {
                    ui.label("📱 Embedded WebView Area");
                    
                    // Reserve space for WebView
                    let available_size = ui.available_size();
                    let webview_size = egui::vec2(
                        (available_size.x - 20.0).max(400.0),
                        (available_size.y - 100.0).max(300.0)
                    );
                    
                    let (rect, _response) = ui.allocate_exact_size(
                        webview_size,
                        egui::Sense::hover()
                    );
                    
                    // Store the rect for potential native window positioning
                    self.webview_rect = rect;
                    
                    // Draw a placeholder border
                    ui.painter().rect_stroke(
                        rect,
                        egui::Rounding::same(4.0),
                        egui::Stroke::new(2.0, egui::Color32::from_rgb(100, 150, 200))
                    );
                    
                    // Draw info text
                    let text_pos = rect.center() - egui::vec2(100.0, 10.0);
                    ui.painter().text(
                        text_pos,
                        egui::Align2::CENTER_CENTER,
                        "WebView runs in separate window\n(Native embedding limitations)",
                        egui::FontId::default(),
                        egui::Color32::GRAY
                    );
                    
                    // Show WebView info
                    ui.label(format!("WebView area: {}x{}", rect.width(), rect.height()));
                    ui.label(format!("Position: ({:.0}, {:.0})", rect.min.x, rect.min.y));
                });
            }

            ui.add_space(10.0);

            // Information section
            ui.group(|ui| {
                ui.label("ℹ️ About native WebView embedding:");
                ui.label("• WebView creates its own native window");
                ui.label("• True pixel-level embedding requires platform-specific code");
                ui.label("• This example shows the reserved space for WebView");
                ui.label("• WebView window appears separately but can be positioned");
                ui.label("• For true embedding, consider web rendering alternatives");
            });

            ui.add_space(10.0);

            // Technical details
            ui.collapsing("🔧 Technical Limitations", |ui| {
                ui.label("Native WebView embedding challenges:");
                ui.label("• WebView is a native OS component");
                ui.label("• egui uses immediate mode rendering");
                ui.label("• Different rendering contexts (GPU vs native)");
                ui.label("• Platform-specific window parenting required");
                
                ui.add_space(5.0);
                ui.label("Possible solutions:");
                ui.label("• Use web rendering libraries (like servo)");
                ui.label("• Implement platform-specific window embedding");
                ui.label("• Use texture-based web rendering");
                ui.label("• Position WebView window to overlay egui area");
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
            .with_inner_size([1000.0, 800.0])
            .with_title("Native Embedded WebView in egui"),
        ..Default::default()
    };

    eframe::run_native(
        "Native Embedded WebView",
        options,
        Box::new(|_cc| Ok(Box::new(NativeEmbeddedApp::default()))),
    )
}
