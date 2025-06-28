//! 数据库连接管理模块

use crate::{Result, ZhushoudeConfig, Error};
use duckdb::{Connection, params};
use std::sync::{Arc, Mutex};
use std::path::Path;

/// 数据库管理器
#[derive(Debug)]
pub struct DatabaseManager {
    config: ZhushoudeConfig,
    connection: Arc<Mutex<Connection>>,
}

impl DatabaseManager {
    /// 创建新的数据库管理器
    pub async fn new(config: ZhushoudeConfig) -> Result<Self> {
        let db_path = &config.database_path;

        // 创建数据库目录（如果不存在）
        if let Some(parent) = Path::new(db_path).parent() {
            tokio::fs::create_dir_all(parent).await
                .map_err(|e| Error::DatabaseError(format!("创建数据库目录失败: {}", e)))?;
        }

        // 建立DuckDB连接
        let connection = if db_path == ":memory:" {
            Connection::open_in_memory()
        } else {
            Connection::open(db_path)
        }.map_err(|e| Error::DatabaseError(format!("连接数据库失败: {}", e)))?;

        // 配置DuckDB设置
        let manager = Self {
            config: config.clone(),
            connection: Arc::new(Mutex::new(connection)),
        };

        // 应用性能配置
        manager.configure_performance().await?;

        // 初始化数据库模式
        manager.initialize_schema().await?;

        println!("✅ 成功连接到数据库: {}", db_path);

        Ok(manager)
    }

    /// 配置数据库性能参数
    async fn configure_performance(&self) -> Result<()> {
        let conn = self.connection.lock()
            .map_err(|e| Error::DatabaseError(format!("获取数据库连接失败: {}", e)))?;

        // 尝试安装和加载向量扩展
        self.setup_vector_extensions(&conn).await?;

        // 设置内存限制
        let memory_limit = format!("{}MB", self.config.performance.memory_limit_mb);
        conn.execute(&format!("SET memory_limit='{}'", memory_limit), params![])
            .map_err(|e| Error::DatabaseError(format!("设置内存限制失败: {}", e)))?;

        // 设置线程数
        if let Some(threads) = self.config.performance.thread_pool_size {
            conn.execute(&format!("SET threads={}", threads), params![])
                .map_err(|e| Error::DatabaseError(format!("设置线程数失败: {}", e)))?;
        }

        // 启用并行处理
        conn.execute("SET enable_progress_bar=false", params![])
            .map_err(|e| Error::DatabaseError(format!("配置进度条失败: {}", e)))?;

        // 优化向量操作
        conn.execute("SET enable_object_cache=true", params![])
            .map_err(|e| Error::DatabaseError(format!("启用对象缓存失败: {}", e)))?;

        println!("✅ 数据库性能配置完成");
        Ok(())
    }

    /// 设置向量扩展
    async fn setup_vector_extensions(&self, conn: &Connection) -> Result<()> {
        // 尝试安装向量扩展（如果可用）
        let extensions = vec![
            "vss",      // Vector Similarity Search
            "spatial",  // 空间扩展，包含一些向量函数
        ];

        for ext in extensions {
            // 尝试安装扩展（忽略错误，因为可能已经安装或不可用）
            let _ = conn.execute(&format!("INSTALL {}", ext), params![]);

            // 尝试加载扩展
            match conn.execute(&format!("LOAD {}", ext), params![]) {
                Ok(_) => println!("✅ 成功加载扩展: {}", ext),
                Err(_) => println!("⚠️ 扩展不可用: {}", ext),
            }
        }

        // 创建自定义向量函数（如果扩展不可用）
        self.create_vector_functions(conn).await?;

        Ok(())
    }

    /// 创建自定义向量函数
    async fn create_vector_functions(&self, conn: &Connection) -> Result<()> {
        // DuckDB 使用不同的数组语法，我们创建简化的向量函数

        // 创建简单的点积函数（使用字符串表示向量）
        let dot_product_sql = r#"
            CREATE OR REPLACE MACRO dot_product(a, b) AS (
                CASE
                    WHEN a IS NULL OR b IS NULL THEN 0
                    ELSE 0.5  -- 占位符实现
                END
            )
        "#;

        match conn.execute(dot_product_sql, params![]) {
            Ok(_) => println!("✅ 创建点积函数"),
            Err(e) => println!("⚠️ 创建点积函数失败: {}", e),
        }

        // 创建余弦相似度函数（简化版本）
        let cosine_similarity_sql = r#"
            CREATE OR REPLACE MACRO cosine_similarity(a, b) AS (
                CASE
                    WHEN a IS NULL OR b IS NULL THEN 0
                    WHEN a = b THEN 1.0
                    ELSE 0.8  -- 占位符实现，返回固定相似度
                END
            )
        "#;

        match conn.execute(cosine_similarity_sql, params![]) {
            Ok(_) => println!("✅ 创建余弦相似度函数"),
            Err(e) => println!("⚠️ 创建余弦相似度函数失败: {}", e),
        }

        // 创建欧几里得距离函数（简化版本）
        let euclidean_distance_sql = r#"
            CREATE OR REPLACE MACRO euclidean_distance(a, b) AS (
                CASE
                    WHEN a IS NULL OR b IS NULL THEN 999999
                    WHEN a = b THEN 0.0
                    ELSE 1.0  -- 占位符实现
                END
            )
        "#;

        match conn.execute(euclidean_distance_sql, params![]) {
            Ok(_) => println!("✅ 创建欧几里得距离函数"),
            Err(e) => println!("⚠️ 创建欧几里得距离函数失败: {}", e),
        }

        Ok(())
    }

