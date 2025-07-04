//! 文本缓冲区管理器

use std::collections::HashMap;
use std::path::PathBuf;
use crop::Rope;
use crate::{FileEditorError, FileEditorResult};
use crate::state::{TextBuffer, EditOperation, OperationType, Cursor};
use crate::settings::EditorSettings;

/// 文本缓冲区管理器
pub struct TextBufferManager {
    /// 缓冲区映射
    buffers: HashMap<PathBuf, TextBuffer>,
    
    /// 撤销栈
    undo_stacks: HashMap<PathBuf, Vec<EditOperation>>,
    
    /// 重做栈
    redo_stacks: HashMap<PathBuf, Vec<EditOperation>>,
}

impl TextBufferManager {
    pub fn new() -> Self {
        Self {
            buffers: HashMap::new(),
            undo_stacks: HashMap::new(),
            redo_stacks: HashMap::new(),
        }
    }
    
    /// 创建新的文本缓冲区
    pub fn create_buffer(&mut self, path: PathBuf, content: String, settings: &EditorSettings) -> FileEditorResult<()> {
        let rope = Rope::from(content);
        
        let buffer = TextBuffer {
            rope,
            file_path: path.clone(),
            encoding: settings.default_encoding.clone(),
            line_ending: crate::state::LineEnding::Unix,
            modified: false,
            last_saved: std::time::SystemTime::now(),
            read_only: false, // 新创建的缓冲区默认可编辑
            cursor: Cursor::default(),
            selection: None,
            language: detect_language(&path),
            scroll_offset: 0.0,
            visible_lines: 0..0,
        };
        
        self.buffers.insert(path.clone(), buffer);
        self.undo_stacks.insert(path.clone(), Vec::new());
        self.redo_stacks.insert(path, Vec::new());
        
        Ok(())
    }
    
    /// 获取缓冲区
    pub fn get_buffer(&self, path: &PathBuf) -> Option<&TextBuffer> {
        self.buffers.get(path)
    }
    
    /// 获取可变缓冲区
    pub fn get_buffer_mut(&mut self, path: &PathBuf) -> Option<&mut TextBuffer> {
        self.buffers.get_mut(path)
    }
    
    /// 插入文本
    pub fn insert_text(&mut self, path: &PathBuf, position: usize, text: &str) -> FileEditorResult<()> {
        let buffer = self.buffers.get_mut(path)
            .ok_or_else(|| FileEditorError::FileNotFound { 
                path: path.to_string_lossy().to_string() 
            })?;
        
        // 记录撤销操作
        let operation = EditOperation {
            operation_type: OperationType::Insert,
            range: position..position,
            text: text.to_string(),
            cursor_before: buffer.cursor.clone(),
            cursor_after: calculate_cursor_after_insert(&buffer.cursor, position, text),
        };
        
        // 执行插入
        buffer.rope.insert(position, text);
        buffer.modified = true;
        buffer.cursor = operation.cursor_after.clone();
        
        // 添加到撤销栈
        if let Some(undo_stack) = self.undo_stacks.get_mut(path) {
            undo_stack.push(operation);
        }
        
        // 清空重做栈
        if let Some(redo_stack) = self.redo_stacks.get_mut(path) {
            redo_stack.clear();
        }
        
        Ok(())
    }
    
    /// 删除文本
    pub fn delete_text(&mut self, path: &PathBuf, range: std::ops::Range<usize>) -> FileEditorResult<()> {
        let buffer = self.buffers.get_mut(path)
            .ok_or_else(|| FileEditorError::FileNotFound { 
                path: path.to_string_lossy().to_string() 
            })?;
        
        // 获取要删除的文本
        let deleted_text = buffer.rope.byte_slice(range.clone()).to_string();
        
        // 记录撤销操作
        let operation = EditOperation {
            operation_type: OperationType::Delete,
            range: range.clone(),
            text: deleted_text,
            cursor_before: buffer.cursor.clone(),
            cursor_after: calculate_cursor_after_delete(&buffer.cursor, &range),
        };
        
        // 执行删除
        buffer.rope.delete(range);
        buffer.modified = true;
        buffer.cursor = operation.cursor_after.clone();
        
        // 添加到撤销栈
        if let Some(undo_stack) = self.undo_stacks.get_mut(path) {
            undo_stack.push(operation);
        }
        
        // 清空重做栈
        if let Some(redo_stack) = self.redo_stacks.get_mut(path) {
            redo_stack.clear();
        }
        
        Ok(())
    }
    
    /// 替换文本
    pub fn replace_text(&mut self, path: &PathBuf, range: std::ops::Range<usize>, text: &str) -> FileEditorResult<()> {
        let buffer = self.buffers.get_mut(path)
            .ok_or_else(|| FileEditorError::FileNotFound { 
                path: path.to_string_lossy().to_string() 
            })?;
        
        // 获取要替换的文本
        let old_text = buffer.rope.byte_slice(range.clone()).to_string();
        
        // 记录撤销操作
        let operation = EditOperation {
            operation_type: OperationType::Replace,
            range: range.clone(),
            text: old_text,
            cursor_before: buffer.cursor.clone(),
            cursor_after: calculate_cursor_after_replace(&buffer.cursor, &range, text),
        };
        
        // 执行替换
        buffer.rope.replace(range, text);
        buffer.modified = true;
        buffer.cursor = operation.cursor_after.clone();
        
        // 添加到撤销栈
        if let Some(undo_stack) = self.undo_stacks.get_mut(path) {
            undo_stack.push(operation);
        }
        
        // 清空重做栈
        if let Some(redo_stack) = self.redo_stacks.get_mut(path) {
            redo_stack.clear();
        }
        
        Ok(())
    }
    
