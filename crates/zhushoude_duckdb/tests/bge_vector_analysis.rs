//! BGE模型向量分析测试

use zhushoude_duckdb::{ZhushoudeDB, ZhushoudeConfig, EmbeddingConfig, Document, DocumentType};
use tokio;

#[tokio::test]
async fn test_bge_vector_analysis() {
    println!("🔬 BGE模型向量分析测试");

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

    // 测试文档 - 语义差异明显的文档
    let test_docs = vec![
        Document {
            id: "doc1".to_string(),
            title: "哲学思考".to_string(),
            content: "哲学家思考存在的意义，探讨人生的本质，研究知识的来源和真理的标准".to_string(),
            doc_type: DocumentType::Note,
            metadata: serde_json::json!({}),
        },
        Document {
            id: "doc2".to_string(),
            title: "数学计算".to_string(),
            content: "数学是研究数量、结构、变化和空间的学科，使用严格的逻辑推理".to_string(),
            doc_type: DocumentType::Note,
            metadata: serde_json::json!({}),
        },
        Document {
            id: "doc3".to_string(),
            title: "烹饪技巧".to_string(),
            content: "烹饪是制作食物的艺术，需要掌握火候、调料和食材的搭配".to_string(),
            doc_type: DocumentType::Note,
            metadata: serde_json::json!({}),
        },
        Document {
            id: "doc4".to_string(),
            title: "体育运动".to_string(),
            content: "体育运动能够锻炼身体，提高身体素质，培养团队合作精神".to_string(),
            doc_type: DocumentType::Note,
            metadata: serde_json::json!({}),
        },
    ];

    // 添加文档
    for doc in &test_docs {
        db.add_note(doc).await.expect("Failed to add document");
    }

    println!("\n📊 语义相似性分析:");

    // 测试不同查询的语义相似性
    let test_queries = [
        ("思考", "应该与哲学相关"),
        ("计算", "应该与数学相关"), 
        ("做饭", "应该与烹饪相关"),
        ("运动", "应该与体育相关"),
        ("人工智能", "测试无关内容"),
    ];

    for (query, expected) in &test_queries {
        println!("\n🔍 搜索查询: '{}' ({})", query, expected);
        let results = db.search_notes(query, 4).await.expect("Search failed");
        
        for (i, result) in results.iter().enumerate() {
            println!("  {}. '{}' (score: {:.6})", i+1, result.title, result.similarity_score);
        }
        
        // 分析结果的合理性
        if !results.is_empty() {
            let top_result = &results[0];
            let score_range = if results.len() > 1 {
                results[0].similarity_score - results[results.len()-1].similarity_score
            } else {
                0.0
            };
            
            println!("    📈 最高分: {:.6}, 分数范围: {:.6}", top_result.similarity_score, score_range);
            
            if score_range < 0.01 {
                println!("    ⚠️  分数差异很小，可能存在向量区分度问题");
            } else {
                println!("    ✅ 分数差异合理");
            }
        }
    }

    // 检查所有文档的相似度分数分布
    println!("\n📊 整体相似度分析:");
    let all_results = db.search_notes("测试", 10).await.expect("Search failed");
    
    if all_results.len() >= 2 {
        let scores: Vec<f64> = all_results.iter().map(|r| r.similarity_score).collect();
        let min_score = scores.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max_score = scores.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        let avg_score = scores.iter().sum::<f64>() / scores.len() as f64;
        let std_dev = {
            let variance = scores.iter()
                .map(|&x| (x - avg_score).powi(2))
                .sum::<f64>() / scores.len() as f64;
            variance.sqrt()
        };
        
        println!("  📈 统计信息:");
        println!("    最小值: {:.6}", min_score);
        println!("    最大值: {:.6}", max_score);
        println!("    平均值: {:.6}", avg_score);
        println!("    标准差: {:.6}", std_dev);
        println!("    范围: {:.6}", max_score - min_score);
        
        // 判断向量质量
        if std_dev < 0.001 {
            println!("  ❌ 严重问题: 向量几乎完全相同");
        } else if std_dev < 0.01 {
            println!("  ⚠️  轻微问题: 向量区分度较低");
        } else {
            println!("  ✅ 向量具有良好的区分度");
        }
        
        if max_score - min_score > 0.1 {
            println!("  ✅ 相似度范围合理");
        } else {
            println!("  ⚠️  相似度范围较小，可能需要检查模型配置");
        }
    }
}
