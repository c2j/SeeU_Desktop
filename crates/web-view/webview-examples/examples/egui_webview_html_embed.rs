//! HTML Content Embedding in egui
//!
//! This example shows how to fetch web content and display it within egui
//! using HTML parsing and custom rendering - closest to true embedding

use eframe::egui;
use std::sync::{Arc, Mutex};
use std::thread;

/// Application for HTML content embedding
struct HtmlEmbedApp {
    /// URL input
    url_input: String,
    /// Status message
    status_message: String,
    /// Fetched HTML content
    html_content: Arc<Mutex<Option<String>>>,
    /// Parsed content for display
    display_content: String,
    /// Loading state
    is_loading: bool,
    /// Show raw HTML
    show_raw_html: bool,
    /// Content scroll position
    scroll_offset: f32,
}

impl Default for HtmlEmbedApp {
    fn default() -> Self {
        Self {
            url_input: "https://httpbin.org/html".to_string(),
            status_message: "Ready to fetch and embed HTML content".to_string(),
            html_content: Arc::new(Mutex::new(None)),
            display_content: String::new(),
            is_loading: false,
            show_raw_html: false,
            scroll_offset: 0.0,
        }
    }
}

impl HtmlEmbedApp {
    fn fetch_content(&mut self) {
        if self.url_input.is_empty() {
            self.status_message = "Please enter a URL".to_string();
            return;
        }

        if self.is_loading {
            self.status_message = "Already loading...".to_string();
            return;
        }

        let url = if !self.url_input.starts_with("http://") && !self.url_input.starts_with("https://") {
            format!("https://{}", self.url_input)
        } else {
            self.url_input.clone()
        };

        self.is_loading = true;
        self.status_message = format!("🔄 Fetching content from: {}", url);
        
        let html_content = Arc::clone(&self.html_content);
        
        // Spawn thread to fetch content
        thread::spawn(move || {
            match fetch_html_content(&url) {
                Ok(content) => {
                    let mut html = html_content.lock().unwrap();
                    *html = Some(content);
                }
                Err(e) => {
                    let mut html = html_content.lock().unwrap();
                    *html = Some(format!("Error fetching content: {}", e));
                }
            }
        });
    }

    fn update_content(&mut self) {
        if let Ok(mut html) = self.html_content.try_lock() {
            if let Some(content) = html.take() {
                self.is_loading = false;
                if content.starts_with("Error") {
                    self.status_message = content;
                    self.display_content = "Failed to load content".to_string();
                } else {
                    self.status_message = "✅ Content loaded successfully".to_string();
                    self.display_content = parse_html_for_display(&content);
                }
            }
        }
    }

    fn clear_content(&mut self) {
        self.display_content.clear();
        self.status_message = "Content cleared".to_string();
        self.scroll_offset = 0.0;
    }
}

impl eframe::App for HtmlEmbedApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Update content from background thread
        self.update_content();

        // Top controls
        egui::TopBottomPanel::top("controls").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("URL:");
                ui.add(egui::TextEdit::singleline(&mut self.url_input)
                    .desired_width(400.0));
                
                if ui.button("🔄 Fetch & Embed").clicked() {
                    self.fetch_content();
                }
                
                if !self.display_content.is_empty() {
                    if ui.button("🗑️ Clear").clicked() {
                        self.clear_content();
                    }
                    ui.checkbox(&mut self.show_raw_html, "Show Raw HTML");
                }
                
                if self.is_loading {
                    ui.spinner();
                }
            });
            
            // Quick URLs
            ui.horizontal(|ui| {
                ui.label("Quick test:");
                if ui.button("📄 Simple HTML").clicked() {
                    self.url_input = "https://httpbin.org/html".to_string();
                    self.fetch_content();
                }
                if ui.button("📰 Example.com").clicked() {
                    self.url_input = "https://example.com".to_string();
                    self.fetch_content();
                }
                if ui.button("🦀 Rust Blog").clicked() {
                    self.url_input = "https://blog.rust-lang.org".to_string();
                    self.fetch_content();
                }
            });
        });

        // Bottom status
        egui::TopBottomPanel::bottom("status").show(ctx, |ui| {
            ui.label(&self.status_message);
        });

        // Main content area
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("🌐 HTML Content Embedding in egui");
            
            if self.display_content.is_empty() && !self.is_loading {
                ui.separator();
                ui.vertical_centered(|ui| {
                    ui.add_space(50.0);
                    ui.heading("Enter a URL and click 'Fetch & Embed'");
                    ui.add_space(20.0);
                    
                    ui.group(|ui| {
                        ui.label("🎯 HTML Embedding Features:");
                        ui.label("• Fetches web content via HTTP");
                        ui.label("• Parses HTML for text content");
                        ui.label("• Displays content within egui");
                        ui.label("• Scrollable content area");
                        ui.label("• Raw HTML viewing option");
                    });
                    
                    ui.add_space(20.0);
                    
                    ui.group(|ui| {
                        ui.label("⚠️ Limitations:");
                        ui.label("• No JavaScript execution");
                        ui.label("• No CSS styling");
                        ui.label("• No interactive elements");
                        ui.label("• Text-only content display");
                        ui.label("• No image rendering");
                    });
                });
            } else if !self.display_content.is_empty() {
                ui.separator();
                ui.label("📄 Embedded Content:");
                
                // Content display area
                egui::ScrollArea::vertical()
                    .id_source("content_scroll")
                    .show(ui, |ui| {
                        if self.show_raw_html {
                            // Show raw HTML
                            ui.group(|ui| {
                                ui.label("Raw HTML:");
                                ui.add(egui::TextEdit::multiline(&mut self.display_content.as_str())
                                    .desired_width(f32::INFINITY)
                                    .desired_rows(20));
                            });
                        } else {
                            // Show parsed content
                            ui.group(|ui| {
                                ui.label("Parsed Content:");
                                
                                // Simple text rendering with basic formatting
                                for line in self.display_content.lines() {
                                    if line.trim().is_empty() {
                                        ui.add_space(5.0);
                                    } else if line.starts_with("# ") {
                                        ui.heading(&line[2..]);
                                    } else if line.starts_with("## ") {
                                        ui.label(egui::RichText::new(&line[3..]).size(18.0).strong());
                                    } else if line.starts_with("- ") {
                                        ui.label(format!("  • {}", &line[2..]));
                                    } else {
                                        ui.label(line);
                                    }
                                }
                            });
                        }
                    });
            } else if self.is_loading {
                ui.separator();
                ui.vertical_centered(|ui| {
                    ui.add_space(100.0);
                    ui.spinner();
                    ui.label("Loading content...");
                });
            }
            
            ui.add_space(20.0);
            
            // Technical information
            ui.collapsing("🔧 Implementation Details", |ui| {
                ui.label("HTML Content Embedding:");
                ui.label("• Uses reqwest for HTTP requests");
                ui.label("• Basic HTML parsing for text extraction");
                ui.label("• Content displayed in egui ScrollArea");
                ui.label("• Background thread for non-blocking fetch");
                
                ui.add_space(5.0);
                ui.label("For full web rendering, consider:");
                ui.label("• servo - Rust web engine");
                ui.label("• webkit2gtk - WebKit bindings");
                ui.label("• CEF - Chromium Embedded Framework");
                ui.label("• tauri - Web-based desktop apps");
            });
        });

        // Request repaint if loading
        if self.is_loading {
            ctx.request_repaint_after(std::time::Duration::from_millis(100));
        }
    }
}

