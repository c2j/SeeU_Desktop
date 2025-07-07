use pulldown_cmark::{Parser, Options, html, Alignment};
use eframe::egui::{self, TextFormat, Color32, Ui, FontFamily};
use eframe::egui::text::LayoutJob;

/// 表格自适应颜色配置
#[derive(Debug, Clone)]
struct TableColors {
    border: Color32,
    header_bg: Color32,
    even_row_bg: Color32,
    odd_row_bg: Color32,
    text: Color32,
    header_text: Color32,
}

/// Render markdown text to HTML
pub fn markdown_to_html(markdown: &str) -> String {
    // Set up options and parser
    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_FOOTNOTES);
    options.insert(Options::ENABLE_TASKLISTS);

    let parser = Parser::new_ext(markdown, options);

    // Write to string buffer
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);

    html_output
}

/// Render markdown text directly to egui
pub fn render_markdown(ui: &mut Ui, markdown: &str) {
    render_markdown_with_highlight(ui, markdown, &[], None);
}

/// Render markdown text directly to egui with search term highlighting
pub fn render_markdown_with_highlight(ui: &mut Ui, markdown: &str, search_terms: &[String], font_family: Option<&str>) {
    // Set up options and parser
    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_FOOTNOTES);
    options.insert(Options::ENABLE_TASKLISTS);

    let parser = Parser::new_ext(markdown, options);

    // Process markdown events and render them
    let mut renderer = MarkdownRenderer::new(ui, search_terms, font_family);
    renderer.render(parser);
}

/// Table cell content with formatting information
#[derive(Debug, Clone)]
struct MarkdownTableCell {
    content: String,
    is_bold: bool,
    is_italic: bool,
    is_code: bool,
    is_link: bool,
    link_url: String,
}

impl MarkdownTableCell {
    fn new() -> Self {
        Self {
            content: String::new(),
            is_bold: false,
            is_italic: false,
            is_code: false,
            is_link: false,
            link_url: String::new(),
        }
    }

    fn add_text(&mut self, text: &str) {
        self.content.push_str(text);
    }

    fn set_formatting(&mut self, is_bold: bool, is_italic: bool, is_code: bool, is_link: bool, link_url: String) {
        self.is_bold = is_bold;
        self.is_italic = is_italic;
        self.is_code = is_code;
        self.is_link = is_link;
        self.link_url = link_url;
    }
}

/// Table information with alignment and formatting
#[derive(Debug, Clone)]
struct TableInfo {
    rows: Vec<Vec<MarkdownTableCell>>,
    column_alignments: Vec<Alignment>,
    is_header_row: Vec<bool>,
}

impl TableInfo {
    fn new() -> Self {
        Self {
            rows: Vec::new(),
            column_alignments: Vec::new(),
            is_header_row: Vec::new(),
        }
    }
}

/// Markdown renderer that can handle complex elements like tables
struct MarkdownRenderer<'a> {
    ui: &'a mut Ui,
    current_text: String,
    is_heading: bool,
    heading_level: u8,
    is_bold: bool,
    is_italic: bool,
    is_code: bool,
    is_link: bool,
    link_url: String,
    in_table: bool,
    current_table: TableInfo,
    current_row: Vec<MarkdownTableCell>,
    current_cell: MarkdownTableCell,
    is_table_header: bool,
    table_counter: usize,
    search_terms: Vec<String>,
    in_code_block: bool,
    code_block_content: String,
    code_block_language: String,
    mermaid_renderer: crate::mermaid::MermaidRenderer,
    svg_test_renderer: crate::mermaid::SvgTestRenderer,
    font_family: Option<String>,
}

impl<'a> MarkdownRenderer<'a> {
    fn new(ui: &'a mut Ui, search_terms: &[String], font_family: Option<&str>) -> Self {
        Self {
            ui,
            current_text: String::new(),
            is_heading: false,
            heading_level: 0,
            is_bold: false,
            is_italic: false,
            is_code: false,
            is_link: false,
            link_url: String::new(),
            in_table: false,
            current_table: TableInfo::new(),
            current_row: Vec::new(),
            current_cell: MarkdownTableCell::new(),
            is_table_header: false,
            table_counter: 0,
            search_terms: search_terms.to_vec(),
            in_code_block: false,
            code_block_content: String::new(),
            code_block_language: String::new(),
            mermaid_renderer: crate::mermaid::MermaidRenderer::new(),
            svg_test_renderer: crate::mermaid::SvgTestRenderer::new(),
            font_family: font_family.map(|s| s.to_string()),
        }
    }

