//! 语法高亮 - 基于 egui_code_editor

use egui_code_editor::{Syntax, ColorTheme};
use crate::{FileEditorError, FileEditorResult};

/// 语法高亮器 - 基于 egui_code_editor
pub struct SyntaxHighlighter {
    current_theme: String,
}

impl SyntaxHighlighter {
    pub fn new() -> Self {
        Self {
            current_theme: "gruvbox".to_string(),
        }
    }

    /// 设置主题
    pub fn set_theme(&mut self, theme_name: &str) -> FileEditorResult<()> {
        // 验证主题名称
        match theme_name.to_lowercase().as_str() {
            "gruvbox" | "github_light" | "github_dark" | "ayu" | "ayu_dark" | "ayu_mirage" | "sonokai" => {
                self.current_theme = theme_name.to_string();
                Ok(())
            }
            _ => Err(FileEditorError::SyntaxError(format!("Theme not found: {}", theme_name)))
        }
    }

    /// 获取当前主题
    pub fn get_current_theme(&self) -> ColorTheme {
        match self.current_theme.to_lowercase().as_str() {
            "gruvbox" => ColorTheme::GRUVBOX,
            "github_light" => ColorTheme::GITHUB_LIGHT,
            "github_dark" => ColorTheme::GITHUB_DARK,
            "ayu" => ColorTheme::AYU,
            "ayu_dark" => ColorTheme::AYU_DARK,
            "ayu_mirage" => ColorTheme::AYU_MIRAGE,
            "sonokai" => ColorTheme::SONOKAI,
            _ => ColorTheme::GRUVBOX,
        }
    }

    /// 获取可用主题列表
    pub fn get_available_themes(&self) -> Vec<String> {
        vec![
            "gruvbox".to_string(),
            "github_light".to_string(),
            "github_dark".to_string(),
            "ayu".to_string(),
            "ayu_dark".to_string(),
            "ayu_mirage".to_string(),
            "sonokai".to_string(),
        ]
    }

    /// 根据文件扩展名查找语法
    pub fn find_syntax_by_extension(&self, extension: &str) -> Option<Syntax> {
        match extension.to_lowercase().as_str() {
            "rs" => Some(Syntax::rust()),
            "py" => Some(Syntax::python()),
            "lua" => Some(Syntax::lua()),
            "sql" => Some(Syntax::sql()),
            "sh" | "bash" => Some(Syntax::shell()),
            "asm" | "s" => Some(Syntax::asm()),
            _ => None,
        }
    }

    /// 根据文件名查找语法
    pub fn find_syntax_by_name(&self, name: &str) -> Option<Syntax> {
        match name.to_lowercase().as_str() {
            "rust" => Some(Syntax::rust()),
            "python" => Some(Syntax::python()),
            "lua" => Some(Syntax::lua()),
            "sql" => Some(Syntax::sql()),
            "shell" => Some(Syntax::shell()),
            "asm" | "assembly" => Some(Syntax::asm()),
            _ => None,
        }
    }

    /// 根据第一行内容查找语法（用于脚本文件）
    pub fn find_syntax_by_first_line(&self, first_line: &str) -> Option<Syntax> {
        if first_line.starts_with("#!/bin/bash") || first_line.starts_with("#!/bin/sh") {
            Some(Syntax::shell())
        } else if first_line.starts_with("#!/usr/bin/env python") || first_line.starts_with("#!/usr/bin/python") {
            Some(Syntax::python())
        } else if first_line.starts_with("#!/usr/bin/env lua") {
            Some(Syntax::lua())
        } else {
            None
        }
    }
}

impl Default for SyntaxHighlighter {
    fn default() -> Self {
        Self::new()
    }
}

/// 语法高亮缓存 - 简化版本
pub struct HighlightCache {
    /// 缓存的语法类型
    cache: std::collections::HashMap<String, Syntax>,

    /// 最大缓存大小
    max_size: usize,
}

impl HighlightCache {
    pub fn new(max_size: usize) -> Self {
        Self {
            cache: std::collections::HashMap::new(),
            max_size,
        }
    }

    /// 获取缓存的语法
    pub fn get(&self, key: &str) -> Option<&Syntax> {
        self.cache.get(key)
    }

    /// 设置缓存
    pub fn set(&mut self, key: String, value: Syntax) {
        if self.cache.len() >= self.max_size {
            // 简单的LRU：移除第一个元素
            if let Some(first_key) = self.cache.keys().next().cloned() {
                self.cache.remove(&first_key);
            }
        }

        self.cache.insert(key, value);
    }

    /// 清空缓存
    pub fn clear(&mut self) {
        self.cache.clear();
    }

    /// 获取缓存大小
    pub fn size(&self) -> usize {
        self.cache.len()
    }
}

impl Default for HighlightCache {
    fn default() -> Self {
        Self::new(1000) // 默认缓存1000个文件
    }
}

/// 语法高亮工具函数
pub mod utils {
    use super::*;
    use std::path::Path;

    /// 根据文件路径自动检测语法
    pub fn detect_syntax_by_path(
        highlighter: &SyntaxHighlighter,
        path: &Path,
    ) -> Option<Syntax> {
        // 首先尝试通过扩展名
        if let Some(extension) = path.extension().and_then(|ext| ext.to_str()) {
            if let Some(syntax) = highlighter.find_syntax_by_extension(extension) {
                return Some(syntax);
            }
        }

        // 然后尝试通过文件名
        if let Some(filename) = path.file_name().and_then(|name| name.to_str()) {
            if let Some(syntax) = highlighter.find_syntax_by_name(filename) {
                return Some(syntax);
            }
        }

        None
    }

    /// 根据内容检测语法
    pub fn detect_syntax_by_content(
        highlighter: &SyntaxHighlighter,
        content: &str,
    ) -> Option<Syntax> {
        if let Some(first_line) = content.lines().next() {
            highlighter.find_syntax_by_first_line(first_line)
        } else {
            None
        }
    }

    /// 根据文件路径和内容自动检测语言
    pub fn detect_language(path: &Path) -> Option<String> {
        if let Some(extension) = path.extension().and_then(|ext| ext.to_str()) {
            match extension.to_lowercase().as_str() {
                "rs" => Some("rust".to_string()),
                "py" => Some("python".to_string()),
                "lua" => Some("lua".to_string()),
                "sql" => Some("sql".to_string()),
                "sh" | "bash" => Some("shell".to_string()),
                "asm" | "s" => Some("asm".to_string()),
                _ => None,
            }
        } else {
            None
        }
    }
}
