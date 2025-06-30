use crate::{Result, Error, ChunkVectorIndexManager, ChunkSearchResult, ChunkVectorConfig};
use crate::database::DatabaseManager;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::collections::HashMap;

/// 分块语义搜索配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkSemanticSearchConfig {
    /// 向量索引配置
    pub vector_config: ChunkVectorConfig,
    /// 默认搜索结果数量
    pub default_limit: usize,
    /// 最大搜索结果数量
    pub max_limit: usize,
    /// 相似度阈值
    pub similarity_threshold: f32,
    /// 是否启用结果聚合
    pub enable_aggregation: bool,
    /// 聚合窗口大小
    pub aggregation_window: usize,
    /// 是否启用重排序
    pub enable_reranking: bool,
}

impl Default for ChunkSemanticSearchConfig {
    fn default() -> Self {
        Self {
            vector_config: ChunkVectorConfig::default(),
            default_limit: 10,
            max_limit: 100,
            similarity_threshold: 0.3,
            enable_aggregation: true,
            aggregation_window: 3,
            enable_reranking: true,
        }
    }
}

/// 搜索查询
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkSearchQuery {
    /// 查询文本
    pub query: String,
    /// 搜索结果数量
    pub limit: Option<usize>,
    /// 文档ID过滤
    pub document_ids: Option<Vec<String>>,
    /// 语言过滤
    pub languages: Option<Vec<String>>,
    /// 内容类型过滤
    pub content_types: Option<Vec<String>>,
    /// 最小质量分数
    pub min_quality_score: Option<f32>,
    /// 模型名称
    pub model_name: Option<String>,
    /// 是否启用上下文扩展
    pub enable_context_expansion: Option<bool>,
}

/// 聚合搜索结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatedSearchResult {
    /// 主要分块
    pub primary_chunk: ChunkSearchResult,
    /// 相关分块（上下文）
    pub context_chunks: Vec<ChunkSearchResult>,
    /// 聚合相似度分数
    pub aggregated_score: f32,
    /// 总字符数
    pub total_char_count: usize,
    /// 连续性分数
    pub continuity_score: f32,
}

/// 搜索统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkSearchStats {
    /// 查询时间（毫秒）
    pub query_time_ms: u64,
    /// 总结果数
    pub total_results: usize,
    /// 聚合结果数
    pub aggregated_results: usize,
    /// 平均相似度分数
    pub avg_similarity_score: f32,
    /// 搜索的分块数
    pub chunks_searched: usize,
    /// 使用的模型
    pub model_used: String,
}

/// 分块语义搜索引擎
pub struct ChunkSemanticSearchEngine {
    index_manager: ChunkVectorIndexManager,
    config: ChunkSemanticSearchConfig,
}

impl ChunkSemanticSearchEngine {
    /// 创建新的分块语义搜索引擎
    pub fn new(db_manager: Arc<DatabaseManager>, config: ChunkSemanticSearchConfig) -> Self {
        let index_manager = ChunkVectorIndexManager::new(db_manager, config.vector_config.clone());
        Self {
            index_manager,
            config,
        }
    }

    /// 使用默认配置创建搜索引擎
    pub fn default(db_manager: Arc<DatabaseManager>) -> Self {
        Self::new(db_manager, ChunkSemanticSearchConfig::default())
    }

    /// 初始化搜索引擎
    pub async fn initialize(&self) -> Result<()> {
        self.index_manager.initialize_tables().await?;
        println!("✅ 分块语义搜索引擎初始化完成");
        Ok(())
    }

    /// 索引文档
    pub async fn index_document(&self, document_id: &str, content: &str, model_name: &str) -> Result<()> {
        self.index_manager.index_document(document_id, content, model_name).await?;
        println!("📄 文档 {} 索引完成", document_id);
        Ok(())
    }

