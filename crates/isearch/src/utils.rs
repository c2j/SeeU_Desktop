/// Utility functions for the isearch crate

/// Highlight search terms in text for display (simple text version)
///
/// This function takes a text snippet and a list of search terms,
/// and returns a formatted string with the terms highlighted using text markers.
/// Uses character-based indexing to handle Unicode correctly.
pub fn highlight_search_terms(text: &str, terms: &[String]) -> String {
    if terms.is_empty() {
        return text.to_string();
    }

    // Convert to character vector for safe indexing
    let chars: Vec<char> = text.chars().collect();
    let text_lower = text.to_lowercase();
    let text_lower_chars: Vec<char> = text_lower.chars().collect();

    // Sort terms by length (longest first) to avoid partial replacements
    let mut sorted_terms = terms.to_vec();
    sorted_terms.sort_by(|a, b| b.len().cmp(&a.len()));

    let mut highlighted_ranges = Vec::new();

    // Find all matches first
    for term in &sorted_terms {
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

    // Build result with highlights
    let mut result = String::new();
    let mut last_end = 0;

    for (start, end) in highlighted_ranges {
        // Add text before highlight
        result.extend(chars[last_end..start].iter());

        // Add highlighted text
        result.push_str("【");
        result.extend(chars[start..end].iter());
        result.push_str("】");

        last_end = end;
    }

    // Add remaining text
    result.extend(chars[last_end..].iter());

    result
}

/// Create highlighted rich text for egui display
///
/// This function creates a LayoutJob with proper highlighting for search terms.
/// It provides visual highlighting with colors instead of text markers.
pub fn create_highlighted_rich_text(text: &str, terms: &[String]) -> eframe::egui::text::LayoutJob {
    use eframe::egui::{text::LayoutJob, Color32, FontId, TextFormat};

    if terms.is_empty() {
        let mut job = LayoutJob::default();
        job.append(text, 0.0, TextFormat::default());
        return job;
    }

    // Convert to character vector for safe indexing
    let chars: Vec<char> = text.chars().collect();
    let text_lower = text.to_lowercase();
    let text_lower_chars: Vec<char> = text_lower.chars().collect();

    // Sort terms by length (longest first) to avoid partial replacements
    let mut sorted_terms = terms.to_vec();
    sorted_terms.sort_by(|a, b| b.len().cmp(&a.len()));

    let mut highlighted_ranges = Vec::new();

    // Find all matches first
    for term in &sorted_terms {
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
    let mut job = LayoutJob::default();
    let mut last_end = 0;

    // Default text format
    let normal_format = TextFormat {
        font_id: FontId::default(),
        color: Color32::from_gray(200), // Light gray for normal text
        ..Default::default()
    };

    // Highlighted text format
    let highlight_format = TextFormat {
        font_id: FontId::default(),
        color: Color32::from_rgb(255, 255, 0), // Yellow text
        background: Color32::from_rgb(80, 80, 0), // Dark yellow background
        ..Default::default()
    };

    for (start, end) in highlighted_ranges {
        // Add normal text before highlight
        if start > last_end {
            let normal_text: String = chars[last_end..start].iter().collect();
            job.append(&normal_text, 0.0, normal_format.clone());
        }

        // Add highlighted text
        let highlighted_text: String = chars[start..end].iter().collect();
        job.append(&highlighted_text, 0.0, highlight_format.clone());

        last_end = end;
    }

    // Add remaining normal text
    if last_end < chars.len() {
        let remaining_text: String = chars[last_end..].iter().collect();
        job.append(&remaining_text, 0.0, normal_format);
    }

    job
}

/// Extract search terms from a query string
///
/// This function parses a search query and extracts meaningful search terms,
/// filtering out operators and very short terms. Supports quoted phrases.
pub fn extract_search_terms(query: &str) -> Vec<String> {
    let mut terms = Vec::new();
    let mut current_term = String::new();
    let mut in_quotes = false;
    let mut quote_char = '"';

    let chars: Vec<char> = query.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        let ch = chars[i];

        match ch {
            '"' | '\'' if !in_quotes => {
                // Start of quoted phrase
                in_quotes = true;
                quote_char = ch;
            },
            ch if in_quotes && ch == quote_char => {
                // End of quoted phrase
                in_quotes = false;
                if !current_term.trim().is_empty() {
                    let term = current_term.trim().to_lowercase();
                    if term.len() > 2 {
                        terms.push(term);
                    }
                }
                current_term.clear();
            },
            ' ' | '\t' | '\n' if !in_quotes => {
                // End of unquoted term
                if !current_term.trim().is_empty() {
                    let term = current_term.trim().to_lowercase();
                    // Filter out operators and short terms
                    if !term.starts_with("filetype:") &&
                       !term.starts_with("filename:") &&
                       !term.starts_with("+") &&
                       !term.starts_with("-") &&
                       term.len() > 2 {
                        terms.push(term);
                    }
                }
                current_term.clear();
            },
            _ => {
                current_term.push(ch);
            }
        }

        i += 1;
    }

    // Handle last term
    if !current_term.trim().is_empty() {
        let term = current_term.trim().to_lowercase();
        if in_quotes {
            // Unclosed quote, treat as phrase if long enough
            if term.len() > 2 {
                terms.push(term);
            }
        } else {
            // Regular term filtering
            if !term.starts_with("filetype:") &&
               !term.starts_with("filename:") &&
               !term.starts_with("+") &&
               !term.starts_with("-") &&
               term.len() > 2 {
                terms.push(term);
            }
        }
    }

    terms
}

/// Truncate text with ellipsis if it exceeds the maximum length
///
/// This function handles both ASCII and Unicode text correctly,
/// ensuring that the truncated text doesn't exceed the maximum length
/// and adds an ellipsis at the end if truncation occurs.
pub fn truncate_with_ellipsis(text: &str, max_length: usize) -> String {
    let char_count = text.chars().count();

    if char_count <= max_length {
        return text.to_string();
    }

    // Handle edge case where max_length is very small
    if max_length == 0 {
        return String::new();
    } else if max_length == 1 {
        return ".".to_string();
    } else if max_length == 2 {
        return "..".to_string();
    } else if max_length == 3 {
        // Special case: take 1 character + "..."
        let first_char: String = text.chars().take(1).collect();
        return format!("{}...", first_char);
    }

    // For max_length >= 4, truncate to max_length - 3 to make room for the ellipsis
    let truncated: String = text.chars().take(max_length - 3).collect();
    format!("{}...", truncated)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_truncate_with_ellipsis() {
        // Test with ASCII text
        assert_eq!(truncate_with_ellipsis("Hello", 10), "Hello");
        assert_eq!(truncate_with_ellipsis("Hello World", 10), "Hello W...");

        // Test with Unicode text (Chinese characters)
        assert_eq!(truncate_with_ellipsis("你好", 10), "你好");
        assert_eq!(truncate_with_ellipsis("你好世界", 3), "你...");

        // Test with mixed text
        // "Hello 世界" has 8 characters, so with max_length=8 it should not be truncated
        assert_eq!(truncate_with_ellipsis("Hello 世界", 8), "Hello 世界");
        // But with max_length=7 it should be truncated
        assert_eq!(truncate_with_ellipsis("Hello 世界", 7), "Hell...");
    }

    #[test]
    fn test_extract_search_terms() {
        // Test basic term extraction
        assert_eq!(extract_search_terms("hello world"), vec!["hello", "world"]);

        // Test filtering of short terms
        assert_eq!(extract_search_terms("hello a world"), vec!["hello", "world"]);

        // Test filtering of operators
        assert_eq!(extract_search_terms("hello filetype:txt world"), vec!["hello", "world"]);
        assert_eq!(extract_search_terms("hello +world -test"), vec!["hello"]);

        // Test quote removal
        assert_eq!(extract_search_terms("\"hello world\" test"), vec!["hello world", "test"]);

        // Test case conversion
        assert_eq!(extract_search_terms("Hello WORLD"), vec!["hello", "world"]);
    }

    #[test]
    fn test_highlight_search_terms() {
        let terms = vec!["hello".to_string(), "world".to_string()];

        // Test basic highlighting
        let result = highlight_search_terms("Hello world!", &terms);
        assert!(result.contains("【Hello】"));
        assert!(result.contains("【world】"));

        // Test case insensitive matching
        let result = highlight_search_terms("HELLO WORLD", &terms);
        assert!(result.contains("【HELLO】"));
        assert!(result.contains("【WORLD】"));

        // Test no terms
        let result = highlight_search_terms("Hello world", &vec![]);
        assert_eq!(result, "Hello world");

        // Test overlapping terms (longer terms should be prioritized)
        let terms = vec!["hello".to_string(), "hello world".to_string()];
        let result = highlight_search_terms("Hello world test", &terms);
        assert!(result.contains("【Hello world】"));
    }

    #[test]
    fn test_unicode_character_handling() {
        // Test Chinese characters
        let terms = vec!["澳大利亚".to_string(), "乍得".to_string()];
        let text = "反对 **澳大利亚** 澳大利亚 反对 **玻利维亚** 南美洲 反对 **巴西** 南美洲 反对 **中非** 非洲 反对 **乍得** 非洲";

        let result = highlight_search_terms(text, &terms);
        assert!(result.contains("【澳大利亚】"));
        assert!(result.contains("【乍得】"));

        // Test mixed Chinese and English
        let terms = vec!["rust".to_string(), "安全".to_string()];
        let text = "Rust是一种系统编程语言，专注于安全、速度和并发";

        let result = highlight_search_terms(text, &terms);
        assert!(result.contains("【Rust】"));
        assert!(result.contains("【安全】"));

        // Test emoji and special characters
        let terms = vec!["测试".to_string()];
        let text = "这是一个🎉测试🚀文本";

        let result = highlight_search_terms(text, &terms);
        assert!(result.contains("【测试】"));
    }
}