    fn render(&mut self, parser: Parser) {
        use pulldown_cmark::Event::*;


        for event in parser {
            match event {
                Start(tag) => {
                    self.flush_text();
                    self.handle_start_tag(tag);
                },
                End(tag) => {
                    self.flush_text();
                    self.handle_end_tag(tag);
                },
                Text(text) => {
                    if self.in_code_block {
                        self.code_block_content.push_str(&text);
                    } else if self.in_table {
                        self.current_cell.add_text(&text);
                    } else {
                        self.current_text.push_str(&text);
                    }
                },
                Code(text) => {
                    if self.in_table {
                        self.current_cell.add_text(&format!("`{}`", text));
                        self.current_cell.set_formatting(
                            self.is_bold,
                            self.is_italic,
                            true,
                            self.is_link,
                            self.link_url.clone()
                        );
                    } else {
                        self.is_code = true;
                        self.current_text.push_str(&text);
                        self.is_code = false;
                    }
                },
                SoftBreak => {
                    if self.in_table {
                        self.current_cell.add_text(" ");
                    } else {
                        self.current_text.push(' ');
                    }
                },
                HardBreak => {
                    if self.in_table {
                        self.current_cell.add_text("\n");
                    } else {
                        self.flush_text();
                        self.ui.add(egui::Label::new("\n"));
                    }
                },
                _ => {}
            }
        }

        // Flush any remaining content
        self.flush_text();
        if self.in_table {
            self.render_enhanced_table();
        }
        if self.in_code_block {
            self.render_code_block();
        }
    }

    fn handle_start_tag(&mut self, tag: pulldown_cmark::Tag) {
        use pulldown_cmark::Tag::*;

        match tag {
            Heading(level, _, _) => {
                self.is_heading = true;
                self.heading_level = level as u8;
            },
            Emphasis => self.is_italic = true,
            Strong => self.is_bold = true,
            CodeBlock(kind) => {
                self.in_code_block = true;
                self.code_block_content.clear();
                // Extract language from code block kind
                self.code_block_language = match kind {
                    pulldown_cmark::CodeBlockKind::Fenced(lang) => lang.to_string(),
                    pulldown_cmark::CodeBlockKind::Indented => String::new(),
                };
            },
            Link(_, url, _) => {
                self.is_link = true;
                self.link_url = url.to_string();
            },
            Table(alignments) => {
                self.in_table = true;
                self.current_table = TableInfo::new();
                self.current_table.column_alignments = alignments;
                self.table_counter += 1;
            },
            TableHead => {
                self.is_table_header = true;
                self.current_row.clear();
            },
            TableRow => {
                self.current_row.clear();
            },
            TableCell => {
                self.current_cell = MarkdownTableCell::new();
                // Apply current formatting state to the cell
                self.current_cell.set_formatting(
                    self.is_bold,
                    self.is_italic,
                    self.is_code,
                    self.is_link,
                    self.link_url.clone(),
                );
            },
            _ => {}
        }
    }

    fn handle_end_tag(&mut self, tag: pulldown_cmark::Tag) {
        use pulldown_cmark::Tag::*;

        match tag {
            Heading(_, _, _) => {
                self.is_heading = false;
                self.heading_level = 0;
                self.ui.add_space(10.0);
            },
            Paragraph => {
                self.ui.add_space(5.0);
            },
            Emphasis => self.is_italic = false,
            Strong => self.is_bold = false,
            CodeBlock(_) => {
                self.render_code_block();
                self.in_code_block = false;
                self.code_block_content.clear();
                self.code_block_language.clear();
            },
            Link(_, _, _) => {
                self.is_link = false;
                self.link_url.clear();
            },
            Table(_) => {
                self.render_enhanced_table();
                self.in_table = false;
            },
            TableHead => {
                self.current_table.rows.push(self.current_row.clone());
                self.current_table.is_header_row.push(true);
                self.is_table_header = false;
            },
            TableRow => {
                self.current_table.rows.push(self.current_row.clone());
                self.current_table.is_header_row.push(false);
            },
            TableCell => {
                // Update cell formatting with current state
                self.current_cell.set_formatting(
                    self.is_bold,
                    self.is_italic,
                    self.is_code,
                    self.is_link,
                    self.link_url.clone(),
                );
                self.current_row.push(self.current_cell.clone());
                self.current_cell = MarkdownTableCell::new();
            },
            _ => {}
        }
    }

