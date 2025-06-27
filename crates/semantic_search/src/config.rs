//! 语义搜索配置管理

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// 语义搜索配置
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SemanticSearchConfig {
    /// 是否启用语义搜索
    pub enabled: bool,
    /// HelixDB连接配置
    pub helix_config: HelixDBConfig,
    /// 向量化配置
    pub embedding_config: EmbeddingConfig,
    /// 搜索权重配置
    pub search_weights: SearchWeights,
    /// 性能配置
    pub performance_config: PerformanceConfig,
}

/// HelixDB连接配置
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HelixDBConfig {
    /// 数据库路径
    pub database_path: String,
    /// 连接端口
    pub port: u16,
    /// 连接超时(秒)
    pub connection_timeout: u64,
    /// 查询超时(秒)
    pub query_timeout: u64,
    /// 启动超时(秒)
    pub startup_timeout: u64,
    /// 健康检查间隔(秒)
    pub health_check_interval: u64,
    /// 最大重启次数
    pub max_restart_attempts: u32,
    /// 是否启用自动重启
    pub auto_restart: bool,
    /// 日志级别
    pub log_level: String,
}

/// 向量化服务配置
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EmbeddingConfig {
    /// 嵌入模型提供商 (openai, cohere, local)
    pub provider: String,
    /// 模型名称
    pub model: String,
    /// API密钥
    pub api_key: Option<String>,
    /// API基础URL
    pub api_base_url: Option<String>,
    /// 向量维度
    pub dimension: usize,
    /// 批处理大小
    pub batch_size: usize,
    /// 请求超时(秒)
    pub request_timeout: u64,
    /// 最大重试次数
    pub max_retries: u32,
    /// 缓存配置
    pub cache_config: CacheConfig,
}

/// 搜索权重配置
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SearchWeights {
    /// 关键词搜索权重
    pub keyword_weight: f32,
    /// 语义搜索权重
    pub semantic_weight: f32,
    /// 图关系权重
    pub graph_weight: f32,
}

/// 性能配置
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PerformanceConfig {
    /// 索引批处理大小
    pub index_batch_size: usize,
    /// 搜索结果缓存大小
    pub search_cache_size: usize,
    /// 搜索结果缓存TTL(秒)
    pub search_cache_ttl: u64,
    /// 并发索引任务数
    pub concurrent_index_tasks: usize,
    /// 最大内存使用(MB)
    pub max_memory_mb: usize,
}

/// 缓存配置
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CacheConfig {
    /// 是否启用缓存
    pub enabled: bool,
    /// 缓存大小
    pub max_size: usize,
    /// 缓存TTL(秒)
    pub ttl_seconds: u64,
}

impl Default for SemanticSearchConfig {
    fn default() -> Self {
        Self {
            enabled: false, // 默认禁用，需要用户手动启用
            helix_config: HelixDBConfig::default(),
            embedding_config: EmbeddingConfig::default(),
            search_weights: SearchWeights::default(),
            performance_config: PerformanceConfig::default(),
        }
    }
}

impl Default for HelixDBConfig {
    fn default() -> Self {
        Self {
            database_path: "".to_string(), // 将在初始化时设置
            port: 6969,
            connection_timeout: 30,
            query_timeout: 30,
            startup_timeout: 60,
            health_check_interval: 10,
            max_restart_attempts: 3,
            auto_restart: true,
            log_level: "info".to_string(),
        }
    }
}

impl Default for EmbeddingConfig {
    fn default() -> Self {
        Self {
            provider: "openai".to_string(),
            model: "text-embedding-3-small".to_string(),
            api_key: None,
            api_base_url: None,
            dimension: 1536,
            batch_size: 100,
            request_timeout: 30,
            max_retries: 3,
            cache_config: CacheConfig::default(),
        }
    }
}

impl Default for SearchWeights {
    fn default() -> Self {
        Self {
            keyword_weight: 0.3,
            semantic_weight: 0.5,
            graph_weight: 0.2,
        }
    }
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            index_batch_size: 50,
            search_cache_size: 1000,
            search_cache_ttl: 300, // 5分钟
            concurrent_index_tasks: 4,
            max_memory_mb: 512,
        }
    }
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_size: 10000,
            ttl_seconds: 3600, // 1小时
        }
    }
}

