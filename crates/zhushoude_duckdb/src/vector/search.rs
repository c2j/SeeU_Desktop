//! 向量搜索模块

use crate::{Result, DatabaseManager, EmbeddingEngine, SearchResult};
use crate::types::CacheStats;
use crate::vector::index::{VectorIndexManager, IndexType, VectorSearchResult};
use std::sync::{Arc, Mutex};
use std::time::Instant;

/// 语义搜索引擎
pub struct SemanticSearchEngine {
    db_manager: Arc<DatabaseManager>,
    embedding_engine: Arc<EmbeddingEngine>,
    index_manager: Arc<Mutex<VectorIndexManager>>,
    search_stats: Arc<Mutex<SearchStats>>,
}

/// 搜索统计信息
#[derive(Debug, Default, Clone)]
pub struct SearchStats {
    pub total_searches: u64,
    pub total_search_time_ms: u64,
    pub cache_hits: u64,
    pub index_hits: u64,
}

impl SemanticSearchEngine {
    /// 创建新的语义搜索引擎
    pub fn new(db_manager: Arc<DatabaseManager>, embedding_engine: Arc<EmbeddingEngine>) -> Self {
        let index_manager = VectorIndexManager::new(db_manager.clone());

        Self {
            db_manager,
            embedding_engine,
            index_manager: Arc::new(Mutex::new(index_manager)),
            search_stats: Arc::new(Mutex::new(SearchStats::default())),
        }
    }

    /// 初始化搜索引擎
    pub async fn initialize(&self) -> Result<()> {
        let mut index_manager = self.index_manager.lock()
            .map_err(|e| crate::Error::DatabaseError(format!("获取索引管理器失败: {}", e)))?;

        index_manager.initialize().await?;

        // 检查是否存在默认索引，如果不存在则创建
        self.ensure_default_index().await?;

        println!("✅ 语义搜索引擎初始化完成");
        Ok(())
    }

    /// 确保存在默认索引
    async fn ensure_default_index(&self) -> Result<()> {
        let mut index_manager = self.index_manager.lock()
            .map_err(|e| crate::Error::DatabaseError(format!("获取索引管理器失败: {}", e)))?;

        let default_index_name = "idx_document_embeddings_embedding";

        if index_manager.get_index_info(default_index_name).is_none() {
            // 创建自适应索引
            let dimension = self.embedding_engine.get_config().vector_dimension;
            index_manager.create_index(
                "document_embeddings",
                "embedding",
                IndexType::Adaptive,
                dimension
            ).await?;

            println!("✅ 创建默认向量索引: {}", default_index_name);
        }

        Ok(())
    }
    
