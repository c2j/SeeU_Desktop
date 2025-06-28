//! API响应类型定义

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// API响应基础结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    /// 是否成功
    pub success: bool,
    /// 响应数据
    pub data: Option<T>,
    /// 错误信息
    pub error: Option<String>,
    /// 响应时间戳
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// 请求ID
    pub request_id: String,
}

/// 搜索响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResponse {
    /// 搜索结果
    pub results: Vec<SearchResultItem>,
    /// 总结果数
    pub total_count: usize,
    /// 搜索耗时（毫秒）
    pub duration_ms: u64,
    /// 搜索统计
    pub stats: SearchStats,
}

/// 搜索结果项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResultItem {
    /// 文档ID
    pub document_id: String,
    /// 标题
    pub title: String,
    /// 内容摘要
    pub content_snippet: String,
    /// 相关性分数
    pub score: f64,
    /// 结果类型
    pub result_type: String,
    /// 元数据
    pub metadata: HashMap<String, serde_json::Value>,
    /// 高亮信息
    pub highlights: Vec<Highlight>,
}

/// 高亮信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Highlight {
    /// 字段名
    pub field: String,
    /// 高亮片段
    pub fragments: Vec<String>,
}

/// 搜索统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchStats {
    /// 语义搜索命中数
    pub semantic_hits: usize,
    /// 图搜索命中数
    pub graph_hits: usize,
    /// 缓存命中率
    pub cache_hit_rate: f64,
    /// 查询解析时间
    pub parse_time_ms: u64,
    /// 搜索执行时间
    pub search_time_ms: u64,
    /// 结果合并时间
    pub merge_time_ms: u64,
}

/// 文档添加响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddDocumentResponse {
    /// 文档ID
    pub document_id: String,
    /// 是否成功
    pub success: bool,
    /// 处理时间
    pub processing_time_ms: u64,
    /// 向量化信息
    pub embedding_info: EmbeddingInfo,
}

/// 向量化信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingInfo {
    /// 向量维度
    pub dimension: usize,
    /// 向量化耗时
    pub embedding_time_ms: u64,
    /// 使用的模型
    pub model_name: String,
}

/// 批量添加响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddDocumentsBatchResponse {
    /// 成功添加的文档数
    pub success_count: usize,
    /// 失败的文档数
    pub failed_count: usize,
    /// 总处理时间
    pub total_time_ms: u64,
    /// 详细结果
    pub results: Vec<AddDocumentResponse>,
}

/// 代码分析响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyzeCodeResponse {
    /// 分析结果
    pub analysis: CodeAnalysis,
    /// 处理时间
    pub processing_time_ms: u64,
}

/// 代码分析结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeAnalysis {
    /// 编程语言
    pub language: String,
    /// 代码行数
    pub lines_of_code: usize,
    /// 复杂度
    pub complexity: usize,
    /// 提取的函数
    pub functions: Vec<FunctionInfo>,
    /// 提取的类
    pub classes: Vec<ClassInfo>,
    /// 依赖关系
    pub dependencies: Vec<Dependency>,
}

/// 函数信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionInfo {
    /// 函数名
    pub name: String,
    /// 起始行
    pub start_line: usize,
    /// 结束行
    pub end_line: usize,
    /// 参数
    pub parameters: Vec<String>,
    /// 复杂度
    pub complexity: usize,
}

/// 类信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassInfo {
    /// 类名
    pub name: String,
    /// 起始行
    pub start_line: usize,
    /// 结束行
    pub end_line: usize,
    /// 方法
    pub methods: Vec<String>,
    /// 属性
    pub properties: Vec<String>,
}

/// 依赖关系
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dependency {
    /// 源
    pub from: String,
    /// 目标
    pub to: String,
    /// 依赖类型
    pub dependency_type: String,
    /// 权重
    pub weight: f64,
}

/// 图查询响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphQueryResponse {
    /// 查询结果
    pub result: GraphQueryResult,
    /// 处理时间
    pub processing_time_ms: u64,
}

/// 图查询结果
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum GraphQueryResult {
    /// 路径结果
    Path { nodes: Vec<String> },
    /// 节点列表结果
    Nodes { nodes: Vec<GraphNodeInfo> },
    /// 排名结果
    Rankings { rankings: Vec<(String, f64)> },
    /// 分组结果
    Groups { groups: Vec<Vec<String>> },
}

/// 图节点信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNodeInfo {
    /// 节点ID
    pub id: String,
    /// 节点标签
    pub label: String,
    /// 节点类型
    pub node_type: String,
    /// 属性
    pub properties: HashMap<String, serde_json::Value>,
}

/// 统计信息响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatsResponse {
    /// 统计数据
    pub stats: SystemStats,
    /// 收集时间
    pub collected_at: chrono::DateTime<chrono::Utc>,
}

/// 系统统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemStats {
    /// 文档统计
    pub documents: DocumentStats,
    /// 向量统计
    pub vectors: VectorStats,
    /// 图统计
    pub graph: GraphStats,
    /// 缓存统计
    pub cache: CacheStats,
    /// 性能统计
    pub performance: PerformanceStats,
}

/// 文档统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentStats {
    /// 总文档数
    pub total_count: usize,
    /// 按类型分组的文档数
    pub by_type: HashMap<String, usize>,
    /// 总大小（字节）
    pub total_size_bytes: u64,
}

/// 向量统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorStats {
    /// 总向量数
    pub total_count: usize,
    /// 向量维度
    pub dimension: usize,
    /// 索引大小
    pub index_size_bytes: u64,
}

/// 图统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphStats {
    /// 节点数
    pub node_count: usize,
    /// 边数
    pub edge_count: usize,
    /// 连通分量数
    pub connected_components: usize,
}

/// 缓存统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    /// 命中次数
    pub hits: u64,
    /// 未命中次数
    pub misses: u64,
    /// 命中率
    pub hit_rate: f64,
    /// 缓存大小
    pub size: usize,
}

/// 性能统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceStats {
    /// 平均查询时间
    pub avg_query_time_ms: f64,
    /// 查询吞吐量（QPS）
    pub queries_per_second: f64,
    /// 内存使用量
    pub memory_usage_mb: f64,
    /// CPU使用率
    pub cpu_usage_percent: f64,
}

impl<T> ApiResponse<T> {
    /// 创建成功响应
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            timestamp: chrono::Utc::now(),
            request_id: uuid::Uuid::new_v4().to_string(),
        }
    }
    
    /// 创建错误响应
    pub fn error(error: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(error),
            timestamp: chrono::Utc::now(),
            request_id: uuid::Uuid::new_v4().to_string(),
        }
    }
}
