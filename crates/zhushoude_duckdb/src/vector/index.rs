//! 向量索引模块

use crate::{Result, DatabaseManager};
use std::sync::Arc;
use std::collections::HashMap;

/// 向量索引类型
#[derive(Debug, Clone)]
pub enum IndexType {
    /// 线性扫描（适合小数据集）
    Linear,
    /// 基于哈希的近似索引
    Hash { num_buckets: usize },
    /// 基于聚类的索引
    Cluster { num_clusters: usize },
    /// 自适应索引（根据数据量自动选择）
    Adaptive,
}

/// 向量索引管理器
pub struct VectorIndexManager {
    db_manager: Arc<DatabaseManager>,
    indexes: HashMap<String, IndexMetadata>,
}

/// 索引元数据
#[derive(Debug, Clone)]
pub struct IndexMetadata {
    pub name: String,
    pub table_name: String,
    pub column_name: String,
    pub index_type: IndexType,
    pub dimension: usize,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_updated: chrono::DateTime<chrono::Utc>,
    pub num_vectors: usize,
}

impl VectorIndexManager {
    /// 创建新的索引管理器
    pub fn new(db_manager: Arc<DatabaseManager>) -> Self {
        Self {
            db_manager,
            indexes: HashMap::new(),
        }
    }

    /// 初始化索引管理器
    pub async fn initialize(&mut self) -> Result<()> {
        // 创建索引元数据表
        self.create_index_metadata_table().await?;

        // 加载现有索引
        self.load_existing_indexes().await?;

        Ok(())
    }

    /// 创建索引元数据表
    async fn create_index_metadata_table(&self) -> Result<()> {
        let sql = r#"
            CREATE TABLE IF NOT EXISTS vector_indexes (
                name VARCHAR PRIMARY KEY,
                table_name VARCHAR NOT NULL,
                column_name VARCHAR NOT NULL,
                index_type VARCHAR NOT NULL,
                dimension INTEGER NOT NULL,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                last_updated TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                num_vectors INTEGER DEFAULT 0,
                metadata JSON
            )
        "#;

        let conn = self.db_manager.get_connection();
        let conn = conn.lock()
            .map_err(|e| crate::Error::DatabaseError(format!("获取数据库连接失败: {}", e)))?;

        conn.execute(sql, duckdb::params![])
            .map_err(|e| crate::Error::DatabaseError(format!("创建索引元数据表失败: {}", e)))?;

        Ok(())
    }

    /// 加载现有索引
    async fn load_existing_indexes(&mut self) -> Result<()> {
        let sql = "SELECT name, table_name, column_name, index_type, dimension, created_at, last_updated, num_vectors FROM vector_indexes";

        let conn = self.db_manager.get_connection();
        let conn = conn.lock()
            .map_err(|e| crate::Error::DatabaseError(format!("获取数据库连接失败: {}", e)))?;

        let mut stmt = conn.prepare(sql)
            .map_err(|e| crate::Error::DatabaseError(format!("准备SQL语句失败: {}", e)))?;

        let rows = stmt.query_map(duckdb::params![], |row| {
            let index_type_str: String = row.get(3)?;
            let index_type = match index_type_str.as_str() {
                "Linear" => IndexType::Linear,
                "Adaptive" => IndexType::Adaptive,
                _ => IndexType::Linear, // 默认
            };

            // 安全处理时间戳字段
            let created_at = match row.get::<_, String>(5) {
                Ok(time_str) => {
                    chrono::DateTime::parse_from_rfc3339(&time_str)
                        .map(|dt| dt.with_timezone(&chrono::Utc))
                        .unwrap_or_else(|_| chrono::Utc::now())
                }
                Err(_) => chrono::Utc::now(),
            };

            let last_updated = match row.get::<_, String>(6) {
                Ok(time_str) => {
                    chrono::DateTime::parse_from_rfc3339(&time_str)
                        .map(|dt| dt.with_timezone(&chrono::Utc))
                        .unwrap_or_else(|_| chrono::Utc::now())
                }
                Err(_) => chrono::Utc::now(),
            };

            Ok(IndexMetadata {
                name: row.get(0)?,
                table_name: row.get(1)?,
                column_name: row.get(2)?,
                index_type,
                dimension: row.get::<_, i32>(4)? as usize,
                created_at,
                last_updated,
                num_vectors: row.get::<_, i32>(7)? as usize,
            })
        }).map_err(|e| crate::Error::DatabaseError(format!("查询索引失败: {}", e)))?;

        for row in rows {
            let metadata = row.map_err(|e| crate::Error::DatabaseError(format!("处理索引数据失败: {}", e)))?;
            self.indexes.insert(metadata.name.clone(), metadata);
        }

        println!("✅ 加载了 {} 个现有索引", self.indexes.len());
        Ok(())
    }

