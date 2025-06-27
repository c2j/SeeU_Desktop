//! 混合搜索引擎 - 融合关键词搜索和语义搜索

use std::sync::Arc;
use async_trait::async_trait;
use serde::{Serialize, Deserialize};

use crate::{
    EmbeddedHelixService, SemanticSearchResult, SearchFilters, SearchWeights,
    HybridSearchResult, SearchType, HighlightSnippet, SearchError
};

/// 混合搜索引擎
pub struct HybridSearchEngine {
    /// 语义搜索服务
    semantic_service: Arc<EmbeddedHelixService>,
    /// 搜索权重配置
    weights: SearchWeights,
}

impl HybridSearchEngine {
    pub fn new(
        semantic_service: Arc<EmbeddedHelixService>,
        weights: SearchWeights,
    ) -> Self {
        Self {
            semantic_service,
            weights,
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

        // 目前只实现语义搜索，后续集成关键词搜索
        let semantic_results = self.semantic_search(query, limit, filters.clone()).await?;

        // 转换为混合搜索结果
        let hybrid_results: Vec<HybridSearchResult> = semantic_results
            .into_iter()
            .map(|result| self.convert_to_hybrid_result(result, query))
            .collect();

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
                // 语义搜索失败时返回空结果，不影响整体搜索
                Ok(vec![])
            }
        }
    }

    /// 转换为混合搜索结果
    fn convert_to_hybrid_result(&self, semantic_result: SemanticSearchResult, query: &str) -> HybridSearchResult {
        // 计算综合分数
        let combined_score = self.calculate_combined_score(
            semantic_result.similarity_score,
            semantic_result.keyword_score,
            semantic_result.graph_score,
        );

        HybridSearchResult {
            note_id: semantic_result.note_id,
            title: semantic_result.title,
            content_preview: semantic_result.content_preview,
            semantic_score: semantic_result.similarity_score,
            keyword_score: semantic_result.keyword_score,
            graph_score: semantic_result.graph_score,
            combined_score,
            search_type: SearchType::Semantic,
            matched_terms: self.extract_matched_terms(query),
            related_notes: semantic_result.related_notes,
            matched_concepts: semantic_result.matched_concepts,
            highlight_snippets: vec![], // TODO: 实现高亮片段提取
        }
    }

    /// 计算综合分数
    fn calculate_combined_score(&self, semantic_score: f32, keyword_score: f32, graph_score: f32) -> f32 {
        let semantic_weighted = semantic_score * self.weights.semantic_weight;
        let keyword_weighted = keyword_score * self.weights.keyword_weight;
        let graph_weighted = graph_score * self.weights.graph_weight;

        semantic_weighted + keyword_weighted + graph_weighted
    }

    /// 提取匹配词汇
    fn extract_matched_terms(&self, query: &str) -> Vec<String> {
        // 简单的词汇分割，后续可以改进
        query
            .split_whitespace()
            .map(|term| term.to_lowercase())
            .filter(|term| !term.is_empty())
            .collect()
    }

    /// 智能搜索建议
    pub async fn get_search_suggestions(&self, _query: &str) -> Result<Vec<SearchSuggestion>, SearchError> {
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
            total_searches: 0,
            avg_search_time_ms: 0.0,
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

    async fn index_note(&self, note: &crate::Note) -> Result<(), SearchError>;
    async fn delete_note(&self, note_id: &str) -> Result<(), SearchError>;
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
    pub total_searches: usize,
    pub avg_search_time_ms: f64,
}

impl Default for SearchStatistics {
    fn default() -> Self {
        Self {
            total_indexed_notes: 0,
            semantic_index_size: 0,
            keyword_index_size: 0,
            last_index_update: None,
            total_searches: 0,
            avg_search_time_ms: 0.0,
        }
    }
}

/// Tantivy适配器（将现有搜索包装为KeywordSearchService）
pub struct TantivyAdapter {
    // TODO: 集成现有的isearch模块
}

impl TantivyAdapter {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl KeywordSearchService for TantivyAdapter {
    async fn search(
        &self,
        _query: &str,
        _limit: usize,
        _filters: Option<SearchFilters>,
    ) -> Result<Vec<KeywordSearchResult>, SearchError> {
        // TODO: 集成现有的Tantivy搜索
        Ok(vec![])
    }

    async fn index_note(&self, _note: &crate::Note) -> Result<(), SearchError> {
        // TODO: 集成现有的Tantivy索引
        Ok(())
    }

    async fn delete_note(&self, _note_id: &str) -> Result<(), SearchError> {
        // TODO: 集成现有的Tantivy删除
        Ok(())
    }
}
