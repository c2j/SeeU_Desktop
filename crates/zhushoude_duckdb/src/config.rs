//! 配置管理模块
//! 
//! 提供zhushoude_duckdb的各种配置选项

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// 主配置结构
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ZhushoudeConfig {
    /// 数据库文件路径
    pub database_path: String,
    /// 语义搜索配置
    pub embedding: EmbeddingConfig,
    /// 图计算配置
    pub graph: GraphConfig,
    /// 混合搜索配置
    pub hybrid: HybridConfig,
    /// 性能配置
    pub performance: PerformanceConfig,
}

/// 语义搜索配置
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EmbeddingConfig {
    /// 模型名称
    pub model_name: String,
    /// 批处理大小
    pub batch_size: usize,
    /// 最大缓存大小
    pub max_cache_size: usize,
    /// 向量维度
    pub vector_dimension: usize,
    /// 启用中文优化
    pub enable_chinese_optimization: bool,
    /// 启用向量归一化
    pub normalize_vectors: bool,
}

/// 图计算配置
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GraphConfig {
    /// 启用图计算
    pub enabled: bool,
    /// 最大图节点数
    pub max_nodes: usize,
    /// 最大图边数
    pub max_edges: usize,
    /// 启用图算法缓存
    pub enable_algorithm_cache: bool,
}

/// 混合搜索配置
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HybridConfig {
    /// 语义搜索权重
    pub semantic_weight: f32,
    /// 图搜索权重
    pub graph_weight: f32,
    /// 结果融合策略
    pub fusion_strategy: FusionStrategy,
}

/// 性能配置
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PerformanceConfig {
    /// 内存限制 (MB)
    pub memory_limit_mb: usize,
    /// 启用性能监控
    pub enable_monitoring: bool,
    /// 缓存策略
    pub cache_strategy: CacheStrategy,
    /// 线程池大小
    pub thread_pool_size: Option<usize>,
}

/// 结果融合策略
#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum FusionStrategy {
    /// 加权平均
    WeightedAverage,
    /// 倒数排名融合 (RRF)
    ReciprocalRankFusion,
    /// 最大值融合
    MaxFusion,
}

/// 缓存策略
#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum CacheStrategy {
    /// LRU缓存
    LRU,
    /// LFU缓存
    LFU,
    /// 时间过期缓存
    TimeExpiry,
}

impl Default for ZhushoudeConfig {
    fn default() -> Self {
        Self {
            database_path: "./zhushoude.db".to_string(),
            embedding: EmbeddingConfig::default(),
            graph: GraphConfig::default(),
            hybrid: HybridConfig::default(),
            performance: PerformanceConfig::default(),
        }
    }
}

impl Default for EmbeddingConfig {
    fn default() -> Self {
        Self {
            model_name: "bge-small-zh".to_string(),
            batch_size: 16,
            max_cache_size: 5000,
            vector_dimension: 512,
            enable_chinese_optimization: true,
            normalize_vectors: false, // 关闭归一化以保持向量的原始区分度
        }
    }
}

impl Default for GraphConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_nodes: 100000,
            max_edges: 500000,
            enable_algorithm_cache: true,
        }
    }
}

impl Default for HybridConfig {
    fn default() -> Self {
        Self {
            semantic_weight: 0.7,
            graph_weight: 0.3,
            fusion_strategy: FusionStrategy::WeightedAverage,
        }
    }
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            memory_limit_mb: 200,
            enable_monitoring: true,
            cache_strategy: CacheStrategy::LRU,
            thread_pool_size: None, // 自动检测
        }
    }
}

impl ZhushoudeConfig {
    /// 从文件加载配置
    pub fn from_file<P: AsRef<std::path::Path>>(path: P) -> crate::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: Self = toml::from_str(&content)
            .map_err(|e| crate::Error::TomlError(e.to_string()))?;
        Ok(config)
    }
    
    /// 保存配置到文件
    pub fn save_to_file<P: AsRef<std::path::Path>>(&self, path: P) -> crate::Result<()> {
        let content = toml::to_string_pretty(self)
            .map_err(|e| crate::Error::TomlSerError(e.to_string()))?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// 保存配置到字符串
    pub fn save_to_string(&self) -> crate::Result<String> {
        toml::to_string_pretty(self)
            .map_err(|e| crate::Error::TomlSerError(e.to_string()))
    }

    /// 从字符串加载配置
    pub fn load_from_string(content: &str) -> crate::Result<Self> {
        toml::from_str(content)
            .map_err(|e| crate::Error::TomlError(e.to_string()))
    }
    
    /// 验证配置有效性
    pub fn validate(&self) -> crate::Result<()> {
        if self.embedding.batch_size == 0 {
            return Err(crate::Error::ConfigError("batch_size不能为0".to_string()));
        }
        
        if self.embedding.max_cache_size == 0 {
            return Err(crate::Error::ConfigError("max_cache_size不能为0".to_string()));
        }
        
        if self.hybrid.semantic_weight + self.hybrid.graph_weight <= 0.0 {
            return Err(crate::Error::ConfigError("权重总和必须大于0".to_string()));
        }
        
        if self.performance.memory_limit_mb < 100 {
            return Err(crate::Error::ConfigError("内存限制不能小于100MB".to_string()));
        }
        
        Ok(())
    }
    
    /// 创建轻量级配置 (适用于资源受限环境)
    pub fn lightweight() -> Self {
        Self {
            embedding: EmbeddingConfig {
                batch_size: 8,
                max_cache_size: 1000,
                ..Default::default()
            },
            performance: PerformanceConfig {
                memory_limit_mb: 120,
                ..Default::default()
            },
            ..Default::default()
        }
    }
    
    /// 创建高性能配置 (适用于资源充足环境)
    pub fn high_performance() -> Self {
        Self {
            embedding: EmbeddingConfig {
                batch_size: 32,
                max_cache_size: 10000,
                ..Default::default()
            },
            performance: PerformanceConfig {
                memory_limit_mb: 500,
                ..Default::default()
            },
            ..Default::default()
        }
    }
}
