//! 图数据存储模块

use crate::{Result, DatabaseManager, GraphNode, GraphEdge};
use std::sync::Arc;

/// 图存储管理器
pub struct GraphStorage {
    db_manager: Arc<DatabaseManager>,
}

impl GraphStorage {
    /// 创建新的图存储管理器
    pub fn new(db_manager: Arc<DatabaseManager>) -> Self {
        Self { db_manager }
    }
    
    /// 添加图节点
    pub async fn add_node(&self, node: &GraphNode) -> Result<()> {
        let sql = "INSERT OR REPLACE INTO graph_nodes (id, node_type, name, properties) VALUES (?, ?, ?, ?)";

        let conn = self.db_manager.get_connection();
        let conn = conn.lock()
            .map_err(|e| crate::Error::DatabaseError(format!("获取数据库连接失败: {}", e)))?;

        let properties_json = serde_json::to_string(&node.properties)
            .map_err(|e| crate::Error::DatabaseError(format!("序列化节点属性失败: {}", e)))?;

        conn.execute(sql, duckdb::params![
            node.id,
            format!("{:?}", node.node_type),
            node.label,
            properties_json
        ]).map_err(|e| crate::Error::DatabaseError(format!("添加节点失败: {}", e)))?;

        Ok(())
    }

    /// 添加图边
    pub async fn add_edge(&self, edge: &GraphEdge) -> Result<()> {
        let sql = "INSERT OR REPLACE INTO graph_edges (id, source_id, target_id, edge_type, weight, properties) VALUES (?, ?, ?, ?, ?, ?)";

        let conn = self.db_manager.get_connection();
        let conn = conn.lock()
            .map_err(|e| crate::Error::DatabaseError(format!("获取数据库连接失败: {}", e)))?;

        let properties_json = serde_json::to_string(&edge.properties)
            .map_err(|e| crate::Error::DatabaseError(format!("序列化边属性失败: {}", e)))?;

        conn.execute(sql, duckdb::params![
            edge.id,
            edge.source_id,
            edge.target_id,
            format!("{:?}", edge.edge_type),
            edge.weight,
            properties_json
        ]).map_err(|e| crate::Error::DatabaseError(format!("添加边失败: {}", e)))?;

        Ok(())
    }

    /// 获取节点
    pub async fn get_node(&self, node_id: &str) -> Result<Option<GraphNode>> {
        let sql = "SELECT id, node_type, name, properties FROM graph_nodes WHERE id = ?";

        let conn = self.db_manager.get_connection();
        let conn = conn.lock()
            .map_err(|e| crate::Error::DatabaseError(format!("获取数据库连接失败: {}", e)))?;

        let mut stmt = conn.prepare(sql)
            .map_err(|e| crate::Error::DatabaseError(format!("准备SQL语句失败: {}", e)))?;

        let mut rows = stmt.query(duckdb::params![node_id])
            .map_err(|e| crate::Error::DatabaseError(format!("查询节点失败: {}", e)))?;

        if let Some(row) = rows.next()
            .map_err(|e| crate::Error::DatabaseError(format!("读取查询结果失败: {}", e)))? {

            let properties_str: String = row.get(3)
                .map_err(|e| crate::Error::DatabaseError(format!("获取节点属性失败: {}", e)))?;

            let properties = serde_json::from_str(&properties_str)
                .map_err(|e| crate::Error::DatabaseError(format!("反序列化节点属性失败: {}", e)))?;

            let node_type_str: String = row.get(1)
                .map_err(|e| crate::Error::DatabaseError(format!("获取节点类型失败: {}", e)))?;

            let node_type = match node_type_str.as_str() {
                "Class" => crate::NodeType::Class,
                "Method" => crate::NodeType::Method,
                "Variable" => crate::NodeType::Variable,
                "Package" => crate::NodeType::Package,
                _ => crate::NodeType::Class, // 默认值
            };

            let node = GraphNode {
                id: row.get(0).map_err(|e| crate::Error::DatabaseError(format!("获取节点ID失败: {}", e)))?,
                node_type,
                label: row.get(2).map_err(|e| crate::Error::DatabaseError(format!("获取节点名称失败: {}", e)))?,
                properties,
            };

            Ok(Some(node))
        } else {
            Ok(None)
        }
    }

