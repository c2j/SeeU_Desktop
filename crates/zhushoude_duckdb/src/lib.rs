//! # zhushoude_duckdb
//! 
//! 基于DuckDB的嵌入式语义搜索和图计算库
//! 
//! ## 特性
//! 
//! - 🇨🇳 内置中文语义模型 (bge-small-zh)
//! - 🔍 高性能向量相似度搜索 (HNSW索引)
//! - 📊 图数据存储和计算 (SQL/PGQ)
//! - 🚀 完全进程内运行，无外部依赖
//! - 💾 低内存占用 (~200MB)
//! 
//! ## 使用示例
//! 
//! ```rust,no_run
//! use zhushoude_duckdb::*;
//! 
//! #[tokio::main]
//! async fn main() -> Result<()> {
//!     // 初始化数据库
//!     let config = ZhushoudeConfig::default();
//!     let db = ZhushoudeDB::new(config).await?;
//!     
//!     // 添加文档
//!     db.add_note(&Document {
//!         id: "note1".to_string(),
//!         title: "机器学习笔记".to_string(),
//!         content: "深度学习是人工智能的重要分支".to_string(),
//!         doc_type: DocumentType::Note,
//!         metadata: serde_json::json!({}),
//!     }).await?;
//!     
//!     // 语义搜索
//!     let results = db.search_notes("人工智能", 10).await?;
//!     
//!     for result in results {
//!         println!("找到相关笔记: {} (相似度: {:.3})", 
//!                  result.title, result.similarity_score);
//!     }
//!     
//!     Ok(())
//! }
//! ```

pub mod config;
pub mod database;
pub mod embedding;
pub mod vector;
pub mod graph;
pub mod hybrid;
pub mod types;
pub mod errors;
pub mod utils;
pub mod monitoring;
pub mod api;

// 重新导出主要类型
pub use config::*;
pub use database::*;
pub use embedding::*;
pub use vector::*;
pub use graph::*;
pub use hybrid::*;
pub use types::*;
pub use errors::*;
pub use monitoring::*;
pub use api::*;

use std::sync::Arc;

/// zhushoude_duckdb 主要入口点
pub struct ZhushoudeDB {
    config: ZhushoudeConfig,
    db_manager: Arc<DatabaseManager>,
    semantic_engine: Arc<SemanticSearchEngine>,
    graph_analyzer: Arc<hybrid::analyzer::CodeGraphAnalyzer>,
    hybrid_engine: Arc<HybridSearchEngine>,
}

impl ZhushoudeDB {
    /// 创建新的数据库实例
    pub async fn new(config: ZhushoudeConfig) -> Result<Self> {
        let db_manager = Arc::new(DatabaseManager::new(config.clone()).await?);
        
        let embedding_engine = Arc::new(EmbeddingEngine::new(config.embedding.clone()).await?);
        let semantic_engine = Arc::new(SemanticSearchEngine::new(
            db_manager.clone(),
            embedding_engine,
        ));
        
        let graph_analyzer = Arc::new(hybrid::analyzer::CodeGraphAnalyzer::new(db_manager.clone()));
        
        let hybrid_engine = Arc::new(HybridSearchEngine::new(
            semantic_engine.clone(),
            graph_analyzer.clone(),
            config.hybrid.clone(),
        ));
        
        Ok(Self {
            config,
            db_manager,
            semantic_engine,
            graph_analyzer,
            hybrid_engine,
        })
    }
    
    /// 添加笔记文档
    pub async fn add_note(&self, document: &Document) -> Result<()> {
        self.semantic_engine.add_document(document).await
    }
    
    /// 语义搜索笔记
    pub async fn search_notes(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>> {
        self.semantic_engine.search(query, limit).await
    }

    /// 分析代码
    pub async fn analyze_code(&self, code: &str, language: &str) -> Result<Vec<GraphNode>> {
        self.graph_analyzer.analyze_code(code, language).await
    }

    /// 混合搜索
    pub async fn hybrid_search(&self, query: &HybridQuery) -> Result<Vec<HybridResult>> {
        self.hybrid_engine.hybrid_search(query).await
    }

    /// 获取缓存统计
    pub fn get_cache_stats(&self) -> types::CacheStats {
        self.semantic_engine.get_cache_stats()
    }
    
    /// 获取性能统计
    pub fn get_performance_stats(&self) -> types::PerformanceStats {
        types::PerformanceStats {
            memory_usage: self.get_memory_usage(),
            cache_stats: self.semantic_engine.get_cache_stats(),
            query_stats: self.get_query_stats(),
        }
    }
    
    fn get_memory_usage(&self) -> MemoryUsage {
        // TODO: 实现内存使用统计
        MemoryUsage::default()
    }
    
    fn get_query_stats(&self) -> QueryStats {
        // TODO: 实现查询统计
        QueryStats::default()
    }
}
