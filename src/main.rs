mod app;
mod ui;
mod modules;
mod services;
mod utils;
mod platform;

use eframe::{self, egui};

fn main() -> Result<(), eframe::Error> {
    // Initialize custom logger with DEBUG level and file logging enabled
    utils::logger::Logger::init(log::LevelFilter::Debug, true)
        .expect("Failed to initialize logger");

    // Log startup information
    log::info!("Starting SeeU Desktop v{}", env!("CARGO_PKG_VERSION"));
    log::info!("Log file location: {:?}", utils::logger::Logger::log_path());

    // Setup native options
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 720.0])
            .with_min_inner_size([800.0, 600.0])
            .with_position([100.0, 100.0]) // Position instead of centered
            .with_decorations(true)
            .with_transparent(false),
        ..Default::default()
    };

    // Run the application
    eframe::run_native(
        "SeeU Desktop",
        native_options,
        Box::new(|cc| Ok(Box::new(app::SeeUApp::new(cc))))
    )
}
