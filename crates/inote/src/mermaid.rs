use egui::{TextureHandle, Ui};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use usvg::TreeParsing;
use once_cell::sync::Lazy;

/// Font cache to avoid repeated font loading and matching
#[derive(Debug)]
struct FontCache {
    /// Cached fontdb instance (shared)
    fontdb: Option<Arc<fontdb::Database>>,
    /// Last successfully used font family
    last_working_font: Option<String>,
    /// Available font families (for debugging)
    available_fonts: Vec<String>,
    /// Font mapping from app settings to actual font names
    font_mapping: std::collections::HashMap<String, String>,
    /// Whether the cache has been initialized
    initialized: bool,
}

impl Default for FontCache {
    fn default() -> Self {
        Self {
            fontdb: None,
            last_working_font: None,
            available_fonts: Vec::new(),
            font_mapping: std::collections::HashMap::new(),
            initialized: false,
        }
    }
}

/// Global font cache instance
static FONT_CACHE: Lazy<Mutex<FontCache>> = Lazy::new(|| Mutex::new(FontCache::default()));

/// Initialize or get cached fontdb with optimized font selection
fn get_or_create_fontdb() -> (Arc<fontdb::Database>, Option<String>) {
    let mut cache = FONT_CACHE.lock().unwrap();

    if cache.initialized && cache.fontdb.is_some() {
        // Return cached fontdb and last working font
        log::debug!("Using cached fontdb with {} fonts", cache.fontdb.as_ref().unwrap().len());
        return (cache.fontdb.as_ref().unwrap().clone(), cache.last_working_font.clone());
    }

    // Create new fontdb and load all fonts
    let mut fontdb = fontdb::Database::new();
    fontdb.load_system_fonts();

    // Load embedded fonts that match egui's font configuration
    let source_han_font_data = include_bytes!("../../../assets/fonts/SourceHanSansSC-Regular.otf");
    fontdb.load_font_data(source_han_font_data.to_vec());

    let wqy_font_data = include_bytes!("../../../assets/fonts/wqy-microhei.ttc");
    fontdb.load_font_data(wqy_font_data.to_vec());

    log::info!("Loaded {} fonts into fontdb for Mermaid rendering", fontdb.len());

    // Collect available font families for debugging
    let mut font_families = std::collections::HashSet::new();
    for face in fontdb.faces() {
        for family in &face.families {
            font_families.insert(family.0.clone());
        }
    }
    let mut families: Vec<_> = font_families.into_iter().collect();
    families.sort();

    // Log first 20 font families for debugging
    let sample_families: Vec<_> = families.iter().take(20).cloned().collect();
    log::debug!("Sample font families available: {:?}", sample_families);

    // Check for specific fonts that might work better
    let chinese_fonts: Vec<_> = families.iter().filter(|f|
        f.contains("PingFang") ||
        f.contains("Hiragino") ||
        f.contains("Source Han") ||
        f.contains("Noto") ||
        f.contains("Arial Unicode") ||
        f.contains("WQY") ||
        f.contains("Microsoft YaHei") ||
        f.contains("SimHei")
    ).cloned().collect();
    log::info!("Available Chinese/Unicode fonts: {:?}", chinese_fonts);

    // Build font mapping for app settings with exact name matching
    let mut font_mapping = std::collections::HashMap::new();

    // Find exact font names for mapping
    for family in &families {
        if family.contains("Source Han Sans") {
            font_mapping.insert("Source Han Sans".to_string(), family.clone());
            log::debug!("Mapped 'Source Han Sans' to '{}'", family);
        }
        if family.contains("WQY") || family.contains("wqy") {
            font_mapping.insert("WQY MicroHei".to_string(), family.clone());
            log::debug!("Mapped 'WQY MicroHei' to '{}'", family);
        }
        if family.contains("PingFang") {
            font_mapping.insert("Default".to_string(), family.clone());
            log::debug!("Mapped 'Default' to '{}'", family);
        }
    }

    // Find the best default font with validation
    let target_fonts = [
        "Arial Unicode MS",      // Unicode 字体 - 最高优先级，支持最广泛
        "Arial",                 // 基础字体
        "Helvetica",             // macOS 基础字体
        "PingFang SC",           // macOS 系统中文字体
        "Hiragino Sans GB",      // macOS 中文字体
    ];
    let mut working_font = None;

    // Create a temporary fontdb reference for validation
    let fontdb_ref = Arc::new(fontdb);

    // First try exact matches with validation
    for target in &target_fonts {
        if let Some(exact_font) = families.iter().find(|f| f.eq_ignore_ascii_case(target)) {
            if validate_font_rendering(&fontdb_ref, exact_font) {
                working_font = Some(exact_font.clone());
                log::info!("Selected '{}' as primary font for Mermaid rendering (exact match, validated)", exact_font);
                break;
            } else {
                log::debug!("Font '{}' found but failed validation", exact_font);
            }
        }
    }

    // If no exact match, try partial matches with validation
    if working_font.is_none() {
        for target in &target_fonts {
            if let Some(partial_font) = families.iter().find(|f| f.contains(target)) {
                if validate_font_rendering(&fontdb_ref, partial_font) {
                    working_font = Some(partial_font.clone());
                    log::info!("Selected '{}' as primary font for Mermaid rendering (partial match, validated)", partial_font);
                    break;
                } else {
                    log::debug!("Font '{}' found but failed validation", partial_font);
                }
            }
        }
    }

    // Final fallback to any available font that validates
    if working_font.is_none() && !families.is_empty() {
        // Try common fonts that are likely to work
        let fallback_candidates = [
            "Arial", "Helvetica", "Times", "Courier", "Geneva", "Monaco"
        ];

        for candidate in &fallback_candidates {
            if let Some(font) = families.iter().find(|f| f.contains(candidate)) {
                if validate_font_rendering(&fontdb_ref, font) {
                    working_font = Some(font.clone());
                    log::info!("Selected '{}' as fallback font for Mermaid rendering (validated)", font);
                    break;
                }
            }
        }

        // Last resort - use first available font without validation
        if working_font.is_none() {
            working_font = Some(families[0].clone());
            log::warn!("Selected '{}' as last resort font for Mermaid rendering (no validation)", families[0]);
        }
    }

    // Use the validated fontdb
    let fontdb = fontdb_ref;

    // Cache the results
    cache.fontdb = Some(fontdb.clone());
    cache.last_working_font = working_font.clone();
    cache.available_fonts = families;
    cache.font_mapping = font_mapping;
    cache.initialized = true;

    log::debug!("Font cache initialized with {} available fonts", cache.available_fonts.len());

    (fontdb, working_font)
}

/// Update the last working font in cache
fn update_last_working_font(font_family: String) {
    if let Ok(mut cache) = FONT_CACHE.lock() {
        cache.last_working_font = Some(font_family.clone());
        log::debug!("Updated last working font to: {}", font_family);
    }
}

/// Preload font cache to avoid delays during first render
pub fn preload_font_cache() {
    log::debug!("Preloading Mermaid font cache...");
    let start_time = std::time::Instant::now();

    // This will initialize the font cache if not already done
    let (fontdb, working_font) = get_or_create_fontdb();

    // Validate the selected font by trying to render a simple test
    if let Some(ref font_name) = working_font {
        if validate_font_rendering(&fontdb, font_name) {
            log::info!("Font '{}' validated successfully for Mermaid rendering", font_name);
        } else {
            log::warn!("Font '{}' failed validation, may cause rendering issues", font_name);
        }
    }

    let elapsed = start_time.elapsed();
    log::info!("Font cache preloaded in {:?}, working font: {:?}", elapsed, working_font);
}

/// Validate that a font can be used for text rendering
fn validate_font_rendering(fontdb: &fontdb::Database, font_name: &str) -> bool {
    // Try to find the font in the database
    let query = fontdb::Query {
        families: &[fontdb::Family::Name(font_name)],
        weight: fontdb::Weight::NORMAL,
        stretch: fontdb::Stretch::Normal,
        style: fontdb::Style::Normal,
    };

    if let Some(id) = fontdb.query(&query) {
        if let Some(_face) = fontdb.face(id) {
            log::debug!("Font '{}' found and accessible in fontdb", font_name);
            return true;
        }
    }

    log::debug!("Font '{}' not found or not accessible in fontdb", font_name);
    false
}

/// Simple SVG test renderer for debugging text rendering issues
pub struct SvgTestRenderer {
    test_cache: HashMap<String, Arc<TextureHandle>>,
}

impl Default for SvgTestRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl SvgTestRenderer {
    pub fn new() -> Self {
        Self {
            test_cache: HashMap::new(),
        }
    }

    /// Render a simple test SVG with text and shapes
    pub fn render_test_svg(&mut self, ui: &mut Ui, font_family: Option<&str>) {
        let cache_key = format!("test_svg_{}", font_family.unwrap_or("default"));

        // Check cache first
        if let Some(texture) = self.test_cache.get(&cache_key) {
            self.render_cached_texture(ui, texture.clone());
            return;
        }

        // Generate test SVG
        if let Ok(svg_data) = self.generate_test_svg(font_family) {
            match self.svg_to_texture(ui.ctx(), &svg_data) {
                Ok(texture) => {
                    let texture_arc = Arc::new(texture);
                    self.test_cache.insert(cache_key, texture_arc.clone());

                    // Update font cache with successful font
                    let font_family_str = self.get_svg_font_family(font_family);
                    if let Some(first_font) = font_family_str.split(',').next() {
                        update_last_working_font(first_font.trim().to_string());
                    }

                    self.render_cached_texture(ui, texture_arc);
                    return;
                }
                Err(e) => {
                    log::error!("Failed to convert test SVG to texture: {}", e);
                }
            }
        }

        // Fallback to text display
        ui.label("SVG测试渲染失败");
    }

