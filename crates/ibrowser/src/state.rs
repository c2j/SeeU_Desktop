//! Browser state management

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// Maximum number of history entries to keep
const MAX_HISTORY_SIZE: usize = 100;

/// WebView rendering mode
#[derive(Debug, Clone, PartialEq)]
pub enum WebViewMode {
    /// Show text-only content (fallback)
    TextOnly,
    /// Show full WebView rendering
    FullWebView,
    /// Show iframe-based embedding
    IframeEmbed,
}

/// Browser state management
#[derive(Debug, Clone)]
pub struct IBrowserState {
    /// Current URL being displayed
    pub current_url: String,

    /// URL input field content
    pub url_input: String,

    /// Navigation history (back)
    pub history_back: VecDeque<String>,

    /// Navigation history (forward)
    pub history_forward: VecDeque<String>,

    /// Whether the page is currently loading
    pub is_loading: bool,

    /// Page title
    pub page_title: String,

    /// Whether the webview is initialized
    pub webview_initialized: bool,

    /// Error message if any
    pub error_message: Option<String>,

    /// Loading start time for simulation
    pub loading_start_time: Option<std::time::Instant>,

    /// Whether to show embedded web content
    pub show_embedded_content: bool,

    /// Embedded web content (HTML)
    pub embedded_content: Option<String>,

    /// WebView rendering mode
    pub webview_mode: WebViewMode,

    /// Parsed web page content
    pub web_content: Option<crate::web_renderer::WebPageContent>,

    /// Whether to auto-launch WebView
    pub auto_launch_webview: bool,

    /// Whether WebView has been launched for current URL
    pub webview_launched: bool,

    /// Embedded WebView instance
    pub embedded_webview: Option<std::sync::Arc<std::sync::Mutex<crate::embedded_webview::EmbeddedWebView>>>,

    /// Whether to use embedded WebView instead of external
    pub use_embedded_webview: bool,

    /// Native WebView instance for true web rendering
    pub native_webview: Option<std::sync::Arc<std::sync::Mutex<crate::native_webview::NativeWebView>>>,
}

impl Default for IBrowserState {
    fn default() -> Self {
        Self {
            current_url: "https://www.google.com".to_string(),
            url_input: "https://www.google.com".to_string(),
            history_back: VecDeque::new(),
            history_forward: VecDeque::new(),
            is_loading: false,
            page_title: "新标签页".to_string(),
            webview_initialized: false,
            error_message: None,
            loading_start_time: None,
            show_embedded_content: false,
            embedded_content: None,
            webview_mode: WebViewMode::FullWebView,
            web_content: None,
            auto_launch_webview: true, // Default to auto-launch WebView
            webview_launched: false,
            embedded_webview: None,
            use_embedded_webview: true, // Default to embedded WebView
            native_webview: None,
        }
    }
}

impl IBrowserState {
    /// Create a new browser state
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Navigate to a URL
    pub fn navigate_to(&mut self, url: String) {
        log::info!("navigate_to called with URL: {}", url);
        log::info!("Current URL: {}", self.current_url);

        if !url.is_empty() {
            log::info!("URL is not empty, proceeding with navigation");

            // Add current URL to back history only if it's different
            if !self.current_url.is_empty() && url != self.current_url {
                self.history_back.push_back(self.current_url.clone());
                if self.history_back.len() > MAX_HISTORY_SIZE {
                    self.history_back.pop_front();
                }
                log::info!("Added previous URL to history: {}", self.current_url);
            }

            // Clear forward history when navigating to new URL (but not when reloading)
            if url != self.current_url {
                self.history_forward.clear();
            }

            // Update current URL
            self.current_url = url.clone();
            self.url_input = url.clone();
            self.is_loading = true;
            self.loading_start_time = Some(std::time::Instant::now());
            self.error_message = None;
            self.webview_launched = false; // Reset WebView launch status

            log::info!("Navigation completed. New current URL: {}", self.current_url);

            // Initialize WebView based on mode (removed google.com restriction)
            // Check if we have a running native WebView and update it
            let should_navigate_native = if let Some(native_webview) = &self.native_webview {
                if let Ok(webview) = native_webview.lock() {
                    webview.is_running()
                } else {
                    false
                }
            } else {
                false
            };

            if should_navigate_native {
                log::info!("Updating existing native WebView to: {}", url);
                self.navigate_native_webview(&url);
                return; // Early return to avoid other initialization
            }

            // Always use native WebView for simplified interface
            log::info!("Preparing native WebView for: {}", url);
            self.launch_embedded_native_webview();
        } else {
            log::info!("URL is empty, skipping navigation");
        }
    }
    
