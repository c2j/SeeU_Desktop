//! 知识图谱构建和管理模块
//! 
//! 基于实体识别和关系抽取结果构建知识图谱

use crate::nlp::ner::{Entity, EntityType};
use crate::nlp::relation_extraction::{EntityRelation, RelationType};
use crate::graph::GraphStorage;
use crate::{GraphNode, GraphEdge, NodeType, EdgeType};
use crate::database::DatabaseManager;
use crate::Result;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use uuid::Uuid;

/// 知识图谱节点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeNode {
    /// 节点ID
    pub id: String,
    /// 实体文本
    pub text: String,
    /// 实体类型
    pub entity_type: EntityType,
    /// 置信度
    pub confidence: f64,
    /// 出现频次
    pub frequency: u32,
    /// 相关文档ID列表
    pub document_ids: Vec<String>,
    /// 额外属性
    pub properties: HashMap<String, String>,
}

/// 知识图谱边
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeEdge {
    /// 边ID
    pub id: String,
    /// 源节点ID
    pub source_id: String,
    /// 目标节点ID
    pub target_id: String,
    /// 关系类型
    pub relation_type: RelationType,
    /// 置信度
    pub confidence: f64,
    /// 权重（基于共现频次等）
    pub weight: f64,
    /// 相关文档ID列表
    pub document_ids: Vec<String>,
    /// 额外属性
    pub properties: HashMap<String, String>,
}

/// 知识图谱
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeGraph {
    /// 节点映射 (实体文本 -> 节点)
    pub nodes: HashMap<String, KnowledgeNode>,
    /// 边列表
    pub edges: Vec<KnowledgeEdge>,
    /// 统计信息
    pub stats: KnowledgeGraphStats,
}

/// 知识图谱统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeGraphStats {
    /// 节点总数
    pub node_count: usize,
    /// 边总数
    pub edge_count: usize,
    /// 各类型节点数量
    pub node_type_counts: HashMap<String, usize>,
    /// 各类型关系数量
    pub relation_type_counts: HashMap<String, usize>,
    /// 平均节点度数
    pub average_degree: f64,
}

/// 知识图谱构建器
pub struct KnowledgeGraphBuilder {
    /// 图存储
    graph_storage: Arc<GraphStorage>,
    /// 当前知识图谱
    knowledge_graph: KnowledgeGraph,
}

impl KnowledgeGraphBuilder {
    /// 创建新的知识图谱构建器
    pub fn new(db_manager: Arc<DatabaseManager>) -> Self {
        Self {
            graph_storage: Arc::new(GraphStorage::new(db_manager)),
            knowledge_graph: KnowledgeGraph {
                nodes: HashMap::new(),
                edges: Vec::new(),
                stats: KnowledgeGraphStats {
                    node_count: 0,
                    edge_count: 0,
                    node_type_counts: HashMap::new(),
                    relation_type_counts: HashMap::new(),
                    average_degree: 0.0,
                },
            },
        }
    }
    
    /// 从实体和关系构建知识图谱
    pub async fn build_from_entities_and_relations(
        &mut self,
        entities: &[Entity],
        relations: &[EntityRelation],
        document_id: &str,
    ) -> Result<()> {
        // 添加实体节点
        for entity in entities {
            self.add_or_update_entity_node(entity, document_id).await?;
        }
        
        // 添加关系边
        for relation in relations {
            self.add_or_update_relation_edge(relation, document_id).await?;
        }
        
        // 更新统计信息
        self.update_statistics();
        
        Ok(())
    }
    
    /// 添加或更新实体节点
    async fn add_or_update_entity_node(&mut self, entity: &Entity, document_id: &str) -> Result<()> {
        let node_key = entity.text.clone();
        
        if let Some(existing_node) = self.knowledge_graph.nodes.get_mut(&node_key) {
            // 更新现有节点
            existing_node.frequency += 1;
            existing_node.confidence = (existing_node.confidence + entity.confidence) / 2.0;
            if !existing_node.document_ids.contains(&document_id.to_string()) {
                existing_node.document_ids.push(document_id.to_string());
            }
        } else {
            // 创建新节点
            let node_id = Uuid::new_v4().to_string();
            let knowledge_node = KnowledgeNode {
                id: node_id.clone(),
                text: entity.text.clone(),
                entity_type: entity.entity_type.clone(),
                confidence: entity.confidence,
                frequency: 1,
                document_ids: vec![document_id.to_string()],
                properties: entity.properties.clone(),
            };
            
            // 转换为图节点并存储
            let graph_node = self.convert_to_graph_node(&knowledge_node);
            self.graph_storage.add_node(&graph_node).await?;
            
            self.knowledge_graph.nodes.insert(node_key, knowledge_node);
        }
        
        Ok(())
    }
    
