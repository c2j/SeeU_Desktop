//! True WebView Embedding using iframe approach
//!
//! This example creates a WebView that contains an iframe pointing to the target URL,
//! making it appear as if the web content is embedded within the egui window

use eframe::egui;
use web_view::*;

/// Application for iframe-based embedding
struct IframeEmbedApp {
    /// Target URL input
    url_input: String,
    /// Status message
    status_message: String,
    /// WebView instance
    webview: Option<WebView<'static, ()>>,
    /// WebView created flag
    webview_created: bool,
    /// Current embedded URL
    current_url: String,
}

impl Default for IframeEmbedApp {
    fn default() -> Self {
        Self {
            url_input: "https://www.rust-lang.org".to_string(),
            status_message: "Ready to create iframe-embedded WebView".to_string(),
            webview: None,
            webview_created: false,
            current_url: String::new(),
        }
    }
}

impl IframeEmbedApp {
    fn create_iframe_html(&self, target_url: &str) -> String {
        format!(r#"
<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>Embedded WebView</title>
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
            background: rgba(255, 255, 255, 0.1);
            backdrop-filter: blur(10px);
            padding: 10px 20px;
            display: flex;
            align-items: center;
            justify-content: space-between;
            border-bottom: 1px solid rgba(255, 255, 255, 0.2);
        }}
        
        .header h1 {{
            color: white;
            font-size: 18px;
            font-weight: 600;
        }}
        
        .url-display {{
            color: rgba(255, 255, 255, 0.8);
            font-size: 12px;
            font-family: monospace;
            background: rgba(0, 0, 0, 0.2);
            padding: 4px 8px;
            border-radius: 4px;
            max-width: 300px;
            overflow: hidden;
            text-overflow: ellipsis;
            white-space: nowrap;
        }}
        
        .iframe-container {{
            flex: 1;
            position: relative;
            margin: 10px;
            border-radius: 8px;
            overflow: hidden;
            box-shadow: 0 8px 32px rgba(0, 0, 0, 0.3);
        }}
        
        .iframe-container iframe {{
            width: 100%;
            height: 100%;
            border: none;
            background: white;
        }}
        
        .loading {{
            position: absolute;
            top: 50%;
            left: 50%;
            transform: translate(-50%, -50%);
            color: white;
            font-size: 16px;
            z-index: 10;
        }}
        
        .controls {{
            background: rgba(255, 255, 255, 0.1);
            backdrop-filter: blur(10px);
            padding: 8px 20px;
            display: flex;
            gap: 10px;
            border-top: 1px solid rgba(255, 255, 255, 0.2);
        }}
        
        .btn {{
            background: rgba(255, 255, 255, 0.2);
            border: 1px solid rgba(255, 255, 255, 0.3);
            color: white;
            padding: 6px 12px;
            border-radius: 4px;
            cursor: pointer;
            font-size: 12px;
            transition: all 0.2s;
        }}
        
        .btn:hover {{
            background: rgba(255, 255, 255, 0.3);
            transform: translateY(-1px);
        }}
        
        .status {{
            color: rgba(255, 255, 255, 0.7);
            font-size: 11px;
            margin-left: auto;
            display: flex;
            align-items: center;
        }}
        
        .status::before {{
            content: "🌐";
            margin-right: 5px;
        }}
    </style>
</head>
<body>
    <div class="header">
        <h1>🎯 Embedded WebView</h1>
        <div class="url-display" title="{}">{}</div>
    </div>
    
    <div class="iframe-container">
        <div class="loading" id="loading">Loading web content...</div>
        <iframe 
            src="{}" 
            onload="document.getElementById('loading').style.display='none'"
            onerror="document.getElementById('loading').innerHTML='❌ Failed to load content'"
            sandbox="allow-same-origin allow-scripts allow-forms allow-popups allow-popups-to-escape-sandbox"
        ></iframe>
    </div>
    
