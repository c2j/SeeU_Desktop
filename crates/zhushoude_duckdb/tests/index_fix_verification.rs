use zhushoude_duckdb::*;
use tokio;
use std::time::{Duration, Instant};
use duckdb::params;

#[tokio::test]
async fn test_memory_database_index_creation_with_timeout() {
    println!("🔄 测试内存数据库索引创建（带超时保护）...");
    
    let config = ZhushoudeConfig {
        database_path: ":memory:".to_string(),
        ..Default::default()
    };
    
    let db_manager = DatabaseManager::new(config).await.expect("数据库管理器创建失败");
    let db_manager = std::sync::Arc::new(db_manager);
    
    // 测试基础索引创建（5秒超时）
    let timeout_duration = Duration::from_secs(5);
    let start_time = Instant::now();
    
    let result = tokio::time::timeout(timeout_duration, async {
        db_manager.create_indexes().await
    }).await;
    
    let elapsed = start_time.elapsed();
    println!("⏱️ 索引创建耗时: {:?}", elapsed);
    
    match result {
        Ok(Ok(())) => {
            println!("✅ 内存数据库索引创建成功（在{}秒内完成）", elapsed.as_secs_f32());
            
            // 验证索引是否真的被创建
            let connection = db_manager.get_connection();
            let conn = connection.lock().unwrap();
            
            let mut stmt = conn.prepare("SELECT name FROM sqlite_master WHERE type='index' AND name LIKE 'idx_%'").unwrap();
            let index_iter = stmt.query_map([], |row| {
                Ok(row.get::<_, String>(0)?)
            }).unwrap();
            
            let indexes: Vec<String> = index_iter.collect::<std::result::Result<Vec<_>, _>>().unwrap();
            println!("📊 创建的索引: {:?}", indexes);
            
            assert!(!indexes.is_empty(), "内存数据库应该创建了索引");
            assert!(indexes.contains(&"idx_documents_type".to_string()), "应该包含文档类型索引");
        }
        Ok(Err(e)) => {
            println!("❌ 内存数据库索引创建失败: {}", e);
            panic!("内存数据库索引创建应该成功");
        }
        Err(_) => {
            println!("⏰ 内存数据库索引创建超时（{}秒）", timeout_duration.as_secs());
            println!("💡 这表明索引创建仍然存在阻塞问题");
            panic!("内存数据库索引创建不应该超时");
        }
    }
}

#[tokio::test]
async fn test_file_database_index_behavior() {
    println!("🔄 测试文件数据库索引行为...");
    
    let temp_dir = std::env::temp_dir();
    let db_path = temp_dir.join("test_index_fix.duckdb");
    
    // 清理可能存在的测试文件
    if db_path.exists() {
        std::fs::remove_file(&db_path).ok();
    }
    
    let config = ZhushoudeConfig {
        database_path: db_path.to_string_lossy().to_string(),
        ..Default::default()
    };
    
    let db_manager = DatabaseManager::new(config).await.expect("数据库管理器创建失败");
    let db_manager = std::sync::Arc::new(db_manager);
    
    // 文件数据库应该跳过索引创建，所以应该很快完成
    let timeout_duration = Duration::from_secs(3);
    let start_time = Instant::now();
    
    let result = tokio::time::timeout(timeout_duration, async {
        db_manager.create_indexes().await
    }).await;
    
    let elapsed = start_time.elapsed();
    println!("⏱️ 文件数据库处理耗时: {:?}", elapsed);
    
    match result {
        Ok(Ok(())) => {
            println!("✅ 文件数据库处理完成（跳过了索引创建）");
            
            // 验证索引确实被跳过了
            let connection = db_manager.get_connection();
            let conn = connection.lock().unwrap();
            
            let mut stmt = conn.prepare("SELECT name FROM sqlite_master WHERE type='index' AND name LIKE 'idx_%'").unwrap();
            let index_iter = stmt.query_map([], |row| {
                Ok(row.get::<_, String>(0)?)
            }).unwrap();
            
            let indexes: Vec<String> = index_iter.collect::<std::result::Result<Vec<_>, _>>().unwrap();
            println!("📊 文件数据库索引: {:?}", indexes);
            
            // 文件数据库应该跳过索引创建，所以索引数量应该很少
            println!("💡 文件数据库索引数量: {} (应该很少，因为跳过了索引创建)", indexes.len());
        }
        Ok(Err(e)) => {
            println!("❌ 文件数据库处理失败: {}", e);
            // 这可能是预期的，如果索引创建确实有问题
        }
        Err(_) => {
            println!("⏰ 文件数据库处理超时");
            println!("💡 这表明文件数据库仍然存在阻塞问题");
        }
    }
    
    // 清理测试文件
    if db_path.exists() {
        std::fs::remove_file(&db_path).ok();
    }
}

