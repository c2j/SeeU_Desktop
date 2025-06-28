//! 结果排序模块

use crate::{Result, HybridResult, FusionStrategy};

/// 结果排序器
pub struct ResultRanker {
    strategy: FusionStrategy,
}

impl ResultRanker {
    /// 创建新的结果排序器
    pub fn new(strategy: FusionStrategy) -> Self {
        Self { strategy }
    }
    
    /// 对结果进行排序
    pub fn rank_results(&self, results: &mut Vec<HybridResult>) -> Result<()> {
        match self.strategy {
            FusionStrategy::WeightedAverage => self.weighted_average_ranking(results),
            FusionStrategy::ReciprocalRankFusion => self.reciprocal_rank_fusion(results),
            FusionStrategy::MaxFusion => self.max_fusion_ranking(results),
        }
    }
    
    /// 加权平均排序
    fn weighted_average_ranking(&self, results: &mut Vec<HybridResult>) -> Result<()> {
        // final_score已经在搜索时计算
        results.sort_by(|a, b| b.final_score.partial_cmp(&a.final_score).unwrap());
        Ok(())
    }
    
    /// 倒数排名融合
    fn reciprocal_rank_fusion(&self, results: &mut Vec<HybridResult>) -> Result<()> {
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
        let k = 60.0;
        for (i, result) in results.iter_mut().enumerate() {
            let mut rrf_score = 0.0;

            // 查找在语义搜索中的排名
            if let Some(rank) = semantic_indices.iter().position(|&idx| idx == i) {
                rrf_score += 1.0 / (k + rank as f64 + 1.0);
            }

            // 查找在图搜索中的排名
            if let Some(rank) = graph_indices.iter().position(|&idx| idx == i) {
                rrf_score += 1.0 / (k + rank as f64 + 1.0);
            }

            result.final_score = rrf_score;
        }

        results.sort_by(|a, b| b.final_score.partial_cmp(&a.final_score).unwrap());
        Ok(())
    }
    
    /// 最大值融合排序
    fn max_fusion_ranking(&self, results: &mut Vec<HybridResult>) -> Result<()> {
        for result in results.iter_mut() {
            result.final_score = result.score;
        }
        results.sort_by(|a, b| b.final_score.partial_cmp(&a.final_score).unwrap());
        Ok(())
    }
    
    /// 归一化分数
    pub fn normalize_scores(&self, results: &mut Vec<HybridResult>) -> Result<()> {
        if results.is_empty() {
            return Ok(());
        }
        
        let max_score = results.iter()
            .map(|r| r.final_score)
            .fold(0.0f64, f64::max);
        
        let min_score = results.iter()
            .map(|r| r.final_score)
            .fold(f64::INFINITY, f64::min);
        
        let range = max_score - min_score;
        
        if range > 0.0 {
            for result in results.iter_mut() {
                result.final_score = (result.final_score - min_score) / range;
            }
        }
        
        Ok(())
    }
    
    /// 应用阈值过滤
    pub fn apply_threshold(&self, results: &mut Vec<HybridResult>, threshold: f64) -> Result<()> {
        results.retain(|r| r.final_score >= threshold);
        Ok(())
    }
    
    /// 多样性重排序
    pub fn diversify_results(&self, results: &mut Vec<HybridResult>, diversity_factor: f64) -> Result<()> {
        if results.len() <= 1 {
            return Ok(());
        }
        
        let mut diversified = Vec::new();
        let mut remaining = results.clone();
        
        // 选择分数最高的作为第一个
        if let Some(best_idx) = remaining.iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.final_score.partial_cmp(&b.final_score).unwrap())
            .map(|(idx, _)| idx) {
            let best = remaining.remove(best_idx);
            diversified.push(best);
        }
        
