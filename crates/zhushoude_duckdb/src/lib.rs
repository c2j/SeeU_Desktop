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
pub mod nlp;
pub mod search;
pub mod text;
pub mod types;
pub mod errors;
pub mod utils;
pub mod monitoring;
pub mod api;
pub mod model;

// 重新导出主要类型
pub use config::*;
pub use database::*;
pub use embedding::*;
pub use vector::*;
pub use graph::*;
pub use hybrid::*;
pub use nlp::*;
pub use search::*;
pub use text::*;
pub use types::*;
pub use errors::*;
pub use monitoring::*;
pub use api::*;
pub use model::*;

use std::sync::Arc;

/// zhushoude_duckdb 主要入口点
pub struct ZhushoudeDB {
    config: ZhushoudeConfig,
    db_manager: Arc<DatabaseManager>,
    semantic_engine: Arc<SemanticSearchEngine>,
    graph_analyzer: Arc<hybrid::analyzer::CodeGraphAnalyzer>,
    hybrid_engine: Arc<HybridSearchEngine>,
    /// 中文NER
    ner: Arc<ChineseNER>,
    /// 关系抽取器
    relation_extractor: Arc<ChineseRelationExtractor>,
    /// 知识图谱构建器
    knowledge_graph_builder: Arc<tokio::sync::Mutex<KnowledgeGraphBuilder>>,
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

        // 初始化NLP组件
        let ner = Arc::new(ChineseNER::new()?);
        let relation_extractor = Arc::new(ChineseRelationExtractor::new()?);
        let knowledge_graph_builder = Arc::new(tokio::sync::Mutex::new(
            KnowledgeGraphBuilder::new(db_manager.clone())
        ));

        let instance = Self {
            config,
            db_manager,
            semantic_engine,
            graph_analyzer,
            hybrid_engine,
            ner,
            relation_extractor,
            knowledge_graph_builder,
        };

        // 初始化向量索引管理器
        instance.initialize_vector_indexes().await?;

