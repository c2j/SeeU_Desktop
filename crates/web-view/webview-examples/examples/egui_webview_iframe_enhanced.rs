//! Enhanced iframe WebView with Reliable Reload
//!
//! This example specifically addresses the iframe reload issue by implementing
//! proper iframe content reloading and navigation controls

use eframe::egui;
use web_view::*;

/// Enhanced iframe application
struct EnhancedIframeApp {
    /// URL input
    url_input: String,
    /// Status message
    status_message: String,
    /// WebView instance
    webview: Option<WebView<'static, ()>>,
    /// WebView created
    webview_created: bool,
    /// Current URL being displayed
    current_url: String,
    /// Loading state
    is_loading: bool,
    /// Reload count for debugging
    reload_count: u32,
}

impl Default for EnhancedIframeApp {
    fn default() -> Self {
        Self {
            url_input: "https://www.rust-lang.org".to_string(),
            status_message: "Ready to create enhanced iframe WebView".to_string(),
            webview: None,
            webview_created: false,
            current_url: String::new(),
            is_loading: false,
            reload_count: 0,
        }
    }
}

impl EnhancedIframeApp {
    fn create_enhanced_html(&self, target_url: &str) -> String {
        format!(r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Enhanced iframe WebView</title>
    <style>
        * {{
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }}
        
        body {{
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            height: 100vh;
            display: flex;
            flex-direction: column;
        }}
        
        .header {{
            background: rgba(255, 255, 255, 0.95);
            padding: 12px 20px;
            border-bottom: 2px solid #4a90e2;
            display: flex;
            justify-content: space-between;
            align-items: center;
            box-shadow: 0 2px 10px rgba(0,0,0,0.1);
        }}
        
        .header h1 {{
            color: #2c3e50;
            font-size: 18px;
            font-weight: 600;
        }}
        
        .url-display {{
            background: #f8f9fa;
            padding: 6px 12px;
            border-radius: 20px;
            font-size: 12px;
            color: #495057;
            max-width: 300px;
            overflow: hidden;
            text-overflow: ellipsis;
            white-space: nowrap;
            border: 1px solid #dee2e6;
        }}
        
        .controls {{
            background: rgba(255, 255, 255, 0.9);
            padding: 10px 20px;
            display: flex;
            gap: 10px;
            align-items: center;
            border-bottom: 1px solid #e9ecef;
        }}
        
        .btn {{
            background: #4a90e2;
            color: white;
            border: none;
            padding: 8px 16px;
            border-radius: 6px;
            cursor: pointer;
            font-size: 14px;
            font-weight: 500;
            transition: all 0.2s ease;
        }}
        
        .btn:hover {{
            background: #357abd;
            transform: translateY(-1px);
        }}
        
        .btn:active {{
            transform: translateY(0);
        }}
        
        .btn.reload {{
            background: #28a745;
        }}
        
        .btn.reload:hover {{
            background: #218838;
        }}
        
        .status {{
            margin-left: auto;
            font-size: 12px;
            color: #6c757d;
            display: flex;
            align-items: center;
            gap: 8px;
        }}
        
        .loading-indicator {{
            width: 12px;
            height: 12px;
            border: 2px solid #f3f3f3;
            border-top: 2px solid #4a90e2;
            border-radius: 50%;
            animation: spin 1s linear infinite;
        }}
        
        @keyframes spin {{
            0% {{ transform: rotate(0deg); }}
            100% {{ transform: rotate(360deg); }}
        }}
        
        .iframe-container {{
            flex: 1;
            position: relative;
            background: white;
        }}
        
        .loading-overlay {{
            position: absolute;
            top: 0;
            left: 0;
            right: 0;
            bottom: 0;
            background: rgba(255, 255, 255, 0.9);
            display: flex;
            justify-content: center;
            align-items: center;
            font-size: 16px;
            color: #495057;
            z-index: 10;
        }}
        
        iframe {{
            width: 100%;
            height: 100%;
            border: none;
            background: white;
        }}
        
        .reload-info {{
            font-size: 11px;
            color: #6c757d;
            margin-left: 10px;
        }}
    </style>
</head>
<body>
    <div class="header">
        <h1>🌐 Enhanced iframe WebView</h1>
        <div class="url-display" id="url-display" title="{}">{}</div>
    </div>
    
    <div class="controls">
        <button class="btn reload" onclick="forceReloadIframe()">🔄 Force Reload</button>
        <button class="btn" onclick="navigateIframe('back')">⬅️ Back</button>
        <button class="btn" onclick="navigateIframe('forward')">➡️ Forward</button>
        <button class="btn" onclick="refreshPage()">🔄 Refresh Page</button>
        <button class="btn" onclick="window.external.invoke('fullscreen')">⛶ Fullscreen</button>
        <div class="status">
            <span id="status-text">Ready</span>
            <div class="loading-indicator" id="loading-spinner" style="display: none;"></div>
            <span class="reload-info" id="reload-info">Reloads: 0</span>
        </div>
    </div>
    
    <div class="iframe-container">
        <div class="loading-overlay" id="loading-overlay">
            <div>
                <div class="loading-indicator"></div>
                <div style="margin-top: 10px;">Loading enhanced iframe content...</div>
            </div>
        </div>
        <iframe 
            id="main-iframe"
            src="{}" 
            onload="handleIframeLoad()"
            onerror="handleIframeError()"
            sandbox="allow-same-origin allow-scripts allow-forms allow-popups allow-popups-to-escape-sandbox allow-top-navigation"
        ></iframe>
    </div>
    
    <script>
        let reloadCount = 0;
        let currentUrl = '{}';
        
        // Enhanced iframe control functions
        function forceReloadIframe() {{
            const iframe = document.getElementById('main-iframe');
            const loadingOverlay = document.getElementById('loading-overlay');
            const statusText = document.getElementById('status-text');
            const spinner = document.getElementById('loading-spinner');
            const reloadInfo = document.getElementById('reload-info');
            
            if (iframe) {{
                reloadCount++;
                reloadInfo.textContent = `Reloads: ${{reloadCount}}`;
                
                // Show loading state
                loadingOverlay.style.display = 'flex';
                statusText.textContent = 'Force reloading...';
                spinner.style.display = 'block';
                
                // Force reload by clearing src and setting it again
                const originalSrc = iframe.src;
                iframe.src = 'about:blank';
                
                setTimeout(() => {{
                    iframe.src = originalSrc + (originalSrc.includes('?') ? '&' : '?') + '_reload=' + Date.now();
                    sendMessage('force_reload:' + originalSrc);
                }}, 200);
            }}
        }}
        
        function navigateIframe(direction) {{
            const iframe = document.getElementById('main-iframe');
            const statusText = document.getElementById('status-text');
            
            if (iframe && iframe.contentWindow) {{
                try {{
                    if (direction === 'back') {{
                        iframe.contentWindow.history.back();
                        statusText.textContent = 'Navigating back...';
                    }} else if (direction === 'forward') {{
                        iframe.contentWindow.history.forward();
                        statusText.textContent = 'Navigating forward...';
                    }}
                    sendMessage('navigate:' + direction);
                }} catch (e) {{
                    statusText.textContent = 'Navigation blocked (cross-origin)';
                    sendMessage('navigate_error:' + e.message);
                    setTimeout(() => {{
                        statusText.textContent = 'Ready';
                    }}, 2000);
                }}
            }}
        }}
        
        function refreshPage() {{
            location.reload();
        }}
        
        function updateIframeSrc(newUrl) {{
            const iframe = document.getElementById('main-iframe');
            const urlDisplay = document.getElementById('url-display');
            const loadingOverlay = document.getElementById('loading-overlay');
            const statusText = document.getElementById('status-text');
            const spinner = document.getElementById('loading-spinner');
            
            if (iframe) {{
                currentUrl = newUrl;
                iframe.src = newUrl;
                loadingOverlay.style.display = 'flex';
                statusText.textContent = 'Loading new URL...';
                spinner.style.display = 'block';
                
                if (urlDisplay) {{
                    urlDisplay.textContent = newUrl;
                    urlDisplay.title = newUrl;
                }}
                sendMessage('navigate:' + newUrl);
            }}
        }}
        
        function handleIframeLoad() {{
            const loadingOverlay = document.getElementById('loading-overlay');
            const statusText = document.getElementById('status-text');
            const spinner = document.getElementById('loading-spinner');
            const iframe = document.getElementById('main-iframe');
            
            loadingOverlay.style.display = 'none';
            statusText.textContent = 'Content loaded successfully';
            spinner.style.display = 'none';
            
            setTimeout(() => {{
                statusText.textContent = 'Ready';
            }}, 2000);
            
            sendMessage('loaded:' + iframe.src);
        }}
        
        function handleIframeError() {{
            const loadingOverlay = document.getElementById('loading-overlay');
            const statusText = document.getElementById('status-text');
            const spinner = document.getElementById('loading-spinner');
            
            loadingOverlay.innerHTML = '<div style="color: #dc3545;"><div>❌ Failed to load content</div><div style="margin-top: 10px; font-size: 14px;">Try reloading or check the URL</div></div>';
            statusText.textContent = 'Load failed';
            spinner.style.display = 'none';
            
            sendMessage('error:' + currentUrl);
        }}
        
        // Communication with egui
        function sendMessage(msg) {{
            if (window.external && window.external.invoke) {{
                window.external.invoke(msg);
            }}
        }}
        
        // Send ready signal
        setTimeout(() => {{
            sendMessage('ready');
            document.getElementById('status-text').textContent = 'Ready';
        }}, 100);
        
        // Monitor iframe state
        setInterval(() => {{
            const iframe = document.getElementById('main-iframe');
            if (iframe && iframe.contentDocument) {{
                try {{
                    const iframeUrl = iframe.contentWindow.location.href;
                    if (iframeUrl !== currentUrl && iframeUrl !== 'about:blank') {{
                        currentUrl = iframeUrl;
                        document.getElementById('url-display').textContent = iframeUrl;
                        sendMessage('url_changed:' + iframeUrl);
                    }}
                }} catch (e) {{
                    // Cross-origin restrictions
                }}
            }}
        }}, 1000);
    </script>
</body>
</html>
        "#, target_url, target_url, target_url, target_url)
    }

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

        // Close existing WebView
        if self.webview.is_some() {
            self.webview = None;
            self.webview_created = false;
        }

        // Create HTML content with enhanced iframe
        let html_content = self.create_enhanced_html(&url);
        self.current_url = url.clone();
        self.is_loading = true;

        // Create WebView with the enhanced iframe HTML
        let result = web_view::builder()
            .title("Enhanced iframe WebView - Reliable Reload")
            .content(Content::Html(&html_content))
            .size(1000, 800)
            .resizable(true)
            .debug(false)
            .user_data(())
            .invoke_handler(move |_webview, arg| {
                println!("Enhanced WebView message: {}", arg);
                
                // Handle different message types
                if arg.starts_with("loaded:") {
                    println!("✅ iframe content loaded successfully");
                } else if arg.starts_with("error:") {
                    println!("❌ Failed to load iframe content");
                } else if arg.starts_with("force_reload:") {
                    let url = &arg[13..]; // Remove "force_reload:" prefix
                    println!("🔄 Force reloading iframe content: {}", url);
                } else if arg.starts_with("navigate:") {
                    let target = &arg[9..]; // Remove "navigate:" prefix
                    println!("🧭 iframe navigation: {}", target);
                } else if arg.starts_with("navigate_error:") {
                    let error = &arg[15..]; // Remove "navigate_error:" prefix
                    println!("⚠️ iframe navigation error: {}", error);
                } else if arg.starts_with("url_changed:") {
                    let new_url = &arg[12..]; // Remove "url_changed:" prefix
                    println!("🔗 iframe URL changed: {}", new_url);
                } else if arg == "ready" {
                    println!("🎯 Enhanced iframe WebView is ready");
                } else if arg == "fullscreen" {
                    println!("⛶ Fullscreen requested");
                }
                
                Ok(())
            })
            .build();

        match result {
            Ok(webview) => {
                self.webview = Some(webview);
                self.webview_created = true;
                self.is_loading = false;
                self.status_message = format!("✅ Created enhanced iframe WebView: {}", url);
            }
            Err(e) => {
                self.is_loading = false;
                self.status_message = format!("❌ Error creating WebView: {:?}", e);
            }
        }
    }