    /// 创建向量索引
    pub async fn create_index(&mut self, table_name: &str, column_name: &str, index_type: IndexType, dimension: usize) -> Result<String> {
        let index_name = format!("idx_{}_{}", table_name, column_name);

        // 检查索引是否已存在
        if self.indexes.contains_key(&index_name) {
            return Err(crate::Error::DatabaseError(format!("索引已存在: {}", index_name)));
        }

        // 根据索引类型创建相应的索引结构
        match &index_type {
            IndexType::Linear => {
                self.create_linear_index(table_name, column_name, &index_name).await?;
            }
            IndexType::Hash { num_buckets } => {
                self.create_hash_index(table_name, column_name, &index_name, *num_buckets).await?;
            }
            IndexType::Cluster { num_clusters } => {
                self.create_cluster_index(table_name, column_name, &index_name, *num_clusters).await?;
            }
            IndexType::Adaptive => {
                // 根据数据量自动选择索引类型
                let count = self.get_vector_count(table_name, column_name).await?;
                let adaptive_type = if count < 1000 {
                    IndexType::Linear
                } else if count < 10000 {
                    IndexType::Hash { num_buckets: 100 }
                } else {
                    IndexType::Cluster { num_clusters: 50 }
                };

                // 递归调用，但使用Box::pin避免无限大小的future
                return Box::pin(self.create_index(table_name, column_name, adaptive_type, dimension)).await;
            }
        }

        // 保存索引元数据
        let metadata = IndexMetadata {
            name: index_name.clone(),
            table_name: table_name.to_string(),
            column_name: column_name.to_string(),
            index_type: index_type.clone(),
            dimension,
            created_at: chrono::Utc::now(),
            last_updated: chrono::Utc::now(),
            num_vectors: self.get_vector_count(table_name, column_name).await?,
        };

        self.save_index_metadata(&metadata).await?;
        self.indexes.insert(index_name.clone(), metadata);

        println!("✅ 成功创建向量索引: {}", index_name);
        Ok(index_name)
    }

    /// 创建线性索引（实际上是优化的查询策略）
    async fn create_linear_index(&self, table_name: &str, column_name: &str, index_name: &str) -> Result<()> {
        // 为线性扫描创建优化的查询视图
        let view_sql = format!(
            "CREATE OR REPLACE VIEW {}_view AS
             SELECT *, len({}) as vector_dim
             FROM {}
             WHERE {} IS NOT NULL",
            index_name, column_name, table_name, column_name
        );

        let conn = self.db_manager.get_connection();
        let conn = conn.lock()
            .map_err(|e| crate::Error::DatabaseError(format!("获取数据库连接失败: {}", e)))?;

        conn.execute(&view_sql, duckdb::params![])
            .map_err(|e| crate::Error::DatabaseError(format!("创建线性索引视图失败: {}", e)))?;

        Ok(())
    }

    /// 创建哈希索引
    async fn create_hash_index(&self, table_name: &str, column_name: &str, index_name: &str, num_buckets: usize) -> Result<()> {
        // 创建哈希桶表
        let hash_table_sql = format!(
            "CREATE TABLE {}_hash AS
             SELECT *,
                    hash({}) % {} as bucket_id
             FROM {}
             WHERE {} IS NOT NULL",
            index_name, column_name, num_buckets, table_name, column_name
        );

        let conn = self.db_manager.get_connection();
        let conn = conn.lock()
            .map_err(|e| crate::Error::DatabaseError(format!("获取数据库连接失败: {}", e)))?;

        conn.execute(&hash_table_sql, duckdb::params![])
            .map_err(|e| crate::Error::DatabaseError(format!("创建哈希索引表失败: {}", e)))?;

        // 在桶ID上创建索引
        let bucket_index_sql = format!("CREATE INDEX {}_bucket_idx ON {}_hash(bucket_id)", index_name, index_name);
        conn.execute(&bucket_index_sql, duckdb::params![])
            .map_err(|e| crate::Error::DatabaseError(format!("创建桶索引失败: {}", e)))?;

        Ok(())
    }