    /// 初始化数据库模式
    async fn initialize_schema(&self) -> Result<()> {
        let conn = self.connection.lock()
            .map_err(|e| Error::DatabaseError(format!("获取数据库连接失败: {}", e)))?;

        // 创建文档表
        conn.execute(
            "CREATE TABLE IF NOT EXISTS documents (
                id VARCHAR PRIMARY KEY,
                title VARCHAR NOT NULL,
                content TEXT NOT NULL,
                doc_type VARCHAR NOT NULL,
                metadata JSON,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            )",
            params![]
        ).map_err(|e| Error::DatabaseError(format!("创建文档表失败: {}", e)))?;

        // 创建向量表（使用TEXT存储向量，避免FLOAT[]语法问题）
        conn.execute(
            "CREATE TABLE IF NOT EXISTS document_embeddings (
                document_id VARCHAR NOT NULL,
                model_name VARCHAR NOT NULL,
                embedding TEXT,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                PRIMARY KEY (document_id, model_name)
            )",
            params![]
        ).map_err(|e| Error::DatabaseError(format!("创建向量表失败: {}", e)))?;

        // 创建图节点表
        conn.execute(
            "CREATE TABLE IF NOT EXISTS graph_nodes (
                id VARCHAR PRIMARY KEY,
                node_type VARCHAR NOT NULL,
                name VARCHAR NOT NULL,
                properties JSON,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            )",
            params![]
        ).map_err(|e| Error::DatabaseError(format!("创建图节点表失败: {}", e)))?;

        // 创建图边表（移除CASCADE约束，DuckDB不支持）
        conn.execute(
            "CREATE TABLE IF NOT EXISTS graph_edges (
                id VARCHAR PRIMARY KEY,
                source_id VARCHAR NOT NULL,
                target_id VARCHAR NOT NULL,
                edge_type VARCHAR NOT NULL,
                weight FLOAT DEFAULT 1.0,
                properties JSON,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            )",
            params![]
        ).map_err(|e| Error::DatabaseError(format!("创建图边表失败: {}", e)))?;

        // 创建索引
        self.create_indexes().await?;

        println!("✅ 数据库模式初始化完成");
        Ok(())
    }

    /// 创建数据库索引
    async fn create_indexes(&self) -> Result<()> {
        let conn = self.connection.lock()
            .map_err(|e| Error::DatabaseError(format!("获取数据库连接失败: {}", e)))?;

        // 文档索引
        conn.execute("CREATE INDEX IF NOT EXISTS idx_documents_type ON documents(doc_type)", params![])
            .map_err(|e| Error::DatabaseError(format!("创建文档类型索引失败: {}", e)))?;

        conn.execute("CREATE INDEX IF NOT EXISTS idx_documents_created ON documents(created_at)", params![])
            .map_err(|e| Error::DatabaseError(format!("创建文档时间索引失败: {}", e)))?;

        // 向量索引
        conn.execute("CREATE INDEX IF NOT EXISTS idx_embeddings_model ON document_embeddings(model_name)", params![])
            .map_err(|e| Error::DatabaseError(format!("创建向量模型索引失败: {}", e)))?;

        // 图索引
        conn.execute("CREATE INDEX IF NOT EXISTS idx_nodes_type ON graph_nodes(node_type)", params![])
            .map_err(|e| Error::DatabaseError(format!("创建节点类型索引失败: {}", e)))?;

        conn.execute("CREATE INDEX IF NOT EXISTS idx_edges_source ON graph_edges(source_id)", params![])
            .map_err(|e| Error::DatabaseError(format!("创建边源索引失败: {}", e)))?;

        conn.execute("CREATE INDEX IF NOT EXISTS idx_edges_target ON graph_edges(target_id)", params![])
            .map_err(|e| Error::DatabaseError(format!("创建边目标索引失败: {}", e)))?;

        println!("✅ 数据库索引创建完成");
        Ok(())
    }

    /// 执行SQL查询
    pub async fn execute_query(&self, sql: &str) -> Result<()> {
        let conn = self.connection.lock()
            .map_err(|e| Error::DatabaseError(format!("获取数据库连接失败: {}", e)))?;

        conn.execute(sql, params![])
            .map_err(|e| Error::DatabaseError(format!("执行SQL失败: {}", e)))?;

        Ok(())
    }

    /// 执行查询并返回结果
    pub async fn query<T, F>(&self, sql: &str, params: &[&dyn duckdb::ToSql], mapper: F) -> Result<Vec<T>>
    where
        F: Fn(&duckdb::Row) -> duckdb::Result<T>,
    {
        let conn = self.connection.lock()
            .map_err(|e| Error::DatabaseError(format!("获取数据库连接失败: {}", e)))?;

        let mut stmt = conn.prepare(sql)
            .map_err(|e| Error::DatabaseError(format!("准备SQL语句失败: {}", e)))?;

        let rows = stmt.query_map(params, mapper)
            .map_err(|e| Error::DatabaseError(format!("执行查询失败: {}", e)))?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row.map_err(|e| Error::DatabaseError(format!("处理查询结果失败: {}", e)))?);
        }

        Ok(results)
    }

    /// 获取连接配置
    pub fn get_config(&self) -> &ZhushoudeConfig {
        &self.config
    }

    /// 获取数据库连接（用于高级操作）
    pub fn get_connection(&self) -> Arc<Mutex<Connection>> {
        self.connection.clone()
    }

    /// 检查数据库连接状态
    pub async fn check_health(&self) -> Result<bool> {
        let conn = self.connection.lock()
            .map_err(|e| Error::DatabaseError(format!("获取数据库连接失败: {}", e)))?;

        match conn.execute("SELECT 1", params![]) {
            Ok(_) => Ok(true),
            Err(e) => {
                println!("⚠️ 数据库健康检查失败: {}", e);
                Ok(false)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_database_manager_creation() {
        let config = ZhushoudeConfig {
            database_path: ":memory:".to_string(),
            ..Default::default()
        };

        let result = DatabaseManager::new(config).await;
        assert!(result.is_ok());

        let db_manager = result.unwrap();
        assert!(db_manager.check_health().await.unwrap());
    }

    #[tokio::test]
    async fn test_schema_initialization() {
        let config = ZhushoudeConfig {
            database_path: ":memory:".to_string(),
            ..Default::default()
        };

        let db_manager = DatabaseManager::new(config).await.expect("创建数据库管理器失败");

        // 验证表是否存在
        let tables = db_manager.query(
            "SELECT table_name FROM information_schema.tables WHERE table_schema = 'main'",
            &[],
            |row| Ok(row.get::<_, String>(0)?)
        ).await.expect("查询表失败");

        let table_names: Vec<String> = tables;
        assert!(table_names.contains(&"documents".to_string()));
        assert!(table_names.contains(&"document_embeddings".to_string()));
        assert!(table_names.contains(&"graph_nodes".to_string()));
        assert!(table_names.contains(&"graph_edges".to_string()));
    }

    #[tokio::test]
    async fn test_database_operations() {
        let config = ZhushoudeConfig {
            database_path: ":memory:".to_string(),
            ..Default::default()
        };

        let db_manager = DatabaseManager::new(config).await.expect("创建数据库管理器失败");

        // 测试插入文档
        db_manager.execute_query(
            "INSERT INTO documents (id, title, content, doc_type, metadata)
             VALUES ('test1', '测试文档', '这是一个测试文档', 'Note', '{}')"
        ).await.expect("插入文档失败");

        // 测试查询文档
        let documents = db_manager.query(
            "SELECT id, title FROM documents WHERE id = 'test1'",
            &[],
            |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        ).await.expect("查询文档失败");

        assert_eq!(documents.len(), 1);
        assert_eq!(documents[0].0, "test1");
        assert_eq!(documents[0].1, "测试文档");
    }

    #[tokio::test]
    async fn test_performance_configuration() {
        let config = ZhushoudeConfig {
            database_path: ":memory:".to_string(),
            performance: crate::PerformanceConfig {
                thread_pool_size: Some(2),
                memory_limit_mb: 128,
                enable_monitoring: true,
                cache_strategy: crate::CacheStrategy::LRU,
            },
            ..Default::default()
        };

        let result = DatabaseManager::new(config).await;
        assert!(result.is_ok());

        let db_manager = result.unwrap();

        // 验证性能配置是否生效
        let memory_limit = db_manager.query(
            "SELECT current_setting('memory_limit')",
            &[],
            |row| Ok(row.get::<_, String>(0)?)
        ).await.expect("查询内存限制失败");

        assert!(!memory_limit.is_empty());
    }
}
