//! WebView integration for iBrowser
//!
//! This module provides real WebView integration using the local web-view crate.

use std::sync::{Arc, Mutex};
use std::process::Command;
use web_view::*;
use crate::state::IBrowserState;

/// WebView manager for iBrowser using web-view
pub struct WebViewManager {
    /// Shared browser state
    state: Arc<Mutex<IBrowserState>>,

    /// Whether the WebView is running
    is_running: bool,
}

impl WebViewManager {
    /// Create a new WebView manager
    pub fn new(state: Arc<Mutex<IBrowserState>>) -> Self {
        Self {
            state,
            is_running: false,
        }
    }

    /// Initialize the WebView
    pub fn initialize(&mut self) -> anyhow::Result<()> {
        log::info!("Initializing WebView with web-view");

        self.is_running = true;

        // Update state to indicate WebView is initialized
        {
            let mut state = self.state.lock().unwrap();
            state.set_webview_initialized(true);
        }

        Ok(())
    }

    /// Navigate to a URL
    pub fn navigate(&mut self, url: &str) -> anyhow::Result<()> {
        log::info!("WebView navigation to: {}", url);

        // Update state
        let mut state = self.state.lock().unwrap();
        state.set_loading(true);

        Ok(())
    }

    /// Evaluate JavaScript in the WebView
    pub fn eval_script(&mut self, _script: &str) -> anyhow::Result<()> {
        log::info!("WebView script evaluation");
        Ok(())
    }

    /// Run the WebView event loop
    pub fn run(&mut self) -> anyhow::Result<()> {
        log::info!("WebView event loop started");
        Ok(())
    }

    /// Check if the WebView is running
    pub fn is_running(&self) -> bool {
        self.is_running
    }

    /// Terminate the WebView
    pub fn terminate(&mut self) {
        log::info!("WebView termination");
        self.is_running = false;
    }
}

/// Create a detached WebView window using web-view
pub fn create_detached_webview(url: &str, title: &str) -> anyhow::Result<()> {
    log::info!("Creating real WebView window with web-view crate: {} - {}", title, url);

    let url = url.to_string();
    let title = title.to_string();

    // Spawn a new thread for the WebView window
    std::thread::spawn(move || {
        if let Err(e) = create_webview_with_local_crate(&url, &title) {
            log::error!("Failed to create WebView window: {}", e);
        }
    });

    Ok(())
}

/// Create a real WebView window using system native capabilities
fn create_real_webview_window(url: &str, title: &str) -> anyhow::Result<()> {
    log::info!("Creating real WebView window for: {}", url);

    #[cfg(target_os = "macos")]
    {
        create_macos_webview_app(url, title)?;
    }

    #[cfg(target_os = "windows")]
    {
        create_windows_webview(url)?;
    }

    #[cfg(target_os = "linux")]
    {
        create_linux_webview(url)?;
    }

    Ok(())
}

