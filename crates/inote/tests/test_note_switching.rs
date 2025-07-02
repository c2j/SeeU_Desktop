use inote::db_state::{DbINoteState, SaveStatus};
use inote::db_storage::DbStorageManager;
use std::sync::{Arc, Mutex};

#[test]
fn test_note_switching_preserves_content() {
    // 创建内存数据库用于测试
    let storage = Arc::new(Mutex::new(
        DbStorageManager::new_memory()
            .expect("Failed to create storage")
    ));

    // 初始化状态
    let mut state = DbINoteState::default();
    state.storage = storage;
    
    // 创建测试笔记本
    let notebook_id = state.create_notebook(
        "测试笔记本".to_string(),
        "用于测试笔记切换".to_string()
    ).expect("Failed to create notebook");
    
    // 选择笔记本
    state.current_notebook = Some(0);
    
    // 创建第一个笔记
    let note1_id = state.create_note(
        "笔记A".to_string(),
        "原始内容A".to_string()
    ).expect("Failed to create note 1");
    
    // 创建第二个笔记
    let note2_id = state.create_note(
        "笔记B".to_string(),
        "原始内容B".to_string()
    ).expect("Failed to create note 2");
    
    println!("创建了两个笔记:");
    println!("笔记A ID: {}", note1_id);
    println!("笔记B ID: {}", note2_id);
    
    // 选择笔记A并编辑
    state.select_note(&note1_id);
    assert_eq!(state.note_content, "原始内容A");
    assert_eq!(state.note_title, "笔记A");
    
    // 模拟用户编辑笔记A
    state.note_content = "修改后的内容A - 这是用户的编辑".to_string();
    state.note_title = "修改后的标题A".to_string();
    state.check_note_modified();
    
    println!("编辑笔记A:");
    println!("新标题: {}", state.note_title);
    println!("新内容: {}", state.note_content);
    println!("保存状态: {:?}", state.save_status);
    
    // 验证状态为已修改
    assert_eq!(state.save_status, SaveStatus::Modified);
    
    // 切换到笔记B（这应该触发自动保存）
    println!("切换到笔记B...");
    state.select_note(&note2_id);
    
    // 验证切换到笔记B
    assert_eq!(state.note_content, "原始内容B");
    assert_eq!(state.note_title, "笔记B");
    assert_eq!(state.current_note, Some(note2_id.clone()));
    
    // 再次切换回笔记A，验证内容是否保存
    println!("切换回笔记A...");
    state.select_note(&note1_id);
    
    println!("切换回笔记A后:");
    println!("标题: {}", state.note_title);
    println!("内容: {}", state.note_content);
    
    // 验证笔记A的修改已经保存并正确加载
    assert_eq!(state.note_title, "修改后的标题A");
    assert_eq!(state.note_content, "修改后的内容A - 这是用户的编辑");
    assert_eq!(state.current_note, Some(note1_id.clone()));
    
    // 验证内存中的笔记也已更新
    if let Some(note_a) = state.notes.get(&note1_id) {
        assert_eq!(note_a.title, "修改后的标题A");
        assert_eq!(note_a.content, "修改后的内容A - 这是用户的编辑");
        println!("✅ 内存中的笔记A已正确更新");
    } else {
        panic!("笔记A在内存中不存在");
    }
    
    println!("✅ 测试通过：笔记切换时内容得到正确保存和恢复");
}

#[test]
fn test_auto_save_functionality() {
    // 创建内存数据库用于测试
    let storage = Arc::new(Mutex::new(
        DbStorageManager::new_memory()
            .expect("Failed to create storage")
    ));

    // 初始化状态
    let mut state = DbINoteState::default();
    state.storage = storage;
    
    // 创建测试笔记本
    let _notebook_id = state.create_notebook(
        "自动保存测试".to_string(),
        "测试自动保存功能".to_string()
    ).expect("Failed to create notebook");
    
    // 选择笔记本
    state.current_notebook = Some(0);
    
    // 创建笔记
    let note_id = state.create_note(
        "自动保存测试笔记".to_string(),
        "原始内容".to_string()
    ).expect("Failed to create note");
    
    // 选择笔记
    state.select_note(&note_id);
    
    // 模拟编辑
    state.note_content = "修改后的内容".to_string();
    state.note_title = "修改后的标题".to_string();
    state.check_note_modified();
    
    // 验证状态为已修改
    assert_eq!(state.save_status, SaveStatus::Modified);
    
    // 触发自动保存
    state.auto_save_if_modified();
    
    // 验证保存状态
    assert_eq!(state.save_status, SaveStatus::Saved);
    assert_eq!(state.last_saved_content, "修改后的内容");
    assert_eq!(state.last_saved_title, "修改后的标题");
    
    // 验证内存中的笔记已更新
    if let Some(note) = state.notes.get(&note_id) {
        assert_eq!(note.title, "修改后的标题");
        assert_eq!(note.content, "修改后的内容");
    } else {
        panic!("笔记在内存中不存在");
    }
    
    println!("✅ 自动保存功能测试通过");
}