    fn step_webview(&mut self) {
        // Minimal stepping to avoid issues
        if self.webview.is_some() {
            // Keep WebView alive without calling step()
        }
    }

    fn close_webview(&mut self) {
        if self.webview.is_some() {
            println!("Closing enhanced iframe WebView...");
            self.webview = None;
            self.webview_created = false;
            self.is_loading = false;
            self.status_message = "Enhanced iframe WebView closed".to_string();
        }
    }

    fn eval_js(&mut self, js: &str) {
        if let Some(webview) = &mut self.webview {
            match webview.eval(js) {
                Ok(_) => {
                    self.status_message = format!("✅ JS executed: {}", js);
                }
                Err(e) => {
                    self.status_message = format!("❌ JS error: {:?}", e);
                }
            }
        }
    }

    fn navigate_to(&mut self, url: &str) {
        if let Some(_webview) = &mut self.webview {
            let js = format!("updateIframeSrc('{}');", url);
            self.eval_js(&js);
            self.current_url = url.to_string();
            self.is_loading = true;
            self.status_message = format!("🔄 Navigating to: {}", url);
        }
    }
    
    fn force_reload(&mut self) {
        if let Some(_webview) = &mut self.webview {
            let js = "forceReloadIframe();".to_string();
            self.eval_js(&js);
            self.reload_count += 1;
            self.is_loading = true;
            self.status_message = format!("🔄 Force reloading (#{}) : {}", self.reload_count, self.current_url);
        }
    }
}