    /// Generate a simple test SVG with various text and shapes
    fn generate_test_svg(&self, font_family: Option<&str>) -> Result<String, String> {
        // Get font family once and reuse it
        let font_family_str = self.get_svg_font_family(font_family);
        let font_ref = &font_family_str; // Create a reference to avoid repeated cloning

        let svg = format!(r##"<svg width="400" height="300" xmlns="http://www.w3.org/2000/svg">
            <!-- Background -->
            <rect x="0" y="0" width="400" height="300" fill="#f8f9fa" stroke="#dee2e6" stroke-width="1"/>

            <!-- Title -->
            <text x="200" y="30" text-anchor="middle" font-family="{font}" font-size="18" font-weight="bold" fill="#212529">SVG文字渲染测试</text>

            <!-- Test shapes with labels -->
            <rect x="50" y="60" width="80" height="40" fill="#e3f2fd" stroke="#1976d2" stroke-width="2"/>
            <text x="90" y="85" text-anchor="middle" font-family="{font}" font-size="14" font-weight="bold" fill="#1976d2">矩形</text>

            <circle cx="250" cy="80" r="25" fill="#e8f5e8" stroke="#4caf50" stroke-width="2"/>
            <text x="250" y="85" text-anchor="middle" font-family="{font}" font-size="14" font-weight="bold" fill="#2e7d32">圆形</text>

            <polygon points="320,60 360,80 320,100 280,80" fill="#fff3e0" stroke="#ff9800" stroke-width="2"/>
            <text x="320" y="85" text-anchor="middle" font-family="{font}" font-size="14" font-weight="bold" fill="#e65100">菱形</text>

            <!-- Text samples -->
            <text x="50" y="140" font-family="{font}" font-size="12" fill="#333">中文字体测试：你好世界</text>
            <text x="50" y="160" font-family="{font}" font-size="12" fill="#333">English Font Test: Hello World</text>
            <text x="50" y="180" font-family="{font}" font-size="12" fill="#333">数字测试：1234567890</text>
            <text x="50" y="200" font-family="{font}" font-size="12" fill="#333">符号测试：!@#$%^&*()</text>

            <!-- Font info -->
            <text x="50" y="230" font-family="{font}" font-size="10" fill="#666">当前字体：{font_name}</text>
            <text x="50" y="250" font-family="{font}" font-size="10" fill="#666">渲染引擎：usvg + resvg + tiny-skia</text>

            <!-- Connection lines -->
            <defs>
                <marker id="arrowhead" markerWidth="10" markerHeight="7" refX="9" refY="3.5" orient="auto">
                    <polygon points="0 0, 10 3.5, 0 7" fill="#666"/>
                </marker>
            </defs>
            <line x1="130" y1="80" x2="220" y2="80" stroke="#666" stroke-width="2" marker-end="url(#arrowhead)"/>
            <line x1="275" y1="80" x2="280" y2="80" stroke="#666" stroke-width="2" marker-end="url(#arrowhead)"/>
        </svg>"##, font = font_ref, font_name = font_ref);

        Ok(svg)
    }

    /// Get the appropriate font family for SVG rendering with optimized font selection
    fn get_svg_font_family(&self, _font_family: Option<&str>) -> String {
        // Always use Arial Unicode MS as the primary font to avoid fallback warnings
        // This font has the best Unicode support and is what usvg ultimately falls back to anyway
        "Arial Unicode MS, Arial, Helvetica, sans-serif".to_string()
    }



    /// Convert SVG to texture using the same method as MermaidRenderer
    fn svg_to_texture(&self, ctx: &egui::Context, svg_data: &str) -> Result<TextureHandle, String> {
        // Parse SVG with usvg
        let svg_options = usvg::Options::default();
        let usvg_tree = usvg::Tree::from_str(svg_data, &svg_options)
            .map_err(|e| format!("Failed to parse SVG: {}", e))?;

        // Use cached fontdb
        let (fontdb, _working_font) = get_or_create_fontdb();

        // Convert text to paths
        let mut usvg_tree = usvg_tree;
        usvg::TreeTextToPath::convert_text(&mut usvg_tree, &*fontdb);

        // Create resvg tree
        let resvg_tree = resvg::Tree::from_usvg(&usvg_tree);

        let size = resvg_tree.size;
        let width = size.width() as u32;
        let height = size.height() as u32;

        // Create pixmap
        let mut pixmap = tiny_skia::Pixmap::new(width, height)
            .ok_or("Failed to create pixmap")?;

        // Render SVG to pixmap
        resvg_tree.render(tiny_skia::Transform::default(), &mut pixmap.as_mut());

        // Convert to egui ColorImage
        let pixels = pixmap.data();
        let mut rgba_pixels = Vec::with_capacity(pixels.len());

        // Convert BGRA to RGBA
        for chunk in pixels.chunks_exact(4) {
            rgba_pixels.push(chunk[2]); // R
            rgba_pixels.push(chunk[1]); // G
            rgba_pixels.push(chunk[0]); // B
            rgba_pixels.push(chunk[3]); // A
        }

        let color_image = egui::ColorImage::from_rgba_unmultiplied([width as usize, height as usize], &rgba_pixels);

        // Create texture
        let texture = ctx.load_texture("svg_test", color_image, egui::TextureOptions::default());

        Ok(texture)
    }

    /// Render cached texture
    fn render_cached_texture(&self, ui: &mut Ui, texture: Arc<TextureHandle>) {
        let size = texture.size_vec2();
        let max_width = ui.available_width();
        let max_height = 400.0;

        // Calculate scaled size to fit within bounds
        let scale = (max_width / size.x).min(max_height / size.y).min(1.0);
        let scaled_size = size * scale;

        ui.add_space(8.0);

        // Center the image
        ui.allocate_ui_with_layout(
            egui::Vec2::new(max_width, scaled_size.y),
            egui::Layout::top_down(egui::Align::Center),
            |ui| {
                ui.add(egui::Image::from_texture(texture.as_ref()).fit_to_exact_size(scaled_size));
            }
        );

        ui.add_space(8.0);
    }

    /// Clear the test cache
    pub fn clear_cache(&mut self) {
        self.test_cache.clear();
    }
}

/// Mermaid diagram types
#[derive(Debug, Clone, PartialEq)]
pub enum MermaidDiagramType {
    Flowchart,
    Sequence,
    ClassDiagram,
    StateDiagram,
    EntityRelationship,
    UserJourney,
    Gantt,
    PieChart,
    GitGraph,
    Unknown,
}

impl MermaidDiagramType {
    /// Detect diagram type from mermaid code
    pub fn from_code(code: &str) -> Self {
        let trimmed = code.trim().to_lowercase();

        if trimmed.starts_with("graph") || trimmed.starts_with("flowchart") {
            Self::Flowchart
        } else if trimmed.starts_with("sequencediagram") || trimmed.contains("participant") {
            Self::Sequence
        } else if trimmed.starts_with("classdiagram") {
            Self::ClassDiagram
        } else if trimmed.starts_with("statediagram") {
            Self::StateDiagram
        } else if trimmed.starts_with("erdiagram") {
            Self::EntityRelationship
        } else if trimmed.starts_with("journey") {
            Self::UserJourney
        } else if trimmed.starts_with("gantt") {
            Self::Gantt
        } else if trimmed.starts_with("pie") {
            Self::PieChart
        } else if trimmed.starts_with("gitgraph") {
            Self::GitGraph
        } else {
            Self::Unknown
        }
    }
}

/// Mermaid diagram renderer
pub struct MermaidRenderer {
    /// Cache for rendered diagrams
    diagram_cache: HashMap<String, Arc<TextureHandle>>,
    /// SVG renderer options
    svg_options: usvg::Options,
    /// Pending requests to avoid duplicate fetches
    pending_requests: HashMap<String, bool>,
}

impl Default for MermaidRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl MermaidRenderer {
    /// Create a new Mermaid renderer
    pub fn new() -> Self {
        let svg_options = usvg::Options::default();

        Self {
            diagram_cache: HashMap::new(),
            svg_options,
            pending_requests: HashMap::new(),
        }
    }

    /// Get the appropriate font family for SVG rendering with optimized font selection
    fn get_svg_font_family(&self, _font_family: Option<&str>) -> String {
        // Always use Arial Unicode MS as the primary font to avoid fallback warnings
        // This font has the best Unicode support and is what usvg ultimately falls back to anyway
        "Arial Unicode MS, Arial, Helvetica, sans-serif".to_string()
    }



    /// Render a mermaid diagram to egui
    pub fn render_diagram(&mut self, ui: &mut Ui, mermaid_code: &str, font_family: Option<&str>) {
        let diagram_type = MermaidDiagramType::from_code(mermaid_code);

        // Create a cache key from the code
        let cache_key = format!("{:x}", md5::compute(mermaid_code.as_bytes()));

        // Check local cache
        if let Some(texture) = self.diagram_cache.get(&cache_key) {
            self.render_cached_diagram(ui, texture.clone());
            return;
        }

        // Try to render a simple SVG placeholder
        if let Ok(svg_data) = self.generate_simple_svg(&diagram_type, mermaid_code) {
            match self.svg_to_texture(ui.ctx(), &svg_data) {
                Ok(texture) => {
                    let texture_arc = Arc::new(texture);
                    self.diagram_cache.insert(cache_key, texture_arc.clone());

                    // Update font cache with successful font
                    let font_family_str = self.get_svg_font_family(font_family);
                    if let Some(first_font) = font_family_str.split(',').next() {
                        update_last_working_font(first_font.trim().to_string());
                    }

                    self.render_cached_diagram(ui, texture_arc);
                    return;
                }
                Err(e) => {
                    log::error!("Failed to convert SVG to texture: {}", e);
                }
            }
        }

        // Fallback to enhanced placeholder
        self.render_enhanced_placeholder(ui, mermaid_code, &diagram_type);
    }

    /// Render cached diagram
    fn render_cached_diagram(&self, ui: &mut Ui, texture: Arc<TextureHandle>) {
        let size = texture.size_vec2();
        let max_width = ui.available_width();
        let max_height = 400.0; // Maximum height for diagrams
        
        // Calculate scaled size to fit within bounds
        let scale = (max_width / size.x).min(max_height / size.y).min(1.0);
        let scaled_size = size * scale;
        
        ui.add_space(8.0);
        
        // Center the diagram
        ui.allocate_ui_with_layout(
            egui::Vec2::new(max_width, scaled_size.y),
            egui::Layout::top_down(egui::Align::Center),
            |ui| {
                ui.add(egui::Image::from_texture(texture.as_ref()).fit_to_exact_size(scaled_size));
            }
        );
        
        ui.add_space(8.0);
    }

    /// Generate a simple SVG based on diagram type and content
    fn generate_simple_svg(&self, diagram_type: &MermaidDiagramType, mermaid_code: &str) -> Result<String, String> {
        match diagram_type {
            MermaidDiagramType::Flowchart => self.generate_flowchart_svg(mermaid_code),
            MermaidDiagramType::Sequence => self.generate_sequence_svg(mermaid_code),
            MermaidDiagramType::ClassDiagram => self.generate_class_svg(mermaid_code),
            MermaidDiagramType::StateDiagram => self.generate_state_svg(mermaid_code),
            MermaidDiagramType::Gantt => self.generate_gantt_svg(mermaid_code),
            MermaidDiagramType::PieChart => self.generate_pie_svg(mermaid_code),
            MermaidDiagramType::EntityRelationship => self.generate_er_svg(mermaid_code),
            MermaidDiagramType::GitGraph => self.generate_gitgraph_svg(mermaid_code),
            MermaidDiagramType::UserJourney => self.generate_userjourney_svg(mermaid_code),
            _ => self.generate_generic_svg(diagram_type, mermaid_code),
        }
    }

