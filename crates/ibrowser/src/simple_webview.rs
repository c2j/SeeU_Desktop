//! Ultra-simple WebView implementation based on minimal.rs
//! This is designed to be as stable as possible, avoiding all complex features

use std::thread;
use anyhow::Result;

/// Create a simple WebView window using the exact same approach as minimal.rs
pub fn create_simple_webview(url: &str) -> Result<()> {
    log::info!("Creating ultra-simple WebView for: {}", url);
    
    let url_clone = url.to_string();
    
    // Spawn in a separate thread to avoid blocking the main UI
    thread::spawn(move || {
        log::info!("Starting minimal.rs-style WebView thread");
        
        // This is EXACTLY the same as minimal.rs
        let result = web_view::builder()
            .title("iBrowser - Simple WebView")
            .content(web_view::Content::Url(&url_clone))
            .size(800, 600)
            .resizable(true)
            .debug(false)
            .user_data(())
            .invoke_handler(|_webview, _arg| Ok(()))
            .run();
            
        match result {
            Ok(_) => log::info!("Simple WebView closed normally"),
            Err(e) => log::error!("Simple WebView error: {:?}", e),
        }
    });
    
    log::info!("Simple WebView thread started");
    Ok(())
}

/// Test function to verify WebView works
pub fn test_simple_webview() -> Result<()> {
    create_simple_webview("https://www.google.com")
}