impl SemanticSearchConfig {
    /// 从文件加载配置
    pub fn load_from_file(path: &PathBuf) -> Result<Self, crate::SemanticSearchError> {
        if !path.exists() {
            return Ok(Self::default());
        }

        let content = std::fs::read_to_string(path)
            .map_err(|e| crate::SemanticSearchError::IoError(format!("读取配置文件失败: {}", e)))?;

        let config: Self = toml::from_str(&content)?;
        Ok(config)
    }

    /// 保存配置到文件
    pub fn save_to_file(&self, path: &PathBuf) -> Result<(), crate::SemanticSearchError> {
        // 确保配置目录存在
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| crate::SemanticSearchError::IoError(format!("创建配置目录失败: {}", e)))?;
        }

        let content = toml::to_string_pretty(self)?;
        std::fs::write(path, content)
            .map_err(|e| crate::SemanticSearchError::IoError(format!("写入配置文件失败: {}", e)))?;

        Ok(())
    }

    /// 获取默认配置文件路径
    pub fn default_config_path() -> Result<PathBuf, crate::SemanticSearchError> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| crate::SemanticSearchError::ConfigError("无法获取配置目录".to_string()))?
            .join("SeeU_Desktop");

        Ok(config_dir.join("semantic_search.toml"))
    }

    /// 验证配置
    pub fn validate(&self) -> Result<(), crate::SemanticSearchError> {
        // 验证权重总和
        let total_weight = self.search_weights.keyword_weight 
            + self.search_weights.semantic_weight 
            + self.search_weights.graph_weight;
        
        if (total_weight - 1.0).abs() > 0.01 {
            return Err(crate::SemanticSearchError::ConfigError(
                format!("搜索权重总和应为1.0，当前为: {:.2}", total_weight)
            ));
        }

        // 验证端口范围
        if self.helix_config.port < 1024 || self.helix_config.port > 65535 {
            return Err(crate::SemanticSearchError::ConfigError(
                format!("端口号无效: {}", self.helix_config.port)
            ));
        }

        // 验证向量维度
        if self.embedding_config.dimension == 0 {
            return Err(crate::SemanticSearchError::ConfigError(
                "向量维度不能为0".to_string()
            ));
        }

        // 验证批处理大小
        if self.embedding_config.batch_size == 0 {
            return Err(crate::SemanticSearchError::ConfigError(
                "批处理大小不能为0".to_string()
            ));
        }

        Ok(())
    }

    /// 获取数据目录路径
    pub fn get_data_directory(&self) -> Result<PathBuf, crate::SemanticSearchError> {
        if !self.helix_config.database_path.is_empty() {
            return Ok(PathBuf::from(&self.helix_config.database_path));
        }

        let data_dir = dirs::data_dir()
            .ok_or_else(|| crate::SemanticSearchError::ConfigError("无法获取数据目录".to_string()))?
            .join("SeeU_Desktop")
            .join("semantic_search");

        Ok(data_dir)
    }

    /// 检查是否需要API密钥
    pub fn requires_api_key(&self) -> bool {
        matches!(self.embedding_config.provider.as_str(), "openai" | "cohere")
    }

    /// 检查API密钥是否已配置
    pub fn has_api_key(&self) -> bool {
        self.embedding_config.api_key.is_some() && 
        !self.embedding_config.api_key.as_ref().unwrap().is_empty()
    }

    /// 获取向量化服务的显示名称
    pub fn get_embedding_provider_name(&self) -> &str {
        match self.embedding_config.provider.as_str() {
            "openai" => "OpenAI",
            "cohere" => "Cohere",
            "local" => "本地模型",
            _ => "未知",
        }
    }
}

impl EmbeddingConfig {
    /// 获取API基础URL
    pub fn get_api_base_url(&self) -> String {
        self.api_base_url.clone().unwrap_or_else(|| {
            match self.provider.as_str() {
                "openai" => "https://api.openai.com/v1".to_string(),
                "cohere" => "https://api.cohere.ai/v1".to_string(),
                "local" => "http://localhost:11434".to_string(),
                _ => "".to_string(),
            }
        })
    }

    /// 获取嵌入API端点
    pub fn get_embedding_endpoint(&self) -> String {
        match self.provider.as_str() {
            "openai" => format!("{}/embeddings", self.get_api_base_url()),
            "cohere" => format!("{}/embed", self.get_api_base_url()),
            "local" => format!("{}/api/embeddings", self.get_api_base_url()),
            _ => "".to_string(),
        }
    }
}
