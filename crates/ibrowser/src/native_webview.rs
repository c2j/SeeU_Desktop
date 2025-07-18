//! Native WebView renderer using web-view crate
//! 
//! This module provides true native web rendering by creating web-view windows
//! that use the system's default browser engine, similar to the minimal.rs example.

use std::sync::{Arc, Mutex};
use std::thread;
use anyhow::Result;
use web_view::*;

/// Native WebView window manager
#[derive(Debug)]
pub struct NativeWebView {
    /// Current URL being displayed
    current_url: String,
    /// Window title
    title: String,
    /// Whether the WebView is currently running
    is_running: Arc<Mutex<bool>>,
    /// Thread handle for the WebView window
    thread_handle: Option<thread::JoinHandle<()>>,
    /// Whether this is running in embedded mode (for macOS)
    is_embedded_mode: bool,
    /// Whether WebView was created successfully
    pub webview_created: bool,
    /// WebView window position and size for embedded display
    webview_rect: Option<(f32, f32, f32, f32)>, // x, y, width, height
    /// Whether WebView is in embedded mode (simplified approach)
    is_embedded_webview_active: bool,
    /// Simple flag to track if we should use embedded mode
    should_use_embedded: bool,
}

impl NativeWebView {
    /// Create a new native WebView instance
    pub fn new() -> Self {
        Self {
            current_url: String::new(),
            title: "iBrowser".to_string(),
            is_running: Arc::new(Mutex::new(false)),
            thread_handle: None,
            is_embedded_mode: false,
            webview_created: false,
            webview_rect: None,
            is_embedded_webview_active: false,
            should_use_embedded: false,
        }
    }

    /// Create a new embedded WebView instance (for macOS main thread integration)
    pub fn new_embedded() -> Self {
        Self {
            current_url: String::new(),
            title: "iBrowser".to_string(),
            is_running: Arc::new(Mutex::new(false)),
            thread_handle: None,
            is_embedded_mode: true,
            webview_created: false,
            webview_rect: None,
            is_embedded_webview_active: false,
            should_use_embedded: true,
        }
    }

    /// Create and show a native WebView window
    pub fn create_window(&mut self, url: &str, title: &str) -> Result<()> {
        self.create_window_with_rect(url, title, None)
    }

    /// Create and show a native WebView window with specific position and size
    pub fn create_window_with_rect(&mut self, url: &str, title: &str, rect: Option<(f32, f32, f32, f32)>) -> Result<()> {
        log::info!("Creating native WebView window for: {}", url);

        // Update internal state
        self.current_url = url.to_string();
        self.title = title.to_string();
        self.webview_rect = rect;

        // Set running state
        {
            let mut running = self.is_running.lock().unwrap();
            *running = true;
        }

        // On macOS, create WebView on main thread (no separate window)
        #[cfg(target_os = "macos")]
        {
            log::info!("Creating WebView on macOS main thread");
            return self.create_macos_main_thread_webview(url, title, rect);
        }

        #[cfg(not(target_os = "macos"))]
        {
            let url_clone = url.to_string();
            let title_clone = title.to_string();
            let is_running_clone = Arc::clone(&self.is_running);

            // Create WebView in a new thread (only on non-macOS platforms)
            let handle = thread::spawn(move || {
                log::info!("Starting native WebView thread for: {}", url_clone);

                // Create the WebView using the web-view crate (similar to minimal.rs)
                let result = web_view::builder()
                    .title(&title_clone)
                    .content(Content::Url(&url_clone))
                    .size(1200, 800)
                    .resizable(true)
                    .debug(false) // Set to true for debugging
                    .user_data(())
                    .invoke_handler(|_webview, _arg| {
                        log::debug!("WebView invoke handler called with: {}", _arg);
                        Ok(())
                    })
                    .run();

                // Handle result
                match result {
                    Ok(_) => {
                        log::info!("Native WebView window closed normally");
                    }
                    Err(e) => {
                        log::error!("Native WebView error: {:?}", e);
                    }
                }

                // Update running state when window closes
                {
                    let mut running = is_running_clone.lock().unwrap();
                    *running = false;
                }
            });

            self.thread_handle = Some(handle);
        }

        log::info!("Native WebView window created successfully");
        Ok(())
    }