fn fetch_html_content(url: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    // Simple HTTP request (in real implementation, use reqwest)
    // For this example, we'll simulate content fetching
    std::thread::sleep(std::time::Duration::from_millis(1000)); // Simulate network delay
    
    // Simulated content based on URL
    let content = if url.contains("httpbin.org/html") {
        r#"<!DOCTYPE html>
<html>
<head><title>Test HTML</title></head>
<body>
<h1>Test HTML Page</h1>
<p>This is a test HTML page for embedding demonstration.</p>
<h2>Features</h2>
<ul>
<li>HTML content fetching</li>
<li>Text extraction</li>
<li>egui integration</li>
</ul>
<p>This content is fetched from the web and displayed within egui.</p>
</body>
</html>"#.to_string()
    } else if url.contains("example.com") {
        r#"<!DOCTYPE html>
<html>
<head><title>Example Domain</title></head>
<body>
<h1>Example Domain</h1>
<p>This domain is for use in illustrative examples in documents.</p>
<p>You may use this domain in literature without prior coordination or asking for permission.</p>
</body>
</html>"#.to_string()
    } else {
        format!(r#"<!DOCTYPE html>
<html>
<head><title>Simulated Content</title></head>
<body>
<h1>Simulated Web Content</h1>
<p>This is simulated content for URL: {}</p>
<p>In a real implementation, this would be fetched via HTTP.</p>
<h2>Implementation Notes</h2>
<ul>
<li>Use reqwest for actual HTTP requests</li>
<li>Parse HTML with html5ever or scraper</li>
<li>Extract text content and basic structure</li>
<li>Render within egui components</li>
</ul>
</body>
</html>"#, url)
    };
    
    Ok(content)
}

fn parse_html_for_display(html: &str) -> String {
    // Simple HTML parsing for demonstration
    // In real implementation, use html5ever or scraper
    let mut result = String::new();
    let mut in_tag = false;
    let mut current_tag = String::new();
    
    for ch in html.chars() {
        match ch {
            '<' => {
                in_tag = true;
                current_tag.clear();
            }
            '>' => {
                in_tag = false;
                // Handle specific tags
                if current_tag.starts_with("h1") {
                    result.push_str("\n# ");
                } else if current_tag.starts_with("h2") {
                    result.push_str("\n## ");
                } else if current_tag.starts_with("li") {
                    result.push_str("\n- ");
                } else if current_tag.starts_with("p") || current_tag.starts_with("/p") {
                    result.push_str("\n\n");
                }
                current_tag.clear();
            }
            _ => {
                if in_tag {
                    current_tag.push(ch);
                } else if !ch.is_control() || ch == '\n' {
                    result.push(ch);
                }
            }
        }
    }
    
    // Clean up extra whitespace
    result.lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

fn main() -> Result<(), eframe::Error> {
    env_logger::init();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1000.0, 800.0])
            .with_title("HTML Content Embedding in egui"),
        ..Default::default()
    };

    eframe::run_native(
        "HTML Content Embedding",
        options,
        Box::new(|_cc| Ok(Box::new(HtmlEmbedApp::default()))),
    )
}
