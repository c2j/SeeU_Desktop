use crate::egui_terminal::EguiTerminalSession;
use alacritty_terminal::grid::Dimensions;
use alacritty_terminal::term::cell::Cell;
use chrono::{DateTime, Utc};
use std::fmt::Write;
use uuid::Uuid;

/// Export format options
#[derive(Debug, Clone, PartialEq)]
pub enum ExportFormat {
    /// Plain text format
    PlainText,
    /// Markdown format with code blocks
    Markdown,
    /// HTML format with styling
    Html,
}

/// Export options
#[derive(Debug, Clone)]
pub struct ExportOptions {
    /// Export format
    pub format: ExportFormat,
    /// Include session metadata (title, timestamp, etc.)
    pub include_metadata: bool,
    /// Include empty lines
    pub include_empty_lines: bool,
    /// Maximum number of lines to export (None for all)
    pub max_lines: Option<usize>,
    /// Strip ANSI escape sequences
    pub strip_ansi: bool,
    /// Include line numbers
    pub include_line_numbers: bool,
}

impl Default for ExportOptions {
    fn default() -> Self {
        Self {
            format: ExportFormat::Markdown,
            include_metadata: true,
            include_empty_lines: false,
            max_lines: None,
            strip_ansi: true,
            include_line_numbers: false,
        }
    }
}

/// Export result containing the formatted content
#[derive(Debug, Clone)]
pub struct ExportResult {
    /// The formatted content
    pub content: String,
    /// Export metadata
    pub metadata: ExportMetadata,
}

/// Metadata about the export
#[derive(Debug, Clone)]
pub struct ExportMetadata {
    /// Session ID
    pub session_id: Uuid,
    /// Session title
    pub session_title: String,
    /// Export timestamp
    pub exported_at: DateTime<Utc>,
    /// Number of lines exported
    pub line_count: usize,
    /// Export format used
    pub format: ExportFormat,
}

/// Terminal content exporter
pub struct TerminalExporter;

impl TerminalExporter {
    /// Export terminal session content
    pub fn export_session(
        session: &EguiTerminalSession,
        options: &ExportOptions,
    ) -> Result<ExportResult, ExportError> {
        if let Some(ref backend) = session.backend {
            let content = backend.last_content();
            let grid = &content.grid;
            
            // Extract text content from the grid
            let lines = Self::extract_lines_from_grid(grid, options);
            
            // Format according to the specified format
            let formatted_content = match options.format {
                ExportFormat::PlainText => Self::format_as_plain_text(&lines, session, options),
                ExportFormat::Markdown => Self::format_as_markdown(&lines, session, options),
                ExportFormat::Html => Self::format_as_html(&lines, session, options),
            };

            let metadata = ExportMetadata {
                session_id: session.id,
                session_title: session.title.clone(),
                exported_at: Utc::now(),
                line_count: lines.len(),
                format: options.format.clone(),
            };

            Ok(ExportResult {
                content: formatted_content,
                metadata,
            })
        } else {
            Err(ExportError::NoBackend)
        }
    }

    /// Extract text lines from the terminal grid
    fn extract_lines_from_grid(
        grid: &alacritty_terminal::Grid<Cell>,
        options: &ExportOptions,
    ) -> Vec<String> {
        let mut lines = Vec::new();
        let total_lines = grid.screen_lines();

        // Determine how many lines to process
        let lines_to_process = if let Some(max) = options.max_lines {
            std::cmp::min(max, total_lines)
        } else {
            total_lines
        };

        for line_index in 0..lines_to_process {
            let mut line_text = String::new();

            // Extract characters from the line using grid indexing
            let line = alacritty_terminal::index::Line(line_index as i32);
            let row = &grid[line];

            for col_index in 0..grid.columns() {
                let column = alacritty_terminal::index::Column(col_index);
                let cell = &row[column];
                let ch = cell.c;

                if ch != ' ' || !line_text.is_empty() {
                    line_text.push(ch);
                }
            }

            // Trim trailing whitespace
            line_text = line_text.trim_end().to_string();

            // Skip empty lines if requested
            if !options.include_empty_lines && line_text.is_empty() {
                continue;
            }

            // Strip ANSI escape sequences if requested
            if options.strip_ansi {
                line_text = Self::strip_ansi_codes(&line_text);
            }

            lines.push(line_text);
        }

        lines
    }

    /// Strip ANSI escape sequences from text
    fn strip_ansi_codes(text: &str) -> String {
        // Simple ANSI escape sequence removal
        // This is a basic implementation - could be enhanced with a proper ANSI parser
        let mut result = String::new();
        let mut chars = text.chars().peekable();
        
        while let Some(ch) = chars.next() {
            if ch == '\x1b' {
                // Skip escape sequence
                if chars.peek() == Some(&'[') {
                    chars.next(); // consume '['
                    // Skip until we find a letter (end of escape sequence)
                    while let Some(next_ch) = chars.next() {
                        if next_ch.is_ascii_alphabetic() {
                            break;
                        }
                    }
                }
            } else {
                result.push(ch);
            }
        }
        
        result
    }