    /// Convert SVG to texture
    fn svg_to_texture(&self, ctx: &egui::Context, svg_data: &str) -> Result<TextureHandle, String> {
        // Parse SVG with usvg
        let svg_options = usvg::Options::default();
        let usvg_tree = usvg::Tree::from_str(svg_data, &svg_options)
            .map_err(|e| format!("Failed to parse SVG: {}", e))?;

        // Use cached fontdb
        let (fontdb, _working_font) = get_or_create_fontdb();

        // Convert text to paths
        let mut usvg_tree = usvg_tree;
        usvg::TreeTextToPath::convert_text(&mut usvg_tree, &*fontdb);

        // Create resvg tree
        let resvg_tree = resvg::Tree::from_usvg(&usvg_tree);

        let size = resvg_tree.size;
        let width = size.width() as u32;
        let height = size.height() as u32;

        // Create pixmap
        let mut pixmap = tiny_skia::Pixmap::new(width, height)
            .ok_or("Failed to create pixmap")?;

        // Render SVG to pixmap
        resvg_tree.render(tiny_skia::Transform::default(), &mut pixmap.as_mut());

        // Convert to egui ColorImage
        let pixels = pixmap.data();
        let mut rgba_pixels = Vec::with_capacity(pixels.len());

        // Convert BGRA to RGBA
        for chunk in pixels.chunks_exact(4) {
            rgba_pixels.push(chunk[2]); // R
            rgba_pixels.push(chunk[1]); // G
            rgba_pixels.push(chunk[0]); // B
            rgba_pixels.push(chunk[3]); // A
        }

        let color_image = egui::ColorImage::from_rgba_unmultiplied([width as usize, height as usize], &rgba_pixels);

        // Create texture
        let texture = ctx.load_texture("mermaid_diagram", color_image, egui::TextureOptions::default());

        Ok(texture)
    }

    /// Generate a simple flowchart SVG
    fn generate_flowchart_svg(&self, mermaid_code: &str) -> Result<String, String> {
        // Parse the flowchart nodes and connections
        let nodes = self.parse_flowchart_nodes(mermaid_code);
        let connections = self.parse_flowchart_connections(mermaid_code);

        if nodes.is_empty() {
            // Fallback to default flowchart
            return self.generate_default_flowchart();
        }

        // Generate SVG based on parsed content
        self.generate_flowchart_svg_from_nodes(&nodes, &connections)
    }

    /// Generate a simple sequence diagram SVG
    fn generate_sequence_svg(&self, mermaid_code: &str) -> Result<String, String> {
        // Parse participants and messages
        let participants = self.parse_sequence_participants(mermaid_code);
        let messages = self.parse_sequence_messages(mermaid_code);

        if participants.is_empty() {
            // Fallback to default sequence diagram
            return self.generate_default_sequence();
        }

        // Generate SVG based on parsed content
        self.generate_sequence_svg_from_data(&participants, &messages)
    }

    /// Generate a simple class diagram SVG
    fn generate_class_svg(&self, mermaid_code: &str) -> Result<String, String> {
        // Parse class definitions
        let classes = self.parse_class_definitions(mermaid_code);

        if classes.is_empty() {
            return self.generate_default_class();
        }

        self.generate_class_svg_from_data(&classes)
    }

