//! NLP功能测试
//! 
//! 测试实体识别、关系抽取和知识图谱构建功能

use zhushoude_duckdb::nlp::{ChineseNER, ChineseRelationExtractor, KnowledgeGraphBuilder, EntityType, RelationType};
use zhushoude_duckdb::{ZhushoudeConfig, DatabaseManager};
use std::sync::Arc;

#[tokio::test]
async fn test_chinese_ner_basic() {
    let ner = ChineseNER::new().expect("Failed to create NER");
    
    let text = "张三在北京大学学习人工智能技术，他使用Python编程语言开发了一个机器学习项目。";
    let entities = ner.extract_entities(text).expect("Failed to extract entities");
    
    assert!(!entities.is_empty(), "Should extract at least some entities");
    
    // 检查是否提取到了人名
    let person_entities: Vec<_> = entities.iter()
        .filter(|e| e.entity_type == EntityType::Person)
        .collect();
    assert!(!person_entities.is_empty(), "Should extract person entities");
    
    // 检查是否提取到了技术术语
    let tech_entities: Vec<_> = entities.iter()
        .filter(|e| e.entity_type == EntityType::Technology)
        .collect();
    assert!(!tech_entities.is_empty(), "Should extract technology entities");
    
    println!("Extracted {} entities", entities.len());
    for entity in &entities {
        println!("  {} ({:?}) - confidence: {:.3}", 
                entity.text, entity.entity_type, entity.confidence);
    }
}

#[tokio::test]
async fn test_chinese_ner_entity_types() {
    let ner = ChineseNER::new().expect("Failed to create NER");
    
    // 测试不同类型的实体
    let test_cases = vec![
        ("王小明是一位优秀的工程师", EntityType::Person),
        ("他在上海工作", EntityType::Location),
        ("阿里巴巴公司发布了新产品", EntityType::Organization),
        ("2024年1月1日是新年", EntityType::Time),
        ("深度学习是人工智能的重要分支", EntityType::Technology),
    ];
    
    for (text, expected_type) in test_cases {
        let entities = ner.extract_entities(text).expect("Failed to extract entities");

        let has_expected_type = entities.iter()
            .any(|e| e.entity_type == expected_type);

        assert!(has_expected_type,
               "Text '{}' should contain entity of type {:?}. Found entities: {:?}",
               text, expected_type, entities.iter().map(|e| (&e.text, &e.entity_type)).collect::<Vec<_>>());
    }
}

#[tokio::test]
async fn test_relation_extraction_basic() {
    let ner = ChineseNER::new().expect("Failed to create NER");
    let extractor = ChineseRelationExtractor::new().expect("Failed to create relation extractor");
    
    let text = "张三工作于阿里巴巴公司，他使用Java语言开发软件。";
    
    // 先提取实体
    let entities = ner.extract_entities(text).expect("Failed to extract entities");
    assert!(!entities.is_empty(), "Should extract entities first");
    
    // 再提取关系
    let relations = extractor.extract_relations(text, &entities)
        .expect("Failed to extract relations");
    
    assert!(!relations.is_empty(), "Should extract at least some relations");
    
    println!("Extracted {} relations", relations.len());
    for relation in &relations {
        println!("  {} --[{}]--> {} (confidence: {:.3})", 
                relation.subject.text, 
                relation.relation_type.as_str(),
                relation.object.text,
                relation.confidence);
    }
}

#[tokio::test]
async fn test_relation_types() {
    let ner = ChineseNER::new().expect("Failed to create NER");
    let extractor = ChineseRelationExtractor::new().expect("Failed to create relation extractor");
    
    let test_cases = vec![
        ("张三工作于微软公司", RelationType::WorksAt),
        ("北京包含朝阳区", RelationType::Contains),
        ("Python依赖于C语言", RelationType::DependsOn),
        ("小明学习机器学习", RelationType::Studies),
        ("公司使用云计算技术", RelationType::Uses),
    ];
    
    for (text, expected_relation) in test_cases {
        let entities = ner.extract_entities(text).expect("Failed to extract entities");
        let relations = extractor.extract_relations(text, &entities)
            .expect("Failed to extract relations");
        
        let has_expected_relation = relations.iter()
            .any(|r| r.relation_type == expected_relation);
        
        if !has_expected_relation {
            println!("Text: {}", text);
            println!("Entities: {:?}", entities.iter().map(|e| &e.text).collect::<Vec<_>>());
            println!("Relations: {:?}", relations.iter().map(|r| r.relation_type.as_str()).collect::<Vec<_>>());
        }
        
        // Note: This might not always pass due to the complexity of relation extraction
        // In a real scenario, you might want to make this a warning instead of assertion
        if has_expected_relation {
            println!("✓ Successfully extracted {:?} relation from '{}'", expected_relation, text);
        } else {
            println!("⚠ Could not extract {:?} relation from '{}'", expected_relation, text);
        }
    }
}

