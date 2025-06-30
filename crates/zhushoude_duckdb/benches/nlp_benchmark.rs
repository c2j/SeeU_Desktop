//! NLP功能性能基准测试

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use zhushoude_duckdb::nlp::{ChineseNER, ChineseRelationExtractor, KnowledgeGraphBuilder};
use zhushoude_duckdb::{ZhushoudeConfig, DatabaseManager};
use std::sync::Arc;
use tokio::runtime::Runtime;

/// 创建测试文本数据
fn create_test_texts() -> Vec<String> {
    vec![
        "张三是清华大学的教授，他研究人工智能技术。".to_string(),
        "李四在北京大学工作，专门从事机器学习和深度学习研究。".to_string(),
        "王五是阿里巴巴公司的工程师，他使用Python开发大数据系统。".to_string(),
        "赵六在2024年1月加入了腾讯公司，负责自然语言处理项目。".to_string(),
        "孙七毕业于斯坦福大学，现在在谷歌研究院从事计算机视觉工作。".to_string(),
        "周八创建了一家人工智能初创公司，专注于智能对话系统的开发。".to_string(),
        "吴九是微软亚洲研究院的研究员，他的研究方向是强化学习。".to_string(),
        "郑十在华为技术有限公司担任首席科学家，领导5G通信技术研发。".to_string(),
        "钱一是字节跳动的算法工程师，负责推荐系统的优化和改进。".to_string(),
        "陈二在上海交通大学攻读博士学位，研究方向是区块链技术。".to_string(),
    ]
}

/// 创建长文本用于压力测试
fn create_long_text(repeat_count: usize) -> String {
    let base_text = "张三在清华大学研究人工智能技术，他与李四合作开发了基于深度学习的自然语言处理系统。该系统使用Python和TensorFlow框架实现，能够处理中文文本的语义分析和实体识别任务。";
    base_text.repeat(repeat_count)
}

/// NER性能基准测试
fn bench_ner_performance(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let ner = rt.block_on(async {
        ChineseNER::new().expect("Failed to create NER")
    });
    
    let test_texts = create_test_texts();
    
    // 单个文本的NER性能
    c.bench_function("ner_single_text", |b| {
        b.iter(|| {
            let text = black_box(&test_texts[0]);
            ner.extract_entities(text).expect("Failed to extract entities")
        })
    });
    
    // 批量文本的NER性能
    c.bench_function("ner_batch_texts", |b| {
        b.iter(|| {
            for text in black_box(&test_texts) {
                ner.extract_entities(text).expect("Failed to extract entities");
            }
        })
    });
    
    // 不同长度文本的NER性能
    let mut group = c.benchmark_group("ner_text_length");
    for length in [1, 5, 10, 20, 50].iter() {
        let long_text = create_long_text(*length);
        group.bench_with_input(BenchmarkId::new("repeat_count", length), &long_text, |b, text| {
            b.iter(|| {
                ner.extract_entities(black_box(text)).expect("Failed to extract entities")
            })
        });
    }
    group.finish();
}

/// 关系抽取性能基准测试
fn bench_relation_extraction_performance(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let (ner, extractor) = rt.block_on(async {
        let ner = ChineseNER::new().expect("Failed to create NER");
        let extractor = ChineseRelationExtractor::new().expect("Failed to create relation extractor");
        (ner, extractor)
    });
    
    let test_texts = create_test_texts();
    
    // 预先提取实体以便测试关系抽取性能
    let entities_list: Vec<_> = test_texts.iter()
        .map(|text| ner.extract_entities(text).expect("Failed to extract entities"))
        .collect();
    
    // 单个文本的关系抽取性能
    c.bench_function("relation_extraction_single", |b| {
        b.iter(|| {
            let text = black_box(&test_texts[0]);
            let entities = black_box(&entities_list[0]);
            extractor.extract_relations(text, entities).expect("Failed to extract relations")
        })
    });
    
    // 批量关系抽取性能
    c.bench_function("relation_extraction_batch", |b| {
        b.iter(|| {
            for (text, entities) in black_box(&test_texts).iter().zip(black_box(&entities_list).iter()) {
                extractor.extract_relations(text, entities).expect("Failed to extract relations");
            }
        })
    });
}