        Ok(instance)
    }

    /// 初始化向量索引管理器
    async fn initialize_vector_indexes(&self) -> Result<()> {
        println!("🔧 初始化向量索引管理器...");

        // 获取语义搜索引擎的索引管理器并初始化
        let index_manager = self.semantic_engine.get_index_manager().await;
        let mut index_manager = index_manager.lock().await;

        // 初始化索引管理器（创建元数据表，加载现有索引）
        index_manager.initialize().await?;

        // 检查是否存在默认索引，如果不存在则创建
        let default_index_name = "idx_document_embeddings_embedding";
        if !index_manager.index_exists(default_index_name) {
            println!("📋 创建默认向量索引: {}", default_index_name);

            // 创建自适应索引（根据数据量自动选择最佳策略）
            index_manager.create_index(
                "document_embeddings",
                "embedding",
                crate::vector::index::IndexType::Adaptive,
                self.config.embedding.vector_dimension
            ).await?;

            println!("✅ 默认向量索引创建完成");
        } else {
            println!("✅ 发现现有向量索引: {}", default_index_name);
        }

        println!("✅ 向量索引管理器初始化完成");
        Ok(())
    }
    
    /// 添加笔记文档
    pub async fn add_note(&self, document: &Document) -> Result<()> {
        // 添加到语义搜索引擎
        self.semantic_engine.add_document(document).await?;

        // 进行实体提取和关系识别
        self.process_note_entities_and_relations(document).await?;

        Ok(())
    }

    /// 添加笔记文档并返回提取的实体和关系
    pub async fn add_note_with_entities(&self, document: &Document) -> Result<(Vec<Entity>, Vec<EntityRelation>)> {
        // 添加到语义搜索引擎
        self.semantic_engine.add_document(document).await?;

        // 进行实体提取和关系识别
        let (entities, relations) = self.extract_entities_and_relations(&document.content).await?;

        // 构建知识图谱
        let mut kg_builder = self.knowledge_graph_builder.lock().await;
        kg_builder.build_from_entities_and_relations(&entities, &relations, &document.id).await?;

        Ok((entities, relations))
    }

    /// 批量添加笔记文档
    pub async fn add_notes_batch(&self, documents: &[Document]) -> Result<()> {
        // 批量添加到语义搜索引擎
        self.semantic_engine.add_documents_batch(documents).await?;

        // 批量进行实体提取和关系识别
        for document in documents {
            self.process_note_entities_and_relations(document).await?;
        }

        Ok(())
    }
    
    /// 语义搜索笔记
    pub async fn search_notes(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>> {
        self.semantic_engine.search(query, limit).await
    }

    /// 清除所有语义索引
    pub async fn clear_all_semantic_indexes(&self) -> Result<()> {
        // 清除语义搜索引擎中的所有文档
        self.semantic_engine.clear_all_documents().await?;

        // 清除向量索引
        let index_manager = self.semantic_engine.get_index_manager().await;
        let mut index_manager = index_manager.lock().await;

        // 获取所有索引名称并删除
        let index_names = index_manager.list_indexes();
        for index_name in index_names {
            if let Err(e) = index_manager.drop_index(&index_name).await {
                log::warn!("删除索引 {} 时出错: {}", index_name, e);
            }
        }

        log::info!("✅ 所有语义索引已清除");
        Ok(())
    }

    /// 重建所有语义索引
    pub async fn rebuild_all_semantic_indexes(&self) -> Result<()> {
        // 重建语义搜索引擎的索引
        let index_manager = self.semantic_engine.get_index_manager().await;
        let mut index_manager = index_manager.lock().await;

        // 获取所有索引名称并重建
        let index_names = index_manager.list_indexes();
        for index_name in index_names {
            if let Err(e) = index_manager.rebuild_index(&index_name).await {
                log::warn!("重建索引 {} 时出错: {}", index_name, e);
            }
        }

        log::info!("✅ 所有语义索引已重建");
        Ok(())
    }

    /// 创建向量索引以优化搜索性能
    pub async fn create_vector_index(&self) -> Result<()> {
        let index_manager = self.semantic_engine.get_index_manager().await;
        let mut index_manager = index_manager.lock().await;

        // 检查是否已存在索引
        let existing_indexes = index_manager.list_indexes();
        if existing_indexes.iter().any(|idx| idx.contains("document_embeddings") && idx.contains("embedding")) {
            log::info!("向量索引已存在，跳过创建");
            return Ok(());
        }

        // 创建自适应向量索引
        match index_manager.create_index(
            "document_embeddings",
            "embedding",
            crate::vector::index::IndexType::Adaptive,
            384 // BGE-small-zh的向量维度
        ).await {
            Ok(index_name) => {
                log::info!("✅ 成功创建向量索引: {}", index_name);
            }
            Err(e) => {
                log::warn!("⚠️ 创建向量索引失败: {}", e);
                return Err(e);
            }
        }

        Ok(())
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

    /// 从文本中提取实体和关系
    pub async fn extract_entities_and_relations(&self, text: &str) -> Result<(Vec<Entity>, Vec<EntityRelation>)> {
        // 提取实体
        let entities = self.ner.extract_entities(text)?;

        // 提取关系
        let relations = self.relation_extractor.extract_relations(text, &entities)?;

        Ok((entities, relations))
    }

    /// 处理笔记的实体和关系（内部方法）
    async fn process_note_entities_and_relations(&self, document: &Document) -> Result<()> {
        let (entities, relations) = self.extract_entities_and_relations(&document.content).await?;

        // 构建知识图谱
        let mut kg_builder = self.knowledge_graph_builder.lock().await;
        kg_builder.build_from_entities_and_relations(&entities, &relations, &document.id).await?;

        Ok(())
    }

    /// 获取实体的相关实体
    pub async fn get_related_entities(&self, entity_text: &str, max_depth: usize) -> Result<Vec<String>> {
        let kg_builder = self.knowledge_graph_builder.lock().await;
        Ok(kg_builder.find_related_entities(entity_text, max_depth))
    }

    /// 获取实体的直接关系
    pub async fn get_entity_relations(&self, entity_text: &str) -> Result<Vec<KnowledgeEdge>> {
        let kg_builder = self.knowledge_graph_builder.lock().await;
        Ok(kg_builder.get_entity_relations(entity_text).into_iter().cloned().collect())
    }

    /// 获取知识图谱统计信息
    pub async fn get_knowledge_graph_stats(&self) -> Result<KnowledgeGraphStats> {
        let kg_builder = self.knowledge_graph_builder.lock().await;
        Ok(kg_builder.get_knowledge_graph().stats.clone())
    }

    /// 基于实体的增强语义搜索
    pub async fn search_notes_with_entities(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>> {
        // 首先进行常规语义搜索
        let mut results = self.semantic_engine.search(query, limit).await?;

        // 提取查询中的实体
        let (query_entities, _) = self.extract_entities_and_relations(query).await?;

        if !query_entities.is_empty() {
            // 基于实体查找相关笔记
            let kg_builder = self.knowledge_graph_builder.lock().await;
            for entity in &query_entities {
                let related_entities = kg_builder.find_related_entities(&entity.text, 2);

                // 为相关实体的笔记增加权重
                for result in &mut results {
                    for related_entity in &related_entities {
                        if result.content.contains(related_entity) {
                            result.similarity_score *= 1.2; // 增加20%权重
                        }
                    }
                }
            }

            // 重新排序
            results.sort_by(|a, b| b.similarity_score.partial_cmp(&a.similarity_score).unwrap_or(std::cmp::Ordering::Equal));
        }

        Ok(results)
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