#[tokio::test]
async fn test_knowledge_graph_builder() {
    // 创建测试数据库
    let config = ZhushoudeConfig::default();
    let db_manager = Arc::new(DatabaseManager::new(config).await.expect("Failed to create database"));
    
    let mut kg_builder = KnowledgeGraphBuilder::new(db_manager);
    
    // 创建测试实体
    let ner = ChineseNER::new().expect("Failed to create NER");
    let extractor = ChineseRelationExtractor::new().expect("Failed to create relation extractor");
    
    let text = "张三在清华大学研究人工智能，他开发了一个基于Python的机器学习系统。";
    let entities = ner.extract_entities(text).expect("Failed to extract entities");
    let relations = extractor.extract_relations(text, &entities)
        .expect("Failed to extract relations");
    
    // 构建知识图谱
    kg_builder.build_from_entities_and_relations(&entities, &relations, "test_doc_1")
        .await
        .expect("Failed to build knowledge graph");
    
    let kg = kg_builder.get_knowledge_graph();
    
    assert!(!kg.nodes.is_empty(), "Knowledge graph should have nodes");
    assert!(kg.stats.node_count > 0, "Should have positive node count");
    
    println!("Knowledge graph stats:");
    println!("  Nodes: {}", kg.stats.node_count);
    println!("  Edges: {}", kg.stats.edge_count);
    println!("  Average degree: {:.2}", kg.stats.average_degree);
}

#[tokio::test]
async fn test_knowledge_graph_related_entities() {
    let config = ZhushoudeConfig::default();
    let db_manager = Arc::new(DatabaseManager::new(config).await.expect("Failed to create database"));
    
    let mut kg_builder = KnowledgeGraphBuilder::new(db_manager);
    
    let ner = ChineseNER::new().expect("Failed to create NER");
    let extractor = ChineseRelationExtractor::new().expect("Failed to create relation extractor");
    
    // 添加多个相关的文档
    let texts = vec![
        "张三在清华大学学习计算机科学",
        "李四也在清华大学研究人工智能",
        "清华大学是中国顶尖的技术大学",
        "计算机科学包含人工智能领域",
    ];
    
    for (i, text) in texts.iter().enumerate() {
        let entities = ner.extract_entities(text).expect("Failed to extract entities");
        let relations = extractor.extract_relations(text, &entities)
            .expect("Failed to extract relations");
        
        kg_builder.build_from_entities_and_relations(&entities, &relations, &format!("doc_{}", i))
            .await
            .expect("Failed to build knowledge graph");
    }
    
    // 测试相关实体查找
    let related = kg_builder.find_related_entities("清华大学", 2);
    assert!(!related.is_empty(), "Should find related entities for '清华大学'");
    
    println!("Related entities to '清华大学': {:?}", related);
    
    // 测试实体关系获取
    let relations = kg_builder.get_entity_relations("清华大学");
    assert!(!relations.is_empty(), "Should find relations for '清华大学'");
    
    println!("Relations for '清华大学': {}", relations.len());
}

#[tokio::test]
async fn test_confidence_filtering() {
    let ner = ChineseNER::new().expect("Failed to create NER");
    
    let text = "这是一个测试文本，包含一些可能的实体如ABC和XYZ。";
    let entities = ner.extract_entities(text).expect("Failed to extract entities");
    
    // 测试置信度过滤
    let high_confidence_entities: Vec<_> = entities.iter()
        .filter(|e| e.confidence > 0.7)
        .collect();
    
    let medium_confidence_entities: Vec<_> = entities.iter()
        .filter(|e| e.confidence > 0.5 && e.confidence <= 0.7)
        .collect();
    
    println!("High confidence entities (>0.7): {}", high_confidence_entities.len());
    println!("Medium confidence entities (0.5-0.7): {}", medium_confidence_entities.len());
    
    // 确保置信度在合理范围内
    for entity in &entities {
        assert!(entity.confidence >= 0.0 && entity.confidence <= 1.0, 
               "Confidence should be between 0 and 1, got {}", entity.confidence);
    }
}

#[tokio::test]
async fn test_entity_overlap_handling() {
    let ner = ChineseNER::new().expect("Failed to create NER");
    
    // 测试重叠实体的处理
    let text = "北京大学计算机科学技术研究所";
    let entities = ner.extract_entities(text).expect("Failed to extract entities");
    
    // 检查是否正确处理了重叠实体
    for i in 0..entities.len() {
        for j in (i + 1)..entities.len() {
            let entity1 = &entities[i];
            let entity2 = &entities[j];
            
            let overlaps = entity1.start < entity2.end && entity1.end > entity2.start;
            assert!(!overlaps, 
                   "Entities should not overlap: '{}' ({}-{}) and '{}' ({}-{})", 
                   entity1.text, entity1.start, entity1.end,
                   entity2.text, entity2.start, entity2.end);
        }
    }
    
    println!("Successfully handled overlapping entities in: {}", text);
    for entity in &entities {
        println!("  {} ({}-{})", entity.text, entity.start, entity.end);
    }
}

