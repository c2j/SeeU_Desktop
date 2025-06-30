//! 搜索性能基准测试

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use zhushoude_duckdb::*;
use zhushoude_duckdb::types::SearchWeights;
use tokio::runtime::Runtime;

fn create_test_documents(count: usize) -> Vec<Document> {
    let mut documents = Vec::with_capacity(count);
    
    for i in 0..count {
        documents.push(Document {
            id: format!("doc_{}", i),
            title: format!("测试文档 {}", i),
            content: format!(
                "这是第{}个测试文档。它包含了关于机器学习、深度学习、人工智能的内容。\
                文档中还包含了数据处理、算法优化、性能调优等技术话题。\
                这些内容用于测试语义搜索的性能和准确性。", 
                i
            ),
            doc_type: DocumentType::Note,
            metadata: serde_json::json!({
                "index": i,
                "category": if i % 3 == 0 { "AI" } else if i % 3 == 1 { "ML" } else { "DL" },
                "priority": i % 5
            }),
        });
    }
    
    documents
}

async fn setup_database(doc_count: usize) -> ZhushoudeDB {
    let config = ZhushoudeConfig {
        database_path: format!("bench_test_{}.db", doc_count),
        embedding: EmbeddingConfig {
            model_name: "bge-small-zh".to_string(),
            vector_dimension: 512,
            batch_size: 32,
            max_cache_size: 1000,
            enable_chinese_optimization: true,
            normalize_vectors: true,
        },
        performance: PerformanceConfig {
            thread_pool_size: Some(4),
            memory_limit_mb: 1024,
            enable_monitoring: true,
            cache_strategy: CacheStrategy::LRU,
        },
        ..Default::default()
    };
    
    let db = ZhushoudeDB::new(config).await.expect("创建数据库失败");
    
    // 添加测试文档
    let documents = create_test_documents(doc_count);
    for doc in &documents {
        db.add_note(doc).await.expect("添加文档失败");
    }
    
    db
}

fn bench_document_insertion(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("document_insertion");
    
    for doc_count in [100, 500, 1000, 2000].iter() {
        group.bench_with_input(
            BenchmarkId::new("add_documents", doc_count),
            doc_count,
            |b, &doc_count| {
                b.to_async(&rt).iter(|| async {
                    let db = setup_database(0).await;
                    let documents = create_test_documents(doc_count);
                    
                    for doc in &documents {
                        black_box(db.add_note(doc).await.expect("添加文档失败"));
                    }
                });
            },
        );
    }
    
    group.finish();
}

fn bench_semantic_search(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("semantic_search");
    
    for doc_count in [100, 500, 1000, 2000].iter() {
        let db = rt.block_on(setup_database(*doc_count));
        
        group.bench_with_input(
            BenchmarkId::new("search_notes", doc_count),
            doc_count,
            |b, _| {
                b.to_async(&rt).iter(|| async {
                    let queries = [
                        "机器学习算法",
                        "深度学习神经网络",
                        "人工智能应用",
                        "数据处理技术",
                        "性能优化方法",
                    ];
                    
                    for query in &queries {
                        black_box(
                            db.search_notes(query, 10)
                                .await
                                .expect("搜索失败")
                        );
                    }
                });
            },
        );
    }
    
    group.finish();
}

fn bench_hybrid_search(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("hybrid_search");
    
    for doc_count in [100, 500, 1000].iter() {
        let db = rt.block_on(setup_database(*doc_count));
        
        group.bench_with_input(
            BenchmarkId::new("hybrid_search", doc_count),
            doc_count,
            |b, _| {
                b.to_async(&rt).iter(|| async {
                    let query = HybridQuery {
                        text: "机器学习数据处理".to_string(),
                        query_type: QueryType::General,
                        limit: 20,
                        enable_semantic: true,
                        enable_graph: true,
                        weights: SearchWeights {
                            semantic: 0.7,
                            graph: 0.3,
                        },
                    };
                    
                    black_box(
                        db.hybrid_search(&query)
                            .await
                            .expect("混合搜索失败")
                    );
                });
            },
        );
    }
    
    group.finish();
}

fn bench_chinese_text_processing(c: &mut Criterion) {
    let processor = ChineseTextProcessor::new();
    
    let test_texts = vec![
        "機器學習是人工智能的重要分支，它使計算機能夠學習和改進。".to_string(),
        "深度學習使用多層神經網絡來模擬人腦的工作方式，在圖像識別、自然語言處理等領域取得了突破性進展。".to_string(),
        "數據處理是機器學習流程中的關鍵步驟，包括數據清洗、特徵工程、數據轉換等操作。".to_string(),
    ];
    
    c.bench_function("chinese_text_processing", |b| {
        b.iter(|| {
            for text in &test_texts {
                black_box(processor.preprocess(text));
                black_box(processor.detect_language(text));
            }
        });
    });
}

fn bench_cache_operations(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("cache_operations");
    
    for cache_size in [100, 500, 1000, 2000].iter() {
        group.bench_with_input(
            BenchmarkId::new("cache_ops", cache_size),
            cache_size,
            |b, &cache_size| {
                b.to_async(&rt).iter(|| async {
                    let cache = EmbeddingCache::new(cache_size);
                    
                    // 插入操作
                    for i in 0..cache_size {
                        let key = format!("key_{}", i);
                        let value = vec![i as f32; 512];
                        black_box(cache.insert(key, value).await);
                    }
                    
                    // 查询操作
                    for i in 0..cache_size {
                        let key = format!("key_{}", i);
                        black_box(cache.get(&key).await);
                    }
                    
                    // 统计操作
                    black_box(cache.get_stats());
                });
            },
        );
    }
    
    group.finish();
}

fn bench_vector_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("vector_operations");
    
    for vector_dim in [128, 256, 512, 1024].iter() {
        group.bench_with_input(
            BenchmarkId::new("cosine_similarity", vector_dim),
            vector_dim,
            |b, &vector_dim| {
                let vec1: Vec<f32> = (0..vector_dim).map(|i| i as f32).collect();
                let vec2: Vec<f32> = (0..vector_dim).map(|i| (i * 2) as f32).collect();
                
                b.iter(|| {
                    black_box(cosine_similarity(&vec1, &vec2));
                });
            },
        );
    }
    
    group.finish();
}

// 辅助函数：计算余弦相似度
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

criterion_group!(
    benches,
    bench_document_insertion,
    bench_semantic_search,
    bench_hybrid_search,
    bench_chinese_text_processing,
    bench_cache_operations,
    bench_vector_operations
);

criterion_main!(benches);