    /// 添加或更新关系边
    async fn add_or_update_relation_edge(&mut self, relation: &EntityRelation, document_id: &str) -> Result<()> {
        let source_key = relation.subject.text.clone();
        let target_key = relation.object.text.clone();
        
        // 确保源节点和目标节点存在
        if !self.knowledge_graph.nodes.contains_key(&source_key) ||
           !self.knowledge_graph.nodes.contains_key(&target_key) {
            return Ok(()); // 跳过无效的关系
        }
        
        let source_id = self.knowledge_graph.nodes[&source_key].id.clone();
        let target_id = self.knowledge_graph.nodes[&target_key].id.clone();
        
        // 查找现有边
        let existing_edge_index = self.knowledge_graph.edges.iter().position(|edge| {
            edge.source_id == source_id &&
            edge.target_id == target_id &&
            edge.relation_type == relation.relation_type
        });
        
        if let Some(index) = existing_edge_index {
            // 更新现有边
            let edge = &mut self.knowledge_graph.edges[index];
            edge.confidence = (edge.confidence + relation.confidence) / 2.0;
            edge.weight += 1.0; // 增加权重
            if !edge.document_ids.contains(&document_id.to_string()) {
                edge.document_ids.push(document_id.to_string());
            }
        } else {
            // 创建新边
            let edge_id = Uuid::new_v4().to_string();
            let knowledge_edge = KnowledgeEdge {
                id: edge_id.clone(),
                source_id: source_id.clone(),
                target_id: target_id.clone(),
                relation_type: relation.relation_type.clone(),
                confidence: relation.confidence,
                weight: 1.0,
                document_ids: vec![document_id.to_string()],
                properties: relation.properties.clone(),
            };
            
            // 转换为图边并存储
            let graph_edge = self.convert_to_graph_edge(&knowledge_edge);
            self.graph_storage.add_edge(&graph_edge).await?;
            
            self.knowledge_graph.edges.push(knowledge_edge);
        }
        
        Ok(())
    }
    
    /// 将知识节点转换为图节点
    fn convert_to_graph_node(&self, knowledge_node: &KnowledgeNode) -> GraphNode {
        let mut properties = std::collections::HashMap::new();

        // 转换原有属性
        for (key, value) in &knowledge_node.properties {
            properties.insert(key.clone(), serde_json::Value::String(value.clone()));
        }

        // 添加额外属性
        properties.insert("entity_type".to_string(), serde_json::Value::String(knowledge_node.entity_type.as_str().to_string()));
        properties.insert("confidence".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(knowledge_node.confidence).unwrap_or_else(|| serde_json::Number::from(0))));
        properties.insert("frequency".to_string(), serde_json::Value::Number(serde_json::Number::from(knowledge_node.frequency)));
        properties.insert("document_ids".to_string(), serde_json::Value::String(knowledge_node.document_ids.join(",")));