    fn flush_text(&mut self) {
        if !self.current_text.is_empty() {
            if self.is_link {
                self.render_link(&self.current_text.clone(), &self.link_url.clone());
            } else {
                self.render_text(&self.current_text.clone());
            }
            self.current_text.clear();
        }
    }

    fn render_text(&mut self, text: &str) {
        // If we have search terms, create highlighted layout job
        if !self.search_terms.is_empty() {
            let layout_job = self.create_highlighted_text_layout(text);
            self.ui.add(egui::Label::new(layout_job));
        } else {
            let mut rich_text = egui::RichText::new(text);

            // Apply formatting
            if self.is_heading {
                let size = match self.heading_level {
                    1 => 24.0,
                    2 => 20.0,
                    3 => 18.0,
                    4 => 16.0,
                    _ => 14.0,
                };
                rich_text = rich_text.size(size).color(Color32::from_rgb(60, 120, 216));
            }

            if self.is_bold {
                rich_text = rich_text.strong();
            }

            if self.is_code {
                rich_text = rich_text.monospace().background_color(Color32::from_rgb(240, 240, 240));
            }

            self.ui.add(egui::Label::new(rich_text));
        }
    }

    fn create_highlighted_text_layout(&self, text: &str) -> LayoutJob {
        use egui::{text::LayoutJob, Color32, FontId, TextFormat};

        // Convert to character vector for safe indexing
        let chars: Vec<char> = text.chars().collect();
        let text_lower = text.to_lowercase();
        let text_lower_chars: Vec<char> = text_lower.chars().collect();

        let mut highlighted_ranges = Vec::new();

        // Find all matches
        for term in &self.search_terms {
            let term_lower = term.to_lowercase();
            let term_chars: Vec<char> = term_lower.chars().collect();

            if term_chars.is_empty() {
                continue;
            }

            let mut start = 0;
            while start + term_chars.len() <= text_lower_chars.len() {
                // Check if term matches at current position
                let mut matches = true;
                for (i, &term_char) in term_chars.iter().enumerate() {
                    if text_lower_chars[start + i] != term_char {
                        matches = false;
                        break;
                    }
                }

                if matches {
                    // Check if this range overlaps with existing highlights
                    let range = (start, start + term_chars.len());
                    let overlaps = highlighted_ranges.iter().any(|&(existing_start, existing_end)| {
                        range.0 < existing_end && range.1 > existing_start
                    });

                    if !overlaps {
                        highlighted_ranges.push(range);
                    }

                    start += term_chars.len();
                } else {
                    start += 1;
                }
            }
        }

        // Sort ranges by start position
        highlighted_ranges.sort_by_key(|&(start, _)| start);

        // Create LayoutJob with highlighting
        let mut layout_job = LayoutJob::default();
        let mut last_end = 0;

        // Determine base font size and style
        let (font_size, text_color) = if self.is_heading {
            let size = match self.heading_level {
                1 => 24.0,
                2 => 20.0,
                3 => 18.0,
                4 => 16.0,
                _ => 14.0,
            };
            (size, Color32::from_rgb(60, 120, 216))
        } else {
            (14.0, Color32::from_gray(200))
        };

        // Default text format
        let mut normal_format = TextFormat {
            font_id: if self.is_code {
                FontId::monospace(font_size)
            } else {
                FontId::default()
            },
            color: text_color,
            ..Default::default()
        };

        if self.is_bold {
            // Note: egui doesn't have a direct bold font, but we can use strong style
        }

        if self.is_code {
            normal_format.background = Color32::from_rgb(240, 240, 240);
        }

        // Highlighted text format
        let highlight_format = TextFormat {
            font_id: normal_format.font_id.clone(),
            color: Color32::BLACK,
            background: Color32::YELLOW,
            ..Default::default()
        };

        for (start, end) in highlighted_ranges {
            // Add normal text before highlight
            if start > last_end {
                let normal_text: String = chars[last_end..start].iter().collect();
                layout_job.append(&normal_text, 0.0, normal_format.clone());
            }

            // Add highlighted text
            let highlighted_text: String = chars[start..end].iter().collect();
            layout_job.append(&highlighted_text, 0.0, highlight_format.clone());

            last_end = end;
        }

        // Add remaining normal text
        if last_end < chars.len() {
            let remaining_text: String = chars[last_end..].iter().collect();
            layout_job.append(&remaining_text, 0.0, normal_format);
        }

        layout_job
    }

