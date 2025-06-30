use crate::{Result, Error, TextChunk, ChunkingConfig, TextChunker};
use crate::database::DatabaseManager;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::collections::HashMap;
use duckdb::params;

/// 分块向量索引配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkVectorConfig {
    /// 分块配置
    pub chunking_config: ChunkingConfig,
    /// 向量维度
    pub vector_dimension: usize,
    /// 索引类型
    pub index_type: ChunkIndexType,
    /// 批处理大小
    pub batch_size: usize,
    /// 是否启用增量索引
    pub incremental_indexing: bool,
    /// 索引更新阈值
    pub update_threshold: usize,
}

impl Default for ChunkVectorConfig {
    fn default() -> Self {
        Self {
            chunking_config: ChunkingConfig::default(),
            vector_dimension: 384, // 常见的轻量级模型维度
            index_type: ChunkIndexType::HNSW,
            batch_size: 100,
            incremental_indexing: true,
            update_threshold: 1000,
        }
    }
}

/// 分块索引类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChunkIndexType {
    /// 层次化可导航小世界图
    HNSW,
    /// 倒排文件索引
    IVF,
    /// 平坦索引（暴力搜索）
    Flat,
    /// 产品量化索引
    PQ,
}

/// 分块向量索引
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkVectorIndex {
    /// 索引ID
    pub id: String,
    /// 文档ID
    pub document_id: String,
    /// 分块ID
    pub chunk_id: String,
    /// 向量数据
    pub vector: Vec<f32>,
    /// 向量范数
    pub norm: f32,
    /// 创建时间
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// 更新时间
    pub updated_at: chrono::DateTime<chrono::Utc>,
    /// 索引元数据
    pub metadata: ChunkIndexMetadata,
}

/// 分块索引元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkIndexMetadata {
    /// 模型名称
    pub model_name: String,
    /// 向量维度
    pub dimension: usize,
    /// 索引类型
    pub index_type: String,
    /// 质量分数
    pub quality_score: f32,
    /// 语言
    pub language: String,
    /// 内容类型
    pub content_type: String,
    /// 额外属性
    pub properties: HashMap<String, String>,
}

/// 分块向量索引管理器
pub struct ChunkVectorIndexManager {
    db_manager: Arc<DatabaseManager>,
    config: ChunkVectorConfig,
    chunker: TextChunker,
}

impl ChunkVectorIndexManager {
    /// 创建新的分块向量索引管理器
    pub fn new(db_manager: Arc<DatabaseManager>, config: ChunkVectorConfig) -> Self {
        let chunker = TextChunker::new(config.chunking_config.clone());
        Self {
            db_manager,
            config,
            chunker,
        }
    }

    /// 使用默认配置创建管理器
    pub fn default(db_manager: Arc<DatabaseManager>) -> Self {
        Self::new(db_manager, ChunkVectorConfig::default())
    }

    /// 初始化分块向量索引表
    pub async fn initialize_tables(&self) -> Result<()> {
        let connection = self.db_manager.get_connection();
        let conn = connection.lock()
            .map_err(|e| Error::DatabaseError(format!("Failed to get connection: {}", e)))?;

        // 创建分块表
        conn.execute(
            "CREATE TABLE IF NOT EXISTS text_chunks (
                id TEXT PRIMARY KEY,
                document_id TEXT NOT NULL,
                chunk_index INTEGER NOT NULL,
                content TEXT NOT NULL,
                start_offset INTEGER NOT NULL,
                end_offset INTEGER NOT NULL,
                char_count INTEGER NOT NULL,
                word_count INTEGER NOT NULL,
                sentence_count INTEGER NOT NULL,
                paragraph_count INTEGER NOT NULL,
                language TEXT NOT NULL,
                content_type TEXT NOT NULL,
                quality_score REAL NOT NULL,
                properties TEXT, -- JSON
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            )",
            params![],
        )?;