/// 知识图谱构建性能基准测试
fn bench_knowledge_graph_building(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    c.bench_function("knowledge_graph_building", |b| {
        b.to_async(&rt).iter(|| async {
            let config = ZhushoudeConfig::default();
            let db_manager = Arc::new(DatabaseManager::new(config).await.expect("Failed to create database"));
            let mut kg_builder = KnowledgeGraphBuilder::new(db_manager);
            
            let ner = ChineseNER::new().expect("Failed to create NER");
            let extractor = ChineseRelationExtractor::new().expect("Failed to create relation extractor");
            
            let test_texts = create_test_texts();
            
            for (i, text) in test_texts.iter().enumerate() {
                let entities = ner.extract_entities(text).expect("Failed to extract entities");
                let relations = extractor.extract_relations(text, &entities)
                    .expect("Failed to extract relations");
                
                kg_builder.build_from_entities_and_relations(
                    black_box(&entities), 
                    black_box(&relations), 
                    &format!("doc_{}", i)
                ).await.expect("Failed to build knowledge graph");
            }
            
            kg_builder.get_knowledge_graph().stats.clone()
        })
    });
}

/// 端到端NLP流水线性能基准测试
fn bench_end_to_end_pipeline(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("end_to_end_pipeline");
    
    // 测试不同数量文档的处理性能
    for doc_count in [1, 5, 10, 20].iter() {
        group.bench_with_input(BenchmarkId::new("document_count", doc_count), doc_count, |b, &count| {
            b.to_async(&rt).iter(|| async {
                let config = ZhushoudeConfig::default();
                let db_manager = Arc::new(DatabaseManager::new(config).await.expect("Failed to create database"));
                let mut kg_builder = KnowledgeGraphBuilder::new(db_manager);
                
                let ner = ChineseNER::new().expect("Failed to create NER");
                let extractor = ChineseRelationExtractor::new().expect("Failed to create relation extractor");
                
                let test_texts = create_test_texts();
                
                for i in 0..count {
                    let text = &test_texts[i % test_texts.len()];
                    
                    // 完整的NLP流水线
                    let entities = ner.extract_entities(text).expect("Failed to extract entities");
                    let relations = extractor.extract_relations(text, &entities)
                        .expect("Failed to extract relations");
                    
                    kg_builder.build_from_entities_and_relations(
                        black_box(&entities), 
                        black_box(&relations), 
                        &format!("doc_{}", i)
                    ).await.expect("Failed to build knowledge graph");
                }
                
                kg_builder.get_knowledge_graph().stats.clone()
            })
        });
    }
    
    group.finish();
}

/// 内存使用情况基准测试
fn bench_memory_usage(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    c.bench_function("memory_usage_large_graph", |b| {
        b.to_async(&rt).iter(|| async {
            let config = ZhushoudeConfig::default();
            let db_manager = Arc::new(DatabaseManager::new(config).await.expect("Failed to create database"));
            let mut kg_builder = KnowledgeGraphBuilder::new(db_manager);
            
            let ner = ChineseNER::new().expect("Failed to create NER");
            let extractor = ChineseRelationExtractor::new().expect("Failed to create relation extractor");
            
            // 创建大量文档来测试内存使用
            let large_text = create_long_text(10);
            
            for i in 0..50 {
                let entities = ner.extract_entities(&large_text).expect("Failed to extract entities");
                let relations = extractor.extract_relations(&large_text, &entities)
                    .expect("Failed to extract relations");
                
                kg_builder.build_from_entities_and_relations(
                    black_box(&entities), 
                    black_box(&relations), 
                    &format!("large_doc_{}", i)
                ).await.expect("Failed to build knowledge graph");
            }
            
            let kg = kg_builder.get_knowledge_graph();
            (kg.stats.node_count, kg.stats.edge_count)
        })
    });
}

/// 并发性能基准测试
fn bench_concurrent_processing(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    c.bench_function("concurrent_ner_processing", |b| {
        b.to_async(&rt).iter(|| async {
            let ner = Arc::new(ChineseNER::new().expect("Failed to create NER"));
            let test_texts = create_test_texts();
            
            // 并发处理多个文本
            let mut handles = Vec::new();
            for text in test_texts {
                let ner_clone = ner.clone();
                let handle = tokio::spawn(async move {
                    ner_clone.extract_entities(black_box(&text)).expect("Failed to extract entities")
                });
                handles.push(handle);
            }
            
            // 等待所有任务完成
            let mut total_entities = 0;
            for handle in handles {
                let entities = handle.await.expect("Task should complete");
                total_entities += entities.len();
            }
            
            total_entities
        })
    });
}

criterion_group!(
    nlp_benches,
    bench_ner_performance,
    bench_relation_extraction_performance,
    bench_knowledge_graph_building,
    bench_end_to_end_pipeline,
    bench_memory_usage,
    bench_concurrent_processing
);

criterion_main!(nlp_benches);