    /// 创建聚类索引
    async fn create_cluster_index(&self, table_name: &str, column_name: &str, index_name: &str, num_clusters: usize) -> Result<()> {
        // 使用简单的K-means聚类（这里使用随机分配作为简化实现）
        let cluster_table_sql = format!(
            "CREATE TABLE {}_clusters AS
             SELECT *,
                    (hash({}) % {}) as cluster_id
             FROM {}
             WHERE {} IS NOT NULL",
            index_name, column_name, num_clusters, table_name, column_name
        );

        let conn = self.db_manager.get_connection();
        let conn = conn.lock()
            .map_err(|e| crate::Error::DatabaseError(format!("获取数据库连接失败: {}", e)))?;

        conn.execute(&cluster_table_sql, duckdb::params![])
            .map_err(|e| crate::Error::DatabaseError(format!("创建聚类索引表失败: {}", e)))?;

        // 在聚类ID上创建索引
        let cluster_index_sql = format!("CREATE INDEX {}_cluster_idx ON {}_clusters(cluster_id)", index_name, index_name);
        conn.execute(&cluster_index_sql, duckdb::params![])
            .map_err(|e| crate::Error::DatabaseError(format!("创建聚类索引失败: {}", e)))?;

        Ok(())
    }

    /// 获取向量数量
    async fn get_vector_count(&self, table_name: &str, column_name: &str) -> Result<usize> {
        let sql = format!("SELECT COUNT(*) FROM {} WHERE {} IS NOT NULL", table_name, column_name);

        let conn = self.db_manager.get_connection();
        let conn = conn.lock()
            .map_err(|e| crate::Error::DatabaseError(format!("获取数据库连接失败: {}", e)))?;

        let count: i64 = conn.query_row(&sql, duckdb::params![], |row| row.get(0))
            .map_err(|e| crate::Error::DatabaseError(format!("查询向量数量失败: {}", e)))?;

        Ok(count as usize)
    }

    /// 保存索引元数据
    async fn save_index_metadata(&self, metadata: &IndexMetadata) -> Result<()> {
        let sql = r#"
            INSERT INTO vector_indexes
            (name, table_name, column_name, index_type, dimension, created_at, last_updated, num_vectors)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        "#;

        let conn = self.db_manager.get_connection();
        let conn = conn.lock()
            .map_err(|e| crate::Error::DatabaseError(format!("获取数据库连接失败: {}", e)))?;

        let index_type_str = match metadata.index_type {
            IndexType::Linear => "Linear",
            IndexType::Hash { .. } => "Hash",
            IndexType::Cluster { .. } => "Cluster",
            IndexType::Adaptive => "Adaptive",
        };

        conn.execute(sql, duckdb::params![
            metadata.name,
            metadata.table_name,
            metadata.column_name,
            index_type_str,
            metadata.dimension as i32,
            metadata.created_at.to_rfc3339(),
            metadata.last_updated.to_rfc3339(),
            metadata.num_vectors as i32
        ]).map_err(|e| crate::Error::DatabaseError(format!("保存索引元数据失败: {}", e)))?;

        Ok(())
    }

