// 语义搜索设计方案 - 核心模块定义

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// 向量化服务配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingConfig {
    /// 嵌入模型提供商 (openai, cohere, local)
    pub provider: String,
    /// 模型名称
    pub model: String,
    /// API密钥
    pub api_key: Option<String>,
    /// 向量维度
    pub dimension: usize,
    /// 批处理大小
    pub batch_size: usize,
}

/// 语义搜索配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticSearchConfig {
    /// 是否启用语义搜索
    pub enabled: bool,
    /// HelixDB连接配置
    pub helix_config: HelixDBConfig,
    /// 向量化配置
    pub embedding_config: EmbeddingConfig,
    /// 搜索权重配置
    pub search_weights: SearchWeights,
}

/// HelixDB连接配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HelixDBConfig {
    /// 数据库路径
    pub database_path: String,
    /// 连接超时(秒)
    pub connection_timeout: u64,
    /// 查询超时(秒)
    pub query_timeout: u64,
}

/// 搜索权重配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchWeights {
    /// 关键词搜索权重
    pub keyword_weight: f32,
    /// 语义搜索权重
    pub semantic_weight: f32,
    /// 图关系权重
    pub graph_weight: f32,
}

/// 笔记向量表示
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoteEmbedding {
    pub note_id: String,
    pub title_embedding: Vec<f32>,
    pub content_embedding: Vec<f32>,
    pub combined_embedding: Vec<f32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 语义搜索结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticSearchResult {
    pub note_id: String,
    pub title: String,
    pub content_preview: String,
    pub similarity_score: f32,
    pub keyword_score: f32,
    pub graph_score: f32,
    pub combined_score: f32,
    pub related_notes: Vec<RelatedNote>,
    pub matched_concepts: Vec<String>,
}

/// 相关笔记
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelatedNote {
    pub note_id: String,
    pub title: String,
    pub relation_type: String,
    pub similarity_score: f32,
}

/// 概念提取结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConceptExtraction {
    pub concepts: Vec<ExtractedConcept>,
    pub entities: Vec<ExtractedEntity>,
    pub keywords: Vec<String>,
}

/// 提取的概念
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedConcept {
    pub name: String,
    pub concept_type: String,
    pub confidence: f32,
    pub context: String,
}

/// 提取的实体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedEntity {
    pub name: String,
    pub entity_type: String,
    pub confidence: f32,
    pub mentions: Vec<EntityMention>,
}

/// 实体提及
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityMention {
    pub text: String,
    pub start_pos: usize,
    pub end_pos: usize,
    pub context: String,
}

/// 向量化服务接口
pub trait EmbeddingService {
    /// 生成文本向量
    async fn embed_text(&self, text: &str) -> Result<Vec<f32>, EmbeddingError>;
    
    /// 批量生成向量
    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, EmbeddingError>;
    
    /// 获取向量维度
    fn dimension(&self) -> usize;
}

/// 语义搜索服务接口
pub trait SemanticSearchService {
    /// 索引笔记
    async fn index_note(&self, note: &Note) -> Result<(), SemanticSearchError>;
    
    /// 批量索引笔记
    async fn index_notes(&self, notes: &[Note]) -> Result<(), SemanticSearchError>;
    
    /// 语义搜索
    async fn semantic_search(
        &self, 
        query: &str, 
        limit: usize,
        filters: Option<SearchFilters>
    ) -> Result<Vec<SemanticSearchResult>, SemanticSearchError>;
    
    /// 混合搜索(关键词+语义+图关系)
    async fn hybrid_search(
        &self,
        query: &str,
        limit: usize,
        filters: Option<SearchFilters>
    ) -> Result<Vec<SemanticSearchResult>, SemanticSearchError>;
    
    /// 查找相似笔记
    async fn find_similar_notes(
        &self,
        note_id: &str,
        limit: usize
    ) -> Result<Vec<RelatedNote>, SemanticSearchError>;
    
    /// 概念提取
    async fn extract_concepts(&self, text: &str) -> Result<ConceptExtraction, SemanticSearchError>;
    
    /// 更新笔记向量
    async fn update_note_embedding(&self, note_id: &str) -> Result<(), SemanticSearchError>;
    
    /// 删除笔记索引
    async fn delete_note_index(&self, note_id: &str) -> Result<(), SemanticSearchError>;
}

/// 搜索过滤器
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchFilters {
    pub notebook_ids: Option<Vec<String>>,
    pub tag_ids: Option<Vec<String>>,
    pub date_range: Option<DateRange>,
    pub content_types: Option<Vec<String>>,
    pub min_similarity: Option<f32>,
}

/// 日期范围
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DateRange {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

/// 笔记数据结构(简化版)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Note {
    pub id: String,
    pub title: String,
    pub content: String,
    pub notebook_id: String,
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 向量化错误
#[derive(Debug, thiserror::Error)]
pub enum EmbeddingError {
    #[error("API错误: {0}")]
    ApiError(String),
    #[error("网络错误: {0}")]
    NetworkError(String),
    #[error("配置错误: {0}")]
    ConfigError(String),
    #[error("向量维度不匹配")]
    DimensionMismatch,
}

/// 语义搜索错误
#[derive(Debug, thiserror::Error)]
pub enum SemanticSearchError {
    #[error("数据库错误: {0}")]
    DatabaseError(String),
    #[error("向量化错误: {0}")]
    EmbeddingError(#[from] EmbeddingError),
    #[error("查询解析错误: {0}")]
    QueryParseError(String),
    #[error("索引错误: {0}")]
    IndexError(String),
}

/// 搜索统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchStats {
    pub total_results: usize,
    pub semantic_results: usize,
    pub keyword_results: usize,
    pub graph_results: usize,
    pub search_time_ms: u64,
    pub embedding_time_ms: u64,
    pub query_time_ms: u64,
}

impl Default for SearchWeights {
    fn default() -> Self {
        Self {
            keyword_weight: 0.3,
            semantic_weight: 0.5,
            graph_weight: 0.2,
        }
    }
}

impl Default for EmbeddingConfig {
    fn default() -> Self {
        Self {
            provider: "openai".to_string(),
            model: "text-embedding-3-small".to_string(),
            api_key: None,
            dimension: 1536,
            batch_size: 100,
        }
    }
}
