//! 语义搜索模块的错误类型定义

use thiserror::Error;

/// 语义搜索错误
#[derive(Debug, Error)]
pub enum SemanticSearchError {
    #[error("服务未初始化")]
    NotInitialized,

    #[error("服务不可用")]
    ServiceUnavailable,

    #[error("数据库错误: {0}")]
    DatabaseError(String),

    #[error("向量化错误: {0}")]
    EmbeddingError(#[from] EmbeddingError),

    #[error("查询解析错误: {0}")]
    QueryParseError(String),

    #[error("索引错误: {0}")]
    IndexError(String),

    #[error("配置错误: {0}")]
    ConfigError(String),

    #[error("IO错误: {0}")]
    IoError(String),

    #[error("网络错误: {0}")]
    NetworkError(String),

    #[error("HelixDB错误: {0}")]
    HelixDBError(#[from] HelixDBError),

    #[error("搜索错误: {0}")]
    SearchError(String),

    #[error("结果融合错误: {0}")]
    MergeError(String),
}

/// 向量化错误
#[derive(Debug, Error)]
pub enum EmbeddingError {
    #[error("API错误: {0}")]
    ApiError(String),

    #[error("网络错误: {0}")]
    NetworkError(String),

    #[error("配置错误: {0}")]
    ConfigError(String),

    #[error("向量维度不匹配")]
    DimensionMismatch,

    #[error("缓存错误: {0}")]
    CacheError(String),

    #[error("序列化错误: {0}")]
    SerializationError(String),

    #[error("认证错误: {0}")]
    AuthenticationError(String),

    #[error("配额超限: {0}")]
    QuotaExceeded(String),
}

/// HelixDB错误类型
#[derive(Debug, Error)]
pub enum HelixDBError {
    #[error("CLI未找到: {0}")]
    CliNotFound(String),

    #[error("初始化错误: {0}")]
    InitError(String),

    #[error("进程错误: {0}")]
    ProcessError(String),

    #[error("启动超时: {0}")]
    StartupTimeout(String),

    #[error("IO错误: {0}")]
    IoError(String),

    #[error("网络错误: {0}")]
    NetworkError(String),

    #[error("健康检查失败: {0}")]
    HealthCheckFailed(String),

    #[error("服务不可用")]
    ServiceUnavailable,

    #[error("配置错误: {0}")]
    ConfigError(String),

    #[error("查询错误: {0}")]
    QueryError(String),

    #[error("连接错误: {0}")]
    ConnectionError(String),
}

/// 搜索错误
#[derive(Debug, Error)]
pub enum SearchError {
    #[error("语义搜索错误: {0}")]
    SemanticError(#[from] SemanticSearchError),

    #[error("关键词搜索错误: {0}")]
    KeywordError(String),

    #[error("结果融合错误: {0}")]
    MergeError(String),

    #[error("配置错误: {0}")]
    ConfigError(String),

    #[error("超时错误: {0}")]
    TimeoutError(String),

    #[error("无效查询: {0}")]
    InvalidQuery(String),
}

// 实现从其他错误类型的转换
impl From<std::io::Error> for SemanticSearchError {
    fn from(err: std::io::Error) -> Self {
        SemanticSearchError::IoError(err.to_string())
    }
}

impl From<reqwest::Error> for EmbeddingError {
    fn from(err: reqwest::Error) -> Self {
        if err.is_timeout() {
            EmbeddingError::NetworkError(format!("请求超时: {}", err))
        } else if err.is_connect() {
            EmbeddingError::NetworkError(format!("连接失败: {}", err))
        } else {
            EmbeddingError::ApiError(err.to_string())
        }
    }
}

impl From<serde_json::Error> for EmbeddingError {
    fn from(err: serde_json::Error) -> Self {
        EmbeddingError::SerializationError(err.to_string())
    }
}

impl From<toml::de::Error> for SemanticSearchError {
    fn from(err: toml::de::Error) -> Self {
        SemanticSearchError::ConfigError(format!("配置解析错误: {}", err))
    }
}

impl From<toml::ser::Error> for SemanticSearchError {
    fn from(err: toml::ser::Error) -> Self {
        SemanticSearchError::ConfigError(format!("配置序列化错误: {}", err))
    }
}

// 为错误类型实现一些便利方法
impl SemanticSearchError {
    /// 检查是否为临时错误（可重试）
    pub fn is_temporary(&self) -> bool {
        matches!(
            self,
            SemanticSearchError::NetworkError(_)
                | SemanticSearchError::ServiceUnavailable
                | SemanticSearchError::HelixDBError(HelixDBError::NetworkError(_))
                | SemanticSearchError::HelixDBError(HelixDBError::ServiceUnavailable)
        )
    }

    /// 检查是否为配置错误
    pub fn is_config_error(&self) -> bool {
        matches!(
            self,
            SemanticSearchError::ConfigError(_)
                | SemanticSearchError::EmbeddingError(EmbeddingError::ConfigError(_))
                | SemanticSearchError::HelixDBError(HelixDBError::ConfigError(_))
        )
    }

    /// 获取用户友好的错误消息
    pub fn user_message(&self) -> String {
        match self {
            SemanticSearchError::NotInitialized => "语义搜索功能尚未初始化".to_string(),
            SemanticSearchError::ServiceUnavailable => "语义搜索服务暂时不可用".to_string(),
            SemanticSearchError::NetworkError(_) => "网络连接错误，请检查网络设置".to_string(),
            SemanticSearchError::ConfigError(_) => "配置错误，请检查设置".to_string(),
            SemanticSearchError::EmbeddingError(EmbeddingError::AuthenticationError(_)) => {
                "API认证失败，请检查API密钥".to_string()
            }
            SemanticSearchError::EmbeddingError(EmbeddingError::QuotaExceeded(_)) => {
                "API配额已用完，请稍后再试".to_string()
            }
            _ => "语义搜索出现错误，请稍后再试".to_string(),
        }
    }
}

impl EmbeddingError {
    /// 检查是否为认证相关错误
    pub fn is_auth_error(&self) -> bool {
        matches!(self, EmbeddingError::AuthenticationError(_))
    }

    /// 检查是否为配额相关错误
    pub fn is_quota_error(&self) -> bool {
        matches!(self, EmbeddingError::QuotaExceeded(_))
    }

    /// 检查是否为网络相关错误
    pub fn is_network_error(&self) -> bool {
        matches!(self, EmbeddingError::NetworkError(_))
    }
}

impl HelixDBError {
    /// 检查是否为启动相关错误
    pub fn is_startup_error(&self) -> bool {
        matches!(
            self,
            HelixDBError::CliNotFound(_)
                | HelixDBError::InitError(_)
                | HelixDBError::StartupTimeout(_)
        )
    }

    /// 检查是否为运行时错误
    pub fn is_runtime_error(&self) -> bool {
        matches!(
            self,
            HelixDBError::ProcessError(_)
                | HelixDBError::HealthCheckFailed(_)
                | HelixDBError::ServiceUnavailable
        )
    }
}
