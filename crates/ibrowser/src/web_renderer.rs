//! Web content renderer for iBrowser
//!
//! This module fetches and renders real web content within the egui interface.

use scraper::{Html, Selector};
use std::collections::HashMap;
use std::sync::Arc;

/// Parsed web page content
#[derive(Debug, Clone)]
pub struct WebPageContent {
    pub title: String,
    pub description: String,
    pub links: Vec<WebLink>,
    pub headings: Vec<WebHeading>,
    pub paragraphs: Vec<String>,
    pub images: Vec<WebImage>,
    pub meta_info: HashMap<String, String>,
}

/// Web link information
#[derive(Debug, Clone)]
pub struct WebLink {
    pub text: String,
    pub url: String,
    pub is_external: bool,
}

/// Web heading information
#[derive(Debug, Clone)]
pub struct WebHeading {
    pub level: u8, // 1-6 for h1-h6
    pub text: String,
}

/// Web image information
#[derive(Debug, Clone)]
pub struct WebImage {
    pub alt: String,
    pub src: String,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub data: Option<Arc<Vec<u8>>>, // Downloaded image data
}

impl Default for WebPageContent {
    fn default() -> Self {
        Self {
            title: String::new(),
            description: String::new(),
            links: Vec::new(),
            headings: Vec::new(),
            paragraphs: Vec::new(),
            images: Vec::new(),
            meta_info: HashMap::new(),
        }
    }
}

/// Fetch and parse web page content
pub async fn fetch_and_parse_webpage(url: &str) -> anyhow::Result<WebPageContent> {
    log::info!("Fetching and parsing webpage: {}", url);
    
    // Create HTTP client
    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36")
        .timeout(std::time::Duration::from_secs(10))
        .build()?;
    
    // Fetch the webpage
    let response = client.get(url).send().await?;
    
    if !response.status().is_success() {
        return Err(anyhow::anyhow!("HTTP error: {}", response.status()));
    }
    
    let html_content = response.text().await?;
    
    // Parse HTML
    let document = Html::parse_document(&html_content);
    let base_url = url::Url::parse(url)?;
    
    let mut content = WebPageContent::default();
    
    // Extract title
    if let Some(title_element) = document.select(&Selector::parse("title").unwrap()).next() {
        content.title = title_element.text().collect::<String>().trim().to_string();
    }
    
    // Extract meta description
    if let Some(desc_element) = document.select(&Selector::parse("meta[name='description']").unwrap()).next() {
        if let Some(desc) = desc_element.value().attr("content") {
            content.description = desc.to_string();
        }
    }
    
    // Extract headings (h1-h6)
    for level in 1..=6 {
        let selector = Selector::parse(&format!("h{}", level)).unwrap();
        for element in document.select(&selector) {
            let text = element.text().collect::<String>().trim().to_string();
            if !text.is_empty() {
                content.headings.push(WebHeading {
                    level: level as u8,
                    text,
                });
            }
        }
    }
    
    // Extract paragraphs and other text content
    let text_selectors = ["p", "div", "article", "section"];
    for selector_str in &text_selectors {
        let selector = Selector::parse(selector_str).unwrap();
        for element in document.select(&selector) {
            let text = element.text().collect::<String>().trim().to_string();
            if !text.is_empty() && text.len() > 20 { // Filter out very short text
                // Avoid duplicates
                if !content.paragraphs.contains(&text) {
                    content.paragraphs.push(text);
                }
            }
        }
    }
    
    // Extract links
    let a_selector = Selector::parse("a[href]").unwrap();
    for element in document.select(&a_selector) {
        if let Some(href) = element.value().attr("href") {
            let text = element.text().collect::<String>().trim().to_string();
            if !text.is_empty() {
                let absolute_url = base_url.join(href).unwrap_or_else(|_| base_url.clone());
                let is_external = absolute_url.host_str() != base_url.host_str();
                
                content.links.push(WebLink {
                    text,
                    url: absolute_url.to_string(),
                    is_external,
                });
            }
        }
    }
    
    // Extract images and download them
    let img_selector = Selector::parse("img[src]").unwrap();
    for element in document.select(&img_selector) {
        if let Some(src) = element.value().attr("src") {
            let alt = element.value().attr("alt").unwrap_or("").to_string();
            let absolute_url = base_url.join(src).unwrap_or_else(|_| base_url.clone());

            // Try to download the image
            let image_data = match download_image(&client, &absolute_url.to_string()).await {
                Ok(data) => Some(Arc::new(data)),
                Err(e) => {
                    log::warn!("Failed to download image {}: {}", absolute_url, e);
                    None
                }
            };

            content.images.push(WebImage {
                alt,
                src: absolute_url.to_string(),
                width: None,
                height: None,
                data: image_data,
            });
        }
    }
    
    // Extract some meta information
    let meta_selector = Selector::parse("meta[name]").unwrap();
    for element in document.select(&meta_selector) {
        if let (Some(name), Some(content_attr)) = (
            element.value().attr("name"),
            element.value().attr("content")
        ) {
            content.meta_info.insert(name.to_string(), content_attr.to_string());
        }
    }
    
    log::info!("Successfully parsed webpage: {} headings, {} paragraphs, {} links, {} images", 
               content.headings.len(), content.paragraphs.len(), content.links.len(), content.images.len());
    
    Ok(content)
}

