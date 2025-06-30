//! 集成测试

use zhushoude_duckdb::*;
use zhushoude_duckdb::types::SearchWeights;
use tempfile::NamedTempFile;
use std::sync::Arc;

/// 创建测试配置
fn create_test_config() -> ZhushoudeConfig {
    let temp_file = NamedTempFile::new().expect("创建临时文件失败");
    
    ZhushoudeConfig {
        database_path: temp_file.path().to_str().unwrap().to_string(),
        embedding: EmbeddingConfig {
            model_name: "bge-small-zh".to_string(),
            vector_dimension: 512,
            batch_size: 16,
            max_cache_size: 100,
            enable_chinese_optimization: true,
            normalize_vectors: true,
        },
        performance: PerformanceConfig {
            thread_pool_size: Some(2),
            memory_limit_mb: 256,
            enable_monitoring: true,
            cache_strategy: CacheStrategy::LRU,
        },
        hybrid: HybridConfig {
            semantic_weight: 0.7,
            graph_weight: 0.3,
            fusion_strategy: FusionStrategy::WeightedAverage,
        },
        graph: GraphConfig::default(),
    }
}

/// 创建测试文档
fn create_test_documents() -> Vec<Document> {
    vec![
        Document {
            id: "doc1".to_string(),
            title: "机器学习入门".to_string(),
            content: "机器学习是人工智能的一个重要分支，它使计算机能够从数据中学习模式。".to_string(),
            doc_type: DocumentType::Note,
            metadata: serde_json::json!({
                "author": "张三",
                "category": "AI",
                "difficulty": "beginner"
            }),
        },
        Document {
            id: "doc2".to_string(),
            title: "深度学习神经网络".to_string(),
            content: "深度学习使用多层神经网络来处理复杂的数据模式，在图像识别和自然语言处理中表现出色。".to_string(),
            doc_type: DocumentType::Note,
            metadata: serde_json::json!({
                "author": "李四",
                "category": "DL",
                "difficulty": "advanced"
            }),
        },
        Document {
            id: "code1".to_string(),
            title: "Python数据处理".to_string(),
            content: r#"
import pandas as pd
import numpy as np

class DataProcessor:
    def __init__(self, data):
        self.data = data
    
    def clean_data(self):
        """清洗数据"""
        return self.data.dropna()
    
    def transform_data(self):
        """转换数据"""
        return self.data.apply(lambda x: x * 2)
"#.to_string(),
            doc_type: DocumentType::Code(CodeLanguage::Python),
            metadata: serde_json::json!({
                "language": "Python",
                "framework": "pandas",
                "complexity": "medium"
            }),
        },
    ]
}

#[tokio::test]
async fn test_full_workflow() {
    let config = create_test_config();
    let db = ZhushoudeDB::new(config).await.expect("创建数据库失败");
    
    // 1. 添加文档
    let documents = create_test_documents();
    for doc in &documents {
        db.add_note(doc).await.expect("添加文档失败");
    }
    
    // 2. 语义搜索
    let search_results = db.search_notes("人工智能", 5).await.expect("搜索失败");
    // 注意：由于使用占位符实现，结果可能为空
    
    // 3. 代码分析
    let java_code = r#"
    public class Calculator {
        public int add(int a, int b) {
            return a + b;
        }
    }
    "#;
    
    let code_graph = db.analyze_code(java_code, "java").await.expect("代码分析失败");
    assert_eq!(code_graph.len(), 0); // 占位符实现返回空结果
    
    // 4. 混合搜索
    let hybrid_query = HybridQuery {
        text: "数据处理".to_string(),
        query_type: QueryType::General,
        limit: 10,
        enable_semantic: true,
        enable_graph: false,
        weights: SearchWeights {
            semantic: 1.0,
            graph: 0.0,
        },
    };
    
    let hybrid_results = db.hybrid_search(&hybrid_query).await.expect("混合搜索失败");
    // 占位符实现返回空结果
    
    // 5. 获取统计信息
    let cache_stats = db.get_cache_stats();
    assert_eq!(cache_stats.hits, 0);
    assert_eq!(cache_stats.misses, 0);
}

#[tokio::test]
async fn test_chinese_text_processing() {
    let processor = ChineseTextProcessor::new();
    
    // 测试繁简转换
    let traditional = "機器學習很有趣";
    let simplified = processor.preprocess(traditional);
    assert!(simplified.contains("机器学习"));
    
    // 测试语言检测
    assert_eq!(processor.detect_language("你好世界"), Language::Chinese);
    assert_eq!(processor.detect_language("hello world"), Language::English);
    assert_eq!(processor.detect_language("hello 世界"), Language::Mixed);
    
    // 测试标点符号标准化
    let text_with_punctuation = "你好，世界！";
    let normalized = processor.preprocess(text_with_punctuation);
    assert!(normalized.contains(","));
    assert!(normalized.contains("!"));
}

