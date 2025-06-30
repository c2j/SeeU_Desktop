use arboard::{Clipboard, ImageData};
use std::error::Error;
use std::fmt;
use regex::Regex;

/// Clipboard manager for handling rich text conversion
pub struct ClipboardManager {
    clipboard: Clipboard,
}

/// Error types for clipboard operations
#[derive(Debug)]
pub enum ClipboardError {
    AccessError(String),
    ConversionError(String),
    UnsupportedFormat,
}

impl fmt::Display for ClipboardError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ClipboardError::AccessError(msg) => write!(f, "Clipboard access error: {}", msg),
            ClipboardError::ConversionError(msg) => write!(f, "Conversion error: {}", msg),
            ClipboardError::UnsupportedFormat => write!(f, "Unsupported clipboard format"),
        }
    }
}

impl Error for ClipboardError {}

impl ClipboardManager {
    /// Create a new clipboard manager
    pub fn new() -> Result<Self, ClipboardError> {
        let clipboard = Clipboard::new()
            .map_err(|e| ClipboardError::AccessError(e.to_string()))?;
        
        Ok(Self { clipboard })
    }

    /// Get plain text from clipboard
    pub fn get_text(&mut self) -> Result<String, ClipboardError> {
        self.clipboard
            .get_text()
            .map_err(|e| ClipboardError::AccessError(e.to_string()))
    }

    /// Set plain text to clipboard
    pub fn set_text(&mut self, text: &str) -> Result<(), ClipboardError> {
        self.clipboard
            .set_text(text)
            .map_err(|e| ClipboardError::AccessError(e.to_string()))
    }

    /// Get HTML content from clipboard and convert to Markdown
    pub fn get_html_as_markdown(&mut self) -> Result<Option<String>, ClipboardError> {
        // Strategy 1: Try to get text and analyze its format
        if let Ok(text) = self.get_text() {
            if text.trim().is_empty() {
                return Ok(None);
            }

            // Check if the text contains HTML tags
            if self.is_html_content(&text) {
                log::info!("Detected HTML content in clipboard");
                let markdown = self.convert_html_to_markdown(&text)?;
                return Ok(Some(markdown));
            }

            // Check if the text contains rich formatting patterns
            if self.is_rich_text_content(&text) {
                log::info!("Detected rich text patterns in clipboard");
                let markdown = self.convert_rich_text_to_markdown(&text);
                return Ok(Some(markdown));
            }

            // Check if the text looks like it might be from a formatted source
            if self.might_be_formatted_content(&text) {
                log::info!("Detected potentially formatted content in clipboard");
                let markdown = self.enhance_plain_text_to_markdown(&text);
                return Ok(Some(markdown));
            }

            // Return as plain text
            Ok(Some(text))
        } else {
            Ok(None)
        }
    }

    /// Check if content is HTML
    fn is_html_content(&self, text: &str) -> bool {
        let text = text.trim();

        // Check for HTML tags
        let html_patterns = [
            r"<html[^>]*>",
            r"<head[^>]*>",
            r"<body[^>]*>",
            r"<div[^>]*>",
            r"<p[^>]*>",
            r"<span[^>]*>",
            r"<h[1-6][^>]*>",
            r"<table[^>]*>",
            r"<tr[^>]*>",
            r"<td[^>]*>",
            r"<th[^>]*>",
            r"<ul[^>]*>",
            r"<ol[^>]*>",
            r"<li[^>]*>",
            r"<a[^>]*>",
            r"<img[^>]*>",
            r"<strong[^>]*>",
            r"<em[^>]*>",
            r"<b[^>]*>",
            r"<i[^>]*>",
        ];

        for pattern in &html_patterns {
            if let Ok(re) = Regex::new(pattern) {
                if re.is_match(text) {
                    return true;
                }
            }
        }

        // Also check for simple HTML structure
        (text.starts_with('<') && text.contains("</")) ||
        text.contains("&lt;") || text.contains("&gt;") ||
        text.contains("&amp;") || text.contains("&nbsp;")
    }

    /// Check if content contains rich text patterns
    fn is_rich_text_content(&self, text: &str) -> bool {
        // Look for patterns that suggest rich formatting
        let rich_patterns = [
            r"\*\*[^*]+\*\*",           // Bold markdown
            r"\*[^*]+\*",               // Italic markdown
            r"`[^`]+`",                 // Code markdown
            r"#{1,6}\s+",               // Headers
            r"^\s*[-*+]\s+",            // Unordered lists
            r"^\s*\d+\.\s+",            // Ordered lists
            r"^\s*>\s+",                // Blockquotes
            r"\[([^\]]+)\]\(([^)]+)\)", // Links
            r"!\[([^\]]*)\]\(([^)]+)\)", // Images
            r"\|.*\|",                  // Tables
        ];

        for pattern in &rich_patterns {
            if let Ok(re) = Regex::new(pattern) {
                if re.is_match(text) {
                    return true;
                }
            }
        }

        false
    }

