/// Utility functions for the isearch crate

/// Truncate text with ellipsis if it exceeds the maximum length
/// 
/// This function handles both ASCII and Unicode text correctly,
/// ensuring that the truncated text doesn't exceed the maximum length
/// and adds an ellipsis at the end if truncation occurs.
pub fn truncate_with_ellipsis(text: &str, max_length: usize) -> String {
    if text.chars().count() <= max_length {
        return text.to_string();
    }
    
    // Truncate to max_length - 3 to make room for the ellipsis
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
        assert_eq!(truncate_with_ellipsis("Hello 世界", 8), "Hello 世...");
    }
}