        // 基于多样性选择剩余结果
        while !remaining.is_empty() && diversified.len() < results.len() {
            let mut best_candidate = None;
            let mut best_score = f64::NEG_INFINITY;
            
            for candidate in &remaining {
                // 计算多样性分数
                let diversity_score = self.calculate_diversity_score(candidate, &diversified);
                let combined_score = candidate.final_score * (1.0 - diversity_factor) + 
                                   diversity_score * diversity_factor;
                
                if combined_score > best_score {
                    best_score = combined_score;
                    best_candidate = Some(candidate.clone());
                }
            }
            
            if let Some(candidate) = best_candidate {
                remaining.retain(|r| r.document_id != candidate.document_id);
                diversified.push(candidate);
            } else {
                break;
            }
        }
        
        *results = diversified;
        Ok(())
    }
    
    /// 计算多样性分数
    fn calculate_diversity_score(&self, candidate: &HybridResult, selected: &[HybridResult]) -> f64 {
        if selected.is_empty() {
            return 1.0;
        }
        
        // 简单的基于内容长度差异的多样性计算
        let candidate_len = candidate.content.len() as f64;
        let avg_len = selected.iter()
            .map(|r| r.content.len() as f64)
            .sum::<f64>() / selected.len() as f64;
        
        let diversity = (candidate_len - avg_len).abs() / avg_len.max(1.0);
        diversity.min(1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ResultType;
    
    #[test]
    fn test_weighted_average_ranking() {
        let ranker = ResultRanker::new(FusionStrategy::WeightedAverage);
        let mut results = vec![
            HybridResult {
                document_id: "doc1".to_string(),
                title: "Test1".to_string(),
                content: "Content1".to_string(),
                score: 0.8,
                final_score: 0.8,
                result_type: ResultType::Semantic,
                metadata: serde_json::json!({}),
            },
            HybridResult {
                document_id: "doc2".to_string(),
                title: "Test2".to_string(),
                content: "Content2".to_string(),
                score: 0.9,
                final_score: 0.9,
                result_type: ResultType::Graph,
                metadata: serde_json::json!({}),
            },
        ];
        
        let result = ranker.rank_results(&mut results);
        assert!(result.is_ok());
        assert_eq!(results[0].document_id, "doc2");
        assert_eq!(results[1].document_id, "doc1");
    }
    
    #[test]
    fn test_normalize_scores() {
        let ranker = ResultRanker::new(FusionStrategy::WeightedAverage);
        let mut results = vec![
            HybridResult {
                document_id: "doc1".to_string(),
                title: "Test1".to_string(),
                content: "Content1".to_string(),
                score: 0.5,
                final_score: 0.5,
                result_type: ResultType::Semantic,
                metadata: serde_json::json!({}),
            },
            HybridResult {
                document_id: "doc2".to_string(),
                title: "Test2".to_string(),
                content: "Content2".to_string(),
                score: 1.0,
                final_score: 1.0,
                result_type: ResultType::Graph,
                metadata: serde_json::json!({}),
            },
        ];
        
        let result = ranker.normalize_scores(&mut results);
        assert!(result.is_ok());
        assert_eq!(results[0].final_score, 0.0);
        assert_eq!(results[1].final_score, 1.0);
    }
    
    #[test]
    fn test_apply_threshold() {
        let ranker = ResultRanker::new(FusionStrategy::WeightedAverage);
        let mut results = vec![
            HybridResult {
                document_id: "doc1".to_string(),
                title: "Test1".to_string(),
                content: "Content1".to_string(),
                score: 0.3,
                final_score: 0.3,
                result_type: ResultType::Semantic,
                metadata: serde_json::json!({}),
            },
            HybridResult {
                document_id: "doc2".to_string(),
                title: "Test2".to_string(),
                content: "Content2".to_string(),
                score: 0.8,
                final_score: 0.8,
                result_type: ResultType::Graph,
                metadata: serde_json::json!({}),
            },
        ];
        
        let result = ranker.apply_threshold(&mut results, 0.5);
        assert!(result.is_ok());
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].document_id, "doc2");
    }
}