    /// Check if content might be from a formatted source
    fn might_be_formatted_content(&self, text: &str) -> bool {
        let lines: Vec<&str> = text.lines().collect();

        // Check for patterns that suggest structured content
        let mut has_title_case = false;
        let mut has_bullet_points = false;
        let mut has_numbered_items = false;
        let mut has_indentation = false;
        let mut has_multiple_paragraphs = false;

        let mut paragraph_count = 0;
        let mut current_paragraph_lines = 0;

        for line in &lines {
            let trimmed = line.trim();

            if trimmed.is_empty() {
                if current_paragraph_lines > 0 {
                    paragraph_count += 1;
                    current_paragraph_lines = 0;
                }
                continue;
            }

            current_paragraph_lines += 1;

            // Check for title case (words starting with capital letters)
            if trimmed.len() < 100 && trimmed.split_whitespace().count() <= 10 {
                let words: Vec<&str> = trimmed.split_whitespace().collect();
                if words.len() >= 2 {
                    let title_case_count = words.iter()
                        .filter(|word| word.chars().next().map_or(false, |c| c.is_uppercase()))
                        .count();
                    if title_case_count >= words.len() / 2 {
                        has_title_case = true;
                    }
                }
            }

            // Check for bullet points
            if trimmed.starts_with("•") || trimmed.starts_with("·") ||
               trimmed.starts_with("‣") || trimmed.starts_with("▪") ||
               trimmed.starts_with("▫") || trimmed.starts_with("‒") {
                has_bullet_points = true;
            }

            // Check for numbered items
            if let Ok(re) = Regex::new(r"^\d+[\.\)]\s+") {
                if re.is_match(trimmed) {
                    has_numbered_items = true;
                }
            }

            // Check for indentation
            if line.starts_with("    ") || line.starts_with("\t") {
                has_indentation = true;
            }
        }

        if current_paragraph_lines > 0 {
            paragraph_count += 1;
        }

        has_multiple_paragraphs = paragraph_count > 1;

        // Return true if we found evidence of formatting
        has_title_case || has_bullet_points || has_numbered_items ||
        has_indentation || has_multiple_paragraphs
    }

    /// Convert HTML to Markdown
    fn convert_html_to_markdown(&self, html: &str) -> Result<String, ClipboardError> {
        // Clean up the HTML first
        let cleaned_html = self.clean_html(html);

        // Convert HTML to Markdown using html2md
        let markdown = html2md::parse_html(&cleaned_html);

        // Post-process the markdown to improve formatting
        let processed_markdown = self.post_process_markdown(&markdown);

        Ok(processed_markdown)
    }

    /// Convert rich text patterns to Markdown
    fn convert_rich_text_to_markdown(&self, text: &str) -> String {
        // If it's already markdown-like, just clean it up
        self.post_process_markdown(text)
    }

    /// Enhance plain text to Markdown by detecting structure
    fn enhance_plain_text_to_markdown(&self, text: &str) -> String {
        let lines: Vec<&str> = text.lines().collect();
        let mut result = Vec::new();
        let mut in_list = false;
        let mut list_indent_level = 0;

        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();

            if trimmed.is_empty() {
                result.push("".to_string());
                in_list = false;
                continue;
            }

            // Detect and convert headers (title-case lines that are short)
            if self.looks_like_header(trimmed, i, &lines) {
                if !result.is_empty() && !result.last().unwrap().is_empty() {
                    result.push("".to_string());
                }
                result.push(format!("## {}", trimmed));
                result.push("".to_string());
                in_list = false;
                continue;
            }

            // Detect and convert bullet points
            if let Some(converted) = self.convert_bullet_point(trimmed) {
                if !in_list {
                    if !result.is_empty() && !result.last().unwrap().is_empty() {
                        result.push("".to_string());
                    }
                    in_list = true;
                }
                result.push(converted);
                continue;
            }

            // Detect and convert numbered lists
            if let Some(converted) = self.convert_numbered_item(trimmed) {
                if !in_list {
                    if !result.is_empty() && !result.last().unwrap().is_empty() {
                        result.push("".to_string());
                    }
                    in_list = true;
                }
                result.push(converted);
                continue;
            }

            // Regular paragraph
            if in_list {
                result.push("".to_string());
                in_list = false;
            }

            result.push(trimmed.to_string());
        }