    /// Go back in history
    pub fn go_back(&mut self) -> bool {
        if let Some(url) = self.history_back.pop_back() {
            self.history_forward.push_front(self.current_url.clone());
            if self.history_forward.len() > MAX_HISTORY_SIZE {
                self.history_forward.pop_back();
            }
            
            self.current_url = url.clone();
            self.url_input = url;
            self.is_loading = true;
            self.error_message = None;
            true
        } else {
            false
        }
    }
    
    /// Go forward in history
    pub fn go_forward(&mut self) -> bool {
        if let Some(url) = self.history_forward.pop_front() {
            self.history_back.push_back(self.current_url.clone());
            if self.history_back.len() > MAX_HISTORY_SIZE {
                self.history_back.pop_front();
            }
            
            self.current_url = url.clone();
            self.url_input = url;
            self.is_loading = true;
            self.error_message = None;
            true
        } else {
            false
        }
    }
    
    /// Refresh current page
    pub fn refresh(&mut self) {
        self.is_loading = true;
        self.error_message = None;
    }
    
    /// Check if can go back
    pub fn can_go_back(&self) -> bool {
        !self.history_back.is_empty()
    }
    
    /// Check if can go forward
    pub fn can_go_forward(&self) -> bool {
        !self.history_forward.is_empty()
    }
    
    /// Set page title
    pub fn set_page_title(&mut self, title: String) {
        self.page_title = title;
    }
    
    /// Set loading state
    pub fn set_loading(&mut self, loading: bool) {
        self.is_loading = loading;
        if !loading {
            self.loading_start_time = None;
        }
    }
    
    /// Set error message
    pub fn set_error(&mut self, error: Option<String>) {
        self.error_message = error.clone();
        if error.is_some() {
            self.is_loading = false;
        }
    }
    
    /// Mark webview as initialized
    pub fn set_webview_initialized(&mut self, initialized: bool) {
        self.webview_initialized = initialized;
    }

    /// Set embedded web content
    pub fn set_embedded_content(&mut self, content: Option<String>) {
        self.show_embedded_content = content.is_some();
        self.embedded_content = content;
    }

    /// Initialize embedded WebView
    pub fn init_embedded_webview(&mut self, url: &str) {
        log::info!("Initializing embedded WebView for URL: {}", url);

        // Create embedded WebView if not exists
        if self.embedded_webview.is_none() {
            let webview = std::sync::Arc::new(std::sync::Mutex::new(
                crate::embedded_webview::EmbeddedWebView::new()
            ));
            self.embedded_webview = Some(webview);
        }

        // Load URL in embedded WebView
        if let Some(webview) = &self.embedded_webview {
            if let Ok(mut webview) = webview.lock() {
                webview.load_url(url);
            }
        }

        log::info!("Embedded WebView initialized for: {}", url);
    }

    /// Load web content for the current URL
    pub fn load_web_content(&mut self) {
        if !self.current_url.is_empty() {
            match self.webview_mode {
                WebViewMode::FullWebView => {
                    log::info!("Loading real web content for: {}", self.current_url);
                    // Fetch and parse real web content
                    let url = self.current_url.clone();
                    let rt = tokio::runtime::Runtime::new().unwrap();

                    match rt.block_on(crate::web_renderer::fetch_and_parse_webpage(&url)) {
                        Ok(content) => {
                            self.web_content = Some(content);
                            self.set_embedded_content(Some("real_web_content".to_string())); // Flag for real content
                            log::info!("Successfully loaded real web content for: {}", self.current_url);
                        }
                        Err(e) => {
                            log::error!("Failed to fetch real web content: {}", e);
                            let fallback_content = format!(
                                "无法加载网页内容: {}\n\n错误: {}\n\n请点击下方按钮在系统浏览器中查看完整内容。",
                                self.current_url, e
                            );
                            self.set_embedded_content(Some(fallback_content));
                            self.web_content = None;
                        }
                    }
                }
                WebViewMode::IframeEmbed => {
                    log::info!("Loading iframe embed for: {}", self.current_url);
                    let iframe_html = create_iframe_html(&self.current_url);
                    self.set_embedded_content(Some(iframe_html));
                    self.web_content = None;
                }
                WebViewMode::TextOnly => {
                    log::info!("Starting to fetch text content for: {}", self.current_url);
                    // Fallback to text-only content
                    let url = self.current_url.clone();
                    let rt = tokio::runtime::Runtime::new().unwrap();

                    match rt.block_on(fetch_web_content(&url)) {
                        Ok(content) => {
                            self.set_embedded_content(Some(content));
                            log::info!("Successfully loaded text content for: {}", self.current_url);
                        }
                        Err(e) => {
                            log::error!("Failed to fetch web content: {}", e);
                            let fallback_content = format!(
                                "无法加载网页内容: {}\n\n错误: {}\n\n请点击下方按钮在系统浏览器中查看完整内容。",
                                self.current_url, e
                            );
                            self.set_embedded_content(Some(fallback_content));
                        }
                    }
                    self.web_content = None;
                }
            }
        }
    }

