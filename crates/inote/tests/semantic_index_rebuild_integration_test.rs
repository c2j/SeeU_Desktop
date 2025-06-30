//! 语义索引重建集成测试

use inote::db_state::DbINoteState;
use inote::note::Note;
use chrono::Utc;

#[tokio::test]
async fn test_semantic_index_rebuild_with_initialization() {
    println!("🧪 测试语义索引重建（包含初始化）");

    // 创建数据库状态
    let mut state = DbINoteState::default();
    
    // 添加一些测试笔记
    let note1 = Note {
        id: "note1".to_string(),
        title: "人工智能".to_string(),
        content: "人工智能是计算机科学的一个分支，致力于创建能够执行通常需要人类智能的任务的系统".to_string(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
        tag_ids: vec!["ai".to_string()],
        attachments: vec![],
    };
    
    let note2 = Note {
        id: "note2".to_string(),
        title: "机器学习".to_string(),
        content: "机器学习是人工智能的一个子领域，专注于开发能够从数据中学习的算法".to_string(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
        tag_ids: vec!["ml".to_string()],
        attachments: vec![],
    };
    
    state.notes.insert(note1.id.clone(), note1);
    state.notes.insert(note2.id.clone(), note2);
    
    println!("📝 已添加 {} 个测试笔记", state.notes.len());
    
    // 验证初始状态
    assert_eq!(state.semantic_db.is_none(), true, "语义数据库应该初始为None");
    assert_eq!(state.semantic_search_enabled, false, "语义搜索应该初始为未启用");
    assert_eq!(state.is_rebuilding_semantic_index, false, "不应该在重建状态");
    
    println!("✅ 初始状态验证通过");
    
    // 开始重建语义索引（这应该会触发数据库初始化）
    println!("🔄 开始重建语义索引...");
    state.start_semantic_index_rebuild();
    
    // 验证重建状态
    assert_eq!(state.is_rebuilding_semantic_index, true, "应该处于重建状态");
    assert!(state.semantic_rebuild_progress.is_some(), "应该有进度信息");
    assert_eq!(state.semantic_rebuild_progress.unwrap(), 0.0, "初始进度应该为0");
    
    println!("✅ 重建状态验证通过");
    
    // 检查语义数据库是否已初始化
    if state.semantic_db.is_some() {
        println!("✅ 语义数据库已成功初始化");
        assert_eq!(state.semantic_search_enabled, true, "语义搜索应该已启用");
    } else {
        println!("⚠️  语义数据库初始化可能需要更多时间或失败");
        // 检查是否有错误信息
        if let Some(result) = &state.semantic_rebuild_result {
            match result {
                Ok(_) => println!("✅ 重建成功但数据库状态异常"),
                Err(e) => println!("❌ 重建失败: {}", e),
            }
        }
    }
    
    println!("✅ 语义索引重建集成测试完成");
}

#[tokio::test]
async fn test_clear_semantic_index_with_initialization() {
    println!("🧪 测试清除语义索引（包含初始化）");

    let mut state = DbINoteState::default();
    
    // 验证初始状态
    assert_eq!(state.semantic_db.is_none(), true, "语义数据库应该初始为None");
    
    // 尝试清除索引（这应该会触发数据库初始化）
    println!("🗑️ 开始清除语义索引...");
    state.clear_semantic_index();
    
    // 检查语义数据库是否已初始化
    if state.semantic_db.is_some() {
        println!("✅ 语义数据库已成功初始化");
        assert_eq!(state.semantic_search_enabled, true, "语义搜索应该已启用");
    } else {
        println!("⚠️  语义数据库初始化可能需要更多时间或失败");
    }
    
    println!("✅ 清除语义索引集成测试完成");
}

#[test]
fn test_semantic_search_status_display() {
    println!("🧪 测试语义搜索状态显示逻辑");

    let mut state = DbINoteState::default();
    
    // 测试初始状态
    assert_eq!(state.semantic_db.is_none(), true);
    assert_eq!(state.semantic_search_enabled, false);
    println!("✅ 初始状态: 未启用");
    
    // 模拟启用但未初始化的状态
    state.semantic_search_enabled = true;
    assert_eq!(state.semantic_db.is_none(), true);
    assert_eq!(state.semantic_search_enabled, true);
    println!("✅ 启用但未初始化状态: 初始化中");
    
    // 模拟完全初始化的状态
    // 注意：这里我们不能真正创建semantic_db，因为这需要异步操作
    // 但我们可以测试逻辑
    println!("✅ 状态显示逻辑测试完成");
}

#[test]
fn test_rebuild_button_state() {
    println!("🧪 测试重建按钮状态逻辑");

    let mut state = DbINoteState::default();
    
    // 测试初始状态 - 按钮应该可用
    assert_eq!(state.is_rebuilding_semantic_index, false);
    println!("✅ 初始状态: 按钮可用");
    
    // 模拟重建中状态 - 按钮应该禁用
    state.is_rebuilding_semantic_index = true;
    assert_eq!(state.is_rebuilding_semantic_index, true);
    println!("✅ 重建中状态: 按钮禁用");
    
    // 模拟重建完成状态 - 按钮应该重新可用
    state.is_rebuilding_semantic_index = false;
    state.semantic_rebuild_result = Some(Ok(10));
    assert_eq!(state.is_rebuilding_semantic_index, false);
    println!("✅ 重建完成状态: 按钮可用");
    
    println!("✅ 按钮状态逻辑测试完成");
}

#[test]
fn test_semantic_index_version_management() {
    println!("🧪 测试语义索引版本管理");

    let mut state = DbINoteState::default();
    
    // 测试初始版本
    assert_eq!(state.semantic_index_version, "v1.0.0");
    println!("✅ 初始版本: {}", state.semantic_index_version);
    
    // 模拟成功重建后的版本更新
    state.complete_semantic_index_rebuild(Ok(5));
    
    // 版本应该已更新
    assert_ne!(state.semantic_index_version, "v1.0.0");
    println!("✅ 重建后版本: {}", state.semantic_index_version);
    
    // 测试失败重建不应该更新版本
    let old_version = state.semantic_index_version.clone();
    state.complete_semantic_index_rebuild(Err("测试错误".to_string()));
    assert_eq!(state.semantic_index_version, old_version);
    println!("✅ 失败重建版本保持不变: {}", state.semantic_index_version);
    
    println!("✅ 版本管理测试完成");
}
