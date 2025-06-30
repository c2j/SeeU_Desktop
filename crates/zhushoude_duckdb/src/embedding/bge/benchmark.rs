//! BGE性能基准测试
//! 
//! 验证BGE中文语义模型的正确性和性能

use crate::{Result, Error};
use crate::embedding::provider::{EmbeddingProvider, BGEConfig};
use crate::embedding::bge::BGEEmbeddingProvider;
use std::time::Instant;
use serde::{Serialize, Deserialize};

/// 基准测试结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkResult {
    pub test_name: String,
    pub success: bool,
    pub duration_ms: u64,
    pub throughput_texts_per_sec: f64,
    pub memory_usage_mb: u64,
    pub cache_hit_rate: f64,
    pub error_message: Option<String>,
}

/// 语义相似性测试用例
#[derive(Debug, Clone)]
pub struct SemanticTestCase {
    pub text1: &'static str,
    pub text2: &'static str,
    pub expected_similarity: f64, // 0.0 - 1.0
    pub tolerance: f64,
}

/// BGE基准测试套件
pub struct BGEBenchmark {
    provider: BGEEmbeddingProvider,
}

impl BGEBenchmark {
    /// 创建新的基准测试
    pub async fn new() -> Result<Self> {
        let config = BGEConfig::lightweight();
        let provider = BGEEmbeddingProvider::new(config).await?;
        
        Ok(Self { provider })
    }

    /// 运行完整的基准测试套件
    pub async fn run_full_benchmark(&self) -> Result<Vec<BenchmarkResult>> {
        let mut results = Vec::new();

        // 1. 基础功能测试
        results.push(self.test_basic_encoding().await);
        results.push(self.test_batch_encoding().await);
        results.push(self.test_chinese_text_support().await);

        // 2. 性能测试
        results.push(self.test_throughput_performance().await);
        results.push(self.test_cache_performance().await);
        results.push(self.test_memory_efficiency().await);

        // 3. 语义相似性测试
        results.push(self.test_semantic_similarity().await);

        Ok(results)
    }

    /// 测试基础编码功能
    async fn test_basic_encoding(&self) -> BenchmarkResult {
        let start_time = Instant::now();
        
        match self.provider.encode_single("这是一个测试文本").await {
            Ok(embedding) => {
                let duration = start_time.elapsed();
                BenchmarkResult {
                    test_name: "基础编码功能".to_string(),
                    success: !embedding.is_empty(),
                    duration_ms: duration.as_millis() as u64,
                    throughput_texts_per_sec: 1000.0 / duration.as_millis() as f64,
                    memory_usage_mb: self.provider.get_detailed_stats().memory_current_mb,
                    cache_hit_rate: 0.0,
                    error_message: None,
                }
            }
            Err(e) => BenchmarkResult {
                test_name: "基础编码功能".to_string(),
                success: false,
                duration_ms: start_time.elapsed().as_millis() as u64,
                throughput_texts_per_sec: 0.0,
                memory_usage_mb: 0,
                cache_hit_rate: 0.0,
                error_message: Some(e.to_string()),
            }
        }
    }

    /// 测试批量编码功能
    async fn test_batch_encoding(&self) -> BenchmarkResult {
        let texts = vec![
            "人工智能技术",
            "机器学习算法",
            "深度学习模型",
            "自然语言处理",
            "计算机视觉",
        ];

        let start_time = Instant::now();
        
        match self.provider.encode_batch(&texts).await {
            Ok(embeddings) => {
                let duration = start_time.elapsed();
                BenchmarkResult {
                    test_name: "批量编码功能".to_string(),
                    success: embeddings.len() == texts.len(),
                    duration_ms: duration.as_millis() as u64,
                    throughput_texts_per_sec: texts.len() as f64 * 1000.0 / duration.as_millis() as f64,
                    memory_usage_mb: self.provider.get_detailed_stats().memory_current_mb,
                    cache_hit_rate: self.provider.get_cache_hit_rate(),
                    error_message: None,
                }
            }
            Err(e) => BenchmarkResult {
                test_name: "批量编码功能".to_string(),
                success: false,
                duration_ms: start_time.elapsed().as_millis() as u64,
                throughput_texts_per_sec: 0.0,
                memory_usage_mb: 0,
                cache_hit_rate: 0.0,
                error_message: Some(e.to_string()),
            }
        }
    }

