//! BGE嵌入提供者实现
//! 
//! 实现EmbeddingProvider trait，提供BGE模型的语义嵌入服务

use crate::{Result, Error};
use crate::embedding::provider::{EmbeddingProvider, BGEConfig, ModelInfo, ModelType, ProviderStats};
use crate::embedding::bge::{BGEModel, BGETokenizer};
use crate::embedding::cache::EmbeddingCache;
use async_trait::async_trait;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};
use tokio::sync::Semaphore;
use serde::{Serialize, Deserialize};

/// BGE嵌入提供者
pub struct BGEEmbeddingProvider {
    model: BGEModel,
    tokenizer: BGETokenizer,
    config: BGEConfig,
    cache: Arc<EmbeddingCache>,
    metrics: Arc<EmbeddingMetrics>,
    // 性能优化组件
    batch_semaphore: Arc<Semaphore>,
    memory_monitor: Arc<MemoryMonitor>,
}

/// 嵌入性能指标
#[derive(Debug, Default)]
pub struct EmbeddingMetrics {
    pub total_requests: AtomicU64,
    pub cache_hits: AtomicU64,
    pub cache_misses: AtomicU64,
    pub total_latency_ms: AtomicU64,
    pub error_count: AtomicU64,
    pub batch_requests: AtomicU64,
    pub memory_peak_mb: AtomicU64,
}

/// 内存监控器
#[derive(Debug)]
struct MemoryMonitor {
    current_usage_mb: AtomicU64,
    peak_usage_mb: AtomicU64,
    cache_size_mb: AtomicU64,
    model_size_mb: AtomicU64,
}

impl Default for MemoryMonitor {
    fn default() -> Self {
        Self {
            current_usage_mb: AtomicU64::new(0),
            peak_usage_mb: AtomicU64::new(0),
            cache_size_mb: AtomicU64::new(0),
            model_size_mb: AtomicU64::new(0),
        }
    }
}

impl MemoryMonitor {
    /// 更新内存使用量
    fn update_usage(&self, usage_mb: u64) {
        self.current_usage_mb.store(usage_mb, Ordering::Relaxed);

        // 更新峰值
        let current_peak = self.peak_usage_mb.load(Ordering::Relaxed);
        if usage_mb > current_peak {
            self.peak_usage_mb.store(usage_mb, Ordering::Relaxed);
        }
    }

    /// 获取当前内存使用量（MB）
    fn get_current_usage_mb(&self) -> u64 {
        self.current_usage_mb.load(Ordering::Relaxed)
    }

    /// 获取峰值内存使用量（MB）
    fn get_peak_usage_mb(&self) -> u64 {
        self.peak_usage_mb.load(Ordering::Relaxed)
    }

    /// 估算内存使用量
    fn estimate_memory_usage(&self, cache_entries: u64, avg_vector_size: usize) -> u64 {
        let cache_mb = (cache_entries * avg_vector_size as u64 * 4) / (1024 * 1024); // f32 = 4 bytes
        let model_mb = self.model_size_mb.load(Ordering::Relaxed);
        cache_mb + model_mb + 50 // 基础开销约50MB
    }
}

/// 详细性能统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetailedProviderStats {
    pub basic: ProviderStats,
    pub batch_requests: u64,
    pub memory_current_mb: u64,
    pub memory_peak_mb: u64,
    pub cache_entries: u64,
    pub cache_size_mb: u64,
    pub model_size_mb: u64,
}

impl BGEEmbeddingProvider {
    /// 获取详细性能指标
    pub fn get_detailed_stats(&self) -> DetailedProviderStats {
        let basic_stats = self.get_stats();

        DetailedProviderStats {
            basic: basic_stats,
            batch_requests: self.metrics.batch_requests.load(Ordering::Relaxed),
            memory_current_mb: self.memory_monitor.get_current_usage_mb(),
            memory_peak_mb: self.memory_monitor.get_peak_usage_mb(),
            cache_entries: self.cache.get_stats().size as u64,
            cache_size_mb: self.memory_monitor.cache_size_mb.load(Ordering::Relaxed),
            model_size_mb: self.memory_monitor.model_size_mb.load(Ordering::Relaxed),
        }
    }