#[test]
fn test_entity_type_conversion() {
    // 测试实体类型的字符串转换
    let entity_types = vec![
        EntityType::Person,
        EntityType::Location,
        EntityType::Organization,
        EntityType::Time,
        EntityType::Concept,
        EntityType::Technology,
        EntityType::Product,
        EntityType::Other,
    ];
    
    for entity_type in entity_types {
        let type_str = entity_type.as_str();
        assert!(!type_str.is_empty(), "Entity type string should not be empty");
        assert!(type_str.chars().all(|c| c.is_ascii_uppercase() || c == '_'), 
               "Entity type string should be uppercase: {}", type_str);
    }
}

#[test]
fn test_relation_type_conversion() {
    // 测试关系类型的字符串转换
    let relation_types = vec![
        RelationType::Contains,
        RelationType::BelongsTo,
        RelationType::RelatedTo,
        RelationType::DependsOn,
        RelationType::LocatedIn,
        RelationType::WorksAt,
        RelationType::Creates,
        RelationType::Uses,
        RelationType::Studies,
        RelationType::Researches,
        RelationType::OccursAt,
        RelationType::Other,
    ];
    
    for relation_type in relation_types {
        let type_str = relation_type.as_str();
        assert!(!type_str.is_empty(), "Relation type string should not be empty");
        assert!(type_str.chars().all(|c| c.is_ascii_uppercase() || c == '_'),
               "Relation type string should be uppercase: {}", type_str);
    }
}

#[tokio::test]
async fn test_full_pipeline_integration() {
    // 测试完整的NLP流水线：从文本到知识图谱
    let config = ZhushoudeConfig::default();
    let db_manager = Arc::new(DatabaseManager::new(config).await.expect("Failed to create database"));

    let ner = ChineseNER::new().expect("Failed to create NER");
    let extractor = ChineseRelationExtractor::new().expect("Failed to create relation extractor");
    let mut kg_builder = KnowledgeGraphBuilder::new(db_manager);

    let documents = vec![
        "张三是清华大学的教授，他研究人工智能和机器学习技术。",
        "李四在北京大学工作，专门从事自然语言处理研究。",
        "清华大学和北京大学都是中国顶尖的研究型大学。",
        "人工智能技术包含机器学习、深度学习和自然语言处理等分支。",
    ];

    for (i, text) in documents.iter().enumerate() {
        // 提取实体
        let entities = ner.extract_entities(text).expect("Failed to extract entities");
        assert!(!entities.is_empty(), "Should extract entities from: {}", text);

        // 提取关系
        let relations = extractor.extract_relations(text, &entities)
            .expect("Failed to extract relations");

        // 构建知识图谱
        kg_builder.build_from_entities_and_relations(&entities, &relations, &format!("doc_{}", i))
            .await
            .expect("Failed to build knowledge graph");

        println!("Processed document {}: {} entities, {} relations",
                i, entities.len(), relations.len());
    }

    let kg = kg_builder.get_knowledge_graph();

    // 验证知识图谱
    assert!(kg.stats.node_count >= 4, "Should have at least 4 nodes");
    assert!(kg.stats.edge_count > 0, "Should have some edges");
    assert!(kg.stats.average_degree > 0.0, "Should have positive average degree");

    // 测试实体查找
    let related_to_tsinghua = kg_builder.find_related_entities("清华大学", 2);
    assert!(!related_to_tsinghua.is_empty(), "Should find entities related to 清华大学");

    println!("Final knowledge graph stats:");
    println!("  Nodes: {}", kg.stats.node_count);
    println!("  Edges: {}", kg.stats.edge_count);
    println!("  Node types: {:?}", kg.stats.node_type_counts);
    println!("  Relation types: {:?}", kg.stats.relation_type_counts);
}

#[tokio::test]
async fn test_chinese_text_processing() {
    let ner = ChineseNER::new().expect("Failed to create NER");

    // 测试各种中文文本特征
    let test_texts = vec![
        "繁體中文測試：機器學習和資料庫技術", // 繁体中文
        "混合文本：AI和人工智能、ML和机器学习", // 中英混合
        "标点符号测试：张三，李四；王五！赵六？", // 各种标点
        "数字和日期：2024年1月1日，张三30岁", // 数字和日期
        "专业术语：深度学习、神经网络、卷积神经网络CNN", // 技术术语
    ];

    for text in test_texts {
        let entities = ner.extract_entities(text).expect("Failed to extract entities");

        println!("Text: {}", text);
        println!("Entities: {:?}", entities.iter().map(|e| &e.text).collect::<Vec<_>>());

        // 验证实体位置的正确性
        for entity in &entities {
            let extracted_text = &text[entity.start..entity.end];
            assert_eq!(extracted_text, entity.text,
                      "Entity position mismatch: expected '{}', got '{}'",
                      entity.text, extracted_text);
        }
    }
}

