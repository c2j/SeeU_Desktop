//! BGE模型配置管理
//! 
//! 提供BGE模型的各种配置选项和预设

use crate::embedding::provider::{BGEConfig, BGEVariant, Device};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// BGE配置预设
impl BGEConfig {
    /// 轻量级配置 - 适用于资源受限环境
    pub fn lightweight() -> Self {
        Self {
            model_variant: BGEVariant::Small,
            device: Device::CPU,
            batch_size: 16,
            max_length: 256,
            normalize_embeddings: true,
            cache_size: 5000,
            enable_quantization: true,
            cache_dir: PathBuf::from("./models/bge"),
        }
    }

    /// 高性能配置 - 适用于资源充足环境
    pub fn high_performance() -> Self {
        Self {
            model_variant: BGEVariant::Base,
            device: Device::CPU, // 可以根据硬件情况调整为CUDA或Metal
            batch_size: 64,
            max_length: 512,
            normalize_embeddings: true,
            cache_size: 50000,
            enable_quantization: false,
            cache_dir: PathBuf::from("./models/bge"),
        }
    }

    /// GPU配置 - 使用CUDA加速
    pub fn gpu_accelerated(gpu_id: usize) -> Self {
        Self {
            model_variant: BGEVariant::Base,
            device: Device::CUDA(gpu_id),
            batch_size: 128,
            max_length: 512,
            normalize_embeddings: true,
            cache_size: 20000,
            enable_quantization: false,
            cache_dir: PathBuf::from("./models/bge"),
        }
    }

    /// Metal配置 - 使用Apple Silicon加速
    pub fn metal_accelerated() -> Self {
        Self {
            model_variant: BGEVariant::Base,
            device: Device::Metal,
            batch_size: 64,
            max_length: 512,
            normalize_embeddings: true,
            cache_size: 20000,
            enable_quantization: false,
            cache_dir: PathBuf::from("./models/bge"),
        }
    }

    /// 验证配置有效性
    pub fn validate(&self) -> crate::Result<()> {
        if self.batch_size == 0 {
            return Err(crate::Error::ConfigError("batch_size不能为0".to_string()));
        }

        if self.max_length == 0 {
            return Err(crate::Error::ConfigError("max_length不能为0".to_string()));
        }

        if self.cache_size == 0 {
            return Err(crate::Error::ConfigError("cache_size不能为0".to_string()));
        }

        // 检查缓存目录是否可创建
        if let Some(parent) = self.cache_dir.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| crate::Error::IoError(e.to_string()))?;
            }
        }

        Ok(())
    }

    /// 估算内存使用量
    pub fn estimate_memory_usage(&self) -> MemoryEstimate {
        let model_memory = self.model_variant.estimated_memory_mb();
        let cache_memory = (self.cache_size * self.model_variant.dimension() * 4) / (1024 * 1024); // 4 bytes per f32
        let batch_memory = (self.batch_size * self.max_length * 4) / (1024 * 1024);

        MemoryEstimate {
            model_mb: model_memory,
            cache_mb: cache_memory,
            batch_mb: batch_memory,
            total_mb: model_memory + cache_memory + batch_memory,
        }
    }

    /// 根据可用内存调整配置
    pub fn adjust_for_memory_limit(&mut self, memory_limit_mb: usize) {
        let estimate = self.estimate_memory_usage();
        
        if estimate.total_mb > memory_limit_mb {
            // 降级模型变体
            if matches!(self.model_variant, BGEVariant::Large) {
                self.model_variant = BGEVariant::Base;
            } else if matches!(self.model_variant, BGEVariant::Base) {
                self.model_variant = BGEVariant::Small;
            }

            // 减少缓存大小
            let available_for_cache = memory_limit_mb.saturating_sub(self.model_variant.estimated_memory_mb() + 50);
            let max_cache_entries = (available_for_cache * 1024 * 1024) / (self.model_variant.dimension() * 4);
            self.cache_size = self.cache_size.min(max_cache_entries);

            // 减少批处理大小
            self.batch_size = self.batch_size.min(32);

            // 启用量化以节省内存
            self.enable_quantization = true;
        }
    }
}

/// 内存使用估算
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEstimate {
    pub model_mb: usize,
    pub cache_mb: usize,
    pub batch_mb: usize,
    pub total_mb: usize,
}

/// BGE运行时配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BGERuntimeConfig {
    /// 推理超时时间(秒)
    pub inference_timeout_secs: u64,
    /// 模型预热
    pub enable_warmup: bool,
    /// 预热文本
    pub warmup_texts: Vec<String>,
    /// 错误重试次数
    pub max_retries: usize,
    /// 性能监控
    pub enable_metrics: bool,
}

impl Default for BGERuntimeConfig {
    fn default() -> Self {
        Self {
            inference_timeout_secs: 30,
            enable_warmup: true,
            warmup_texts: vec![
                "测试文本".to_string(),
                "这是一个中文语义测试".to_string(),
            ],
            max_retries: 3,
            enable_metrics: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_presets() {
        let lightweight = BGEConfig::lightweight();
        assert_eq!(lightweight.model_variant.dimension(), 512);
        assert_eq!(lightweight.batch_size, 16);
        assert!(lightweight.enable_quantization);

        let high_perf = BGEConfig::high_performance();
        assert_eq!(high_perf.model_variant.dimension(), 768);
        assert_eq!(high_perf.batch_size, 64);
        assert!(!high_perf.enable_quantization);
    }

    #[test]
    fn test_memory_estimation() {
        let config = BGEConfig::default();
        let estimate = config.estimate_memory_usage();
        
        assert!(estimate.model_mb > 0);
        assert!(estimate.cache_mb > 0);
        assert!(estimate.total_mb > estimate.model_mb);
    }

    #[test]
    fn test_memory_adjustment() {
        let mut config = BGEConfig::high_performance();
        let original_variant = config.model_variant.clone();
        
        // 设置很低的内存限制
        config.adjust_for_memory_limit(100);
        
        // 应该降级到更小的模型
        assert!(config.model_variant.estimated_memory_mb() <= original_variant.estimated_memory_mb());
        assert!(config.enable_quantization);
    }

    #[test]
    fn test_config_validation() {
        let mut config = BGEConfig::default();
        assert!(config.validate().is_ok());

        config.batch_size = 0;
        assert!(config.validate().is_err());

        config.batch_size = 32;
        config.max_length = 0;
        assert!(config.validate().is_err());
    }
}
