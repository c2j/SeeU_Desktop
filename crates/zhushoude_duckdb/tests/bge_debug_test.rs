//! BGE模型调试测试 - 检查中间输出

use zhushoude_duckdb::{ZhushoudeDB, ZhushoudeConfig, EmbeddingConfig, Document, DocumentType};
use tokio;

#[tokio::test]
async fn test_bge_debug_output() {
    println!("🔬 BGE模型调试测试");

    // 创建配置
    let config = ZhushoudeConfig {
        database_path: ":memory:".to_string(),
        embedding: EmbeddingConfig {
            model_name: "bge-small-zh".to_string(),
            batch_size: 16,
            max_cache_size: 100,
            vector_dimension: 384,
            enable_chinese_optimization: true,
            normalize_vectors: false, // 关闭归一化来检查原始向量
        },
        ..Default::default()
    };

    // 初始化数据库
    let db = ZhushoudeDB::new(config).await.expect("Failed to create database");

    // 测试简单的文档
    let test_docs = vec![
        Document {
            id: "doc1".to_string(),
            title: "A".to_string(),
            content: "A".to_string(),
            doc_type: DocumentType::Note,
            metadata: serde_json::json!({}),
        },
        Document {
            id: "doc2".to_string(),
            title: "B".to_string(),
            content: "B".to_string(),
            doc_type: DocumentType::Note,
            metadata: serde_json::json!({}),
        },
        Document {
            id: "doc3".to_string(),
            title: "哲学".to_string(),
            content: "哲学".to_string(),
            doc_type: DocumentType::Note,
            metadata: serde_json::json!({}),
        },
        Document {
            id: "doc4".to_string(),
            title: "数学".to_string(),
            content: "数学".to_string(),
            doc_type: DocumentType::Note,
            metadata: serde_json::json!({}),
        },
    ];

    // 添加文档
    for doc in &test_docs {
        db.add_note(doc).await.expect("Failed to add document");
    }

    println!("\n📊 调试BGE向量输出:");

    // 测试不同查询
    let test_queries = ["A", "B", "哲学", "数学"];

    for query in &test_queries {
        println!("\n🔍 查询: '{}'", query);
        let results = db.search_notes(query, 4).await.expect("Search failed");
        
        for (i, result) in results.iter().enumerate() {
            println!("  {}. '{}' (score: {:.8})", i+1, result.title, result.similarity_score);
        }
        
        // 分析分数分布
        if results.len() >= 2 {
            let scores: Vec<f64> = results.iter().map(|r| r.similarity_score).collect();
            let min_score = scores.iter().fold(f64::INFINITY, |a, &b| a.min(b));
            let max_score = scores.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
            
            println!("    📈 分数范围: {:.8} - {:.8} (差值: {:.8})", 
                min_score, max_score, max_score - min_score);
        }
    }

    // 测试完全不同的内容
    println!("\n🧪 测试极端差异:");
    
    let extreme_docs = vec![
        Document {
            id: "extreme1".to_string(),
            title: "数字".to_string(),
            content: "1234567890".to_string(),
            doc_type: DocumentType::Note,
            metadata: serde_json::json!({}),
        },
        Document {
            id: "extreme2".to_string(),
            title: "符号".to_string(),
            content: "!@#$%^&*()".to_string(),
            doc_type: DocumentType::Note,
            metadata: serde_json::json!({}),
        },
    ];

    for doc in &extreme_docs {
        db.add_note(doc).await.expect("Failed to add document");
    }

    let extreme_results = db.search_notes("测试", 6).await.expect("Search failed");
    println!("\n🔍 极端测试结果:");
    for (i, result) in extreme_results.iter().enumerate() {
        println!("  {}. '{}' (score: {:.8})", i+1, result.title, result.similarity_score);
    }

    // 分析整体分布
    if extreme_results.len() >= 2 {
        let scores: Vec<f64> = extreme_results.iter().map(|r| r.similarity_score).collect();
        let min_score = scores.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max_score = scores.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        let avg_score = scores.iter().sum::<f64>() / scores.len() as f64;
        let variance = scores.iter()
            .map(|&x| (x - avg_score).powi(2))
            .sum::<f64>() / scores.len() as f64;
        let std_dev = variance.sqrt();
        
        println!("\n📊 整体统计:");
        println!("  最小值: {:.8}", min_score);
        println!("  最大值: {:.8}", max_score);
        println!("  平均值: {:.8}", avg_score);
        println!("  标准差: {:.8}", std_dev);
        println!("  范围: {:.8}", max_score - min_score);
        
        // 判断问题严重程度
        if std_dev < 0.0001 {
            println!("  ❌ 严重问题: 向量几乎完全相同 (std_dev < 0.0001)");
        } else if std_dev < 0.001 {
            println!("  ⚠️  中等问题: 向量区分度很低 (std_dev < 0.001)");
        } else if std_dev < 0.01 {
            println!("  ⚠️  轻微问题: 向量区分度较低 (std_dev < 0.01)");
        } else {
            println!("  ✅ 向量具有合理的区分度");
        }
        
        if max_score - min_score < 0.001 {
            println!("  ❌ 严重问题: 相似度范围极小");
        } else if max_score - min_score < 0.01 {
            println!("  ⚠️  轻微问题: 相似度范围较小");
        } else {
            println!("  ✅ 相似度范围合理");
        }
    }
}