    <div class="controls">
        <button class="btn" onclick="reloadIframe()">🔄 Reload</button>
        <button class="btn" onclick="navigateIframe('back')">⬅️ Back</button>
        <button class="btn" onclick="navigateIframe('forward')">➡️ Forward</button>
        <button class="btn" onclick="window.external.invoke('fullscreen')">⛶ Fullscreen</button>
        <div class="status">Embedded in egui</div>
    </div>
    
    <script>
        // Handle iframe loading
        const iframe = document.querySelector('iframe');
        const loading = document.getElementById('loading');
        
        iframe.addEventListener('load', function() {{
            loading.style.display = 'none';
            window.external.invoke('loaded:' + iframe.src);
        }});
        
        iframe.addEventListener('error', function() {{
            loading.innerHTML = '❌ Failed to load content';
            loading.style.color = '#ff6b6b';
            window.external.invoke('error:' + iframe.src);
        }});
        
        // Communicate with egui
        function sendMessage(msg) {{
            if (window.external && window.external.invoke) {{
                window.external.invoke(msg);
            }}
        }}
        
        // Send ready signal
        setTimeout(() => {{
            sendMessage('ready');
        }}, 100);

        // iframe control functions
        function reloadIframe() {{
            const iframe = document.querySelector('iframe');
            if (iframe) {{
                const currentSrc = iframe.src;
                iframe.src = '';
                setTimeout(() => {{
                    iframe.src = currentSrc;
                    document.getElementById('loading').style.display = 'block';
                    sendMessage('reload:' + currentSrc);
                }}, 100);
            }}
        }}

        function navigateIframe(direction) {{
            const iframe = document.querySelector('iframe');
            if (iframe && iframe.contentWindow) {{
                try {{
                    if (direction === 'back') {{
                        iframe.contentWindow.history.back();
                    }} else if (direction === 'forward') {{
                        iframe.contentWindow.history.forward();
                    }}
                    sendMessage('navigate:' + direction);
                }} catch (e) {{
                    // Cross-origin restrictions may prevent this
                    sendMessage('navigate_error:' + e.message);
                }}
            }}
        }}

