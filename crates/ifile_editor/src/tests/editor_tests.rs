//! 编辑器功能单元测试

use crate::state::{EditorState, TextBuffer, Cursor};
use crate::settings::EditorSettings;
use std::path::PathBuf;
use tempfile::TempDir;

#[test]
fn test_editor_state_creation() {
    let editor = EditorState::new();
    assert!(editor.buffers.is_empty());
    assert!(editor.tabs.is_empty());
    assert!(editor.active_tab.is_none());
    assert!(editor.undo_stack.is_empty());
}

#[test]
fn test_open_file() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.txt");
    std::fs::write(&file_path, "Hello, World!").unwrap();

    let mut editor = EditorState::new();
    let settings = EditorSettings::default();

    // 打开文件
    editor.open_file(file_path.clone(), &settings).unwrap();

    // 验证文件已打开
    assert_eq!(editor.buffers.len(), 1);
    assert_eq!(editor.tabs.len(), 1);
    assert_eq!(editor.active_tab, Some(0));
    assert!(editor.buffers.contains_key(&file_path));
    assert_eq!(editor.tabs[0], file_path);
}

#[test]
fn test_open_same_file_twice() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.txt");
    std::fs::write(&file_path, "Hello, World!").unwrap();

    let mut editor = EditorState::new();
    let settings = EditorSettings::default();

    // 打开文件两次
    editor.open_file(file_path.clone(), &settings).unwrap();
    editor.open_file(file_path.clone(), &settings).unwrap();

    // 验证只有一个缓冲区和标签页
    assert_eq!(editor.buffers.len(), 1);
    assert_eq!(editor.tabs.len(), 1);
    assert_eq!(editor.active_tab, Some(0));
}

#[test]
fn test_get_active_buffer() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.txt");
    std::fs::write(&file_path, "Hello, World!").unwrap();

    let mut editor = EditorState::new();
    let settings = EditorSettings::default();

    // 没有打开文件时
    assert!(editor.get_active_buffer().is_none());

    // 打开文件后
    editor.open_file(file_path.clone(), &settings).unwrap();
    let buffer = editor.get_active_buffer().unwrap();
    assert_eq!(buffer.file_path, file_path);
}

#[test]
fn test_save_file() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.txt");
    std::fs::write(&file_path, "Hello, World!").unwrap();

    let mut editor = EditorState::new();
    let settings = EditorSettings::default();

    // 打开文件
    editor.open_file(file_path.clone(), &settings).unwrap();

    // 修改文件内容
    {
        let buffer = editor.buffers.get_mut(&file_path).unwrap();
        buffer.insert_text(0, "Modified: ");
        assert!(buffer.modified);
    }

    // 保存文件
    editor.save_file(&file_path).unwrap();

    // 验证文件已保存
    let buffer = editor.buffers.get(&file_path).unwrap();
    assert!(!buffer.modified);

    // 验证文件内容
    let content = std::fs::read_to_string(&file_path).unwrap();
    assert_eq!(content, "Modified: Hello, World!");
}

#[test]
fn test_close_file() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.txt");
    std::fs::write(&file_path, "Hello, World!").unwrap();

    let mut editor = EditorState::new();
    let settings = EditorSettings::default();

    // 打开文件
    editor.open_file(file_path.clone(), &settings).unwrap();
    assert_eq!(editor.buffers.len(), 1);
    assert_eq!(editor.tabs.len(), 1);

    // 关闭文件
    editor.close_file(&file_path).unwrap();
    assert_eq!(editor.buffers.len(), 0);
    assert_eq!(editor.tabs.len(), 0);
    assert!(editor.active_tab.is_none());
}

#[test]
fn test_text_buffer_operations() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.txt");
    std::fs::write(&file_path, "Hello, World!").unwrap();

    let settings = EditorSettings::default();
    let mut buffer = TextBuffer::from_file(&file_path, &settings).unwrap();

    // 测试基本属性
    assert_eq!(buffer.line_count(), 1);
    assert_eq!(buffer.char_count(), 13);
    assert!(!buffer.modified);

    // 测试插入文本
    buffer.insert_text(0, "Hi, ");
    assert!(buffer.modified);
    assert_eq!(buffer.rope.to_string(), "Hi, Hello, World!");

    // 测试删除文本
    buffer.delete_text(0..4);
    assert_eq!(buffer.rope.to_string(), "Hello, World!");

    // 测试替换文本
    buffer.replace_text(0..5, "Goodbye");
    assert_eq!(buffer.rope.to_string(), "Goodbye, World!");

    // 测试获取行文本
    let line_text = buffer.get_line_text(0);
    assert_eq!(line_text, "Goodbye, World!");
}

#[test]
fn test_cursor_operations() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.txt");
    std::fs::write(&file_path, "Line 1\nLine 2\nLine 3").unwrap();

    let settings = EditorSettings::default();
    let mut buffer = TextBuffer::from_file(&file_path, &settings).unwrap();

    // 测试设置光标位置（字节偏移）
    buffer.set_cursor_position(7); // 第二行开始
    assert_eq!(buffer.cursor.line, 1);
    assert_eq!(buffer.cursor.column, 0);

    // 测试设置光标位置（行列）
    buffer.set_cursor_line_column(2, 3);
    assert_eq!(buffer.cursor.line, 2);
    assert_eq!(buffer.cursor.column, 3);
}

#[test]
fn test_large_file_rejection() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("large.txt");
    
    // 创建一个大文件（模拟）
    let large_content = "x".repeat(1024 * 1024 * 15); // 15MB
    std::fs::write(&file_path, large_content).unwrap();

    let mut settings = EditorSettings::default();
    settings.max_file_size_mb = 10; // 设置最大10MB

    // 尝试打开大文件应该失败
    let result = TextBuffer::from_file(&file_path, &settings);
    assert!(result.is_err());
}

#[test]
fn test_read_only_file() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("readonly.txt");
    std::fs::write(&file_path, "Read only content").unwrap();

    // 设置文件为只读
    let mut perms = std::fs::metadata(&file_path).unwrap().permissions();
    perms.set_readonly(true);
    std::fs::set_permissions(&file_path, perms).unwrap();

    let settings = EditorSettings::default();
    let buffer = TextBuffer::from_file(&file_path, &settings).unwrap();
    
    // 验证文件被标记为只读
    assert!(buffer.read_only);
}