    /// 删除索引
    pub async fn drop_index(&mut self, index_name: &str) -> Result<()> {
        if let Some(metadata) = self.indexes.get(index_name) {
            let index_type = metadata.index_type.clone();

            // 根据索引类型删除相应的结构
            match index_type {
                IndexType::Linear => {
                    let view_sql = format!("DROP VIEW IF EXISTS {}_view", index_name);
                    self.execute_sql(&view_sql).await?;
                }
                IndexType::Hash { .. } => {
                    let table_sql = format!("DROP TABLE IF EXISTS {}_hash", index_name);
                    self.execute_sql(&table_sql).await?;
                }
                IndexType::Cluster { .. } => {
                    let table_sql = format!("DROP TABLE IF EXISTS {}_clusters", index_name);
                    self.execute_sql(&table_sql).await?;
                }
                IndexType::Adaptive => {
                    // 自适应索引可能创建了不同类型的结构，都尝试删除
                    let _ = self.execute_sql(&format!("DROP VIEW IF EXISTS {}_view", index_name)).await;
                    let _ = self.execute_sql(&format!("DROP TABLE IF EXISTS {}_hash", index_name)).await;
                    let _ = self.execute_sql(&format!("DROP TABLE IF EXISTS {}_clusters", index_name)).await;
                }
            }

            // 从元数据表中删除
            let delete_sql = "DELETE FROM vector_indexes WHERE name = ?";
            let conn = self.db_manager.get_connection();
            let conn = conn.lock()
                .map_err(|e| crate::Error::DatabaseError(format!("获取数据库连接失败: {}", e)))?;

            conn.execute(delete_sql, duckdb::params![index_name])
                .map_err(|e| crate::Error::DatabaseError(format!("删除索引元数据失败: {}", e)))?;

            // 从内存中删除
            self.indexes.remove(index_name);

            println!("✅ 成功删除索引: {}", index_name);
        }

        Ok(())
    }

    /// 执行SQL语句的辅助方法
    async fn execute_sql(&self, sql: &str) -> Result<()> {
        let conn = self.db_manager.get_connection();
        let conn = conn.lock()
            .map_err(|e| crate::Error::DatabaseError(format!("获取数据库连接失败: {}", e)))?;

        conn.execute(sql, duckdb::params![])
            .map_err(|e| crate::Error::DatabaseError(format!("执行SQL失败: {}", e)))?;

        Ok(())
    }

    /// 使用索引进行向量搜索
    pub async fn search_with_index(&self, index_name: &str, query_vector: &[f32], limit: usize, threshold: f32) -> Result<Vec<VectorSearchResult>> {
        let metadata = self.indexes.get(index_name)
            .ok_or_else(|| crate::Error::DatabaseError(format!("索引不存在: {}", index_name)))?;

        match &metadata.index_type {
            IndexType::Linear => {
                self.linear_search(&metadata.table_name, &metadata.column_name, query_vector, limit, threshold).await
            }
            IndexType::Hash { num_buckets } => {
                self.hash_search(index_name, query_vector, limit, threshold, *num_buckets).await
            }
            IndexType::Cluster { num_clusters } => {
                self.cluster_search(index_name, query_vector, limit, threshold, *num_clusters).await
            }
            IndexType::Adaptive => {
                // 对于自适应索引，根据当前数据量选择最佳搜索策略
                if metadata.num_vectors < 1000 {
                    self.linear_search(&metadata.table_name, &metadata.column_name, query_vector, limit, threshold).await
                } else {
                    self.cluster_search(index_name, query_vector, limit, threshold, 50).await
                }
            }
        }
    }

    /// 线性搜索
    async fn linear_search(&self, table_name: &str, column_name: &str, query_vector: &[f32], limit: usize, threshold: f32) -> Result<Vec<VectorSearchResult>> {
        let query_vector_str = format!("[{}]",
            query_vector.iter()
                .map(|f| f.to_string())
                .collect::<Vec<_>>()
                .join(",")
        );

        let sql = format!(
            "SELECT *, cosine_similarity({}, CAST('{}' AS FLOAT[])) as similarity
             FROM {}
             WHERE {} IS NOT NULL
             AND cosine_similarity({}, CAST('{}' AS FLOAT[])) >= {}
             ORDER BY similarity DESC
             LIMIT {}",
            column_name, query_vector_str, table_name, column_name,
            column_name, query_vector_str, threshold, limit
        );

        self.execute_search_query(&sql).await
    }

