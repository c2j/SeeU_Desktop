//! 语义搜索相关的类型定义

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// 笔记数据结构
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

/// 混合搜索结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HybridSearchResult {
    pub note_id: String,
    pub title: String,
    pub content_preview: String,
    pub semantic_score: f32,
    pub keyword_score: f32,
    pub graph_score: f32,
    pub combined_score: f32,
    pub search_type: SearchType,
    pub matched_terms: Vec<String>,
    pub related_notes: Vec<RelatedNote>,
    pub matched_concepts: Vec<String>,
    pub highlight_snippets: Vec<HighlightSnippet>,
}

/// 相关笔记
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelatedNote {
    pub note_id: String,
    pub title: String,
    pub relation_type: String,
    pub similarity_score: f32,
}

/// 搜索类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SearchType {
    Semantic,
    Keyword,
    Hybrid,
}

/// 高亮片段
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HighlightSnippet {
    pub text: String,
    pub start_pos: usize,
    pub end_pos: usize,
    pub matched_term: String,
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

/// 搜索统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchStatistics {
    pub total_indexed_notes: usize,
    pub semantic_index_size: usize,
    pub keyword_index_size: usize,
    pub last_index_update: Option<DateTime<Utc>>,
    pub total_searches: usize,
    pub avg_search_time_ms: f64,
}

impl Default for SearchStatistics {
    fn default() -> Self {
        Self {
            total_indexed_notes: 0,
            semantic_index_size: 0,
            keyword_index_size: 0,
            last_index_update: None,
            total_searches: 0,
            avg_search_time_ms: 0.0,
        }
    }
}

/// 搜索建议
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchSuggestion {
    pub text: String,
    pub suggestion_type: SuggestionType,
    pub confidence: f32,
}

/// 建议类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SuggestionType {
    Completion,
    Correction,
    Related,
    Concept,
}

/// 服务状态
#[derive(Debug, Clone, PartialEq)]
pub enum ServiceStatus {
    Stopped,
    Starting,
    Running,
    Error,
}

/// HelixDB状态
#[derive(Debug, Clone, PartialEq)]
pub enum HelixDBStatus {
    Stopped,
    Starting,
    Running,
    Stopping,
    Error(String),
}

impl std::fmt::Display for HelixDBStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HelixDBStatus::Stopped => write!(f, "已停止"),
            HelixDBStatus::Starting => write!(f, "启动中"),
            HelixDBStatus::Running => write!(f, "运行中"),
            HelixDBStatus::Stopping => write!(f, "停止中"),
            HelixDBStatus::Error(e) => write!(f, "错误: {}", e),
        }
    }
}

impl std::fmt::Display for ServiceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ServiceStatus::Stopped => write!(f, "已停止"),
            ServiceStatus::Starting => write!(f, "启动中"),
            ServiceStatus::Running => write!(f, "运行中"),
            ServiceStatus::Error => write!(f, "错误"),
        }
    }
}

impl std::fmt::Display for SearchType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SearchType::Semantic => write!(f, "语义搜索"),
            SearchType::Keyword => write!(f, "关键词搜索"),
            SearchType::Hybrid => write!(f, "混合搜索"),
        }
    }
}

// 注意：由于Rust的孤儿规则，我们不能为外部类型实现From trait
// 相反，我们提供一个转换函数
impl Note {
    /// 从inote::note::Note转换
    pub fn from_inote_note(note: inote::note::Note, notebook_id: String) -> Self {
        Self {
            id: note.id,
            title: note.title,
            content: note.content,
            notebook_id, // 需要从外部传入，因为inote::Note没有这个字段
            tags: note.tag_ids, // inote::Note使用tag_ids字段
            created_at: note.created_at,
            updated_at: note.updated_at,
        }
    }
}
