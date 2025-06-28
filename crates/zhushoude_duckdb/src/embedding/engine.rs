//! 语义嵌入引擎

use crate::{Result, EmbeddingConfig, BgeSmallZhModel, EmbeddingCache, ChineseTextProcessor};
use std::sync::Arc;

/// 语义嵌入引擎
pub struct EmbeddingEngine {
    model: BgeSmallZhModel,
    cache: EmbeddingCache,
    processor: ChineseTextProcessor,
    config: EmbeddingConfig,
}

impl EmbeddingEngine {
    /// 创建新的嵌入引擎
    pub async fn new(config: EmbeddingConfig) -> Result<Self> {
        println!("🔧 初始化语义嵌入引擎...");

        let model = BgeSmallZhModel::new(config.clone()).await?;
        let cache = EmbeddingCache::new(config.max_cache_size);
        let processor = ChineseTextProcessor::new();

        println!("✅ 语义嵌入引擎初始化完成");

        Ok(Self {
            model,
            cache,
            processor,
            config,
        })
    }

    /// 对单个文本进行向量化
    pub async fn encode_single(&self, text: &str) -> Result<Vec<f32>> {
        // 检查缓存
        let cache_key = self.generate_cache_key(text);
        if let Some(cached_embedding) = self.cache.get(&cache_key).await {
            return Ok(cached_embedding);
        }

        // 预处理文本
        let processed_text = if self.config.enable_chinese_optimization {
            self.processor.preprocess(text)
        } else {
            text.to_string()
        };

        // 使用模型进行编码
        let embedding = self.model.encode_single(&processed_text).await?;

        // 缓存结果
        self.cache.insert(cache_key, embedding.clone()).await;

        Ok(embedding)
    }

    /// 对多个文本进行批量向量化
    pub async fn encode_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        let mut results = Vec::with_capacity(texts.len());
        let mut uncached_texts = Vec::new();
        let mut uncached_indices = Vec::new();

        // 检查缓存
        for (i, text) in texts.iter().enumerate() {
            let cache_key = self.generate_cache_key(text);
            if let Some(cached_embedding) = self.cache.get(&cache_key).await {
                results.push(Some(cached_embedding));
            } else {
                results.push(None);
                uncached_texts.push(text.clone());
                uncached_indices.push(i);
            }
        }

        // 批量处理未缓存的文本
        if !uncached_texts.is_empty() {
            let processed_texts: Vec<String> = if self.config.enable_chinese_optimization {
                uncached_texts.iter()
                    .map(|text| self.processor.preprocess(text))
                    .collect()
            } else {
                uncached_texts
            };

            let embeddings = self.model.encode_batch(&processed_texts).await?;

            // 更新结果和缓存
            for (i, embedding) in embeddings.into_iter().enumerate() {
                let original_index = uncached_indices[i];
                let cache_key = self.generate_cache_key(&texts[original_index]);

                self.cache.insert(cache_key, embedding.clone()).await;
                results[original_index] = Some(embedding);
            }
        }

        // 转换为最终结果
        Ok(results.into_iter().map(|opt| opt.unwrap()).collect())
    }

    /// 生成缓存键
    fn generate_cache_key(&self, text: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        text.hash(&mut hasher);
        self.config.model_name.hash(&mut hasher);
        format!("{}_{}", self.config.model_name, hasher.finish())
    }

    /// 获取缓存统计
    pub fn get_cache_stats(&self) -> crate::types::CacheStats {
        self.cache.get_stats()
    }

    /// 清理缓存
    pub fn clear_cache(&self) {
        self.cache.clear();
    }

    /// 获取配置
    pub fn get_config(&self) -> &EmbeddingConfig {
        &self.config
    }

    /// 检查模型是否已加载
    pub fn is_model_loaded(&self) -> bool {
        self.model.is_loaded()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_embedding_engine_creation() {
        let config = EmbeddingConfig::default();
        let engine = EmbeddingEngine::new(config).await;
        assert!(engine.is_ok());
    }

    #[tokio::test]
    async fn test_single_encoding() {
        let config = EmbeddingConfig::default();
        let engine = EmbeddingEngine::new(config.clone()).await.unwrap();

        let result = engine.encode_single("测试文本").await;
        assert!(result.is_ok());

        let embedding = result.unwrap();
        assert_eq!(embedding.len(), config.vector_dimension);
    }

    #[tokio::test]
    async fn test_batch_encoding() {
        let config = EmbeddingConfig::default();
        let engine = EmbeddingEngine::new(config.clone()).await.unwrap();

        let texts = vec!["文本1".to_string(), "文本2".to_string()];
        let result = engine.encode_batch(&texts).await;
        assert!(result.is_ok());

        let embeddings = result.unwrap();
        assert_eq!(embeddings.len(), 2);
        assert_eq!(embeddings[0].len(), config.vector_dimension);
        assert_eq!(embeddings[1].len(), config.vector_dimension);
    }

    #[tokio::test]
    async fn test_caching() {
        let config = EmbeddingConfig {
            max_cache_size: 10,
            ..Default::default()
        };
        let engine = EmbeddingEngine::new(config).await.unwrap();

        let text = "缓存测试文本";

        // 第一次编码
        let result1 = engine.encode_single(text).await;
        assert!(result1.is_ok());

        // 第二次编码（应该从缓存获取）
        let result2 = engine.encode_single(text).await;
        assert!(result2.is_ok());

        // 结果应该相同
        assert_eq!(result1.unwrap(), result2.unwrap());

        // 检查缓存统计
        let stats = engine.get_cache_stats();
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 1);
    }

    #[tokio::test]
    async fn test_chinese_optimization() {
        let config = EmbeddingConfig {
            enable_chinese_optimization: true,
            normalize_vectors: true,
            ..Default::default()
        };
        let engine = EmbeddingEngine::new(config).await.unwrap();

        // 测试繁体中文
        let traditional_text = "機器學習很有趣";
        let result = engine.encode_single(traditional_text).await;
        assert!(result.is_ok());

        let embedding = result.unwrap();

        // 检查向量是否已归一化
        let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((norm - 1.0).abs() < 0.001);
    }

    #[tokio::test]
    async fn test_cache_key_generation() {
        let config = EmbeddingConfig::default();
        let engine = EmbeddingEngine::new(config).await.unwrap();

        let text1 = "测试文本1";
        let text2 = "测试文本2";

        let key1 = engine.generate_cache_key(text1);
        let key2 = engine.generate_cache_key(text2);
        let key3 = engine.generate_cache_key(text1); // 相同文本

        assert_ne!(key1, key2); // 不同文本应该有不同的键
        assert_eq!(key1, key3); // 相同文本应该有相同的键
    }
}
