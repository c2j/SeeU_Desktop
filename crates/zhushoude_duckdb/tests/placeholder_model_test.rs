//! 测试占位符模型的向量生成和相似度计算

use zhushoude_duckdb::{ZhushoudeDB, ZhushoudeConfig, EmbeddingConfig, Document, DocumentType};
use tokio;

#[tokio::test]
async fn test_placeholder_model_vector_diversity() {
    println!("🧪 测试占位符模型向量多样性");

    // 创建配置
    let config = ZhushoudeConfig {
        database_path: ":memory:".to_string(),
        embedding: EmbeddingConfig {
            model_name: "bge-small-zh".to_string(),
            batch_size: 16,
            max_cache_size: 100,
            vector_dimension: 384,
            enable_chinese_optimization: true,
            normalize_vectors: true,
        },
        ..Default::default()
    };

    // 初始化数据库
    let db = ZhushoudeDB::new(config).await.expect("Failed to create database");

    // 测试文档
    let test_docs = vec![
        Document {
            id: "doc1".to_string(),
            title: "哲学家".to_string(),
            content: "哲学家研究存在、知识、价值、理性、心灵和语言等问题".to_string(),
            doc_type: DocumentType::Note,
            metadata: serde_json::json!({}),
        },
        Document {
            id: "doc2".to_string(),
            title: "科学家".to_string(),
            content: "科学家通过科学方法研究自然现象和社会现象".to_string(),
            doc_type: DocumentType::Note,
            metadata: serde_json::json!({}),
        },
        Document {
            id: "doc3".to_string(),
            title: "云数据库".to_string(),
            content: "云数据库是部署在云计算环境中的数据库服务".to_string(),
            doc_type: DocumentType::Note,
            metadata: serde_json::json!({}),
        },
    ];

    // 添加文档
    for doc in &test_docs {
        db.add_note(doc).await.expect("Failed to add document");
    }

    println!("\n📊 测试搜索结果:");

    // 测试搜索"哲学家"
    let results1 = db.search_notes("哲学家", 10).await.expect("Search failed");
    println!("\n🔍 搜索'哲学家'的结果:");
    for (i, result) in results1.iter().enumerate() {
        println!("  {}. '{}' (score: {:.6})", i+1, result.title, result.similarity_score);
    }

    // 测试搜索"科学家"
    let results2 = db.search_notes("科学家", 10).await.expect("Search failed");
    println!("\n🔍 搜索'科学家'的结果:");
    for (i, result) in results2.iter().enumerate() {
        println!("  {}. '{}' (score: {:.6})", i+1, result.title, result.similarity_score);
    }

    // 测试搜索"数据库"
    let results3 = db.search_notes("数据库", 10).await.expect("Search failed");
    println!("\n🔍 搜索'数据库'的结果:");
    for (i, result) in results3.iter().enumerate() {
        println!("  {}. '{}' (score: {:.6})", i+1, result.title, result.similarity_score);
    }

    // 分析相似度分数的多样性
    let mut all_scores = Vec::new();
    all_scores.extend(results1.iter().map(|r| r.similarity_score));
    all_scores.extend(results2.iter().map(|r| r.similarity_score));
    all_scores.extend(results3.iter().map(|r| r.similarity_score));

    if !all_scores.is_empty() {
        let min_score = all_scores.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max_score = all_scores.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        let avg_score = all_scores.iter().sum::<f64>() / all_scores.len() as f64;
        
        println!("\n📈 相似度分数统计:");
        println!("  最小值: {:.6}", min_score);
        println!("  最大值: {:.6}", max_score);
        println!("  平均值: {:.6}", avg_score);
        println!("  范围: {:.6}", max_score - min_score);
        
        if max_score - min_score < 0.001 {
            println!("❌ 问题确认: 所有相似度分数几乎相同，向量缺乏多样性");
        } else {
            println!("✅ 向量具有一定多样性");
        }
    }
}
