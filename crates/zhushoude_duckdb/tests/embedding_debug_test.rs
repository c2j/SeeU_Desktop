use zhushoude_duckdb::*;
use tokio;

#[tokio::test]
async fn test_embedding_quality() {
    println!("🔍 调试向量化质量...");
    
    let config = EmbeddingConfig::default();
    let embedding_engine = EmbeddingEngine::new(config).await.expect("创建嵌入引擎失败");
    
    // 测试文本
    let test_texts = vec![
        "人工智能技术",
        "量子计算",
        "区块链技术",
        "完全不同的内容",
        "苹果香蕉橘子",
        "今天天气很好",
        "数学物理化学",
        "猫狗鸟鱼",
    ];
    
    println!("\n📊 详细向量分析:");
    let mut vectors = Vec::new();
    
    for text in &test_texts {
        let vector = embedding_engine.encode_single(text).await.unwrap();
        vectors.push(vector.clone());
        
        // 计算向量统计信息
        let mean: f32 = vector.iter().sum::<f32>() / vector.len() as f32;
        let variance: f32 = vector.iter().map(|x| (x - mean).powi(2)).sum::<f32>() / vector.len() as f32;
        let std_dev = variance.sqrt();
        let min_val = vector.iter().fold(f32::INFINITY, |a, &b| a.min(b));
        let max_val = vector.iter().fold(f32::NEG_INFINITY, |a, &b| a.max(b));
        let norm: f32 = vector.iter().map(|x| x * x).sum::<f32>().sqrt();
        
        println!("📝 '{}' -> 维度: {}", text, vector.len());
        println!("   📊 统计: 均值={:.6}, 标准差={:.6}, 范围=[{:.6}, {:.6}], 模长={:.6}", 
            mean, std_dev, min_val, max_val, norm);
        println!("   🔢 前10个值: {:?}", &vector[0..10.min(vector.len())]);
        println!("   🔢 后10个值: {:?}", &vector[vector.len().saturating_sub(10)..]);
        println!();
    }
    
    println!("📐 相似度矩阵分析:");
    for (i, text1) in test_texts.iter().enumerate() {
        for (j, text2) in test_texts.iter().enumerate() {
            if i <= j {
                let similarity = cosine_similarity(&vectors[i], &vectors[j]);
                let expected_similarity = estimate_expected_similarity(text1, text2);
                let diff = (similarity - expected_similarity).abs();
                
                println!("  '{}' vs '{}': {:.6} (期望: {:.6}, 差异: {:.6})", 
                    text1, text2, similarity, expected_similarity, diff);
            }
        }
    }
    
    // 分析向量多样性
    println!("\n🔍 向量多样性分析:");
    let mut all_similarities = Vec::new();
    for i in 0..vectors.len() {
        for j in (i+1)..vectors.len() {
            let sim = cosine_similarity(&vectors[i], &vectors[j]);
            all_similarities.push(sim);
        }
    }
    
    all_similarities.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let min_sim = all_similarities[0];
    let max_sim = all_similarities[all_similarities.len() - 1];
    let median_sim = all_similarities[all_similarities.len() / 2];
    let mean_sim: f32 = all_similarities.iter().sum::<f32>() / all_similarities.len() as f32;
    
    println!("📊 相似度分布: 最小={:.6}, 最大={:.6}, 中位数={:.6}, 平均={:.6}", 
        min_sim, max_sim, median_sim, mean_sim);
    println!("📊 相似度范围: {:.6}", max_sim - min_sim);
    
    // 检查向量质量
    if max_sim - min_sim < 0.01 {
        println!("⚠️  警告: 向量多样性不足，所有向量过于相似");
    } else if max_sim - min_sim > 0.5 {
        println!("✅ 向量多样性良好");
    } else {
        println!("🔶 向量多样性一般，可以改进");
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

/// 估计期望的相似度（基于语义相关性）
fn estimate_expected_similarity(text1: &str, text2: &str) -> f32 {
    if text1 == text2 {
        return 1.0;
    }
    
    // 简单的语义相关性估计
    let tech_words = ["人工智能", "技术", "量子", "计算", "区块链"];
    let nature_words = ["苹果", "香蕉", "橘子", "天气"];
    let science_words = ["数学", "物理", "化学"];
    let animal_words = ["猫", "狗", "鸟", "鱼"];
    
    let is_tech1 = tech_words.iter().any(|&w| text1.contains(w));
    let is_tech2 = tech_words.iter().any(|&w| text2.contains(w));
    
    let is_nature1 = nature_words.iter().any(|&w| text1.contains(w));
    let is_nature2 = nature_words.iter().any(|&w| text2.contains(w));
    
    let is_science1 = science_words.iter().any(|&w| text1.contains(w));
    let is_science2 = science_words.iter().any(|&w| text2.contains(w));
    
    let is_animal1 = animal_words.iter().any(|&w| text1.contains(w));
    let is_animal2 = animal_words.iter().any(|&w| text2.contains(w));
    
    if (is_tech1 && is_tech2) || (is_nature1 && is_nature2) || 
       (is_science1 && is_science2) || (is_animal1 && is_animal2) {
        0.7 // 同类别，应该比较相似
    } else if text1.contains("完全不同") || text2.contains("完全不同") {
        0.1 // 明确表示不同，应该很不相似
    } else {
        0.3 // 不同类别，应该不太相似
    }
}

#[tokio::test]
async fn test_jieba_segmentation() {
    println!("🔍 测试结巴分词...");

    // 直接创建结巴分词器进行测试
    let jieba = jieba_rs::Jieba::new();

    let test_texts = vec![
        "人工智能技术",
        "量子计算",
        "区块链技术",
        "完全不同的内容",
    ];

    for text in &test_texts {
        let words = jieba.cut(text, false);
        println!("📝 '{}' -> 分词结果: {:?}", text, words);
    }
}
