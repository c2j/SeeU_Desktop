//! ZhushoudeDB 基本使用示例

use zhushoude_duckdb::*;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🚀 ZhushoudeDB 基本使用示例");
    
    // 1. 创建配置
    println!("\n📋 创建配置...");
    let config = ZhushoudeConfig {
        database_path: "examples/data/demo.db".to_string(),
        embedding: EmbeddingConfig {
            model_name: "bge-small-zh".to_string(),
            vector_dimension: 512,
            batch_size: 16,
            max_cache_size: 500,
            enable_chinese_optimization: true,
            normalize_vectors: true,
        },
        performance: PerformanceConfig {
            thread_pool_size: Some(4),
            memory_limit_mb: 512,
            enable_monitoring: true,
            cache_strategy: CacheStrategy::LRU,
        },
        ..Default::default()
    };
    
    println!("✅ 配置创建完成");
    
    // 2. 初始化数据库
    println!("\n🗄️ 初始化数据库...");
    let db = ZhushoudeDB::new(config).await?;
    println!("✅ 数据库初始化完成");
    
    // 3. 添加示例文档
    println!("\n📝 添加示例文档...");
    
    let documents = vec![
        Document {
            id: "note1".to_string(),
            title: "机器学习基础".to_string(),
            content: "机器学习是人工智能的一个重要分支，它使计算机能够在没有明确编程的情况下学习和改进。".to_string(),
            doc_type: DocumentType::Note,
            metadata: serde_json::json!({
                "author": "张三",
                "tags": ["AI", "机器学习", "基础"]
            }),
        },
        Document {
            id: "note2".to_string(),
            title: "深度学习神经网络".to_string(),
            content: "深度学习是机器学习的一个子集，它使用多层神经网络来模拟人脑的工作方式。".to_string(),
            doc_type: DocumentType::Note,
            metadata: serde_json::json!({
                "author": "李四",
                "tags": ["深度学习", "神经网络", "AI"]
            }),
        },
        Document {
            id: "code1".to_string(),
            title: "Python 数据处理".to_string(),
            content: r#"
import pandas as pd
import numpy as np

def process_data(df):
    """处理数据的函数"""
    # 数据清洗
    df = df.dropna()
    
    # 数据转换
    df['processed'] = df['value'].apply(lambda x: x * 2)
    
    return df

# 使用示例
data = pd.DataFrame({'value': [1, 2, 3, None, 5]})
result = process_data(data)
print(result)
"#.to_string(),
            doc_type: DocumentType::Code(CodeLanguage::Python),
            metadata: serde_json::json!({
                "language": "Python",
                "framework": "pandas",
                "tags": ["数据处理", "Python", "pandas"]
            }),
        },
    ];
    
    for doc in &documents {
        db.add_note(doc).await?;
        println!("  ✅ 已添加文档: {}", doc.title);
    }
    
    // 4. 语义搜索示例
    println!("\n🔍 执行语义搜索...");
    
    let search_queries = vec![
        "人工智能算法",
        "数据分析处理",
        "神经网络模型",
    ];
    
    for query in search_queries {
        println!("\n🔎 搜索: \"{}\"", query);
        let results = db.search_notes(query, 5).await?;
        
        if results.is_empty() {
            println!("  📭 未找到相关结果");
        } else {
            for (i, result) in results.iter().enumerate() {
                println!("  {}. {} (相似度: {:.3})", 
                    i + 1, 
                    result.title, 
                    result.similarity_score
                );
            }
        }
    }
    
    // 5. 代码分析示例
    println!("\n🔧 代码分析示例...");
    
    let java_code = r#"
public class Calculator {
    private int result;
    
    public Calculator() {
        this.result = 0;
    }
    
    public int add(int a, int b) {
        result = a + b;
        return result;
    }
    
    public int multiply(int a, int b) {
        result = a * b;
        return result;
    }
    
    public int getResult() {
        return result;
    }
}
"#;
    
    println!("📊 分析 Java 代码...");
    let code_graph = db.analyze_code(java_code, &CodeLanguage::Java).await?;
    println!("  ✅ 发现 {} 个节点, {} 条边", 
        code_graph.nodes.len(), 
        code_graph.edges.len()
    );
    
    // 6. 混合搜索示例
    println!("\n🎯 混合搜索示例...");
    
    let hybrid_query = HybridQuery {
        text: "数据处理算法".to_string(),
        query_type: QueryType::General,
        limit: 10,
        enable_semantic: true,
        enable_graph: true,
        weights: SearchWeights {
            semantic: 0.7,
            graph: 0.3,
        },
    };
    
    println!("🔍 执行混合搜索: \"{}\"", hybrid_query.text);
    let hybrid_results = db.hybrid_search(&hybrid_query).await?;
    
    if hybrid_results.is_empty() {
        println!("  📭 未找到相关结果");
    } else {
        for (i, result) in hybrid_results.iter().enumerate() {
            println!("  {}. {} (最终分数: {:.3}, 类型: {:?})", 
                i + 1, 
                result.title, 
                result.final_score,
                result.result_type
            );
        }
    }
    
    // 7. 性能统计
    println!("\n📈 性能统计...");
    let cache_stats = db.get_cache_stats();
    println!("  缓存命中率: {:.1}%", cache_stats.hit_rate * 100.0);
    println!("  缓存大小: {}", cache_stats.size);
    println!("  缓存命中: {}", cache_stats.hits);
    println!("  缓存未命中: {}", cache_stats.misses);
    
    // 8. 中文文本处理示例
    println!("\n🇨🇳 中文文本处理示例...");
    
    let chinese_processor = ChineseTextProcessor::new();
    let test_texts = vec![
        "機器學習，很有趣！",
        "  这是一个很长的文本，包含了各种标点符号：，。？！；：\"\"''（）【】  ",
        "hello 世界 mix language text",
    ];
    
    for text in test_texts {
        let processed = chinese_processor.preprocess(text);
        let language = chinese_processor.detect_language(text);
        println!("  原文: \"{}\"", text);
        println!("  处理后: \"{}\"", processed);
        println!("  语言: {:?}", language);
        println!();
    }
    
    println!("🎉 示例运行完成！");
    
    Ok(())
}