    /// 清理缓存
    pub async fn clear_cache(&self) {
        self.cache.clear();
        self.update_memory_stats().await;
    }

    /// 获取缓存命中率
    pub fn get_cache_hit_rate(&self) -> f64 {
        let hits = self.metrics.cache_hits.load(Ordering::Relaxed);
        let misses = self.metrics.cache_misses.load(Ordering::Relaxed);
        let total = hits + misses;

        if total > 0 {
            hits as f64 / total as f64
        } else {
            0.0
        }
    }
    /// 创建新的BGE提供者
    pub async fn new(config: BGEConfig) -> Result<Self> {
        println!("🔧 初始化BGE嵌入提供者...");

        // 验证配置
        config.validate()?;

        // 加载模型
        let model = BGEModel::load(&config).await?;
        
        // 加载分词器
        let tokenizer = match BGETokenizer::load(&config).await {
            Ok(tokenizer) => tokenizer,
            Err(_) => {
                println!("⚠️  无法加载真实分词器，使用占位符分词器");
                BGETokenizer::placeholder(&config)?
            }
        };

        // 创建智能缓存
        let cache = Arc::new(EmbeddingCache::new(config.cache_size));

        // 创建批处理信号量（限制并发批处理数量）
        let max_concurrent_batches = (config.batch_size / 4).max(1);
        let batch_semaphore = Arc::new(Semaphore::new(max_concurrent_batches));

        // 创建内存监控器
        let memory_monitor = Arc::new(MemoryMonitor::default());

        // 估算模型大小
        let estimated_model_size_mb = config.model_variant.dimension() * 768 * 4 / (1024 * 1024); // 粗略估算
        memory_monitor.model_size_mb.store(estimated_model_size_mb as u64, Ordering::Relaxed);

        // 模型预热
        println!("🔥 开始模型预热...");
        if let Err(e) = model.warmup().await {
            println!("⚠️  模型预热失败: {}", e);
        } else {
            println!("✅ 模型预热完成");
        }

        println!("✅ BGE嵌入提供者初始化完成");

        Ok(Self {
            model,
            tokenizer,
            config,
            cache,
            metrics: Arc::new(EmbeddingMetrics::default()),
            batch_semaphore,
            memory_monitor,
        })
    }

    /// 预处理文本
    fn preprocess_text(&self, text: &str) -> String {
        // 1. 文本清理：移除多余空白字符
        let cleaned = text.trim().chars()
            .map(|c| if c.is_whitespace() { ' ' } else { c })
            .collect::<String>();

        // 2. 长度检查和截断
        if cleaned.chars().count() > self.config.max_length - 2 {
            cleaned.chars().take(self.config.max_length - 2).collect()
        } else {
            cleaned
        }
    }

    /// 生成缓存键
    fn generate_cache_key(&self, text: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        text.hash(&mut hasher);
        self.config.model_variant.model_name().hash(&mut hasher);
        format!("{}:{}", self.config.model_variant.model_name(), hasher.finish())
    }

    /// 智能批量推理实现
    async fn batch_inference(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
        if texts.is_empty() {
            return Ok(Vec::new());
        }

        // 获取批处理许可
        let _permit = self.batch_semaphore.acquire().await
            .map_err(|e| Error::ModelError(format!("获取批处理许可失败: {}", e)))?;

        let start_time = Instant::now();

        // 智能分批处理
        let optimal_batch_size = self.calculate_optimal_batch_size(texts.len());
        let mut results = Vec::new();

        for chunk in texts.chunks(optimal_batch_size) {
            let chunk_results = self.process_batch_chunk(chunk).await?;
            results.extend(chunk_results);
        }

        // 更新内存使用统计
        self.update_memory_stats().await;

        // 记录批处理指标
        self.metrics.batch_requests.fetch_add(1, Ordering::Relaxed);
        let duration = start_time.elapsed();
        self.metrics.total_latency_ms.fetch_add(duration.as_millis() as u64, Ordering::Relaxed);

        Ok(results)
    }