    /// Switch WebView mode
    pub fn set_webview_mode(&mut self, mode: WebViewMode) {
        self.webview_mode = mode;
        // Reload content with new mode
        if self.show_embedded_content {
            self.load_web_content();
        }
    }

    /// Launch native WebView for the current URL
    pub fn launch_native_webview(&mut self) {
        log::info!("Launching native WebView for: {}", self.current_url);

        // Create native WebView if not exists
        if self.native_webview.is_none() {
            #[cfg(target_os = "macos")]
            let webview = std::sync::Arc::new(std::sync::Mutex::new(
                crate::native_webview::NativeWebView::new_embedded()  // Use embedded mode on macOS
            ));

            #[cfg(not(target_os = "macos"))]
            let webview = std::sync::Arc::new(std::sync::Mutex::new(
                crate::native_webview::NativeWebView::new()
            ));

            self.native_webview = Some(webview);
        }

        // Launch WebView window
        let mut error_message = None;
        if let Some(native_webview) = &self.native_webview {
            let current_url = self.current_url.clone(); // Clone to avoid borrowing issues
            if let Ok(mut webview) = native_webview.lock() {
                if let Err(e) = webview.create_window(&current_url, "iBrowser - 原生 WebView") {
                    log::error!("Failed to create native WebView window: {}", e);
                    error_message = Some(format!("无法创建原生 WebView: {}", e));
                } else {
                    log::info!("Native WebView window created successfully");
                }
            }
        }

        // Set error after releasing all locks
        if let Some(error) = error_message {
            self.set_error(Some(error));
        }
    }

    /// Reload the native WebView
    pub fn reload_native_webview(&mut self) {
        log::info!("Reloading native WebView");

        let mut error_message = None;
        if let Some(native_webview) = &self.native_webview {
            let current_url = self.current_url.clone(); // Clone to avoid borrowing issues
            if let Ok(mut webview) = native_webview.lock() {
                if let Err(e) = webview.navigate(&current_url) {
                    log::error!("Failed to reload native WebView: {}", e);
                    error_message = Some(format!("无法重新加载 WebView: {}", e));
                }
            }
        } else {
            // If no WebView exists, create one
            self.launch_native_webview();
        }

        // Set error after releasing all locks
        if let Some(error) = error_message {
            self.set_error(Some(error));
        }
    }

    /// Close the native WebView
    pub fn close_native_webview(&mut self) {
        log::info!("Closing native WebView");

        if let Some(native_webview) = &self.native_webview {
            if let Ok(mut webview) = native_webview.lock() {
                webview.close();
            }
        }
    }

    /// Navigate native WebView to a new URL
    pub fn navigate_native_webview(&mut self, url: &str) {
        log::info!("Navigating native WebView to: {}", url);

        // Update current URL
        self.current_url = url.to_string();
        self.url_input = url.to_string();

        // Navigate or create WebView
        let mut error_message = None;
        if let Some(native_webview) = &self.native_webview {
            if let Ok(mut webview) = native_webview.lock() {
                if let Err(e) = webview.navigate(url) {
                    log::error!("Failed to navigate native WebView: {}", e);
                    error_message = Some(format!("导航失败: {}", e));
                }
            }
        } else {
            // Create new WebView with the URL
            self.launch_native_webview();
        }

        // Set error after releasing all locks
        if let Some(error) = error_message {
            self.set_error(Some(error));
        }
    }