    /// 哈希搜索
    async fn hash_search(&self, index_name: &str, query_vector: &[f32], limit: usize, threshold: f32, num_buckets: usize) -> Result<Vec<VectorSearchResult>> {
        // 计算查询向量的哈希桶
        let query_hash = self.compute_vector_hash(query_vector, num_buckets);

        // 搜索相同桶和邻近桶
        let mut bucket_ids = vec![query_hash];
        if query_hash > 0 {
            bucket_ids.push(query_hash - 1);
        }
        if query_hash < num_buckets - 1 {
            bucket_ids.push(query_hash + 1);
        }

        let bucket_list = bucket_ids.iter()
            .map(|id| id.to_string())
            .collect::<Vec<_>>()
            .join(",");

        let query_vector_str = format!("[{}]",
            query_vector.iter()
                .map(|f| f.to_string())
                .collect::<Vec<_>>()
                .join(",")
        );

        let sql = format!(
            "SELECT *, cosine_similarity({}, CAST('{}' AS FLOAT[])) as similarity
             FROM {}_hash
             WHERE bucket_id IN ({})
             AND cosine_similarity({}, CAST('{}' AS FLOAT[])) >= {}
             ORDER BY similarity DESC
             LIMIT {}",
            index_name.replace("idx_", ""), query_vector_str, index_name,
            bucket_list,
            index_name.replace("idx_", ""), query_vector_str, threshold, limit
        );

        self.execute_search_query(&sql).await
    }

    /// 聚类搜索
    async fn cluster_search(&self, index_name: &str, query_vector: &[f32], limit: usize, threshold: f32, num_clusters: usize) -> Result<Vec<VectorSearchResult>> {
        // 计算查询向量的聚类ID
        let cluster_id = self.compute_vector_hash(query_vector, num_clusters);

        let query_vector_str = format!("[{}]",
            query_vector.iter()
                .map(|f| f.to_string())
                .collect::<Vec<_>>()
                .join(",")
        );

        let sql = format!(
            "SELECT *, cosine_similarity({}, CAST('{}' AS FLOAT[])) as similarity
             FROM {}_clusters
             WHERE cluster_id = {}
             AND cosine_similarity({}, CAST('{}' AS FLOAT[])) >= {}
             ORDER BY similarity DESC
             LIMIT {}",
            index_name.replace("idx_", ""), query_vector_str, index_name,
            cluster_id,
            index_name.replace("idx_", ""), query_vector_str, threshold, limit
        );

        self.execute_search_query(&sql).await
    }

    /// 计算向量哈希值
    fn compute_vector_hash(&self, vector: &[f32], num_buckets: usize) -> usize {
        let mut hash = 0u64;
        for (i, &val) in vector.iter().enumerate() {
            hash = hash.wrapping_add((val.to_bits() as u64).wrapping_mul((i + 1) as u64));
        }
        (hash % num_buckets as u64) as usize
    }

