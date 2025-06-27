// 混合搜索引擎 - 融合关键词搜索和语义搜索

use std::collections::HashMap;
use std::sync::Arc;
use async_trait::async_trait;
use serde::{Serialize, Deserialize};

/// 混合搜索引擎
pub struct HybridSearchEngine {
    /// 语义搜索服务
    semantic_service: Arc<dyn SemanticSearchService + Send + Sync>,
    /// 关键词搜索服务（现有的Tantivy）
    keyword_service: Arc<dyn KeywordSearchService + Send + Sync>,
    /// 搜索配置
    config: SearchWeights,
}

impl HybridSearchEngine {
    pub fn new(
        semantic_service: Arc<dyn SemanticSearchService + Send + Sync>,
        keyword_service: Arc<dyn KeywordSearchService + Send + Sync>,
        config: SearchWeights,
    ) -> Self {
        Self {
            semantic_service,
            keyword_service,
            config,
        }
    }

    /// 执行混合搜索
    pub async fn hybrid_search(
        &self,
        query: &str,
        limit: usize,
        filters: Option<SearchFilters>,
    ) -> Result<Vec<HybridSearchResult>, SearchError> {
        log::info!("🔍 执行混合搜索: {}", query);

        // 并行执行语义搜索和关键词搜索
        let (semantic_results, keyword_results) = tokio::try_join!(
            self.semantic_search(query, limit * 2, filters.clone()),
            self.keyword_search(query, limit * 2, filters.clone())
        )?;

        // 融合搜索结果
        let hybrid_results = self.merge_results(
            semantic_results,
            keyword_results,
            query,
            limit,
        ).await?;

        log::info!("✅ 混合搜索完成，返回 {} 个结果", hybrid_results.len());
        Ok(hybrid_results)
    }

    /// 语义搜索
    async fn semantic_search(
        &self,
        query: &str,
        limit: usize,
        filters: Option<SearchFilters>,
    ) -> Result<Vec<SemanticSearchResult>, SearchError> {
        match self.semantic_service.semantic_search(query, limit, filters).await {
            Ok(results) => {
                log::debug!("语义搜索返回 {} 个结果", results.len());
                Ok(results)
            }
            Err(e) => {
                log::warn!("语义搜索失败: {}", e);
                // 语义搜索失败时返回空结果，不影响关键词搜索
                Ok(vec![])
            }
        }
    }

    /// 关键词搜索
    async fn keyword_search(
        &self,
        query: &str,
        limit: usize,
        filters: Option<SearchFilters>,
    ) -> Result<Vec<KeywordSearchResult>, SearchError> {
        match self.keyword_service.search(query, limit, filters).await {
            Ok(results) => {
                log::debug!("关键词搜索返回 {} 个结果", results.len());
                Ok(results)
            }
            Err(e) => {
                log::warn!("关键词搜索失败: {}", e);
                Ok(vec![])
            }
        }
    }

    /// 融合搜索结果
    async fn merge_results(
        &self,
        semantic_results: Vec<SemanticSearchResult>,
        keyword_results: Vec<KeywordSearchResult>,
        query: &str,
        limit: usize,
    ) -> Result<Vec<HybridSearchResult>, SearchError> {
        let mut result_map: HashMap<String, HybridSearchResult> = HashMap::new();

        // 处理语义搜索结果
        for semantic_result in semantic_results {
            let hybrid_result = HybridSearchResult {
                note_id: semantic_result.note_id.clone(),
                title: semantic_result.title.clone(),
                content_preview: semantic_result.content_preview.clone(),
                semantic_score: semantic_result.similarity_score,
                keyword_score: 0.0,
                graph_score: semantic_result.graph_score,
                combined_score: 0.0, // 稍后计算
                search_type: SearchType::Semantic,
                matched_terms: vec![],
                related_notes: semantic_result.related_notes,
                matched_concepts: semantic_result.matched_concepts,
                highlight_snippets: vec![],
            };

            result_map.insert(semantic_result.note_id, hybrid_result);
        }

        // 处理关键词搜索结果
        for keyword_result in keyword_results {
            match result_map.get_mut(&keyword_result.note_id) {
                Some(existing_result) => {
                    // 更新现有结果
                    existing_result.keyword_score = keyword_result.relevance_score;
                    existing_result.search_type = SearchType::Hybrid;
                    existing_result.matched_terms = keyword_result.matched_terms;
                    existing_result.highlight_snippets = keyword_result.highlight_snippets;
                }
                None => {
                    // 创建新结果
                    let hybrid_result = HybridSearchResult {
                        note_id: keyword_result.note_id.clone(),
                        title: keyword_result.title.clone(),
                        content_preview: keyword_result.content_preview.clone(),
                        semantic_score: 0.0,
                        keyword_score: keyword_result.relevance_score,
                        graph_score: 0.0,
                        combined_score: 0.0, // 稍后计算
                        search_type: SearchType::Keyword,
                        matched_terms: keyword_result.matched_terms,
                        related_notes: vec![],
                        matched_concepts: vec![],
                        highlight_snippets: keyword_result.highlight_snippets,
                    };

                    result_map.insert(keyword_result.note_id, hybrid_result);
                }
            }
        }

        // 计算综合分数并排序
        let mut results: Vec<HybridSearchResult> = result_map
            .into_values()
            .map(|mut result| {
                result.combined_score = self.calculate_combined_score(&result);
                result
            })
            .collect();

        // 按综合分数排序
        results.sort_by(|a, b| b.combined_score.partial_cmp(&a.combined_score).unwrap());

        // 限制结果数量
        results.truncate(limit);

        Ok(results)
    }