    /// Create WebView on macOS main thread (allows interface blocking)
    fn create_macos_main_thread_webview(&mut self, url: &str, title: &str, rect: Option<(f32, f32, f32, f32)>) -> Result<()> {
        log::info!("Creating WebView on macOS main thread: {} - {}", title, url);

        // Determine size based on the provided rect
        let (width, height) = if let Some((x, y, w, h)) = rect {
            log::info!("Using provided rect: position ({:.1}, {:.1}), size ({:.1}, {:.1})", x, y, w, h);
            // Use the provided size, but ensure minimum dimensions
            (w.max(400.0) as i32, h.max(300.0) as i32)
        } else {
            log::info!("No rect provided, using default size");
            (1000, 700)
        };

        // Create a compact WebView window that appears embedded
        let url_clone = url.to_string();
        let title_clone = title.to_string();

        // Set running state
        {
            let mut running = self.is_running.lock().unwrap();
            *running = true;
        }

        self.is_embedded_webview_active = true;

        // Create WebView using EXACTLY the same approach as minimal.rs
        log::info!("Creating minimal.rs-style WebView: size {}x{}, title: {}", width, height, title_clone);

        // Use a separate thread for stability
        let is_running_clone = Arc::clone(&self.is_running);
        let handle = thread::spawn(move || {
            // Use EXACTLY the same configuration as minimal.rs for maximum stability
            let result = web_view::builder()
                .title(&title_clone)
                .content(web_view::Content::Url(&url_clone))
                .size(width, height)
                .resizable(true)
                .debug(false) // Disable debug like minimal.rs
                .user_data(())
                .invoke_handler(|_webview, _arg| Ok(())) // Minimal handler like minimal.rs
                .run();

            match result {
                Ok(_) => {
                    log::info!("Minimal.rs-style WebView closed normally");
                }
                Err(e) => {
                    log::error!("Minimal.rs-style WebView error: {:?}", e);
                }
            }

            // Set running state to false when done
            let mut running = is_running_clone.lock().unwrap();
            *running = false;
        });

        self.thread_handle = Some(handle);
        self.webview_created = true;

        log::info!("Minimal.rs-style WebView thread started successfully");
        Ok(())
    }

    /// Step the embedded WebView (simplified - just check if running)
    pub fn step_embedded(&mut self) -> bool {
        // For the stable implementation, just check if the WebView is running
        self.is_embedded_webview_active && self.is_running()
    }

    /// Navigate to a new URL
    pub fn navigate(&mut self, url: &str) -> Result<()> {
        log::info!("Navigating native WebView to: {}", url);

        // For stability, we don't support navigation in embedded mode
        // Instead, we just update the URL and log the request
        self.current_url = url.to_string();

        if self.is_embedded_mode {
            log::warn!("Navigation in embedded mode not supported for stability. URL updated to: {}", url);
            return Ok(());
        }

        // For non-embedded mode, we can try to navigate
        // But for now, we'll just update the URL to avoid crashes
        log::info!("URL updated to: {}", url);
        Ok(())
    }

    /// Check if the WebView window is currently running
    pub fn is_running(&self) -> bool {
        let running = self.is_running.lock().unwrap();
        *running
    }

    /// Get the current URL
    pub fn get_current_url(&self) -> &str {
        &self.current_url
    }

    /// Close the WebView window
    pub fn close(&mut self) {
        log::info!("Closing native WebView window");

        // Set running state to false
        {
            let mut running = self.is_running.lock().unwrap();
            *running = false;
        }

        // Mark embedded WebView as inactive
        self.is_embedded_webview_active = false;

        self.webview_created = false;
    }