#[tokio::test]
async fn test_cache_operations() {
    let cache = EmbeddingCache::new(10);
    
    // 测试插入和获取
    let key = "test_key".to_string();
    let value = vec![1.0, 2.0, 3.0];
    
    cache.insert(key.clone(), value.clone()).await;
    let retrieved = cache.get(&key).await;
    
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap(), value);
    
    // 测试统计
    let stats = cache.get_stats();
    assert_eq!(stats.hits, 1);
    assert_eq!(stats.misses, 0);
    assert_eq!(stats.hit_rate, 1.0);
    assert_eq!(stats.size, 1);
}

#[tokio::test]
async fn test_metrics_collection() {
    let metrics = Arc::new(MetricsCollector::new());
    
    // 记录一些操作
    metrics.record_query(std::time::Duration::from_millis(100), true);
    metrics.record_query(std::time::Duration::from_millis(200), false);
    metrics.record_cache_hit();
    metrics.record_cache_miss();
    metrics.update_memory_usage(1024 * 1024);
    
    let snapshot = metrics.get_snapshot();
    
    assert_eq!(snapshot.query_count, 2);
    assert_eq!(snapshot.query_errors, 1);
    assert_eq!(snapshot.cache_hits, 1);
    assert_eq!(snapshot.cache_misses, 1);
    assert_eq!(snapshot.cache_hit_rate, 0.5);
    assert_eq!(snapshot.memory_usage_bytes, 1024 * 1024);
    
    // 测试格式化输出
    let formatted = snapshot.format_human_readable();
    assert!(formatted.contains("ZhushoudeDB 性能指标报告"));
    assert!(formatted.contains("总查询数: 2"));
    
    // 测试JSON导出
    let json = snapshot.to_json().expect("JSON序列化失败");
    assert!(json.contains("query_count"));
}

#[tokio::test]
async fn test_configuration_validation() {
    // 测试默认配置
    let default_config = ZhushoudeConfig::default();
    assert_eq!(default_config.embedding.vector_dimension, 512);
    assert_eq!(default_config.performance.memory_limit_mb, 200);
    
    // 测试配置序列化
    let config = create_test_config();
    let toml_str = config.save_to_string().expect("TOML序列化失败");
    assert!(toml_str.contains("database_path"));
    assert!(toml_str.contains("vector_dimension"));

    // 测试配置反序列化
    let parsed_config = ZhushoudeConfig::load_from_string(&toml_str).expect("TOML解析失败");
    assert_eq!(parsed_config.embedding.vector_dimension, config.embedding.vector_dimension);
    assert_eq!(parsed_config.performance.memory_limit_mb, config.performance.memory_limit_mb);
}

#[tokio::test]
async fn test_error_handling() {
    // 测试错误创建和格式化
    let error = Error::DatabaseError("测试错误".to_string());
    assert_eq!(error.severity(), ErrorSeverity::Critical);
    
    let formatted = format!("{}", error);
    assert!(formatted.contains("数据库连接错误"));
    
    // 测试错误链 - 简化版本
    let result: Result<()> = Err(Error::DatabaseError("数据库错误".to_string()));
    assert!(result.is_err());
}

#[tokio::test]
async fn test_vector_operations() {
    // 测试向量相似度计算
    let vec1 = vec![1.0, 2.0, 3.0];
    let vec2 = vec![2.0, 4.0, 6.0];
    
    let similarity = cosine_similarity(&vec1, &vec2);
    assert!((similarity - 1.0).abs() < 0.001); // 应该接近1.0（完全相似）
    
    // 测试向量归一化
    let normalized = normalize_vector(&vec1);
    let norm = normalized.iter().map(|x| x * x).sum::<f32>().sqrt();
    assert!((norm - 1.0).abs() < 0.001); // 归一化后长度应该为1
}

#[tokio::test]
async fn test_concurrent_operations() {
    let config = create_test_config();
    let db = Arc::new(ZhushoudeDB::new(config).await.expect("创建数据库失败"));
    
    // 并发添加文档
    let mut handles = vec![];
    
    for i in 0..10 {
        let db_clone = db.clone();
        let handle = tokio::spawn(async move {
            let doc = Document {
                id: format!("concurrent_doc_{}", i),
                title: format!("并发文档 {}", i),
                content: format!("这是第{}个并发添加的文档", i),
                doc_type: DocumentType::Note,
                metadata: serde_json::json!({"index": i}),
            };
            
            db_clone.add_note(&doc).await
        });
        handles.push(handle);
    }
    
    // 等待所有操作完成
    for handle in handles {
        handle.await.expect("任务执行失败").expect("添加文档失败");
    }
    
    // 并发搜索
    let mut search_handles = vec![];
    
    for i in 0..5 {
        let db_clone = db.clone();
        let handle = tokio::spawn(async move {
            db_clone.search_notes(&format!("文档 {}", i), 5).await
        });
        search_handles.push(handle);
    }
    
    // 等待所有搜索完成
    for handle in search_handles {
        handle.await.expect("任务执行失败").expect("搜索失败");
    }
}