    /// 计算综合分数
    fn calculate_combined_score(&self, result: &HybridSearchResult) -> f32 {
        let semantic_weighted = result.semantic_score * self.config.semantic_weight;
        let keyword_weighted = result.keyword_score * self.config.keyword_weight;
        let graph_weighted = result.graph_score * self.config.graph_weight;

        semantic_weighted + keyword_weighted + graph_weighted
    }

    /// 智能搜索建议
    pub async fn get_search_suggestions(&self, query: &str) -> Result<Vec<SearchSuggestion>, SearchError> {
        // TODO: 实现基于历史搜索和内容的智能建议
        Ok(vec![])
    }

    /// 搜索统计
    pub async fn get_search_stats(&self) -> Result<SearchStatistics, SearchError> {
        // TODO: 实现搜索统计功能
        Ok(SearchStatistics {
            total_indexed_notes: 0,
            semantic_index_size: 0,
            keyword_index_size: 0,
            last_index_update: None,
        })
    }
}

/// 关键词搜索服务接口（适配现有Tantivy）
#[async_trait]
pub trait KeywordSearchService {
    async fn search(
        &self,
        query: &str,
        limit: usize,
        filters: Option<SearchFilters>,
    ) -> Result<Vec<KeywordSearchResult>, SearchError>;

    async fn index_note(&self, note: &Note) -> Result<(), SearchError>;
    async fn delete_note(&self, note_id: &str) -> Result<(), SearchError>;
}

/// 混合搜索结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HybridSearchResult {
    pub note_id: String,
    pub title: String,
    pub content_preview: String,
    pub semantic_score: f32,
    pub keyword_score: f32,
    pub graph_score: f32,
    pub combined_score: f32,
    pub search_type: SearchType,
    pub matched_terms: Vec<String>,
    pub related_notes: Vec<RelatedNote>,
    pub matched_concepts: Vec<String>,
    pub highlight_snippets: Vec<HighlightSnippet>,
}

/// 关键词搜索结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeywordSearchResult {
    pub note_id: String,
    pub title: String,
    pub content_preview: String,
    pub relevance_score: f32,
    pub matched_terms: Vec<String>,
    pub highlight_snippets: Vec<HighlightSnippet>,
}

/// 搜索类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SearchType {
    Semantic,
    Keyword,
    Hybrid,
}

/// 高亮片段
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HighlightSnippet {
    pub text: String,
    pub start_pos: usize,
    pub end_pos: usize,
    pub matched_term: String,
}

/// 搜索建议
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchSuggestion {
    pub text: String,
    pub suggestion_type: SuggestionType,
    pub confidence: f32,
}

/// 建议类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SuggestionType {
    Completion,
    Correction,
    Related,
    Concept,
}

/// 搜索统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchStatistics {
    pub total_indexed_notes: usize,
    pub semantic_index_size: usize,
    pub keyword_index_size: usize,
    pub last_index_update: Option<chrono::DateTime<chrono::Utc>>,
}

/// 搜索错误
#[derive(Debug, thiserror::Error)]
pub enum SearchError {
    #[error("语义搜索错误: {0}")]
    SemanticError(#[from] SemanticSearchError),
    #[error("关键词搜索错误: {0}")]
    KeywordError(String),
    #[error("结果融合错误: {0}")]
    MergeError(String),
    #[error("配置错误: {0}")]
    ConfigError(String),
}

// 重新导出类型
pub use crate::{
    SemanticSearchService, SemanticSearchResult, SemanticSearchError,
    RelatedNote, SearchFilters, SearchWeights, Note,
};