    /// 执行分块语义搜索
    pub async fn search(&self, query: &ChunkSearchQuery) -> Result<(Vec<AggregatedSearchResult>, ChunkSearchStats)> {
        let start_time = std::time::Instant::now();
        
        // 1. 参数验证和默认值设置
        let limit = query.limit.unwrap_or(self.config.default_limit).min(self.config.max_limit);
        let model_name = query.model_name.as_deref().unwrap_or("default");
        let enable_context = query.enable_context_expansion.unwrap_or(self.config.enable_aggregation);
        
        // 2. 执行向量搜索
        let raw_results = self.index_manager.search_chunks(&query.query, model_name, limit * 3).await?;
        
        // 3. 应用过滤器
        let filtered_results = self.apply_filters(&raw_results, query)?;
        
        // 4. 聚合结果（如果启用）
        let aggregated_results = if enable_context {
            self.aggregate_results(&filtered_results).await?
        } else {
            filtered_results.into_iter()
                .map(|chunk| AggregatedSearchResult {
                    aggregated_score: chunk.similarity_score,
                    total_char_count: chunk.char_count,
                    continuity_score: 1.0,
                    context_chunks: Vec::new(),
                    primary_chunk: chunk,
                })
                .collect()
        };
        
        // 5. 重排序（如果启用）
        let final_results = if self.config.enable_reranking {
            self.rerank_results(aggregated_results, &query.query).await?
        } else {
            aggregated_results
        };
        
        // 6. 限制结果数量
        let limited_results: Vec<AggregatedSearchResult> = final_results.into_iter().take(limit).collect();
        
        // 7. 生成统计信息
        let stats = ChunkSearchStats {
            query_time_ms: start_time.elapsed().as_millis() as u64,
            total_results: raw_results.len(),
            aggregated_results: limited_results.len(),
            avg_similarity_score: if !limited_results.is_empty() {
                limited_results.iter().map(|r| r.aggregated_score).sum::<f32>() / limited_results.len() as f32
            } else {
                0.0
            },
            chunks_searched: raw_results.len(),
            model_used: model_name.to_string(),
        };
        
        Ok((limited_results, stats))
    }

    /// 应用搜索过滤器
    fn apply_filters(&self, results: &[ChunkSearchResult], query: &ChunkSearchQuery) -> Result<Vec<ChunkSearchResult>> {
        let mut filtered = results.to_vec();
        
        // 相似度阈值过滤
        filtered.retain(|r| r.similarity_score >= self.config.similarity_threshold);
        
        // 文档ID过滤
        if let Some(ref document_ids) = query.document_ids {
            filtered.retain(|r| document_ids.contains(&r.document_id));
        }
        
        // 语言过滤
        if let Some(ref languages) = query.languages {
            filtered.retain(|r| languages.contains(&r.language));
        }
        
        // 内容类型过滤
        if let Some(ref content_types) = query.content_types {
            filtered.retain(|r| content_types.contains(&r.content_type));
        }
        
        // 质量分数过滤
        if let Some(min_quality) = query.min_quality_score {
            filtered.retain(|r| r.quality_score >= min_quality);
        }
        
        Ok(filtered)
    }

    /// 聚合搜索结果
    async fn aggregate_results(&self, results: &[ChunkSearchResult]) -> Result<Vec<AggregatedSearchResult>> {
        let mut aggregated = Vec::new();
        let mut used_chunks = std::collections::HashSet::new();
        
        for result in results {
            if used_chunks.contains(&result.chunk_id) {
                continue;
            }
            
            // 查找相邻的分块
            let context_chunks = self.find_context_chunks(result).await?;
            
            // 标记已使用的分块
            used_chunks.insert(result.chunk_id.clone());
            for chunk in &context_chunks {
                used_chunks.insert(chunk.chunk_id.clone());
            }
            
            // 计算聚合分数
            let aggregated_score = self.calculate_aggregated_score(result, &context_chunks);
            let total_char_count = result.char_count + context_chunks.iter().map(|c| c.char_count).sum::<usize>();
            let continuity_score = self.calculate_continuity_score(result, &context_chunks);
            
            aggregated.push(AggregatedSearchResult {
                primary_chunk: result.clone(),
                context_chunks,
                aggregated_score,
                total_char_count,
                continuity_score,
            });
        }
        
        // 按聚合分数排序
        aggregated.sort_by(|a, b| b.aggregated_score.partial_cmp(&a.aggregated_score).unwrap_or(std::cmp::Ordering::Equal));
        
        Ok(aggregated)
    }