        function updateIframeSrc(newUrl) {{
            const iframe = document.querySelector('iframe');
            const urlDisplay = document.querySelector('.url-display');
            if (iframe) {{
                iframe.src = newUrl;
                document.getElementById('loading').style.display = 'block';
                if (urlDisplay) {{
                    urlDisplay.textContent = newUrl;
                    urlDisplay.title = newUrl;
                }}
                sendMessage('navigate:' + newUrl);
            }}
        }}
    </script>
</body>
</html>
        "#, target_url, target_url, target_url)
    }

    fn create_embedded_webview(&mut self) {
        if self.url_input.is_empty() {
            self.status_message = "Please enter a URL".to_string();
            return;
        }

        let target_url = if !self.url_input.starts_with("http://") && !self.url_input.starts_with("https://") {
            format!("https://{}", self.url_input)
        } else {
            self.url_input.clone()
        };

        // Close existing WebView
        if self.webview.is_some() {
            self.webview = None;
            self.webview_created = false;
        }

        // Create HTML content with embedded iframe
        let html_content = self.create_iframe_html(&target_url);
        self.current_url = target_url.clone();

        // Create WebView with the iframe HTML
        let result = web_view::builder()
            .title("Embedded WebView - egui Integration")
            .content(Content::Html(&html_content))
            .size(900, 700)
            .resizable(true)
            .debug(false)
            .user_data(())
            .invoke_handler(move |_webview, arg| {
                println!("WebView message: {}", arg);

                // Handle different message types
                if arg.starts_with("loaded:") {
                    println!("✅ Content loaded successfully");
                } else if arg.starts_with("error:") {
                    println!("❌ Failed to load content");
                } else if arg.starts_with("reload:") {
                    let url = &arg[7..]; // Remove "reload:" prefix
                    println!("🔄 Reloading iframe content: {}", url);
                } else if arg.starts_with("navigate:") {
                    let target = &arg[9..]; // Remove "navigate:" prefix
                    println!("🧭 Navigating iframe: {}", target);
                } else if arg.starts_with("navigate_error:") {
                    let error = &arg[15..]; // Remove "navigate_error:" prefix
                    println!("⚠️ Navigation error: {}", error);
                } else if arg == "ready" {
                    println!("🎯 Embedded WebView is ready");
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
                self.status_message = format!("✅ Created embedded WebView with iframe: {}", target_url);
            }
            Err(e) => {
                self.status_message = format!("❌ Error creating WebView: {:?}", e);
            }
        }
    }

    fn step_webview(&mut self) {
        if let Some(webview) = &mut self.webview {
            match webview.step() {
                Some(Ok(_)) => {
                    // WebView is running
                }
                Some(Err(e)) => {
                    self.status_message = format!("WebView error: {:?}", e);
                    self.webview = None;
                    self.webview_created = false;
                }
                None => {
                    self.webview = None;
                    self.webview_created = false;
                    self.status_message = "WebView was closed".to_string();
                }
            }
        }
    }

    fn close_webview(&mut self) {
        if self.webview.is_some() {
            self.webview = None;
            self.webview_created = false;
            self.status_message = "WebView closed".to_string();
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
        if let Some(webview) = &mut self.webview {
            let js = format!("updateIframeSrc('{}');", url);
            self.eval_js(&js);
            self.current_url = url.to_string();
            self.status_message = format!("🔄 Navigating to: {}", url);
        }
    }

    fn reload_current_page(&mut self) {
        if let Some(webview) = &mut self.webview {
            let js = "reloadIframe();".to_string();
            self.eval_js(&js);
            self.status_message = format!("🔄 Reloading: {}", self.current_url);
        }
    }
}

impl eframe::App for IframeEmbedApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Step WebView
        self.step_webview();

        // Top controls
        egui::TopBottomPanel::top("controls").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("URL:");
                ui.add(egui::TextEdit::singleline(&mut self.url_input)
                    .desired_width(400.0));
                
                if ui.button("🔗 Create Embedded WebView").clicked() {
                    self.create_embedded_webview();
                }
                
                if self.webview_created && ui.button("🔄 Navigate").clicked() {
                    let url = if !self.url_input.starts_with("http://") && !self.url_input.starts_with("https://") {
                        format!("https://{}", self.url_input)
                    } else {
                        self.url_input.clone()
                    };
                    self.navigate_to(&url);
                }

                if self.webview_created && ui.button("🔄 Reload Page").clicked() {
                    self.reload_current_page();
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
                    if ui.button("🔍 GitHub").clicked() {
                        self.navigate_to("https://github.com");
                    }
                });
            }
        });

        // Bottom status
        egui::TopBottomPanel::bottom("status").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Status:");
                if self.webview_created {
                    ui.colored_label(egui::Color32::GREEN, "✅ Embedded WebView Active");
                    if !self.current_url.is_empty() {
                        ui.separator();
                        ui.label(format!("Current: {}", self.current_url));
                    }
                } else {
                    ui.colored_label(egui::Color32::GRAY, "⭕ No WebView");
                }
            });
            ui.label(&self.status_message);
        });

        // Main content area
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("🎯 True WebView Embedding with iframe");
            
            if self.webview_created {
                ui.separator();
                
                // WebView controls
                ui.horizontal(|ui| {
                    if ui.button("💬 Alert in iframe").clicked() {
                        self.eval_js("document.querySelector('iframe').contentWindow.postMessage('alert(\"Hello from egui!\")', '*');");
                    }
                    if ui.button("🎨 Change iframe style").clicked() {
                        self.eval_js("document.querySelector('.iframe-container').style.border = '3px solid #4CAF50';");
                    }
                    if ui.button("📄 Get iframe info").clicked() {
                        self.eval_js("window.external.invoke('iframe-url:' + document.querySelector('iframe').src);");
                    }
                });
                
                ui.add_space(10.0);
                
                // Embedding visualization
                ui.group(|ui| {
                    ui.label("🌐 WebView with Embedded iframe");
                    ui.label("The WebView contains a custom HTML page with an iframe that loads the target URL.");
                    ui.label("This creates the appearance of embedded web content within the WebView window.");
                    
                    ui.add_space(10.0);
                    
                    // Visual representation
                    let available = ui.available_size();
                    let demo_size = egui::vec2(
                        (available.x - 20.0).max(400.0),
                        200.0
                    );
                    
                    let (rect, _response) = ui.allocate_exact_size(
                        demo_size,
                        egui::Sense::hover()
                    );
                    
                    // Draw WebView representation
                    ui.painter().rect_filled(
                        rect,
                        egui::Rounding::same(8.0),
                        egui::Color32::from_rgb(240, 248, 255)
                    );
                    
                    ui.painter().rect_stroke(
                        rect,
                        egui::Rounding::same(8.0),
                        egui::Stroke::new(2.0, egui::Color32::from_rgb(76, 175, 80))
                    );
                    
                    // Header
                    let header_rect = egui::Rect::from_min_size(
                        rect.min,
                        egui::vec2(rect.width(), 30.0)
                    );
                    ui.painter().rect_filled(
                        header_rect,
                        egui::Rounding::same(8.0),
                        egui::Color32::from_rgb(102, 126, 234)
                    );
                    
                    ui.painter().text(
                        header_rect.center(),
                        egui::Align2::CENTER_CENTER,
                        "🎯 Embedded WebView (with iframe)",
                        egui::FontId::proportional(14.0),
                        egui::Color32::WHITE
                    );
                    
                    // Content area
                    let content_rect = egui::Rect::from_min_size(
                        rect.min + egui::vec2(10.0, 40.0),
                        egui::vec2(rect.width() - 20.0, rect.height() - 60.0)
                    );
                    ui.painter().rect_filled(
                        content_rect,
                        egui::Rounding::same(4.0),
                        egui::Color32::WHITE
                    );
                    
                    ui.painter().text(
                        content_rect.center(),
                        egui::Align2::CENTER_CENTER,
                        format!("iframe: {}", self.current_url),
                        egui::FontId::monospace(12.0),
                        egui::Color32::from_rgb(100, 100, 100)
                    );
                });
                
            } else {
                // Instructions
                ui.vertical_centered(|ui| {
                    ui.add_space(50.0);
                    ui.heading("Create iframe-embedded WebView");
                    ui.add_space(20.0);
                    
                    ui.group(|ui| {
                        ui.label("🎯 iframe Embedding Approach:");
                        ui.label("• Creates a WebView with custom HTML content");
                        ui.label("• HTML contains an iframe pointing to target URL");
                        ui.label("• Provides embedded appearance with full functionality");
                        ui.label("• Supports navigation and JavaScript interaction");
                        ui.label("• Works around native embedding limitations");
                    });
                    
                    ui.add_space(20.0);
                    
                    ui.group(|ui| {
                        ui.label("✅ Advantages:");
                        ui.label("• True web content display within WebView");
                        ui.label("• No segmentation faults or threading issues");
                        ui.label("• Full JavaScript and CSS support");
                        ui.label("• Cross-platform compatibility");
                        ui.label("• Easy to implement and maintain");
                    });
                });
            }
        });

        // Request repaint for WebView stepping
        if self.webview_created {
            ctx.request_repaint_after(std::time::Duration::from_millis(16));
        }
    }
}

fn main() -> Result<(), eframe::Error> {
    env_logger::init();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 900.0])
            .with_title("True WebView Embedding with iframe"),
        ..Default::default()
    };

    eframe::run_native(
        "iframe Embedded WebView",
        options,
        Box::new(|_cc| Ok(Box::new(IframeEmbedApp::default()))),
    )
}
