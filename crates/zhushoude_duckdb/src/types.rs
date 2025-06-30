//! 公共类型定义模块
//! 
//! 定义zhushoude_duckdb中使用的所有公共类型

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// 文档类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    /// 文档ID
    pub id: String,
    /// 文档标题
    pub title: String,
    /// 文档内容
    pub content: String,
    /// 文档类型
    pub doc_type: DocumentType,
    /// 元数据
    pub metadata: serde_json::Value,
}

/// 笔记类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Note {
    /// 笔记ID
    pub id: String,
    /// 笔记标题
    pub title: String,
    /// 笔记内容
    pub content: String,
    /// 元数据
    pub metadata: serde_json::Value,
}

/// 代码文档类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeDocument {
    /// 代码文档ID
    pub id: String,
    /// 文件名
    pub filename: String,
    /// 代码内容
    pub content: String,
    /// 编程语言
    pub language: CodeLanguage,
    /// 元数据
    pub metadata: serde_json::Value,
}

/// 文档类型枚举
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DocumentType {
    /// 笔记
    Note,
    /// 代码
    Code(CodeLanguage),
    /// Markdown文档
    Markdown,
    /// 纯文本
    Text,
}

/// 编程语言枚举
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CodeLanguage {
    /// Java
    Java,
    /// SQL
    SQL,
    /// Rust
    Rust,
    /// Python
    Python,
    /// JavaScript
    JavaScript,
    /// TypeScript
    TypeScript,
    /// C++
    Cpp,
    /// C
    C,
    /// Go
    Go,
    /// 其他语言
    Other(String),
}

/// 搜索结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    /// 文档ID
    pub document_id: String,
    /// 文档标题
    pub title: String,
    /// 文档内容
    pub content: String,
    /// 文档类型
    pub doc_type: String,
    /// 相似度分数 (0.0-1.0)
    pub similarity_score: f64,
    /// 元数据
    pub metadata: Option<serde_json::Value>,
}

/// 混合搜索结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HybridResult {
    /// 文档ID
    pub document_id: String,
    /// 标题
    pub title: String,
    /// 内容
    pub content: String,
    /// 原始分数
    pub score: f64,
    /// 最终分数
    pub final_score: f64,
    /// 结果类型
    pub result_type: ResultType,
    /// 元数据
    pub metadata: serde_json::Value,
}

/// 结果类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResultType {
    /// 语义搜索结果
    Semantic,
    /// 图搜索结果
    Graph,
    /// 混合结果
    Hybrid,
}

/// 混合搜索查询
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HybridQuery {
    /// 查询文本
    pub text: String,
    /// 查询类型
    pub query_type: QueryType,
    /// 结果数量限制
    pub limit: usize,
    /// 启用语义搜索
    pub enable_semantic: bool,
    /// 启用图搜索
    pub enable_graph: bool,
    /// 权重配置
    pub weights: SearchWeights,
}

/// 查询类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum QueryType {
    /// 笔记查询
    Note,
    /// 代码查询
    Code,
    /// 通用查询
    General,
}

/// 搜索权重配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchWeights {
    /// 语义搜索权重
    pub semantic: f32,
    /// 图搜索权重
    pub graph: f32,
}

/// 图节点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    /// 节点ID
    pub id: String,
    /// 节点标签
    pub label: String,
    /// 节点类型
    pub node_type: NodeType,
    /// 节点属性
    pub properties: HashMap<String, serde_json::Value>,
}

/// 图边
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdge {
    /// 边ID
    pub id: String,
    /// 源节点ID
    pub source_id: String,
    /// 目标节点ID
    pub target_id: String,
    /// 边类型
    pub edge_type: EdgeType,
    /// 边权重
    pub weight: f64,
    /// 边属性
    pub properties: HashMap<String, serde_json::Value>,
}

/// 节点类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeType {
    /// 类
    Class,
    /// 方法
    Method,
    /// 变量
    Variable,
    /// 包/模块
    Package,
    /// 接口
    Interface,
    /// 实体（人名、地名、机构等）
    Entity,
    /// 概念（技术术语、抽象概念等）
    Concept,
    /// 属性（时间、数值等）
    Attribute,
    /// 其他
    Other,
}

