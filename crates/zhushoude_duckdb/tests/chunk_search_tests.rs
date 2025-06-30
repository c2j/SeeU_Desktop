use zhushoude_duckdb::*;
use tokio;

#[tokio::test]
async fn test_text_chunking() {
    println!("🔄 测试文本分块功能...");
    
    let config = ChunkingConfig {
        chunk_size: 150, // 减小分块大小以产生多个分块
        overlap_size: 20,
        min_chunk_size: 30,
        preserve_semantic_boundaries: true,
        strategy: ChunkingStrategy::Semantic,
    };
    
    let chunker = TextChunker::new(config);
    
    let test_text = "
        人工智能是计算机科学的一个分支，它企图了解智能的实质，并生产出一种新的能以人类智能相似的方式做出反应的智能机器。
        
        机器学习是人工智能的一个重要分支。它是一种通过算法使机器能够自动学习和改进的技术。深度学习是机器学习的一个子集，它使用神经网络来模拟人脑的工作方式。
        
        自然语言处理（NLP）是人工智能的另一个重要领域。它涉及计算机和人类语言之间的交互，使计算机能够理解、解释和生成人类语言。
        
        计算机视觉是使计算机能够从数字图像或视频中获取高层次理解的领域。它包括图像识别、物体检测、人脸识别等技术。
    ";
    
    let chunks = chunker.chunk_text("test_doc", test_text).expect("分块失败");
    
    println!("📄 文本分块结果:");
    println!("  总分块数: {}", chunks.len());
    
    for (i, chunk) in chunks.iter().enumerate() {
        println!("  分块 {}: {} 字符 (质量分数: {:.2})", 
                i, chunk.metadata.char_count, chunk.metadata.quality_score);
        println!("    内容预览: {}...", 
                chunk.content.chars().take(50).collect::<String>());
    }
    
    let stats = chunker.get_chunking_stats(&chunks);
    println!("📊 分块统计:");
    println!("  平均分块大小: {} 字符", stats.avg_chunk_size);
    println!("  最小分块大小: {} 字符", stats.min_chunk_size);
    println!("  最大分块大小: {} 字符", stats.max_chunk_size);
    println!("  平均质量分数: {:.2}", stats.avg_quality_score);
    
    assert!(chunks.len() >= 2, "应该产生多个分块");
    assert!(stats.avg_chunk_size > 0, "平均分块大小应该大于0");
}

#[tokio::test]
async fn test_chunk_vector_indexing() {
    println!("🔄 测试分块向量索引...");
    
    let config = ZhushoudeConfig {
        database_path: ":memory:".to_string(),
        ..Default::default()
    };
    
    let db_manager = DatabaseManager::new(config).await.expect("数据库管理器创建失败");
    let db_manager = std::sync::Arc::new(db_manager);
    
    let vector_config = ChunkVectorConfig {
        chunking_config: ChunkingConfig {
            chunk_size: 256,
            overlap_size: 32,
            min_chunk_size: 50,
            preserve_semantic_boundaries: true,
            strategy: ChunkingStrategy::Semantic,
        },
        vector_dimension: 128,
        index_type: ChunkIndexType::Flat,
        batch_size: 10,
        incremental_indexing: true,
        update_threshold: 100,
    };
    
    let index_manager = ChunkVectorIndexManager::new(db_manager.clone(), vector_config);
    
    // 初始化表
    index_manager.initialize_tables().await.expect("表初始化失败");
    
    // 测试文档
    let test_documents = vec![
        ("doc1", "人工智能技术正在快速发展。机器学习和深度学习是其核心技术。自然语言处理让计算机能够理解人类语言。"),
        ("doc2", "量子计算是一种全新的计算范式。它利用量子力学原理进行信息处理。量子比特可以同时处于多个状态。"),
        ("doc3", "区块链技术提供了去中心化的解决方案。它通过密码学确保数据的安全性和不可篡改性。智能合约自动执行协议条款。"),
    ];
    
    // 索引文档
    for (doc_id, content) in &test_documents {
        let indexes = index_manager.index_document(doc_id, content, "test_model").await
            .expect("文档索引失败");
        println!("📄 文档 {} 创建了 {} 个向量索引", doc_id, indexes.len());
    }
    
    // 获取统计信息
    let stats = index_manager.get_index_stats().await.expect("获取统计信息失败");
    println!("📊 索引统计:");
    println!("  总文档数: {}", stats.total_documents);
    println!("  总分块数: {}", stats.total_chunks);
    println!("  平均分块大小: {:.1} 字符", stats.avg_chunk_size);
    println!("  平均质量分数: {:.2}", stats.avg_quality_score);
    
    assert_eq!(stats.total_documents, 3, "应该有3个文档");
    assert!(stats.total_chunks > 3, "应该有多个分块");
}

