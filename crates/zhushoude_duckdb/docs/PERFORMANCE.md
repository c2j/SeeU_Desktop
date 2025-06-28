# zhushoude_duckdb 性能指南

## 概述

本文档提供了 zhushoude_duckdb 的性能优化指南，包括配置调优、最佳实践和性能基准测试结果。

## 性能基准

### 测试环境

- **CPU**: Intel i7-10700K @ 3.8GHz (8核16线程)
- **内存**: 32GB DDR4-3200
- **存储**: NVMe SSD
- **操作系统**: macOS 14.0
- **Rust版本**: 1.75.0

### 基准测试结果

#### 文档操作性能

| 操作 | 数量 | 平均时间 | 吞吐量 | 说明 |
|------|------|----------|--------|------|
| 添加文档 | 1个 | ~10ms | 100 docs/sec | 包含向量化 |
| 批量添加 | 100个 | ~800ms | 125 docs/sec | 批量优化 |
| 删除文档 | 1个 | ~2ms | 500 ops/sec | 数据库操作 |

#### 搜索性能

| 搜索类型 | 数据量 | 平均时间 | 吞吐量 | 说明 |
|----------|--------|----------|--------|------|
| 语义搜索 | 1K文档 | ~5ms | 200 QPS | 向量相似度 |
| 语义搜索 | 10K文档 | ~15ms | 67 QPS | 线性扫描 |
| 图搜索 | 1K节点 | ~8ms | 125 QPS | 图遍历 |
| 混合搜索 | 1K文档 | ~12ms | 83 QPS | 结果融合 |

#### 向量化性能

| 操作 | 数量 | 平均时间 | 吞吐量 | 说明 |
|------|------|----------|--------|------|
| 首次向量化 | 1个文本 | ~100ms | 10 texts/sec | 模型推理 |
| 缓存命中 | 1个文本 | ~0.1ms | 10K texts/sec | 内存访问 |
| 批量向量化 | 8个文本 | ~600ms | 13 texts/sec | 批量优化 |

#### 图算法性能

| 算法 | 图规模 | 平均时间 | 说明 |
|------|--------|----------|------|
| 最短路径 | 1K节点 | ~20ms | Dijkstra算法 |
| DFS | 1K节点 | ~15ms | 深度限制=5 |
| BFS | 1K节点 | ~18ms | 深度限制=5 |
| PageRank | 1K节点 | ~50ms | 10次迭代 |
| 社区检测 | 1K节点 | ~80ms | 连通分量 |

#### 内存使用

| 组件 | 内存使用 | 说明 |
|------|----------|------|
| BGE模型 | ~500MB | 模型权重 |
| 向量缓存 | ~50MB | 1000个向量 |
| 数据库 | ~100MB | 10K文档 |
| 图数据 | ~20MB | 1K节点+边 |
| **总计** | **~670MB** | 典型配置 |

## 性能优化

### 1. 配置优化

#### 内存配置

```rust
let config = ZhushoudeConfig {
    performance: PerformanceConfig {
        memory_limit_mb: 2048,  // 根据可用内存调整
        thread_pool_size: Some(8),  // CPU核心数
        enable_monitoring: true,
        cache_strategy: CacheStrategy::LRU,
    },
    embedding: EmbeddingConfig {
        max_cache_size: 5000,  // 增加缓存大小
        batch_size: 16,        // 增加批处理大小
        ..Default::default()
    },
    ..Default::default()
};
```

#### 数据库优化

```rust
let config = ZhushoudeConfig {
    database_path: "/fast/ssd/path/db.duckdb".to_string(),  // 使用SSD
    performance: PerformanceConfig {
        memory_limit_mb: 1024,  // DuckDB内存限制
        ..Default::default()
    },
    ..Default::default()
};
```

### 2. 使用模式优化

#### 批量操作

```rust
// ❌ 低效：逐个添加
for doc in documents {
    engine.add_document(&doc).await?;
}

// ✅ 高效：批量添加
engine.add_documents_batch(&documents).await?;
```

#### 缓存预热

```rust
// 预热常用查询的向量缓存
let common_queries = vec!["人工智能", "机器学习", "深度学习"];
for query in common_queries {
    engine.encode_single(query).await?;
}
```

#### 连接复用

```rust
// ✅ 复用引擎实例
let engine = Arc::new(ZhushoudeEngine::new(config).await?);

// 在多个任务中共享
let engine_clone = engine.clone();
tokio::spawn(async move {
    engine_clone.semantic_search("query", 10).await
});
```

### 3. 搜索优化

#### 结果数量限制

```rust
// ❌ 过多结果影响性能
let results = engine.semantic_search("query", 1000).await?;

// ✅ 合理的结果数量
let results = engine.semantic_search("query", 20).await?;
```

#### 混合搜索权重调优