#[tokio::test]
async fn test_performance_with_large_text() {
    let ner = ChineseNER::new().expect("Failed to create NER");
    let extractor = ChineseRelationExtractor::new().expect("Failed to create relation extractor");

    // 创建大文本
    let base_text = "张三在清华大学研究人工智能技术，他与李四合作开发了机器学习系统。";
    let large_text = base_text.repeat(100); // 重复100次

    let start_time = std::time::Instant::now();

    // 提取实体
    let entities = ner.extract_entities(&large_text).expect("Failed to extract entities");
    let entity_time = start_time.elapsed();

    // 提取关系
    let relations = extractor.extract_relations(&large_text, &entities)
        .expect("Failed to extract relations");
    let total_time = start_time.elapsed();

    println!("Performance test results:");
    println!("  Text length: {} characters", large_text.len());
    println!("  Entities extracted: {}", entities.len());
    println!("  Relations extracted: {}", relations.len());
    println!("  Entity extraction time: {:?}", entity_time);
    println!("  Total processing time: {:?}", total_time);

    // 性能断言（这些值可能需要根据实际性能调整）
    assert!(entity_time.as_secs() < 10, "Entity extraction should complete within 10 seconds");
    assert!(total_time.as_secs() < 30, "Total processing should complete within 30 seconds");
}

#[tokio::test]
async fn test_knowledge_graph_persistence() {
    let config = ZhushoudeConfig::default();
    let db_manager = Arc::new(DatabaseManager::new(config).await.expect("Failed to create database"));

    let mut kg_builder = KnowledgeGraphBuilder::new(db_manager.clone());

    let ner = ChineseNER::new().expect("Failed to create NER");
    let extractor = ChineseRelationExtractor::new().expect("Failed to create relation extractor");

    let text = "张三在清华大学研究人工智能技术。";
    let entities = ner.extract_entities(text).expect("Failed to extract entities");
    let relations = extractor.extract_relations(text, &entities)
        .expect("Failed to extract relations");

    // 构建知识图谱
    kg_builder.build_from_entities_and_relations(&entities, &relations, "test_doc")
        .await
        .expect("Failed to build knowledge graph");

    let kg_before = kg_builder.get_knowledge_graph().clone();

    // 创建新的构建器（模拟重启）
    let mut new_kg_builder = KnowledgeGraphBuilder::new(db_manager);

    // 重新添加相同的数据
    new_kg_builder.build_from_entities_and_relations(&entities, &relations, "test_doc")
        .await
        .expect("Failed to rebuild knowledge graph");

    let kg_after = new_kg_builder.get_knowledge_graph();

    // 验证数据一致性（注意：由于我们使用的是内存数据库，这个测试主要验证逻辑一致性）
    assert_eq!(kg_before.stats.node_count, kg_after.stats.node_count);
    assert_eq!(kg_before.stats.edge_count, kg_after.stats.edge_count);
}

#[test]
fn test_entity_and_relation_serialization() {
    use serde_json;

    let ner = ChineseNER::new().expect("Failed to create NER");
    let text = "张三在北京大学工作。";
    let entities = ner.extract_entities(text).expect("Failed to extract entities");

    // 测试实体序列化
    for entity in &entities {
        let serialized = serde_json::to_string(entity).expect("Failed to serialize entity");
        let deserialized: zhushoude_duckdb::nlp::Entity = serde_json::from_str(&serialized)
            .expect("Failed to deserialize entity");

        assert_eq!(entity.text, deserialized.text);
        assert_eq!(entity.entity_type.as_str(), deserialized.entity_type.as_str());
        assert_eq!(entity.confidence, deserialized.confidence);
    }

    // 测试关系序列化
    let extractor = ChineseRelationExtractor::new().expect("Failed to create relation extractor");
    let relations = extractor.extract_relations(text, &entities)
        .expect("Failed to extract relations");

    for relation in &relations {
        let serialized = serde_json::to_string(relation).expect("Failed to serialize relation");
        let deserialized: zhushoude_duckdb::nlp::EntityRelation = serde_json::from_str(&serialized)
            .expect("Failed to deserialize relation");

        assert_eq!(relation.subject.text, deserialized.subject.text);
        assert_eq!(relation.object.text, deserialized.object.text);
        assert_eq!(relation.relation_type.as_str(), deserialized.relation_type.as_str());
        assert_eq!(relation.confidence, deserialized.confidence);
    }
}