        GraphNode {
            id: knowledge_node.id.clone(),
            node_type: self.entity_type_to_node_type(&knowledge_node.entity_type),
            label: knowledge_node.text.clone(),
            properties,
        }
    }
    
    /// 将知识边转换为图边
    fn convert_to_graph_edge(&self, knowledge_edge: &KnowledgeEdge) -> GraphEdge {
        let mut properties = std::collections::HashMap::new();

        // 转换原有属性
        for (key, value) in &knowledge_edge.properties {
            properties.insert(key.clone(), serde_json::Value::String(value.clone()));
        }

        // 添加额外属性
        properties.insert("relation_type".to_string(), serde_json::Value::String(knowledge_edge.relation_type.as_str().to_string()));
        properties.insert("confidence".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(knowledge_edge.confidence).unwrap_or_else(|| serde_json::Number::from(0))));
        properties.insert("document_ids".to_string(), serde_json::Value::String(knowledge_edge.document_ids.join(",")));

        GraphEdge {
            id: knowledge_edge.id.clone(),
            source_id: knowledge_edge.source_id.clone(),
            target_id: knowledge_edge.target_id.clone(),
            edge_type: self.relation_type_to_edge_type(&knowledge_edge.relation_type),
            weight: knowledge_edge.weight,
            properties,
        }
    }
    
    /// 实体类型转换为图节点类型
    fn entity_type_to_node_type(&self, entity_type: &EntityType) -> NodeType {
        match entity_type {
            EntityType::Person => NodeType::Entity,
            EntityType::Location => NodeType::Entity,
            EntityType::Organization => NodeType::Entity,
            EntityType::Time => NodeType::Attribute,
            EntityType::Concept => NodeType::Concept,
            EntityType::Technology => NodeType::Concept,
            EntityType::Product => NodeType::Entity,
            EntityType::Other => NodeType::Other,
        }
    }
    
    /// 关系类型转换为图边类型
    fn relation_type_to_edge_type(&self, relation_type: &RelationType) -> EdgeType {
        match relation_type {
            RelationType::Contains => EdgeType::Contains,
            RelationType::BelongsTo => EdgeType::BelongsTo,
            RelationType::RelatedTo => EdgeType::RelatedTo,
            RelationType::DependsOn => EdgeType::DependsOn,
            RelationType::LocatedIn => EdgeType::LocatedIn,
            RelationType::WorksAt => EdgeType::WorksAt,
            RelationType::Creates => EdgeType::Creates,
            RelationType::Uses => EdgeType::Uses,
            RelationType::Studies => EdgeType::Studies,
            RelationType::Researches => EdgeType::Researches,
            RelationType::OccursAt => EdgeType::OccursAt,
            RelationType::Other => EdgeType::Other,
        }
    }
    
    /// 更新统计信息
    fn update_statistics(&mut self) {
        let mut stats = KnowledgeGraphStats {
            node_count: self.knowledge_graph.nodes.len(),
            edge_count: self.knowledge_graph.edges.len(),
            node_type_counts: HashMap::new(),
            relation_type_counts: HashMap::new(),
            average_degree: 0.0,
        };
        
        // 统计节点类型
        for node in self.knowledge_graph.nodes.values() {
            let type_str = node.entity_type.as_str().to_string();
            *stats.node_type_counts.entry(type_str).or_insert(0) += 1;
        }
        
        // 统计关系类型
        for edge in &self.knowledge_graph.edges {
            let type_str = edge.relation_type.as_str().to_string();
            *stats.relation_type_counts.entry(type_str).or_insert(0) += 1;
        }
        
        // 计算平均度数
        if stats.node_count > 0 {
            stats.average_degree = (stats.edge_count * 2) as f64 / stats.node_count as f64;
        }
        
        self.knowledge_graph.stats = stats;
    }
    
    /// 获取知识图谱
    pub fn get_knowledge_graph(&self) -> &KnowledgeGraph {
        &self.knowledge_graph
    }
    
    /// 查找相关实体
    pub fn find_related_entities(&self, entity_text: &str, max_depth: usize) -> Vec<String> {
        let mut related = HashSet::new();
        let mut current_level = HashSet::new();
        current_level.insert(entity_text.to_string());
        
        for _ in 0..max_depth {
            let mut next_level = HashSet::new();
            
            for entity in &current_level {
                if let Some(node) = self.knowledge_graph.nodes.get(entity) {
                    // 查找所有相关的边
                    for edge in &self.knowledge_graph.edges {
                        if edge.source_id == node.id {
                            // 找到目标节点
                            for (text, target_node) in &self.knowledge_graph.nodes {
                                if target_node.id == edge.target_id && !related.contains(text) {
                                    next_level.insert(text.clone());
                                }
                            }
                        } else if edge.target_id == node.id {
                            // 找到源节点
                            for (text, source_node) in &self.knowledge_graph.nodes {
                                if source_node.id == edge.source_id && !related.contains(text) {
                                    next_level.insert(text.clone());
                                }
                            }
                        }
                    }
                }
            }
            
            related.extend(current_level.clone());
            current_level = next_level;
            
            if current_level.is_empty() {
                break;
            }
        }
        
        related.remove(entity_text); // 移除自身
        related.into_iter().collect()
    }
    
    /// 获取实体的直接关系
    pub fn get_entity_relations(&self, entity_text: &str) -> Vec<&KnowledgeEdge> {
        if let Some(node) = self.knowledge_graph.nodes.get(entity_text) {
            self.knowledge_graph.edges.iter()
                .filter(|edge| edge.source_id == node.id || edge.target_id == node.id)
                .collect()
        } else {
            Vec::new()
        }
    }
}