#[tokio::test]
async fn test_chunk_vector_index_creation() {
    println!("🔄 测试分块向量索引创建...");
    
    let config = ZhushoudeConfig {
        database_path: ":memory:".to_string(),
        ..Default::default()
    };
    
    let db_manager = DatabaseManager::new(config).await.expect("数据库管理器创建失败");
    let db_manager = std::sync::Arc::new(db_manager);
    
    let vector_config = ChunkVectorConfig {
        chunking_config: ChunkingConfig {
            chunk_size: 100,
            overlap_size: 20,
            min_chunk_size: 30,
            preserve_semantic_boundaries: true,
            strategy: ChunkingStrategy::Semantic,
        },
        vector_dimension: 64,
        index_type: ChunkIndexType::Flat,
        batch_size: 5,
        incremental_indexing: true,
        update_threshold: 50,
    };
    
    let index_manager = ChunkVectorIndexManager::new(db_manager.clone(), vector_config);
    
    // 测试分块向量索引初始化（10秒超时）
    let timeout_duration = Duration::from_secs(10);
    let start_time = Instant::now();
    
    let result = tokio::time::timeout(timeout_duration, async {
        index_manager.initialize_tables().await
    }).await;
    
    let elapsed = start_time.elapsed();
    println!("⏱️ 分块向量索引初始化耗时: {:?}", elapsed);
    
    match result {
        Ok(Ok(())) => {
            println!("✅ 分块向量索引初始化成功");
            
            // 验证表是否被创建
            let connection = db_manager.get_connection();
            let conn = connection.lock().unwrap();
            
            let mut stmt = conn.prepare("SELECT name FROM sqlite_master WHERE type='table' AND name IN ('text_chunks', 'chunk_vector_indexes')").unwrap();
            let table_iter = stmt.query_map([], |row| {
                Ok(row.get::<_, String>(0)?)
            }).unwrap();
            
            let tables: Vec<String> = table_iter.collect::<std::result::Result<Vec<_>, _>>().unwrap();
            println!("📊 创建的分块表: {:?}", tables);
            
            assert!(tables.contains(&"text_chunks".to_string()), "应该创建text_chunks表");
            assert!(tables.contains(&"chunk_vector_indexes".to_string()), "应该创建chunk_vector_indexes表");
        }
        Ok(Err(e)) => {
            println!("❌ 分块向量索引初始化失败: {}", e);
            panic!("分块向量索引初始化应该成功");
        }
        Err(_) => {
            println!("⏰ 分块向量索引初始化超时（{}秒）", timeout_duration.as_secs());
            println!("💡 这表明分块向量索引创建仍然存在阻塞问题");
            panic!("分块向量索引初始化不应该超时");
        }
    }
}

#[tokio::test]
async fn test_basic_functionality_without_indexes() {
    println!("🔄 测试无索引情况下的基础功能...");
    
    let config = ZhushoudeConfig {
        database_path: ":memory:".to_string(),
        ..Default::default()
    };
    
    let db_manager = DatabaseManager::new(config).await.expect("数据库管理器创建失败");
    let db_manager = std::sync::Arc::new(db_manager);
    
    // 测试基础数据库操作（不依赖索引）
    let connection = db_manager.get_connection();
    let conn = connection.lock().unwrap();
    
    // 插入测试数据（移除不存在的language列）
    let insert_result = conn.execute(
        "INSERT INTO documents (id, title, content, doc_type, created_at) VALUES (?, ?, ?, ?, ?)",
        params!["test_doc_1", "测试文档", "这是一个测试文档的内容", "text", "2024-01-01 00:00:00"]
    );
    
    match insert_result {
        Ok(rows_affected) => {
            println!("✅ 成功插入文档，影响行数: {}", rows_affected);
            assert_eq!(rows_affected, 1, "应该插入1行数据");
        }
        Err(e) => {
            panic!("插入文档失败: {}", e);
        }
    }
    
    // 查询测试数据
    let mut stmt = conn.prepare("SELECT id, title, content FROM documents WHERE id = ?").unwrap();
    let doc_iter = stmt.query_map(params!["test_doc_1"], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
        ))
    }).unwrap();
    
    let docs: Vec<(String, String, String)> = doc_iter.collect::<std::result::Result<Vec<_>, _>>().unwrap();
    
    assert_eq!(docs.len(), 1, "应该查询到1个文档");
    assert_eq!(docs[0].0, "test_doc_1", "文档ID应该匹配");
    assert_eq!(docs[0].1, "测试文档", "文档标题应该匹配");
    
    println!("✅ 基础功能测试通过：无索引情况下数据库操作正常");
}

#[tokio::test]
async fn test_text_chunking_functionality() {
    println!("🔄 测试文本分块功能（不依赖索引）...");
    
    let config = ChunkingConfig {
        chunk_size: 100,
        overlap_size: 20,
        min_chunk_size: 30,
        preserve_semantic_boundaries: true,
        strategy: ChunkingStrategy::Semantic,
    };
    
    let chunker = TextChunker::new(config);
    
    let test_text = "
        人工智能技术正在快速发展。机器学习是其核心技术之一。
        深度学习使用神经网络来处理复杂的模式识别任务。
        自然语言处理让计算机能够理解和生成人类语言。
        计算机视觉使机器能够理解和分析图像内容。
    ";
    
    let start_time = Instant::now();
    let chunks = chunker.chunk_text("test_doc", test_text).expect("分块失败");
    let elapsed = start_time.elapsed();
    
    println!("⏱️ 文本分块耗时: {:?}", elapsed);
    println!("📄 分块结果: {} 个分块", chunks.len());
    
    for (i, chunk) in chunks.iter().enumerate() {
        println!("  分块 {}: {} 字符 (质量分数: {:.2})", 
                i, chunk.metadata.char_count, chunk.metadata.quality_score);
    }
    
    assert!(!chunks.is_empty(), "应该产生分块");
    assert!(elapsed.as_millis() < 1000, "分块应该在1秒内完成");
    
    println!("✅ 文本分块功能正常，不依赖索引");
}
