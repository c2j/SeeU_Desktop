//! 中文文本处理模块

use crate::Result;

/// 中文文本处理器
pub struct ChineseTextProcessor {
    enable_traditional_conversion: bool,
    enable_punctuation_normalization: bool,
}

impl ChineseTextProcessor {
    /// 创建新的中文文本处理器
    pub fn new() -> Self {
        Self {
            enable_traditional_conversion: true,
            enable_punctuation_normalization: true,
        }
    }
    
    /// 预处理中文文本
    pub fn preprocess(&self, text: &str) -> String {
        let mut processed = text.to_string();
        
        // 1. 繁体转简体 (可选)
        if self.enable_traditional_conversion {
            processed = self.traditional_to_simplified(&processed);
        }
        
        // 2. 标点符号标准化
        if self.enable_punctuation_normalization {
            processed = self.normalize_punctuation(&processed);
        }
        
        // 3. 空白字符标准化
        processed = self.normalize_whitespace(&processed);
        
        // 4. 长度控制
        self.truncate_text(&processed, 400)
    }
    
    /// 繁体转简体
    fn traditional_to_simplified(&self, text: &str) -> String {
        // 简单的繁简转换映射
        let mappings = [
            ("機器學習", "机器学习"),
            ("資料庫", "数据库"),
            ("軟體", "软件"),
            ("電腦", "电脑"),
            ("網路", "网络"),
            ("檔案", "文件"),
            ("資訊", "信息"),
            ("運算", "运算"),
            ("處理", "处理"),
            ("開發", "开发"),
        ];
        
        let mut result = text.to_string();
        for (traditional, simplified) in mappings.iter() {
            result = result.replace(traditional, simplified);
        }
        result
    }
    
    /// 标点符号标准化
    fn normalize_punctuation(&self, text: &str) -> String {
        text.chars()
            .map(|c| match c {
                '，' => ',',
                '。' => '.',
                '？' => '?',
                '！' => '!',
                '：' => ':',
                '；' => ';',
                '\u{201C}' | '\u{201D}' => '"',
                '\u{2018}' | '\u{2019}' => '\'',
                '（' => '(',
                '）' => ')',
                '【' => '[',
                '】' => ']',
                _ => c,
            })
            .collect()
    }
    
    /// 空白字符标准化
    fn normalize_whitespace(&self, text: &str) -> String {
        text.split_whitespace().collect::<Vec<_>>().join(" ")
    }
    
    /// 文本截断
    fn truncate_text(&self, text: &str, max_chars: usize) -> String {
        if text.chars().count() <= max_chars {
            text.to_string()
        } else {
            text.chars().take(max_chars).collect()
        }
    }
    
    /// 检测文本语言
    pub fn detect_language(&self, text: &str) -> Language {
        let chinese_chars = text.chars().filter(|c| self.is_chinese_char(*c)).count();
        let total_chars = text.chars().filter(|c| c.is_alphabetic()).count();
        
        if total_chars == 0 {
            return Language::Unknown;
        }
        
        let chinese_ratio = chinese_chars as f64 / total_chars as f64;
        
        if chinese_ratio > 0.5 {
            Language::Chinese
        } else if chinese_ratio > 0.1 {
            Language::Mixed
        } else {
            Language::English
        }
    }
    
    /// 判断是否为中文字符
    fn is_chinese_char(&self, c: char) -> bool {
        matches!(c as u32, 0x4E00..=0x9FFF | 0x3400..=0x4DBF | 0x20000..=0x2A6DF)
    }
}

/// 语言类型
#[derive(Debug, Clone, PartialEq)]
pub enum Language {
    Chinese,
    English,
    Mixed,
    Unknown,
}

impl Default for ChineseTextProcessor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_traditional_to_simplified() {
        let processor = ChineseTextProcessor::new();
        let result = processor.traditional_to_simplified("機器學習和資料庫");
        assert_eq!(result, "机器学习和数据库");
    }
    
    #[test]
    fn test_punctuation_normalization() {
        let processor = ChineseTextProcessor::new();
        let result = processor.normalize_punctuation("你好，世界！這是測試。");
        assert_eq!(result, "你好,世界!這是測試.");
    }
    
    #[test]
    fn test_whitespace_normalization() {
        let processor = ChineseTextProcessor::new();
        let result = processor.normalize_whitespace("  hello   world  \n\t  ");
        assert_eq!(result, "hello world");
    }
    
    #[test]
    fn test_text_truncation() {
        let processor = ChineseTextProcessor::new();
        let long_text = "这是一个很长的文本".repeat(50);
        let result = processor.truncate_text(&long_text, 10);
        assert_eq!(result.chars().count(), 10);
    }
    
    #[test]
    fn test_preprocess() {
        let processor = ChineseTextProcessor::new();
        let result = processor.preprocess("  機器學習，很有趣！  ");
        assert_eq!(result, "机器学习,很有趣!");
    }
    
    #[test]
    fn test_language_detection() {
        let processor = ChineseTextProcessor::new();
        
        assert_eq!(processor.detect_language("你好世界"), Language::Chinese);
        assert_eq!(processor.detect_language("hello world"), Language::English);
        assert_eq!(processor.detect_language("hello 世界"), Language::Mixed);
        assert_eq!(processor.detect_language("123456"), Language::Unknown);
    }
    
    #[test]
    fn test_chinese_char_detection() {
        let processor = ChineseTextProcessor::new();
        
        assert!(processor.is_chinese_char('中'));
        assert!(processor.is_chinese_char('文'));
        assert!(!processor.is_chinese_char('a'));
        assert!(!processor.is_chinese_char('1'));
        assert!(!processor.is_chinese_char('!'));
    }
}
