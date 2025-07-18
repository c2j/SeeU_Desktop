//! Simplified Browser UI - focused on embedded WebView display

use eframe::egui;
use crate::state::IBrowserState;

/// Render the simplified browser interface
pub fn render_ibrowser(ui: &mut egui::Ui, state: &mut IBrowserState) {
    ui.vertical(|ui| {
        // Simple URL input bar
        ui.horizontal(|ui| {
            ui.label("网址:");
            let response = ui.add(
                egui::TextEdit::singleline(&mut state.url_input)
                    .desired_width(ui.available_width() - 80.0)
                    .hint_text("输入网址...")
            );
            
            // Handle Enter key
            if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                navigate_and_show_webview(state);
            }
            
            // Go button
            if ui.button("转到").clicked() {
                navigate_and_show_webview(state);
            }

            // Test simple WebView button
            if ui.button("测试简单WebView").clicked() {
                let url = if state.url_input.is_empty() {
                    "https://www.google.com".to_string()
                } else {
                    state.url_input.clone()
                };

                let formatted_url = if !url.starts_with("http://") && !url.starts_with("https://") {
                    format!("https://{}", url)
                } else {
                    url
                };

                log::info!("Testing simple WebView with URL: {}", formatted_url);
                if let Err(e) = crate::simple_webview::create_simple_webview(&formatted_url) {
                    log::error!("Failed to create simple WebView: {}", e);
                }
            }
        });
        
        ui.add_space(10.0);
        
        // Main WebView content area
        render_webview_content(ui, state);
    });
}

/// Navigate to URL and show embedded WebView
fn navigate_and_show_webview(state: &mut IBrowserState) {
    let url = state.url_input.clone();
    if !url.is_empty() {
        let formatted_url = if !url.starts_with("http://") && !url.starts_with("https://") {
            format!("https://{}", url)
        } else {
            url
        };
        log::info!("Navigating to: {}", formatted_url);
        state.navigate_to(formatted_url);
        state.launch_embedded_native_webview();
    }
}

/// Render the main WebView content area
fn render_webview_content(ui: &mut egui::Ui, state: &mut IBrowserState) {
    // Check if we have a running native WebView
    let has_running_webview = state.native_webview.as_ref()
        .map_or(false, |nw| nw.lock().unwrap().is_running());

    if has_running_webview {
        // Show the embedded WebView area
        render_embedded_webview(ui, state);
    } else {
        // Show placeholder
        render_placeholder(ui, state);
    }
}

/// Render the embedded WebView area
fn render_embedded_webview(ui: &mut egui::Ui, state: &mut IBrowserState) {
    // Step the embedded WebView if it exists (for macOS embedded mode)
    if let Some(native_webview) = &state.native_webview {
        if let Ok(mut webview) = native_webview.lock() {
            if webview.step_embedded() {
                // WebView is running, request continuous repaint
                ui.ctx().request_repaint();
            }
        }
    }

    // Create a frame for the WebView
    egui::Frame::none()
        .fill(egui::Color32::WHITE)
        .stroke(egui::Stroke::new(2.0, egui::Color32::DARK_GRAY))
        .inner_margin(egui::Margin::same(0.0))
        .show(ui, |ui| {
            ui.vertical(|ui| {
                // Simple header
                egui::Frame::none()
                    .fill(egui::Color32::from_gray(240))
                    .inner_margin(egui::Margin::same(8.0))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label("🌐");
                            ui.label(egui::RichText::new("内置 WebView").strong());
                            ui.separator();
                            ui.label(egui::RichText::new(&state.current_url).small().color(egui::Color32::DARK_GRAY));

                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if ui.small_button("❌").clicked() {
                                    state.close_native_webview();
                                }
                            });
                        });
                    });

                // WebView content area - this is where we need precise positioning
                let available_height = ui.available_height() - 10.0;
                let content_height = available_height.max(400.0);

                // Allocate space for the WebView and get its screen coordinates
                let webview_response = ui.allocate_response(
                    egui::Vec2::new(ui.available_width(), content_height),
                    egui::Sense::hover()
                );

                // Calculate precise screen coordinates for WebView positioning
                let ctx = ui.ctx();
                let pixels_per_point = ctx.pixels_per_point();

                // Get the WebView area in screen coordinates
                let webview_rect = webview_response.rect;
                let screen_x = webview_rect.min.x * pixels_per_point;
                let screen_y = webview_rect.min.y * pixels_per_point;
                let screen_width = webview_rect.width() * pixels_per_point;
                let screen_height = webview_rect.height() * pixels_per_point;

                // Try to create stable embedded WebView
                if let Some(native_webview) = &state.native_webview {
                    if let Ok(mut webview) = native_webview.lock() {
                        if !webview.is_running() && !webview.webview_created {
                            log::info!("Creating stable embedded WebView at screen coords ({:.1}, {:.1}) with size {:.1}x{:.1}",
                                screen_x, screen_y, screen_width, screen_height);

                            let rect = Some((screen_x, screen_y, screen_width, screen_height));
                            if let Err(e) = webview.create_window_with_rect(&state.current_url, "iBrowser", rect) {
                                log::error!("Failed to create stable embedded WebView: {}", e);
                            }
                        }
                    }
                }

                // Draw WebView placeholder background
                ui.painter().rect_filled(
                    webview_response.rect,
                    egui::Rounding::same(4.0),
                    egui::Color32::from_gray(245)
                );

                // Add positioning info text
                let text_pos = webview_response.rect.center();
                ui.painter().text(
                    text_pos,
                    egui::Align2::CENTER_CENTER,
                    format!("🌐 WebView 定位区域\n屏幕坐标: ({:.0}, {:.0})\n尺寸: {:.0} x {:.0}",
                        screen_x, screen_y, screen_width, screen_height),
                    egui::FontId::proportional(14.0),
                    egui::Color32::DARK_GRAY
                );

                // Add a border to show the WebView area
                ui.painter().rect_stroke(
                    webview_response.rect,
                    egui::Rounding::same(4.0),
                    egui::Stroke::new(2.0, egui::Color32::from_rgb(100, 150, 255))
                );
            });
        });
}

/// Render placeholder when no WebView is active
fn render_placeholder(ui: &mut egui::Ui, state: &mut IBrowserState) {
    ui.vertical_centered(|ui| {
        ui.add_space(150.0);
        ui.label(egui::RichText::new("🌐 WebView 浏览器").heading());
        ui.add_space(20.0);
        
        if !state.current_url.is_empty() {
            ui.label(format!("准备加载: {}", state.current_url));
            ui.add_space(10.0);
        }
        
        ui.label("输入网址并点击转到按钮开始浏览");
        ui.add_space(150.0);
    });
}