    /// 查找上下文分块
    async fn find_context_chunks(&self, primary_chunk: &ChunkSearchResult) -> Result<Vec<ChunkSearchResult>> {
        let document_chunks = self.index_manager.get_document_chunks(&primary_chunk.document_id).await?;
        
        let mut context_chunks = Vec::new();
        
        // 查找主分块的索引
        if let Some(primary_index) = document_chunks.iter().position(|c| c.id == primary_chunk.chunk_id) {
            let window_size = self.config.aggregation_window;
            
            // 添加前面的分块
            for i in (primary_index.saturating_sub(window_size)..primary_index).rev() {
                if let Some(chunk) = document_chunks.get(i) {
                    context_chunks.push(ChunkSearchResult {
                        chunk_id: chunk.id.clone(),
                        document_id: chunk.document_id.clone(),
                        content: chunk.content.clone(),
                        similarity_score: 0.5, // 上下文分块的相似度分数较低
                        start_offset: chunk.start_offset,
                        end_offset: chunk.end_offset,
                        char_count: chunk.metadata.char_count,
                        quality_score: chunk.metadata.quality_score,
                        language: chunk.metadata.language.clone(),
                        content_type: chunk.metadata.content_type.clone(),
                    });
                }
            }
            
            // 添加后面的分块
            for i in (primary_index + 1)..=(primary_index + window_size).min(document_chunks.len() - 1) {
                if let Some(chunk) = document_chunks.get(i) {
                    context_chunks.push(ChunkSearchResult {
                        chunk_id: chunk.id.clone(),
                        document_id: chunk.document_id.clone(),
                        content: chunk.content.clone(),
                        similarity_score: 0.5,
                        start_offset: chunk.start_offset,
                        end_offset: chunk.end_offset,
                        char_count: chunk.metadata.char_count,
                        quality_score: chunk.metadata.quality_score,
                        language: chunk.metadata.language.clone(),
                        content_type: chunk.metadata.content_type.clone(),
                    });
                }
            }
        }
        
        Ok(context_chunks)
    }

    /// 计算聚合分数
    fn calculate_aggregated_score(&self, primary: &ChunkSearchResult, context: &[ChunkSearchResult]) -> f32 {
        let primary_weight = 0.7;
        let context_weight = 0.3;

        let primary_score = primary.similarity_score * primary_weight;
        let context_score = if !context.is_empty() {
            let avg_context_score = context.iter().map(|c| c.similarity_score).sum::<f32>() / context.len() as f32;
            avg_context_score * context_weight
        } else {
            0.0
        };

        primary_score + context_score
    }

    /// 计算连续性分数
    fn calculate_continuity_score(&self, primary: &ChunkSearchResult, context: &[ChunkSearchResult]) -> f32 {
        if context.is_empty() {
            return 1.0;
        }

        let mut continuity_score = 0.0;
        let mut total_pairs = 0;

        // 检查分块之间的连续性
        for chunk in context {
            let distance = if chunk.end_offset <= primary.start_offset {
                primary.start_offset - chunk.end_offset
            } else if chunk.start_offset >= primary.end_offset {
                chunk.start_offset - primary.end_offset
            } else {
                0 // 重叠
            };

            // 距离越小，连续性分数越高
            let chunk_continuity = if distance == 0 {
                1.0
            } else {
                1.0 / (1.0 + distance as f32 / 100.0)
            };

            continuity_score += chunk_continuity;
            total_pairs += 1;
        }

        if total_pairs > 0 {
            continuity_score / total_pairs as f32
        } else {
            1.0
        }
    }

