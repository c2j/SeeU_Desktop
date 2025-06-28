# zhushoude_duckdb API 文档

## 概述

zhushoude_duckdb 提供了完整的 Rust API 和 HTTP 风格的客户端 API，支持中文语义搜索、图计算和混合搜索功能。

## 核心 API

### ZhushoudeEngine

主要的引擎类，提供所有核心功能。

#### 初始化

```rust
use zhushoude_duckdb::*;

let config = ZhushoudeConfig {
    database_path: ":memory:".to_string(),
    embedding: EmbeddingConfig {
        model_name: "BAAI/bge-small-zh-v1.5".to_string(),
        vector_dimension: 512,
        batch_size: 8,
        max_cache_size: 1000,
        enable_chinese_optimization: true,
        normalize_vectors: true,
    },
    ..Default::default()
};

let engine = ZhushoudeEngine::new(config).await?;
```

#### 文档管理

##### 添加文档

```rust
let document = Document {
    id: "doc1".to_string(),
    title: "人工智能简介".to_string(),
    content: "人工智能是计算机科学的一个分支...".to_string(),
    doc_type: DocumentType::Note,
    metadata: serde_json::json!({"category": "AI"}),
};

engine.add_document(&document).await?;
```

##### 批量添加文档

```rust
let documents = vec![doc1, doc2, doc3];
engine.add_documents_batch(&documents).await?;
```

##### 删除文档

```rust
engine.delete_document("doc1").await?;
```

#### 搜索功能

##### 语义搜索

```rust
let results = engine.semantic_search("人工智能技术", 10).await?;

for result in results {
    println!("文档: {} (分数: {:.4})", result.title, result.final_score);
}
```

##### 图搜索

```rust
let results = engine.graph_search("相关代码", 10).await?;
```

##### 混合搜索

```rust
let query = HybridQuery {
    text: "深度学习".to_string(),
    query_type: QueryType::General,
    limit: 10,
    enable_semantic: true,
    enable_graph: true,
    weights: SearchWeights {
        semantic: 0.8,
        graph: 0.2,
    },
};

let results = engine.hybrid_search(&query).await?;
```

#### 图算法

##### 最短路径

```rust
let path = engine.shortest_path("node1", "node2").await?;
```

##### 深度优先搜索

```rust
let nodes = engine.dfs("start_node", 3).await?;
```

##### 广度优先搜索

```rust
let nodes = engine.bfs("start_node", 3).await?;
```

##### PageRank

```rust
let rankings = engine.pagerank(10).await?;
```

##### 连通分量

```rust
let components = engine.connected_components().await?;
```

##### 社区检测

```rust
let communities = engine.community_detection().await?;
```

#### 代码分析

```rust
let code = r#"
fn main() {
    println!("Hello, world!");
}
"#;

let analysis = engine.analyze_code(code, "rust").await?;
```

## 客户端 API

### ZhushoudeClient

高级客户端 API，提供结构化的请求和响应。

#### 初始化

```rust
use std::sync::Arc;

let engine = Arc::new(ZhushoudeEngine::new(config).await?);
let client = ZhushoudeClient::new(engine);
```

#### 文档操作

##### 添加文档

```rust
let request = api::AddDocumentRequest {
    id: "test_doc".to_string(),
    title: "测试文档".to_string(),
    content: "这是一个测试文档".to_string(),
    doc_type: "Note".to_string(),
    metadata: std::collections::HashMap::new(),
};

let response = client.add_document(request).await?;
```

##### 批量添加文档

```rust
let request = api::AddDocumentsBatchRequest {
    documents: vec![doc1, doc2, doc3],
};

let response = client.add_documents_batch(request).await?;
```

#### 搜索操作

```rust
let request = api::SearchRequest {
    query: "人工智能".to_string(),
    search_type: api::SearchType::Semantic,
    limit: 10,
    options: api::SearchOptions {
        enable_semantic: true,
        enable_graph: false,
        weights: api::SearchWeights {
            semantic: 0.8,
            graph: 0.2,
        },
        filters: std::collections::HashMap::new(),
        sort_by: api::SortBy::Relevance,
    },
};

let response = client.search(request).await?;
```

#### 代码分析

```rust
let request = api::AnalyzeCodeRequest {
    code: "fn main() {}".to_string(),
    language: "rust".to_string(),
    options: api::AnalyzeOptions {
        build_dependency_graph: true,
        extract_functions: true,
        extract_classes: true,
        calculate_complexity: true,
    },
};

let response = client.analyze_code(request).await?;
```

#### 图查询