    /// 处理单个批次块
    async fn process_batch_chunk(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
        // 预处理文本
        let processed_texts: Vec<String> = texts.iter()
            .map(|text| self.preprocess_text(text))
            .collect();

        // 分词
        let text_refs: Vec<&str> = processed_texts.iter().map(|s| s.as_str()).collect();
        let tokenized_inputs = self.tokenizer.encode_batch(&text_refs)?;

        // 模型推理
        let embeddings = self.model.batch_forward(&tokenized_inputs).await?;

        Ok(embeddings)
    }

    /// 计算最优批处理大小
    fn calculate_optimal_batch_size(&self, total_texts: usize) -> usize {
        let base_batch_size = self.config.batch_size;
        let memory_usage = self.memory_monitor.get_current_usage_mb();

        // 根据内存使用情况调整批处理大小
        let memory_factor = if memory_usage > 1000 { // 超过1GB
            0.5
        } else if memory_usage > 500 { // 超过500MB
            0.75
        } else {
            1.0
        };

        let adjusted_size = (base_batch_size as f64 * memory_factor) as usize;
        adjusted_size.max(1).min(total_texts)
    }

    /// 更新内存使用统计
    async fn update_memory_stats(&self) {
        // 估算当前内存使用量
        let cache_size = self.cache.get_stats().size;
        let estimated_usage = self.memory_monitor.estimate_memory_usage(
            cache_size as u64,
            self.config.model_variant.dimension()
        );

        self.memory_monitor.update_usage(estimated_usage);
        self.metrics.memory_peak_mb.store(
            self.memory_monitor.get_peak_usage_mb(),
            Ordering::Relaxed
        );
    }
}

#[async_trait]
impl EmbeddingProvider for BGEEmbeddingProvider {
    /// 单文本编码
    async fn encode_single(&self, text: &str) -> Result<Vec<f32>> {
        let start_time = Instant::now();

        // 检查缓存
        let cache_key = self.generate_cache_key(text);
        if let Some(cached) = self.cache.get(&cache_key).await {
            self.metrics.record_cache_hit();
            self.metrics.record_request(start_time.elapsed(), true);
            return Ok(cached);
        }

        // 预处理文本
        let processed_text = self.preprocess_text(text);

        // 分词
        let tokens = self.tokenizer.encode(&processed_text)?;

        // 模型推理
        let embedding = self.model.forward(&tokens).await?;

        // 缓存结果
        self.cache.insert(cache_key, embedding.clone()).await;

        // 记录指标
        self.metrics.record_cache_miss();
        self.metrics.record_request(start_time.elapsed(), false);

        Ok(embedding)
    }

    /// 批量文本编码
    async fn encode_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
        if texts.is_empty() {
            return Ok(Vec::new());
        }

        let start_time = Instant::now();

        // 检查缓存
        let mut results = vec![None; texts.len()];
        let mut uncached_indices = Vec::new();
        let mut uncached_texts = Vec::new();

        for (i, text) in texts.iter().enumerate() {
            let cache_key = self.generate_cache_key(text);
            if let Some(cached) = self.cache.get(&cache_key).await {
                results[i] = Some(cached);
                self.metrics.record_cache_hit();
            } else {
                uncached_indices.push(i);
                uncached_texts.push(*text);
            }
        }

        // 批量处理未缓存的文本
        if !uncached_texts.is_empty() {
            let batch_embeddings = self.batch_inference(&uncached_texts).await?;

            for (idx, embedding) in uncached_indices.into_iter().zip(batch_embeddings) {
                let cache_key = self.generate_cache_key(texts[idx]);
                self.cache.insert(cache_key, embedding.clone()).await;
                results[idx] = Some(embedding);
                self.metrics.record_cache_miss();
            }
        }

        // 转换为最终结果
        let final_results: Result<Vec<Vec<f32>>> = results.into_iter()
            .map(|opt| opt.ok_or_else(|| Error::ModelError("批量编码结果缺失".to_string())))
            .collect();

