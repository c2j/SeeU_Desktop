use zhushoude_duckdb::*;
use tokio;
use std::path::PathBuf;
use tempfile::TempDir;

#[tokio::test]
async fn test_vector_index_initialization_fix() {
    println!("🔄 测试向量索引初始化修复...");
    
    // 创建临时目录
    let temp_dir = TempDir::new().expect("创建临时目录失败");
    let db_path = temp_dir.path().join("test_fix.db");
    
    let config = ZhushoudeConfig {
        database_path: db_path.to_string_lossy().to_string(),
        ..Default::default()
    };
    
    println!("📋 配置信息:");
    println!("  - 数据库路径: {}", config.database_path);
    println!("  - 向量维度: {}", config.embedding.vector_dimension);
    
    // 第一次创建数据库实例（应该创建索引）
    println!("\n🔧 第一次创建数据库实例...");
    let db1 = ZhushoudeDB::new(config.clone()).await.expect("创建数据库实例失败");
    println!("✅ 第一次创建完成");
    
    // 添加一些测试文档
    let test_documents = vec![
        Document {
            id: "doc1".to_string(),
            title: "人工智能技术".to_string(),
            content: "人工智能技术正在快速发展，机器学习和深度学习是其核心技术。".to_string(),
            doc_type: DocumentType::Note,
            metadata: serde_json::json!({}),
        },
        Document {
            id: "doc2".to_string(),
            title: "量子计算".to_string(),
            content: "量子计算是一种全新的计算范式，利用量子力学原理进行信息处理。".to_string(),
            doc_type: DocumentType::Note,
            metadata: serde_json::json!({}),
        },
    ];
    
    println!("\n📄 添加测试文档...");
    for doc in &test_documents {
        db1.add_note(doc).await.expect("添加文档失败");
        println!("  ✅ 添加文档: {}", doc.title);
    }
    
    // 测试搜索功能
    println!("\n🔍 测试搜索功能...");
    let search_results = db1.search_notes("人工智能", 5).await.expect("搜索失败");
    println!("  📊 搜索结果数量: {}", search_results.len());
    
    for (i, result) in search_results.iter().enumerate() {
        println!("  {}. {} (相似度: {:.3})", i + 1, result.title, result.similarity_score);
    }
    
    // 释放第一个实例
    drop(db1);
    println!("\n🗑️ 释放第一个数据库实例");
    
    // 第二次创建数据库实例（应该加载现有索引）
    println!("\n🔧 第二次创建数据库实例...");
    let db2 = ZhushoudeDB::new(config.clone()).await.expect("创建数据库实例失败");
    println!("✅ 第二次创建完成");
    
    // 测试搜索功能（应该能找到之前的文档）
    println!("\n🔍 测试持久化后的搜索功能...");
    let search_results2 = db2.search_notes("量子", 5).await.expect("搜索失败");
    println!("  📊 搜索结果数量: {}", search_results2.len());
    
    for (i, result) in search_results2.iter().enumerate() {
        println!("  {}. {} (相似度: {:.3})", i + 1, result.title, result.similarity_score);
    }
    
    // 验证搜索结果不为空
    assert!(!search_results2.is_empty(), "搜索结果不应为空");
    
    // 添加新文档测试增量索引
    println!("\n📄 添加新文档测试增量索引...");
    let new_doc = Document {
        id: "doc3".to_string(),
        title: "区块链技术".to_string(),
        content: "区块链技术提供了去中心化的解决方案，通过密码学确保数据安全。".to_string(),
        doc_type: DocumentType::Note,
        metadata: serde_json::json!({}),
    };
    
    db2.add_note(&new_doc).await.expect("添加新文档失败");
    println!("  ✅ 添加新文档: {}", new_doc.title);
    
    // 测试新文档的搜索
    println!("\n🔍 测试新文档搜索...");
    let search_results3 = db2.search_notes("区块链", 5).await.expect("搜索失败");
    println!("  📊 搜索结果数量: {}", search_results3.len());
    
    for (i, result) in search_results3.iter().enumerate() {
        println!("  {}. {} (相似度: {:.3})", i + 1, result.title, result.similarity_score);
    }
    
    // 验证能找到新文档
    assert!(!search_results3.is_empty(), "应该能找到新添加的文档");
    assert!(search_results3.iter().any(|r| r.title.contains("区块链")), "应该能找到区块链相关文档");
    
    println!("\n✅ 向量索引初始化修复测试完成");
}

#[tokio::test]
async fn test_memory_database_vector_index() {
    println!("🔄 测试内存数据库向量索引...");
    
    let config = ZhushoudeConfig {
        database_path: ":memory:".to_string(),
        ..Default::default()
    };
    
    let db = ZhushoudeDB::new(config).await.expect("创建内存数据库失败");
    
    // 添加测试文档
    let doc = Document {
        id: "memory_doc1".to_string(),
        title: "内存测试文档".to_string(),
        content: "这是一个内存数据库的测试文档，用于验证向量索引功能。".to_string(),
        doc_type: DocumentType::Note,
        metadata: serde_json::json!({}),
    };
    
    db.add_note(&doc).await.expect("添加文档失败");
    
    // 测试搜索
    let results = db.search_notes("测试", 5).await.expect("搜索失败");
    assert!(!results.is_empty(), "内存数据库搜索结果不应为空");
    
    println!("✅ 内存数据库向量索引测试完成");
}
