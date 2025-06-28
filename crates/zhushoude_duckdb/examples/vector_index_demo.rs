//! 向量索引和搜索演示
//! 
//! 这个示例展示了如何使用zhushoude_duckdb的向量索引和搜索功能

use zhushoude_duckdb::*;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🚀 向量索引和搜索演示");
    println!("{}", "=".repeat(60));
    
    // 1. 创建配置
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
        performance: PerformanceConfig {
            thread_pool_size: Some(4),
            memory_limit_mb: 512,
            enable_monitoring: true,
            cache_strategy: CacheStrategy::LRU,
        },
        ..Default::default()
    };
    
    println!("📋 配置信息:");
    println!("  - 数据库路径: {}", config.database_path);
    println!("  - 向量维度: {}", config.embedding.vector_dimension);
    println!("  - 内存限制: {}MB", config.performance.memory_limit_mb);
    println!();
    
    // 2. 初始化数据库管理器
    println!("🔧 初始化数据库...");
    let db_manager = Arc::new(DatabaseManager::new(config.clone()).await?);
    println!("✅ 数据库初始化完成");
    
    // 3. 创建向量索引管理器
    println!("🔧 初始化向量索引管理器...");
    let mut index_manager = VectorIndexManager::new(db_manager.clone());
    index_manager.initialize().await?;
    println!("✅ 向量索引管理器初始化完成");
    
    // 4. 演示不同类型的索引创建
    println!("\n📊 创建不同类型的向量索引:");
    
    // 创建线性索引
    let linear_index = index_manager.create_index(
        "document_embeddings",
        "embedding", 
        IndexType::Linear,
        512
    ).await?;
    println!("  ✅ 线性索引: {}", linear_index);
    
    // 创建哈希索引
    let hash_index = index_manager.create_index(
        "code_embeddings",
        "embedding",
        IndexType::Hash { num_buckets: 50 },
        256
    ).await?;
    println!("  ✅ 哈希索引: {}", hash_index);
    
    // 创建聚类索引
    let cluster_index = index_manager.create_index(
        "note_embeddings",
        "embedding",
        IndexType::Cluster { num_clusters: 20 },
        384
    ).await?;
    println!("  ✅ 聚类索引: {}", cluster_index);
    
    // 创建自适应索引
    let adaptive_index = index_manager.create_index(
        "mixed_embeddings",
        "embedding",
        IndexType::Adaptive,
        128
    ).await?;
    println!("  ✅ 自适应索引: {}", adaptive_index);
    
    // 5. 列出所有索引
    println!("\n📋 当前索引列表:");
    let indexes = index_manager.list_indexes();
    for (i, index_name) in indexes.iter().enumerate() {
        if let Some(info) = index_manager.get_index_info(index_name) {
            println!("  {}. {} (类型: {:?}, 维度: {})", 
                i + 1, index_name, info.index_type, info.dimension);
        }
    }
    
    // 6. 获取索引统计信息
    println!("\n📊 索引统计信息:");
    for index_name in &indexes {
        match index_manager.get_index_stats(index_name).await {
            Ok(stats) => {
                println!("  📈 {}: {} 个向量, {} bytes, 类型: {}", 
                    index_name, stats.num_vectors, stats.size_bytes, stats.index_type);
            }
            Err(e) => {
                println!("  ❌ {}: 获取统计失败 - {}", index_name, e);
            }
        }
    }
    
    // 7. 演示向量搜索
    println!("\n🔍 向量搜索演示:");
    
    // 创建测试向量
    let test_vectors = vec![
        vec![0.1, 0.2, 0.3, 0.4, 0.5],
        vec![0.5, 0.4, 0.3, 0.2, 0.1],
        vec![0.3, 0.3, 0.3, 0.3, 0.3],
    ];
    
    for (i, query_vector) in test_vectors.iter().enumerate() {
        println!("  🔎 搜索向量 {}: {:?}", i + 1, query_vector);
        
        // 使用线性索引搜索
        match index_manager.search_with_index(&linear_index, query_vector, 5, 0.0).await {
            Ok(results) => {
                println!("    📊 找到 {} 个结果", results.len());
                for (j, result) in results.iter().enumerate() {
                    println!("      {}. ID: {}, 相似度: {:.4}", 
                        j + 1, result.id, result.similarity);
                }
            }
            Err(e) => {
                println!("    ❌ 搜索失败: {}", e);
            }
        }
    }
    
    // 8. 索引优化演示
    println!("\n⚡ 索引优化演示:");
    for index_name in &indexes {
        println!("  🔧 优化索引: {}", index_name);
        match index_manager.optimize_index(index_name).await {
            Ok(_) => println!("    ✅ 优化完成"),
            Err(e) => println!("    ❌ 优化失败: {}", e),
        }
    }
    
    // 9. 更新索引统计
    println!("\n📊 更新索引统计:");
    for index_name in &indexes {
        match index_manager.update_index_stats(index_name).await {
            Ok(_) => println!("  ✅ 更新统计: {}", index_name),
            Err(e) => println!("  ❌ 更新失败: {} - {}", index_name, e),
        }
    }
    
    // 10. 性能测试
    println!("\n⏱️ 性能测试:");
    let start_time = std::time::Instant::now();
    
    // 执行多次搜索
    let query_vector = vec![0.2, 0.4, 0.6, 0.8, 1.0];
    for i in 0..10 {
        let _ = index_manager.search_with_index(&linear_index, &query_vector, 3, 0.0).await;
        if i % 3 == 0 {
            print!(".");
        }
    }
    println!();
    
    let duration = start_time.elapsed();
    println!("  📊 10次搜索耗时: {:?}", duration);
    println!("  📊 平均搜索时间: {:?}", duration / 10);
    
    // 11. 清理演示
    println!("\n🧹 清理索引:");
    for index_name in indexes {
        match index_manager.drop_index(&index_name).await {
            Ok(_) => println!("  ✅ 删除索引: {}", index_name),
            Err(e) => println!("  ❌ 删除失败: {} - {}", index_name, e),
        }
    }
    
    let remaining_indexes = index_manager.list_indexes();
    println!("  📋 剩余索引数量: {}", remaining_indexes.len());
    
    println!("\n🎉 向量索引和搜索演示完成！");
    println!("💡 提示: 这个演示展示了zhushoude_duckdb的向量索引管理功能");
    println!("   包括多种索引类型、搜索优化和性能监控等。");
    
    Ok(())
}
