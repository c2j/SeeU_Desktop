//! 混合搜索引擎模块

use crate::{Result, SemanticSearchEngine, HybridQuery, HybridResult, HybridConfig};
use crate::hybrid::analyzer::CodeGraphAnalyzer;
use std::sync::Arc;

/// 混合搜索引擎
pub struct HybridSearchEngine {
    semantic_engine: Arc<SemanticSearchEngine>,
    graph_analyzer: Arc<CodeGraphAnalyzer>,
    config: HybridConfig,
}

impl HybridSearchEngine {
    /// 创建新的混合搜索引擎
    pub fn new(
        semantic_engine: Arc<SemanticSearchEngine>,
        graph_analyzer: Arc<CodeGraphAnalyzer>,
        config: HybridConfig,
    ) -> Self {
        Self {
            semantic_engine,
            graph_analyzer,
            config,
        }
    }
    
    /// 执行混合搜索
    pub async fn hybrid_search(&self, query: &HybridQuery) -> Result<Vec<HybridResult>> {
        let mut results = Vec::new();
        
        // 1. 语义搜索
        if query.enable_semantic {
            let semantic_results = self.semantic_engine
                .search(&query.text, query.limit)
                .await?;
            
            for result in semantic_results {
                results.push(HybridResult {
                    document_id: result.document_id,
                    title: result.title,
                    content: result.content,
                    score: result.similarity_score,
                    final_score: result.similarity_score * query.weights.semantic as f64,
                    result_type: crate::ResultType::Semantic,
                    metadata: result.metadata.unwrap_or_default(),
                });
            }
        }
        
        // 2. 图搜索
        if query.enable_graph {
            let graph_results = self.graph_analyzer
                .find_related_code(&query.text)
                .await?;

            for result in graph_results {
                // 检查是否已存在语义搜索结果
                if let Some(existing) = results.iter_mut().find(|r| r.document_id == result.node_id) {
                    // 融合分数
                    existing.final_score = existing.score * query.weights.semantic as f64
                        + result.relevance_score * query.weights.graph as f64;
                    existing.result_type = crate::ResultType::Hybrid;
                } else {
                    // 添加新的图搜索结果
                    results.push(HybridResult {
                        document_id: result.node_id,
                        title: result.label,
                        content: result.code_snippet,
                        score: result.relevance_score,
                        final_score: result.relevance_score * query.weights.graph as f64,
                        result_type: crate::ResultType::Graph,
                        metadata: result.properties,
                    });
                }
            }
        }
        
        // 3. 去重结果
        self.deduplicate_results(&mut results);

        // 4. 结果融合和排序
        self.merge_and_rank_results(&mut results, query).await?;

        // 5. 返回Top-K结果
        results.truncate(query.limit);
        Ok(results)
    }
    
    /// 结果融合和排序
    async fn merge_and_rank_results(
        &self,
        results: &mut Vec<HybridResult>,
        query: &HybridQuery,
    ) -> Result<()> {
        use crate::FusionStrategy;
        
        match self.config.fusion_strategy {
            FusionStrategy::WeightedAverage => {
                // 已经在上面计算了加权分数
            }
            FusionStrategy::ReciprocalRankFusion => {
                self.apply_rrf(results)?;
            }
            FusionStrategy::MaxFusion => {
                self.apply_max_fusion(results)?;
            }
        }
        
        // 按最终分数排序
        results.sort_by(|a, b| b.final_score.partial_cmp(&a.final_score).unwrap());
        
        Ok(())
    }
    
    /// 应用倒数排名融合(RRF)
    fn apply_rrf(&self, results: &mut Vec<HybridResult>) -> Result<()> {
        // 收集语义搜索和图搜索结果的索引
        let semantic_indices: Vec<usize> = results.iter()
            .enumerate()
            .filter(|(_, r)| matches!(r.result_type, crate::ResultType::Semantic))
            .map(|(i, _)| i)
            .collect();

        let graph_indices: Vec<usize> = results.iter()
            .enumerate()
            .filter(|(_, r)| matches!(r.result_type, crate::ResultType::Graph))
            .map(|(i, _)| i)
            .collect();

        // 计算RRF分数
        for (i, result) in results.iter_mut().enumerate() {
            let mut rrf_score = 0.0;
            let k = 60.0; // RRF参数

            // 在语义搜索结果中查找排名
            if let Some(rank) = semantic_indices.iter().position(|&idx| idx == i) {
                rrf_score += 1.0 / (k + rank as f64 + 1.0);
            }

            // 在图搜索结果中查找排名
            if let Some(rank) = graph_indices.iter().position(|&idx| idx == i) {
                rrf_score += 1.0 / (k + rank as f64 + 1.0);
            }

            result.final_score = rrf_score;
        }

        Ok(())
    }
    
    /// 应用最大值融合
    fn apply_max_fusion(&self, results: &mut Vec<HybridResult>) -> Result<()> {
        for result in results.iter_mut() {
            result.final_score = result.score;
        }
        Ok(())
    }
    
