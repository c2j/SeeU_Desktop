//! zhushoude_duckdb API演示
//! 
//! 这个示例展示了如何使用zhushoude_duckdb的完整API功能

use zhushoude_duckdb::*;
use std::sync::Arc;
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🚀 zhushoude_duckdb API演示");
    println!("{}", "=".repeat(60));
    
    // 1. 创建配置
    let config = ZhushoudeConfig {
        database_path: ":memory:".to_string(),
        embedding: EmbeddingConfig {
            model_name: "BAAI/bge-small-zh-v1.5".to_string(),
            vector_dimension: 512,
            batch_size: 8,
            max_cache_size: 1000,
            enable_chinese_optimization: true,
            normalize_vectors: true,
        },
        performance: PerformanceConfig {
            thread_pool_size: Some(4),
            memory_limit_mb: 512,
            enable_monitoring: true,
            cache_strategy: CacheStrategy::LRU,
        },
        hybrid: HybridConfig {
            semantic_weight: 0.7,
            graph_weight: 0.3,
            fusion_strategy: FusionStrategy::WeightedAverage,
        },
        graph: GraphConfig::default(),
    };
    
    println!("📋 配置信息:");
    println!("  - 数据库路径: {}", config.database_path);
    println!("  - 向量模型: {}", config.embedding.model_name);
    println!("  - 向量维度: {}", config.embedding.vector_dimension);
    println!("  - 内存限制: {}MB", config.performance.memory_limit_mb);
    println!();
    
    // 2. 初始化数据库和API客户端
    println!("🔧 初始化系统...");
    let db = Arc::new(ZhushoudeDB::new(config).await?);
    let client = ZhushoudeClient::new();
    println!("✅ 系统初始化完成");
    println!();
    
    // 3. 演示文档添加API
    println!("📝 文档添加演示:");
    
    let documents = vec![
        api::AddDocumentRequest {
            id: "ai_intro".to_string(),
            title: "人工智能简介".to_string(),
            content: "人工智能（AI）是计算机科学的一个分支，致力于创建能够执行通常需要人类智能的任务的系统。AI包括机器学习、深度学习、自然语言处理等多个子领域。".to_string(),
            doc_type: "Note".to_string(),
            metadata: {
                let mut meta = HashMap::new();
                meta.insert("category".to_string(), serde_json::Value::String("AI".to_string()));
                meta.insert("level".to_string(), serde_json::Value::String("beginner".to_string()));
                meta.insert("tags".to_string(), serde_json::Value::Array(vec![
                    serde_json::Value::String("AI".to_string()),
                    serde_json::Value::String("机器学习".to_string()),
                ]));
                meta
            },
        },
        api::AddDocumentRequest {
            id: "ml_basics".to_string(),
            title: "机器学习基础".to_string(),
            content: "机器学习是人工智能的一个重要分支，它使计算机能够从数据中学习模式，而无需明确编程。主要包括监督学习、无监督学习和强化学习三种类型。".to_string(),
            doc_type: "Note".to_string(),
            metadata: {
                let mut meta = HashMap::new();
                meta.insert("category".to_string(), serde_json::Value::String("ML".to_string()));
                meta.insert("level".to_string(), serde_json::Value::String("intermediate".to_string()));
                meta
            },
        },
        api::AddDocumentRequest {
            id: "python_code".to_string(),
            title: "Python数据分析代码".to_string(),
            content: r#"
import pandas as pd
import numpy as np
from sklearn.model_selection import train_test_split
from sklearn.linear_model import LinearRegression

class DataAnalyzer:
    def __init__(self, data_path):
        self.data = pd.read_csv(data_path)
        self.model = LinearRegression()
    
    def preprocess_data(self):
        """数据预处理"""
        # 处理缺失值
        self.data = self.data.dropna()
        
        # 特征工程
        self.data['feature_ratio'] = self.data['feature1'] / self.data['feature2']
        
        return self.data
    
    def train_model(self, target_column):
        """训练模型"""
        X = self.data.drop(columns=[target_column])
        y = self.data[target_column]
        
        X_train, X_test, y_train, y_test = train_test_split(X, y, test_size=0.2)
        
        self.model.fit(X_train, y_train)
        score = self.model.score(X_test, y_test)
        
        return score
"#.to_string(),
            doc_type: "Code".to_string(),
            metadata: {
                let mut meta = HashMap::new();
                meta.insert("language".to_string(), serde_json::Value::String("Python".to_string()));
                meta.insert("framework".to_string(), serde_json::Value::String("scikit-learn".to_string()));
                meta.insert("complexity".to_string(), serde_json::Value::String("medium".to_string()));
                meta
            },
        },
    ];
    
    // 批量添加文档
    let batch_request = api::AddDocumentsBatchRequest { documents };
    let batch_response = client.add_documents_batch(batch_request).await?;
    
    if batch_response.success {
        if let Some(batch_data) = batch_response.data {
            println!("  ✅ 成功添加 {} 个文档", batch_data.success_count);
            println!("  ⏱️ 总处理时间: {}ms", batch_data.total_time_ms);
            
            for (i, result) in batch_data.results.iter().enumerate() {
                if result.success {
                    println!("    📄 文档 {}: {} ({}ms)", 
                        i + 1, result.document_id, result.processing_time_ms);
                }
            }
        }
    }
    println!();
    
    // 4. 演示搜索API
    println!("🔍 搜索功能演示:");
    
    let search_queries = vec![
        ("人工智能技术", api::SearchType::Semantic),
        ("机器学习算法", api::SearchType::Semantic),
        ("Python数据处理", api::SearchType::Hybrid),
    ];
    
    for (query, search_type) in search_queries {
        println!("  🔎 搜索: \"{}\" (类型: {:?})", query, search_type);
        
        let search_request = api::SearchRequest {
            query: query.to_string(),
            search_type,
            limit: 5,
            options: api::SearchOptions {
                enable_semantic: true,
                enable_graph: matches!(search_type, api::SearchType::Hybrid),
                weights: api::SearchWeights {
                    semantic: 0.8,
                    graph: 0.2,
                },
                filters: HashMap::new(),
                sort_by: api::SortBy::Relevance,
            },
        };
        
        let search_response = client.search(search_request).await?;
        
        if search_response.success {
            if let Some(search_data) = search_response.data {
                println!("    📊 找到 {} 个结果 (耗时: {}ms)", 
                    search_data.total_count, search_data.duration_ms);
                
                for (i, result) in search_data.results.iter().enumerate() {
                    println!("      {}. {} (分数: {:.4})", 
                        i + 1, result.title, result.score);
                    println!("         摘要: {}", 
                        if result.content_snippet.len() > 100 {
                            format!("{}...", &result.content_snippet[..100])
                        } else {
                            result.content_snippet.clone()
                        });
                }
            }
        }
        println!();
    }
    
    // 5. 演示代码分析API
    println!("🔬 代码分析演示:");
    
    let rust_code = r#"
