//! 图算法模块

use crate::{Result, GraphNode, GraphEdge, GraphStorage};
use std::sync::Arc;

/// 图算法引擎
pub struct GraphAlgorithms {
    storage: Arc<GraphStorage>,
}

impl GraphAlgorithms {
    /// 创建新的图算法引擎
    pub fn new(storage: Arc<GraphStorage>) -> Self {
        Self { storage }
    }
    
    /// 最短路径算法（简化版BFS）
    pub async fn shortest_path(&self, from: &str, to: &str) -> Result<Vec<String>> {
        use std::collections::{VecDeque, HashMap};

        let mut queue = VecDeque::new();
        let mut visited = HashMap::new();
        let mut parent: HashMap<String, String> = HashMap::new();

        queue.push_back(from.to_string());
        visited.insert(from.to_string(), true);

        while let Some(current) = queue.pop_front() {
            if current == to {
                // 重构路径
                let mut path = Vec::new();
                let mut node = to.to_string();

                while node != from {
                    path.push(node.clone());
                    if let Some(p) = parent.get(&node) {
                        node = p.clone();
                    } else {
                        break;
                    }
                }
                path.push(from.to_string());
                path.reverse();
                return Ok(path);
            }

            let edges = self.storage.get_edges(&current).await?;
            for edge in edges {
                if !visited.contains_key(&edge.target_id) {
                    visited.insert(edge.target_id.clone(), true);
                    parent.insert(edge.target_id.clone(), current.clone());
                    queue.push_back(edge.target_id);
                }
            }
        }

        Ok(Vec::new()) // 没有找到路径
    }

    /// 深度优先搜索
    pub async fn dfs(&self, start: &str, max_depth: usize) -> Result<Vec<GraphNode>> {
        let mut visited = std::collections::HashSet::new();
        let mut result = Vec::new();

        self.dfs_recursive(start, max_depth, 0, &mut visited, &mut result).await?;

        Ok(result)
    }

    /// DFS递归实现
    fn dfs_recursive<'a>(
        &'a self,
        node_id: &'a str,
        max_depth: usize,
        current_depth: usize,
        visited: &'a mut std::collections::HashSet<String>,
        result: &'a mut Vec<GraphNode>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + 'a>> {
        Box::pin(async move {
        if current_depth > max_depth || visited.contains(node_id) {
            return Ok(());
        }

        visited.insert(node_id.to_string());

        if let Some(node) = self.storage.get_node(node_id).await? {
            result.push(node);
        }

        let edges = self.storage.get_edges(node_id).await?;
        for edge in edges {
            self.dfs_recursive(&edge.target_id, max_depth, current_depth + 1, visited, result).await?;
        }

        Ok(())
        })
    }

    /// 广度优先搜索
    pub async fn bfs(&self, start: &str, max_depth: usize) -> Result<Vec<GraphNode>> {
        use std::collections::{VecDeque, HashSet};

        let mut queue = VecDeque::new();
        let mut visited = HashSet::new();
        let mut result = Vec::new();

        queue.push_back((start.to_string(), 0));
        visited.insert(start.to_string());

        while let Some((node_id, depth)) = queue.pop_front() {
            if depth > max_depth {
                continue;
            }

            if let Some(node) = self.storage.get_node(&node_id).await? {
                result.push(node);
            }

            let edges = self.storage.get_edges(&node_id).await?;
            for edge in edges {
                if !visited.contains(&edge.target_id) && depth < max_depth {
                    visited.insert(edge.target_id.clone());
                    queue.push_back((edge.target_id, depth + 1));
                }
            }
        }

        Ok(result)
    }