/// 边类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EdgeType {
    /// 依赖关系
    DependsOn,
    /// 继承关系
    Inherits,
    /// 实现关系
    Implements,
    /// 调用关系
    Calls,
    /// 包含关系
    Contains,
    /// 属于关系
    BelongsTo,
    /// 相关关系
    RelatedTo,
    /// 位于关系
    LocatedIn,
    /// 工作于关系
    WorksAt,
    /// 创建关系
    Creates,
    /// 使用关系
    Uses,
    /// 学习关系
    Studies,
    /// 研究关系
    Researches,
    /// 时间关系
    OccursAt,
    /// 其他关系
    Other,
}

/// 性能统计
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PerformanceStats {
    /// 内存使用情况
    pub memory_usage: MemoryUsage,
    /// 缓存统计
    pub cache_stats: CacheStats,
    /// 查询统计
    pub query_stats: QueryStats,
}

/// 内存使用情况
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MemoryUsage {
    /// 总内存使用 (MB)
    pub total_mb: f64,
    /// 模型内存使用 (MB)
    pub model_mb: f64,
    /// 缓存内存使用 (MB)
    pub cache_mb: f64,
    /// 数据库内存使用 (MB)
    pub database_mb: f64,
}

/// 缓存统计
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CacheStats {
    /// 缓存命中次数
    pub hits: u64,
    /// 缓存未命中次数
    pub misses: u64,
    /// 缓存命中率
    pub hit_rate: f64,
    /// 缓存大小
    pub size: usize,
}

/// 查询统计
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct QueryStats {
    /// 总查询次数
    pub total_queries: u64,
    /// 平均查询时间 (ms)
    pub avg_query_time_ms: f64,
    /// 语义搜索次数
    pub semantic_queries: u64,
    /// 图搜索次数
    pub graph_queries: u64,
    /// 混合搜索次数
    pub hybrid_queries: u64,
}

// 实现Display trait
impl std::fmt::Display for DocumentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DocumentType::Note => write!(f, "note"),
            DocumentType::Code(lang) => write!(f, "code_{}", lang),
            DocumentType::Markdown => write!(f, "markdown"),
            DocumentType::Text => write!(f, "text"),
        }
    }
}

impl std::fmt::Display for CodeLanguage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CodeLanguage::Java => write!(f, "java"),
            CodeLanguage::SQL => write!(f, "sql"),
            CodeLanguage::Rust => write!(f, "rust"),
            CodeLanguage::Python => write!(f, "python"),
            CodeLanguage::JavaScript => write!(f, "javascript"),
            CodeLanguage::TypeScript => write!(f, "typescript"),
            CodeLanguage::Cpp => write!(f, "cpp"),
            CodeLanguage::C => write!(f, "c"),
            CodeLanguage::Go => write!(f, "go"),
            CodeLanguage::Other(name) => write!(f, "{}", name),
        }
    }
}

impl std::fmt::Display for NodeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NodeType::Class => write!(f, "class"),
            NodeType::Method => write!(f, "method"),
            NodeType::Variable => write!(f, "variable"),
            NodeType::Package => write!(f, "package"),
            NodeType::Interface => write!(f, "interface"),
            NodeType::Entity => write!(f, "entity"),
            NodeType::Concept => write!(f, "concept"),
            NodeType::Attribute => write!(f, "attribute"),
            NodeType::Other => write!(f, "other"),
        }
    }
}

impl std::fmt::Display for EdgeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EdgeType::DependsOn => write!(f, "depends_on"),
            EdgeType::Inherits => write!(f, "inherits"),
            EdgeType::Implements => write!(f, "implements"),
            EdgeType::Calls => write!(f, "calls"),
            EdgeType::Contains => write!(f, "contains"),
            EdgeType::BelongsTo => write!(f, "belongs_to"),
            EdgeType::RelatedTo => write!(f, "related_to"),
            EdgeType::LocatedIn => write!(f, "located_in"),
            EdgeType::WorksAt => write!(f, "works_at"),
            EdgeType::Creates => write!(f, "creates"),
            EdgeType::Uses => write!(f, "uses"),
            EdgeType::Studies => write!(f, "studies"),
            EdgeType::Researches => write!(f, "researches"),
            EdgeType::OccursAt => write!(f, "occurs_at"),
            EdgeType::Other => write!(f, "other"),
        }
    }
}