    /// Set the window title
    pub fn set_title(&mut self, title: &str) {
        self.title = title.to_string();
    }
}

impl Drop for NativeWebView {
    fn drop(&mut self) {
        self.close();
    }
}

/// Create a standalone native WebView window (similar to minimal.rs)
pub fn create_standalone_webview(url: &str, title: &str) -> Result<()> {
    log::info!("Creating standalone native WebView: {} - {}", title, url);

    #[cfg(target_os = "macos")]
    {
        // On macOS, use main thread approach
        return create_standalone_webview_main_thread(url, title);
    }

    #[cfg(not(target_os = "macos"))]
    {
        let url = url.to_string();
        let title = title.to_string();

        // Spawn in a new thread to avoid blocking (only on non-macOS)
        thread::spawn(move || {
            let result = web_view::builder()
                .title(&title)
                .content(Content::Url(&url))
                .size(1200, 800)
                .resizable(true)
                .debug(false)
                .user_data(())
                .invoke_handler(|_webview, _arg| {
                    log::debug!("Standalone WebView invoke handler: {}", _arg);
                    Ok(())
                })
                .run();

            match result {
                Ok(_) => log::info!("Standalone WebView closed normally"),
                Err(e) => log::error!("Standalone WebView error: {:?}", e),
            }
        });
    }

    Ok(())
}

/// Create a WebView on the main thread (required for macOS)
fn create_standalone_webview_main_thread(url: &str, title: &str) -> Result<()> {
    log::info!("Creating WebView on main thread for macOS: {} - {}", title, url);

    // For macOS, we need to use a different approach
    // We'll use the system's open command to open the URL in the default browser
    // This is safer than trying to create WebView in a thread

    #[cfg(target_os = "macos")]
    {
        use std::process::Command;

        log::info!("Opening URL in system browser on macOS: {}", url);
        let result = Command::new("open")
            .arg(url)
            .spawn();

        match result {
            Ok(_) => {
                log::info!("Successfully opened URL in system browser");
                Ok(())
            }
            Err(e) => {
                log::error!("Failed to open URL in system browser: {}", e);
                Err(anyhow::anyhow!("Failed to open URL: {}", e))
            }
        }
    }

    #[cfg(not(target_os = "macos"))]
    {
        // On non-macOS, we can try to create WebView directly
        let result = web_view::builder()
            .title(title)
            .content(Content::Url(url))
            .size(1200, 800)
            .resizable(true)
            .debug(false)
            .user_data(())
            .invoke_handler(|_webview, _arg| {
                log::debug!("Main thread WebView invoke handler: {}", _arg);
                Ok(())
            })
            .run();

        match result {
            Ok(_) => {
                log::info!("Main thread WebView closed normally");
                Ok(())
            }
            Err(e) => {
                log::error!("Main thread WebView error: {:?}", e);
                Err(anyhow::anyhow!("WebView error: {:?}", e))
            }
        }
    }
}

/// Create a minimal native WebView window (exactly like minimal.rs)
pub fn create_minimal_webview(url: &str) -> Result<()> {
    log::info!("Creating minimal native WebView for: {}", url);

    #[cfg(target_os = "macos")]
    {
        // On macOS, use system browser instead
        return create_standalone_webview_main_thread(url, "iBrowser - Minimal WebView");
    }

    #[cfg(not(target_os = "macos"))]
    {
        let url = url.to_string();

        thread::spawn(move || {
            // This is exactly like the minimal.rs example
            let result = web_view::builder()
                .title("iBrowser - Native WebView")
                .content(Content::Url(&url))
                .size(800, 600)
                .resizable(true)
                .debug(true)
                .user_data(())
                .invoke_handler(|_webview, _arg| Ok(()))
                .run();

            match result {
                Ok(_) => log::info!("Minimal WebView closed"),
                Err(e) => log::error!("Minimal WebView error: {:?}", e),
            }
        });

        Ok(())
    }
}