        // 创建分块向量索引表
        conn.execute(
            "CREATE TABLE IF NOT EXISTS chunk_vector_indexes (
                id TEXT PRIMARY KEY,
                document_id TEXT NOT NULL,
                chunk_id TEXT NOT NULL,
                vector REAL[], -- DuckDB数组类型
                norm REAL NOT NULL,
                model_name TEXT NOT NULL,
                dimension INTEGER NOT NULL,
                index_type TEXT NOT NULL,
                quality_score REAL NOT NULL,
                language TEXT NOT NULL,
                content_type TEXT NOT NULL,
                properties TEXT, -- JSON
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (chunk_id) REFERENCES text_chunks(id)
            )",
            params![],
        )?;

        // 创建索引（如果不会阻塞的话）
        if self.should_create_indexes() {
            self.create_chunk_indexes().await?;
        }

        println!("✅ 分块向量索引表初始化完成");
        Ok(())
    }

    /// 判断是否应该创建索引
    fn should_create_indexes(&self) -> bool {
        // 在内存数据库中创建索引，文件数据库中跳过
        let db_path = &self.db_manager.get_config().database_path;
        let should_create = db_path == ":memory:";
        println!("🔍 数据库路径: '{}', 是否创建索引: {}", db_path, should_create);
        should_create
    }

    /// 创建分块索引
    async fn create_chunk_indexes(&self) -> Result<()> {
        let connection = self.db_manager.get_connection();
        let conn = connection.lock()
            .map_err(|e| Error::DatabaseError(format!("Failed to get connection: {}", e)))?;

        // 分块表索引
        conn.execute("CREATE INDEX IF NOT EXISTS idx_chunks_document ON text_chunks(document_id)", params![])?;
        conn.execute("CREATE INDEX IF NOT EXISTS idx_chunks_language ON text_chunks(language)", params![])?;
        conn.execute("CREATE INDEX IF NOT EXISTS idx_chunks_content_type ON text_chunks(content_type)", params![])?;
        conn.execute("CREATE INDEX IF NOT EXISTS idx_chunks_quality ON text_chunks(quality_score)", params![])?;

        // 向量索引表索引
        conn.execute("CREATE INDEX IF NOT EXISTS idx_vector_document ON chunk_vector_indexes(document_id)", params![])?;
        conn.execute("CREATE INDEX IF NOT EXISTS idx_vector_chunk ON chunk_vector_indexes(chunk_id)", params![])?;
        conn.execute("CREATE INDEX IF NOT EXISTS idx_vector_model ON chunk_vector_indexes(model_name)", params![])?;
        conn.execute("CREATE INDEX IF NOT EXISTS idx_vector_type ON chunk_vector_indexes(index_type)", params![])?;

        println!("✅ 分块索引创建完成");
        Ok(())
    }

    /// 对文档进行分块并创建向量索引
    pub async fn index_document(&self, document_id: &str, content: &str, model_name: &str) -> Result<Vec<ChunkVectorIndex>> {
        // 1. 文本分块
        let chunks = self.chunker.chunk_text(document_id, content)?;
        println!("📄 文档 {} 分块完成: {} 个分块", document_id, chunks.len());

        // 2. 存储分块
        self.store_chunks(&chunks).await?;

        // 3. 生成向量并创建索引
        let mut vector_indexes = Vec::new();
        for chunk in chunks {
            // 这里应该调用实际的向量化模型，暂时使用占位符
            let vector = self.generate_chunk_vector(&chunk.content, model_name).await?;
            let norm = self.calculate_vector_norm(&vector);

            let index = ChunkVectorIndex {
                id: format!("{}_{}_vector", chunk.document_id, chunk.chunk_index),
                document_id: chunk.document_id.clone(),
                chunk_id: chunk.id.clone(),
                vector,
                norm,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
                metadata: ChunkIndexMetadata {
                    model_name: model_name.to_string(),
                    dimension: self.config.vector_dimension,
                    index_type: format!("{:?}", self.config.index_type),
                    quality_score: chunk.metadata.quality_score,
                    language: chunk.metadata.language,
                    content_type: chunk.metadata.content_type,
                    properties: HashMap::new(),
                },
            };

            vector_indexes.push(index);
        }

        // 4. 批量存储向量索引
        self.store_vector_indexes(&vector_indexes).await?;

        println!("✅ 文档 {} 向量索引创建完成: {} 个向量", document_id, vector_indexes.len());
        Ok(vector_indexes)
    }

    /// 存储文本分块
    async fn store_chunks(&self, chunks: &[TextChunk]) -> Result<()> {
        let connection = self.db_manager.get_connection();
        let conn = connection.lock()
            .map_err(|e| Error::DatabaseError(format!("Failed to get connection: {}", e)))?;

        for chunk in chunks {
            let properties_json = serde_json::to_string(&chunk.metadata.properties)
                .unwrap_or_else(|_| "{}".to_string());

            conn.execute(
                "INSERT OR REPLACE INTO text_chunks 
                (id, document_id, chunk_index, content, start_offset, end_offset, 
                 char_count, word_count, sentence_count, paragraph_count, 
                 language, content_type, quality_score, properties) 
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
                params![
                    chunk.id,
                    chunk.document_id,
                    chunk.chunk_index as i32,
                    chunk.content,
                    chunk.start_offset as i32,
                    chunk.end_offset as i32,
                    chunk.metadata.char_count as i32,
                    chunk.metadata.word_count as i32,
                    chunk.metadata.sentence_count as i32,
                    chunk.metadata.paragraph_count as i32,
                    chunk.metadata.language,
                    chunk.metadata.content_type,
                    chunk.metadata.quality_score,
                    properties_json,
                ],
            )?;
        }

        Ok(())
    }

    /// 存储向量索引
    async fn store_vector_indexes(&self, indexes: &[ChunkVectorIndex]) -> Result<()> {
        let connection = self.db_manager.get_connection();
        let conn = connection.lock()
            .map_err(|e| Error::DatabaseError(format!("Failed to get connection: {}", e)))?;

        for index in indexes {
            let properties_json = serde_json::to_string(&index.metadata.properties)
                .unwrap_or_else(|_| "{}".to_string());

            // 将向量转换为DuckDB数组格式
            let vector_array = format!("[{}]", index.vector.iter()
                .map(|v| v.to_string())
                .collect::<Vec<_>>()
                .join(","));

            conn.execute(
                "INSERT OR REPLACE INTO chunk_vector_indexes 
                (id, document_id, chunk_id, vector, norm, model_name, dimension, 
                 index_type, quality_score, language, content_type, properties) 
                VALUES (?, ?, ?, ?::REAL[], ?, ?, ?, ?, ?, ?, ?, ?)",
                params![
                    index.id,
                    index.document_id,
                    index.chunk_id,
                    vector_array,
                    index.norm,
                    index.metadata.model_name,
                    index.metadata.dimension as i32,
                    index.metadata.index_type,
                    index.metadata.quality_score,
                    index.metadata.language,
                    index.metadata.content_type,
                    properties_json,
                ],
            )?;
        }

        Ok(())
    }

    /// 生成分块向量（占位符实现）
    async fn generate_chunk_vector(&self, content: &str, _model_name: &str) -> Result<Vec<f32>> {
        // TODO: 集成真实的向量化模型
        // 这里使用简单的哈希向量作为占位符
        let mut vector = vec![0.0; self.config.vector_dimension];
        let content_hash = self.simple_hash(content);

        for (i, val) in vector.iter_mut().enumerate() {
            *val = ((content_hash.wrapping_add(i) % 1000) as f32 - 500.0) / 500.0;
        }

        // 归一化向量
        let norm = self.calculate_vector_norm(&vector);
        if norm > 0.0 {
            for val in vector.iter_mut() {
                *val /= norm;
            }
        }

        Ok(vector)
    }

    /// 简单哈希函数
    fn simple_hash(&self, content: &str) -> usize {
        let mut hash = 0usize;
        for byte in content.bytes() {
            hash = hash.wrapping_mul(31).wrapping_add(byte as usize);
        }
        hash
    }

    /// 计算向量范数
    fn calculate_vector_norm(&self, vector: &[f32]) -> f32 {
        vector.iter().map(|x| x * x).sum::<f32>().sqrt()
    }

    /// 分块语义搜索
    pub async fn search_chunks(&self, query: &str, model_name: &str, limit: usize) -> Result<Vec<ChunkSearchResult>> {
        // 1. 生成查询向量
        let query_vector = self.generate_chunk_vector(query, model_name).await?;

        // 2. 执行向量相似度搜索
        let connection = self.db_manager.get_connection();
        let conn = connection.lock()
            .map_err(|e| Error::DatabaseError(format!("Failed to get connection: {}", e)))?;

        // 使用简化的查询，避免复杂的向量函数
        let mut stmt = conn.prepare(
            "SELECT
                cvi.id, cvi.document_id, cvi.chunk_id, cvi.vector, cvi.norm,
                cvi.model_name, cvi.quality_score, cvi.language, cvi.content_type,
                tc.content, tc.char_count, tc.start_offset, tc.end_offset
             FROM chunk_vector_indexes cvi
             JOIN text_chunks tc ON cvi.chunk_id = tc.id
             WHERE cvi.model_name = ?
             LIMIT ?"
        )?;

        let rows = stmt.query_map(
            params![model_name, limit as i32],
            |row| {
                let vector_str: String = row.get(3)?;
                let vector = self.parse_vector_array(&vector_str).unwrap_or_default();

                Ok(ChunkSearchResult {
                    chunk_id: row.get(2)?,
                    document_id: row.get(1)?,
                    content: row.get(9)?,
                    similarity_score: self.calculate_cosine_similarity(&query_vector, &vector),
                    start_offset: row.get(11)?,
                    end_offset: row.get(12)?,
                    char_count: row.get(10)?,
                    quality_score: row.get(6)?,
                    language: row.get(7)?,
                    content_type: row.get(8)?,
                })
            }
        )?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }

        // 按相似度排序
        results.sort_by(|a, b| b.similarity_score.partial_cmp(&a.similarity_score).unwrap_or(std::cmp::Ordering::Equal));

        Ok(results)
    }

    /// 解析向量数组字符串
    fn parse_vector_array(&self, vector_str: &str) -> Result<Vec<f32>> {
        // 移除方括号并分割
        let cleaned = vector_str.trim_start_matches('[').trim_end_matches(']');
        if cleaned.is_empty() {
            return Ok(vec![0.0; self.config.vector_dimension]);
        }

        let values: std::result::Result<Vec<f32>, std::num::ParseFloatError> = cleaned
            .split(',')
            .map(|s| s.trim().parse::<f32>())
            .collect();

        values.map_err(|e| Error::DatabaseError(format!("Failed to parse vector: {}", e)))
    }

    /// 计算余弦相似度
    fn calculate_cosine_similarity(&self, vec1: &[f32], vec2: &[f32]) -> f32 {
        if vec1.len() != vec2.len() || vec1.is_empty() {
            return 0.0;
        }

        let dot_product: f32 = vec1.iter().zip(vec2.iter()).map(|(a, b)| a * b).sum();
        let norm1 = self.calculate_vector_norm(vec1);
        let norm2 = self.calculate_vector_norm(vec2);

        if norm1 > 0.0 && norm2 > 0.0 {
            dot_product / (norm1 * norm2)
        } else {
            0.0
        }
    }

    /// 获取文档的所有分块
    pub async fn get_document_chunks(&self, document_id: &str) -> Result<Vec<TextChunk>> {
        let connection = self.db_manager.get_connection();
        let conn = connection.lock()
            .map_err(|e| Error::DatabaseError(format!("Failed to get connection: {}", e)))?;

        let mut stmt = conn.prepare(
            "SELECT id, document_id, chunk_index, content, start_offset, end_offset,
                    char_count, word_count, sentence_count, paragraph_count,
                    language, content_type, quality_score, properties
             FROM text_chunks
             WHERE document_id = ?
             ORDER BY chunk_index"
        )?;

        let rows = stmt.query_map(params![document_id], |row| {
            let properties_json: String = row.get(13)?;
            let properties: HashMap<String, String> = serde_json::from_str(&properties_json)
                .unwrap_or_default();

            Ok(TextChunk {
                id: row.get(0)?,
                document_id: row.get(1)?,
                chunk_index: row.get(2)?,
                content: row.get(3)?,
                start_offset: row.get(4)?,
                end_offset: row.get(5)?,
                metadata: crate::text::ChunkMetadata {
                    char_count: row.get(6)?,
                    word_count: row.get(7)?,
                    sentence_count: row.get(8)?,
                    paragraph_count: row.get(9)?,
                    language: row.get(10)?,
                    content_type: row.get(11)?,
                    quality_score: row.get(12)?,
                    properties,
                },
            })
        })?;

        let mut chunks = Vec::new();
        for row in rows {
            chunks.push(row?);
        }

        Ok(chunks)
    }

    /// 删除文档的所有分块和向量索引
    pub async fn delete_document(&self, document_id: &str) -> Result<()> {
        let connection = self.db_manager.get_connection();
        let conn = connection.lock()
            .map_err(|e| Error::DatabaseError(format!("Failed to get connection: {}", e)))?;

        // 删除向量索引
        conn.execute(
            "DELETE FROM chunk_vector_indexes WHERE document_id = ?",
            params![document_id],
        )?;

        // 删除分块
        conn.execute(
            "DELETE FROM text_chunks WHERE document_id = ?",
            params![document_id],
        )?;

        println!("🗑️ 文档 {} 的分块和向量索引已删除", document_id);
        Ok(())
    }

    /// 获取索引统计信息
    pub async fn get_index_stats(&self) -> Result<ChunkIndexStats> {
        let connection = self.db_manager.get_connection();
        let conn = connection.lock()
            .map_err(|e| Error::DatabaseError(format!("Failed to get connection: {}", e)))?;

        let mut stmt = conn.prepare(
            "SELECT
                COUNT(*) as total_chunks,
                COUNT(DISTINCT document_id) as total_documents,
                AVG(char_count) as avg_chunk_size,
                MIN(char_count) as min_chunk_size,
                MAX(char_count) as max_chunk_size,
                AVG(quality_score) as avg_quality_score
             FROM text_chunks"
        )?;

        let row = stmt.query_row(params![], |row| {
            Ok(ChunkIndexStats {
                total_chunks: row.get(0)?,
                total_documents: row.get(1)?,
                avg_chunk_size: row.get(2)?,
                min_chunk_size: row.get(3)?,
                max_chunk_size: row.get(4)?,
                avg_quality_score: row.get(5)?,
            })
        })?;

        Ok(row)
    }
}

/// 分块搜索结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkSearchResult {
    pub chunk_id: String,
    pub document_id: String,
    pub content: String,
    pub similarity_score: f32,
    pub start_offset: usize,
    pub end_offset: usize,
    pub char_count: usize,
    pub quality_score: f32,
    pub language: String,
    pub content_type: String,
}

/// 分块索引统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkIndexStats {
    pub total_chunks: usize,
    pub total_documents: usize,
    pub avg_chunk_size: f32,
    pub min_chunk_size: usize,
    pub max_chunk_size: usize,
    pub avg_quality_score: f32,
}