    /// 语义搜索
    pub async fn search(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>> {
        self.search_with_threshold(query, limit, 0.0).await
    }

    /// 带阈值的语义搜索
    pub async fn search_with_threshold(&self, query: &str, limit: usize, threshold: f32) -> Result<Vec<SearchResult>> {
        let start_time = Instant::now();

        // 1. 对查询文本进行向量化
        let query_embedding = self.embedding_engine.encode_single(query).await?;

        // 2. 使用索引进行向量相似度搜索
        let vector_results = self.indexed_vector_search(&query_embedding, limit, threshold).await?;

        // 3. 转换为SearchResult格式
        let results = self.convert_vector_results_to_search_results(vector_results).await?;

        // 4. 更新搜索统计
        self.update_search_stats(start_time.elapsed().as_millis() as u64).await;

        Ok(results)
    }

    /// 使用索引进行向量搜索
    async fn indexed_vector_search(&self, query_embedding: &[f32], limit: usize, threshold: f32) -> Result<Vec<VectorSearchResult>> {
        let index_manager = self.index_manager.lock()
            .map_err(|e| crate::Error::DatabaseError(format!("获取索引管理器失败: {}", e)))?;

        let default_index_name = "idx_document_embeddings_embedding";

        // 尝试使用索引搜索
        match index_manager.search_with_index(default_index_name, query_embedding, limit, threshold).await {
            Ok(results) => {
                // 更新索引命中统计
                self.increment_index_hits().await;
                Ok(results)
            }
            Err(_) => {
                // 如果索引搜索失败，回退到线性搜索
                println!("⚠️ 索引搜索失败，回退到线性搜索");
                self.fallback_linear_search(query_embedding, limit, threshold).await
            }
        }
    }

    /// 回退线性搜索
    async fn fallback_linear_search(&self, query_embedding: &[f32], limit: usize, threshold: f32) -> Result<Vec<VectorSearchResult>> {
        let query_vector_str = format!("[{}]",
            query_embedding.iter()
                .map(|f| f.to_string())
                .collect::<Vec<_>>()
                .join(",")
        );

        let sql = format!(
            "SELECT document_id, cosine_similarity(embedding, CAST('{}' AS FLOAT[])) as similarity
             FROM document_embeddings
             WHERE embedding IS NOT NULL
             AND cosine_similarity(embedding, CAST('{}' AS FLOAT[])) >= {}
             ORDER BY similarity DESC
             LIMIT {}",
            query_vector_str, query_vector_str, threshold, limit
        );

        let conn = self.db_manager.get_connection();
        let conn = conn.lock()
            .map_err(|e| crate::Error::DatabaseError(format!("获取数据库连接失败: {}", e)))?;

        let mut stmt = conn.prepare(&sql)
            .map_err(|e| crate::Error::DatabaseError(format!("准备搜索查询失败: {}", e)))?;

        let rows = stmt.query_map(duckdb::params![], |row| {
            Ok(VectorSearchResult {
                id: row.get::<_, String>(0)?,
                similarity: row.get::<_, f64>(1)?,
                metadata: None,
            })
        }).map_err(|e| crate::Error::DatabaseError(format!("执行搜索查询失败: {}", e)))?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row.map_err(|e| crate::Error::DatabaseError(format!("处理搜索结果失败: {}", e)))?);
        }