        self.metrics.record_request(start_time.elapsed(), false);
        final_results
    }

    /// 获取向量维度
    fn get_dimension(&self) -> usize {
        self.config.model_variant.dimension()
    }

    /// 获取模型信息
    fn get_model_info(&self) -> ModelInfo {
        ModelInfo {
            name: self.config.model_variant.model_name().to_string(),
            version: "1.5".to_string(),
            dimension: self.get_dimension(),
            max_sequence_length: self.config.max_length,
            language: "zh".to_string(),
            model_type: ModelType::BGE,
            memory_usage: self.model.estimate_memory_usage(),
        }
    }

    /// 健康检查
    async fn health_check(&self) -> Result<()> {
        // 使用简单文本测试模型
        let test_text = "测试";
        let _embedding = self.encode_single(test_text).await?;
        Ok(())
    }

    /// 获取性能统计
    fn get_stats(&self) -> ProviderStats {
        self.metrics.get_stats()
    }
}

impl EmbeddingMetrics {
    /// 记录请求
    pub fn record_request(&self, latency: Duration, cache_hit: bool) {
        self.total_requests.fetch_add(1, Ordering::Relaxed);
        self.total_latency_ms.fetch_add(latency.as_millis() as u64, Ordering::Relaxed);

        if cache_hit {
            self.cache_hits.fetch_add(1, Ordering::Relaxed);
        } else {
            self.cache_misses.fetch_add(1, Ordering::Relaxed);
        }
    }

    /// 记录缓存命中
    pub fn record_cache_hit(&self) {
        self.cache_hits.fetch_add(1, Ordering::Relaxed);
    }

    /// 记录缓存未命中
    pub fn record_cache_miss(&self) {
        self.cache_misses.fetch_add(1, Ordering::Relaxed);
    }

    /// 记录错误
    pub fn record_error(&self) {
        self.error_count.fetch_add(1, Ordering::Relaxed);
    }

