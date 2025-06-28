//! 向量存储模块

use crate::{Result, DatabaseManager};
use std::sync::Arc;

/// 向量存储管理器
pub struct VectorStorage {
    db_manager: Arc<DatabaseManager>,
}

impl VectorStorage {
    /// 创建新的向量存储管理器
    pub fn new(db_manager: Arc<DatabaseManager>) -> Self {
        Self { db_manager }
    }
    
    /// 存储文档向量
    pub async fn store_vector(&self, document_id: &str, embedding: &[f32], model_name: &str) -> Result<()> {
        // 将向量转换为DuckDB数组格式
        let embedding_array = format!("[{}]",
            embedding.iter()
                .map(|f| f.to_string())
                .collect::<Vec<_>>()
                .join(",")
        );

        let sql = "INSERT OR REPLACE INTO document_embeddings (document_id, model_name, embedding) VALUES (?, ?, ?)";

        let conn = self.db_manager.get_connection();
        let conn = conn.lock()
            .map_err(|e| crate::Error::DatabaseError(format!("获取数据库连接失败: {}", e)))?;

        conn.execute(sql, duckdb::params![document_id, model_name, embedding_array])
            .map_err(|e| crate::Error::DatabaseError(format!("存储向量失败: {}", e)))?;

        Ok(())
    }

    /// 获取文档向量
    pub async fn get_vector(&self, document_id: &str, model_name: &str) -> Result<Option<Vec<f32>>> {
        let sql = "SELECT embedding FROM document_embeddings WHERE document_id = ? AND model_name = ?";

        let conn = self.db_manager.get_connection();
        let conn = conn.lock()
            .map_err(|e| crate::Error::DatabaseError(format!("获取数据库连接失败: {}", e)))?;

        let mut stmt = conn.prepare(sql)
            .map_err(|e| crate::Error::DatabaseError(format!("准备SQL语句失败: {}", e)))?;

        let mut rows = stmt.query(duckdb::params![document_id, model_name])
            .map_err(|e| crate::Error::DatabaseError(format!("查询向量失败: {}", e)))?;

        if let Some(row) = rows.next()
            .map_err(|e| crate::Error::DatabaseError(format!("读取查询结果失败: {}", e)))? {

            let embedding_str: String = row.get(0)
                .map_err(|e| crate::Error::DatabaseError(format!("获取向量数据失败: {}", e)))?;

            // 解析向量字符串 "[1.0,2.0,3.0]" -> Vec<f32>
            let embedding = self.parse_vector_string(&embedding_str)?;
            Ok(Some(embedding))
        } else {
            Ok(None)
        }
    }

    /// 删除文档向量
    pub async fn delete_vector(&self, document_id: &str, model_name: &str) -> Result<()> {
        let sql = "DELETE FROM document_embeddings WHERE document_id = ? AND model_name = ?";

        let conn = self.db_manager.get_connection();
        let conn = conn.lock()
            .map_err(|e| crate::Error::DatabaseError(format!("获取数据库连接失败: {}", e)))?;

        conn.execute(sql, duckdb::params![document_id, model_name])
            .map_err(|e| crate::Error::DatabaseError(format!("删除向量失败: {}", e)))?;

        Ok(())
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

    /// 批量存储向量
    pub async fn store_vectors_batch(&self, vectors: &[(String, Vec<f32>, String)]) -> Result<()> {
        let conn = self.db_manager.get_connection();
        let conn = conn.lock()
            .map_err(|e| crate::Error::DatabaseError(format!("获取数据库连接失败: {}", e)))?;

        // 开始事务
        conn.execute("BEGIN TRANSACTION", duckdb::params![])
            .map_err(|e| crate::Error::DatabaseError(format!("开始事务失败: {}", e)))?;

        let sql = "INSERT OR REPLACE INTO document_embeddings (document_id, model_name, embedding) VALUES (?, ?, ?)";
        let mut stmt = conn.prepare(sql)
            .map_err(|e| crate::Error::DatabaseError(format!("准备SQL语句失败: {}", e)))?;

        for (document_id, embedding, model_name) in vectors {
            let embedding_array = format!("[{}]",
                embedding.iter()
                    .map(|f| f.to_string())
                    .collect::<Vec<_>>()
                    .join(",")
            );

            stmt.execute(duckdb::params![document_id, model_name, embedding_array])
                .map_err(|e| crate::Error::DatabaseError(format!("批量存储向量失败: {}", e)))?;
        }

        // 提交事务
        conn.execute("COMMIT", duckdb::params![])
            .map_err(|e| crate::Error::DatabaseError(format!("提交事务失败: {}", e)))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ZhushoudeConfig;
    use tempfile::NamedTempFile;
    
    #[tokio::test]
    async fn test_vector_storage() {
        let config = ZhushoudeConfig {
            database_path: ":memory:".to_string(),
            ..Default::default()
        };

        let db_manager = Arc::new(DatabaseManager::new(config).await.unwrap());
        let storage = VectorStorage::new(db_manager);

        let embedding = vec![1.0, 2.0, 3.0];

        // 测试存储向量
        let result = storage.store_vector("doc1", &embedding, "test_model").await;
        assert!(result.is_ok());

        // 测试获取向量
        let retrieved = storage.get_vector("doc1", "test_model").await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap(), embedding);

        // 测试删除向量
        let result = storage.delete_vector("doc1", "test_model").await;
        assert!(result.is_ok());

        // 验证删除后获取不到向量
        let retrieved = storage.get_vector("doc1", "test_model").await.unwrap();
        assert!(retrieved.is_none());
    }

    #[tokio::test]
    async fn test_batch_vector_storage() {
        let config = ZhushoudeConfig {
            database_path: ":memory:".to_string(),
            ..Default::default()
        };

        let db_manager = Arc::new(DatabaseManager::new(config).await.unwrap());
        let storage = VectorStorage::new(db_manager);

        let vectors = vec![
            ("doc1".to_string(), vec![1.0, 2.0, 3.0], "model1".to_string()),
            ("doc2".to_string(), vec![4.0, 5.0, 6.0], "model1".to_string()),
            ("doc3".to_string(), vec![7.0, 8.0, 9.0], "model2".to_string()),
        ];

        // 测试批量存储
        let result = storage.store_vectors_batch(&vectors).await;
        assert!(result.is_ok());

        // 验证所有向量都已存储
        for (doc_id, expected_embedding, model_name) in &vectors {
            let retrieved = storage.get_vector(doc_id, model_name).await.unwrap();
            assert!(retrieved.is_some());
            assert_eq!(retrieved.unwrap(), *expected_embedding);
        }
    }

    #[tokio::test]
    async fn test_vector_string_parsing() {
        let config = ZhushoudeConfig {
            database_path: ":memory:".to_string(),
            ..Default::default()
        };

        let db_manager = Arc::new(DatabaseManager::new(config).await.unwrap());
        let storage = VectorStorage::new(db_manager);

        // 测试正常向量字符串解析
        let vector_str = "[1.5,2.7,3.14]";
        let parsed = storage.parse_vector_string(vector_str).unwrap();
        assert_eq!(parsed, vec![1.5, 2.7, 3.14]);

        // 测试空向量
        let empty_str = "[]";
        let parsed = storage.parse_vector_string(empty_str).unwrap();
        assert_eq!(parsed, Vec::<f32>::new());

        // 测试带空格的向量字符串
        let spaced_str = "[ 1.0 , 2.0 , 3.0 ]";
        let parsed = storage.parse_vector_string(spaced_str).unwrap();
        assert_eq!(parsed, vec![1.0, 2.0, 3.0]);
    }
}