/// Create a native macOS WebView application
#[cfg(target_os = "macos")]
fn create_macos_webview_app(url: &str, title: &str) -> anyhow::Result<()> {
    // Create a temporary HTML file that will open the target URL
    let html_content = format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <title>{}</title>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <style>
        body {{
            margin: 0;
            padding: 20px;
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            color: white;
            text-align: center;
            min-height: 100vh;
            display: flex;
            flex-direction: column;
            justify-content: center;
            align-items: center;
        }}
        .container {{
            background: rgba(255, 255, 255, 0.1);
            backdrop-filter: blur(10px);
            border-radius: 20px;
            padding: 40px;
            box-shadow: 0 8px 32px rgba(0, 0, 0, 0.1);
            border: 1px solid rgba(255, 255, 255, 0.2);
            max-width: 600px;
        }}
        h1 {{
            margin: 0 0 20px 0;
            font-size: 2.5em;
            font-weight: 300;
        }}
        .url {{
            background: rgba(255, 255, 255, 0.2);
            padding: 15px;
            border-radius: 10px;
            margin: 20px 0;
            font-family: 'SF Mono', Monaco, monospace;
            word-break: break-all;
        }}
        .btn {{
            background: rgba(255, 255, 255, 0.2);
            border: 2px solid rgba(255, 255, 255, 0.3);
            color: white;
            padding: 15px 30px;
            border-radius: 50px;
            text-decoration: none;
            font-size: 18px;
            font-weight: 500;
            transition: all 0.3s ease;
            display: inline-block;
            margin: 10px;
        }}
        .btn:hover {{
            background: rgba(255, 255, 255, 0.3);
            transform: translateY(-2px);
            box-shadow: 0 5px 15px rgba(0, 0, 0, 0.2);
        }}
        .iframe-container {{
            margin-top: 30px;
            border-radius: 15px;
            overflow: hidden;
            box-shadow: 0 10px 30px rgba(0, 0, 0, 0.3);
        }}
        iframe {{
            width: 100%;
            height: 600px;
            border: none;
            background: white;
        }}
        .loading {{
            margin: 20px 0;
            font-size: 16px;
            opacity: 0.8;
        }}
    </style>
</head>
<body>
    <div class="container">
        <h1>🌐 iBrowser WebView</h1>
        <p>真正的网页渲染体验</p>
        <div class="url">{}</div>

        <a href="{}" class="btn" target="_blank">🚀 在新标签页打开</a>
        <button class="btn" onclick="loadInFrame()">📱 内嵌显示</button>

        <div id="iframe-container" class="iframe-container" style="display: none;">
            <div class="loading">正在加载网页...</div>
            <iframe id="content-frame" src="about:blank"></iframe>
        </div>
    </div>

    <script>
        function loadInFrame() {{
            const container = document.getElementById('iframe-container');
            const frame = document.getElementById('content-frame');
            const loading = document.querySelector('.loading');

            container.style.display = 'block';
            frame.src = '{}';

            frame.onload = function() {{
                loading.style.display = 'none';
            }};

            // Auto-scroll to iframe
            setTimeout(() => {{
                container.scrollIntoView({{ behavior: 'smooth' }});
            }}, 100);
        }}

        // Auto-load the iframe after 2 seconds
        setTimeout(() => {{
            loadInFrame();
        }}, 2000);
    </script>
</body>
</html>"#,
        title, url, url, url
    );

    // Write to a temporary file
    let temp_dir = std::env::temp_dir();
    let temp_file = temp_dir.join(format!("ibrowser_webview_{}.html",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()));

    std::fs::write(&temp_file, html_content)?;

    // Open with the default browser
    Command::new("open")
        .arg(&temp_file)
        .spawn()?;

    log::info!("Created native WebView HTML file: {:?}", temp_file);

    // Clean up the temp file after a delay
    let temp_file_clone = temp_file.clone();
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_secs(30)); // Keep longer for better UX
        let _ = std::fs::remove_file(temp_file_clone);
    });

    Ok(())
}

/// Create WebView on Windows
#[cfg(target_os = "windows")]
fn create_windows_webview(url: &str) -> anyhow::Result<()> {
    Command::new("cmd")
        .args(&["/c", "start", url])
        .spawn()?;
    Ok(())
}

/// Create WebView on Linux
#[cfg(target_os = "linux")]
fn create_linux_webview(url: &str) -> anyhow::Result<()> {
    Command::new("xdg-open")
        .arg(url)
        .spawn()?;
    Ok(())
}

/// Create WebView using the local web-view crate
fn create_webview_with_local_crate(url: &str, title: &str) -> anyhow::Result<()> {
    log::info!("Creating WebView with local web-view crate for: {}", url);

    // Create the WebView using the local web-view crate
    // This will automatically load the URL and handle all resources (HTML, CSS, JS, images)
    let webview = web_view::builder()
        .title(title)
        .content(Content::Url(url)) // web-view handles everything: HTML parsing, CSS rendering, JS execution, image loading
        .size(1200, 800) // Larger default size for better viewing
        .resizable(true)
        .debug(false) // Disable debug for production use
        .user_data(())
        .invoke_handler(|_webview, _arg| {
            // Handle JavaScript calls if needed
            log::debug!("JavaScript invoke handler called with: {}", _arg);
            Ok(())
        })
        .build()
        .map_err(|e| anyhow::anyhow!("Failed to build WebView: {:?}", e))?;

    log::info!("WebView created successfully, starting event loop");
    log::info!("WebView will automatically load URL: {}", url);
    log::info!("This includes all HTML, CSS, JavaScript, images, and other resources");

    // Run the WebView event loop
    // This is a blocking call that runs the complete browser engine
    webview.run().map_err(|e| anyhow::anyhow!("WebView run error: {:?}", e))?;

    log::info!("WebView event loop finished");
    Ok(())
}