```rust
// 根据使用场景调整权重
let query = HybridQuery {
    text: "查询文本".to_string(),
    weights: SearchWeights {
        semantic: 0.8,  // 语义搜索为主
        graph: 0.2,     // 图搜索为辅
    },
    ..Default::default()
};
```

### 4. 内存管理

#### 定期清理

```rust
// 定期清理缓存
if cache_stats.size > max_cache_size {
    engine.clear_cache().await?;
}

// 删除不需要的文档
engine.delete_document("old_doc_id").await?;
```

#### 监控内存使用

```rust
let stats = engine.get_system_stats().await?;
println!("内存使用: {:.2}MB", stats.performance.memory_usage_mb);

if stats.performance.memory_usage_mb > 1000.0 {
    // 触发内存清理
}
```

## 性能监控

### 1. 缓存监控

```rust
let cache_stats = engine.get_cache_stats();
println!("缓存命中率: {:.2}%", cache_stats.hit_rate * 100.0);
println!("缓存大小: {}", cache_stats.size);

// 缓存命中率低于80%时考虑增加缓存大小
if cache_stats.hit_rate < 0.8 {
    println!("建议增加缓存大小");
}
```

### 2. 查询性能监控

```rust
let start = std::time::Instant::now();
let results = engine.semantic_search("query", 10).await?;
let duration = start.elapsed();

println!("查询耗时: {:?}", duration);

// 查询时间超过100ms时需要优化
if duration.as_millis() > 100 {
    println!("查询性能需要优化");
}
```

### 3. 系统资源监控

```rust
let stats = engine.get_system_stats().await?;

println!("性能统计:");
println!("  平均查询时间: {:.2}ms", stats.performance.avg_query_time_ms);
println!("  查询吞吐量: {:.2} QPS", stats.performance.queries_per_second);
println!("  内存使用: {:.2}MB", stats.performance.memory_usage_mb);
println!("  CPU使用率: {:.2}%", stats.performance.cpu_usage_percent);
```

## 性能调优建议

### 1. 硬件建议

#### CPU
- **推荐**: 8核以上现代CPU
- **最低**: 4核CPU
- **优化**: 设置 `thread_pool_size` 为CPU核心数

#### 内存
- **推荐**: 16GB以上
- **最低**: 8GB
- **优化**: 模型(500MB) + 缓存(可配置) + 数据(可变)

#### 存储
- **推荐**: NVMe SSD
- **最低**: SATA SSD
- **避免**: 机械硬盘

### 2. 软件配置

#### 操作系统
- **Linux**: 最佳性能，推荐生产环境
- **macOS**: 良好性能，适合开发
- **Windows**: 基本支持

#### Rust配置
```toml
[profile.release]
opt-level = 3
lto = true
codegen-units = 1
panic = "abort"
```

### 3. 部署建议

#### 单机部署
- 使用内存数据库 (`:memory:`) 获得最佳性能
- 定期备份数据到持久化存储
- 监控内存使用，避免OOM

#### 生产环境
- 使用持久化数据库文件
- 配置适当的内存限制
- 实施监控和告警
- 定期性能测试

## 故障排除

### 1. 性能问题

#### 搜索慢
- 检查数据量是否过大
- 验证索引是否正确创建
- 考虑增加内存限制
- 优化查询条件

#### 内存使用高
- 检查缓存大小配置
- 清理不需要的文档
- 减少批处理大小
- 监控内存泄漏

#### CPU使用高
- 检查并发查询数量
- 优化向量化批处理
- 调整线程池大小
- 避免频繁的模型推理

### 2. 常见问题

#### 缓存命中率低
```rust
// 检查查询模式
let stats = engine.get_cache_stats();
if stats.hit_rate < 0.5 {
    // 增加缓存大小或优化查询
}
```

#### 向量化慢
```rust
// 使用批量处理
let texts = vec!["text1", "text2", "text3"];
let embeddings = engine.encode_batch(&texts).await?;
```

#### 数据库锁定
```rust
// 避免长时间事务
// 使用适当的并发控制
```

## 基准测试

### 运行基准测试

```bash
# 运行所有基准测试
cargo bench

# 运行特定基准测试
cargo bench --bench search_benchmark
cargo bench --bench embedding_benchmark
```

### 自定义基准测试

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_search(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let engine = rt.block_on(async {
        ZhushoudeEngine::new(ZhushoudeConfig::default()).await.unwrap()
    });
    
    c.bench_function("semantic_search", |b| {
        b.to_async(&rt).iter(|| async {
            engine.semantic_search(black_box("test query"), 10).await
        })
    });
}

criterion_group!(benches, benchmark_search);
criterion_main!(benches);
```

## 总结

zhushoude_duckdb 在合理配置下能够提供优秀的性能表现。关键的优化点包括：

1. **合理配置内存和线程数**
2. **使用批量操作**
3. **优化缓存策略**
4. **监控系统资源**
5. **选择合适的硬件**

通过遵循本指南的建议，您可以获得最佳的性能体验。