    /// 测试中文文本支持
    async fn test_chinese_text_support(&self) -> BenchmarkResult {
        let chinese_texts = vec![
            "中华人民共和国成立于1949年",
            "北京是中国的首都",
            "长江是中国最长的河流",
            "汉语是世界上使用人数最多的语言",
        ];

        let start_time = Instant::now();
        
        match self.provider.encode_batch(&chinese_texts).await {
            Ok(embeddings) => {
                let duration = start_time.elapsed();
                let all_valid = embeddings.iter().all(|e| !e.is_empty());
                
                BenchmarkResult {
                    test_name: "中文文本支持".to_string(),
                    success: all_valid && embeddings.len() == chinese_texts.len(),
                    duration_ms: duration.as_millis() as u64,
                    throughput_texts_per_sec: chinese_texts.len() as f64 * 1000.0 / duration.as_millis() as f64,
                    memory_usage_mb: self.provider.get_detailed_stats().memory_current_mb,
                    cache_hit_rate: self.provider.get_cache_hit_rate(),
                    error_message: None,
                }
            }
            Err(e) => BenchmarkResult {
                test_name: "中文文本支持".to_string(),
                success: false,
                duration_ms: start_time.elapsed().as_millis() as u64,
                throughput_texts_per_sec: 0.0,
                memory_usage_mb: 0,
                cache_hit_rate: 0.0,
                error_message: Some(e.to_string()),
            }
        }
    }

    /// 测试吞吐量性能
    async fn test_throughput_performance(&self) -> BenchmarkResult {
        // 生成100个测试文本
        let texts: Vec<String> = (0..100).map(|i| {
            format!("这是第{}个测试文本，用于验证BGE模型的处理性能", i)
        }).collect();
        
        let text_refs: Vec<&str> = texts.iter().map(|s| s.as_str()).collect();
        let start_time = Instant::now();
        
        match self.provider.encode_batch(&text_refs).await {
            Ok(embeddings) => {
                let duration = start_time.elapsed();
                let throughput = texts.len() as f64 * 1000.0 / duration.as_millis() as f64;
                
                BenchmarkResult {
                    test_name: "吞吐量性能测试".to_string(),
                    success: embeddings.len() == texts.len() && throughput > 10.0, // 至少10文本/秒
                    duration_ms: duration.as_millis() as u64,
                    throughput_texts_per_sec: throughput,
                    memory_usage_mb: self.provider.get_detailed_stats().memory_current_mb,
                    cache_hit_rate: self.provider.get_cache_hit_rate(),
                    error_message: None,
                }
            }
            Err(e) => BenchmarkResult {
                test_name: "吞吐量性能测试".to_string(),
                success: false,
                duration_ms: start_time.elapsed().as_millis() as u64,
                throughput_texts_per_sec: 0.0,
                memory_usage_mb: 0,
                cache_hit_rate: 0.0,
                error_message: Some(e.to_string()),
            }
        }
    }

    /// 测试缓存性能
    async fn test_cache_performance(&self) -> BenchmarkResult {
        let texts = vec!["缓存测试文本1", "缓存测试文本2", "缓存测试文本3"];
        
        // 第一次编码
        let _ = self.provider.encode_batch(&texts).await;
        
        // 第二次编码（应该命中缓存）
        let start_time = Instant::now();
        match self.provider.encode_batch(&texts).await {
            Ok(_) => {
                let duration = start_time.elapsed();
                let cache_hit_rate = self.provider.get_cache_hit_rate();
                
                BenchmarkResult {
                    test_name: "缓存性能测试".to_string(),
                    success: cache_hit_rate > 0.0 && duration.as_millis() < 100, // 缓存命中且快速
                    duration_ms: duration.as_millis() as u64,
                    throughput_texts_per_sec: texts.len() as f64 * 1000.0 / duration.as_millis() as f64,
                    memory_usage_mb: self.provider.get_detailed_stats().memory_current_mb,
                    cache_hit_rate,
                    error_message: None,
                }
            }
            Err(e) => BenchmarkResult {
                test_name: "缓存性能测试".to_string(),
                success: false,
                duration_ms: start_time.elapsed().as_millis() as u64,
                throughput_texts_per_sec: 0.0,
                memory_usage_mb: 0,
                cache_hit_rate: 0.0,
                error_message: Some(e.to_string()),
            }
        }
    }

