//! API请求类型定义

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 搜索请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchRequest {
    /// 查询文本
    pub query: String,
    /// 搜索类型
    pub search_type: SearchType,
    /// 结果数量限制
    pub limit: usize,
    /// 搜索选项
    pub options: SearchOptions,
}

/// 搜索类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SearchType {
    /// 语义搜索
    Semantic,
    /// 图搜索
    Graph,
    /// 混合搜索
    Hybrid,
}

/// 搜索选项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchOptions {
    /// 是否启用语义搜索
    pub enable_semantic: bool,
    /// 是否启用图搜索
    pub enable_graph: bool,
    /// 搜索权重
    pub weights: SearchWeights,
    /// 过滤条件
    pub filters: HashMap<String, String>,
    /// 排序方式
    pub sort_by: SortBy,
}

/// 搜索权重
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchWeights {
    /// 语义搜索权重
    pub semantic: f32,
    /// 图搜索权重
    pub graph: f32,
}

/// 排序方式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SortBy {
    /// 按相关性排序
    Relevance,
    /// 按时间排序
    Time,
    /// 按分数排序
    Score,
}

/// 文档添加请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddDocumentRequest {
    /// 文档ID
    pub id: String,
    /// 文档标题
    pub title: String,
    /// 文档内容
    pub content: String,
    /// 文档类型
    pub doc_type: String,
    /// 元数据
    pub metadata: HashMap<String, serde_json::Value>,
}

/// 批量添加文档请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddDocumentsBatchRequest {
    /// 文档列表
    pub documents: Vec<AddDocumentRequest>,
}

/// 代码分析请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyzeCodeRequest {
    /// 代码内容
    pub code: String,
    /// 编程语言
    pub language: String,
    /// 分析选项
    pub options: AnalyzeOptions,
}

/// 分析选项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyzeOptions {
    /// 是否构建依赖图
    pub build_dependency_graph: bool,
    /// 是否提取函数
    pub extract_functions: bool,
    /// 是否提取类
    pub extract_classes: bool,
    /// 是否计算复杂度
    pub calculate_complexity: bool,
}

/// 图查询请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphQueryRequest {
    /// 查询类型
    pub query_type: GraphQueryType,
    /// 查询参数
    pub parameters: HashMap<String, serde_json::Value>,
}

/// 图查询类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GraphQueryType {
    /// 最短路径
    ShortestPath { from: String, to: String },
    /// 深度优先搜索
    DFS { start: String, max_depth: usize },
    /// 广度优先搜索
    BFS { start: String, max_depth: usize },
    /// PageRank
    PageRank { iterations: usize },
    /// 连通分量
    ConnectedComponents,
    /// 社区检测
    CommunityDetection,
}

/// 统计信息请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatsRequest {
    /// 统计类型
    pub stats_type: StatsType,
}

/// 统计类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StatsType {
    /// 系统统计
    System,
    /// 搜索统计
    Search,
    /// 图统计
    Graph,
    /// 缓存统计
    Cache,
}

impl Default for SearchOptions {
    fn default() -> Self {
        Self {
            enable_semantic: true,
            enable_graph: false,
            weights: SearchWeights {
                semantic: 0.7,
                graph: 0.3,
            },
            filters: HashMap::new(),
            sort_by: SortBy::Relevance,
        }
    }
}

impl Default for AnalyzeOptions {
    fn default() -> Self {
        Self {
            build_dependency_graph: true,
            extract_functions: true,
            extract_classes: true,
            calculate_complexity: true,
        }
    }
}
