//! 知识图谱UI测试
//! 
//! 测试知识图谱可视化界面功能

use inote::knowledge_graph_ui::KnowledgeGraphUI;
use inote::knowledge_graph_integration::{EntityInfo, RelationInfo};

#[test]
fn test_knowledge_graph_ui_creation() {
    let ui = KnowledgeGraphUI::new();
    
    assert!(ui.current_entities.is_empty());
    assert!(ui.current_relations.is_empty());
    assert!(ui.selected_entity.is_none());
    assert!(ui.entity_positions.is_empty());
    assert!(ui.show_entity_types);
    assert!(ui.show_confidence);
    assert_eq!(ui.min_confidence, 0.5);
    assert_eq!(ui.zoom_level, 1.0);
}

#[test]
fn test_knowledge_graph_ui_default() {
    let ui = KnowledgeGraphUI::default();
    
    assert!(ui.current_entities.is_empty());
    assert!(ui.current_relations.is_empty());
    assert!(ui.selected_entity.is_none());
    assert!(ui.entity_positions.is_empty());
}

#[test]
fn test_update_data() {
    let mut ui = KnowledgeGraphUI::new();
    
    let entities = vec![
        EntityInfo {
            text: "张三".to_string(),
            entity_type: "PERSON".to_string(),
            confidence: 0.9,
            start: 0,
            end: 2,
        },
        EntityInfo {
            text: "北京大学".to_string(),
            entity_type: "ORGANIZATION".to_string(),
            confidence: 0.85,
            start: 3,
            end: 6,
        },
    ];
    
    let relations = vec![
        RelationInfo {
            subject: entities[0].clone(),
            relation_type: "WORKS_AT".to_string(),
            object: entities[1].clone(),
            confidence: 0.8,
        },
    ];
    
    ui.update_data(entities.clone(), relations.clone());
    
    assert_eq!(ui.current_entities.len(), 2);
    assert_eq!(ui.current_relations.len(), 1);
    assert_eq!(ui.entity_positions.len(), 2);
    
    // 检查实体位置是否已计算
    assert!(ui.entity_positions.contains_key("张三"));
    assert!(ui.entity_positions.contains_key("北京大学"));
}

#[test]
fn test_entity_position_calculation() {
    let mut ui = KnowledgeGraphUI::new();
    
    let entities = vec![
        EntityInfo {
            text: "实体1".to_string(),
            entity_type: "PERSON".to_string(),
            confidence: 0.9,
            start: 0,
            end: 2,
        },
        EntityInfo {
            text: "实体2".to_string(),
            entity_type: "ORGANIZATION".to_string(),
            confidence: 0.8,
            start: 3,
            end: 5,
        },
        EntityInfo {
            text: "实体3".to_string(),
            entity_type: "LOCATION".to_string(),
            confidence: 0.7,
            start: 6,
            end: 8,
        },
    ];
    
    ui.update_data(entities, vec![]);
    
    // 检查所有实体都有位置
    assert_eq!(ui.entity_positions.len(), 3);
    
    // 检查位置是否合理（不为零点）
    for (entity_name, pos) in &ui.entity_positions {
        assert!(pos.x != 0.0 || pos.y != 0.0, "Entity {} should have non-zero position", entity_name);
    }
}

#[test]
fn test_confidence_filtering() {
    let mut ui = KnowledgeGraphUI::new();
    ui.min_confidence = 0.8; // 设置较高的置信度阈值
    
    let entities = vec![
        EntityInfo {
            text: "高置信度实体".to_string(),
            entity_type: "PERSON".to_string(),
            confidence: 0.9,
            start: 0,
            end: 5,
        },
        EntityInfo {
            text: "低置信度实体".to_string(),
            entity_type: "ORGANIZATION".to_string(),
            confidence: 0.6,
            start: 6,
            end: 11,
        },
    ];
    
    ui.update_data(entities, vec![]);
    
    // 虽然添加了两个实体，但只有高置信度的应该被显示
    // 这个测试主要验证数据结构，实际的过滤在渲染时进行
    assert_eq!(ui.current_entities.len(), 2);
    
    // 验证置信度设置
    assert_eq!(ui.min_confidence, 0.8);
}

#[test]
fn test_view_controls() {
    let mut ui = KnowledgeGraphUI::new();
    
    // 测试显示选项
    ui.show_entity_types = false;
    assert!(!ui.show_entity_types);
    
    ui.show_confidence = false;
    assert!(!ui.show_confidence);
    
    // 测试置信度阈值
    ui.min_confidence = 0.7;
    assert_eq!(ui.min_confidence, 0.7);
    
    // 测试缩放
    ui.zoom_level = 1.5;
    assert_eq!(ui.zoom_level, 1.5);
}

#[test]
fn test_reset_view() {
    let mut ui = KnowledgeGraphUI::new();
    
    // 修改一些设置
    ui.zoom_level = 2.0;
    ui.offset = eframe::egui::Vec2::new(100.0, 50.0);
    ui.selected_entity = Some("测试实体".to_string());
    
    // 重置视图
    ui.reset_view();
    
    assert_eq!(ui.zoom_level, 1.0);
    assert_eq!(ui.offset, eframe::egui::Vec2::ZERO);
    assert!(ui.selected_entity.is_none());
}

#[test]
fn test_entity_selection() {
    let mut ui = KnowledgeGraphUI::new();
    
    // 测试选择实体
    ui.selected_entity = Some("张三".to_string());
    assert_eq!(ui.selected_entity, Some("张三".to_string()));
    
    // 测试取消选择
    ui.selected_entity = None;
    assert!(ui.selected_entity.is_none());
}

