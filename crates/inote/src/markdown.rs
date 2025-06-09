use pulldown_cmark::{Parser, Options, html};
use eframe::egui::{self, TextFormat, Color32, Ui, FontFamily, FontId};
use eframe::egui::text::{LayoutJob, TextWrapping};

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
    // Set up options and parser
    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_FOOTNOTES);
    options.insert(Options::ENABLE_TASKLISTS);

    let parser = Parser::new_ext(markdown, options);

    // Create a layout job for the markdown
    let mut job = LayoutJob::default();
    job.wrap = TextWrapping {
        max_width: ui.available_width(),
        ..Default::default()
    };

    // Process markdown events
    let mut current_text = String::new();
    let mut is_heading = false;
    let mut heading_level = 0;
    let mut is_bold = false;
    let mut is_italic = false;
    let mut is_code = false;
    let mut is_link = false;
    let mut link_url = String::new();

    for event in parser {
        use pulldown_cmark::Event::*;
        use pulldown_cmark::Tag::*;

        match event {
            Start(tag) => {
                // Flush any accumulated text
                if !current_text.is_empty() {
                    append_text(&mut job, &current_text, is_heading, heading_level, is_bold, is_italic, is_code);
                    current_text.clear();
                }

                match tag {
                    Heading(level, _, _) => {
                        is_heading = true;
                        heading_level = level as u8;
                    },
                    Emphasis => is_italic = true,
                    Strong => is_bold = true,
                    CodeBlock(_) => is_code = true,
                    Link(_, url, _) => {
                        is_link = true;
                        link_url = url.to_string();
                    },
                    _ => {}
                }
            },
            End(tag) => {
                // Flush any accumulated text
                if !current_text.is_empty() {
                    if is_link {
                        append_link(&mut job, &current_text, &link_url);
                    } else {
                        append_text(&mut job, &current_text, is_heading, heading_level, is_bold, is_italic, is_code);
                    }
                    current_text.clear();
                }

                match tag {
                    Heading(_, _, _) => {
                        is_heading = false;
                        heading_level = 0;
                        job.append("\n\n", 0.0, TextFormat::default());
                    },
                    Paragraph => {
                        job.append("\n\n", 0.0, TextFormat::default());
                    },
                    Emphasis => is_italic = false,
                    Strong => is_bold = false,
                    CodeBlock(_) => is_code = false,
                    Link(_, _, _) => {
                        is_link = false;
                        link_url.clear();
                    },
                    _ => {}
                }
            },
            Text(text) => {
                current_text.push_str(&text);
            },
            Code(text) => {
                is_code = true;
                current_text.push_str(&text);
                is_code = false;
            },
            SoftBreak => {
                current_text.push(' ');
            },
            HardBreak => {
                if !current_text.is_empty() {
                    append_text(&mut job, &current_text, is_heading, heading_level, is_bold, is_italic, is_code);
                    current_text.clear();
                }
                job.append("\n", 0.0, TextFormat::default());
            },
            _ => {}
        }
    }

    // Flush any remaining text
    if !current_text.is_empty() {
        append_text(&mut job, &current_text, is_heading, heading_level, is_bold, is_italic, is_code);
    }

    // Display the layout job
    ui.add(egui::Label::new(job));
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
fn append_link(job: &mut LayoutJob, text: &str, url: &str) {
    let mut format = TextFormat::default();
    format.color = Color32::from_rgb(0, 102, 204); // Blue for links
    format.underline = egui::Stroke::new(1.0, Color32::from_rgb(0, 102, 204));

    job.append(text, 0.0, format);
}
