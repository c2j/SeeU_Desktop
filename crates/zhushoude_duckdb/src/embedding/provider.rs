//! 嵌入提供者抽象层
//! 
//! 定义统一的嵌入提供者接口，支持BGE和外部服务

use crate::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// 嵌入提供者统一接口
#[async_trait]
pub trait EmbeddingProvider: Send + Sync {
    /// 单文本编码
    async fn encode_single(&self, text: &str) -> Result<Vec<f32>>;

    /// 批量文本编码
    async fn encode_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>>;

    /// 获取向量维度
    fn get_dimension(&self) -> usize;

    /// 获取模型信息
    fn get_model_info(&self) -> ModelInfo;

    /// 健康检查
    async fn health_check(&self) -> Result<()>;

    /// 获取性能统计
    fn get_stats(&self) -> ProviderStats;
}

/// 模型信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub name: String,
    pub version: String,
    pub dimension: usize,
    pub max_sequence_length: usize,
    pub language: String,
    pub model_type: ModelType,
    pub memory_usage: usize,
}

/// 模型类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModelType {
    BGE,
    External,
    Custom,
}

/// 提供者性能统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderStats {
    pub total_requests: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub average_latency_ms: f64,
    pub error_count: u64,
    pub memory_usage_mb: f64,
}

impl Default for ProviderStats {
    fn default() -> Self {
        Self {
            total_requests: 0,
            cache_hits: 0,
            cache_misses: 0,
            average_latency_ms: 0.0,
            error_count: 0,
            memory_usage_mb: 0.0,
        }
    }
}

/// 嵌入提供者配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EmbeddingProviderConfig {
    BGE(BGEConfig),
    External(ExternalProviderConfig),
}

/// BGE模型配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BGEConfig {
    /// 模型变体
    pub model_variant: BGEVariant,
    /// 计算设备
    pub device: Device,
    /// 批处理大小
    pub batch_size: usize,
    /// 最大序列长度
    pub max_length: usize,
    /// 是否归一化向量
    pub normalize_embeddings: bool,
    /// 缓存大小
    pub cache_size: usize,
    /// 是否启用量化
    pub enable_quantization: bool,
    /// 模型缓存目录
    pub cache_dir: std::path::PathBuf,
}

/// BGE模型变体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BGEVariant {
    /// bge-small-zh-v1.5 (33M参数, 512维)
    Small,
    /// bge-base-zh-v1.5 (102M参数, 768维)
    Base,
    /// bge-large-zh-v1.5 (326M参数, 1024维)
    Large,
}

/// 计算设备
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Device {
    CPU,
    CUDA(usize),
    Metal,
}

/// 外部提供者配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalProviderConfig {
    pub provider_type: ExternalProviderType,
    pub api_key: String,
    pub base_url: Option<String>,
    pub model_name: String,
    pub timeout: Duration,
    pub max_retries: usize,
}

/// 外部提供者类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExternalProviderType {
    OpenAI,
    HuggingFace,
    Custom,
}

impl Default for BGEConfig {
    fn default() -> Self {
        Self {
            model_variant: BGEVariant::Small,
            device: Device::CPU,
            batch_size: 32,
            max_length: 512,
            normalize_embeddings: true,
            cache_size: 10000,
            enable_quantization: true,
            cache_dir: std::path::PathBuf::from("./models/bge"),
        }
    }
}

impl BGEVariant {
    /// 获取模型维度
    pub fn dimension(&self) -> usize {
        match self {
            BGEVariant::Small => 512,
            BGEVariant::Base => 768,
            BGEVariant::Large => 1024,
        }
    }

    /// 获取模型名称
    pub fn model_name(&self) -> &'static str {
        match self {
            BGEVariant::Small => "bge-small-zh-v1.5",
            BGEVariant::Base => "bge-base-zh-v1.5",
            BGEVariant::Large => "bge-large-zh-v1.5",
        }
    }

    /// 获取预估内存使用量(MB)
    pub fn estimated_memory_mb(&self) -> usize {
        match self {
            BGEVariant::Small => 150,
            BGEVariant::Base => 300,
            BGEVariant::Large => 800,
        }
    }
}

impl std::fmt::Display for BGEVariant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.model_name())
    }
}

impl Default for EmbeddingProviderConfig {
    fn default() -> Self {
        Self::BGE(BGEConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bge_variant_properties() {
        assert_eq!(BGEVariant::Small.dimension(), 512);
        assert_eq!(BGEVariant::Base.dimension(), 768);
        assert_eq!(BGEVariant::Large.dimension(), 1024);

        assert_eq!(BGEVariant::Small.model_name(), "bge-small-zh-v1.5");
        assert_eq!(BGEVariant::Base.model_name(), "bge-base-zh-v1.5");
        assert_eq!(BGEVariant::Large.model_name(), "bge-large-zh-v1.5");
    }

    #[test]
    fn test_bge_config_default() {
        let config = BGEConfig::default();
        assert_eq!(config.model_variant.dimension(), 512);
        assert_eq!(config.batch_size, 32);
        assert_eq!(config.max_length, 512);
        assert!(config.normalize_embeddings);
        assert!(config.enable_quantization);
    }

    #[test]
    fn test_provider_stats_default() {
        let stats = ProviderStats::default();
        assert_eq!(stats.total_requests, 0);
        assert_eq!(stats.cache_hits, 0);
        assert_eq!(stats.average_latency_ms, 0.0);
    }
}