        self.post_process_markdown(&result.join("\n"))
    }

    /// Check if a line looks like a header
    fn looks_like_header(&self, line: &str, index: usize, all_lines: &[&str]) -> bool {
        // Short lines with title case
        if line.len() > 100 {
            return false;
        }

        let words: Vec<&str> = line.split_whitespace().collect();
        if words.len() < 2 || words.len() > 10 {
            return false;
        }

        // Check if most words start with capital letters
        let title_case_count = words.iter()
            .filter(|word| word.chars().next().map_or(false, |c| c.is_uppercase()))
            .count();

        if title_case_count < words.len() / 2 {
            return false;
        }

        // Check if followed by content (not another potential header)
        if index + 1 < all_lines.len() {
            let next_line = all_lines[index + 1].trim();
            if !next_line.is_empty() && !self.looks_like_header(next_line, index + 1, all_lines) {
                return true;
            }
        }

        false
    }

    /// Convert bullet point to markdown
    fn convert_bullet_point(&self, line: &str) -> Option<String> {
        let bullet_chars = ["•", "·", "‣", "▪", "▫", "‒", "○", "●"];

        for bullet in &bullet_chars {
            if line.starts_with(bullet) {
                let content = line.strip_prefix(bullet).unwrap_or(line).trim();
                return Some(format!("- {}", content));
            }
        }

        // Check for dash-style bullets
        if let Ok(re) = Regex::new(r"^[-–—]\s+(.+)") {
            if let Some(captures) = re.captures(line) {
                if let Some(content) = captures.get(1) {
                    return Some(format!("- {}", content.as_str()));
                }
            }
        }

        None
    }

    /// Convert numbered item to markdown
    fn convert_numbered_item(&self, line: &str) -> Option<String> {
        if let Ok(re) = Regex::new(r"^(\d+)[\.\)]\s+(.+)") {
            if let Some(captures) = re.captures(line) {
                if let (Some(num), Some(content)) = (captures.get(1), captures.get(2)) {
                    return Some(format!("{}. {}", num.as_str(), content.as_str()));
                }
            }
        }

        None
    }

    /// Clean up HTML content before conversion
    fn clean_html(&self, html: &str) -> String {
        let mut cleaned = html.to_string();
        
        // Remove common unwanted elements
        let unwanted_patterns = [
            r"<script[^>]*>.*?</script>",
            r"<style[^>]*>.*?</style>",
            r"<meta[^>]*>",
            r"<link[^>]*>",
            r"<!--.*?-->",
        ];
        
        for pattern in &unwanted_patterns {
            if let Ok(re) = Regex::new(pattern) {
                cleaned = re.replace_all(&cleaned, "").to_string();
            }
        }
        
        // Normalize whitespace
        cleaned = cleaned
            .lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>>()
            .join("\n");
        
        cleaned
    }

    /// Post-process markdown to improve formatting
    fn post_process_markdown(&self, markdown: &str) -> String {
        let mut processed = markdown.to_string();
        
        // Fix common formatting issues
        processed = processed
            .replace("\n\n\n", "\n\n")  // Remove excessive line breaks
            .replace("  \n", "\n")      // Remove trailing spaces
            .trim()
            .to_string();
        
        // Ensure proper spacing around headers
        let lines: Vec<&str> = processed.lines().collect();
        let mut result = Vec::new();
        
        for (i, line) in lines.iter().enumerate() {
            if line.starts_with('#') {
                // Add space before header if previous line is not empty
                if i > 0 && !lines[i - 1].is_empty() {
                    result.push("");
                }
                result.push(line);
                // Add space after header if next line exists and is not empty
                if i < lines.len() - 1 && !lines[i + 1].is_empty() {
                    result.push("");
                }
            } else {
                result.push(line);
            }
        }
        
        result.join("\n")
    }

    /// Check if clipboard contains rich text content
    pub fn has_rich_content(&mut self) -> bool {
        if let Ok(text) = self.get_text() {
            if text.trim().is_empty() {
                return false;
            }

            // Use our enhanced detection methods
            self.is_html_content(&text) ||
            self.is_rich_text_content(&text) ||
            self.might_be_formatted_content(&text)
        } else {
            false
        }
    }

    /// Paste rich text content as markdown
    pub fn paste_as_markdown(&mut self) -> Result<String, Box<dyn std::error::Error>> {
        // Try to get HTML content first
        if let Ok(text) = self.get_text() {
            if text.trim().is_empty() {
                return Err("No content in clipboard".into());
            }

            // For now, just return the plain text
            // In a full implementation, you would check for HTML content
            // and convert it to markdown using html2md
            Ok(text)
        } else {
            Err("Failed to get clipboard content".into())
        }
    }
}

impl Default for ClipboardManager {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| {
            // Fallback implementation if clipboard access fails
            Self {
                clipboard: Clipboard::new().expect("Failed to create clipboard"),
            }
        })
    }
}
