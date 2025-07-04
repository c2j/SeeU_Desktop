//! 文件树功能单元测试

use std::path::PathBuf;
use tempfile::TempDir;
use crate::state::{FileTreeState, FileNodeId};

/// 创建测试目录结构
fn create_test_directory() -> TempDir {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let root = temp_dir.path();
    
    // 创建测试文件和目录
    std::fs::create_dir(root.join("src")).unwrap();
    std::fs::create_dir(root.join("docs")).unwrap();
    std::fs::create_dir(root.join("src/ui")).unwrap();
    
    std::fs::write(root.join("README.md"), "# Test Project").unwrap();
    std::fs::write(root.join("Cargo.toml"), "[package]\nname = \"test\"").unwrap();
    std::fs::write(root.join("src/main.rs"), "fn main() {}").unwrap();
    std::fs::write(root.join("src/lib.rs"), "// lib").unwrap();
    std::fs::write(root.join("src/ui/mod.rs"), "// ui mod").unwrap();
    std::fs::write(root.join("docs/README.md"), "# Docs").unwrap();
    
    temp_dir
}

#[test]
fn test_file_tree_state_creation() {
    let state = FileTreeState::new();
    assert!(state.root_path.is_none());
    assert!(state.file_entries.is_empty());
    assert!(state.directory_children.is_empty());
}

#[test]
fn test_set_root_directory() {
    let temp_dir = create_test_directory();
    let mut state = FileTreeState::new();
    
    let result = state.set_root(temp_dir.path().to_path_buf());
    assert!(result.is_ok());
    assert_eq!(state.root_path, Some(temp_dir.path().to_path_buf()));
    assert!(!state.file_entries.is_empty());
}

#[test]
fn test_scan_directory_structure() {
    let temp_dir = create_test_directory();
    let mut state = FileTreeState::new();
    
    state.set_root(temp_dir.path().to_path_buf()).unwrap();
    
    // 检查根目录的子项
    let root_children = state.get_root_children();
    assert!(!root_children.is_empty());
    
    // 应该包含我们创建的目录和文件
    let child_names: Vec<String> = root_children
        .iter()
        .filter_map(|path| path.file_name())
        .map(|name| name.to_string_lossy().to_string())
        .collect();
    
    assert!(child_names.contains(&"src".to_string()));
    assert!(child_names.contains(&"docs".to_string()));
    assert!(child_names.contains(&"README.md".to_string()));
    assert!(child_names.contains(&"Cargo.toml".to_string()));
}

#[test]
fn test_file_entry_properties() {
    let temp_dir = create_test_directory();
    let mut state = FileTreeState::new();
    
    state.set_root(temp_dir.path().to_path_buf()).unwrap();
    
    // 测试目录条目
    let src_path = temp_dir.path().join("src");
    if let Some(entry) = state.get_file_entry(&src_path) {
        assert!(entry.is_dir);
        assert_eq!(entry.name, "src");
        assert_eq!(entry.icon, "📁");
    } else {
        panic!("src directory entry not found");
    }
    
    // 测试文件条目
    let readme_path = temp_dir.path().join("README.md");
    if let Some(entry) = state.get_file_entry(&readme_path) {
        assert!(!entry.is_dir);
        assert_eq!(entry.name, "README.md");
        assert_eq!(entry.icon, "📝");
    } else {
        panic!("README.md file entry not found");
    }
}

#[test]
fn test_file_icon_detection() {
    let temp_dir = create_test_directory();
    let mut state = FileTreeState::new();
    
    state.set_root(temp_dir.path().to_path_buf()).unwrap();
    
    // 测试不同文件类型的图标
    let test_cases = vec![
        ("Cargo.toml", "⚙️"),
        ("README.md", "📝"),
        ("src/main.rs", "🦀"),
    ];
    
    for (file_path, expected_icon) in test_cases {
        let full_path = temp_dir.path().join(file_path);
        if let Some(entry) = state.get_file_entry(&full_path) {
            assert_eq!(entry.icon, expected_icon, "Icon mismatch for {}", file_path);
        }
    }
}

