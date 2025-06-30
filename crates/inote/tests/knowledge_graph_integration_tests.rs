//! 知识图谱集成测试
//! 
//! 测试inote模块与zhushoude_duckdb的知识图谱功能集成

use inote::knowledge_graph_integration::KnowledgeGraphManager;
use inote::note::Note;

#[tokio::test]
async fn test_knowledge_graph_manager_creation() {
    // 测试禁用状态
    let manager_disabled = KnowledgeGraphManager::new(false).await;
    assert!(manager_disabled.is_ok(), "Should create disabled manager successfully");
    
    let manager = manager_disabled.unwrap();
    assert!(!manager.is_enabled(), "Manager should be disabled");
    
    // 测试启用状态（可能会失败，因为需要模型文件）
    let manager_enabled = KnowledgeGraphManager::new(true).await;
    match manager_enabled {
        Ok(manager) => {
            assert!(manager.is_enabled(), "Manager should be enabled");
            println!("✓ Knowledge graph manager created successfully");
        }
        Err(e) => {
            println!("⚠ Could not create enabled knowledge graph manager: {}", e);
            println!("  This is expected if model files are not available");
        }
    }
}

#[tokio::test]
async fn test_note_processing_disabled() {
    let manager = KnowledgeGraphManager::new(false).await
        .expect("Should create disabled manager");
    
    let note = Note::new(
        "测试笔记".to_string(),
        "张三在北京大学学习人工智能技术。".to_string(),
    );
    
    let result = manager.process_note_added(&note).await;
    assert!(result.is_ok(), "Processing should succeed even when disabled");
    
    let entities_relations = result.unwrap();
    assert!(entities_relations.is_none(), "Should return None when disabled");
}

#[tokio::test]
async fn test_semantic_search_disabled() {
    let manager = KnowledgeGraphManager::new(false).await
        .expect("Should create disabled manager");
    
    let results = manager.semantic_search("人工智能", 10).await;
    assert!(results.is_ok(), "Search should succeed even when disabled");
    
    let search_results = results.unwrap();
    assert!(search_results.is_empty(), "Should return empty results when disabled");
}

#[tokio::test]
async fn test_entity_extraction_disabled() {
    let manager = KnowledgeGraphManager::new(false).await
        .expect("Should create disabled manager");
    
    let result = manager.extract_entities_and_relations("测试文本").await;
    assert!(result.is_ok(), "Extraction should succeed even when disabled");
    
    let extraction_result = result.unwrap();
    assert!(extraction_result.is_none(), "Should return None when disabled");
}

#[tokio::test]
async fn test_knowledge_graph_stats_disabled() {
    let manager = KnowledgeGraphManager::new(false).await
        .expect("Should create disabled manager");
    
    let stats = manager.get_stats().await;
    assert!(stats.is_ok(), "Stats should succeed even when disabled");
    
    let stats_result = stats.unwrap();
    assert!(stats_result.is_none(), "Should return None when disabled");
}

#[tokio::test]
async fn test_note_to_document_conversion() {
    let manager = KnowledgeGraphManager::new(false).await
        .expect("Should create manager");
    
    let note = Note::new(
        "测试笔记标题".to_string(),
        "这是测试笔记的内容，包含一些实体如张三和北京大学。".to_string(),
    );
    
    // 通过反射或其他方式测试内部转换逻辑
    // 由于note_to_document是私有方法，我们通过process_note_added来间接测试
    let result = manager.process_note_added(&note).await;
    assert!(result.is_ok(), "Note processing should not fail");
}

#[tokio::test]
async fn test_manager_enable_disable() {
    let mut manager = KnowledgeGraphManager::new(false).await
        .expect("Should create manager");
    
    assert!(!manager.is_enabled(), "Should start disabled");
    
    manager.enable();
    assert!(manager.is_enabled(), "Should be enabled after enable()");
    
    manager.disable();
    assert!(!manager.is_enabled(), "Should be disabled after disable()");
}

#[tokio::test]
async fn test_multiple_notes_processing() {
    let manager = KnowledgeGraphManager::new(false).await
        .expect("Should create manager");
    
    let notes = vec![
        Note::new("笔记1".to_string(), "张三在清华大学学习。".to_string()),
        Note::new("笔记2".to_string(), "李四在北京大学工作。".to_string()),
        Note::new("笔记3".to_string(), "王五研究人工智能技术。".to_string()),
    ];
    
    for note in &notes {
        let result = manager.process_note_added(note).await;
        assert!(result.is_ok(), "Each note should be processed successfully");
    }
    
    // 测试笔记更新
    let updated_note = Note::new("笔记1".to_string(), "张三在清华大学研究机器学习。".to_string());
    let result = manager.process_note_updated(&updated_note).await;
    assert!(result.is_ok(), "Note update should be processed successfully");
}

