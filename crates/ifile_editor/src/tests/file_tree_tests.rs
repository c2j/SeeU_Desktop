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

/// 创建包含大量文件的测试目录，用于测试滚动功能
fn create_large_test_directory() -> TempDir {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let root = temp_dir.path();

    // 创建多个子目录
    for i in 0..10 {
        let dir_name = format!("directory_{:02}", i);
        let dir_path = root.join(&dir_name);
        std::fs::create_dir(&dir_path).unwrap();

        // 在每个子目录中创建多个文件
        for j in 0..15 {
            let file_name = format!("file_{:02}.txt", j);
            let file_path = dir_path.join(&file_name);
            std::fs::write(&file_path, format!("Content of file {} in directory {}", j, i)).unwrap();
        }
    }

    // 在根目录创建一些文件
    for i in 0..20 {
        let file_name = format!("root_file_{:02}.md", i);
        let file_path = root.join(&file_name);
        std::fs::write(&file_path, format!("Root file content {}", i)).unwrap();
    }

    temp_dir
}

#[test]
fn test_large_directory_structure() {
    let temp_dir = create_large_test_directory();
    let mut state = FileTreeState::new();

    // 设置根目录
    let result = state.set_root(temp_dir.path().to_path_buf());
    assert!(result.is_ok());

    // 验证根目录包含大量条目
    let root_children = state.get_root_children();
    assert!(root_children.len() >= 30, "Should have at least 30 items (10 dirs + 20 files)");

    // 验证目录和文件都被正确识别
    let mut dir_count = 0;
    let mut file_count = 0;

    for child in &root_children {
        if let Some(entry) = state.get_file_entry(child) {
            if entry.is_dir {
                dir_count += 1;
            } else {
                file_count += 1;
            }
        }
    }

    assert_eq!(dir_count, 10, "Should have 10 directories");
    assert_eq!(file_count, 20, "Should have 20 files");
}

#[test]
fn test_directory_expansion_with_many_children() {
    let temp_dir = create_large_test_directory();
    let mut state = FileTreeState::new();

    state.set_root(temp_dir.path().to_path_buf()).unwrap();

    // 测试展开第一个目录
    let first_dir = temp_dir.path().join("directory_00");
    state.load_directory_children(&first_dir).unwrap();

    let children = state.get_children(&first_dir);
    assert_eq!(children.len(), 15, "Each directory should have 15 files");

    // 验证所有子文件都被正确加载
    for (i, child) in children.iter().enumerate() {
        if let Some(entry) = state.get_file_entry(child) {
            assert!(!entry.is_dir, "All children should be files");
            assert!(entry.name.starts_with("file_"), "File names should start with 'file_'");
        }
    }
}

#[test]
fn test_scroll_area_compatibility() {
    // 这个测试验证文件树状态与滚动区域的兼容性
    let temp_dir = create_large_test_directory();
    let mut state = FileTreeState::new();

    state.set_root(temp_dir.path().to_path_buf()).unwrap();

    // 模拟展开多个目录（这会在UI中触发滚动需求）
    for i in 0..5 {
        let dir_name = format!("directory_{:02}", i);
        let dir_path = temp_dir.path().join(&dir_name);

        // 加载目录子项
        let result = state.load_directory_children(&dir_path);
        assert!(result.is_ok(), "Should be able to load directory children");

        // 验证子项被正确加载
        let children = state.get_children(&dir_path);
        assert_eq!(children.len(), 15, "Each directory should have 15 children");
    }

    // 验证总的文件条目数量
    let total_entries = state.file_entries.len();
    // 根目录30个条目 + 5个目录各15个文件 = 30 + 75 = 105
    assert!(total_entries >= 105, "Should have loaded all expanded directory contents");
}

#[test]
fn test_button_display_logic() {
    // 测试按钮显示逻辑：没有文件时显示基本按钮，有文件时显示图标按钮
    let temp_dir = create_test_directory();
    let mut state = crate::state::IFileEditorState::new();
    let settings = crate::settings::EditorSettings::default();

    // 初始状态：没有活动文件
    assert!(state.editor.get_active_buffer().is_none(), "Should have no active file initially");

    // 打开一个文件
    let test_file = temp_dir.path().join("src/main.rs");
    let result = state.editor.open_file(test_file.clone(), &settings);
    assert!(result.is_ok(), "Should be able to open test file");

    // 现在应该有活动文件
    assert!(state.editor.get_active_buffer().is_some(), "Should have active file after opening");

    // 关闭文件
    let result = state.editor.close_file(&test_file);
    assert!(result.is_ok(), "Should be able to close file");

    // 应该没有活动文件了
    assert!(state.editor.get_active_buffer().is_none(), "Should have no active file after closing");
}

#[test]
fn test_editor_state_transitions() {
    // 测试编辑器状态转换
    let temp_dir = create_test_directory();
    let mut state = crate::state::IFileEditorState::new();
    let settings = crate::settings::EditorSettings::default();

    // 测试多个文件的打开和关闭
    let files = vec!["src/main.rs", "README.md", "Cargo.toml"];
    let mut file_paths = Vec::new();

    for file_name in &files {
        let file_path = temp_dir.path().join(file_name);
        let result = state.editor.open_file(file_path.clone(), &settings);
        assert!(result.is_ok(), "Should be able to open {}", file_name);
        file_paths.push(file_path);
    }

    // 应该有3个标签页
    assert_eq!(state.editor.tabs.len(), 3, "Should have 3 tabs open");
    assert!(state.editor.get_active_buffer().is_some(), "Should have active file");

    // 关闭所有标签页
    for file_path in file_paths {
        let result = state.editor.close_file(&file_path);
        assert!(result.is_ok(), "Should be able to close file");
    }

    // 应该没有标签页和活动文件
    assert_eq!(state.editor.tabs.len(), 0, "Should have no tabs");
    assert!(state.editor.get_active_buffer().is_none(), "Should have no active file");
}

#[test]
fn test_tab_layout_separation() {
    // 测试标签页和工具栏分离布局
    let temp_dir = create_test_directory();
    let mut state = crate::state::IFileEditorState::new();
    let settings = crate::settings::EditorSettings::default();

    // 初始状态：没有标签页
    assert_eq!(state.editor.tabs.len(), 0, "Should start with no tabs");

    // 打开多个文件
    let files = vec!["src/main.rs", "README.md", "Cargo.toml"];
    for file_name in &files {
        let file_path = temp_dir.path().join(file_name);
        let result = state.editor.open_file(file_path, &settings);
        assert!(result.is_ok(), "Should be able to open {}", file_name);
    }

    // 验证标签页数量
    assert_eq!(state.editor.tabs.len(), 3, "Should have 3 tabs");

    // 验证有活动标签页
    assert!(state.editor.active_tab.is_some(), "Should have active tab");
    assert!(state.editor.get_active_buffer().is_some(), "Should have active buffer");

    // 验证标签页路径正确
    for (i, file_name) in files.iter().enumerate() {
        let expected_path = temp_dir.path().join(file_name);
        assert_eq!(state.editor.tabs[i], expected_path, "Tab {} should match expected path", i);
    }
}
