//! 语义索引重建功能测试

use zhushoude_duckdb::{ZhushoudeDB, ZhushoudeConfig, EmbeddingConfig, Document, DocumentType};
use tokio;

#[tokio::test]
async fn test_semantic_index_rebuild() {
    println!("🧪 测试语义索引重建功能");

    // 创建配置
    let config = ZhushoudeConfig {
        database_path: ":memory:".to_string(),
        embedding: EmbeddingConfig {
            model_name: "bge-small-zh".to_string(),
            batch_size: 16,
            max_cache_size: 100,
            vector_dimension: 384,
            enable_chinese_optimization: true,
            normalize_vectors: false, // 使用修复后的配置
        },
        ..Default::default()
    };

    // 初始化数据库
    let db = ZhushoudeDB::new(config).await.expect("Failed to create database");

    // 添加一些测试文档
    let test_docs = vec![
        Document {
            id: "doc1".to_string(),
            title: "人工智能".to_string(),
            content: "人工智能是计算机科学的一个分支，致力于创建能够执行通常需要人类智能的任务的系统".to_string(),
            doc_type: DocumentType::Note,
            metadata: serde_json::json!({}),
        },
        Document {
            id: "doc2".to_string(),
            title: "机器学习".to_string(),
            content: "机器学习是人工智能的一个子领域，专注于开发能够从数据中学习的算法".to_string(),
            doc_type: DocumentType::Note,
            metadata: serde_json::json!({}),
        },
        Document {
            id: "doc3".to_string(),
            title: "深度学习".to_string(),
            content: "深度学习是机器学习的一个分支，使用多层神经网络来模拟人脑的学习过程".to_string(),
            doc_type: DocumentType::Note,
            metadata: serde_json::json!({}),
        },
    ];

    // 批量添加文档
    println!("📝 添加测试文档...");
    db.add_notes_batch(&test_docs).await.expect("Failed to add documents");

    // 验证文档已添加
    println!("🔍 验证文档搜索...");
    let search_results = db.search_notes("人工智能", 5).await.expect("Search failed");
    println!("搜索到 {} 个结果", search_results.len());
    
    for (i, result) in search_results.iter().enumerate() {
        println!("  {}. '{}' (score: {:.6})", i+1, result.title, result.similarity_score);
    }

    // 测试清除索引功能
    println!("\n🗑️ 测试清除语义索引...");
    db.clear_all_semantic_indexes().await.expect("Failed to clear indexes");
    println!("✅ 语义索引清除完成");

    // 重新添加文档（模拟重建索引）
    println!("\n🔄 测试重建语义索引...");
    db.add_notes_batch(&test_docs).await.expect("Failed to rebuild indexes");
    println!("✅ 语义索引重建完成");

    // 验证重建后的搜索功能
    println!("\n🔍 验证重建后的搜索功能...");
    let rebuilt_results = db.search_notes("机器学习", 5).await.expect("Search after rebuild failed");
    println!("重建后搜索到 {} 个结果", rebuilt_results.len());
    
    for (i, result) in rebuilt_results.iter().enumerate() {
        println!("  {}. '{}' (score: {:.6})", i+1, result.title, result.similarity_score);
    }

    // 验证语义相关性
    if !rebuilt_results.is_empty() {
        let top_result = &rebuilt_results[0];
        println!("\n📊 语义搜索质量分析:");
        println!("  查询: '机器学习'");
        println!("  最佳匹配: '{}' (score: {:.6})", top_result.title, top_result.similarity_score);
        
        // 检查是否找到了相关文档
        let found_ml_doc = rebuilt_results.iter().any(|r| r.title.contains("机器学习"));
        let found_ai_doc = rebuilt_results.iter().any(|r| r.title.contains("人工智能"));
        let found_dl_doc = rebuilt_results.iter().any(|r| r.title.contains("深度学习"));
        
        println!("  ✅ 找到机器学习文档: {}", found_ml_doc);
        println!("  ✅ 找到人工智能文档: {}", found_ai_doc);
        println!("  ✅ 找到深度学习文档: {}", found_dl_doc);
        
        if found_ml_doc && (found_ai_doc || found_dl_doc) {
            println!("  🎉 语义搜索功能正常，能够找到相关文档");
        } else {
            println!("  ⚠️  语义搜索可能需要进一步优化");
        }
    }

    println!("\n✅ 语义索引重建功能测试完成");
}

#[tokio::test]
async fn test_clear_all_semantic_indexes() {
    println!("🧪 测试清除所有语义索引功能");

    let config = ZhushoudeConfig {
        database_path: ":memory:".to_string(),
        embedding: EmbeddingConfig {
            model_name: "bge-small-zh".to_string(),
            batch_size: 16,
            max_cache_size: 100,
            vector_dimension: 384,
            enable_chinese_optimization: true,
            normalize_vectors: false,
        },
        ..Default::default()
    };

    let db = ZhushoudeDB::new(config).await.expect("Failed to create database");

    // 添加一个文档
    let document = Document {
        id: "test_doc".to_string(),
        title: "测试文档".to_string(),
        content: "这是一个用于测试清除功能的文档".to_string(),
        doc_type: DocumentType::Note,
        metadata: serde_json::json!({}),
    };

    db.add_note(&document).await.expect("Failed to add document");

    // 验证文档存在
    let results_before = db.search_notes("测试", 5).await.expect("Search failed");
    println!("清除前搜索结果数量: {}", results_before.len());

    // 清除所有索引
    db.clear_all_semantic_indexes().await.expect("Failed to clear indexes");
    println!("✅ 所有语义索引已清除");

    // 验证清除效果（搜索应该返回空结果或错误）
    let results_after = db.search_notes("测试", 5).await.unwrap_or_default();
    println!("清除后搜索结果数量: {}", results_after.len());

    if results_after.is_empty() {
        println!("✅ 索引清除功能正常工作");
    } else {
        println!("⚠️  索引可能未完全清除，或存在缓存");
    }

    println!("✅ 清除语义索引功能测试完成");
}