use std::collections::HashMap;

pub struct UserManager {
    users: HashMap<String, User>,
}

impl UserManager {
    pub fn new() -> Self {
        Self {
            users: HashMap::new(),
        }
    }
    
    pub fn add_user(&mut self, user: User) -> Result<(), String> {
        if self.users.contains_key(&user.id) {
            return Err("用户已存在".to_string());
        }
        
        self.users.insert(user.id.clone(), user);
        Ok(())
    }
    
    pub fn get_user(&self, id: &str) -> Option<&User> {
        self.users.get(id)
    }
    
    pub fn update_user(&mut self, id: &str, user: User) -> Result<(), String> {
        if !self.users.contains_key(id) {
            return Err("用户不存在".to_string());
        }
        
        self.users.insert(id.to_string(), user);
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct User {
    pub id: String,
    pub name: String,
    pub email: String,
}
"#;
    
    let analyze_request = api::AnalyzeCodeRequest {
        code: rust_code.to_string(),
        language: "rust".to_string(),
        options: api::AnalyzeOptions {
            build_dependency_graph: true,
            extract_functions: true,
            extract_classes: true,
            calculate_complexity: true,
        },
    };
    
    let analyze_response = client.analyze_code(analyze_request).await?;
    
    if analyze_response.success {
        if let Some(analysis_data) = analyze_response.data {
            println!("  📋 代码分析结果:");
            println!("    - 语言: {}", analysis_data.analysis.language);
            println!("    - 代码行数: {}", analysis_data.analysis.lines_of_code);
            println!("    - 复杂度: {}", analysis_data.analysis.complexity);
            println!("    - 函数数量: {}", analysis_data.analysis.functions.len());
            println!("    - 类数量: {}", analysis_data.analysis.classes.len());
            println!("    - 处理时间: {}ms", analysis_data.processing_time_ms);
        }
    }
    println!();
    
    // 6. 演示统计信息API
    println!("📊 系统统计信息:");
    
    let stats_request = api::StatsRequest {
        stats_type: api::StatsType::System,
    };
    
    let stats_response = client.get_stats(stats_request).await?;
    
    if stats_response.success {
        if let Some(stats_data) = stats_response.data {
            let stats = &stats_data.stats;
            
            println!("  📄 文档统计:");
            println!("    - 总文档数: {}", stats.documents.total_count);
            println!("    - 总大小: {} bytes", stats.documents.total_size_bytes);
            
            println!("  🔢 向量统计:");
            println!("    - 总向量数: {}", stats.vectors.total_count);
            println!("    - 向量维度: {}", stats.vectors.dimension);
            println!("    - 索引大小: {} bytes", stats.vectors.index_size_bytes);
            
            println!("  🕸️ 图统计:");
            println!("    - 节点数: {}", stats.graph.node_count);
            println!("    - 边数: {}", stats.graph.edge_count);
            println!("    - 连通分量: {}", stats.graph.connected_components);
            
            println!("  💾 缓存统计:");
            println!("    - 命中次数: {}", stats.cache.hits);
            println!("    - 未命中次数: {}", stats.cache.misses);
            println!("    - 命中率: {:.2}%", stats.cache.hit_rate * 100.0);
            println!("    - 缓存大小: {}", stats.cache.size);
            
            println!("  ⚡ 性能统计:");
            println!("    - 平均查询时间: {:.2}ms", stats.performance.avg_query_time_ms);
            println!("    - 查询吞吐量: {:.2} QPS", stats.performance.queries_per_second);
            println!("    - 内存使用: {:.2}MB", stats.performance.memory_usage_mb);
            println!("    - CPU使用率: {:.2}%", stats.performance.cpu_usage_percent);
            
            println!("  🕐 统计时间: {}", stats_data.collected_at);
        }
    }
    println!();
    
    println!("🎉 API演示完成！");
    println!("💡 提示: 这个演示展示了zhushoude_duckdb的主要API功能");
    println!("   包括文档管理、语义搜索、代码分析和系统监控等。");
    
    Ok(())
}