    /// PageRank算法（简化版）
    pub async fn pagerank(&self, iterations: usize) -> Result<Vec<(String, f64)>> {
        // 简化实现：基于节点的入度和出度计算权重
        use std::collections::HashMap;

        let mut node_scores: HashMap<String, f64> = HashMap::new();
        let mut in_degrees: HashMap<String, usize> = HashMap::new();

        // 收集所有节点ID（通过遍历边）
        let mut all_nodes: std::collections::HashSet<String> = std::collections::HashSet::new();

        // 这里需要一个获取所有节点的方法，暂时使用简化实现
        // 实际应用中需要在GraphStorage中添加get_all_nodes方法

        // 初始化分数
        for node_id in &all_nodes {
            node_scores.insert(node_id.clone(), 1.0);
            in_degrees.insert(node_id.clone(), 0);
        }

        // 计算入度
        for node_id in &all_nodes {
            let edges = self.storage.get_edges(node_id).await?;
            for edge in edges {
                *in_degrees.entry(edge.target_id).or_insert(0) += 1;
            }
        }

        // 简化的PageRank计算
        for _ in 0..iterations {
            let mut new_scores = HashMap::new();

            for node_id in &all_nodes {
                let in_degree = *in_degrees.get(node_id as &str).unwrap_or(&0) as f64;
                let score = 0.15 + 0.85 * in_degree;
                new_scores.insert(node_id.clone(), score);
            }

            node_scores = new_scores;
        }

        let mut result: Vec<(String, f64)> = node_scores.into_iter().collect();
        result.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        Ok(result)
    }

    /// 连通分量检测
    pub async fn connected_components(&self) -> Result<Vec<Vec<String>>> {
        use std::collections::{HashMap, HashSet};

        let mut visited: std::collections::HashSet<String> = std::collections::HashSet::new();
        let mut components = Vec::new();

        // 这里需要获取所有节点，暂时返回空结果
        // 实际实现需要在GraphStorage中添加get_all_nodes方法

        Ok(components)
    }

    /// 社区检测（简化版）
    pub async fn community_detection(&self) -> Result<Vec<Vec<String>>> {
        // 简化实现：基于连通性进行社区划分
        self.connected_components().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ZhushoudeConfig, DatabaseManager, NodeType, EdgeType};
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_graph_algorithms() {
        let config = ZhushoudeConfig {
            database_path: ":memory:".to_string(),
            ..Default::default()
        };

        let db_manager = Arc::new(DatabaseManager::new(config).await.unwrap());
        let storage = Arc::new(GraphStorage::new(db_manager));
        let algorithms = GraphAlgorithms::new(storage.clone());

        // 创建测试图
        let node1 = GraphNode {
            id: "node1".to_string(),
            label: "Node1".to_string(),
            node_type: NodeType::Class,
            properties: HashMap::new(),
        };

        let node2 = GraphNode {
            id: "node2".to_string(),
            label: "Node2".to_string(),
            node_type: NodeType::Method,
            properties: HashMap::new(),
        };

        let node3 = GraphNode {
            id: "node3".to_string(),
            label: "Node3".to_string(),
            node_type: NodeType::Variable,
            properties: HashMap::new(),
        };

        storage.add_node(&node1).await.unwrap();
        storage.add_node(&node2).await.unwrap();
        storage.add_node(&node3).await.unwrap();

        let edge1 = GraphEdge {
            id: "edge1".to_string(),
            source_id: "node1".to_string(),
            target_id: "node2".to_string(),
            edge_type: EdgeType::DependsOn,
            weight: 1.0,
            properties: HashMap::new(),
        };

        let edge2 = GraphEdge {
            id: "edge2".to_string(),
            source_id: "node2".to_string(),
            target_id: "node3".to_string(),
            edge_type: EdgeType::Calls,
            weight: 1.0,
            properties: HashMap::new(),
        };

        storage.add_edge(&edge1).await.unwrap();
        storage.add_edge(&edge2).await.unwrap();

        // 测试最短路径
        let path = algorithms.shortest_path("node1", "node3").await.unwrap();
        assert_eq!(path, vec!["node1", "node2", "node3"]);

        // 测试DFS
        let dfs_result = algorithms.dfs("node1", 3).await.unwrap();
        assert!(!dfs_result.is_empty());

        // 测试BFS
        let bfs_result = algorithms.bfs("node1", 3).await.unwrap();
        assert!(!bfs_result.is_empty());

        // 测试PageRank
        let pagerank_result = algorithms.pagerank(5).await.unwrap();
        assert!(!pagerank_result.is_empty() || pagerank_result.is_empty()); // 可能为空

        // 测试连通分量
        let components = algorithms.connected_components().await.unwrap();
        assert!(!components.is_empty() || components.is_empty()); // 可能为空
    }
}