```rust
let request = api::GraphQueryRequest {
    query_type: api::GraphQueryType::ShortestPath {
        from: "node1".to_string(),
        to: "node2".to_string(),
    },
    parameters: std::collections::HashMap::new(),
};

let response = client.graph_query(request).await?;
```

#### 统计信息

```rust
let request = api::StatsRequest {
    stats_type: api::StatsType::System,
};

let response = client.get_stats(request).await?;
```

## 数据类型

### 核心类型

#### Document

```rust
pub struct Document {
    pub id: String,
    pub title: String,
    pub content: String,
    pub doc_type: DocumentType,
    pub metadata: serde_json::Value,
}
```

#### DocumentType

```rust
pub enum DocumentType {
    Note,
    Code,
    Markdown,
    Text,
}
```

#### SearchResult

```rust
pub struct SearchResult {
    pub document_id: String,
    pub title: String,
    pub content: String,
    pub similarity_score: f64,
    pub metadata: Option<serde_json::Value>,
}
```

#### HybridResult

```rust
pub struct HybridResult {
    pub document_id: String,
    pub title: String,
    pub content: String,
    pub score: f64,
    pub final_score: f64,
    pub result_type: ResultType,
    pub metadata: std::collections::HashMap<String, serde_json::Value>,
}
```

### 图类型

#### GraphNode

```rust
pub struct GraphNode {
    pub id: String,
    pub label: String,
    pub node_type: NodeType,
    pub properties: std::collections::HashMap<String, serde_json::Value>,
}
```

#### GraphEdge

```rust
pub struct GraphEdge {
    pub id: String,
    pub source_id: String,
    pub target_id: String,
    pub edge_type: EdgeType,
    pub weight: f32,
    pub properties: std::collections::HashMap<String, serde_json::Value>,
}
```

### 配置类型

#### ZhushoudeConfig

```rust
pub struct ZhushoudeConfig {
    pub database_path: String,
    pub embedding: EmbeddingConfig,
    pub performance: PerformanceConfig,
    pub hybrid: HybridConfig,
    pub graph: GraphConfig,
}
```

#### EmbeddingConfig

```rust
pub struct EmbeddingConfig {
    pub model_name: String,
    pub vector_dimension: usize,
    pub batch_size: usize,
    pub max_cache_size: usize,
    pub enable_chinese_optimization: bool,
    pub normalize_vectors: bool,
}
```

## 错误处理

### Error 类型

```rust
pub enum Error {
    DatabaseError(String),
    EmbeddingError(String),
    SearchError(String),
    GraphError(String),
    ConfigError(String),
    IoError(String),
}
```

### 错误处理示例

```rust
match engine.semantic_search("query", 10).await {
    Ok(results) => {
        // 处理搜索结果
    }
    Err(Error::SearchError(msg)) => {
        eprintln!("搜索错误: {}", msg);
    }
    Err(e) => {
        eprintln!("其他错误: {}", e);
    }
}
```

## 性能优化

### 缓存配置

```rust
let config = EmbeddingConfig {
    max_cache_size: 10000,  // 增加缓存大小
    batch_size: 16,         // 增加批处理大小
    ..Default::default()
};
```

### 内存管理

```rust
let config = PerformanceConfig {
    memory_limit_mb: 1024,  // 设置内存限制
    thread_pool_size: Some(8),  // 设置线程数
    enable_monitoring: true,
    cache_strategy: CacheStrategy::LRU,
};
```

### 批量操作

```rust
// 使用批量操作提高性能
let documents = vec![/* 大量文档 */];
engine.add_documents_batch(&documents).await?;

let texts = vec![/* 大量文本 */];
let embeddings = engine.encode_batch(&texts).await?;
```

## 最佳实践

### 1. 配置优化

- 根据可用内存调整 `memory_limit_mb`
- 根据 CPU 核心数设置 `thread_pool_size`
- 根据使用模式调整 `max_cache_size`

### 2. 搜索优化

- 使用混合搜索获得最佳结果
- 根据场景调整语义和图搜索权重
- 合理设置搜索结果数量限制

### 3. 内存管理

- 定期清理不需要的文档
- 监控缓存命中率
- 使用批量操作减少内存分配

### 4. 错误处理

- 总是处理可能的错误
- 使用适当的重试机制
- 记录详细的错误信息

## 示例程序

查看 `examples/` 目录中的完整示例：

- `bge_demo.rs` - BGE 模型演示
- `api_demo.rs` - API 使用演示
- `benchmark.rs` - 性能基准测试

## 更多信息

- [README.md](../README.md) - 项目概述
- [CHANGELOG.md](../CHANGELOG.md) - 版本变更
- [examples/](../examples/) - 示例代码
- [tests/](../tests/) - 测试用例
