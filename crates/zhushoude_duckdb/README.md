# ZhushoudeDB - 智能语义搜索与图计算数据库

ZhushoudeDB 是一个基于 DuckDB 的高性能嵌入式数据库，专为中文语义搜索和代码图分析而设计。

## 🚀 特性

### 核心功能
- **嵌入式 DuckDB**: 高性能的分析型数据库引擎
- **中文语义搜索**: 基于 BGE-Small-ZH 模型的向量化搜索
- **代码图分析**: 支持多种编程语言的代码依赖关系分析
- **混合搜索**: 结合语义搜索和图搜索的智能融合算法
- **向量索引**: 支持线性、哈希、聚类、自适应等多种索引类型
- **智能索引**: 根据数据量自动选择最优索引策略

### 语言支持
- **中文文本处理**: 繁简转换、标点符号标准化、语言检测
- **多语言代码分析**: Java、Python、Rust、SQL 等
- **智能分词**: 针对中文优化的文本预处理

### 性能优化
- **内存缓存**: LRU 缓存机制，提升查询性能
- **批量处理**: 支持文档和向量的批量操作
- **异步处理**: 全异步 API 设计
- **资源控制**: 可配置的内存和 CPU 使用限制

## 📦 安装

将以下内容添加到您的 `Cargo.toml`:

```toml
[dependencies]
zhushoude_duckdb = "0.1.0"
```

## 🔧 快速开始

### 基本配置

```rust
use zhushoude_duckdb::*;

#[tokio::main]
async fn main() -> Result<()> {
    // 创建配置
    let config = ZhushoudeConfig {
        database_path: "data/zhushoude.db".to_string(),
        embedding: EmbeddingConfig {
            model_name: "bge-small-zh".to_string(),
            vector_dimension: 512,
            batch_size: 32,
            cache_size: 1000,
        },
        ..Default::default()
    };
    
    // 初始化数据库
    let db = ZhushoudeDB::new(config).await?;
    
    Ok(())
}
```

### 文档搜索

```rust
// 添加文档
let document = Document {
    id: "doc1".to_string(),
    title: "机器学习基础".to_string(),
    content: "机器学习是人工智能的一个重要分支...".to_string(),
    doc_type: DocumentType::Note,
    metadata: serde_json::json!({"author": "张三"}),
};

db.add_document(&document).await?;

// 语义搜索
let results = db.search("深度学习算法", 10).await?;
for result in results {
    println!("标题: {}, 相似度: {:.3}", result.title, result.similarity_score);
}
```

### 代码分析

```rust
// 分析代码
let java_code = r#"
public class Calculator {
    public int add(int a, int b) {
        return a + b;
    }
}
"#;

let graph = db.analyze_code(java_code, &CodeLanguage::Java).await?;

// 查找依赖关系
let dependencies = db.find_dependencies("Calculator").await?;
```

### 向量索引管理

```rust
// 创建索引管理器
let mut index_manager = VectorIndexManager::new(db_manager);
index_manager.initialize().await?;

// 创建不同类型的索引
let linear_index = index_manager.create_index(
    "document_embeddings",
    "embedding",
    IndexType::Linear,
    512
).await?;

let hash_index = index_manager.create_index(
    "code_embeddings",
    "embedding",
    IndexType::Hash { num_buckets: 100 },
    256
).await?;

let adaptive_index = index_manager.create_index(
    "mixed_embeddings",
    "embedding",
    IndexType::Adaptive,  // 自动选择最优类型
    384
).await?;

// 获取索引信息
let indexes = index_manager.list_indexes();
for index_name in indexes {
    if let Some(info) = index_manager.get_index_info(&index_name) {
        println!("索引: {}, 类型: {:?}, 维度: {}",
            index_name, info.index_type, info.dimension);
    }
}

// 使用索引进行搜索
let query_vector = vec![0.1, 0.2, 0.3, /* ... */];
let results = index_manager.search_with_index(
    &linear_index,
    &query_vector,
    10,     // 返回结果数量
    0.7     // 相似度阈值
).await?;
```

### 混合搜索

```rust
// 混合搜索查询
let query = HybridQuery {
    text: "数据库连接池".to_string(),
    query_type: QueryType::General,
    limit: 20,
    enable_semantic: true,
    enable_graph: true,
    weights: SearchWeights {
        semantic: 0.7,
        graph: 0.3,
    },
};

let results = db.hybrid_search(&query).await?;
```

