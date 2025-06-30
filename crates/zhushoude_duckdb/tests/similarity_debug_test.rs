use zhushoude_duckdb::*;
use tokio;

#[tokio::test]
async fn test_similarity_calculation_debug() {
    println!("🔍 调试相似度计算...");
    
    let config = ZhushoudeConfig {
        database_path: ":memory:".to_string(),
        ..Default::default()
    };
    
    let db = ZhushoudeDB::new(config.clone()).await.expect("创建数据库失败");
    
    // 测试文档
    let docs = vec![
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
        Document {
            id: "doc3".to_string(),
            title: "区块链技术".to_string(),
            content: "区块链技术提供了去中心化的解决方案，通过密码学确保数据安全。".to_string(),
            doc_type: DocumentType::Note,
            metadata: serde_json::json!({}),
        },
    ];
    
    // 添加文档
    for doc in &docs {
        db.add_note(doc).await.expect("添加文档失败");
        println!("✅ 添加文档: {}", doc.title);
    }
    
    // 测试不同查询的相似度
    let queries = vec![
        "人工智能",
        "机器学习", 
        "量子",
        "区块链",
        "技术",
        "完全不相关的内容xyz123",
    ];
    
    for query in &queries {
        println!("\n🔍 查询: '{}'", query);
        let results = db.search_notes(query, 10).await.expect("搜索失败");
        
        for (i, result) in results.iter().enumerate() {
            println!("  {}. {} (相似度: {:.6})", i + 1, result.title, result.similarity_score);
        }
        
        if results.len() > 1 {
            // 检查相似度是否有差异
            let scores: Vec<f32> = results.iter().map(|r| r.similarity_score as f32).collect();
            let max_score = scores.iter().fold(f32::NEG_INFINITY, |a, &b| a.max(b));
            let min_score = scores.iter().fold(f32::INFINITY, |a, &b| a.min(b));
            let score_range = max_score - min_score;
            
            println!("  📊 相似度范围: {:.6} - {:.6} (差异: {:.6})", min_score, max_score, score_range);
            
            if score_range < 0.001 {
                println!("  ⚠️  警告: 所有相似度几乎相同，可能存在计算问题");
            }
        }
    }
    
    // 直接测试向量化
    println!("\n🧮 直接测试向量化...");
    
    // 获取嵌入引擎进行直接测试
    let embedding_engine = EmbeddingEngine::new(config.embedding.clone()).await.expect("创建嵌入引擎失败");
    
    let test_texts = vec![
        "人工智能技术",
        "量子计算",
        "区块链技术",
        "完全不同的内容",
    ];
    
    let mut vectors = Vec::new();
    for text in &test_texts {
        let vector = embedding_engine.encode_single(text).await.expect("向量化失败");
        vectors.push(vector);
        println!("📝 '{}' -> 向量维度: {}, 前5个值: {:?}", 
            text, 
            vectors.last().unwrap().len(),
            &vectors.last().unwrap()[0..5.min(vectors.last().unwrap().len())]
        );
    }
    
    // 计算向量间的相似度
    println!("\n📐 向量相似度矩阵:");
    for (i, text1) in test_texts.iter().enumerate() {
        for (j, text2) in test_texts.iter().enumerate() {
            if i <= j {
                let similarity = cosine_similarity(&vectors[i], &vectors[j]);
                println!("  '{}' vs '{}': {:.6}", text1, text2, similarity);
            }
        }
    }
}

/// 计算余弦相似度
fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() {
        return 0.0;
    }
    
    let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    
    if norm_a == 0.0 || norm_b == 0.0 {
        0.0
    } else {
        dot_product / (norm_a * norm_b)
    }
}

#[tokio::test]
async fn test_vector_diversity() {
    println!("🎯 测试向量多样性...");
    
    let config = EmbeddingConfig::default();
    let embedding_engine = EmbeddingEngine::new(config).await.expect("创建嵌入引擎失败");
    
    // 测试相似和不相似的文本
    let similar_texts = vec![
        "人工智能技术",
        "机器学习算法", 
        "深度学习模型",
    ];
    
    let different_texts = vec![
        "人工智能技术",
        "美味的苹果派",
        "今天天气很好",
    ];
    
    println!("\n📊 相似文本的相似度:");
    for i in 0..similar_texts.len() {
        for j in (i+1)..similar_texts.len() {
            let vec1 = embedding_engine.encode_single(&similar_texts[i]).await.unwrap();
            let vec2 = embedding_engine.encode_single(&similar_texts[j]).await.unwrap();
            let sim = cosine_similarity(&vec1, &vec2);
            println!("  '{}' vs '{}': {:.6}", similar_texts[i], similar_texts[j], sim);
        }
    }
    
    println!("\n📊 不同文本的相似度:");
    for i in 0..different_texts.len() {
        for j in (i+1)..different_texts.len() {
            let vec1 = embedding_engine.encode_single(&different_texts[i]).await.unwrap();
            let vec2 = embedding_engine.encode_single(&different_texts[j]).await.unwrap();
            let sim = cosine_similarity(&vec1, &vec2);
            println!("  '{}' vs '{}': {:.6}", different_texts[i], different_texts[j], sim);
        }
    }
}