    /// Format content as plain text
    fn format_as_plain_text(
        lines: &[String],
        session: &EguiTerminalSession,
        options: &ExportOptions,
    ) -> String {
        let mut content = String::new();
        
        if options.include_metadata {
            writeln!(content, "Terminal Session: {}", session.title).unwrap();
            writeln!(content, "Session ID: {}", session.id).unwrap();
            // 转换为本地时间显示
            let local_created = session.created_at.with_timezone(&chrono::Local);
            let local_exported = Utc::now().with_timezone(&chrono::Local);
            writeln!(content, "Created: {}", local_created.format("%Y-%m-%d %H:%M:%S")).unwrap();
            writeln!(content, "Exported: {}", local_exported.format("%Y-%m-%d %H:%M:%S")).unwrap();
            writeln!(content, "Lines: {}", lines.len()).unwrap();
            writeln!(content, "{}", "=".repeat(50)).unwrap();
            writeln!(content).unwrap();
        }
        
        for (index, line) in lines.iter().enumerate() {
            if options.include_line_numbers {
                writeln!(content, "{:4}: {}", index + 1, line).unwrap();
            } else {
                writeln!(content, "{}", line).unwrap();
            }
        }
        
        content
    }

    /// Format content as Markdown
    fn format_as_markdown(
        lines: &[String],
        session: &EguiTerminalSession,
        options: &ExportOptions,
    ) -> String {
        let mut content = String::new();
        
        if options.include_metadata {
            writeln!(content, "# Terminal Session Export").unwrap();
            writeln!(content).unwrap();
            writeln!(content, "## Session Information").unwrap();
            writeln!(content).unwrap();
            writeln!(content, "- **Session Title**: {}", session.title).unwrap();
            writeln!(content, "- **Session ID**: `{}`", session.id).unwrap();
            // 转换为本地时间显示
            let local_created = session.created_at.with_timezone(&chrono::Local);
            let local_exported = Utc::now().with_timezone(&chrono::Local);
            writeln!(content, "- **Created**: {}", local_created.format("%Y-%m-%d %H:%M:%S")).unwrap();
            writeln!(content, "- **Exported**: {}", local_exported.format("%Y-%m-%d %H:%M:%S")).unwrap();
            writeln!(content, "- **Total Lines**: {}", lines.len()).unwrap();
            writeln!(content).unwrap();
            writeln!(content, "## Terminal Output").unwrap();
            writeln!(content).unwrap();
        }
        
        writeln!(content, "```bash").unwrap();
        for line in lines {
            writeln!(content, "{}", line).unwrap();
        }
        writeln!(content, "```").unwrap();
        
        content
    }

    /// Format content as HTML
    fn format_as_html(
        lines: &[String],
        session: &EguiTerminalSession,
        options: &ExportOptions,
    ) -> String {
        let mut content = String::new();
        
        writeln!(content, "<!DOCTYPE html>").unwrap();
        writeln!(content, "<html>").unwrap();
        writeln!(content, "<head>").unwrap();
        writeln!(content, "    <meta charset=\"UTF-8\">").unwrap();
        writeln!(content, "    <title>Terminal Session: {}</title>", session.title).unwrap();
        writeln!(content, "    <style>").unwrap();
        writeln!(content, "        body {{ font-family: 'Courier New', monospace; background: #1e1e1e; color: #d4d4d4; margin: 20px; }}").unwrap();
        writeln!(content, "        .metadata {{ background: #2d2d30; padding: 15px; border-radius: 5px; margin-bottom: 20px; }}").unwrap();
        writeln!(content, "        .terminal {{ background: #0c0c0c; padding: 15px; border-radius: 5px; overflow-x: auto; }}").unwrap();
        writeln!(content, "        .line {{ margin: 0; padding: 2px 0; }}").unwrap();
        writeln!(content, "        .line-number {{ color: #858585; margin-right: 10px; }}").unwrap();
        writeln!(content, "    </style>").unwrap();
        writeln!(content, "</head>").unwrap();
        writeln!(content, "<body>").unwrap();
        
        if options.include_metadata {
            writeln!(content, "    <div class=\"metadata\">").unwrap();
            writeln!(content, "        <h2>Terminal Session Export</h2>").unwrap();
            writeln!(content, "        <p><strong>Session Title:</strong> {}</p>", session.title).unwrap();
            writeln!(content, "        <p><strong>Session ID:</strong> {}</p>", session.id).unwrap();
            // 转换为本地时间显示
            let local_created = session.created_at.with_timezone(&chrono::Local);
            let local_exported = Utc::now().with_timezone(&chrono::Local);
            writeln!(content, "        <p><strong>Created:</strong> {}</p>", local_created.format("%Y-%m-%d %H:%M:%S")).unwrap();
            writeln!(content, "        <p><strong>Exported:</strong> {}</p>", local_exported.format("%Y-%m-%d %H:%M:%S")).unwrap();
            writeln!(content, "        <p><strong>Total Lines:</strong> {}</p>", lines.len()).unwrap();
            writeln!(content, "    </div>").unwrap();
        }
        
        writeln!(content, "    <div class=\"terminal\">").unwrap();
        for (index, line) in lines.iter().enumerate() {
            let escaped_line = html_escape::encode_text(line);
            if options.include_line_numbers {
                writeln!(content, "        <div class=\"line\"><span class=\"line-number\">{:4}:</span>{}</div>", index + 1, escaped_line).unwrap();
            } else {
                writeln!(content, "        <div class=\"line\">{}</div>", escaped_line).unwrap();
            }
        }
        writeln!(content, "    </div>").unwrap();
        writeln!(content, "</body>").unwrap();
        writeln!(content, "</html>").unwrap();
        
        content
    }
}

/// Export error types
#[derive(Debug, Clone)]
pub enum ExportError {
    /// No backend available for the session
    NoBackend,
    /// IO error during export
    IoError(String),
    /// Format error
    FormatError(String),
}

impl std::fmt::Display for ExportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExportError::NoBackend => write!(f, "No terminal backend available"),
            ExportError::IoError(msg) => write!(f, "IO error: {}", msg),
            ExportError::FormatError(msg) => write!(f, "Format error: {}", msg),
        }
    }
}

impl std::error::Error for ExportError {}