    /// 执行搜索查询
    async fn execute_search_query(&self, sql: &str) -> Result<Vec<VectorSearchResult>> {
        let conn = self.db_manager.get_connection();
        let conn = conn.lock()
            .map_err(|e| crate::Error::DatabaseError(format!("获取数据库连接失败: {}", e)))?;

        let mut stmt = conn.prepare(sql)
            .map_err(|e| crate::Error::DatabaseError(format!("准备搜索查询失败: {}", e)))?;

        let rows = stmt.query_map(duckdb::params![], |row| {
            Ok(VectorSearchResult {
                id: row.get::<_, String>(0)?,
                similarity: row.get::<_, f64>("similarity")?,
                metadata: None, // 可以根据需要扩展
            })
        }).map_err(|e| crate::Error::DatabaseError(format!("执行搜索查询失败: {}", e)))?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row.map_err(|e| crate::Error::DatabaseError(format!("处理搜索结果失败: {}", e)))?);
        }

        Ok(results)
    }

    /// 重建索引
    pub async fn rebuild_index(&mut self, index_name: &str) -> Result<()> {
        if let Some(metadata) = self.indexes.get(index_name).cloned() {
            // 删除旧索引
            self.drop_index(index_name).await?;

            // 重新创建索引
            self.create_index(&metadata.table_name, &metadata.column_name, metadata.index_type, metadata.dimension).await?;

            println!("✅ 成功重建索引: {}", index_name);
        } else {
            return Err(crate::Error::DatabaseError(format!("索引不存在: {}", index_name)));
        }

        Ok(())
    }

    /// 获取索引信息
    pub fn get_index_info(&self, index_name: &str) -> Option<&IndexMetadata> {
        self.indexes.get(index_name)
    }

    /// 更新索引统计信息
    pub async fn update_index_stats(&mut self, index_name: &str) -> Result<()> {
        // 先获取表名和列名，避免借用冲突
        let (table_name, column_name) = if let Some(metadata) = self.indexes.get(index_name) {
            (metadata.table_name.clone(), metadata.column_name.clone())
        } else {
            return Ok(());
        };

        // 获取向量数量
        let num_vectors = self.get_vector_count(&table_name, &column_name).await?;
        let now = chrono::Utc::now();

        // 更新内存中的元数据
        if let Some(metadata) = self.indexes.get_mut(index_name) {
            metadata.num_vectors = num_vectors;
            metadata.last_updated = now;
        }

        // 更新数据库中的元数据
        let sql = "UPDATE vector_indexes SET num_vectors = ?, last_updated = ? WHERE name = ?";
        let conn = self.db_manager.get_connection();
        let conn = conn.lock()
            .map_err(|e| crate::Error::DatabaseError(format!("获取数据库连接失败: {}", e)))?;

        conn.execute(sql, duckdb::params![
            num_vectors as i32,
            now.to_rfc3339(),
            index_name
        ]).map_err(|e| crate::Error::DatabaseError(format!("更新索引统计失败: {}", e)))?;

        println!("✅ 更新索引统计: {} ({} 个向量)", index_name, num_vectors);
        Ok(())
    }

    /// 列出所有向量索引
    pub fn list_indexes(&self) -> Vec<String> {
        self.indexes.keys().cloned().collect()
    }

    /// 获取所有索引的详细信息
    pub fn get_all_indexes(&self) -> Vec<&IndexMetadata> {
        self.indexes.values().collect()
    }

    /// 检查索引是否存在
    pub fn index_exists(&self, index_name: &str) -> bool {
        self.indexes.contains_key(index_name)
    }

    /// 获取索引统计信息
    pub async fn get_index_stats(&self, index_name: &str) -> Result<IndexStats> {
        let metadata = self.indexes.get(index_name)
            .ok_or_else(|| crate::Error::DatabaseError(format!("索引不存在: {}", index_name)))?;

        // 计算索引大小（简化实现）
        let size_bytes = self.estimate_index_size(index_name, metadata).await?;

        Ok(IndexStats {
            size_bytes,
            num_vectors: metadata.num_vectors,
            index_type: format!("{:?}", metadata.index_type),
            dimension: metadata.dimension,
            last_updated: metadata.last_updated,
            search_performance: SearchPerformanceStats {
                avg_search_time_ms: 10.0, // 占位符
                total_searches: 0,
                cache_hit_rate: 0.0,
            },
        })
    }

    /// 估算索引大小
    async fn estimate_index_size(&self, index_name: &str, metadata: &IndexMetadata) -> Result<usize> {
        // 基础大小：向量数量 * 维度 * 4字节（float32）
        let base_size = metadata.num_vectors * metadata.dimension * 4;

        // 根据索引类型添加额外开销
        let overhead = match metadata.index_type {
            IndexType::Linear => base_size / 10, // 10% 开销
            IndexType::Hash { .. } => base_size / 5, // 20% 开销
            IndexType::Cluster { .. } => base_size / 3, // 33% 开销
            IndexType::Adaptive => base_size / 4, // 25% 开销
        };

        Ok(base_size + overhead)
    }

    /// 优化索引
    pub async fn optimize_index(&mut self, index_name: &str) -> Result<()> {
        if let Some(metadata) = self.indexes.get(index_name) {
            let table_name = metadata.table_name.clone();
            let column_name = metadata.column_name.clone();

            // 执行表分析
            let analyze_sql = format!("ANALYZE {}", table_name);
            self.execute_sql(&analyze_sql).await?;

            // 根据索引类型执行特定优化
            match &metadata.index_type {
                IndexType::Hash { .. } => {
                    let optimize_sql = format!("ANALYZE {}_hash", index_name);
                    self.execute_sql(&optimize_sql).await?;
                }
                IndexType::Cluster { .. } => {
                    let optimize_sql = format!("ANALYZE {}_clusters", index_name);
                    self.execute_sql(&optimize_sql).await?;
                }
                _ => {
                    // 对于线性和自适应索引，只需要分析主表
                }
            }

            // 更新统计信息
            self.update_index_stats(index_name).await?;

            println!("✅ 成功优化索引: {}", index_name);
        } else {
            return Err(crate::Error::DatabaseError(format!("索引不存在: {}", index_name)));
        }

        Ok(())
    }
}