    /// Generate default class diagram
    fn generate_default_class(&self) -> Result<String, String> {
        let font_family_str = self.get_svg_font_family(None);
        let svg = format!(r##"<svg width="300" height="200" xmlns="http://www.w3.org/2000/svg">
            <rect x="50" y="30" width="120" height="80" fill="#f5f5f5" stroke="#333" stroke-width="2"/>
            <line x1="50" y1="55" x2="170" y2="55" stroke="#333" stroke-width="1"/>
            <line x1="50" y1="85" x2="170" y2="85" stroke="#333" stroke-width="1"/>
            <text x="110" y="50" text-anchor="middle" font-family="{}" font-size="12" font-weight="bold" fill="#333">类名</text>
            <text x="60" y="75" font-family="{}" font-size="10" fill="#666">+ 属性1: String</text>
            <text x="60" y="100" font-family="{}" font-size="10" fill="#666">+ 方法1(): void</text>
        </svg>"##, font_family_str, font_family_str, font_family_str);
        Ok(svg)
    }

    /// Generate a simple state diagram SVG
    fn generate_state_svg(&self, mermaid_code: &str) -> Result<String, String> {
        // Parse state definitions
        let states = self.parse_state_definitions(mermaid_code);

        if states.is_empty() {
            return self.generate_default_state();
        }

        self.generate_state_svg_from_data(&states)
    }

    /// Generate default state diagram
    fn generate_default_state(&self) -> Result<String, String> {
        let font_family_str = self.get_svg_font_family(None);
        let svg = format!(r##"<svg width="350" height="200" xmlns="http://www.w3.org/2000/svg">
            <defs>
                <marker id="arrowhead" markerWidth="10" markerHeight="7" refX="9" refY="3.5" orient="auto">
                    <polygon points="0 0, 10 3.5, 0 7" fill="#666"/>
                </marker>
            </defs>
            <circle cx="50" cy="100" r="8" fill="#333"/>
            <rect x="120" y="80" width="80" height="40" rx="20" fill="#e8f5e8" stroke="#4caf50" stroke-width="2"/>
            <text x="160" y="105" text-anchor="middle" font-family="{}" font-size="12" font-weight="500" fill="#2e7d32">状态1</text>
            <rect x="250" y="80" width="80" height="40" rx="20" fill="#fff3e0" stroke="#ff9800" stroke-width="2"/>
            <text x="290" y="105" text-anchor="middle" font-family="{}" font-size="12" font-weight="500" fill="#e65100">状态2</text>
            <line x1="58" y1="100" x2="120" y2="100" stroke="#666" stroke-width="2" marker-end="url(#arrowhead)"/>
            <line x1="200" y1="100" x2="250" y2="100" stroke="#666" stroke-width="2" marker-end="url(#arrowhead)"/>
        </svg>"##, font_family_str, font_family_str);
        Ok(svg)
    }

    /// Generate a simple gantt chart SVG
    fn generate_gantt_svg(&self, mermaid_code: &str) -> Result<String, String> {
        // Parse gantt tasks
        let tasks = self.parse_gantt_tasks(mermaid_code);

        if tasks.is_empty() {
            return self.generate_default_gantt();
        }

        self.generate_gantt_svg_from_data(&tasks)
    }

    /// Generate default gantt chart
    fn generate_default_gantt(&self) -> Result<String, String> {
        let font_family_str = self.get_svg_font_family(None);
        let svg = format!(r##"<svg width="400" height="200" xmlns="http://www.w3.org/2000/svg">
            <rect x="50" y="30" width="300" height="20" fill="#f5f5f5" stroke="#ddd" stroke-width="1"/>
            <text x="10" y="45" font-family="{}" font-size="11" fill="#333">任务1</text>
            <rect x="50" y="30" width="100" height="20" fill="#2196f3"/>

            <rect x="50" y="60" width="300" height="20" fill="#f5f5f5" stroke="#ddd" stroke-width="1"/>
            <text x="10" y="75" font-family="{}" font-size="11" fill="#333">任务2</text>
            <rect x="120" y="60" width="120" height="20" fill="#4caf50"/>

            <rect x="50" y="90" width="300" height="20" fill="#f5f5f5" stroke="#ddd" stroke-width="1"/>
            <text x="10" y="105" font-family="{}" font-size="11" fill="#333">任务3</text>
            <rect x="200" y="90" width="80" height="20" fill="#ff9800"/>

            <!-- Time axis -->
            <line x1="50" y1="130" x2="350" y2="130" stroke="#666" stroke-width="1"/>
            <text x="100" y="145" text-anchor="middle" font-family="{}" font-size="10" fill="#666">周1</text>
            <text x="200" y="145" text-anchor="middle" font-family="{}" font-size="10" fill="#666">周2</text>
            <text x="300" y="145" text-anchor="middle" font-family="{}" font-size="10" fill="#666">周3</text>
        </svg>"##, font_family_str, font_family_str, font_family_str, font_family_str, font_family_str, font_family_str);
        Ok(svg)
    }

    /// Generate a simple ER diagram SVG
    fn generate_er_svg(&self, mermaid_code: &str) -> Result<String, String> {
        // Parse ER entities and relationships
        let entities = self.parse_er_entities(mermaid_code);

        if entities.is_empty() {
            return self.generate_default_er();
        }

        self.generate_er_svg_from_data(&entities)
    }

    /// Generate default ER diagram
    fn generate_default_er(&self) -> Result<String, String> {
        let font_family_str = self.get_svg_font_family(None);
        let svg = format!(r##"<svg width="400" height="250" xmlns="http://www.w3.org/2000/svg">
            <defs>
                <marker id="arrowhead" markerWidth="10" markerHeight="7" refX="9" refY="3.5" orient="auto">
                    <polygon points="0 0, 10 3.5, 0 7" fill="#666"/>
                </marker>
            </defs>
            <!-- Entity 1 -->
            <rect x="50" y="50" width="100" height="60" fill="#e3f2fd" stroke="#1976d2" stroke-width="2"/>
            <text x="100" y="75" text-anchor="middle" font-family="{}" font-size="12" font-weight="bold" fill="#1976d2">用户</text>
            <text x="60" y="90" font-family="{}" font-size="10" fill="#666">id: int</text>
            <text x="60" y="105" font-family="{}" font-size="10" fill="#666">name: string</text>

            <!-- Entity 2 -->
            <rect x="250" y="50" width="100" height="60" fill="#e8f5e8" stroke="#4caf50" stroke-width="2"/>
            <text x="300" y="75" text-anchor="middle" font-family="{}" font-size="12" font-weight="bold" fill="#2e7d32">订单</text>
            <text x="260" y="90" font-family="{}" font-size="10" fill="#666">id: int</text>
            <text x="260" y="105" font-family="{}" font-size="10" fill="#666">amount: decimal</text>

            <!-- Relationship -->
            <polygon points="200,70 220,80 200,90 180,80" fill="#fff3e0" stroke="#ff9800" stroke-width="2"/>
            <text x="200" y="85" text-anchor="middle" font-family="{}" font-size="10" fill="#e65100">拥有</text>

            <!-- Lines -->
            <line x1="150" y1="80" x2="180" y2="80" stroke="#666" stroke-width="2"/>
            <line x1="220" y1="80" x2="250" y2="80" stroke="#666" stroke-width="2" marker-end="url(#arrowhead)"/>
        </svg>"##, font_family_str, font_family_str, font_family_str, font_family_str, font_family_str, font_family_str, font_family_str);
        Ok(svg)
    }

    /// Generate a simple pie chart SVG
    fn generate_pie_svg(&self, mermaid_code: &str) -> Result<String, String> {
        // Parse pie chart data
        let data = self.parse_pie_data(mermaid_code);

        if data.is_empty() {
            return self.generate_default_pie();
        }

        self.generate_pie_svg_from_data(&data)
    }

    /// Generate default pie chart
    fn generate_default_pie(&self) -> Result<String, String> {
        let font_family_str = self.get_svg_font_family(None);
        let svg = format!(r##"<svg width="300" height="200" xmlns="http://www.w3.org/2000/svg">
            <circle cx="150" cy="100" r="60" fill="#e3f2fd" stroke="#1976d2" stroke-width="2"/>
            <path d="M 150 100 L 150 40 A 60 60 0 0 1 210 100 Z" fill="#2196f3"/>
            <path d="M 150 100 L 210 100 A 60 60 0 0 1 150 160 Z" fill="#4caf50"/>
            <path d="M 150 100 L 150 160 A 60 60 0 0 1 90 100 Z" fill="#ff9800"/>
            <path d="M 150 100 L 90 100 A 60 60 0 0 1 150 40 Z" fill="#f44336"/>
            <text x="180" y="70" font-family="{}" font-size="11" font-weight="500" fill="#1976d2">25%</text>
            <text x="180" y="130" font-family="{}" font-size="11" font-weight="500" fill="#388e3c">25%</text>
            <text x="120" y="130" font-family="{}" font-size="11" font-weight="500" fill="#f57c00">25%</text>
            <text x="120" y="70" font-family="{}" font-size="11" font-weight="500" fill="#d32f2f">25%</text>
        </svg>"##, font_family_str, font_family_str, font_family_str, font_family_str);
        Ok(svg)
    }

    /// Generate a simple Git graph SVG
    fn generate_gitgraph_svg(&self, mermaid_code: &str) -> Result<String, String> {
        // Parse git commits and branches
        let commits = self.parse_git_commits(mermaid_code);

        if commits.is_empty() {
            return self.generate_default_gitgraph();
        }

        // Generate SVG based on parsed content
        self.generate_gitgraph_svg_from_data(&commits)
    }

    /// Generate a simple user journey SVG
    fn generate_userjourney_svg(&self, mermaid_code: &str) -> Result<String, String> {
        // Parse journey steps and actors
        let journey_data = self.parse_user_journey(mermaid_code);

        if journey_data.is_empty() {
            return self.generate_default_userjourney();
        }

        // Generate SVG based on parsed content
        self.generate_userjourney_svg_from_data(&journey_data)
    }

    /// Generate a generic diagram SVG
    fn generate_generic_svg(&self, diagram_type: &MermaidDiagramType, _mermaid_code: &str) -> Result<String, String> {
        let (icon, color, bg_color) = match diagram_type {
            MermaidDiagramType::StateDiagram => ("🔄", "#9c27b0", "#f3e5f5"),
            MermaidDiagramType::Gantt => ("📅", "#795548", "#efebe9"),
            MermaidDiagramType::GitGraph => ("🌳", "#607d8b", "#eceff1"),
            MermaidDiagramType::UserJourney => ("👤", "#e91e63", "#fce4ec"),
            MermaidDiagramType::EntityRelationship => ("🗃️", "#3f51b5", "#e8eaf6"),
            _ => ("📊", "#2196f3", "#e3f2fd"),
        };

        let font_family_str = self.get_svg_font_family(None);
        let svg = format!(r##"<svg width="300" height="200" xmlns="http://www.w3.org/2000/svg">
            <rect x="50" y="50" width="200" height="100" rx="10" fill="{}" stroke="{}" stroke-width="2"/>
            <text x="150" y="90" text-anchor="middle" font-family="{}" font-size="24" font-weight="bold" fill="{}" stroke="none">{}</text>
            <text x="150" y="115" text-anchor="middle" font-family="{}" font-size="14" font-weight="bold" fill="{}" stroke="none">Mermaid 图表</text>
        </svg>"##, bg_color, color, font_family_str, color, icon, font_family_str, color);
        Ok(svg)
    }

    /// Parse flowchart nodes from Mermaid code
    fn parse_flowchart_nodes(&self, mermaid_code: &str) -> Vec<(String, String, String)> {
        let mut nodes = Vec::new();

        for line in mermaid_code.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with("graph") || line.starts_with("flowchart") {
                continue;
            }

            // Parse node definitions like: A[开始] or B{是否登录?}
            if let Some(captures) = self.extract_node_definition(line) {
                nodes.push(captures);
            }
        }

        nodes
    }

    /// Parse flowchart connections from Mermaid code
    fn parse_flowchart_connections(&self, mermaid_code: &str) -> Vec<(String, String, String)> {
        let mut connections = Vec::new();

        for line in mermaid_code.lines() {
            let line = line.trim();
            if line.contains("-->") {
                // Parse connections like: A --> B or B -->|是| C
                if let Some((from, to, label)) = self.extract_connection(line) {
                    connections.push((from, to, label));
                }
            }
        }

        connections
    }

    /// Extract node definition from a line
    fn extract_node_definition(&self, line: &str) -> Option<(String, String, String)> {
        // Handle different node types: A[text], B{text}, C(text), D((text))
        if let Some(pos) = line.find('[') {
            if let Some(end_pos) = line.find(']') {
                let id = line[..pos].trim().to_string();
                let text = line[pos+1..end_pos].trim().to_string();
                return Some((id, text, "rect".to_string()));
            }
        }

        if let Some(pos) = line.find('{') {
            if let Some(end_pos) = line.find('}') {
                let id = line[..pos].trim().to_string();
                let text = line[pos+1..end_pos].trim().to_string();
                return Some((id, text, "diamond".to_string()));
            }
        }

        if let Some(pos) = line.find('(') {
            if let Some(end_pos) = line.rfind(')') {
                let id = line[..pos].trim().to_string();
                let text = line[pos+1..end_pos].trim().to_string();
                let shape = if line[pos..=end_pos].starts_with("((") { "circle" } else { "rounded" };
                return Some((id, text, shape.to_string()));
            }
        }

        None
    }

    /// Extract connection from a line
    fn extract_connection(&self, line: &str) -> Option<(String, String, String)> {
        if let Some(arrow_pos) = line.find("-->") {
            let before_arrow = line[..arrow_pos].trim();
            let after_arrow = line[arrow_pos+3..].trim();

            // Handle labeled connections like: B -->|是| C
            let (from, label) = if let Some(pipe_pos) = before_arrow.rfind('|') {
                let from_part = before_arrow[..pipe_pos].trim();
                let label_part = before_arrow[pipe_pos+1..].trim();
                (from_part.to_string(), label_part.to_string())
            } else {
                (before_arrow.to_string(), String::new())
            };

            let to = if let Some(pipe_pos) = after_arrow.find('|') {
                after_arrow[pipe_pos+1..].trim().to_string()
            } else {
                after_arrow.to_string()
            };

            return Some((from, to, label));
        }

        None
    }

    /// Generate default flowchart when parsing fails
    fn generate_default_flowchart(&self) -> Result<String, String> {
        let font_family_str = self.get_svg_font_family(None);
        let svg = format!(r##"<svg width="300" height="200" xmlns="http://www.w3.org/2000/svg">
            <rect x="50" y="30" width="80" height="40" rx="5" fill="#e1f5fe" stroke="#0277bd" stroke-width="2"/>
            <text x="90" y="55" text-anchor="middle" font-family="{}" font-size="14" font-weight="bold" fill="#0277bd" stroke="none">开始</text>
            <rect x="50" y="120" width="80" height="40" rx="5" fill="#f3e5f5" stroke="#7b1fa2" stroke-width="2"/>
            <text x="90" y="145" text-anchor="middle" font-family="{}" font-size="14" font-weight="bold" fill="#7b1fa2" stroke="none">结束</text>
            <line x1="90" y1="70" x2="90" y2="120" stroke="#666" stroke-width="2" marker-end="url(#arrowhead)"/>
            <defs>
                <marker id="arrowhead" markerWidth="10" markerHeight="7" refX="9" refY="3.5" orient="auto">
                    <polygon points="0 0, 10 3.5, 0 7" fill="#666"/>
                </marker>
            </defs>
        </svg>"##, font_family_str, font_family_str);
        Ok(svg)
    }

    /// Generate flowchart SVG from parsed nodes and connections
    fn generate_flowchart_svg_from_nodes(&self, nodes: &[(String, String, String)], connections: &[(String, String, String)]) -> Result<String, String> {
        let mut svg_content = String::new();
        let mut y_pos = 50;
        let x_center = 150;
        let node_spacing = 80;

        // Start SVG
        svg_content.push_str(r##"<svg width="400" height="300" xmlns="http://www.w3.org/2000/svg">"##);

        // Add arrow marker definition
        svg_content.push_str(r##"
            <defs>
                <marker id="arrowhead" markerWidth="10" markerHeight="7" refX="9" refY="3.5" orient="auto">
                    <polygon points="0 0, 10 3.5, 0 7" fill="#666"/>
                </marker>
            </defs>"##);

        // Draw nodes
        for (i, (id, text, shape)) in nodes.iter().enumerate() {
            let y = y_pos + i * node_spacing;
            let text_y = match shape.as_str() {
                "diamond" => y + 20, // Center of diamond
                "circle" => y + 25,  // Center of circle
                _ => y + 25,         // Center of rectangle
            };

            match shape.as_str() {
                "rect" => {
                    svg_content.push_str(&format!(
                        r##"<rect x="{}" y="{}" width="120" height="40" rx="5" fill="#e1f5fe" stroke="#0277bd" stroke-width="2"/>"##,
                        x_center - 60, y
                    ));
                }
                "diamond" => {
                    svg_content.push_str(&format!(
                        r##"<polygon points="{},{} {},{} {},{} {},{}" fill="#fff3e0" stroke="#ff9800" stroke-width="2"/>"##,
                        x_center, y - 20,
                        x_center + 60, y + 20,
                        x_center, y + 60,
                        x_center - 60, y + 20
                    ));
                }
                "circle" => {
                    svg_content.push_str(&format!(
                        r##"<circle cx="{}" cy="{}" r="30" fill="#e8f5e8" stroke="#4caf50" stroke-width="2"/>"##,
                        x_center, y + 20
                    ));
                }
                _ => {
                    svg_content.push_str(&format!(
                        r##"<rect x="{}" y="{}" width="120" height="40" rx="20" fill="#f3e5f5" stroke="#9c27b0" stroke-width="2"/>"##,
                        x_center - 60, y
                    ));
                }
            }

            // Add text with proper positioning and configured font
            let font_family_str = self.get_svg_font_family(None);
            svg_content.push_str(&format!(
                r##"<text x="{}" y="{}" text-anchor="middle" font-family="{}" font-size="14" font-weight="bold" fill="#333" stroke="none">{}</text>"##,
                x_center, text_y, font_family_str, self.escape_xml(text)
            ));
        }

        // Draw connections
        for (i, _) in nodes.iter().enumerate() {
            if i + 1 < nodes.len() {
                let y1 = y_pos + i * node_spacing + 40;
                let y2 = y_pos + (i + 1) * node_spacing;

                svg_content.push_str(&format!(
                    r##"<line x1="{}" y1="{}" x2="{}" y2="{}" stroke="#666" stroke-width="2" marker-end="url(#arrowhead)"/>"##,
                    x_center, y1, x_center, y2
                ));
            }
        }

        svg_content.push_str("</svg>");
        Ok(svg_content)
    }

    /// Escape XML special characters
    fn escape_xml(&self, text: &str) -> String {
        text.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&#39;")
    }

    /// Parse sequence diagram participants
    fn parse_sequence_participants(&self, mermaid_code: &str) -> Vec<String> {
        let mut participants = Vec::new();

        for line in mermaid_code.lines() {
            let line = line.trim();
            if line.starts_with("participant ") {
                let participant = line[12..].trim().to_string();
                participants.push(participant);
            } else if line.contains("->") || line.contains("->>") {
                // Extract participants from message lines
                if let Some(arrow_pos) = line.find("->") {
                    let from = line[..arrow_pos].trim().to_string();
                    let after_arrow = line[arrow_pos+2..].trim();
                    let to = if let Some(colon_pos) = after_arrow.find(':') {
                        after_arrow[..colon_pos].trim().to_string()
                    } else {
                        after_arrow.to_string()
                    };

                    if !participants.contains(&from) {
                        participants.push(from);
                    }
                    if !participants.contains(&to) {
                        participants.push(to);
                    }
                }
            }
        }

        participants
    }

    /// Parse sequence diagram messages
    fn parse_sequence_messages(&self, mermaid_code: &str) -> Vec<(String, String, String)> {
        let mut messages = Vec::new();

        for line in mermaid_code.lines() {
            let line = line.trim();
            if line.contains("->") || line.contains("->>") {
                if let Some(arrow_pos) = line.find("->") {
                    let from = line[..arrow_pos].trim().to_string();
                    let after_arrow = line[arrow_pos+2..].trim();
                    let (to, message) = if let Some(colon_pos) = after_arrow.find(':') {
                        let to_part = after_arrow[..colon_pos].trim().to_string();
                        let msg_part = after_arrow[colon_pos+1..].trim().to_string();
                        (to_part, msg_part)
                    } else {
                        (after_arrow.to_string(), String::new())
                    };

                    messages.push((from, to, message));
                }
            }
        }

        messages
    }

    /// Generate default sequence diagram
    fn generate_default_sequence(&self) -> Result<String, String> {
        let font_family_str = self.get_svg_font_family(None);
        let svg = format!(r##"<svg width="300" height="200" xmlns="http://www.w3.org/2000/svg">
            <rect x="30" y="20" width="60" height="30" fill="#e8f5e8" stroke="#4caf50" stroke-width="2"/>
            <text x="60" y="40" text-anchor="middle" font-family="{}" font-size="14" font-weight="bold" fill="#2e7d32" stroke="none">用户</text>
            <rect x="210" y="20" width="60" height="30" fill="#fff3e0" stroke="#ff9800" stroke-width="2"/>
            <text x="240" y="40" text-anchor="middle" font-family="{}" font-size="14" font-weight="bold" fill="#e65100" stroke="none">系统</text>
            <line x1="60" y1="50" x2="60" y2="170" stroke="#666" stroke-width="2" stroke-dasharray="5,5"/>
            <line x1="240" y1="50" x2="240" y2="170" stroke="#666" stroke-width="2" stroke-dasharray="5,5"/>
            <line x1="60" y1="80" x2="240" y2="80" stroke="#2196f3" stroke-width="2" marker-end="url(#arrowhead)"/>
            <text x="150" y="75" text-anchor="middle" font-family="{}" font-size="12" font-weight="bold" fill="#1976d2" stroke="none">请求</text>
            <line x1="240" y1="120" x2="60" y2="120" stroke="#4caf50" stroke-width="2" marker-end="url(#arrowhead)"/>
            <text x="150" y="115" text-anchor="middle" font-family="{}" font-size="12" font-weight="bold" fill="#388e3c" stroke="none">响应</text>
            <defs>
                <marker id="arrowhead" markerWidth="10" markerHeight="7" refX="9" refY="3.5" orient="auto">
                    <polygon points="0 0, 10 3.5, 0 7" fill="#666"/>
                </marker>
            </defs>
        </svg>"##, font_family_str, font_family_str, font_family_str, font_family_str);
        Ok(svg)
    }

    /// Generate sequence diagram SVG from parsed data
    fn generate_sequence_svg_from_data(&self, participants: &[String], messages: &[(String, String, String)]) -> Result<String, String> {
        let mut svg_content = String::new();
        let participant_width = 100;
        let participant_spacing = 120;
        let start_x = 50;
        let start_y = 30;
        let message_spacing = 40;

        let total_width = start_x * 2 + participants.len() * participant_spacing;
        let total_height = start_y + 50 + messages.len() * message_spacing + 50;

        // Start SVG
        svg_content.push_str(&format!(
            r##"<svg width="{}" height="{}" xmlns="http://www.w3.org/2000/svg">"##,
            total_width, total_height
        ));

        // Add arrow marker definition
        svg_content.push_str(r##"
            <defs>
                <marker id="arrowhead" markerWidth="10" markerHeight="7" refX="9" refY="3.5" orient="auto">
                    <polygon points="0 0, 10 3.5, 0 7" fill="#666"/>
                </marker>
            </defs>"##);

        // Draw participants
        for (i, participant) in participants.iter().enumerate() {
            let x = start_x + i * participant_spacing;

            // Participant box
            svg_content.push_str(&format!(
                r##"<rect x="{}" y="{}" width="{}" height="30" fill="#e8f5e8" stroke="#4caf50" stroke-width="2"/>"##,
                x - participant_width/2, start_y, participant_width
            ));

            // Participant name
            let font_family_str = self.get_svg_font_family(None);
            svg_content.push_str(&format!(
                r##"<text x="{}" y="{}" text-anchor="middle" font-family="{}" font-size="14" font-weight="bold" fill="#2e7d32" stroke="none">{}</text>"##,
                x, start_y + 20, font_family_str, self.escape_xml(participant)
            ));

            // Lifeline
            let lifeline_end = start_y + 60 + messages.len() * message_spacing;
            svg_content.push_str(&format!(
                r##"<line x1="{}" y1="{}" x2="{}" y2="{}" stroke="#666" stroke-width="2" stroke-dasharray="5,5"/>"##,
                x, start_y + 30, x, lifeline_end
            ));
        }

        // Draw messages
        for (i, (from, to, message)) in messages.iter().enumerate() {
            let y = start_y + 80 + i * message_spacing;

            if let (Some(from_idx), Some(to_idx)) = (
                participants.iter().position(|p| p == from),
                participants.iter().position(|p| p == to)
            ) {
                let from_x = start_x + from_idx * participant_spacing;
                let to_x = start_x + to_idx * participant_spacing;

                // Message arrow
                svg_content.push_str(&format!(
                    r##"<line x1="{}" y1="{}" x2="{}" y2="{}" stroke="#2196f3" stroke-width="2" marker-end="url(#arrowhead)"/>"##,
                    from_x, y, to_x, y
                ));

                // Message text
                if !message.is_empty() {
                    let text_x = (from_x + to_x) / 2;
                    let font_family_str = self.get_svg_font_family(None);
                    svg_content.push_str(&format!(
                        r##"<text x="{}" y="{}" text-anchor="middle" font-family="{}" font-size="12" font-weight="bold" fill="#1976d2" stroke="none">{}</text>"##,
                        text_x, y - 5, font_family_str, self.escape_xml(message)
                    ));
                }
            }
        }

        svg_content.push_str("</svg>");
        Ok(svg_content)
    }

    /// Render loading placeholder while diagram is being fetched
    fn render_loading_placeholder(&self, ui: &mut Ui, mermaid_code: &str, diagram_type: &MermaidDiagramType) {
        ui.add_space(8.0);

        egui::Frame::none()
            .fill(egui::Color32::from_rgb(248, 249, 250))
            .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(220, 220, 220)))
            .rounding(8.0)
            .inner_margin(16.0)
            .show(ui, |ui| {
                ui.vertical_centered(|ui| {
                    // Show loading spinner
                    ui.add(egui::Spinner::new().size(24.0));

                    ui.add_space(8.0);

                    ui.label(
                        egui::RichText::new("🎨 正在渲染Mermaid图表...")
                            .size(14.0)
                            .color(egui::Color32::from_rgb(100, 150, 255))
                    );

                    ui.add_space(4.0);

                    // Show diagram type specific information
                    match diagram_type {
                        MermaidDiagramType::Flowchart => {
                            ui.label(
                                egui::RichText::new("🔄 流程图 - 显示工作流程和决策点")
                                    .size(11.0)
                                    .color(egui::Color32::from_rgb(100, 150, 255))
                            );
                        }
                        MermaidDiagramType::Sequence => {
                            ui.label(
                                egui::RichText::new("📋 时序图 - 显示对象间的交互序列")
                                    .size(11.0)
                                    .color(egui::Color32::from_rgb(100, 150, 255))
                            );
                        }
                        MermaidDiagramType::ClassDiagram => {
                            ui.label(
                                egui::RichText::new("🏗️ 类图 - 显示类的结构和关系")
                                    .size(11.0)
                                    .color(egui::Color32::from_rgb(100, 150, 255))
                            );
                        }
                        MermaidDiagramType::StateDiagram => {
                            ui.label(
                                egui::RichText::new("🔄 状态图 - 显示状态转换")
                                    .size(11.0)
                                    .color(egui::Color32::from_rgb(100, 150, 255))
                            );
                        }
                        MermaidDiagramType::PieChart => {
                            ui.label(
                                egui::RichText::new("🥧 饼图 - 显示数据比例")
                                    .size(11.0)
                                    .color(egui::Color32::from_rgb(100, 150, 255))
                            );
                        }
                        MermaidDiagramType::Gantt => {
                            ui.label(
                                egui::RichText::new("📅 甘特图 - 显示项目时间线")
                                    .size(11.0)
                                    .color(egui::Color32::from_rgb(100, 150, 255))
                            );
                        }
                        MermaidDiagramType::GitGraph => {
                            ui.label(
                                egui::RichText::new("🌳 Git图 - 显示版本控制分支和提交")
                                    .size(11.0)
                                    .color(egui::Color32::from_rgb(100, 150, 255))
                            );
                        }
                        MermaidDiagramType::UserJourney => {
                            ui.label(
                                egui::RichText::new("👤 用户旅程图 - 显示用户体验流程")
                                    .size(11.0)
                                    .color(egui::Color32::from_rgb(100, 150, 255))
                            );
                        }
                        MermaidDiagramType::EntityRelationship => {
                            ui.label(
                                egui::RichText::new("🗃️ 实体关系图 - 显示数据库结构")
                                    .size(11.0)
                                    .color(egui::Color32::from_rgb(100, 150, 255))
                            );
                        }
                        _ => {
                            ui.label(
                                egui::RichText::new(format!("📊 {} - Mermaid图表", format!("{:?}", diagram_type)))
                                    .size(11.0)
                                    .color(egui::Color32::from_rgb(100, 150, 255))
                            );
                        }
                    }

                    ui.add_space(4.0);

                    // Show code preview
                    let preview = if mermaid_code.chars().count() > 50 {
                        let truncated: String = mermaid_code.chars().take(50).collect();
                        format!("{}...", truncated)
                    } else {
                        mermaid_code.to_string()
                    };

                    ui.label(
                        egui::RichText::new(format!("代码: {}", preview))
                            .size(10.0)
                            .color(egui::Color32::from_rgb(120, 120, 120))
                            .italics()
                    );
                });
            });

        ui.add_space(8.0);
    }

    /// Render enhanced placeholder for Mermaid diagrams
    fn render_enhanced_placeholder(&self, ui: &mut Ui, mermaid_code: &str, diagram_type: &MermaidDiagramType) {
        ui.add_space(8.0);

        // Create a frame for the fallback
        let frame = egui::Frame::none()
            .fill(egui::Color32::from_rgb(50, 50, 50))
            .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(100, 100, 100)))
            .rounding(egui::Rounding::same(6.0))
            .inner_margin(egui::Margin::same(12.0));

        frame.show(ui, |ui| {
            ui.vertical(|ui| {
                // Header
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new("📊")
                            .size(16.0)
                            .color(egui::Color32::from_rgb(100, 150, 255))
                    );
                    ui.label(
                        egui::RichText::new("Mermaid Diagram")
                            .size(14.0)
                            .color(egui::Color32::from_rgb(200, 200, 200))
                    );
                });

                ui.add_space(4.0);

                ui.label(
                    egui::RichText::new(format!("Type: {:?}", diagram_type))
                        .size(12.0)
                        .color(egui::Color32::from_rgb(150, 150, 150))
                );

                ui.add_space(8.0);

                // Show a truncated version of the code
                let preview = if mermaid_code.chars().count() > 100 {
                    let truncated: String = mermaid_code.chars().take(100).collect();
                    format!("{}...", truncated)
                } else {
                    mermaid_code.to_string()
                };

                ui.label(
                    egui::RichText::new(preview)
                        .font(egui::FontId::monospace(10.0))
                        .color(egui::Color32::from_rgb(180, 180, 180))
                );

                ui.add_space(4.0);

                // Show diagram type specific information
                match diagram_type {
                    MermaidDiagramType::Flowchart => {
                        ui.label(
                            egui::RichText::new("🔄 流程图 - 显示工作流程和决策点")
                                .size(11.0)
                                .color(egui::Color32::from_rgb(100, 150, 255))
                        );
                    }
                    MermaidDiagramType::Sequence => {
                        ui.label(
                            egui::RichText::new("📋 时序图 - 显示对象间的交互序列")
                                .size(11.0)
                                .color(egui::Color32::from_rgb(100, 150, 255))
                        );
                    }
                    MermaidDiagramType::ClassDiagram => {
                        ui.label(
                            egui::RichText::new("🏗️ 类图 - 显示类的结构和关系")
                                .size(11.0)
                                .color(egui::Color32::from_rgb(100, 150, 255))
                        );
                    }
                    MermaidDiagramType::StateDiagram => {
                        ui.label(
                            egui::RichText::new("🔄 状态图 - 显示状态转换")
                                .size(11.0)
                                .color(egui::Color32::from_rgb(100, 150, 255))
                        );
                    }
                    MermaidDiagramType::PieChart => {
                        ui.label(
                            egui::RichText::new("🥧 饼图 - 显示数据比例")
                                .size(11.0)
                                .color(egui::Color32::from_rgb(100, 150, 255))
                        );
                    }
                    MermaidDiagramType::Gantt => {
                        ui.label(
                            egui::RichText::new("📅 甘特图 - 显示项目时间线")
                                .size(11.0)
                                .color(egui::Color32::from_rgb(100, 150, 255))
                        );
                    }
                    MermaidDiagramType::GitGraph => {
                        ui.label(
                            egui::RichText::new("🌳 Git图 - 显示版本控制分支和提交")
                                .size(11.0)
                                .color(egui::Color32::from_rgb(100, 150, 255))
                        );
                    }
                    MermaidDiagramType::UserJourney => {
                        ui.label(
                            egui::RichText::new("👤 用户旅程图 - 显示用户体验流程")
                                .size(11.0)
                                .color(egui::Color32::from_rgb(100, 150, 255))
                        );
                    }
                    MermaidDiagramType::EntityRelationship => {
                        ui.label(
                            egui::RichText::new("🗃️ 实体关系图 - 显示数据库结构")
                                .size(11.0)
                                .color(egui::Color32::from_rgb(100, 150, 255))
                        );
                    }
                    _ => {
                        ui.label(
                            egui::RichText::new(format!("📊 {} - Mermaid图表", format!("{:?}", diagram_type)))
                                .size(11.0)
                                .color(egui::Color32::from_rgb(100, 150, 255))
                        );
                    }
                }

                ui.add_space(4.0);

                ui.label(
                    egui::RichText::new("(完整的Mermaid渲染功能正在开发中)")
                        .size(10.0)
                        .color(egui::Color32::from_rgb(120, 120, 120))
                        .italics()
                );
            });
        });

        ui.add_space(8.0);
    }

    /// Clear the diagram cache
    pub fn clear_cache(&mut self) {
        self.diagram_cache.clear();
    }

    // ========== Parsing Methods ==========

    /// Parse class definitions from Mermaid code
    fn parse_class_definitions(&self, mermaid_code: &str) -> Vec<(String, Vec<String>, Vec<String>)> {
        let mut classes = Vec::new();
        let mut current_class = None;
        let mut attributes = Vec::new();
        let mut methods = Vec::new();

        for line in mermaid_code.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with("classDiagram") {
                continue;
            }

            // Parse class definition: class ClassName
            if line.starts_with("class ") {
                if let Some(class_name) = current_class.take() {
                    classes.push((class_name, attributes.clone(), methods.clone()));
                    attributes.clear();
                    methods.clear();
                }
                current_class = Some(line[6..].trim().to_string());
            }
            // Parse attributes and methods
            else if line.contains(":") && current_class.is_some() {
                if line.contains("()") {
                    methods.push(line.to_string());
                } else {
                    attributes.push(line.to_string());
                }
            }
        }

        // Add the last class
        if let Some(class_name) = current_class {
            classes.push((class_name, attributes, methods));
        }

        classes
    }

    /// Generate class diagram SVG from parsed data
    fn generate_class_svg_from_data(&self, classes: &[(String, Vec<String>, Vec<String>)]) -> Result<String, String> {
        let mut svg_content = String::new();
        let class_width = 150;
        let class_spacing = 200;
        let start_x = 50;
        let start_y = 30;

        let total_width = start_x * 2 + classes.len() * class_spacing;
        let total_height = 200;

        svg_content.push_str(&format!(
            r##"<svg width="{}" height="{}" xmlns="http://www.w3.org/2000/svg">"##,
            total_width, total_height
        ));

        for (i, (class_name, attributes, methods)) in classes.iter().enumerate() {
            let x = start_x + i * class_spacing;
            let class_height = 30 + (attributes.len() + methods.len()) * 15 + 20;

            // Class box
            svg_content.push_str(&format!(
                r##"<rect x="{}" y="{}" width="{}" height="{}" fill="#f5f5f5" stroke="#333" stroke-width="2"/>"##,
                x, start_y, class_width, class_height
            ));

            // Class name
            let font_family_str = self.get_svg_font_family(None);
            svg_content.push_str(&format!(
                r##"<text x="{}" y="{}" text-anchor="middle" font-family="{}" font-size="12" font-weight="bold" fill="#333">{}</text>"##,
                x + class_width/2, start_y + 20, font_family_str, self.escape_xml(class_name)
            ));

            // Separator line
            let separator_y = start_y + 25;
            svg_content.push_str(&format!(
                r##"<line x1="{}" y1="{}" x2="{}" y2="{}" stroke="#333" stroke-width="1"/>"##,
                x, separator_y, x + class_width, separator_y
            ));

            // Attributes
            let mut text_y = separator_y + 15;
            for attr in attributes {
                svg_content.push_str(&format!(
                    r##"<text x="{}" y="{}" font-family="{}" font-size="10" fill="#666">{}</text>"##,
                    x + 10, text_y, font_family_str, self.escape_xml(attr)
                ));
                text_y += 15;
            }

            // Methods separator
            if !methods.is_empty() {
                svg_content.push_str(&format!(
                    r##"<line x1="{}" y1="{}" x2="{}" y2="{}" stroke="#333" stroke-width="1"/>"##,
                    x, text_y - 5, x + class_width, text_y - 5
                ));

                // Methods
                for method in methods {
                    svg_content.push_str(&format!(
                        r##"<text x="{}" y="{}" font-family="{}" font-size="10" fill="#666">{}</text>"##,
                        x + 10, text_y + 10, font_family_str, self.escape_xml(method)
                    ));
                    text_y += 15;
                }
            }
        }

        svg_content.push_str("</svg>");
        Ok(svg_content)
    }

    /// Parse state definitions from Mermaid code
    fn parse_state_definitions(&self, mermaid_code: &str) -> Vec<(String, String)> {
        let mut states = Vec::new();

        for line in mermaid_code.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with("stateDiagram") {
                continue;
            }

            // Parse state transitions: StateA --> StateB
            if line.contains("-->") {
                if let Some(arrow_pos) = line.find("-->") {
                    let from = line[..arrow_pos].trim().to_string();
                    let to = line[arrow_pos+3..].trim().to_string();

                    if !states.iter().any(|(name, _)| name == &from) {
                        states.push((from.clone(), from));
                    }
                    if !states.iter().any(|(name, _)| name == &to) {
                        states.push((to.clone(), to));
                    }
                }
            }
        }

        states
    }

    /// Generate state diagram SVG from parsed data
    fn generate_state_svg_from_data(&self, states: &[(String, String)]) -> Result<String, String> {
        let mut svg_content = String::new();
        let state_width = 100;
        let state_spacing = 150;
        let start_x = 50;
        let start_y = 80;

        let total_width = start_x * 2 + states.len() * state_spacing;
        let total_height = 200;

        svg_content.push_str(&format!(
            r##"<svg width="{}" height="{}" xmlns="http://www.w3.org/2000/svg">"##,
            total_width, total_height
        ));

        // Add arrow marker
        svg_content.push_str(r##"
            <defs>
                <marker id="arrowhead" markerWidth="10" markerHeight="7" refX="9" refY="3.5" orient="auto">
                    <polygon points="0 0, 10 3.5, 0 7" fill="#666"/>
                </marker>
            </defs>"##);

        // Draw states
        for (i, (state_id, state_name)) in states.iter().enumerate() {
            let x = start_x + i * state_spacing;

            // State box
            svg_content.push_str(&format!(
                r##"<rect x="{}" y="{}" width="{}" height="40" rx="20" fill="#e8f5e8" stroke="#4caf50" stroke-width="2"/>"##,
                x, start_y, state_width
            ));

            // State name
            let font_family_str = self.get_svg_font_family(None);
            svg_content.push_str(&format!(
                r##"<text x="{}" y="{}" text-anchor="middle" font-family="{}" font-size="12" font-weight="500" fill="#2e7d32">{}</text>"##,
                x + state_width/2, start_y + 25, font_family_str, self.escape_xml(state_name)
            ));

            // Draw arrow to next state
            if i + 1 < states.len() {
                let arrow_start_x = x + state_width;
                let arrow_end_x = start_x + (i + 1) * state_spacing;
                svg_content.push_str(&format!(
                    r##"<line x1="{}" y1="{}" x2="{}" y2="{}" stroke="#666" stroke-width="2" marker-end="url(#arrowhead)"/>"##,
                    arrow_start_x, start_y + 20, arrow_end_x, start_y + 20
                ));
            }
        }

        svg_content.push_str("</svg>");
        Ok(svg_content)
    }

    /// Parse gantt tasks from Mermaid code
    fn parse_gantt_tasks(&self, mermaid_code: &str) -> Vec<(String, String, String)> {
        let mut tasks = Vec::new();

        for line in mermaid_code.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with("gantt") || line.starts_with("title") || line.starts_with("dateFormat") {
                continue;
            }

            // Parse task: TaskName : status, start, duration
            if line.contains(":") {
                let parts: Vec<&str> = line.split(':').collect();
                if parts.len() >= 2 {
                    let task_name = parts[0].trim().to_string();
                    let task_info = parts[1].trim().to_string();
                    tasks.push((task_name, task_info, "active".to_string()));
                }
            }
        }

        tasks
    }

    /// Generate gantt chart SVG from parsed data
    fn generate_gantt_svg_from_data(&self, tasks: &[(String, String, String)]) -> Result<String, String> {
        let mut svg_content = String::new();
        let task_height = 25;
        let task_spacing = 30;
        let start_x = 120;
        let start_y = 30;
        let chart_width = 300;

        let total_width = start_x + chart_width + 50;
        let total_height = start_y + tasks.len() * task_spacing + 50;

        svg_content.push_str(&format!(
            r##"<svg width="{}" height="{}" xmlns="http://www.w3.org/2000/svg">"##,
            total_width, total_height
        ));

        // Get font family for all text elements
        let font_family_str = self.get_svg_font_family(None);

        // Draw tasks
        for (i, (task_name, _task_info, _status)) in tasks.iter().enumerate() {
            let y = start_y + i * task_spacing;

            // Task background
            svg_content.push_str(&format!(
                r##"<rect x="{}" y="{}" width="{}" height="{}" fill="#f5f5f5" stroke="#ddd" stroke-width="1"/>"##,
                start_x, y, chart_width, task_height
            ));

            // Task name
            svg_content.push_str(&format!(
                r##"<text x="{}" y="{}" font-family="{}" font-size="11" fill="#333">{}</text>"##,
                10, y + 17, font_family_str, self.escape_xml(task_name)
            ));

            // Task bar (simplified - random width for demo)
            let bar_width = 80 + (i * 40) % 120;
            let bar_start = start_x + (i * 30) % 100;
            let colors = ["#2196f3", "#4caf50", "#ff9800", "#9c27b0"];
            let color = colors[i % colors.len()];

            svg_content.push_str(&format!(
                r##"<rect x="{}" y="{}" width="{}" height="{}" fill="{}"/>"##,
                bar_start, y, bar_width, task_height, color
            ));
        }

        // Time axis
        let axis_y = start_y + tasks.len() * task_spacing + 20;
        svg_content.push_str(&format!(
            r##"<line x1="{}" y1="{}" x2="{}" y2="{}" stroke="#666" stroke-width="1"/>"##,
            start_x, axis_y, start_x + chart_width, axis_y
        ));

        // Time labels
        for i in 0..4 {
            let x = start_x + i * (chart_width / 3);
            svg_content.push_str(&format!(
                r##"<text x="{}" y="{}" text-anchor="middle" font-family="{}" font-size="10" fill="#666">周{}</text>"##,
                x, axis_y + 15, font_family_str, i + 1
            ));
        }

        svg_content.push_str("</svg>");
        Ok(svg_content)
    }

    /// Parse ER entities from Mermaid code
    fn parse_er_entities(&self, mermaid_code: &str) -> Vec<(String, Vec<String>)> {
        let mut entities = Vec::new();
        let mut current_entity = None;
        let mut attributes = Vec::new();

        for line in mermaid_code.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with("erDiagram") {
                continue;
            }

            // Parse entity definition
            if line.contains("{") {
                if let Some(entity_name) = current_entity.take() {
                    entities.push((entity_name, attributes.clone()));
                    attributes.clear();
                }
                current_entity = Some(line.replace("{", "").trim().to_string());
            }
            // Parse attributes
            else if line.contains("}") {
                if let Some(entity_name) = current_entity.take() {
                    entities.push((entity_name, attributes.clone()));
                    attributes.clear();
                }
            }
            // Parse attribute lines
            else if current_entity.is_some() && !line.is_empty() {
                attributes.push(line.to_string());
            }
        }

        entities
    }

