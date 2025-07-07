//! TextBuffer 适配器，用于与 egui_code_editor 集成

use std::ops::Range;
use crop::Rope;
use crate::state::TextBuffer;

/// TextBuffer 适配器，包装我们的 TextBuffer 以实现 egui 的 TextBuffer trait
pub struct TextBufferAdapter<'a> {
    buffer: &'a mut TextBuffer,
    cached_string: Option<String>,
}

impl<'a> TextBufferAdapter<'a> {
    pub fn new(buffer: &'a mut TextBuffer) -> Self {
        Self {
            buffer,
            cached_string: None,
        }
    }
    

    
    /// 更新 ROPE 内容并清除缓存
    fn update_rope(&mut self, text: String) {
        self.buffer.rope = Rope::from(text);
        self.buffer.modified = true;
        self.cached_string = None;
    }
    
    /// 字符索引转换为字节索引
    fn char_to_byte_index(&self, char_index: usize) -> usize {
        let text = self.buffer.rope.to_string();
        text.char_indices()
            .nth(char_index)
            .map(|(byte_idx, _)| byte_idx)
            .unwrap_or(text.len())
    }
}

impl<'a> egui::widgets::text_edit::TextBuffer for TextBufferAdapter<'a> {
    fn is_mutable(&self) -> bool {
        !self.buffer.read_only
    }
    
    fn as_str(&self) -> &str {
        // 由于生命周期限制，我们需要返回一个静态引用
        // 这里我们使用一个不太优雅但可行的方案
        let text = self.buffer.rope.to_string();
        let leaked = Box::leak(text.into_boxed_str());
        leaked
    }
    
    fn insert_text(&mut self, text: &str, char_index: usize) -> usize {
        if self.buffer.read_only {
            return 0;
        }
        
        let current_text = self.buffer.rope.to_string();
        let char_count = current_text.chars().count();
        
        if char_index > char_count {
            return 0;
        }
        
        // 将字符索引转换为字节索引
        let byte_index = self.char_to_byte_index(char_index);
        
        // 插入文本
        let mut new_text = current_text;
        new_text.insert_str(byte_index, text);
        
        // 更新 ROPE
        self.update_rope(new_text);
        
        // 返回插入的字符数
        text.chars().count()
    }
    
    fn delete_char_range(&mut self, char_range: Range<usize>) {
        if self.buffer.read_only {
            return;
        }
        
        let current_text = self.buffer.rope.to_string();
        let char_count = current_text.chars().count();
        
        let start = char_range.start.min(char_count);
        let end = char_range.end.min(char_count);
        
        if start >= end {
            return;
        }
        
        // 转换为字节索引
        let start_byte = self.char_to_byte_index(start);
        let end_byte = self.char_to_byte_index(end);
        
        // 删除文本
        let mut new_text = current_text;
        new_text.drain(start_byte..end_byte);
        
        // 更新 ROPE
        self.update_rope(new_text);
    }
    
    fn char_range(&self, char_range: Range<usize>) -> &str {
        let text = self.buffer.rope.to_string();
        let char_count = text.chars().count();

        let start = char_range.start.min(char_count);
        let end = char_range.end.min(char_count);

        if start >= end {
            return "";
        }

        let start_byte = self.char_to_byte_index(start);
        let end_byte = self.char_to_byte_index(end);

        // 同样的生命周期问题，使用 leak
        let slice = text[start_byte..end_byte].to_string();
        let leaked = Box::leak(slice.into_boxed_str());
        leaked
    }
    
    fn byte_index_from_char_index(&self, char_index: usize) -> usize {
        self.char_to_byte_index(char_index)
    }
    
    fn clear(&mut self) {
        if !self.buffer.read_only {
            self.update_rope(String::new());
        }
    }
    
    fn replace_with(&mut self, text: &str) {
        if !self.buffer.read_only {
            self.update_rope(text.to_string());
        }
    }
    
    fn take(&mut self) -> String {
        if self.buffer.read_only {
            self.buffer.rope.to_string()
        } else {
            let text = self.buffer.rope.to_string();
            self.update_rope(String::new());
            text
        }
    }
}

/// 简化的 TextBuffer 包装器，直接使用 String
pub struct SimpleTextBuffer {
    pub text: String,
    pub read_only: bool,
}

impl SimpleTextBuffer {
    pub fn new(text: String, read_only: bool) -> Self {
        Self { text, read_only }
    }
    
    pub fn from_rope(rope: &Rope, read_only: bool) -> Self {
        Self {
            text: rope.to_string(),
            read_only,
        }
    }
    
    pub fn to_rope(&self) -> Rope {
        Rope::from(self.text.clone())
    }
}

impl egui::widgets::text_edit::TextBuffer for SimpleTextBuffer {
    fn is_mutable(&self) -> bool {
        !self.read_only
    }
    
    fn as_str(&self) -> &str {
        &self.text
    }
    
    fn insert_text(&mut self, text: &str, char_index: usize) -> usize {
        if self.read_only {
            return 0;
        }
        
        let char_count = self.text.chars().count();
        if char_index > char_count {
            return 0;
        }
        
        // 找到字节索引
        let byte_index = self.text
            .char_indices()
            .nth(char_index)
            .map(|(i, _)| i)
            .unwrap_or(self.text.len());
        
        self.text.insert_str(byte_index, text);
        text.chars().count()
    }
    
    fn delete_char_range(&mut self, char_range: Range<usize>) {
        if self.read_only {
            return;
        }
        
        let char_count = self.text.chars().count();
        let start = char_range.start.min(char_count);
        let end = char_range.end.min(char_count);
        
        if start >= end {
            return;
        }
        
        // 转换为字节索引
        let start_byte = self.text
            .char_indices()
            .nth(start)
            .map(|(i, _)| i)
            .unwrap_or(self.text.len());
        
        let end_byte = self.text
            .char_indices()
            .nth(end)
            .map(|(i, _)| i)
            .unwrap_or(self.text.len());
        
        self.text.drain(start_byte..end_byte);
    }
}