impl eframe::App for EnhancedIframeApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Step WebView
        self.step_webview();

        // Top controls
        egui::TopBottomPanel::top("controls").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("URL:");
                ui.add(egui::TextEdit::singleline(&mut self.url_input)
                    .desired_width(350.0));
                
                if ui.button("🔗 Create Enhanced WebView").clicked() {
                    self.create_webview();
                }
                
                if self.webview_created && ui.button("🔄 Navigate").clicked() {
                    let url = if !self.url_input.starts_with("http://") && !self.url_input.starts_with("https://") {
                        format!("https://{}", self.url_input)
                    } else {
                        self.url_input.clone()
                    };
                    self.navigate_to(&url);
                }
                
                if self.webview_created && ui.button("🔄 Force Reload").clicked() {
                    self.force_reload();
                }
                
                if self.webview.is_some() && ui.button("❌ Close").clicked() {
                    self.close_webview();
                }
            });
            
            // Quick navigation
            if self.webview_created {
                ui.horizontal(|ui| {
                    ui.label("Quick navigate:");
                    if ui.button("🦀 Rust").clicked() {
                        self.navigate_to("https://www.rust-lang.org");
                    }
                    if ui.button("📖 egui").clicked() {
                        self.navigate_to("https://github.com/emilk/egui");
                    }
                    if ui.button("🌍 Wikipedia").clicked() {
                        self.navigate_to("https://en.wikipedia.org");
                    }
                    if ui.button("📰 Hacker News").clicked() {
                        self.navigate_to("https://news.ycombinator.com");
                    }
                    if ui.button("🔍 Google").clicked() {
                        self.navigate_to("https://www.google.com");
                    }
                });
            }
        });

        // Bottom status
        egui::TopBottomPanel::bottom("status").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Status:");
                if self.webview_created {
                    if self.is_loading {
                        ui.colored_label(egui::Color32::YELLOW, "🔄 Loading...");
                    } else {
                        ui.colored_label(egui::Color32::GREEN, "✅ Enhanced WebView Active");
                    }
                } else {
                    ui.colored_label(egui::Color32::GRAY, "⭕ No WebView");
                }
                ui.separator();
                ui.label(format!("Current URL: {}", self.current_url));
                ui.separator();
                ui.label(format!("Reloads: {}", self.reload_count));
            });
            ui.label(&self.status_message);
        });

        // Main content area
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("🌐 Enhanced iframe WebView with Reliable Reload");
            
            if self.webview_created {
                ui.separator();
                
                // Enhanced controls
                ui.horizontal(|ui| {
                    if ui.button("💬 Test Alert").clicked() {
                        self.eval_js("alert('Hello from enhanced iframe WebView!');");
                    }
                    if ui.button("🎨 Change Background").clicked() {
                        self.eval_js("document.body.style.background = 'linear-gradient(45deg, #ff6b6b, #4ecdc4)';");
                    }
                    if ui.button("📊 Show Reload Count").clicked() {
                        let js = format!("alert('This WebView has been reloaded {} times');", self.reload_count);
                        self.eval_js(&js);
                    }
                });
                
                ui.add_space(10.0);
                
                // Enhanced iframe visualization
                ui.group(|ui| {
                    ui.label("🌐 Enhanced iframe WebView Features");
                    ui.label("This version specifically addresses iframe reload issues with:");
                    
                    ui.add_space(5.0);
                    
                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            ui.label("✅ Force Reload Function");
                            ui.label("✅ Enhanced Loading States");
                            ui.label("✅ Reload Counter");
                            ui.label("✅ Better Error Handling");
                        });
                        
                        ui.separator();
                        
                        ui.vertical(|ui| {
                            ui.label("✅ URL Change Detection");
                            ui.label("✅ Cross-origin Safety");
                            ui.label("✅ Visual Loading Indicators");
                            ui.label("✅ Reliable Navigation");
                        });
                    });
                    
                    ui.add_space(10.0);
                    
                    // Visual representation
                    let available = ui.available_size();
                    let demo_size = egui::vec2(
                        (available.x - 20.0).max(600.0),
                        (available.y - 150.0).max(300.0)
                    );
                    
                    let (rect, _response) = ui.allocate_exact_size(
                        demo_size,
                        egui::Sense::hover()
                    );
                    
                    // Draw enhanced iframe representation
                    ui.painter().rect_filled(
                        rect,
                        egui::Rounding::same(8.0),
                        egui::Color32::from_rgba_premultiplied(76, 175, 80, 100)  // Green with transparency
                    );
                    
                    ui.painter().rect_stroke(
                        rect,
                        egui::Rounding::same(8.0),
                        egui::Stroke::new(3.0, egui::Color32::from_rgb(76, 175, 80))
                    );
                    
                    // Center text
                    let center = rect.center();
                    ui.painter().text(
                        center,
                        egui::Align2::CENTER_CENTER,
                        format!("🌐 Enhanced iframe WebView\n\nCurrent URL: {}\nReload Count: {}\nStatus: {}\n\nThe WebView above contains an enhanced iframe\nwith reliable reload functionality and better\nerror handling for web content display.",
                            if self.current_url.is_empty() { "None" } else { &self.current_url },
                            self.reload_count,
                            if self.is_loading { "Loading..." } else { "Ready" }
                        ),
                        egui::FontId::proportional(14.0),
                        egui::Color32::from_rgb(50, 50, 50)
                    );
                });
                
            } else {
                // Instructions
                ui.vertical_centered(|ui| {
                    ui.add_space(50.0);
                    ui.heading("Enhanced iframe WebView");
                    ui.add_space(20.0);
                    
                    ui.group(|ui| {
                        ui.label("🎯 Enhanced iframe Features:");
                        ui.label("• Reliable force reload functionality");
                        ui.label("• Enhanced loading states and indicators");
                        ui.label("• Reload counter for debugging");
                        ui.label("• Better error handling and recovery");
                        ui.label("• URL change detection and monitoring");
                        ui.label("• Cross-origin navigation safety");
                    });
                    
                    ui.add_space(20.0);
                    
                    ui.group(|ui| {
                        ui.label("🔧 Reload Problem Solutions:");
                        ui.label("• Force reload clears iframe src and resets it");
                        ui.label("• Adds cache-busting parameters to URLs");
                        ui.label("• Proper loading state management");
                        ui.label("• Visual feedback for all operations");
                        ui.label("• Separate controls for iframe vs page reload");
                        ui.label("• Enhanced error detection and reporting");
                    });
                });
            }
        });

        // No automatic repaint to avoid issues
    }
}

fn main() -> Result<(), eframe::Error> {
    env_logger::init();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 900.0])
            .with_title("Enhanced iframe WebView - Reliable Reload"),
        ..Default::default()
    };

    eframe::run_native(
        "Enhanced iframe WebView",
        options,
        Box::new(|_cc| Ok(Box::new(EnhancedIframeApp::default()))),
    )
}