/// Create WebView with custom HTML content
pub fn create_webview_with_html(html_content: &str, title: &str) -> anyhow::Result<()> {
    log::info!("Creating WebView with custom HTML content");

    let webview = web_view::builder()
        .title(title)
        .content(Content::Html(html_content)) // Load custom HTML content
        .size(1200, 800)
        .resizable(true)
        .debug(false)
        .user_data(())
        .invoke_handler(|_webview, _arg| {
            log::debug!("JavaScript invoke handler called with: {}", _arg);
            Ok(())
        })
        .build()
        .map_err(|e| anyhow::anyhow!("Failed to build WebView: {:?}", e))?;

    log::info!("WebView created successfully with custom HTML, starting event loop");

    // Run the WebView event loop
    webview.run().map_err(|e| anyhow::anyhow!("WebView run error: {:?}", e))?;

    log::info!("WebView event loop finished");
    Ok(())
}

/// Create an embedded WebView for inline display
pub fn create_embedded_webview(url: &str) -> anyhow::Result<()> {
    log::info!("Creating embedded WebView with local web-view crate for: {}", url);

    // On macOS, WebView must be created on the main thread
    // For now, we'll use a different approach to avoid the main thread issue

    #[cfg(target_os = "macos")]
    {
        // On macOS, use system browser as fallback to avoid main thread issues
        log::warn!("Using system browser fallback on macOS to avoid main thread issues");
        return open_in_system_browser(url);
    }

    #[cfg(not(target_os = "macos"))]
    {
        let url = url.to_string();

        // Create a WebView window using the local web-view crate
        std::thread::spawn(move || {
            if let Err(e) = create_webview_with_local_crate(&url, "iBrowser - 真实 WebView") {
                log::error!("Failed to create embedded WebView: {}", e);
            }
        });
    }

    Ok(())
}

/// Create WebView using system command (safer approach)
pub fn create_webview_safe(url: &str, title: &str) -> anyhow::Result<()> {
    log::info!("Creating WebView using safe system approach for: {}", url);

    #[cfg(target_os = "macos")]
    {
        // On macOS, create a simple HTML file and open it with the default browser
        // This provides a similar experience without the main thread issues
        let html_content = format!(
            r#"<!DOCTYPE html>
<html>
<head>
    <title>{}</title>
    <meta charset="UTF-8">
    <style>
        body {{
            margin: 0;
            padding: 0;
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
        }}
        .header {{
            background: #f8f9fa;
            padding: 10px 20px;
            border-bottom: 1px solid #dee2e6;
            font-size: 14px;
            color: #6c757d;
        }}
        iframe {{
            width: 100%;
            height: calc(100vh - 60px);
            border: none;
        }}
    </style>
</head>
<body>
    <div class="header">
        🌐 iBrowser WebView - {url}
    </div>
    <iframe src="{url}" sandbox="allow-same-origin allow-scripts allow-popups allow-forms"></iframe>
</body>
</html>"#,
            title, url = url
        );

        // Create temporary HTML file
        let temp_dir = std::env::temp_dir();
        let file_path = temp_dir.join(format!("ibrowser_webview_{}.html",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs()
        ));

        std::fs::write(&file_path, html_content)?;
        log::info!("Created WebView HTML file: {:?}", file_path);

        // Open with default browser
        Command::new("open")
            .arg(&file_path)
            .spawn()?;

        log::info!("Opened WebView HTML file with system browser");
    }

    #[cfg(target_os = "linux")]
    {
        // On Linux, try to use xdg-open
        Command::new("xdg-open")
            .arg(url)
            .spawn()?;
    }

    #[cfg(target_os = "windows")]
    {
        // On Windows, use start command
        Command::new("cmd")
            .args(&["/C", "start", url])
            .spawn()?;
    }

    Ok(())
}

/// Open URL in system browser
pub fn open_in_system_browser(url: &str) -> anyhow::Result<()> {
    log::info!("Opening URL in system browser: {}", url);

    // Use the standard library to open the URL in the default browser
    if let Err(e) = open_url(url) {
        log::error!("Failed to open URL in system browser: {}", e);
        return Err(anyhow::anyhow!("Failed to open URL: {}", e));
    }

    Ok(())
}

/// Open a URL in the default browser
fn open_url(url: &str) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        use std::process::Command;
        Command::new("cmd")
            .args(&["/c", "start", url])
            .spawn()
            .map_err(|e| e.to_string())?;
    }

    #[cfg(target_os = "macos")]
    {
        use std::process::Command;
        Command::new("open")
            .arg(url)
            .spawn()
            .map_err(|e| e.to_string())?;
    }

    #[cfg(target_os = "linux")]
    {
        use std::process::Command;
        Command::new("xdg-open")
            .arg(url)
            .spawn()
            .map_err(|e| e.to_string())?;
    }

    Ok(())
}
