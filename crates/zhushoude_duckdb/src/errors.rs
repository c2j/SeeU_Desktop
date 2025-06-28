//! 错误处理模块
//! 
//! 定义zhushoude_duckdb的所有错误类型

use thiserror::Error;

/// zhushoude_duckdb的结果类型
pub type Result<T> = std::result::Result<T, Error>;

/// zhushoude_duckdb的错误类型
#[derive(Error, Debug)]
pub enum Error {
    /// 数据库错误
    #[error("数据库错误: {0}")]
    DatabaseError(String), // 暂时使用String，避免duckdb依赖
    
    /// 配置错误
    #[error("配置错误: {0}")]
    ConfigError(String),
    
    /// 模型加载错误
    #[error("模型加载错误: {0}")]
    ModelError(String),
    
    /// 推理错误
    #[error("推理错误: {0}")]
    InferenceError(String),
    
    /// 向量化错误
    #[error("向量化错误: {0}")]
    EmbeddingError(String),
    
    /// 搜索错误
    #[error("搜索错误: {0}")]
    SearchError(String),
    
    /// 图计算错误
    #[error("图计算错误: {0}")]
    GraphError(String),
    
    /// 缓存错误
    #[error("缓存错误: {0}")]
    CacheError(String),
    
    /// IO错误
    #[error("IO错误: {0}")]
    IoError(#[from] std::io::Error),
    
    /// 序列化错误
    #[error("序列化错误: {0}")]
    SerializationError(#[from] serde_json::Error),
    
    /// TOML解析错误
    #[error("TOML解析错误: {0}")]
    TomlError(String), // 暂时使用String

    /// TOML序列化错误
    #[error("TOML序列化错误: {0}")]
    TomlSerError(String), // 暂时使用String
    
    /// 分词器错误
    #[error("分词器错误: {0}")]
    TokenizerError(String),
    
    /// Candle框架错误
    #[error("Candle错误: {0}")]
    CandleError(String),
    
    /// HuggingFace Hub错误
    #[error("HuggingFace Hub错误: {0}")]
    HfHubError(String),
    
    /// 内存不足错误
    #[error("内存不足: 当前使用 {current_mb}MB, 限制 {limit_mb}MB")]
    OutOfMemoryError {
        current_mb: usize,
        limit_mb: usize,
    },
    
    /// 超时错误
    #[error("操作超时: {operation} 超过 {timeout_ms}ms")]
    TimeoutError {
        operation: String,
        timeout_ms: u64,
    },
    
    /// 不支持的操作
    #[error("不支持的操作: {0}")]
    UnsupportedOperation(String),
    
    /// 数据验证错误
    #[error("数据验证错误: {0}")]
    ValidationError(String),
    
    /// 并发错误
    #[error("并发错误: {0}")]
    ConcurrencyError(String),
}

// 实现从各种外部错误类型的转换
// 暂时注释掉这些From实现，避免依赖问题
// impl From<tokenizers::Error> for Error {
//     fn from(err: tokenizers::Error) -> Self {
//         Error::TokenizerError(err.to_string())
//     }
// }

// impl From<candle_core::Error> for Error {
//     fn from(err: candle_core::Error) -> Self {
//         Error::CandleError(err.to_string())
//     }
// }

// impl From<hf_hub::api::tokio::ApiError> for Error {
//     fn from(err: hf_hub::api::tokio::ApiError) -> Self {
//         Error::HfHubError(err.to_string())
//     }
// }

impl From<Box<dyn std::error::Error + Send + Sync>> for Error {
    fn from(err: Box<dyn std::error::Error + Send + Sync>) -> Self {
        Error::ModelError(err.to_string())
    }
}

/// 错误上下文扩展trait
pub trait ErrorContext<T> {
    /// 添加上下文信息
    fn with_context<F>(self, f: F) -> Result<T>
    where
        F: FnOnce() -> String;
    
    /// 添加操作上下文
    fn with_operation(self, operation: &str) -> Result<T>;
}

impl<T, E> ErrorContext<T> for std::result::Result<T, E>
where
    E: Into<Error>,
{
    fn with_context<F>(self, f: F) -> Result<T>
    where
        F: FnOnce() -> String,
    {
        self.map_err(|e| {
            let base_error = e.into();
            let context = f();
            match base_error {
                Error::ModelError(msg) => Error::ModelError(format!("{}: {}", context, msg)),
                Error::InferenceError(msg) => Error::InferenceError(format!("{}: {}", context, msg)),
                Error::SearchError(msg) => Error::SearchError(format!("{}: {}", context, msg)),
                Error::GraphError(msg) => Error::GraphError(format!("{}: {}", context, msg)),
                other => other,
            }
        })
    }
    
    fn with_operation(self, operation: &str) -> Result<T> {
        self.with_context(|| format!("操作 '{}' 失败", operation))
    }
}

/// 性能监控相关的错误辅助函数
impl Error {
    /// 检查是否为内存相关错误
    pub fn is_memory_error(&self) -> bool {
        matches!(self, Error::OutOfMemoryError { .. })
    }
    
    /// 检查是否为超时错误
    pub fn is_timeout_error(&self) -> bool {
        matches!(self, Error::TimeoutError { .. })
    }
    
    /// 检查是否为可重试的错误
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            Error::TimeoutError { .. } | 
            Error::ConcurrencyError(_) |
            Error::IoError(_)
        )
    }
    
    /// 获取错误的严重程度
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            Error::OutOfMemoryError { .. } => ErrorSeverity::Critical,
            Error::DatabaseError(_) => ErrorSeverity::High,
            Error::ModelError(_) => ErrorSeverity::High,
            Error::ConfigError(_) => ErrorSeverity::Medium,
            Error::TimeoutError { .. } => ErrorSeverity::Medium,
            Error::ValidationError(_) => ErrorSeverity::Low,
            _ => ErrorSeverity::Medium,
        }
    }
}

/// 错误严重程度
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorSeverity {
    /// 低严重程度
    Low,
    /// 中等严重程度
    Medium,
    /// 高严重程度
    High,
    /// 严重错误
    Critical,
}