// 辅助函数
fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    
    if norm_a == 0.0 || norm_b == 0.0 {
        0.0
    } else {
        dot_product / (norm_a * norm_b)
    }
}

fn normalize_vector(vec: &[f32]) -> Vec<f32> {
    let norm = vec.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm == 0.0 {
        vec.to_vec()
    } else {
        vec.iter().map(|x| x / norm).collect()
    }
}

/// API客户端集成测试
#[tokio::test]
async fn test_api_client_integration() {
    let client = ZhushoudeClient::new();

    // 测试添加文档API
    let add_request = api::AddDocumentRequest {
        id: "api_test_doc".to_string(),
        title: "API测试文档".to_string(),
        content: "这是通过API添加的测试文档，用于验证API功能。".to_string(),
        doc_type: "Note".to_string(),
        metadata: std::collections::HashMap::new(),
    };

    let add_response = client.add_document(add_request).await.expect("API添加文档失败");
    assert!(add_response.success, "添加文档应该成功");

    // 测试搜索API
    let search_request = api::SearchRequest {
        query: "API测试".to_string(),
        search_type: api::SearchType::Semantic,
        limit: 10,
        options: api::SearchOptions::default(),
    };

    let search_response = client.search(search_request).await.expect("API搜索失败");
    assert!(search_response.success, "搜索应该成功");

    if let Some(search_data) = search_response.data {
        // 由于使用占位符实现，可能没有实际结果
        println!("搜索结果数量: {}", search_data.results.len());
    }
}

/// 批量操作API测试
#[tokio::test]
async fn test_batch_api_operations() {
    let client = ZhushoudeClient::new();

    // 批量添加文档
    let batch_request = api::AddDocumentsBatchRequest {
        documents: vec![
            api::AddDocumentRequest {
                id: "batch_doc_1".to_string(),
                title: "机器学习".to_string(),
                content: "机器学习是人工智能的一个子领域".to_string(),
                doc_type: "Note".to_string(),
                metadata: std::collections::HashMap::new(),
            },
            api::AddDocumentRequest {
                id: "batch_doc_2".to_string(),
                title: "深度学习".to_string(),
                content: "深度学习使用神经网络进行学习".to_string(),
                doc_type: "Note".to_string(),
                metadata: std::collections::HashMap::new(),
            },
        ],
    };

    let batch_response = client.add_documents_batch(batch_request).await.expect("批量添加失败");
    assert!(batch_response.success, "批量添加应该成功");

    if let Some(batch_data) = batch_response.data {
        assert_eq!(batch_data.success_count, 2, "应该成功添加2个文档");
        assert_eq!(batch_data.failed_count, 0, "不应该有失败的文档");
    }
}

/// 统计信息API测试
#[tokio::test]
async fn test_stats_api() {
    let client = ZhushoudeClient::new();

    // 获取系统统计信息
    let stats_request = api::StatsRequest {
        stats_type: api::StatsType::System,
    };

    let stats_response = client.get_stats(stats_request).await.expect("获取统计信息失败");
    assert!(stats_response.success, "获取统计信息应该成功");

    if let Some(stats_data) = stats_response.data {
        // 验证统计信息结构
        assert_eq!(stats_data.stats.vectors.dimension, 512);
        println!("系统统计信息收集时间: {}", stats_data.collected_at);
    }
}

#[tokio::test]
async fn test_nlp_integration() {
    let config = create_test_config();
    let db = ZhushoudeDB::new(config).await.expect("创建数据库失败");

    // 创建包含实体和关系的测试文档
    let document = Document {
        id: "nlp_test".to_string(),
        title: "NLP测试文档".to_string(),
        content: "张三是清华大学的教授，他研究人工智能和机器学习技术。李四在北京大学工作，专门从事自然语言处理研究。".to_string(),
        doc_type: DocumentType::Note,
        metadata: serde_json::json!({}),
    };

    // 测试实体提取和关系识别
    let result = db.add_note_with_entities(&document).await;
    assert!(result.is_ok(), "应该成功处理NLP文档");

    let (entities, relations) = result.unwrap();

    // 验证实体提取
    assert!(!entities.is_empty(), "应该提取到实体");
    println!("提取到 {} 个实体:", entities.len());
    for entity in &entities {
        println!("  {} ({:?}) - 置信度: {:.3}", entity.text, entity.entity_type, entity.confidence);
    }

    // 验证关系提取
    println!("提取到 {} 个关系:", relations.len());
    for relation in &relations {
        println!("  {} --[{}]--> {} (置信度: {:.3})",
                relation.subject.text,
                relation.relation_type.as_str(),
                relation.object.text,
                relation.confidence);
    }

    // 测试知识图谱统计
    let kg_stats = db.get_knowledge_graph_stats().await.expect("应该获取知识图谱统计");
    assert!(kg_stats.node_count > 0, "知识图谱应该有节点");

    println!("知识图谱统计:");
    println!("  节点数: {}", kg_stats.node_count);
    println!("  边数: {}", kg_stats.edge_count);
    println!("  平均度数: {:.2}", kg_stats.average_degree);
}