    /// 获取边
    pub async fn get_edges(&self, source_id: &str) -> Result<Vec<GraphEdge>> {
        let sql = "SELECT id, source_id, target_id, edge_type, weight, properties FROM graph_edges WHERE source_id = ?";

        let conn = self.db_manager.get_connection();
        let conn = conn.lock()
            .map_err(|e| crate::Error::DatabaseError(format!("获取数据库连接失败: {}", e)))?;

        let mut stmt = conn.prepare(sql)
            .map_err(|e| crate::Error::DatabaseError(format!("准备SQL语句失败: {}", e)))?;

        let rows = stmt.query_map(duckdb::params![source_id], |row| {
            let properties_str: String = row.get(5)?;
            let properties = serde_json::from_str(&properties_str)
                .map_err(|e| duckdb::Error::FromSqlConversionFailure(
                    5, duckdb::types::Type::Text, Box::new(e)
                ))?;

            let edge_type_str: String = row.get(3)?;
            let edge_type = match edge_type_str.as_str() {
                "DependsOn" => crate::EdgeType::DependsOn,
                "Calls" => crate::EdgeType::Calls,
                "Inherits" => crate::EdgeType::Inherits,
                "Uses" => crate::EdgeType::DependsOn, // 映射到现有类型
                _ => crate::EdgeType::DependsOn, // 默认值
            };

            Ok(GraphEdge {
                id: row.get(0)?,
                source_id: row.get(1)?,
                target_id: row.get(2)?,
                edge_type,
                weight: row.get(4)?,
                properties,
            })
        }).map_err(|e| crate::Error::DatabaseError(format!("查询边失败: {}", e)))?;

        let mut edges = Vec::new();
        for row in rows {
            edges.push(row.map_err(|e| crate::Error::DatabaseError(format!("处理边数据失败: {}", e)))?);
        }

        Ok(edges)
    }

    /// 删除节点
    pub async fn delete_node(&self, node_id: &str) -> Result<()> {
        let conn = self.db_manager.get_connection();
        let conn = conn.lock()
            .map_err(|e| crate::Error::DatabaseError(format!("获取数据库连接失败: {}", e)))?;

        // 开始事务
        conn.execute("BEGIN TRANSACTION", duckdb::params![])
            .map_err(|e| crate::Error::DatabaseError(format!("开始事务失败: {}", e)))?;

        // 删除节点（边会因为外键约束自动删除）
        conn.execute("DELETE FROM graph_nodes WHERE id = ?", duckdb::params![node_id])
            .map_err(|e| crate::Error::DatabaseError(format!("删除节点失败: {}", e)))?;

        // 提交事务
        conn.execute("COMMIT", duckdb::params![])
            .map_err(|e| crate::Error::DatabaseError(format!("提交事务失败: {}", e)))?;

        Ok(())
    }

    /// 删除边
    pub async fn delete_edge(&self, edge_id: &str) -> Result<()> {
        let sql = "DELETE FROM graph_edges WHERE id = ?";

        let conn = self.db_manager.get_connection();
        let conn = conn.lock()
            .map_err(|e| crate::Error::DatabaseError(format!("获取数据库连接失败: {}", e)))?;

        conn.execute(sql, duckdb::params![edge_id])
            .map_err(|e| crate::Error::DatabaseError(format!("删除边失败: {}", e)))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ZhushoudeConfig, NodeType, EdgeType};
    use std::collections::HashMap;
    use tempfile::NamedTempFile;
    
    #[tokio::test]
    async fn test_graph_storage() {
        let config = ZhushoudeConfig {
            database_path: ":memory:".to_string(),
            ..Default::default()
        };

        let db_manager = Arc::new(DatabaseManager::new(config).await.unwrap());
        let storage = GraphStorage::new(db_manager);

        let node = GraphNode {
            id: "node1".to_string(),
            label: "TestNode".to_string(),
            node_type: NodeType::Class,
            properties: HashMap::new(),
        };

        // 测试添加节点
        let result = storage.add_node(&node).await;
        assert!(result.is_ok());

        // 测试获取节点
        let retrieved_node = storage.get_node("node1").await.unwrap();
        assert!(retrieved_node.is_some());
        let retrieved_node = retrieved_node.unwrap();
        assert_eq!(retrieved_node.id, "node1");
        assert_eq!(retrieved_node.label, "TestNode");

        // 添加第二个节点
        let node2 = GraphNode {
            id: "node2".to_string(),
            label: "TestNode2".to_string(),
            node_type: NodeType::Method,
            properties: HashMap::new(),
        };
        storage.add_node(&node2).await.unwrap();

        let edge = GraphEdge {
            id: "edge1".to_string(),
            source_id: "node1".to_string(),
            target_id: "node2".to_string(),
            edge_type: EdgeType::DependsOn,
            weight: 1.0,
            properties: HashMap::new(),
        };

        // 测试添加边
        let result = storage.add_edge(&edge).await;
        assert!(result.is_ok());

        // 测试获取边
        let edges = storage.get_edges("node1").await.unwrap();
        assert_eq!(edges.len(), 1);
        assert_eq!(edges[0].id, "edge1");
        assert_eq!(edges[0].target_id, "node2");

        // 测试删除边
        storage.delete_edge("edge1").await.unwrap();
        let edges = storage.get_edges("node1").await.unwrap();
        assert_eq!(edges.len(), 0);

        // 测试删除节点
        storage.delete_node("node1").await.unwrap();
        let retrieved_node = storage.get_node("node1").await.unwrap();
        assert!(retrieved_node.is_none());
    }
}
