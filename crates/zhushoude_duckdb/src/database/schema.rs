//! 数据库模式定义

use crate::types::*;

/// 数据库模式管理
pub struct SchemaManager;

impl SchemaManager {
    /// 获取创建文档表的SQL
    pub fn create_documents_table() -> &'static str {
        "
        CREATE TABLE IF NOT EXISTS documents (
            id VARCHAR PRIMARY KEY,
            title VARCHAR NOT NULL,
            content TEXT NOT NULL,
            doc_type VARCHAR DEFAULT 'text',
            metadata JSON,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        )
        "
    }
    
    /// 获取创建向量表的SQL
    pub fn create_vectors_table() -> &'static str {
        "
        CREATE TABLE IF NOT EXISTS document_vectors (
            document_id VARCHAR REFERENCES documents(id),
            embedding FLOAT[512],
            model_name VARCHAR DEFAULT 'bge-small-zh',
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            PRIMARY KEY (document_id, model_name)
        )
        "
    }
    
    /// 获取创建图节点表的SQL
    pub fn create_graph_nodes_table() -> &'static str {
        "
        CREATE TABLE IF NOT EXISTS graph_nodes (
            id VARCHAR PRIMARY KEY,
            label VARCHAR NOT NULL,
            node_type VARCHAR NOT NULL,
            properties JSON,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        )
        "
    }
    
    /// 获取创建图边表的SQL
    pub fn create_graph_edges_table() -> &'static str {
        "
        CREATE TABLE IF NOT EXISTS graph_edges (
            id VARCHAR PRIMARY KEY,
            source_id VARCHAR NOT NULL,
            target_id VARCHAR NOT NULL,
            edge_type VARCHAR NOT NULL,
            weight FLOAT DEFAULT 1.0,
            properties JSON,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (source_id) REFERENCES graph_nodes(id),
            FOREIGN KEY (target_id) REFERENCES graph_nodes(id)
        )
        "
    }
    
    /// 获取创建HNSW索引的SQL
    pub fn create_hnsw_index() -> &'static str {
        "
        CREATE INDEX IF NOT EXISTS doc_vectors_hnsw_idx 
        ON document_vectors 
        USING HNSW (embedding)
        WITH (metric = 'cosine')
        "
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_sql_statements() {
        // 测试SQL语句不为空
        assert!(!SchemaManager::create_documents_table().is_empty());
        assert!(!SchemaManager::create_vectors_table().is_empty());
        assert!(!SchemaManager::create_graph_nodes_table().is_empty());
        assert!(!SchemaManager::create_graph_edges_table().is_empty());
        assert!(!SchemaManager::create_hnsw_index().is_empty());
        
        // 测试SQL语句包含关键字
        assert!(SchemaManager::create_documents_table().contains("CREATE TABLE"));
        assert!(SchemaManager::create_vectors_table().contains("document_vectors"));
        assert!(SchemaManager::create_graph_nodes_table().contains("graph_nodes"));
        assert!(SchemaManager::create_graph_edges_table().contains("graph_edges"));
        assert!(SchemaManager::create_hnsw_index().contains("HNSW"));
    }
}