#[tokio::test]
async fn test_error_handling() {
    // 测试各种错误情况的处理
    let manager = KnowledgeGraphManager::new(false).await
        .expect("Should create manager");
    
    // 测试空内容笔记
    let empty_note = Note::new("".to_string(), "".to_string());
    let result = manager.process_note_added(&empty_note).await;
    assert!(result.is_ok(), "Empty note should be handled gracefully");
    
    // 测试非常长的内容
    let long_content = "很长的内容 ".repeat(1000);
    let long_note = Note::new("长笔记".to_string(), long_content);
    let result = manager.process_note_added(&long_note).await;
    assert!(result.is_ok(), "Long note should be handled gracefully");
    
    // 测试特殊字符
    let special_note = Note::new(
        "特殊字符测试".to_string(),
        "包含特殊字符的文本：@#$%^&*()[]{}|\\:;\"'<>,.?/~`".to_string(),
    );
    let result = manager.process_note_added(&special_note).await;
    assert!(result.is_ok(), "Special characters should be handled gracefully");
}

#[tokio::test]
async fn test_concurrent_processing() {
    let manager = std::sync::Arc::new(
        KnowledgeGraphManager::new(false).await
            .expect("Should create manager")
    );
    
    let notes = vec![
        Note::new("并发笔记1".to_string(), "内容1".to_string()),
        Note::new("并发笔记2".to_string(), "内容2".to_string()),
        Note::new("并发笔记3".to_string(), "内容3".to_string()),
    ];
    
    // 并发处理多个笔记
    let mut handles = Vec::new();
    for note in notes {
        let manager_clone = manager.clone();
        let handle = tokio::spawn(async move {
            manager_clone.process_note_added(&note).await
        });
        handles.push(handle);
    }
    
    // 等待所有任务完成
    for handle in handles {
        let result = handle.await.expect("Task should complete");
        assert!(result.is_ok(), "Concurrent processing should succeed");
    }
}

#[test]
fn test_semantic_search_result_structure() {
    use inote::knowledge_graph_integration::SemanticSearchResult;
    
    let result = SemanticSearchResult {
        note_id: "test_id".to_string(),
        title: "测试标题".to_string(),
        content: "测试内容".to_string(),
        similarity_score: 0.85,
        metadata: Some(serde_json::json!({"test": "value"})),
    };
    
    assert_eq!(result.note_id, "test_id");
    assert_eq!(result.title, "测试标题");
    assert_eq!(result.content, "测试内容");
    assert_eq!(result.similarity_score, 0.85);
    assert!(result.metadata.is_some());
}

#[test]
fn test_entity_relation_info_structure() {
    use inote::knowledge_graph_integration::EntityRelationInfo;
    
    let relation_info = EntityRelationInfo {
        source_entity: "张三".to_string(),
        target_entity: "北京大学".to_string(),
        relation_type: "WORKS_AT".to_string(),
        confidence: 0.9,
        weight: 1.5,
    };
    
    assert_eq!(relation_info.source_entity, "张三");
    assert_eq!(relation_info.target_entity, "北京大学");
    assert_eq!(relation_info.relation_type, "WORKS_AT");
    assert_eq!(relation_info.confidence, 0.9);
    assert_eq!(relation_info.weight, 1.5);
}

#[test]
fn test_entity_info_conversion() {
    use inote::knowledge_graph_integration::EntityInfo;
    use zhushoude_duckdb::nlp::{Entity, EntityType};
    use std::collections::HashMap;
    
    let entity = Entity {
        text: "张三".to_string(),
        entity_type: EntityType::Person,
        start: 0,
        end: 2,
        confidence: 0.95,
        properties: HashMap::new(),
    };
    
    let entity_info = EntityInfo::from(&entity);
    
    assert_eq!(entity_info.text, "张三");
    assert_eq!(entity_info.entity_type, "PERSON");
    assert_eq!(entity_info.confidence, 0.95);
    assert_eq!(entity_info.start, 0);
    assert_eq!(entity_info.end, 2);
}

#[test]
fn test_relation_info_conversion() {
    use inote::knowledge_graph_integration::RelationInfo;
    use zhushoude_duckdb::nlp::{EntityRelation, Entity, EntityType, RelationType};
    use std::collections::HashMap;
    
    let subject = Entity {
        text: "张三".to_string(),
        entity_type: EntityType::Person,
        start: 0,
        end: 2,
        confidence: 0.9,
        properties: HashMap::new(),
    };
    
    let object = Entity {
        text: "北京大学".to_string(),
        entity_type: EntityType::Organization,
        start: 3,
        end: 6,
        confidence: 0.85,
        properties: HashMap::new(),
    };
    
    let relation = EntityRelation {
        subject,
        relation_type: RelationType::WorksAt,
        object,
        confidence: 0.8,
        relation_span: Some((0, 6)),
        properties: HashMap::new(),
    };
    
    let relation_info = RelationInfo::from(&relation);
    
    assert_eq!(relation_info.subject.text, "张三");
    assert_eq!(relation_info.object.text, "北京大学");
    assert_eq!(relation_info.relation_type, "WORKS_AT");
    assert_eq!(relation_info.confidence, 0.8);
}
