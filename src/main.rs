mod app;
mod ui;
mod modules;
mod services;
mod utils;
mod platform;
mod config;
mod tray;
mod sidebar;

use eframe::{self, egui};

fn main() -> Result<(), eframe::Error> {
    // Set up panic hook to handle panics gracefully
    std::panic::set_hook(Box::new(|panic_info| {
        let location = panic_info.location().unwrap_or_else(|| {
            std::panic::Location::caller()
        });

        let msg = match panic_info.payload().downcast_ref::<&'static str>() {
            Some(s) => *s,
            None => match panic_info.payload().downcast_ref::<String>() {
                Some(s) => &s[..],
                None => "Box<dyn Any>",
            },
        };

        eprintln!("Panic occurred at {}:{}: {}", location.file(), location.line(), msg);

        // Try to log the panic if logger is available
        if log::log_enabled!(log::Level::Error) {
            log::error!("Panic occurred at {}:{}: {}", location.file(), location.line(), msg);
        }
    }));

    // Initialize custom logger with WARN level to reduce log noise
    utils::logger::Logger::init(log::LevelFilter::Info, true)
        .expect("Failed to initialize logger");

    // Log startup information
    log::info!("Starting SeeU Desktop v{}", env!("CARGO_PKG_VERSION"));
    log::info!("Log file location: {:?}", utils::logger::Logger::log_path());

    // Log icon loading information
    utils::icon::log_icon_info();

    // TEMPORARILY DISABLED: Load saved window state
    // let window_state = load_window_state();

    // Log the loaded window state for debugging
    // log::info!("Loaded window state (physical pixels): {}x{} at ({}, {}), maximized: {}",
    //           window_state.width, window_state.height,
    //           window_state.x, window_state.y, window_state.maximized);

    // Load application icon
    let icon_data = match utils::icon::load_window_icon() {
        Ok(icon) => {
            log::info!("Successfully loaded application icon for window");
            Some(icon)
        }
        Err(e) => {
            log::error!("Failed to load application icon: {}", e);
            None
        }
    };

    // Setup native options with DEFAULT window state (no restoration)
    let mut viewport_builder = egui::ViewportBuilder::default()
        .with_inner_size([1280.0, 720.0])  // Use default size
        .with_min_inner_size([800.0, 600.0])
        .with_decorations(true)
        .with_transparent(false);

    // Set application icon if loaded successfully
    if let Some(icon) = icon_data {
        viewport_builder = viewport_builder.with_icon(icon);
    }

    // DISABLED: Apply maximized state if needed
    // if window_state.maximized {
    //     viewport_builder = viewport_builder.with_maximized(true);
    // }

    let native_options = eframe::NativeOptions {
        viewport: viewport_builder,
        // Note: In egui 0.28.1, event_loop_builder API has changed
        // IME support is now handled differently
        ..Default::default()
    };

    // Run the application
    eframe::run_native(
        "SeeU Desktop",
        native_options,
        Box::new(|cc| Ok(Box::new(app::SeeUApp::new(cc))))
    )
}