#[test]
fn test_complex_graph_data() {
    let mut ui = KnowledgeGraphUI::new();
    
    // 创建复杂的图数据
    let entities = vec![
        EntityInfo {
            text: "张三".to_string(),
            entity_type: "PERSON".to_string(),
            confidence: 0.95,
            start: 0,
            end: 2,
        },
        EntityInfo {
            text: "李四".to_string(),
            entity_type: "PERSON".to_string(),
            confidence: 0.90,
            start: 3,
            end: 5,
        },
        EntityInfo {
            text: "清华大学".to_string(),
            entity_type: "ORGANIZATION".to_string(),
            confidence: 0.98,
            start: 6,
            end: 9,
        },
        EntityInfo {
            text: "北京".to_string(),
            entity_type: "LOCATION".to_string(),
            confidence: 0.92,
            start: 10,
            end: 12,
        },
        EntityInfo {
            text: "人工智能".to_string(),
            entity_type: "TECHNOLOGY".to_string(),
            confidence: 0.88,
            start: 13,
            end: 17,
        },
    ];
    
    let relations = vec![
        RelationInfo {
            subject: entities[0].clone(),
            relation_type: "WORKS_AT".to_string(),
            object: entities[2].clone(),
            confidence: 0.85,
        },
        RelationInfo {
            subject: entities[1].clone(),
            relation_type: "STUDIES".to_string(),
            object: entities[4].clone(),
            confidence: 0.80,
        },
        RelationInfo {
            subject: entities[2].clone(),
            relation_type: "LOCATED_IN".to_string(),
            object: entities[3].clone(),
            confidence: 0.95,
        },
    ];
    
    ui.update_data(entities, relations);
    
    assert_eq!(ui.current_entities.len(), 5);
    assert_eq!(ui.current_relations.len(), 3);
    assert_eq!(ui.entity_positions.len(), 5);
    
    // 验证所有实体都有位置
    for entity in &ui.current_entities {
        assert!(ui.entity_positions.contains_key(&entity.text));
    }
}

#[test]
fn test_empty_graph_handling() {
    let mut ui = KnowledgeGraphUI::new();
    
    // 测试空图的处理
    ui.update_data(vec![], vec![]);
    
    assert!(ui.current_entities.is_empty());
    assert!(ui.current_relations.is_empty());
    assert!(ui.entity_positions.is_empty());
}

#[test]
fn test_single_entity_graph() {
    let mut ui = KnowledgeGraphUI::new();
    
    let entities = vec![
        EntityInfo {
            text: "孤立实体".to_string(),
            entity_type: "PERSON".to_string(),
            confidence: 0.9,
            start: 0,
            end: 4,
        },
    ];
    
    ui.update_data(entities, vec![]);
    
    assert_eq!(ui.current_entities.len(), 1);
    assert!(ui.current_relations.is_empty());
    assert_eq!(ui.entity_positions.len(), 1);
    
    // 单个实体应该有位置
    assert!(ui.entity_positions.contains_key("孤立实体"));
}

#[test]
fn test_relations_without_entities() {
    let mut ui = KnowledgeGraphUI::new();
    
    // 创建没有对应实体的关系（这种情况在实际中不应该发生，但测试边界情况）
    let entity1 = EntityInfo {
        text: "实体1".to_string(),
        entity_type: "PERSON".to_string(),
        confidence: 0.9,
        start: 0,
        end: 3,
    };
    
    let entity2 = EntityInfo {
        text: "实体2".to_string(),
        entity_type: "ORGANIZATION".to_string(),
        confidence: 0.8,
        start: 4,
        end: 7,
    };
    
    let relations = vec![
        RelationInfo {
            subject: entity1,
            relation_type: "WORKS_AT".to_string(),
            object: entity2,
            confidence: 0.7,
        },
    ];
    
    // 只添加关系，不添加实体
    ui.update_data(vec![], relations);
    
    assert!(ui.current_entities.is_empty());
    assert_eq!(ui.current_relations.len(), 1);
    assert!(ui.entity_positions.is_empty());
}

#[test]
fn test_ui_state_persistence() {
    let mut ui = KnowledgeGraphUI::new();
    
    // 设置一些状态
    ui.show_entity_types = false;
    ui.show_confidence = false;
    ui.min_confidence = 0.8;
    ui.zoom_level = 1.5;
    ui.selected_entity = Some("测试实体".to_string());
    
    // 更新数据不应该影响UI状态
    let entities = vec![
        EntityInfo {
            text: "新实体".to_string(),
            entity_type: "PERSON".to_string(),
            confidence: 0.9,
            start: 0,
            end: 3,
        },
    ];
    
    ui.update_data(entities, vec![]);
    
    // UI状态应该保持不变
    assert!(!ui.show_entity_types);
    assert!(!ui.show_confidence);
    assert_eq!(ui.min_confidence, 0.8);
    assert_eq!(ui.zoom_level, 1.5);
    assert_eq!(ui.selected_entity, Some("测试实体".to_string()));
}

#[test]
fn test_zoom_and_offset_bounds() {
    let mut ui = KnowledgeGraphUI::new();
    
    // 测试极端缩放值
    ui.zoom_level = 0.1; // 很小的缩放
    assert_eq!(ui.zoom_level, 0.1);
    
    ui.zoom_level = 10.0; // 很大的缩放
    assert_eq!(ui.zoom_level, 10.0);
    
    // 测试极端偏移值
    ui.offset = eframe::egui::Vec2::new(-1000.0, -1000.0);
    assert_eq!(ui.offset.x, -1000.0);
    assert_eq!(ui.offset.y, -1000.0);
    
    ui.offset = eframe::egui::Vec2::new(1000.0, 1000.0);
    assert_eq!(ui.offset.x, 1000.0);
    assert_eq!(ui.offset.y, 1000.0);
}
