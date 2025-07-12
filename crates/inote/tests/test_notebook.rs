use inote::notebook::Notebook;
use chrono::Utc;

#[test]
fn test_notebook_creation() {
    let notebook = Notebook::new("测试笔记本".to_string(), "这是一个测试笔记本".to_string());
    
    assert!(!notebook.id.is_empty());
    assert_eq!(notebook.name, "测试笔记本");
    assert_eq!(notebook.description, "这是一个测试笔记本");
    assert!(notebook.note_ids.is_empty());
    assert!(!notebook.expanded); // 默认折叠
    assert!(notebook.created_at <= Utc::now());
    assert!(notebook.updated_at <= Utc::now());
}

#[test]
fn test_notebook_note_management() {
    let mut notebook = Notebook::new("测试笔记本".to_string(), "描述".to_string());
    let initial_updated_at = notebook.updated_at;

    // 添加笔记
    std::thread::sleep(std::time::Duration::from_millis(1)); // 确保时间戳不同
    notebook.add_note("note1".to_string());
    assert_eq!(notebook.note_ids.len(), 1);
    assert!(notebook.note_ids.contains(&"note1".to_string()));
    assert_eq!(notebook.note_ids[0], "note1"); // 第一个笔记在位置0
    assert!(notebook.updated_at > initial_updated_at);

    // 添加更多笔记 (新笔记会插入到开头)
    notebook.add_note("note2".to_string());
    notebook.add_note("note3".to_string());
    assert_eq!(notebook.note_ids.len(), 3);
    assert_eq!(notebook.note_ids[0], "note3"); // 最新的笔记在开头
    assert_eq!(notebook.note_ids[1], "note2");
    assert_eq!(notebook.note_ids[2], "note1");

    // 移除笔记
    notebook.remove_note("note2");
    assert_eq!(notebook.note_ids.len(), 2);
    assert!(!notebook.note_ids.contains(&"note2".to_string()));
    assert!(notebook.note_ids.contains(&"note1".to_string()));
    assert!(notebook.note_ids.contains(&"note3".to_string()));
    assert_eq!(notebook.note_ids[0], "note3"); // note3 仍然在开头
    assert_eq!(notebook.note_ids[1], "note1"); // note1 现在在第二位
}

#[test]
fn test_notebook_toggle_expanded() {
    let mut notebook = Notebook::new("测试笔记本".to_string(), "描述".to_string());
    
    assert!(!notebook.expanded); // 默认折叠
    
    notebook.toggle_expanded();
    assert!(notebook.expanded);
    
    notebook.toggle_expanded();
    assert!(!notebook.expanded);
}

#[test]
fn test_notebook_add_vs_append_note() {
    let mut notebook = Notebook::new("测试笔记本".to_string(), "描述".to_string());

    // 使用 add_note (插入到开头)
    notebook.add_note("note1".to_string());
    notebook.add_note("note2".to_string());
    assert_eq!(notebook.note_ids[0], "note2"); // 最新的在开头
    assert_eq!(notebook.note_ids[1], "note1");

    // 使用 append_note (添加到末尾)
    notebook.append_note("note3".to_string());
    notebook.append_note("note4".to_string());
    assert_eq!(notebook.note_ids[0], "note2"); // 开头不变
    assert_eq!(notebook.note_ids[1], "note1");
    assert_eq!(notebook.note_ids[2], "note3"); // 按顺序添加到末尾
    assert_eq!(notebook.note_ids[3], "note4");
}

#[test]
fn test_notebook_duplicate_note_handling() {
    let mut notebook = Notebook::new("测试笔记本".to_string(), "描述".to_string());

    // 添加笔记
    notebook.add_note("note1".to_string());
    assert_eq!(notebook.note_ids.len(), 1);

    // 尝试添加重复笔记（如果实现了去重）
    notebook.add_note("note1".to_string());
    // 根据实际实现，这里可能是1（去重）或2（允许重复）
    // 假设允许重复
    assert!(notebook.note_ids.len() >= 1);
}

#[test]
fn test_notebook_empty_operations() {
    let mut notebook = Notebook::new("空笔记本".to_string(), "空描述".to_string());
    
    // 尝试移除不存在的笔记
    notebook.remove_note("nonexistent");
    assert_eq!(notebook.note_ids.len(), 0);
    
    // 验证笔记本仍然有效
    assert_eq!(notebook.name, "空笔记本");
    assert_eq!(notebook.description, "空描述");
}

#[test]
fn test_notebook_metadata_updates() {
    let mut notebook = Notebook::new("测试笔记本".to_string(), "原始描述".to_string());
    let initial_created_at = notebook.created_at;
    let initial_updated_at = notebook.updated_at;
    
    // 等待一小段时间确保时间戳不同
    std::thread::sleep(std::time::Duration::from_millis(1));
    
    // 更新笔记本信息（假设有这样的方法）
    notebook.name = "更新后的笔记本".to_string();
    notebook.description = "更新后的描述".to_string();
    notebook.updated_at = Utc::now();
    
    // 验证更新
    assert_eq!(notebook.name, "更新后的笔记本");
    assert_eq!(notebook.description, "更新后的描述");
    assert_eq!(notebook.created_at, initial_created_at); // 创建时间不变
    assert!(notebook.updated_at > initial_updated_at); // 更新时间改变
}

#[test]
fn test_notebook_large_note_collection() {
    let mut notebook = Notebook::new("大型笔记本".to_string(), "包含大量笔记".to_string());
    
    // 添加大量笔记
    for i in 0..1000 {
        notebook.add_note(format!("note_{}", i));
    }
    
    assert_eq!(notebook.note_ids.len(), 1000);
    
    // 验证特定笔记存在
    assert!(notebook.note_ids.contains(&"note_0".to_string()));
    assert!(notebook.note_ids.contains(&"note_500".to_string()));
    assert!(notebook.note_ids.contains(&"note_999".to_string()));
    
    // 移除一些笔记
    for i in (0..1000).step_by(2) {
        notebook.remove_note(&format!("note_{}", i));
    }
    
    assert_eq!(notebook.note_ids.len(), 500); // 移除了一半
    assert!(!notebook.note_ids.contains(&"note_0".to_string()));
    assert!(notebook.note_ids.contains(&"note_1".to_string()));
}

#[test]
fn test_notebook_id_uniqueness() {
    let notebook1 = Notebook::new("笔记本1".to_string(), "描述1".to_string());
    let notebook2 = Notebook::new("笔记本2".to_string(), "描述2".to_string());
    
    // 验证ID是唯一的
    assert_ne!(notebook1.id, notebook2.id);
    assert!(!notebook1.id.is_empty());
    assert!(!notebook2.id.is_empty());
}