    fn render_link(&mut self, text: &str, url: &str) {
        let link_text = egui::RichText::new(text)
            .color(Color32::from_rgb(0, 102, 204))
            .underline();

        if self.ui.add(egui::Label::new(link_text).sense(egui::Sense::click())).clicked() {
            log::info!("Link clicked: {}", url);
        }
    }

    fn render_enhanced_table(&mut self) {
        if self.current_table.rows.is_empty() {
            return;
        }

        self.ui.add_space(10.0);

        let max_cols = self.current_table.rows.iter().map(|row| row.len()).max().unwrap_or(0);

        if max_cols == 0 {
            return;
        }

        // Clone the table data to avoid borrowing issues
        let table_rows = self.current_table.rows.clone();
        let is_header_rows = self.current_table.is_header_row.clone();
        let column_alignments = self.current_table.column_alignments.clone();

        // Calculate table dimensions for border drawing
        let cell_padding = egui::vec2(8.0, 4.0);
        let min_cell_width = 80.0;

        // Get adaptive colors based on current UI theme
        let table_colors = Self::get_adaptive_table_colors(self.ui);

        // Use a frame to create table border
        egui::Frame::none()
            .stroke(egui::Stroke::new(1.0, table_colors.border))
            .inner_margin(egui::Margin::same(1.0))
            .show(self.ui, |ui| {
                // Create unique grid ID for each table
                let grid_id = format!("markdown_table_{}", self.table_counter);

                egui::Grid::new(grid_id)
                    .striped(false) // We'll handle striping manually
                    .spacing([0.0, 0.0]) // No spacing, we'll draw borders
                    .min_col_width(min_cell_width)
                    .show(ui, |ui| {
                        for (row_index, row) in table_rows.iter().enumerate() {
                            let is_header = is_header_rows.get(row_index).unwrap_or(&false);

                            for (col_index, cell) in row.iter().enumerate() {
                                if col_index >= max_cols {
                                    break;
                                }

                                // Create a frame for each cell with borders
                                let cell_frame = if *is_header {
                                    egui::Frame::none()
                                        .fill(table_colors.header_bg)
                                        .stroke(egui::Stroke::new(1.0, table_colors.border))
                                        .inner_margin(cell_padding)
                                } else if row_index % 2 == 0 {
                                    egui::Frame::none()
                                        .fill(table_colors.even_row_bg)
                                        .stroke(egui::Stroke::new(1.0, table_colors.border))
                                        .inner_margin(cell_padding)
                                } else {
                                    egui::Frame::none()
                                        .fill(table_colors.odd_row_bg)
                                        .stroke(egui::Stroke::new(1.0, table_colors.border))
                                        .inner_margin(cell_padding)
                                };

                                cell_frame.show(ui, |ui| {
                                    ui.set_min_width(min_cell_width);
                                    Self::render_table_cell_static(ui, cell, *is_header, col_index, &column_alignments, &table_colors);
                                });
                            }

                            // Fill empty cells with borders
                            for _ in row.len()..max_cols {
                                egui::Frame::none()
                                    .fill(table_colors.even_row_bg)
                                    .stroke(egui::Stroke::new(1.0, table_colors.border))
                                    .inner_margin(cell_padding)
                                    .show(ui, |ui| {
                                        ui.set_min_width(min_cell_width);
                                        ui.label("");
                                    });
                            }

                            ui.end_row();
                        }
                    });
            });

        self.ui.add_space(10.0);
        self.current_table = TableInfo::new();
    }

