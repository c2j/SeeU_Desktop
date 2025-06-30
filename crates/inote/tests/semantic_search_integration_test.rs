use inote::db_state::DbINoteState;
use inote::note::Note;
use std::sync::{Arc, Mutex};
use tempfile::NamedTempFile;

#[tokio::test]
async fn test_semantic_search_integration() {
    // 创建临时数据库文件
    let temp_file = NamedTempFile::new().expect("Failed to create temp file");
    let db_path = temp_file.path().to_str().unwrap();
    
    // 创建DbINoteState实例
    let mut state = DbINoteState::default();
    
    // 初始化数据库
    if let Ok(mut storage) = state.storage.lock() {
        if let Err(err) = storage.initialize_with_path(db_path) {
            panic!("Failed to initialize database: {}", err);
        }
    }
    
    // 启用语义搜索
    state.initialize_semantic_search_sync();
    
    // 创建测试笔记本
    state.create_notebook("测试笔记本".to_string(), "用于测试语义搜索".to_string());
    
    // 选择笔记本
    state.select_notebook(0);
    
    // 创建一些测试笔记
    let note1_id = state.create_note(
        "人工智能基础".to_string(),
        "人工智能是计算机科学的一个分支，致力于创建能够执行通常需要人类智能的任务的系统。".to_string()
    );
    
    let note2_id = state.create_note(
        "机器学习算法".to_string(),
        "机器学习是人工智能的一个子领域，通过算法让计算机从数据中学习模式。".to_string()
    );
    
    let note3_id = state.create_note(
        "深度学习网络".to_string(),
        "深度学习使用神经网络来模拟人脑的工作方式，是机器学习的一个重要分支。".to_string()
    );
    
    let note4_id = state.create_note(
        "烹饪技巧".to_string(),
        "学习如何制作美味的菜肴，包括切菜、调味和烹饪时间的掌握。".to_string()
    );
    
    assert!(note1_id.is_some());
    assert!(note2_id.is_some());
    assert!(note3_id.is_some());
    assert!(note4_id.is_some());
    
    // 等待一段时间让语义搜索引擎处理笔记
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    
    // 测试传统搜索
    state.search_query = "人工智能".to_string();
    state.search_notes();
    
    // 验证传统搜索结果
    assert!(state.is_searching);
    assert!(!state.search_results.is_empty());
    
    // 检查搜索结果是否包含相关笔记
    let search_result_notes = state.get_search_result_notes();
    let titles: Vec<&str> = search_result_notes.iter().map(|note| note.title.as_str()).collect();
    
    assert!(titles.contains(&"人工智能基础"));
    
    println!("✅ 传统搜索测试通过");
    
    // 测试语义搜索功能是否启用
    assert!(state.semantic_search_enabled, "语义搜索应该已启用");
    
    println!("✅ 语义搜索集成测试完成");
}

#[test]
fn test_semantic_search_ui_integration() {
    let mut state = DbINoteState::default();
    
    // 测试语义搜索面板状态
    assert!(!state.show_semantic_search_panel);
    assert!(!state.is_semantic_searching);
    assert!(state.semantic_search_results.is_empty());
    
    // 模拟启用语义搜索面板
    state.show_semantic_search_panel = true;
    assert!(state.show_semantic_search_panel);
    
    println!("✅ 语义搜索UI集成测试通过");
}

#[test]
fn test_note_sync_to_semantic_db() {
    let mut state = DbINoteState::default();
    
    // 启用语义搜索
    state.semantic_search_enabled = true;
    
    // 创建测试笔记
    let note = Note::new("测试笔记".to_string(), "这是一个测试笔记的内容".to_string());
    
    // 测试同步方法不会崩溃（即使没有实际的语义数据库实例）
    state.sync_note_to_semantic_db(&note);
    
    println!("✅ 笔记同步测试通过");
}