#[tokio::test]
async fn test_chunk_semantic_search() {
    println!("🔄 测试分块语义搜索...");
    
    let config = ZhushoudeConfig {
        database_path: ":memory:".to_string(),
        ..Default::default()
    };
    
    let db_manager = DatabaseManager::new(config).await.expect("数据库管理器创建失败");
    let db_manager = std::sync::Arc::new(db_manager);
    
    let search_config = ChunkSemanticSearchConfig {
        vector_config: ChunkVectorConfig {
            chunking_config: ChunkingConfig {
                chunk_size: 200,
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
        },
        default_limit: 5,
        max_limit: 20,
        similarity_threshold: 0.1,
        enable_aggregation: true,
        aggregation_window: 2,
        enable_reranking: true,
    };
    
    let search_engine = ChunkSemanticSearchEngine::new(db_manager, search_config);
    
    // 初始化搜索引擎
    search_engine.initialize().await.expect("搜索引擎初始化失败");
    
    // 索引测试文档
    let documents = vec![
        ("ai_doc".to_string(), "人工智能是模拟人类智能的技术。机器学习让计算机能够从数据中学习。深度学习使用神经网络处理复杂问题。自然语言处理帮助计算机理解人类语言。".to_string()),
        ("quantum_doc".to_string(), "量子计算利用量子力学原理。量子比特可以处于叠加态。量子纠缠实现远距离关联。量子算法能够解决某些经典计算难题。".to_string()),
        ("blockchain_doc".to_string(), "区块链是分布式账本技术。每个区块包含交易记录。密码学哈希确保数据完整性。共识机制维护网络一致性。".to_string()),
    ];
    
    search_engine.batch_index_documents(&documents, "test_model").await
        .expect("批量索引失败");
    
    // 执行搜索测试
    let test_queries = vec![
        "机器学习算法",
        "量子计算原理", 
        "区块链安全",
        "人工智能应用",
    ];
    
    for query in test_queries {
        println!("\n🔍 搜索查询: '{}'", query);
        
        let search_query = ChunkSearchQuery {
            query: query.to_string(),
            limit: Some(3),
            document_ids: None,
            languages: Some(vec!["zh".to_string()]),
            content_types: None,
            min_quality_score: Some(0.1),
            model_name: Some("test_model".to_string()),
            enable_context_expansion: Some(true),
        };
        
        let (results, stats) = search_engine.search(&search_query).await
            .expect("搜索失败");
        
        println!("📊 搜索统计:");
        println!("  查询时间: {} ms", stats.query_time_ms);
        println!("  总结果数: {}", stats.total_results);
        println!("  聚合结果数: {}", stats.aggregated_results);
        println!("  平均相似度: {:.3}", stats.avg_similarity_score);
        
        println!("🎯 搜索结果:");
        for (i, result) in results.iter().enumerate() {
            println!("  结果 {}: 文档 {} (聚合分数: {:.3})", 
                    i + 1, result.primary_chunk.document_id, result.aggregated_score);
            println!("    主要内容: {}...", 
                    result.primary_chunk.content.chars().take(50).collect::<String>());
            if !result.context_chunks.is_empty() {
                println!("    上下文分块数: {}", result.context_chunks.len());
            }
        }
        
        assert!(!results.is_empty(), "搜索应该返回结果");
    }
    
    // 测试相似文档查找
    println!("\n🔍 测试相似文档查找...");
    let similar_docs = search_engine.find_similar_documents("ai_doc", "test_model", 2).await
        .expect("相似文档查找失败");
    
    println!("📄 与 'ai_doc' 相似的文档: {:?}", similar_docs);
    
    // 获取搜索引擎统计信息
    let engine_stats = search_engine.get_stats().await.expect("获取引擎统计失败");
    println!("\n📊 搜索引擎统计:");
    println!("  总文档数: {}", engine_stats.total_documents);
    println!("  总分块数: {}", engine_stats.total_chunks);
    println!("  向量维度: {}", engine_stats.vector_dimension);
    println!("  分块策略: {}", engine_stats.chunking_strategy);
    
    assert_eq!(engine_stats.total_documents, 3, "应该有3个文档");
    assert!(engine_stats.total_chunks >= 3, "应该有多个分块");
}

#[tokio::test]
async fn test_chunking_strategies() {
    println!("🔄 测试不同分块策略...");
    
    let test_text = "
        第一段：人工智能技术发展迅速。机器学习是核心技术之一。
        
        第二段：深度学习使用神经网络。它能处理复杂的模式识别任务。
        
        第三段：自然语言处理让计算机理解人类语言。这是AI的重要应用领域。
    ";
    
    let strategies = vec![
        ChunkingStrategy::FixedSize,
        ChunkingStrategy::Semantic,
        ChunkingStrategy::Paragraph,
        ChunkingStrategy::Sentence,
        ChunkingStrategy::Hybrid,
    ];
    
    for strategy in strategies {
        println!("\n📝 测试策略: {:?}", strategy);
        
        let config = ChunkingConfig {
            chunk_size: 100,
            overlap_size: 20,
            min_chunk_size: 20,
            preserve_semantic_boundaries: true,
            strategy,
        };
        
        let chunker = TextChunker::new(config);
        let chunks = chunker.chunk_text("test", test_text).expect("分块失败");
        
        println!("  分块数量: {}", chunks.len());
        for (i, chunk) in chunks.iter().enumerate() {
            println!("    分块 {}: {} 字符", i, chunk.metadata.char_count);
        }
        
        let stats = chunker.get_chunking_stats(&chunks);
        println!("  平均大小: {} 字符", stats.avg_chunk_size);
        println!("  质量分数: {:.2}", stats.avg_quality_score);
        
        assert!(!chunks.is_empty(), "应该产生分块");
    }
}