    /// Launch embedded-style native WebView (appears integrated in UI)
    pub fn launch_embedded_native_webview(&mut self) {
        log::info!("Launching embedded-style native WebView for: {}", self.current_url);

        // Create native WebView if not exists
        if self.native_webview.is_none() {
            #[cfg(target_os = "macos")]
            let webview = std::sync::Arc::new(std::sync::Mutex::new(
                crate::native_webview::NativeWebView::new_embedded()  // Use embedded mode on macOS
            ));

            #[cfg(not(target_os = "macos"))]
            let webview = std::sync::Arc::new(std::sync::Mutex::new(
                crate::native_webview::NativeWebView::new()
            ));

            self.native_webview = Some(webview);
        }

        // Create the WebView window only if not already created
        let mut error_message = None;
        if let Some(native_webview) = &self.native_webview {
            let current_url = self.current_url.clone();
            if let Ok(mut webview) = native_webview.lock() {
                if !webview.is_running() && !webview.webview_created {
                    log::info!("Creating new WebView window for: {}", current_url);
                    if let Err(e) = webview.create_window(&current_url, "iBrowser - 内置 WebView") {
                        log::error!("Failed to create embedded-style WebView: {}", e);
                        error_message = Some(format!("无法创建内置 WebView: {}", e));
                    } else {
                        log::info!("Embedded-style WebView created successfully");
                    }
                } else {
                    log::info!("WebView already exists or is running, skipping creation");
                }
            }
        }

        // Set error after releasing all locks
        if let Some(error) = error_message {
            self.set_error(Some(error));
        }
    }

    /// Step the embedded WebView (for macOS main thread integration)
    pub fn step_embedded_webview(&mut self) -> bool {
        if let Some(native_webview) = &self.native_webview {
            if let Ok(mut webview) = native_webview.lock() {
                return webview.step_embedded();
            }
        }
        false
    }

    /// Check if we have an embedded WebView running
    pub fn has_embedded_webview(&self) -> bool {
        if let Some(native_webview) = &self.native_webview {
            if let Ok(webview) = native_webview.lock() {
                return webview.is_running();
            }
        }
        false
    }
}

/// Fetch real web content from a URL
async fn fetch_web_content(url: &str) -> anyhow::Result<String> {
    log::info!("Fetching web content from: {}", url);

    // Create a client with a user agent
    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36")
        .timeout(std::time::Duration::from_secs(10))
        .build()?;

    // Fetch the web page
    let response = client.get(url).send().await?;

    if !response.status().is_success() {
        return Err(anyhow::anyhow!("HTTP error: {}", response.status()));
    }

    let html = response.text().await?;

    // Convert HTML to readable text
    let text_content = html2text::from_read(html.as_bytes(), 80);

    // Format the content for display
    let formatted_content = format!(
        "📄 网页内容 ({})\n\n{}\n\n--- 内容结束 ---\n\n💡 这是网页的文本内容。要查看完整的网页（包括图片、样式等），请点击下方按钮在系统浏览器中打开。",
        url,
        text_content.trim()
    );

    Ok(formatted_content)
}

/// Create HTML for full WebView rendering
fn create_webview_html(url: &str) -> String {
    format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <title>iBrowser - {}</title>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <style>
        body {{
            margin: 0;
            padding: 0;
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            background: #f5f5f5;
        }}
        .header {{
            background: #fff;
            border-bottom: 1px solid #e1e5e9;
            padding: 10px 15px;
            display: flex;
            align-items: center;
            gap: 10px;
        }}
        .url {{
            color: #586069;
            font-size: 14px;
            font-family: 'SF Mono', Monaco, monospace;
        }}
        .content {{
            height: calc(100vh - 50px);
            background: white;
        }}
        iframe {{
            width: 100%;
            height: 100%;
            border: none;
            display: block;
        }}
        .loading {{
            display: flex;
            align-items: center;
            justify-content: center;
            height: 200px;
            color: #586069;
        }}
        .error {{
            padding: 20px;
            text-align: center;
            color: #d73a49;
        }}
    </style>
</head>
<body>
    <div class="header">
        <span style="color: #28a745;">🌐</span>
        <span class="url">{}</span>
    </div>
    <div class="content">
        <iframe src="{}"
                onload="document.getElementById('loading').style.display='none';"
                onerror="document.getElementById('loading').innerHTML='<div class=error>无法加载页面</div>';">
        </iframe>
        <div id="loading" class="loading">
            <div>正在加载网页...</div>
        </div>
    </div>
</body>
</html>"#,
        url, url, url
    )
}

/// Create HTML for iframe embedding
fn create_iframe_html(url: &str) -> String {
    format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <title>iBrowser - {}</title>
    <style>
        body {{ margin: 0; padding: 10px; font-family: Arial, sans-serif; }}
        .info {{ background: #f8f9fa; padding: 10px; border-radius: 5px; margin-bottom: 10px; }}
        iframe {{ width: 100%; height: 500px; border: 1px solid #ddd; border-radius: 5px; }}
    </style>
</head>
<body>
    <div class="info">
        <strong>🌐 正在显示:</strong> {}
    </div>
    <iframe src="{}"></iframe>
</body>
</html>"#,
        url, url, url
    )
}