    /// 获取自适应的表格颜色配置
    fn get_adaptive_table_colors(ui: &egui::Ui) -> TableColors {
        let visuals = ui.visuals();

        // 检测当前是否为深色主题
        let is_dark_theme = visuals.dark_mode;

        if is_dark_theme {
            // 深色主题配色
            TableColors {
                border: Color32::from_rgb(80, 80, 80),           // 深灰色边框
                header_bg: Color32::from_rgb(60, 60, 60),        // 深灰色表头背景
                even_row_bg: Color32::from_rgb(45, 45, 45),      // 深色偶数行背景
                odd_row_bg: Color32::from_rgb(50, 50, 50),       // 稍浅的奇数行背景
                text: Color32::from_rgb(220, 220, 220),          // 浅色文字
                header_text: Color32::from_rgb(255, 255, 255),   // 白色表头文字
            }
        } else {
            // 浅色主题配色（原有配色）
            TableColors {
                border: Color32::from_rgb(200, 200, 200),        // 浅灰色边框
                header_bg: Color32::from_rgb(248, 249, 250),     // 浅灰色表头背景
                even_row_bg: Color32::from_rgb(255, 255, 255),   // 白色偶数行背景
                odd_row_bg: Color32::from_rgb(249, 249, 249),    // 浅灰色奇数行背景
                text: Color32::from_rgb(40, 40, 40),             // 深色文字
                header_text: Color32::from_rgb(40, 40, 40),      // 深色表头文字
            }
        }
    }

    fn render_table_cell_static(
        ui: &mut egui::Ui,
        cell: &MarkdownTableCell,
        is_header: bool,
        col_index: usize,
        column_alignments: &[Alignment],
        table_colors: &TableColors
    ) {
        let mut rich_text = egui::RichText::new(&cell.content);

        // Apply cell formatting with adaptive colors
        if is_header {
            rich_text = rich_text.strong().color(table_colors.header_text);
        } else {
            rich_text = rich_text.color(table_colors.text);
        }

        if cell.is_bold {
            rich_text = rich_text.strong();
        }

        if cell.is_italic {
            rich_text = rich_text.italics();
        }

        if cell.is_code {
            // Use adaptive background for code in tables
            let code_bg = if ui.visuals().dark_mode {
                Color32::from_rgb(70, 70, 70)  // 深色主题下的代码背景
            } else {
                Color32::from_rgb(245, 245, 245)  // 浅色主题下的代码背景
            };
            rich_text = rich_text.monospace().background_color(code_bg);
        }

        if cell.is_link {
            // Use adaptive link color
            let link_color = if ui.visuals().dark_mode {
                Color32::from_rgb(100, 150, 255)  // 深色主题下的链接颜色
            } else {
                Color32::from_rgb(0, 102, 204)    // 浅色主题下的链接颜色
            };
            rich_text = rich_text.color(link_color).underline();
        }

        // Apply column alignment if available
        let alignment = column_alignments.get(col_index).unwrap_or(&Alignment::None);

        match alignment {
            Alignment::Left | Alignment::None => {
                if cell.is_link && !cell.link_url.is_empty() {
                    if ui.add(egui::Label::new(rich_text).sense(egui::Sense::click())).clicked() {
                        log::info!("Table link clicked: {}", cell.link_url);
                    }
                } else {
                    ui.add(egui::Label::new(rich_text));
                }
            },
            Alignment::Center => {
                ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                    if cell.is_link && !cell.link_url.is_empty() {
                        if ui.add(egui::Label::new(rich_text).sense(egui::Sense::click())).clicked() {
                            log::info!("Table link clicked: {}", cell.link_url);
                        }
                    } else {
                        ui.add(egui::Label::new(rich_text));
                    }
                });
            },
            Alignment::Right => {
                ui.with_layout(egui::Layout::top_down(egui::Align::Max), |ui| {
                    if cell.is_link && !cell.link_url.is_empty() {
                        if ui.add(egui::Label::new(rich_text).sense(egui::Sense::click())).clicked() {
                            log::info!("Table link clicked: {}", cell.link_url);
                        }
                    } else {
                        ui.add(egui::Label::new(rich_text));
                    }
                });
            },
        }
    }
}