/// 向量搜索结果
#[derive(Debug, Clone)]
pub struct VectorSearchResult {
    pub id: String,
    pub similarity: f64,
    pub metadata: Option<serde_json::Value>,
}

/// 索引统计信息
#[derive(Debug, Clone)]
pub struct IndexStats {
    pub size_bytes: usize,
    pub num_vectors: usize,
    pub index_type: String,
    pub dimension: usize,
    pub last_updated: chrono::DateTime<chrono::Utc>,
    pub search_performance: SearchPerformanceStats,
}

/// 搜索性能统计
#[derive(Debug, Clone)]
pub struct SearchPerformanceStats {
    pub avg_search_time_ms: f64,
    pub total_searches: u64,
    pub cache_hit_rate: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ZhushoudeConfig;

    #[tokio::test]
    async fn test_vector_index_manager() {
        let config = ZhushoudeConfig {
            database_path: ":memory:".to_string(),
            ..Default::default()
        };

        let db_manager = Arc::new(DatabaseManager::new(config).await.unwrap());
        let mut manager = VectorIndexManager::new(db_manager);

        // 初始化管理器
        manager.initialize().await.unwrap();

        // 测试创建线性索引
        let index_name = manager.create_index(
            "document_embeddings",
            "embedding",
            IndexType::Linear,
            512
        ).await.unwrap();

        assert_eq!(index_name, "idx_document_embeddings_embedding");

        // 测试列出索引
        let indexes = manager.list_indexes();
        assert!(indexes.contains(&index_name));

        // 测试获取索引信息
        let info = manager.get_index_info(&index_name);
        assert!(info.is_some());
        assert_eq!(info.unwrap().dimension, 512);

        // 测试删除索引
        manager.drop_index(&index_name).await.unwrap();
        let indexes_after = manager.list_indexes();
        assert!(!indexes_after.contains(&index_name));
    }

    #[tokio::test]
    async fn test_adaptive_index() {
        let config = ZhushoudeConfig {
            database_path: ":memory:".to_string(),
            ..Default::default()
        };

        let db_manager = Arc::new(DatabaseManager::new(config).await.unwrap());
        let mut manager = VectorIndexManager::new(db_manager);

        manager.initialize().await.unwrap();

        // 测试自适应索引（应该选择线性索引，因为数据量小）
        let index_name = manager.create_index(
            "document_embeddings",
            "embedding",
            IndexType::Adaptive,
            256
        ).await.unwrap();

        let info = manager.get_index_info(&index_name).unwrap();
        // 由于数据量小，自适应索引应该选择线性类型
        assert!(matches!(info.index_type, IndexType::Linear));

        manager.drop_index(&index_name).await.unwrap();
    }

    #[tokio::test]
    async fn test_index_stats() {
        let config = ZhushoudeConfig {
            database_path: ":memory:".to_string(),
            ..Default::default()
        };

        let db_manager = Arc::new(DatabaseManager::new(config).await.unwrap());
        let mut manager = VectorIndexManager::new(db_manager);

        manager.initialize().await.unwrap();

        // 创建索引
        let index_name = manager.create_index(
            "document_embeddings",
            "embedding",
            IndexType::Hash { num_buckets: 10 },
            128
        ).await.unwrap();

        // 测试获取索引统计
        let stats = manager.get_index_stats(&index_name).await.unwrap();
        assert_eq!(stats.dimension, 128);
        assert_eq!(stats.index_type, "Hash { num_buckets: 10 }");

        // 测试不存在的索引
        let result = manager.get_index_stats("nonexistent_index").await;
        assert!(result.is_err());

        manager.drop_index(&index_name).await.unwrap();
    }
}
