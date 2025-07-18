//! iBrowser - Browser module for SeeU Desktop
//!
//! This module provides a complete browser experience using the system's default browser engine
//! through the web-view component.

pub mod state;
pub mod ui;
pub mod webview;
pub mod web_renderer;
pub mod embedded_webview;
pub mod native_webview;
pub mod simple_webview;

// Re-export main components
pub use state::IBrowserState;
pub use ui::render_ibrowser;

/// Initialize the iBrowser module
pub fn init() -> anyhow::Result<()> {
    log::info!("Initializing iBrowser module");
    Ok(())
}

/// Get module information
pub fn get_module_info() -> &'static str {
    "iBrowser v0.1.0 - Browser module using system's default browser engine"
}