    /// 去重结果
    fn deduplicate_results(&self, results: &mut Vec<HybridResult>) {
        results.sort_by(|a, b| a.document_id.cmp(&b.document_id));
        results.dedup_by(|a, b| a.document_id == b.document_id);
    }

    /// 执行语义相似度搜索
    pub async fn semantic_search(&self, query: &str, limit: usize) -> Result<Vec<HybridResult>> {
        let semantic_results = self.semantic_engine.search(query, limit).await?;

        let mut results = Vec::new();
        for result in semantic_results {
            results.push(HybridResult {
                document_id: result.document_id,
                title: result.title,
                content: result.content,
                score: result.similarity_score,
                final_score: result.similarity_score,
                result_type: crate::ResultType::Semantic,
                metadata: result.metadata.unwrap_or_default(),
            });
        }

        Ok(results)
    }

    /// 执行图结构搜索
    pub async fn graph_search(&self, query: &str, limit: usize) -> Result<Vec<HybridResult>> {
        let graph_results = self.graph_analyzer.find_related_code(query).await?;

        let mut results = Vec::new();
        for result in graph_results.into_iter().take(limit) {
            results.push(HybridResult {
                document_id: result.node_id,
                title: result.label,
                content: result.code_snippet,
                score: result.relevance_score,
                final_score: result.relevance_score,
                result_type: crate::ResultType::Graph,
                metadata: result.properties,
            });
        }

        Ok(results)
    }

    /// 获取搜索统计信息
    pub async fn get_search_stats(&self) -> Result<HybridSearchStats> {
        let semantic_stats = self.semantic_engine.get_cache_stats();
        let document_count = self.semantic_engine.get_document_count().await?;

        Ok(HybridSearchStats {
            total_documents: document_count,
            semantic_cache_hits: semantic_stats.hits,
            semantic_cache_misses: semantic_stats.misses,
            graph_nodes: 0, // 需要从GraphAnalyzer获取
            graph_edges: 0, // 需要从GraphAnalyzer获取
        })
    }

    /// 获取配置信息
    pub fn get_config(&self) -> &HybridConfig {
        &self.config
    }

    /// 更新配置
    pub fn update_config(&mut self, config: HybridConfig) {
        self.config = config;
    }
}

/// 混合搜索统计信息
#[derive(Debug)]
pub struct HybridSearchStats {
    pub total_documents: usize,
    pub semantic_cache_hits: u64,
    pub semantic_cache_misses: u64,
    pub graph_nodes: usize,
    pub graph_edges: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ZhushoudeConfig, EmbeddingConfig, QueryType};
    use crate::types::SearchWeights;
    use tempfile::NamedTempFile;
    
    #[tokio::test]
    async fn test_hybrid_search_engine() {
        let temp_file = NamedTempFile::new().expect("创建临时文件失败");
        let config = ZhushoudeConfig {
            database_path: temp_file.path().to_str().unwrap().to_string(),
            ..Default::default()
        };
        
        let db_manager = Arc::new(crate::DatabaseManager::new(config).await.unwrap());
        let embedding_engine = Arc::new(crate::EmbeddingEngine::new(EmbeddingConfig::default()).await.unwrap());
        let semantic_engine = Arc::new(SemanticSearchEngine::new(db_manager.clone(), embedding_engine));
        let graph_analyzer = Arc::new(CodeGraphAnalyzer::new(db_manager));
        
        let hybrid_engine = HybridSearchEngine::new(
            semantic_engine,
            graph_analyzer,
            HybridConfig::default(),
        );
        
        let query = HybridQuery {
            text: "测试查询".to_string(),
            query_type: QueryType::General,
            limit: 10,
            enable_semantic: true,
            enable_graph: false,
            weights: crate::types::SearchWeights {
                semantic: 0.7,
                graph: 0.3,
            },
        };
        
        let result = hybrid_engine.hybrid_search(&query).await;
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_rrf_calculation() {
        let temp_file = NamedTempFile::new().expect("创建临时文件失败");
        let config = ZhushoudeConfig {
            database_path: temp_file.path().to_str().unwrap().to_string(),
            ..Default::default()
        };
        
        // 这里需要异步运行时来创建依赖项
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let db_manager = Arc::new(crate::DatabaseManager::new(config).await.unwrap());
            let embedding_engine = Arc::new(crate::EmbeddingEngine::new(EmbeddingConfig::default()).await.unwrap());
            let semantic_engine = Arc::new(SemanticSearchEngine::new(db_manager.clone(), embedding_engine));
            let graph_analyzer = Arc::new(CodeGraphAnalyzer::new(db_manager));
            
            let hybrid_engine = HybridSearchEngine::new(
                semantic_engine,
                graph_analyzer,
                HybridConfig::default(),
            );
            
            let mut results = vec![
                HybridResult {
                    document_id: "doc1".to_string(),
                    title: "Test".to_string(),
                    content: "Content".to_string(),
                    score: 0.9,
                    final_score: 0.0,
                    result_type: crate::ResultType::Semantic,
                    metadata: serde_json::json!({}),
                },
            ];
            
            let result = hybrid_engine.apply_rrf(&mut results);
            assert!(result.is_ok());
            assert!(results[0].final_score > 0.0);
        });
    }
}
