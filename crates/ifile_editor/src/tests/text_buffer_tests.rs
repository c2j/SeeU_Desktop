//! 文本缓冲区和ROPE操作单元测试

use crate::state::{TextBuffer, Cursor, Selection};
use crate::settings::EditorSettings;
use crop::Rope;
use std::path::PathBuf;
use tempfile::TempDir;

#[test]
fn test_rope_basic_operations() {
    let mut rope = Rope::from("Hello, World!");
    
    // 测试基本属性
    assert_eq!(rope.byte_len(), 13);
    assert_eq!(rope.to_string(), "Hello, World!");

    // 测试插入
    rope.insert(7, "Rust ");
    assert_eq!(rope.to_string(), "Hello, Rust World!");

    // 测试删除
    rope.delete(0..7);
    assert_eq!(rope.to_string(), "Rust World!");

    // 测试切片
    let slice = rope.byte_slice(0..4);
    assert_eq!(slice.to_string(), "Rust");
}

#[test]
fn test_text_buffer_creation() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.txt");
    let content = "Line 1\nLine 2\nLine 3\n";
    std::fs::write(&file_path, content).unwrap();

    let settings = EditorSettings::default();
    let buffer = TextBuffer::from_file(&file_path, &settings).unwrap();

    assert_eq!(buffer.file_path, file_path);
    assert_eq!(buffer.rope.to_string(), content);
    assert_eq!(buffer.line_count(), 3); // 三行文本
    assert!(!buffer.modified);
    assert_eq!(buffer.cursor.line, 0);
    assert_eq!(buffer.cursor.column, 0);
    assert_eq!(buffer.cursor.byte_offset, 0);
}

#[test]
fn test_text_insertion() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.txt");
    std::fs::write(&file_path, "Hello World").unwrap();

    let settings = EditorSettings::default();
    let mut buffer = TextBuffer::from_file(&file_path, &settings).unwrap();

    // 在开头插入
    buffer.insert_text(0, "Hi, ");
    assert_eq!(buffer.rope.to_string(), "Hi, Hello World");
    assert!(buffer.modified);

    // 在中间插入
    buffer.insert_text(4, "there ");
    assert_eq!(buffer.rope.to_string(), "Hi, there Hello World");

    // 在末尾插入
    buffer.insert_text(buffer.rope.byte_len(), "!");
    assert_eq!(buffer.rope.to_string(), "Hi, there Hello World!");
}

#[test]
fn test_text_deletion() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.txt");
    std::fs::write(&file_path, "Hello, World!").unwrap();

    let settings = EditorSettings::default();
    let mut buffer = TextBuffer::from_file(&file_path, &settings).unwrap();

    // 删除开头
    buffer.delete_text(0..7);
    assert_eq!(buffer.rope.to_string(), "World!");
    assert!(buffer.modified);

    // 删除末尾
    buffer.delete_text(5..6);
    assert_eq!(buffer.rope.to_string(), "World");

    // 删除中间
    buffer.delete_text(1..4);
    assert_eq!(buffer.rope.to_string(), "Wd");
}

#[test]
fn test_text_replacement() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.txt");
    std::fs::write(&file_path, "Hello, World!").unwrap();

    let settings = EditorSettings::default();
    let mut buffer = TextBuffer::from_file(&file_path, &settings).unwrap();

    // 替换单词
    buffer.replace_text(0..5, "Hi");
    assert_eq!(buffer.rope.to_string(), "Hi, World!");
    assert!(buffer.modified);

    // 替换更长的文本
    buffer.replace_text(4..10, "Universe");
    assert_eq!(buffer.rope.to_string(), "Hi, Universe");
}

#[test]
fn test_get_text_range() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.txt");
    std::fs::write(&file_path, "Hello, World!").unwrap();

    let settings = EditorSettings::default();
    let buffer = TextBuffer::from_file(&file_path, &settings).unwrap();

    // 获取部分文本
    assert_eq!(buffer.get_text_range(0..5), "Hello");
    assert_eq!(buffer.get_text_range(7..12), "World");
    assert_eq!(buffer.get_text_range(0..13), "Hello, World!");

    // 边界情况
    assert_eq!(buffer.get_text_range(0..0), "");
    assert_eq!(buffer.get_text_range(13..13), "");
    assert_eq!(buffer.get_text_range(20..25), ""); // 超出范围
}