    /// 重排序结果
    async fn rerank_results(&self, mut results: Vec<AggregatedSearchResult>, _query: &str) -> Result<Vec<AggregatedSearchResult>> {
        // 简单的重排序策略：结合相似度分数、质量分数和连续性分数
        results.sort_by(|a, b| {
            let score_a = a.aggregated_score * 0.6 + a.primary_chunk.quality_score * 0.2 + a.continuity_score * 0.2;
            let score_b = b.aggregated_score * 0.6 + b.primary_chunk.quality_score * 0.2 + b.continuity_score * 0.2;
            score_b.partial_cmp(&score_a).unwrap_or(std::cmp::Ordering::Equal)
        });

        Ok(results)
    }

    /// 删除文档
    pub async fn delete_document(&self, document_id: &str) -> Result<()> {
        self.index_manager.delete_document(document_id).await?;
        println!("🗑️ 文档 {} 已从搜索索引中删除", document_id);
        Ok(())
    }

    /// 获取搜索引擎统计信息
    pub async fn get_stats(&self) -> Result<ChunkSearchEngineStats> {
        let index_stats = self.index_manager.get_index_stats().await?;

        Ok(ChunkSearchEngineStats {
            total_documents: index_stats.total_documents,
            total_chunks: index_stats.total_chunks,
            avg_chunk_size: index_stats.avg_chunk_size,
            avg_quality_score: index_stats.avg_quality_score,
            vector_dimension: self.config.vector_config.vector_dimension,
            chunking_strategy: format!("{:?}", self.config.vector_config.chunking_config.strategy),
            similarity_threshold: self.config.similarity_threshold,
        })
    }

    /// 批量索引文档
    pub async fn batch_index_documents(&self, documents: &[(String, String)], model_name: &str) -> Result<()> {
        println!("📚 开始批量索引 {} 个文档...", documents.len());

        for (i, (doc_id, content)) in documents.iter().enumerate() {
            self.index_document(doc_id, content, model_name).await?;

            if (i + 1) % 10 == 0 {
                println!("📄 已索引 {}/{} 个文档", i + 1, documents.len());
            }
        }

        println!("✅ 批量索引完成");
        Ok(())
    }

    /// 搜索相似文档
    pub async fn find_similar_documents(&self, document_id: &str, model_name: &str, limit: usize) -> Result<Vec<String>> {
        // 获取文档的所有分块
        let chunks = self.index_manager.get_document_chunks(document_id).await?;

        if chunks.is_empty() {
            return Ok(Vec::new());
        }

        // 使用第一个分块作为查询
        let query = ChunkSearchQuery {
            query: chunks[0].content.clone(),
            limit: Some(limit * 5), // 获取更多结果以便过滤
            document_ids: None,
            languages: None,
            content_types: None,
            min_quality_score: None,
            model_name: Some(model_name.to_string()),
            enable_context_expansion: Some(false),
        };

        let (results, _) = self.search(&query).await?;

        // 提取不同的文档ID
        let mut similar_docs = Vec::new();
        let mut seen_docs = std::collections::HashSet::new();
        seen_docs.insert(document_id.to_string()); // 排除自己

        for result in results {
            if !seen_docs.contains(&result.primary_chunk.document_id) {
                similar_docs.push(result.primary_chunk.document_id.clone());
                seen_docs.insert(result.primary_chunk.document_id);

                if similar_docs.len() >= limit {
                    break;
                }
            }
        }

        Ok(similar_docs)
    }
}

/// 搜索引擎统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkSearchEngineStats {
    pub total_documents: usize,
    pub total_chunks: usize,
    pub avg_chunk_size: f32,
    pub avg_quality_score: f32,
    pub vector_dimension: usize,
    pub chunking_strategy: String,
    pub similarity_threshold: f32,
}