    /// 撤销操作
    pub fn undo(&mut self, path: &PathBuf) -> FileEditorResult<()> {
        let undo_stack = self.undo_stacks.get_mut(path)
            .ok_or_else(|| FileEditorError::FileNotFound { 
                path: path.to_string_lossy().to_string() 
            })?;
        
        let operation = undo_stack.pop()
            .ok_or_else(|| FileEditorError::RopeError("No operation to undo".to_string()))?;
        
        let buffer = self.buffers.get_mut(path).unwrap();
        
        match operation.operation_type {
            OperationType::Insert => {
                // 撤销插入 = 删除插入的文本
                let end = operation.range.start + operation.text.len();
                buffer.rope.delete(operation.range.start..end);
            }
            OperationType::Delete => {
                // 撤销删除 = 重新插入删除的文本
                buffer.rope.insert(operation.range.start, &operation.text);
            }
            OperationType::Replace => {
                // 撤销替换 = 恢复原文本
                let current_end = operation.range.start + operation.text.len();
                buffer.rope.replace(operation.range.start..current_end, &operation.text);
            }
        }
        
        buffer.cursor = operation.cursor_before.clone();
        buffer.modified = true;

        // 添加到重做栈
        if let Some(redo_stack) = self.redo_stacks.get_mut(path) {
            redo_stack.push(operation);
        }
        
        Ok(())
    }
    
    /// 重做操作
    pub fn redo(&mut self, path: &PathBuf) -> FileEditorResult<()> {
        let redo_stack = self.redo_stacks.get_mut(path)
            .ok_or_else(|| FileEditorError::FileNotFound { 
                path: path.to_string_lossy().to_string() 
            })?;
        
        let operation = redo_stack.pop()
            .ok_or_else(|| FileEditorError::RopeError("No operation to redo".to_string()))?;
        
        let buffer = self.buffers.get_mut(path).unwrap();
        
        match operation.operation_type {
            OperationType::Insert => {
                buffer.rope.insert(operation.range.start, &operation.text);
            }
            OperationType::Delete => {
                buffer.rope.delete(operation.range.clone());
            }
            OperationType::Replace => {
                buffer.rope.replace(operation.range.clone(), &operation.text);
            }
        }
        
        buffer.cursor = operation.cursor_after.clone();
        buffer.modified = true;

        // 添加回撤销栈
        if let Some(undo_stack) = self.undo_stacks.get_mut(path) {
            undo_stack.push(operation);
        }
        
        Ok(())
    }
    
    /// 移除缓冲区
    pub fn remove_buffer(&mut self, path: &PathBuf) {
        self.buffers.remove(path);
        self.undo_stacks.remove(path);
        self.redo_stacks.remove(path);
    }
    
    /// 获取所有缓冲区路径
    pub fn get_buffer_paths(&self) -> Vec<PathBuf> {
        self.buffers.keys().cloned().collect()
    }
}

impl Default for TextBufferManager {
    fn default() -> Self {
        Self::new()
    }
}

/// 检测文件语言
fn detect_language(path: &PathBuf) -> Option<String> {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| match ext.to_lowercase().as_str() {
            "rs" => "rust",
            "py" => "python",
            "js" | "ts" => "javascript",
            "md" => "markdown",
            "json" => "json",
            "toml" => "toml",
            "yaml" | "yml" => "yaml",
            "html" => "html",
            "css" => "css",
            "xml" => "xml",
            _ => "text",
        })
        .map(|s| s.to_string())
}

/// 计算插入后的光标位置
fn calculate_cursor_after_insert(cursor: &Cursor, position: usize, text: &str) -> Cursor {
    if position <= cursor.byte_offset {
        Cursor {
            line: cursor.line,
            column: cursor.column,
            byte_offset: cursor.byte_offset + text.len(),
        }
    } else {
        cursor.clone()
    }
}

/// 计算删除后的光标位置
fn calculate_cursor_after_delete(cursor: &Cursor, range: &std::ops::Range<usize>) -> Cursor {
    if cursor.byte_offset >= range.end {
        Cursor {
            line: cursor.line,
            column: cursor.column,
            byte_offset: cursor.byte_offset - (range.end - range.start),
        }
    } else if cursor.byte_offset > range.start {
        Cursor {
            line: cursor.line,
            column: cursor.column,
            byte_offset: range.start,
        }
    } else {
        cursor.clone()
    }
}

/// 计算替换后的光标位置
fn calculate_cursor_after_replace(cursor: &Cursor, range: &std::ops::Range<usize>, new_text: &str) -> Cursor {
    if cursor.byte_offset >= range.end {
        Cursor {
            line: cursor.line,
            column: cursor.column,
            byte_offset: cursor.byte_offset - (range.end - range.start) + new_text.len(),
        }
    } else if cursor.byte_offset > range.start {
        Cursor {
            line: cursor.line,
            column: cursor.column,
            byte_offset: range.start + new_text.len(),
        }
    } else {
        cursor.clone()
    }
}