#[test]
fn test_get_line_text() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.txt");
    let content = "Line 1\nLine 2\nLine 3";
    std::fs::write(&file_path, content).unwrap();

    let settings = EditorSettings::default();
    let buffer = TextBuffer::from_file(&file_path, &settings).unwrap();

    assert_eq!(buffer.get_line_text(0), "Line 1");
    assert_eq!(buffer.get_line_text(1), "Line 2");
    assert_eq!(buffer.get_line_text(2), "Line 3");
    assert_eq!(buffer.get_line_text(10), ""); // 超出范围
}

#[test]
fn test_cursor_positioning() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.txt");
    let content = "Line 1\nLine 2\nLine 3";
    std::fs::write(&file_path, content).unwrap();

    let settings = EditorSettings::default();
    let mut buffer = TextBuffer::from_file(&file_path, &settings).unwrap();

    // 测试字节偏移设置
    buffer.set_cursor_position(7); // 第二行开始
    assert_eq!(buffer.cursor.line, 1);
    assert_eq!(buffer.cursor.column, 0);
    assert_eq!(buffer.cursor.byte_offset, 7);

    // 测试行列设置
    buffer.set_cursor_line_column(2, 3);
    assert_eq!(buffer.cursor.line, 2);
    assert_eq!(buffer.cursor.column, 3);

    // 测试边界情况
    buffer.set_cursor_line_column(10, 5); // 超出行数
    assert_eq!(buffer.cursor.line, 2); // 应该限制在最后一行

    buffer.set_cursor_line_column(1, 100); // 超出列数
    assert_eq!(buffer.cursor.line, 1);
    assert!(buffer.cursor.column <= 6); // 应该限制在行末
}

#[test]
fn test_cursor_update_after_edit() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.txt");
    std::fs::write(&file_path, "Hello World").unwrap();

    let settings = EditorSettings::default();
    let mut buffer = TextBuffer::from_file(&file_path, &settings).unwrap();

    // 设置光标在中间
    buffer.set_cursor_position(6); // "World" 前面
    assert_eq!(buffer.cursor.byte_offset, 6);

    // 在光标前插入文本
    buffer.insert_text(0, "Hi, ");
    assert_eq!(buffer.cursor.byte_offset, 10); // 光标应该向后移动

    // 在光标后插入文本
    buffer.insert_text(15, "!");
    assert_eq!(buffer.cursor.byte_offset, 10); // 光标不应该移动

    // 删除光标前的文本
    buffer.delete_text(0..4);
    assert_eq!(buffer.cursor.byte_offset, 6); // 光标应该向前移动
}

#[test]
fn test_multiline_operations() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.txt");
    let content = "Line 1\nLine 2\nLine 3\n";
    std::fs::write(&file_path, content).unwrap();

    let settings = EditorSettings::default();
    let mut buffer = TextBuffer::from_file(&file_path, &settings).unwrap();

    assert_eq!(buffer.line_count(), 3); // 三行文本

    // 插入新行
    buffer.insert_text(7, "New Line\n");
    assert_eq!(buffer.line_count(), 4);

    // 删除一行
    buffer.delete_text(7..17); // 删除 "New Line\n"
    assert_eq!(buffer.line_count(), 3);

    // 合并行
    buffer.delete_text(6..7); // 删除第一个换行符
    assert_eq!(buffer.line_count(), 2);
    assert_eq!(buffer.get_line_text(0), "Line 1ine 2");
}

#[test]
fn test_empty_file() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("empty.txt");
    std::fs::write(&file_path, "").unwrap();

    let settings = EditorSettings::default();
    let mut buffer = TextBuffer::from_file(&file_path, &settings).unwrap();

    assert_eq!(buffer.line_count(), 0); // 空文件没有行
    assert_eq!(buffer.char_count(), 0);
    assert_eq!(buffer.byte_count(), 0);

    // 在空文件中插入文本
    buffer.insert_text(0, "First line");
    assert_eq!(buffer.rope.to_string(), "First line");
    assert_eq!(buffer.line_count(), 1);
}

#[test]
fn test_unicode_handling() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("unicode.txt");
    let content = "Hello 世界! 🦀";
    std::fs::write(&file_path, content).unwrap();

    let settings = EditorSettings::default();
    let mut buffer = TextBuffer::from_file(&file_path, &settings).unwrap();

    // Unicode字符应该正确处理
    assert_eq!(buffer.rope.to_string(), content);
    assert_eq!(buffer.char_count(), 11); // 字符数
    assert!(buffer.byte_count() > 11); // 字节数应该更多

    // 测试Unicode文本编辑
    buffer.insert_text(6, "美丽的");
    assert_eq!(buffer.rope.to_string(), "Hello 美丽的世界! 🦀");

    // 测试光标在Unicode文本中的位置
    buffer.set_cursor_position(6);
    assert_eq!(buffer.cursor.column, 6);
}