#[test]
fn test_directory_children() {
    let temp_dir = create_test_directory();
    let mut state = FileTreeState::new();

    state.set_root(temp_dir.path().to_path_buf()).unwrap();

    // 测试src目录的子项 - 需要手动加载因为现在使用懒加载
    let src_path = temp_dir.path().join("src");
    state.load_directory_children(&src_path).unwrap();
    let src_children = state.get_children(&src_path);

    let child_names: Vec<String> = src_children
        .iter()
        .filter_map(|path| path.file_name())
        .map(|name| name.to_string_lossy().to_string())
        .collect();

    assert!(child_names.contains(&"main.rs".to_string()));
    assert!(child_names.contains(&"lib.rs".to_string()));
    assert!(child_names.contains(&"ui".to_string()));
}

#[test]
fn test_file_node_id() {
    let path = PathBuf::from("/test/path");
    let node_id = FileNodeId(path.clone());
    
    // 测试转换
    let path_from_node: PathBuf = node_id.clone().into();
    assert_eq!(path, path_from_node);
    
    let path_ref: &PathBuf = node_id.as_ref();
    assert_eq!(&path, path_ref);
    
    // 测试相等性
    let node_id2 = FileNodeId(path.clone());
    assert_eq!(node_id, node_id2);
}

#[test]
fn test_tree_view_state_selection() {
    let temp_dir = create_test_directory();
    let mut state = FileTreeState::new();
    
    state.set_root(temp_dir.path().to_path_buf()).unwrap();
    
    // 测试选择文件
    let readme_path = temp_dir.path().join("README.md");
    let node_id = FileNodeId(readme_path.clone());
    
    state.tree_view_state.set_one_selected(node_id.clone());
    let selected = state.tree_view_state.selected();
    
    assert_eq!(selected.len(), 1);
    assert_eq!(selected[0], node_id);
}

#[test]
fn test_refresh_functionality() {
    let temp_dir = create_test_directory();
    let mut state = FileTreeState::new();
    
    state.set_root(temp_dir.path().to_path_buf()).unwrap();
    let initial_count = state.file_entries.len();
    
    // 添加新文件
    std::fs::write(temp_dir.path().join("new_file.txt"), "new content").unwrap();
    
    // 刷新应该检测到新文件
    state.refresh().unwrap();
    let new_count = state.file_entries.len();
    
    assert!(new_count > initial_count, "Refresh should detect new files");
    
    // 验证新文件存在
    let new_file_path = temp_dir.path().join("new_file.txt");
    assert!(state.get_file_entry(&new_file_path).is_some());
}

#[test]
fn test_file_editability_detection() {
    use crate::state::is_editable_file;
    use std::path::PathBuf;

    // 可编辑的文件类型
    assert!(is_editable_file(&PathBuf::from("test.rs")));
    assert!(is_editable_file(&PathBuf::from("test.py")));
    assert!(is_editable_file(&PathBuf::from("test.js")));
    assert!(is_editable_file(&PathBuf::from("test.md")));
    assert!(is_editable_file(&PathBuf::from("test.json")));
    assert!(is_editable_file(&PathBuf::from("test.toml")));
    assert!(is_editable_file(&PathBuf::from("test.txt")));
    assert!(is_editable_file(&PathBuf::from("Makefile")));
    assert!(is_editable_file(&PathBuf::from("README")));

    // 不可编辑的文件类型
    assert!(!is_editable_file(&PathBuf::from("test.exe")));
    assert!(!is_editable_file(&PathBuf::from("test.jpg")));
    assert!(!is_editable_file(&PathBuf::from("test.pdf")));
    assert!(!is_editable_file(&PathBuf::from("test.zip")));
    assert!(!is_editable_file(&PathBuf::from("test.mp4")));
}

#[test]
fn test_text_buffer_read_only_detection() {
    let temp_dir = create_test_directory();
    let settings = crate::settings::EditorSettings::default();

    // 测试可编辑文件
    let rust_file = temp_dir.path().join("src/main.rs");
    let buffer = crate::state::TextBuffer::from_file(&rust_file, &settings).unwrap();
    assert!(!buffer.read_only, "Rust files should be editable");

    // 测试Markdown文件
    let md_file = temp_dir.path().join("README.md");
    let buffer = crate::state::TextBuffer::from_file(&md_file, &settings).unwrap();
    assert!(!buffer.read_only, "Markdown files should be editable");
}
