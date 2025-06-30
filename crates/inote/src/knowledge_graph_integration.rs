//! 知识图谱集成模块
//! 
//! 将zhushoude_duckdb的知识图谱功能集成到笔记系统中

use crate::note::Note;
use zhushoude_duckdb::{ZhushoudeDB, ZhushoudeConfig, Document, DocumentType, Entity, EntityRelation, KnowledgeGraphStats};
use std::sync::Arc;
use tokio::sync::Mutex;
use log::{info, error, debug};

/// 知识图谱管理器
pub struct KnowledgeGraphManager {
    /// ZhushoudeDB实例
    db: Arc<ZhushoudeDB>,
    /// 是否启用
    enabled: bool,
}

impl KnowledgeGraphManager {
    /// 创建新的知识图谱管理器
    pub async fn new(enabled: bool) -> Result<Self, Box<dyn std::error::Error>> {
        if !enabled {
            // 创建一个占位符实例
            let config = ZhushoudeConfig::default();
            let db = Arc::new(ZhushoudeDB::new(config).await?);
            return Ok(Self { db, enabled: false });
        }
        
        info!("初始化知识图谱管理器...");
        
        // 创建配置
        let config = ZhushoudeConfig::default();
        
        // 初始化数据库
        let db = Arc::new(ZhushoudeDB::new(config).await?);
        
        info!("知识图谱管理器初始化完成");
        
        Ok(Self { db, enabled: true })
    }
    
    /// 处理笔记添加
    pub async fn process_note_added(&self, note: &Note) -> Result<Option<(Vec<Entity>, Vec<EntityRelation>)>, Box<dyn std::error::Error>> {
        if !self.enabled {
            return Ok(None);
        }
        
        debug!("开始处理笔记的实体提取: {}", note.title);
        
        // 转换为Document格式
        let document = self.note_to_document(note);
        
        // 添加笔记并提取实体关系
        match self.db.add_note_with_entities(&document).await {
            Ok((entities, relations)) => {
                info!("笔记 '{}' 实体提取完成: {} 个实体, {} 个关系", 
                      note.title, entities.len(), relations.len());
                Ok(Some((entities, relations)))
            }
            Err(e) => {
                error!("处理笔记 '{}' 时出错: {}", note.title, e);
                Err(e.into())
            }
        }
    }
    
    /// 处理笔记更新
    pub async fn process_note_updated(&self, note: &Note) -> Result<Option<(Vec<Entity>, Vec<EntityRelation>)>, Box<dyn std::error::Error>> {
        if !self.enabled {
            return Ok(None);
        }
        
        debug!("开始处理笔记更新的实体提取: {}", note.title);
        
        // 对于更新，我们重新处理整个笔记
        self.process_note_added(note).await
    }
    
    /// 语义搜索笔记
    pub async fn semantic_search(&self, query: &str, limit: usize) -> Result<Vec<SemanticSearchResult>, Box<dyn std::error::Error>> {
        if !self.enabled {
            return Ok(Vec::new());
        }
        
        debug!("执行语义搜索: {}", query);
        
        let results = self.db.search_notes_with_entities(query, limit).await?;
        
        let semantic_results = results.into_iter().map(|r| SemanticSearchResult {
            note_id: r.document_id,
            title: r.title,
            content: r.content,
            similarity_score: r.similarity_score,
            metadata: r.metadata,
        }).collect();
        
        Ok(semantic_results)
    }
    
    /// 获取实体的相关实体
    pub async fn get_related_entities(&self, entity_text: &str, max_depth: usize) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        if !self.enabled {
            return Ok(Vec::new());
        }
        
        Ok(self.db.get_related_entities(entity_text, max_depth).await?)
    }
    
    /// 获取实体的关系
    pub async fn get_entity_relations(&self, entity_text: &str) -> Result<Vec<EntityRelationInfo>, Box<dyn std::error::Error>> {
        if !self.enabled {
            return Ok(Vec::new());
        }
        
        let relations = self.db.get_entity_relations(entity_text).await?;
        
        let relation_infos = relations.into_iter().map(|r| EntityRelationInfo {
            source_entity: r.source_id,
            target_entity: r.target_id,
            relation_type: r.relation_type.as_str().to_string(),
            confidence: r.confidence,
            weight: r.weight,
        }).collect();
        
        Ok(relation_infos)
    }
    
    /// 获取知识图谱统计信息
    pub async fn get_stats(&self) -> Result<Option<KnowledgeGraphStats>, Box<dyn std::error::Error>> {
        if !self.enabled {
            return Ok(None);
        }
        
        Ok(Some(self.db.get_knowledge_graph_stats().await?))
    }
    
    /// 从文本提取实体和关系
    pub async fn extract_entities_and_relations(&self, text: &str) -> Result<Option<(Vec<Entity>, Vec<EntityRelation>)>, Box<dyn std::error::Error>> {
        if !self.enabled {
            return Ok(None);
        }
        
        Ok(Some(self.db.extract_entities_and_relations(text).await?))
    }
    
    /// 将Note转换为Document
    fn note_to_document(&self, note: &Note) -> Document {
        Document {
            id: note.id.clone(),
            title: note.title.clone(),
            content: note.content.clone(),
            doc_type: DocumentType::Note,
            metadata: serde_json::json!({
                "created_at": note.created_at,
                "updated_at": note.updated_at,
                "tag_ids": note.tag_ids,
                "attachments": note.attachments
            }),
        }
    }
    
    /// 检查是否启用
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
    
    /// 启用知识图谱功能
    pub fn enable(&mut self) {
        self.enabled = true;
        info!("知识图谱功能已启用");
    }
    
    /// 禁用知识图谱功能
    pub fn disable(&mut self) {
        self.enabled = false;
        info!("知识图谱功能已禁用");
    }
}

/// 语义搜索结果
#[derive(Debug, Clone)]
pub struct SemanticSearchResult {
    pub note_id: String,
    pub title: String,
    pub content: String,
    pub similarity_score: f64,
    pub metadata: Option<serde_json::Value>,
}

/// 实体关系信息
#[derive(Debug, Clone)]
pub struct EntityRelationInfo {
    pub source_entity: String,
    pub target_entity: String,
    pub relation_type: String,
    pub confidence: f64,
    pub weight: f64,
}

/// 实体信息
#[derive(Debug, Clone)]
pub struct EntityInfo {
    pub text: String,
    pub entity_type: String,
    pub confidence: f64,
    pub start: usize,
    pub end: usize,
}

impl From<&Entity> for EntityInfo {
    fn from(entity: &Entity) -> Self {
        Self {
            text: entity.text.clone(),
            entity_type: entity.entity_type.as_str().to_string(),
            confidence: entity.confidence,
            start: entity.start,
            end: entity.end,
        }
    }
}

/// 关系信息
#[derive(Debug, Clone)]
pub struct RelationInfo {
    pub subject: EntityInfo,
    pub relation_type: String,
    pub object: EntityInfo,
    pub confidence: f64,
}

impl From<&EntityRelation> for RelationInfo {
    fn from(relation: &EntityRelation) -> Self {
        Self {
            subject: EntityInfo::from(&relation.subject),
            relation_type: relation.relation_type.as_str().to_string(),
            object: EntityInfo::from(&relation.object),
            confidence: relation.confidence,
        }
    }
}
