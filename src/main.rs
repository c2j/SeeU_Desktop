mod app;
mod ui;
mod modules;
mod services;
mod utils;
mod platform;
mod config;

use eframe::{self, egui};

fn main() -> Result<(), eframe::Error> {
    // Initialize custom logger with WARN level to reduce log noise
    utils::logger::Logger::init(log::LevelFilter::Info, true)
        .expect("Failed to initialize logger");

    // Log startup information
    log::info!("Starting SeeU Desktop v{}", env!("CARGO_PKG_VERSION"));
    log::info!("Log file location: {:?}", utils::logger::Logger::log_path());

    // TEMPORARILY DISABLED: Load saved window state
    // let window_state = load_window_state();

    // Log the loaded window state for debugging
    // log::info!("Loaded window state (physical pixels): {}x{} at ({}, {}), maximized: {}",
    //           window_state.width, window_state.height,
    //           window_state.x, window_state.y, window_state.maximized);

    // Setup native options with DEFAULT window state (no restoration)
    let viewport_builder = egui::ViewportBuilder::default()
        .with_inner_size([1280.0, 720.0])  // Use default size
        .with_min_inner_size([800.0, 600.0])
        .with_decorations(true)
        .with_transparent(false);

    // DISABLED: Apply maximized state if needed
    // if window_state.maximized {
    //     viewport_builder = viewport_builder.with_maximized(true);
    // }

    let native_options = eframe::NativeOptions {
        viewport: viewport_builder,
        ..Default::default()
    };

    // Run the application
    eframe::run_native(
        "SeeU Desktop",
        native_options,
        Box::new(|cc| Ok(Box::new(app::SeeUApp::new(cc))))
    )
}

/// Load saved window state or return default
fn load_window_state() -> app::WindowState {
    use std::fs;
    use serde_json;

    let base_path = dirs::config_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
    let config_path = base_path.join("seeu_desktop").join("app_settings.json");

    if let Ok(content) = fs::read_to_string(config_path) {
        if let Ok(settings) = serde_json::from_str::<serde_json::Value>(&content) {
            if let Some(window_state) = settings.get("window_state") {
                return app::WindowState {
                    width: window_state.get("width")
                        .and_then(|v| v.as_f64())
                        .map(|v| v as f32)
                        .unwrap_or(1280.0),
                    height: window_state.get("height")
                        .and_then(|v| v.as_f64())
                        .map(|v| v as f32)
                        .unwrap_or(720.0),
                    x: window_state.get("x")
                        .and_then(|v| v.as_f64())
                        .map(|v| v as f32)
                        .unwrap_or(100.0),
                    y: window_state.get("y")
                        .and_then(|v| v.as_f64())
                        .map(|v| v as f32)
                        .unwrap_or(100.0),
                    maximized: window_state.get("maximized")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false),
                };
            }
        }
    }

    // Return default window state if loading fails
    app::WindowState::default()
}