/// Append text to the layout job with appropriate formatting
fn append_text(
    job: &mut LayoutJob,
    text: &str,
    is_heading: bool,
    heading_level: u8,
    is_bold: bool,
    is_italic: bool,
    is_code: bool,
) {
    let mut format = TextFormat::default();

    // Apply formatting
    if is_heading {
        format.font_id.size = match heading_level {
            1 => 24.0,
            2 => 20.0,
            3 => 18.0,
            4 => 16.0,
            _ => 14.0,
        };
        format.color = Color32::from_rgb(60, 120, 216); // Blue for headings
        // 使用粗体字体
    }

    if is_bold {
        // 使用粗体字体
    }

    if is_italic {
        format.italics = true;
    }

    if is_code {
        format.font_id.family = FontFamily::Monospace;
        format.background = Color32::from_rgb(240, 240, 240);
    }

    job.append(text, 0.0, format);
}

/// Append a link to the layout job
fn append_link(job: &mut LayoutJob, text: &str, _url: &str) {
    let mut format = TextFormat::default();
    format.color = Color32::from_rgb(0, 102, 204); // Blue for links
    format.underline = egui::Stroke::new(1.0, Color32::from_rgb(0, 102, 204));

    job.append(text, 0.0, format);
}

impl<'a> MarkdownRenderer<'a> {
    /// Render a code block with syntax highlighting and proper styling
    fn render_code_block(&mut self) {
        if self.code_block_content.is_empty() {
            return;
        }

        let code_content = self.code_block_content.clone();
        let code_language = self.code_block_language.clone();

        // Check if this is a Mermaid diagram
        if code_language.to_lowercase() == "mermaid" {
            self.mermaid_renderer.render_diagram(self.ui, &code_content, self.font_family.as_deref());
            return;
        }

        // Check if this is an SVG test
        if code_language.to_lowercase() == "svg-test" || code_language.to_lowercase() == "svgtest" {
            self.svg_test_renderer.render_test_svg(self.ui, self.font_family.as_deref());
            return;
        }

        // Add some spacing before the code block
        self.ui.add_space(8.0);

        // Create a frame for the code block
        let frame = egui::Frame::none()
            .fill(Color32::from_rgb(45, 45, 45)) // Dark background
            .stroke(egui::Stroke::new(1.0, Color32::from_rgb(80, 80, 80))) // Border
            .rounding(egui::Rounding::same(6.0)) // Rounded corners
            .inner_margin(egui::Margin::same(12.0)); // Padding

        frame.show(self.ui, |ui| {
            // Add language label if available
            if !code_language.is_empty() {
                ui.horizontal(|ui| {
                    ui.add_space(4.0);
                    ui.label(
                        egui::RichText::new(&code_language)
                            .size(11.0)
                            .color(Color32::from_rgb(150, 150, 150))
                            .monospace()
                    );
                });
                ui.add_space(6.0);
            }

            // Render the code with syntax highlighting
            render_highlighted_code_static(ui, &code_content, &code_language);
        });

        // Add some spacing after the code block
        self.ui.add_space(8.0);
    }

    /// Render code with basic syntax highlighting
    fn render_highlighted_code(&self, ui: &mut egui::Ui, code: &str, language: &str) {
        render_highlighted_code_static(ui, code, language);
    }
}

/// Static function to render code with basic syntax highlighting
fn render_highlighted_code_static(ui: &mut egui::Ui, code: &str, language: &str) {
        let mut layout_job = LayoutJob::default();

        // Define colors for syntax highlighting
        let default_color = Color32::from_rgb(220, 220, 220);
        let keyword_color = Color32::from_rgb(86, 156, 214);  // Blue
        let string_color = Color32::from_rgb(206, 145, 120);  // Orange
        let comment_color = Color32::from_rgb(106, 153, 85);  // Green
        let number_color = Color32::from_rgb(181, 206, 168);  // Light green
        let function_color = Color32::from_rgb(220, 220, 170); // Yellow

        // Basic syntax highlighting based on language
        match language.to_lowercase().as_str() {
            "rust" => highlight_rust_code(&mut layout_job, code, keyword_color, string_color, comment_color, number_color, function_color, default_color),
            "python" => highlight_python_code(&mut layout_job, code, keyword_color, string_color, comment_color, number_color, function_color, default_color),
            "javascript" | "js" => highlight_js_code(&mut layout_job, code, keyword_color, string_color, comment_color, number_color, function_color, default_color),
            "json" => highlight_json_code(&mut layout_job, code, keyword_color, string_color, comment_color, number_color, default_color),
            _ => {
                // Default monospace rendering without highlighting
                let format = egui::TextFormat {
                    font_id: egui::FontId::monospace(13.0),
                    color: default_color,
                    ..Default::default()
                };
                layout_job.append(code, 0.0, format);
            }
        }

        ui.add(egui::Label::new(layout_job));
    }