### API 客户端使用

```rust
use zhushoude_duckdb::*;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    // 创建引擎和API客户端
    let config = ZhushoudeConfig::default();
    let engine = Arc::new(ZhushoudeEngine::new(config).await?);
    let client = ZhushoudeClient::new(engine);

    // 添加文档
    let add_request = api::AddDocumentRequest {
        id: "test_doc".to_string(),
        title: "测试文档".to_string(),
        content: "这是一个测试文档".to_string(),
        doc_type: "Note".to_string(),
        metadata: std::collections::HashMap::new(),
    };

    let response = client.add_document(add_request).await?;
    println!("添加结果: {}", response.success);

    // 搜索文档
    let search_request = api::SearchRequest {
        query: "测试".to_string(),
        search_type: api::SearchType::Semantic,
        limit: 10,
        options: api::SearchOptions::default(),
    };

    let search_response = client.search(search_request).await?;
    if let Some(data) = search_response.data {
        println!("找到 {} 个结果", data.total_count);
    }

    Ok(())
}
```

### 图算法使用

```rust
// 最短路径算法
let path = db.shortest_path("node1", "node2").await?;
println!("最短路径: {:?}", path);

// PageRank算法
let rankings = db.pagerank(10).await?;
for (node, score) in rankings.iter().take(5) {
    println!("节点: {}, 分数: {:.4}", node, score);
}

// 社区检测
let communities = db.community_detection().await?;
println!("发现 {} 个社区", communities.len());
```

## 🏗️ 架构设计

### 模块结构

```
zhushoude_duckdb/
├── src/
│   ├── config/          # 配置管理
│   ├── database/        # 数据库连接和模式
│   ├── embedding/       # 语义模型和向量化
│   ├── vector/          # 向量存储和搜索
│   ├── graph/           # 图数据和算法
│   ├── hybrid/          # 混合搜索引擎
│   ├── types/           # 数据类型定义
│   ├── errors/          # 错误处理
│   └── utils/           # 工具函数
├── tests/               # 集成测试
└── benches/            # 性能基准测试
```

### 核心组件

1. **DatabaseManager**: 管理 DuckDB 连接和数据库模式
2. **EmbeddingEngine**: 处理文本向量化和中文语义理解
3. **SemanticSearchEngine**: 执行向量相似度搜索
4. **CodeGraphAnalyzer**: 分析代码结构和依赖关系
5. **HybridSearchEngine**: 融合多种搜索策略

## ⚙️ 配置选项

### 数据库配置
- `database_path`: 数据库文件路径
- `max_connections`: 最大连接数
- `memory_limit`: 内存使用限制

### 嵌入配置
- `model_name`: 语义模型名称
- `vector_dimension`: 向量维度
- `batch_size`: 批处理大小
- `cache_size`: 缓存大小

### 性能配置
- `thread_pool_size`: 线程池大小
- `query_timeout`: 查询超时时间
- `enable_optimization`: 启用查询优化

## 🧪 测试

运行所有测试：

```bash
cargo test
```

运行特定模块测试：

```bash
cargo test embedding::
cargo test graph::
cargo test hybrid::
```

运行性能基准测试：

```bash
cargo bench
```

## 📊 性能指标

- **向量搜索**: < 10ms (1M 向量)
- **文本处理**: > 1000 docs/sec
- **内存使用**: < 500MB (默认配置)
- **并发支持**: 100+ 并发查询

## 🔮 发展路线

### 第一阶段 ✅ (已完成)
- [x] 基础框架搭建
- [x] 核心 API 设计
- [x] 单元测试覆盖
- [x] DuckDB 集成
- [x] BGE 模型集成

### 第二阶段 ✅ (已完成)
- [x] 向量索引优化
- [x] 代码解析器实现
- [x] 混合搜索算法优化
- [x] 性能基准测试

### 第三阶段 ✅ (已完成)
- [x] 图算法实现
- [x] API 客户端设计
- [x] 集成测试完善
- [x] 文档和示例

### 第四阶段 (规划中)
- [ ] 分布式支持
- [ ] 更多语言模型
- [ ] 图算法扩展
- [ ] 可视化界面

## 🤝 贡献

欢迎提交 Issue 和 Pull Request！

## 📄 许可证

MIT License

## 📞 联系

如有问题或建议，请提交 Issue 或联系维护者。