        Ok(results)
    }

    /// 转换向量搜索结果为标准搜索结果
    async fn convert_vector_results_to_search_results(&self, vector_results: Vec<VectorSearchResult>) -> Result<Vec<SearchResult>> {
        let mut results = Vec::new();

        for vector_result in vector_results {
            // 从数据库获取完整的文档信息
            if let Ok(search_result) = self.get_document_by_id(&vector_result.id).await {
                let mut result = search_result;
                result.similarity_score = vector_result.similarity;
                results.push(result);
            }
        }

        Ok(results)
    }

    /// 根据ID获取文档
    async fn get_document_by_id(&self, document_id: &str) -> Result<SearchResult> {
        let sql = "SELECT id, title, content, doc_type FROM documents WHERE id = ?";

        let conn = self.db_manager.get_connection();
        let conn = conn.lock()
            .map_err(|e| crate::Error::DatabaseError(format!("获取数据库连接失败: {}", e)))?;

        let mut stmt = conn.prepare(sql)
            .map_err(|e| crate::Error::DatabaseError(format!("准备查询失败: {}", e)))?;

        let result = stmt.query_row(duckdb::params![document_id], |row| {
            Ok(SearchResult {
                document_id: row.get(0)?,
                title: row.get(1)?,
                content: row.get(2)?,
                doc_type: row.get(3)?,
                similarity_score: 0.0, // 将在调用方设置
                metadata: None,
            })
        }).map_err(|e| crate::Error::DatabaseError(format!("查询文档失败: {}", e)))?;

        Ok(result)
    }

    /// 添加文档
    pub async fn add_document(&self, document: &crate::Document) -> Result<()> {
        // 1. 对文档内容进行向量化
        let embedding = self.embedding_engine.encode_single(&document.content).await?;

        // 2. 存储文档到数据库
        self.store_document_with_embedding(document, &embedding).await?;

        // 3. 更新索引统计
        self.update_index_stats().await?;

        Ok(())
    }

    /// 批量添加文档
    pub async fn add_documents_batch(&self, documents: &[crate::Document]) -> Result<()> {
        // 1. 批量向量化
        let contents: Vec<String> = documents.iter().map(|d| d.content.clone()).collect();
        let embeddings = self.embedding_engine.encode_batch(&contents).await?;

        // 2. 批量存储
        for (document, embedding) in documents.iter().zip(embeddings.iter()) {
            self.store_document_with_embedding(document, embedding).await?;
        }

        // 3. 更新索引统计
        self.update_index_stats().await?;

        println!("✅ 批量添加 {} 个文档", documents.len());
        Ok(())
    }

    /// 更新索引统计
    async fn update_index_stats(&self) -> Result<()> {
        let mut index_manager = self.index_manager.lock()
            .map_err(|e| crate::Error::DatabaseError(format!("获取索引管理器失败: {}", e)))?;

        let default_index_name = "idx_document_embeddings_embedding";
        index_manager.update_index_stats(default_index_name).await?;

        Ok(())
    }

    /// 更新搜索统计
    async fn update_search_stats(&self, search_time_ms: u64) {
        if let Ok(mut stats) = self.search_stats.lock() {
            stats.total_searches += 1;
            stats.total_search_time_ms += search_time_ms;
        }
    }

    /// 增加索引命中计数
    async fn increment_index_hits(&self) {
        if let Ok(mut stats) = self.search_stats.lock() {
            stats.index_hits += 1;
        }
    }

    /// 获取缓存统计
    pub fn get_cache_stats(&self) -> CacheStats {
        self.embedding_engine.get_cache_stats()
    }

    /// 获取搜索统计
    pub fn get_search_stats(&self) -> SearchStats {
        self.search_stats.lock()
            .map(|stats| stats.clone())
            .unwrap_or_default()
    }

    /// 获取索引信息
    pub fn get_index_info(&self) -> Result<Vec<String>> {
        let index_manager = self.index_manager.lock()
            .map_err(|e| crate::Error::DatabaseError(format!("获取索引管理器失败: {}", e)))?;

        Ok(index_manager.list_indexes())
    }

    /// 重建索引
    pub async fn rebuild_index(&self, index_name: &str) -> Result<()> {
        let mut index_manager = self.index_manager.lock()
            .map_err(|e| crate::Error::DatabaseError(format!("获取索引管理器失败: {}", e)))?;

        index_manager.rebuild_index(index_name).await?;
        println!("✅ 重建索引完成: {}", index_name);
        Ok(())
    }

    /// 优化索引
    pub async fn optimize_index(&self, index_name: &str) -> Result<()> {
        let mut index_manager = self.index_manager.lock()
            .map_err(|e| crate::Error::DatabaseError(format!("获取索引管理器失败: {}", e)))?;

        index_manager.optimize_index(index_name).await?;
        println!("✅ 优化索引完成: {}", index_name);
        Ok(())
    }

    /// 创建自定义索引
    pub async fn create_index(&self, table_name: &str, column_name: &str, index_type: IndexType, dimension: usize) -> Result<String> {
        let mut index_manager = self.index_manager.lock()
            .map_err(|e| crate::Error::DatabaseError(format!("获取索引管理器失败: {}", e)))?;

        let index_name = index_manager.create_index(table_name, column_name, index_type, dimension).await?;
        println!("✅ 创建自定义索引: {}", index_name);
        Ok(index_name)
    }

    /// 删除索引
    pub async fn drop_index(&self, index_name: &str) -> Result<()> {
        let mut index_manager = self.index_manager.lock()
            .map_err(|e| crate::Error::DatabaseError(format!("获取索引管理器失败: {}", e)))?;

        index_manager.drop_index(index_name).await?;
        println!("✅ 删除索引: {}", index_name);
        Ok(())
    }

    /// 向量相似度搜索
    async fn vector_similarity_search(&self, query_embedding: &[f32], limit: usize) -> Result<Vec<SearchResult>> {
        // 构建向量相似度查询SQL
        let query_vector_str = format!("[{}]",
            query_embedding.iter()
                .map(|f| f.to_string())
                .collect::<Vec<_>>()
                .join(",")
        );

        // 使用余弦相似度进行搜索
        let sql = format!(
            "SELECT d.id, d.title, d.content, d.doc_type,
                    (1.0 - array_cosine_distance(e.embedding, CAST('{}' AS FLOAT[]))) as similarity
             FROM documents d
             JOIN document_embeddings e ON d.id = e.document_id
             WHERE e.model_name = ?
             ORDER BY similarity DESC
             LIMIT ?",
            query_vector_str
        );

        let conn = self.db_manager.get_connection();
        let conn = conn.lock()
            .map_err(|e| crate::Error::DatabaseError(format!("获取数据库连接失败: {}", e)))?;

        let mut stmt = conn.prepare(&sql)
            .map_err(|e| crate::Error::DatabaseError(format!("准备SQL语句失败: {}", e)))?;

        let model_name = self.embedding_engine.get_config().model_name.clone();
        let rows = stmt.query_map(
            duckdb::params![model_name, limit],
            |row| {
                Ok(SearchResult {
                    document_id: row.get(0)?,
                    title: row.get(1)?,
                    content: row.get(2)?,
                    doc_type: row.get(3)?,
                    similarity_score: row.get(4)?,
                    metadata: None,
                })
            }
        ).map_err(|e| crate::Error::DatabaseError(format!("执行搜索查询失败: {}", e)))?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row.map_err(|e| crate::Error::DatabaseError(format!("处理搜索结果失败: {}", e)))?);
        }

        Ok(results)
    }

    /// 存储文档及其向量
    async fn store_document_with_embedding(&self, document: &crate::Document, embedding: &[f32]) -> Result<()> {
        let conn = self.db_manager.get_connection();
        let conn = conn.lock()
            .map_err(|e| crate::Error::DatabaseError(format!("获取数据库连接失败: {}", e)))?;

        // 开始事务
        conn.execute("BEGIN TRANSACTION", duckdb::params![])
            .map_err(|e| crate::Error::DatabaseError(format!("开始事务失败: {}", e)))?;

        // 存储文档
        let doc_sql = "INSERT OR REPLACE INTO documents (id, title, content, doc_type, metadata) VALUES (?, ?, ?, ?, ?)";
        conn.execute(doc_sql, duckdb::params![
            document.id,
            document.title,
            document.content,
            format!("{:?}", document.doc_type),
            serde_json::to_string(&document.metadata).unwrap_or_default()
        ]).map_err(|e| crate::Error::DatabaseError(format!("存储文档失败: {}", e)))?;

        // 存储向量
        let embedding_array = format!("[{}]",
            embedding.iter()
                .map(|f| f.to_string())
                .collect::<Vec<_>>()
                .join(",")
        );

        let embedding_sql = "INSERT OR REPLACE INTO document_embeddings (document_id, model_name, embedding) VALUES (?, ?, ?)";
        let model_name = &self.embedding_engine.get_config().model_name;
        conn.execute(embedding_sql, duckdb::params![
            document.id,
            model_name,
            embedding_array
        ]).map_err(|e| crate::Error::DatabaseError(format!("存储向量失败: {}", e)))?;

        // 提交事务
        conn.execute("COMMIT", duckdb::params![])
            .map_err(|e| crate::Error::DatabaseError(format!("提交事务失败: {}", e)))?;

        Ok(())
    }



    /// 删除文档
    pub async fn delete_document(&self, document_id: &str) -> Result<()> {
        let conn = self.db_manager.get_connection();
        let conn = conn.lock()
            .map_err(|e| crate::Error::DatabaseError(format!("获取数据库连接失败: {}", e)))?;

        // 开始事务
        conn.execute("BEGIN TRANSACTION", duckdb::params![])
            .map_err(|e| crate::Error::DatabaseError(format!("开始事务失败: {}", e)))?;

        // 删除文档（向量会因为外键约束自动删除）
        conn.execute("DELETE FROM documents WHERE id = ?", duckdb::params![document_id])
            .map_err(|e| crate::Error::DatabaseError(format!("删除文档失败: {}", e)))?;

        // 提交事务
        conn.execute("COMMIT", duckdb::params![])
            .map_err(|e| crate::Error::DatabaseError(format!("提交事务失败: {}", e)))?;

        Ok(())
    }

    /// 获取文档数量
    pub async fn get_document_count(&self) -> Result<usize> {
        let conn = self.db_manager.get_connection();
        let conn = conn.lock()
            .map_err(|e| crate::Error::DatabaseError(format!("获取数据库连接失败: {}", e)))?;

        let mut stmt = conn.prepare("SELECT COUNT(*) FROM documents")
            .map_err(|e| crate::Error::DatabaseError(format!("准备SQL语句失败: {}", e)))?;

        let count: i64 = stmt.query_row(duckdb::params![], |row| row.get(0))
            .map_err(|e| crate::Error::DatabaseError(format!("查询文档数量失败: {}", e)))?;

        Ok(count as usize)
    }

    /// 相似文档推荐
    pub async fn find_similar_documents(&self, document_id: &str, limit: usize) -> Result<Vec<SearchResult>> {
        // 1. 获取目标文档的向量
        let conn = self.db_manager.get_connection();
        let conn = conn.lock()
            .map_err(|e| crate::Error::DatabaseError(format!("获取数据库连接失败: {}", e)))?;

        let model_name = &self.embedding_engine.get_config().model_name;
        let mut stmt = conn.prepare("SELECT embedding FROM document_embeddings WHERE document_id = ? AND model_name = ?")
            .map_err(|e| crate::Error::DatabaseError(format!("准备SQL语句失败: {}", e)))?;

        let embedding_str: String = stmt.query_row(duckdb::params![document_id, model_name], |row| row.get(0))
            .map_err(|e| crate::Error::DatabaseError(format!("获取文档向量失败: {}", e)))?;

        // 2. 使用该向量进行相似度搜索
        let embedding = self.parse_vector_string(&embedding_str)?;
        let mut results = self.vector_similarity_search(&embedding, limit + 1).await?;

        // 3. 移除原文档自身
        results.retain(|r| r.document_id != document_id);
        results.truncate(limit);

        Ok(results)
    }

    /// 解析向量字符串
    fn parse_vector_string(&self, vector_str: &str) -> Result<Vec<f32>> {
        // 移除方括号
        let trimmed = vector_str.trim_start_matches('[').trim_end_matches(']');

        if trimmed.is_empty() {
            return Ok(Vec::new());
        }

        // 分割并解析每个数字
        let result: std::result::Result<Vec<f32>, std::num::ParseFloatError> = trimmed
            .split(',')
            .map(|s| s.trim().parse::<f32>())
            .collect();

        result.map_err(|e| crate::Error::DatabaseError(format!("解析向量字符串失败: {}", e)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ZhushoudeConfig, EmbeddingConfig, Document, DocumentType};
    use tempfile::NamedTempFile;
    
    #[tokio::test]
    async fn test_semantic_search_engine() {
        let config = ZhushoudeConfig {
            database_path: ":memory:".to_string(),
            ..Default::default()
        };

        let db_manager = Arc::new(DatabaseManager::new(config).await.unwrap());
        let embedding_engine = Arc::new(EmbeddingEngine::new(EmbeddingConfig::default()).await.unwrap());
        let search_engine = SemanticSearchEngine::new(db_manager, embedding_engine);

        // 初始化搜索引擎
        search_engine.initialize().await.unwrap();

        // 添加一些测试文档
        let documents = vec![
            Document {
                id: "doc1".to_string(),
                title: "人工智能".to_string(),
                content: "人工智能是计算机科学的一个分支".to_string(),
                doc_type: DocumentType::Note,
                metadata: serde_json::json!({}),
            },
            Document {
                id: "doc2".to_string(),
                title: "机器学习".to_string(),
                content: "机器学习是人工智能的重要组成部分".to_string(),
                doc_type: DocumentType::Note,
                metadata: serde_json::json!({}),
            },
        ];

        // 批量添加文档
        let result = search_engine.add_documents_batch(&documents).await;
        assert!(result.is_ok());

        // 测试搜索
        let results = search_engine.search("人工智能技术", 10).await;
        assert!(results.is_ok());

        // 验证文档数量
        let count = search_engine.get_document_count().await.unwrap();
        assert_eq!(count, 2);

        // 测试索引信息
        let indexes = search_engine.get_index_info().unwrap();
        assert!(!indexes.is_empty());

        // 测试搜索统计
        let stats = search_engine.get_search_stats();
        assert!(stats.total_searches > 0);
    }

    #[tokio::test]
    async fn test_add_document() {
        let config = ZhushoudeConfig {
            database_path: ":memory:".to_string(),
            ..Default::default()
        };

        let db_manager = Arc::new(DatabaseManager::new(config).await.unwrap());
        let embedding_engine = Arc::new(EmbeddingEngine::new(EmbeddingConfig::default()).await.unwrap());
        let search_engine = SemanticSearchEngine::new(db_manager, embedding_engine);

        // 初始化搜索引擎
        search_engine.initialize().await.unwrap();

        let document = Document {
            id: "test1".to_string(),
            title: "测试文档".to_string(),
            content: "这是测试内容".to_string(),
            doc_type: DocumentType::Note,
            metadata: serde_json::json!({}),
        };

        let result = search_engine.add_document(&document).await;
        assert!(result.is_ok());

        // 验证文档已添加
        let count = search_engine.get_document_count().await.unwrap();
        assert_eq!(count, 1);

        // 测试删除文档
        let delete_result = search_engine.delete_document("test1").await;
        assert!(delete_result.is_ok());

        let count_after = search_engine.get_document_count().await.unwrap();
        assert_eq!(count_after, 0);
    }

    #[tokio::test]
    async fn test_similar_documents() {
        let config = ZhushoudeConfig {
            database_path: ":memory:".to_string(),
            ..Default::default()
        };

        let db_manager = Arc::new(DatabaseManager::new(config).await.unwrap());
        let embedding_engine = Arc::new(EmbeddingEngine::new(EmbeddingConfig::default()).await.unwrap());
        let search_engine = SemanticSearchEngine::new(db_manager, embedding_engine);

        // 添加相关文档
        let documents = vec![
            Document {
                id: "ai1".to_string(),
                title: "人工智能基础".to_string(),
                content: "人工智能是模拟人类智能的技术".to_string(),
                doc_type: DocumentType::Note,
                metadata: serde_json::json!({}),
            },
            Document {
                id: "ai2".to_string(),
                title: "机器学习算法".to_string(),
                content: "机器学习是实现人工智能的重要方法".to_string(),
                doc_type: DocumentType::Note,
                metadata: serde_json::json!({}),
            },
            Document {
                id: "other".to_string(),
                title: "天气预报".to_string(),
                content: "今天天气晴朗，适合外出".to_string(),
                doc_type: DocumentType::Note,
                metadata: serde_json::json!({}),
            },
        ];

        search_engine.add_documents_batch(&documents).await.unwrap();

        // 查找与AI相关文档的相似文档
        let similar = search_engine.find_similar_documents("ai1", 2).await.unwrap();
        assert!(!similar.is_empty());

        // 删除文档测试
        search_engine.delete_document("other").await.unwrap();
        let count = search_engine.get_document_count().await.unwrap();
        assert_eq!(count, 2);
    }
}