    /// Generate ER diagram SVG from parsed data
    fn generate_er_svg_from_data(&self, entities: &[(String, Vec<String>)]) -> Result<String, String> {
        let mut svg_content = String::new();
        let entity_width = 150;
        let entity_spacing = 200;
        let start_x = 50;
        let start_y = 50;

        let total_width = start_x * 2 + entities.len() * entity_spacing;
        let total_height = 250;

        svg_content.push_str(&format!(
            r##"<svg width="{}" height="{}" xmlns="http://www.w3.org/2000/svg">"##,
            total_width, total_height
        ));

        // Add arrow marker
        svg_content.push_str(r##"
            <defs>
                <marker id="arrowhead" markerWidth="10" markerHeight="7" refX="9" refY="3.5" orient="auto">
                    <polygon points="0 0, 10 3.5, 0 7" fill="#666"/>
                </marker>
            </defs>"##);

        // Draw entities
        for (i, (entity_name, attributes)) in entities.iter().enumerate() {
            let x = start_x + i * entity_spacing;
            let entity_height = 40 + attributes.len() * 15;
            let colors = [("#e3f2fd", "#1976d2"), ("#e8f5e8", "#2e7d32"), ("#fff3e0", "#e65100")];
            let (bg_color, text_color) = colors[i % colors.len()];

            // Entity box
            svg_content.push_str(&format!(
                r##"<rect x="{}" y="{}" width="{}" height="{}" fill="{}" stroke="{}" stroke-width="2"/>"##,
                x, start_y, entity_width, entity_height, bg_color, text_color
            ));

            // Entity name
            let font_family_str = self.get_svg_font_family(None);
            svg_content.push_str(&format!(
                r##"<text x="{}" y="{}" text-anchor="middle" font-family="{}" font-size="12" font-weight="bold" fill="{}">{}</text>"##,
                x + entity_width/2, start_y + 20, font_family_str, text_color, self.escape_xml(entity_name)
            ));

            // Attributes
            for (j, attr) in attributes.iter().enumerate() {
                svg_content.push_str(&format!(
                    r##"<text x="{}" y="{}" font-family="{}" font-size="10" fill="#666">{}</text>"##,
                    x + 10, start_y + 35 + j * 15, font_family_str, self.escape_xml(attr)
                ));
            }

            // Draw relationship line to next entity
            if i + 1 < entities.len() {
                let line_y = start_y + entity_height / 2;
                let line_start_x = x + entity_width;
                let line_end_x = start_x + (i + 1) * entity_spacing;

                svg_content.push_str(&format!(
                    r##"<line x1="{}" y1="{}" x2="{}" y2="{}" stroke="#666" stroke-width="2" marker-end="url(#arrowhead)"/>"##,
                    line_start_x, line_y, line_end_x, line_y
                ));
            }
        }

        svg_content.push_str("</svg>");
        Ok(svg_content)
    }

    /// Parse pie chart data from Mermaid code
    fn parse_pie_data(&self, mermaid_code: &str) -> Vec<(String, f32)> {
        let mut data = Vec::new();

        for line in mermaid_code.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with("pie") || line.starts_with("title") {
                continue;
            }

            // Parse pie data: "Label" : value
            if line.contains(":") {
                let parts: Vec<&str> = line.split(':').collect();
                if parts.len() >= 2 {
                    let label = parts[0].trim().replace("\"", "");
                    if let Ok(value) = parts[1].trim().parse::<f32>() {
                        data.push((label, value));
                    }
                }
            }
        }

        data
    }