/// Highlight Rust code
fn highlight_rust_code(layout_job: &mut LayoutJob, code: &str, keyword_color: Color32, string_color: Color32, comment_color: Color32, number_color: Color32, function_color: Color32, default_color: Color32) {
    let rust_keywords = [
        "fn", "let", "mut", "const", "static", "if", "else", "match", "for", "while", "loop",
        "break", "continue", "return", "struct", "enum", "impl", "trait", "pub", "use", "mod",
        "crate", "super", "self", "Self", "where", "async", "await", "move", "ref", "type",
        "unsafe", "extern", "dyn", "Box", "Vec", "String", "Option", "Result", "Some", "None",
        "Ok", "Err", "true", "false"
    ];

    highlight_code_generic(layout_job, code, &rust_keywords, "//", keyword_color, string_color, comment_color, number_color, function_color, default_color);
}

/// Highlight Python code
fn highlight_python_code(layout_job: &mut LayoutJob, code: &str, keyword_color: Color32, string_color: Color32, comment_color: Color32, number_color: Color32, function_color: Color32, default_color: Color32) {
    let python_keywords = [
        "def", "class", "if", "elif", "else", "for", "while", "try", "except", "finally",
        "with", "as", "import", "from", "return", "yield", "lambda", "and", "or", "not",
        "in", "is", "True", "False", "None", "pass", "break", "continue", "global", "nonlocal",
        "async", "await", "print", "len", "range", "str", "int", "float", "list", "dict", "set"
    ];

    highlight_code_generic(layout_job, code, &python_keywords, "#", keyword_color, string_color, comment_color, number_color, function_color, default_color);
}

/// Highlight JavaScript code
fn highlight_js_code(layout_job: &mut LayoutJob, code: &str, keyword_color: Color32, string_color: Color32, comment_color: Color32, number_color: Color32, function_color: Color32, default_color: Color32) {
    let js_keywords = [
        "function", "var", "let", "const", "if", "else", "for", "while", "do", "switch",
        "case", "default", "break", "continue", "return", "try", "catch", "finally",
        "throw", "new", "this", "typeof", "instanceof", "true", "false", "null", "undefined",
        "async", "await", "class", "extends", "super", "import", "export", "from", "console"
    ];

    highlight_code_generic(layout_job, code, &js_keywords, "//", keyword_color, string_color, comment_color, number_color, function_color, default_color);
}

/// Highlight JSON code
fn highlight_json_code(layout_job: &mut LayoutJob, code: &str, keyword_color: Color32, string_color: Color32, _comment_color: Color32, number_color: Color32, default_color: Color32) {
    let chars: Vec<char> = code.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        let ch = chars[i];

        match ch {
            '"' => {
                // String
                let start = i;
                i += 1;
                while i < chars.len() && chars[i] != '"' {
                    if chars[i] == '\\' && i + 1 < chars.len() {
                        i += 2; // Skip escaped character
                    } else {
                        i += 1;
                    }
                }
                if i < chars.len() {
                    i += 1; // Include closing quote
                }

                let text: String = chars[start..i].iter().collect();
                let format = egui::TextFormat {
                    font_id: egui::FontId::monospace(13.0),
                    color: string_color,
                    ..Default::default()
                };
                layout_job.append(&text, 0.0, format);
            },
            '0'..='9' | '-' => {
                // Number
                let start = i;
                if chars[i] == '-' {
                    i += 1;
                }
                while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '.' || chars[i] == 'e' || chars[i] == 'E' || chars[i] == '+' || chars[i] == '-') {
                    i += 1;
                }

                let text: String = chars[start..i].iter().collect();
                let format = egui::TextFormat {
                    font_id: egui::FontId::monospace(13.0),
                    color: number_color,
                    ..Default::default()
                };
                layout_job.append(&text, 0.0, format);
            },
            't' | 'f' | 'n' => {
                // Keywords: true, false, null
                let start = i;
                while i < chars.len() && chars[i].is_ascii_alphabetic() {
                    i += 1;
                }

                let text: String = chars[start..i].iter().collect();
                let color = if text == "true" || text == "false" || text == "null" {
                    keyword_color
                } else {
                    default_color
                };

                let format = egui::TextFormat {
                    font_id: egui::FontId::monospace(13.0),
                    color,
                    ..Default::default()
                };
                layout_job.append(&text, 0.0, format);
            },
            _ => {
                // Default character
                let format = egui::TextFormat {
                    font_id: egui::FontId::monospace(13.0),
                    color: default_color,
                    ..Default::default()
                };
                layout_job.append(&ch.to_string(), 0.0, format);
                i += 1;
            }
        }
    }
}