/// Download image data from URL
async fn download_image(client: &reqwest::Client, url: &str) -> anyhow::Result<Vec<u8>> {
    log::debug!("Downloading image: {}", url);

    let response = client.get(url)
        .timeout(std::time::Duration::from_secs(5))
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(anyhow::anyhow!("HTTP error: {}", response.status()));
    }

    let content_type = response.headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    // Check if it's an image
    if !content_type.starts_with("image/") {
        return Err(anyhow::anyhow!("Not an image: {}", content_type));
    }

    let bytes = response.bytes().await?;
    Ok(bytes.to_vec())
}

/// Render web content in egui with improved styling
pub fn render_web_content(ui: &mut egui::Ui, content: &WebPageContent, current_url: &str) {
    // Create a styled frame for the web content
    egui::Frame::none()
        .fill(egui::Color32::WHITE)
        .stroke(egui::Stroke::new(1.0, egui::Color32::LIGHT_GRAY))
        .inner_margin(egui::Margin::same(0.0))
        .show(ui, |ui| {
            egui::ScrollArea::vertical()
                .auto_shrink([false; 2])
                .show(ui, |ui| {
                    ui.vertical(|ui| {
                        // Add some padding
                        ui.add_space(15.0);
                        ui.horizontal(|ui| {
                            ui.add_space(15.0);
                            ui.vertical(|ui| {
                
                                // URL bar style header
                                ui.horizontal(|ui| {
                                    ui.label("🌐");
                                    ui.label(egui::RichText::new(current_url).small().color(egui::Color32::GRAY));
                                });
                                ui.add_space(10.0);

                                // Page title with better styling
                                if !content.title.is_empty() {
                                    ui.label(egui::RichText::new(&content.title).heading().strong().size(24.0));
                                    ui.add_space(8.0);
                                }

                                // Page description with better styling
                                if !content.description.is_empty() {
                                    ui.label(egui::RichText::new(&content.description).italics().color(egui::Color32::DARK_GRAY).size(14.0));
                                    ui.add_space(15.0);
                                }
                
                // Headings and content
                let mut paragraph_index = 0;
                for heading in &content.headings {
                    // Show heading
                    let heading_style = match heading.level {
                        1 => egui::RichText::new(&heading.text).heading().strong(),
                        2 => egui::RichText::new(&heading.text).size(18.0).strong(),
                        3 => egui::RichText::new(&heading.text).size(16.0).strong(),
                        _ => egui::RichText::new(&heading.text).size(14.0).strong(),
                    };
                    ui.label(heading_style);
                    ui.add_space(5.0);
                    
                    // Show a few paragraphs after each heading
                    let paragraphs_to_show = 2;
                    for _ in 0..paragraphs_to_show {
                        if paragraph_index < content.paragraphs.len() {
                            ui.label(&content.paragraphs[paragraph_index]);
                            ui.add_space(8.0);
                            paragraph_index += 1;
                        }
                    }
                    
                    ui.add_space(10.0);
                }
                
                // Show remaining paragraphs
                while paragraph_index < content.paragraphs.len() {
                    ui.label(&content.paragraphs[paragraph_index]);
                    ui.add_space(8.0);
                    paragraph_index += 1;
                }
                
                // Images section
                if !content.images.is_empty() {
                    ui.separator();
                    ui.add_space(10.0);
                    ui.label(egui::RichText::new("🖼️ 页面图片").strong());
                    ui.add_space(5.0);

                    for (i, image) in content.images.iter().take(5).enumerate() { // Show first 5 images
                        ui.horizontal(|ui| {
                            // Try to display the image if we have data
                            if let Some(image_data) = &image.data {
                                match image::load_from_memory(image_data) {
                                    Ok(img) => {
                                        let rgba_image = img.to_rgba8();
                                        let size = [rgba_image.width() as usize, rgba_image.height() as usize];
                                        let pixels = rgba_image.as_flat_samples();

                                        // Create egui texture
                                        let color_image = egui::ColorImage::from_rgba_unmultiplied(size, pixels.as_slice());
                                        let texture = ui.ctx().load_texture(
                                            format!("image_{}", i),
                                            color_image,
                                            egui::TextureOptions::default()
                                        );

                                        // Display image with reasonable size
                                        let max_size = egui::Vec2::new(300.0, 200.0);
                                        let image_size = egui::Vec2::new(size[0] as f32, size[1] as f32);
                                        let scale = (max_size.x / image_size.x).min(max_size.y / image_size.y).min(1.0);
                                        let display_size = image_size * scale;

                                        ui.add(egui::Image::from_texture(&texture).max_size(display_size));
                                    }
                                    Err(e) => {
                                        log::warn!("Failed to decode image: {}", e);
                                        ui.label(format!("🖼️ [图片: {}]", image.alt));
                                    }
                                }
                            } else {
                                ui.label(format!("🖼️ [图片: {}]", image.alt));
                            }
                        });
                        ui.add_space(5.0);
                    }

                    if content.images.len() > 5 {
                        ui.label(format!("... 还有 {} 张图片", content.images.len() - 5));
                    }
                    ui.add_space(10.0);
                }

                // Links section
                if !content.links.is_empty() {
                    ui.separator();
                    ui.add_space(10.0);
                    ui.label(egui::RichText::new("🔗 页面链接").strong());
                    ui.add_space(5.0);
                    
                    for (i, link) in content.links.iter().take(10).enumerate() { // Show first 10 links
                        ui.horizontal(|ui| {
                            let link_color = if link.is_external {
                                egui::Color32::from_rgb(0, 100, 200)
                            } else {
                                egui::Color32::from_rgb(0, 150, 0)
                            };
                            
                            if ui.link(egui::RichText::new(&link.text).color(link_color)).clicked() {
                                log::info!("Link clicked: {}", link.url);
                                // In a real implementation, this could navigate to the link
                            }
                            
                            if link.is_external {
                                ui.label("🔗");
                            }
                        });
                        
                        if i >= 9 && content.links.len() > 10 {
                            ui.label(format!("... 还有 {} 个链接", content.links.len() - 10));
                            break;
                        }
                    }
                }
                
                // Images section
                if !content.images.is_empty() {
                    ui.add_space(10.0);
                    ui.separator();
                    ui.add_space(10.0);
                    ui.label(egui::RichText::new("🖼️ 页面图片").strong());
                    ui.add_space(5.0);
                    
                    for (i, image) in content.images.iter().take(5).enumerate() { // Show first 5 images
                        ui.horizontal(|ui| {
                            ui.label("📷");
                            ui.label(if image.alt.is_empty() { 
                                "图片" 
                            } else { 
                                &image.alt 
                            });
                        });
                        
                        if i >= 4 && content.images.len() > 5 {
                            ui.label(format!("... 还有 {} 张图片", content.images.len() - 5));
                            break;
                        }
                    }
                }
                
                ui.add_space(20.0);
                ui.separator();
                ui.add_space(10.0);
                                ui.add_space(15.0);
                                ui.label(egui::RichText::new("💡 这是真实网页内容的结构化显示").small().color(egui::Color32::GRAY));
                                ui.label(egui::RichText::new("包含了页面的标题、段落、链接和图片信息").small().color(egui::Color32::GRAY));
                                ui.add_space(15.0);
                            });
                            ui.add_space(15.0);
                        });
                        ui.add_space(15.0);
                    });
                });
        });
}