    /// 测试内存效率
    async fn test_memory_efficiency(&self) -> BenchmarkResult {
        let start_time = Instant::now();
        let initial_memory = self.provider.get_detailed_stats().memory_current_mb;
        
        // 处理一批文本
        let texts: Vec<String> = (0..50).map(|i| {
            format!("内存测试文本{}", i)
        }).collect();
        let text_refs: Vec<&str> = texts.iter().map(|s| s.as_str()).collect();
        
        match self.provider.encode_batch(&text_refs).await {
            Ok(_) => {
                let final_memory = self.provider.get_detailed_stats().memory_current_mb;
                let memory_increase = final_memory.saturating_sub(initial_memory);
                
                BenchmarkResult {
                    test_name: "内存效率测试".to_string(),
                    success: memory_increase < 100, // 内存增长小于100MB
                    duration_ms: start_time.elapsed().as_millis() as u64,
                    throughput_texts_per_sec: texts.len() as f64 * 1000.0 / start_time.elapsed().as_millis() as f64,
                    memory_usage_mb: final_memory,
                    cache_hit_rate: self.provider.get_cache_hit_rate(),
                    error_message: None,
                }
            }
            Err(e) => BenchmarkResult {
                test_name: "内存效率测试".to_string(),
                success: false,
                duration_ms: start_time.elapsed().as_millis() as u64,
                throughput_texts_per_sec: 0.0,
                memory_usage_mb: 0,
                cache_hit_rate: 0.0,
                error_message: Some(e.to_string()),
            }
        }
    }

    /// 测试语义相似性
    async fn test_semantic_similarity(&self) -> BenchmarkResult {
        let test_cases = vec![
            SemanticTestCase {
                text1: "科学家",
                text2: "研究人员",
                expected_similarity: 0.7,
                tolerance: 0.3,
            },
            SemanticTestCase {
                text1: "人工智能",
                text2: "机器学习",
                expected_similarity: 0.8,
                tolerance: 0.3,
            },
            SemanticTestCase {
                text1: "苹果",
                text2: "汽车",
                expected_similarity: 0.1,
                tolerance: 0.3,
            },
        ];

        let start_time = Instant::now();
        let mut success_count = 0;
        
        for case in &test_cases {
            if let (Ok(emb1), Ok(emb2)) = (
                self.provider.encode_single(case.text1).await,
                self.provider.encode_single(case.text2).await,
            ) {
                let similarity = cosine_similarity(&emb1, &emb2);
                let diff = (similarity - case.expected_similarity).abs();
                
                if diff <= case.tolerance {
                    success_count += 1;
                }
            }
        }

        BenchmarkResult {
            test_name: "语义相似性测试".to_string(),
            success: success_count >= test_cases.len() / 2, // 至少一半测试通过
            duration_ms: start_time.elapsed().as_millis() as u64,
            throughput_texts_per_sec: (test_cases.len() * 2) as f64 * 1000.0 / start_time.elapsed().as_millis() as f64,
            memory_usage_mb: self.provider.get_detailed_stats().memory_current_mb,
            cache_hit_rate: self.provider.get_cache_hit_rate(),
            error_message: None,
        }
    }
}

/// 计算余弦相似度
fn cosine_similarity(a: &[f32], b: &[f32]) -> f64 {
    if a.len() != b.len() {
        return 0.0;
    }

    let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

    if norm_a == 0.0 || norm_b == 0.0 {
        0.0
    } else {
        (dot_product / (norm_a * norm_b)) as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_bge_benchmark_suite() {
        let benchmark = BGEBenchmark::new().await.unwrap();
        let results = benchmark.run_full_benchmark().await.unwrap();

        println!("\n=== BGE基准测试结果 ===");
        for result in &results {
            println!("测试: {}", result.test_name);
            println!("  成功: {}", result.success);
            println!("  耗时: {}ms", result.duration_ms);
            println!("  吞吐量: {:.2} 文本/秒", result.throughput_texts_per_sec);
            println!("  内存使用: {}MB", result.memory_usage_mb);
            println!("  缓存命中率: {:.2}%", result.cache_hit_rate * 100.0);
            if let Some(error) = &result.error_message {
                println!("  错误: {}", error);
            }
            println!();
        }

        // 验证关键测试通过
        let critical_tests = ["基础编码功能", "批量编码功能", "中文文本支持"];
        for test_name in &critical_tests {
            let result = results.iter().find(|r| r.test_name == *test_name);
            assert!(result.is_some(), "缺少关键测试: {}", test_name);
            assert!(result.unwrap().success, "关键测试失败: {}", test_name);
        }
    }
}