/// Generic code highlighting function
fn highlight_code_generic(layout_job: &mut LayoutJob, code: &str, keywords: &[&str], comment_prefix: &str, keyword_color: Color32, string_color: Color32, comment_color: Color32, number_color: Color32, function_color: Color32, default_color: Color32) {
    let lines: Vec<&str> = code.lines().collect();

    for (line_idx, line) in lines.iter().enumerate() {
        if line_idx > 0 {
            // Add newline
            let format = egui::TextFormat {
                font_id: egui::FontId::monospace(13.0),
                color: default_color,
                ..Default::default()
            };
            layout_job.append("\n", 0.0, format);
        }

        // Check if line is a comment
        if line.trim_start().starts_with(comment_prefix) {
            let format = egui::TextFormat {
                font_id: egui::FontId::monospace(13.0),
                color: comment_color,
                ..Default::default()
            };
            layout_job.append(line, 0.0, format);
            continue;
        }

        // Parse line for tokens
        let chars: Vec<char> = line.chars().collect();
        let mut i = 0;

        while i < chars.len() {
            let ch = chars[i];

            if ch.is_whitespace() {
                // Whitespace
                let format = egui::TextFormat {
                    font_id: egui::FontId::monospace(13.0),
                    color: default_color,
                    ..Default::default()
                };
                layout_job.append(&ch.to_string(), 0.0, format);
                i += 1;
            } else if ch == '"' || ch == '\'' {
                // String literal
                let quote = ch;
                let start = i;
                i += 1;
                while i < chars.len() && chars[i] != quote {
                    if chars[i] == '\\' && i + 1 < chars.len() {
                        i += 2; // Skip escaped character
                    } else {
                        i += 1;
                    }
                }
                if i < chars.len() {
                    i += 1; // Include closing quote
                }

                let text: String = chars[start..i].iter().collect();
                let format = egui::TextFormat {
                    font_id: egui::FontId::monospace(13.0),
                    color: string_color,
                    ..Default::default()
                };
                layout_job.append(&text, 0.0, format);
            } else if ch.is_ascii_digit() {
                // Number
                let start = i;
                while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '.' || chars[i] == '_') {
                    i += 1;
                }

                let text: String = chars[start..i].iter().collect();
                let format = egui::TextFormat {
                    font_id: egui::FontId::monospace(13.0),
                    color: number_color,
                    ..Default::default()
                };
                layout_job.append(&text, 0.0, format);
            } else if ch.is_ascii_alphabetic() || ch == '_' {
                // Identifier or keyword
                let start = i;
                while i < chars.len() && (chars[i].is_ascii_alphanumeric() || chars[i] == '_') {
                    i += 1;
                }

                let text: String = chars[start..i].iter().collect();

                // Check if it's a keyword
                let color = if keywords.contains(&text.as_str()) {
                    keyword_color
                } else if i < chars.len() && chars[i] == '(' {
                    // Likely a function call
                    function_color
                } else {
                    default_color
                };

                let format = egui::TextFormat {
                    font_id: egui::FontId::monospace(13.0),
                    color,
                    ..Default::default()
                };
                layout_job.append(&text, 0.0, format);
            } else {
                // Other characters (operators, punctuation, etc.)
                let format = egui::TextFormat {
                    font_id: egui::FontId::monospace(13.0),
                    color: default_color,
                    ..Default::default()
                };
                layout_job.append(&ch.to_string(), 0.0, format);
                i += 1;
            }
        }
    }
}