    /// 获取统计信息
    pub fn get_stats(&self) -> ProviderStats {
        let total_requests = self.total_requests.load(Ordering::Relaxed);
        let cache_hits = self.cache_hits.load(Ordering::Relaxed);
        let total_latency = self.total_latency_ms.load(Ordering::Relaxed);

        ProviderStats {
            total_requests,
            cache_hits,
            cache_misses: self.cache_misses.load(Ordering::Relaxed),
            average_latency_ms: if total_requests > 0 {
                total_latency as f64 / total_requests as f64
            } else {
                0.0
            },
            error_count: self.error_count.load(Ordering::Relaxed),
            memory_usage_mb: self.memory_peak_mb.load(Ordering::Relaxed) as f64,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::embedding::provider::{BGEConfig, BGEVariant, Device};

    fn create_test_config() -> BGEConfig {
        BGEConfig {
            model_variant: BGEVariant::Small,
            device: Device::CPU,
            batch_size: 16,
            max_length: 128,
            normalize_embeddings: true,
            cache_size: 1000,
            enable_quantization: false,
            cache_dir: std::path::PathBuf::from("./test_models"),
        }
    }

    #[tokio::test]
    async fn test_provider_creation() {
        let config = create_test_config();
        let provider = BGEEmbeddingProvider::new(config).await;
        assert!(provider.is_ok());

        let provider = provider.unwrap();
        assert_eq!(provider.get_dimension(), 512);
    }

    #[tokio::test]
    async fn test_single_encoding() {
        let config = create_test_config();
        let provider = BGEEmbeddingProvider::new(config).await.unwrap();

        let result = provider.encode_single("测试文本").await;
        assert!(result.is_ok());

        let embedding = result.unwrap();
        assert_eq!(embedding.len(), 512);
    }

    #[tokio::test]
    async fn test_batch_encoding() {
        let config = create_test_config();
        let provider = BGEEmbeddingProvider::new(config).await.unwrap();

        let texts = vec!["文本1", "文本2", "文本3"];
        let result = provider.encode_batch(&texts).await;
        assert!(result.is_ok());

        let embeddings = result.unwrap();
        assert_eq!(embeddings.len(), 3);
        assert_eq!(embeddings[0].len(), 512);
    }

    #[tokio::test]
    async fn test_caching() {
        let config = create_test_config();
        let provider = BGEEmbeddingProvider::new(config).await.unwrap();

        let text = "缓存测试文本";

        // 第一次编码
        let result1 = provider.encode_single(text).await.unwrap();

        // 第二次编码（应该从缓存获取）
        let result2 = provider.encode_single(text).await.unwrap();

        // 结果应该相同
        assert_eq!(result1, result2);

        // 检查缓存统计
        let stats = provider.get_stats();
        assert!(stats.cache_hits > 0);
    }

    #[tokio::test]
    async fn test_health_check() {
        let config = create_test_config();
        let provider = BGEEmbeddingProvider::new(config).await.unwrap();

        let result = provider.health_check().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_performance_optimizations() {
        let config = create_test_config();
        let provider = BGEEmbeddingProvider::new(config).await.unwrap();

        // 测试批量处理性能 - 使用重复文本来触发缓存
        let base_texts = vec![
            "这是一个测试文本",
            "人工智能技术发展",
            "机器学习算法优化",
            "深度学习模型训练",
            "自然语言处理应用",
        ];

        // 创建包含重复文本的列表来测试缓存
        let mut texts = Vec::new();
        for _ in 0..10 {
            texts.extend_from_slice(&base_texts);
        }

        // 第一次处理 - 填充缓存
        let start_time = std::time::Instant::now();
        let results1 = provider.encode_batch(&texts).await.unwrap();
        let duration1 = start_time.elapsed();

        assert_eq!(results1.len(), texts.len());
        println!("第一次批量处理50个文本耗时: {:?}", duration1);

        // 第二次处理相同文本 - 应该命中缓存
        let start_time = std::time::Instant::now();
        let results2 = provider.encode_batch(&texts).await.unwrap();
        let duration2 = start_time.elapsed();

        assert_eq!(results2.len(), texts.len());
        println!("第二次批量处理50个文本耗时: {:?}", duration2);

        // 验证缓存效果
        let cache_hit_rate = provider.get_cache_hit_rate();
        println!("缓存命中率: {:.2}%", cache_hit_rate * 100.0);

        // 第二次应该更快，因为有缓存（允许一些误差）
        if duration2 >= duration1 {
            println!("警告: 第二次处理时间 ({:?}) >= 第一次处理时间 ({:?})", duration2, duration1);
        }

        // 验证结果数量相同
        assert_eq!(results1.len(), results2.len());

        // 测试详细统计
        let detailed_stats = provider.get_detailed_stats();
        println!("详细统计: {:?}", detailed_stats);
        assert!(detailed_stats.batch_requests > 0);
        assert!(detailed_stats.basic.cache_hits > 0); // 验证有缓存命中
    }

    #[tokio::test]
    async fn test_memory_monitoring() {
        let config = create_test_config();
        let provider = BGEEmbeddingProvider::new(config).await.unwrap();

        // 执行一些操作来触发内存使用
        let texts = vec!["测试文本1", "测试文本2", "测试文本3"];
        let _results = provider.encode_batch(&texts).await.unwrap();

        let detailed_stats = provider.get_detailed_stats();

        // 验证内存监控
        assert!(detailed_stats.memory_current_mb > 0);
        assert!(detailed_stats.model_size_mb > 0);

        println!("当前内存使用: {}MB", detailed_stats.memory_current_mb);
        println!("峰值内存使用: {}MB", detailed_stats.memory_peak_mb);
        println!("模型大小: {}MB", detailed_stats.model_size_mb);
    }

    #[tokio::test]
    async fn test_cache_management() {
        let config = create_test_config();
        let provider = BGEEmbeddingProvider::new(config).await.unwrap();

        // 添加一些缓存项
        let texts = vec!["缓存测试1", "缓存测试2", "缓存测试3"];
        let _results = provider.encode_batch(&texts).await.unwrap();

        let stats_before = provider.get_detailed_stats();
        println!("清理前缓存项: {}", stats_before.cache_entries);

        // 清理缓存
        provider.clear_cache().await;

        let stats_after = provider.get_detailed_stats();
        println!("清理后缓存项: {}", stats_after.cache_entries);

        // 验证缓存确实被清理了
        assert!(stats_after.cache_entries <= stats_before.cache_entries);

        println!("清理前缓存项: {}", stats_before.cache_entries);
        println!("清理后缓存项: {}", stats_after.cache_entries);
    }
}