    /// Generate pie chart SVG from parsed data
    fn generate_pie_svg_from_data(&self, data: &[(String, f32)]) -> Result<String, String> {
        let mut svg_content = String::new();
        let center_x = 150.0;
        let center_y = 100.0;
        let radius = 60.0;

        svg_content.push_str(r##"<svg width="300" height="200" xmlns="http://www.w3.org/2000/svg">"##);

        let total: f32 = data.iter().map(|(_, value)| value).sum();
        let mut current_angle = 0.0;
        let colors = ["#2196f3", "#4caf50", "#ff9800", "#f44336", "#9c27b0", "#607d8b"];

        for (i, (label, value)) in data.iter().enumerate() {
            let percentage = value / total;
            let angle = percentage * 360.0;
            let end_angle = current_angle + angle;

            // Calculate arc path
            let start_x = center_x + radius * (current_angle.to_radians().cos());
            let start_y = center_y + radius * (current_angle.to_radians().sin());
            let end_x = center_x + radius * (end_angle.to_radians().cos());
            let end_y = center_y + radius * (end_angle.to_radians().sin());

            let large_arc = if angle > 180.0 { 1 } else { 0 };
            let color = colors[i % colors.len()];

            // Draw pie slice
            svg_content.push_str(&format!(
                r##"<path d="M {} {} L {} {} A {} {} 0 {} 1 {} {} Z" fill="{}"/>"##,
                center_x, center_y, start_x, start_y, radius, radius, large_arc, end_x, end_y, color
            ));

            // Add label
            let label_angle = current_angle + angle / 2.0;
            let label_x = center_x + (radius + 20.0) * (label_angle.to_radians().cos());
            let label_y = center_y + (radius + 20.0) * (label_angle.to_radians().sin());

            let font_family_str = self.get_svg_font_family(None);
            svg_content.push_str(&format!(
                r##"<text x="{}" y="{}" text-anchor="middle" font-family="{}" font-size="10" font-weight="500" fill="{}">{}: {:.1}%</text>"##,
                label_x, label_y, font_family_str, color, self.escape_xml(label), percentage * 100.0
            ));

            current_angle = end_angle;
        }

        svg_content.push_str("</svg>");
        Ok(svg_content)
    }

    /// Generate default Git graph SVG
    fn generate_default_gitgraph(&self) -> Result<String, String> {
        let font_family_str = self.get_svg_font_family(None);
        let svg = format!(r##"<svg width="400" height="250" xmlns="http://www.w3.org/2000/svg">
            <rect x="0" y="0" width="400" height="250" fill="#f8f9fa"/>
            <text x="200" y="30" text-anchor="middle" font-family="{}" font-size="16" font-weight="bold" fill="#333">🌳 Git Graph</text>

            <!-- Main branch line -->
            <line x1="50" y1="80" x2="350" y2="80" stroke="#4caf50" stroke-width="3"/>

            <!-- Commits on main branch -->
            <circle cx="80" cy="80" r="6" fill="#4caf50"/>
            <circle cx="150" cy="80" r="6" fill="#4caf50"/>
            <circle cx="220" cy="80" r="6" fill="#4caf50"/>
            <circle cx="290" cy="80" r="6" fill="#4caf50"/>

            <!-- Feature branch -->
            <line x1="150" y1="80" x2="150" y2="120" stroke="#2196f3" stroke-width="2"/>
            <line x1="150" y1="120" x2="220" y2="120" stroke="#2196f3" stroke-width="2"/>
            <line x1="220" y1="120" x2="220" y2="80" stroke="#2196f3" stroke-width="2"/>

            <!-- Feature branch commits -->
            <circle cx="180" cy="120" r="5" fill="#2196f3"/>

            <!-- Commit labels -->
            <text x="80" y="100" text-anchor="middle" font-family="{}" font-size="10" fill="#666">Initial</text>
            <text x="150" y="100" text-anchor="middle" font-family="{}" font-size="10" fill="#666">Feature</text>
            <text x="180" y="140" text-anchor="middle" font-family="{}" font-size="10" fill="#666">Work</text>
            <text x="220" y="100" text-anchor="middle" font-family="{}" font-size="10" fill="#666">Merge</text>
            <text x="290" y="100" text-anchor="middle" font-family="{}" font-size="10" fill="#666">Latest</text>

            <!-- Branch labels -->
            <text x="30" y="85" font-family="{}" font-size="12" font-weight="bold" fill="#4caf50">main</text>
            <text x="30" y="125" font-family="{}" font-size="12" font-weight="bold" fill="#2196f3">feature</text>
        </svg>"##, font_family_str, font_family_str, font_family_str, font_family_str, font_family_str, font_family_str, font_family_str, font_family_str);
        Ok(svg)
    }

    /// Generate default User Journey SVG
    fn generate_default_userjourney(&self) -> Result<String, String> {
        let font_family_str = self.get_svg_font_family(None);
        let svg = format!(r##"<svg width="500" height="300" xmlns="http://www.w3.org/2000/svg">
            <rect x="0" y="0" width="500" height="300" fill="#f8f9fa"/>
            <text x="250" y="30" text-anchor="middle" font-family="{}" font-size="16" font-weight="bold" fill="#333">👤 User Journey</text>

            <!-- Journey timeline -->
            <line x1="50" y1="100" x2="450" y2="100" stroke="#ddd" stroke-width="2"/>

            <!-- Journey steps -->
            <circle cx="100" cy="100" r="8" fill="#4caf50"/>
            <circle cx="200" cy="100" r="8" fill="#ff9800"/>
            <circle cx="300" cy="100" r="8" fill="#2196f3"/>
            <circle cx="400" cy="100" r="8" fill="#9c27b0"/>

            <!-- Step labels -->
            <text x="100" y="130" text-anchor="middle" font-family="{}" font-size="12" font-weight="bold" fill="#4caf50">Discover</text>
            <text x="200" y="130" text-anchor="middle" font-family="{}" font-size="12" font-weight="bold" fill="#ff9800">Research</text>
            <text x="300" y="130" text-anchor="middle" font-family="{}" font-size="12" font-weight="bold" fill="#2196f3">Purchase</text>
            <text x="400" y="130" text-anchor="middle" font-family="{}" font-size="12" font-weight="bold" fill="#9c27b0">Support</text>

            <!-- Satisfaction scores -->
            <text x="100" y="150" text-anchor="middle" font-family="{}" font-size="10" fill="#666">😊 5</text>
            <text x="200" y="150" text-anchor="middle" font-family="{}" font-size="10" fill="#666">😐 3</text>
            <text x="300" y="150" text-anchor="middle" font-family="{}" font-size="10" fill="#666">😊 4</text>
            <text x="400" y="150" text-anchor="middle" font-family="{}" font-size="10" fill="#666">😊 5</text>

            <!-- Actor -->
            <text x="50" y="200" font-family="{}" font-size="14" font-weight="bold" fill="#333">Actor: Customer</text>
            <text x="50" y="220" font-family="{}" font-size="12" fill="#666">Journey through product lifecycle</text>
        </svg>"##, font_family_str, font_family_str, font_family_str, font_family_str, font_family_str, font_family_str, font_family_str, font_family_str, font_family_str, font_family_str, font_family_str);
        Ok(svg)
    }

    /// Parse Git commits from Mermaid code
    fn parse_git_commits(&self, mermaid_code: &str) -> Vec<(String, String, String)> {
        let mut commits = Vec::new();

        for line in mermaid_code.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("commit") {
                // Extract commit info: commit id:"message"
                if let Some(colon_pos) = trimmed.find(':') {
                    let id_part = trimmed[6..colon_pos].trim();
                    let message_part = trimmed[colon_pos + 1..].trim().trim_matches('"');
                    commits.push((id_part.to_string(), message_part.to_string(), "main".to_string()));
                } else {
                    // Simple commit without message
                    let id = trimmed[6..].trim();
                    commits.push((id.to_string(), "Commit".to_string(), "main".to_string()));
                }
            } else if trimmed.starts_with("branch") {
                // Handle branch creation
                let branch_name = trimmed[6..].trim();
                commits.push(("".to_string(), format!("Create branch {}", branch_name), branch_name.to_string()));
            }
        }

        if commits.is_empty() {
            // Default commits for demonstration
            commits.push(("c1".to_string(), "Initial commit".to_string(), "main".to_string()));
            commits.push(("c2".to_string(), "Add feature".to_string(), "main".to_string()));
            commits.push(("c3".to_string(), "Fix bug".to_string(), "main".to_string()));
        }

        commits
    }

    /// Parse User Journey from Mermaid code
    fn parse_user_journey(&self, mermaid_code: &str) -> Vec<(String, String, i32)> {
        let mut journey_data = Vec::new();

        for line in mermaid_code.lines() {
            let trimmed = line.trim();
            if trimmed.contains(':') && !trimmed.starts_with("title") {
                // Parse step: "Step name: score: Actor1, Actor2"
                let parts: Vec<&str> = trimmed.split(':').collect();
                if parts.len() >= 2 {
                    let step_name = parts[0].trim();
                    let score_str = parts[1].trim();
                    let score = score_str.parse::<i32>().unwrap_or(3);
                    let actors = if parts.len() > 2 { parts[2].trim() } else { "User" };
                    journey_data.push((step_name.to_string(), actors.to_string(), score));
                }
            }
        }

        if journey_data.is_empty() {
            // Default journey for demonstration
            journey_data.push(("Discover".to_string(), "Customer".to_string(), 5));
            journey_data.push(("Research".to_string(), "Customer".to_string(), 3));
            journey_data.push(("Purchase".to_string(), "Customer".to_string(), 4));
            journey_data.push(("Support".to_string(), "Customer".to_string(), 5));
        }

        journey_data
    }

    /// Generate Git graph SVG from parsed data
    fn generate_gitgraph_svg_from_data(&self, commits: &[(String, String, String)]) -> Result<String, String> {
        let mut svg_content = String::new();
        let commit_spacing = 80;
        let start_x = 80;
        let main_y = 80;
        let branch_y = 120;

        let font_family_str = self.get_svg_font_family(None);
        svg_content.push_str(&format!(r##"<svg width="{}" height="200" xmlns="http://www.w3.org/2000/svg">"##,
            start_x + commits.len() * commit_spacing + 50));
        svg_content.push_str(r##"<rect x="0" y="0" width="100%" height="200" fill="#f8f9fa"/>"##);
        svg_content.push_str(&format!(r##"<text x="50%" y="30" text-anchor="middle" font-family="{}" font-size="16" font-weight="bold" fill="#333">🌳 Git Graph</text>"##, font_family_str));

        // Draw main branch line
        svg_content.push_str(&format!(r##"<line x1="{}" y1="{}" x2="{}" y2="{}" stroke="#4caf50" stroke-width="3"/>"##,
            start_x - 20, main_y, start_x + commits.len() * commit_spacing, main_y));

        // Draw commits
        for (i, (id, message, branch)) in commits.iter().enumerate() {
            let x = start_x + i * commit_spacing;
            let y = if branch == "main" { main_y } else { branch_y };
            let color = if branch == "main" { "#4caf50" } else { "#2196f3" };

            // Draw commit circle
            svg_content.push_str(&format!(r##"<circle cx="{}" cy="{}" r="6" fill="{}"/>"##, x, y, color));

            // Draw commit label
            let display_text = if id.is_empty() { message } else { id };
            svg_content.push_str(&format!(
                r##"<text x="{}" y="{}" text-anchor="middle" font-family="{}" font-size="10" fill="#666">{}</text>"##,
                x, y + 20, font_family_str, self.escape_xml(display_text)
            ));
        }

        // Add branch labels
        svg_content.push_str(&format!(r##"<text x="30" y="{}" font-family="{}" font-size="12" font-weight="bold" fill="#4caf50">main</text>"##, main_y + 5, font_family_str));

        svg_content.push_str("</svg>");
        Ok(svg_content)
    }

    /// Generate User Journey SVG from parsed data
    fn generate_userjourney_svg_from_data(&self, journey_data: &[(String, String, i32)]) -> Result<String, String> {
        let mut svg_content = String::new();
        let step_spacing = 100;
        let start_x = 80;
        let timeline_y = 100;

        let font_family_str = self.get_svg_font_family(None);
        svg_content.push_str(&format!(r##"<svg width="{}" height="250" xmlns="http://www.w3.org/2000/svg">"##,
            start_x + journey_data.len() * step_spacing + 50));
        svg_content.push_str(r##"<rect x="0" y="0" width="100%" height="250" fill="#f8f9fa"/>"##);
        svg_content.push_str(&format!(r##"<text x="50%" y="30" text-anchor="middle" font-family="{}" font-size="16" font-weight="bold" fill="#333">👤 User Journey</text>"##, font_family_str));

        // Draw timeline
        svg_content.push_str(&format!(r##"<line x1="{}" y1="{}" x2="{}" y2="{}" stroke="#ddd" stroke-width="2"/>"##,
            start_x - 20, timeline_y, start_x + journey_data.len() * step_spacing, timeline_y));

        // Draw journey steps
        for (i, (step, actor, score)) in journey_data.iter().enumerate() {
            let x = start_x + i * step_spacing;
            let color = match score {
                5 => "#4caf50",
                4 => "#8bc34a",
                3 => "#ff9800",
                2 => "#ff5722",
                _ => "#f44336",
            };
            let emoji = match score {
                5 => "😊",
                4 => "🙂",
                3 => "😐",
                2 => "😕",
                _ => "😞",
            };

            // Draw step circle
            svg_content.push_str(&format!(r##"<circle cx="{}" cy="{}" r="8" fill="{}"/>"##, x, timeline_y, color));

            // Draw step label
            svg_content.push_str(&format!(
                r##"<text x="{}" y="{}" text-anchor="middle" font-family="{}" font-size="12" font-weight="bold" fill="{}">{}</text>"##,
                x, timeline_y + 30, font_family_str, color, self.escape_xml(step)
            ));

            // Draw satisfaction score
            svg_content.push_str(&format!(
                r##"<text x="{}" y="{}" text-anchor="middle" font-family="{}" font-size="10" fill="#666">{} {}</text>"##,
                x, timeline_y + 50, font_family_str, emoji, score
            ));
        }

        // Add actor info
        if let Some((_, actor, _)) = journey_data.first() {
            svg_content.push_str(&format!(r##"<text x="50" y="200" font-family="{}" font-size="14" font-weight="bold" fill="#333">Actor: {}</text>"##, font_family_str, self.escape_xml(actor)));
        }

        svg_content.push_str("</svg>");
        Ok(svg_content)
    }
}
