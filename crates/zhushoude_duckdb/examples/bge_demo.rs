//! BGE中文语义模型演示
//! 
//! 这个示例展示了如何使用zhushoude_duckdb crate中的BGE模型进行中文文本向量化

use zhushoude_duckdb::{
    EmbeddingConfig, EmbeddingEngine,
    Result
};

#[tokio::main]
async fn main() -> Result<()> {
    println!("🚀 BGE中文语义模型演示");
    println!("{}", "=".repeat(50));
    
    // 1. 创建嵌入配置
    let config = EmbeddingConfig {
        model_name: "BAAI/bge-small-zh-v1.5".to_string(),
        vector_dimension: 512,
        batch_size: 8,
        max_cache_size: 1000,
        enable_chinese_optimization: true,
        normalize_vectors: true,
    };
    
    println!("📋 配置信息:");
    println!("  - 模型: {}", config.model_name);
    println!("  - 向量维度: {}", config.vector_dimension);
    println!("  - 批处理大小: {}", config.batch_size);
    println!("  - 缓存大小: {}", config.max_cache_size);
    println!("  - 中文优化: {}", config.enable_chinese_optimization);
    println!("  - 向量归一化: {}", config.normalize_vectors);
    println!();
    
    // 2. 创建嵌入引擎
    println!("🔧 初始化嵌入引擎...");
    let engine = EmbeddingEngine::new(config).await?;
    println!("✅ 嵌入引擎初始化完成");
    println!();
    
    // 3. 测试单个文本编码
    println!("📝 单个文本编码测试:");
    let test_texts = vec![
        "人工智能是计算机科学的一个分支",
        "机器学习是人工智能的重要组成部分",
        "深度学习使用神经网络进行模式识别",
        "自然语言处理帮助计算机理解人类语言",
        "今天天气很好，适合出去散步",
    ];
    
    for (i, text) in test_texts.iter().enumerate() {
        println!("  文本 {}: {}", i + 1, text);
        let embedding = engine.encode_single(text).await?;
        println!("    向量维度: {}", embedding.len());
        println!("    前5个值: {:?}", &embedding[..5.min(embedding.len())]);
        
        // 检查向量是否已归一化
        let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        println!("    向量模长: {:.6}", norm);
        println!();
    }
    
    // 4. 测试批量编码
    println!("📦 批量编码测试:");
    let batch_texts: Vec<String> = test_texts.iter().map(|s| s.to_string()).collect();
    let batch_embeddings = engine.encode_batch(&batch_texts).await?;
    
    println!("  批量处理了 {} 个文本", batch_embeddings.len());
    for (i, embedding) in batch_embeddings.iter().enumerate() {
        let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        println!("  文本 {} 向量模长: {:.6}", i + 1, norm);
    }
    println!();
    
    // 5. 测试缓存功能
    println!("💾 缓存功能测试:");
    let cache_test_text = "这是一个缓存测试文本";
    
    // 第一次编码（应该缓存未命中）
    let start = std::time::Instant::now();
    let _embedding1 = engine.encode_single(cache_test_text).await?;
    let duration1 = start.elapsed();
    
    // 第二次编码（应该缓存命中）
    let start = std::time::Instant::now();
    let _embedding2 = engine.encode_single(cache_test_text).await?;
    let duration2 = start.elapsed();
    
    println!("  第一次编码耗时: {:?}", duration1);
    println!("  第二次编码耗时: {:?}", duration2);
    
    let cache_stats = engine.get_cache_stats();
    println!("  缓存统计:");
    println!("    命中次数: {}", cache_stats.hits);
    println!("    未命中次数: {}", cache_stats.misses);
    println!("    命中率: {:.2}%", cache_stats.hit_rate * 100.0);
    println!();
    
    // 6. 测试中文优化
    println!("🇨🇳 中文优化测试:");
    let chinese_texts = vec![
        "機器學習很有趣", // 繁体中文
        "机器学习很有趣", // 简体中文
        "Machine Learning is interesting", // 英文
        "機器學習、深度學習、人工智能", // 混合标点
    ];
    
    for text in chinese_texts {
        println!("  原文: {}", text);
        let embedding = engine.encode_single(text).await?;
        let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        println!("    向量模长: {:.6}", norm);
        println!();
    }
    
    // 7. 计算文本相似度
    println!("🔍 文本相似度计算:");
    let text1 = "人工智能技术发展迅速";
    let text2 = "AI技术进步很快";
    let text3 = "今天天气不错";
    
    let emb1 = engine.encode_single(text1).await?;
    let emb2 = engine.encode_single(text2).await?;
    let emb3 = engine.encode_single(text3).await?;
    
    // 计算余弦相似度
    let similarity_12 = cosine_similarity(&emb1, &emb2);
    let similarity_13 = cosine_similarity(&emb1, &emb3);
    let similarity_23 = cosine_similarity(&emb2, &emb3);
    
    println!("  文本1: {}", text1);
    println!("  文本2: {}", text2);
    println!("  文本3: {}", text3);
    println!();
    println!("  相似度结果:");
    println!("    文本1 vs 文本2: {:.4}", similarity_12);
    println!("    文本1 vs 文本3: {:.4}", similarity_13);
    println!("    文本2 vs 文本3: {:.4}", similarity_23);
    println!();
    
    // 8. 显示模型状态
    println!("📊 模型状态:");
    println!("  模型已加载: {}", engine.is_model_loaded());
    println!("  配置信息: {:?}", engine.get_config());
    
    println!();
    println!("🎉 演示完成！");
    
    Ok(())
}

/// 计算两个向量的余弦相似度
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
