//! 错误处理模块
//! 
//! 提供统一的错误类型和错误处理机制

use thiserror::Error;

/// 统一的结果类型
pub type Result<T> = std::result::Result<T, Error>;

/// 主要错误类型
#[derive(Error, Debug, Clone)]
pub enum Error {
    /// 数据库连接错误
    #[error("数据库连接错误: {0}")]
    DatabaseError(String),

    /// SQL执行错误
    #[error("SQL执行错误: {0}")]
    SqlError(String),

    /// 向量化错误
    #[error("向量化错误: {0}")]
    VectorizationError(String),

    /// 模型加载错误
    #[error("模型加载错误: {0}")]
    ModelError(String),

    /// 推理错误
    #[error("推理错误: {0}")]
    InferenceError(String),

    /// 搜索错误
    #[error("搜索错误: {0}")]
    SearchError(String),

    /// 图数据错误
    #[error("图数据错误: {0}")]
    GraphError(String),

    /// 实体提取错误
    #[error("实体提取错误: {0}")]
    EntityExtractionError(String),

    /// 关系提取错误
    #[error("关系提取错误: {0}")]
    RelationExtractionError(String),

    /// IO错误
    #[error("IO错误: {0}")]
    IoError(String),

    /// 序列化错误
    #[error("序列化错误: {0}")]
    SerializationError(String),

    /// 配置错误
    #[error("配置错误: {0}")]
    ConfigError(String),

    /// TOML解析错误
    #[error("TOML解析错误: {0}")]
    TomlError(String),

    /// TOML序列化错误
    #[error("TOML序列化错误: {0}")]
    TomlSerError(String),

    /// 正则表达式错误
    #[error("正则表达式错误: {0}")]
    RegexError(String),

    /// 网络错误
    #[error("网络错误: {0}")]
    NetworkError(String),
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::IoError(err.to_string())
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::SerializationError(err.to_string())
    }
}

impl From<duckdb::Error> for Error {
    fn from(err: duckdb::Error) -> Self {
        Error::DatabaseError(err.to_string())
    }
}

impl From<regex::Error> for Error {
    fn from(err: regex::Error) -> Self {
        Error::RegexError(err.to_string())
    }
}

/// 性能监控相关的错误辅助函数
impl Error {
    /// 检查是否为内存相关错误
    pub fn is_memory_error(&self) -> bool {
        match self {
            Error::ModelError(msg) | Error::InferenceError(msg) => {
                msg.contains("内存") || msg.contains("memory") || msg.contains("OOM")
            }
            _ => false,
        }
    }

    /// 检查是否为性能相关错误
    pub fn is_performance_error(&self) -> bool {
        match self {
            Error::SearchError(msg) | Error::InferenceError(msg) => {
                msg.contains("超时") || msg.contains("timeout") || msg.contains("slow")
            }
            _ => false,
        }
    }

    /// 获取错误的严重程度
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            Error::DatabaseError(_) | Error::SqlError(_) => ErrorSeverity::Critical,
            Error::ModelError(_) | Error::InferenceError(_) => ErrorSeverity::High,
            Error::SearchError(_) | Error::VectorizationError(_) => ErrorSeverity::Medium,
            Error::EntityExtractionError(_) | Error::RelationExtractionError(_) => ErrorSeverity::Medium,
            Error::IoError(_) | Error::SerializationError(_) => ErrorSeverity::Low,
            Error::ConfigError(_) | Error::GraphError(_) => ErrorSeverity::Medium,
            Error::TomlError(_) | Error::TomlSerError(_) => ErrorSeverity::Low,
            Error::RegexError(_) => ErrorSeverity::Medium,
            Error::NetworkError(_) => ErrorSeverity::Medium,
        }
    }

    /// 是否可以重试
    pub fn is_retryable(&self) -> bool {
        match self {
            Error::DatabaseError(_) | Error::SearchError(_) | Error::InferenceError(_) => true,
            Error::ModelError(_) | Error::ConfigError(_) => false,
            _ => true,
        }
    }
}

/// 错误严重程度
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ErrorSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// 错误统计信息
#[derive(Debug, Clone, Default)]
pub struct ErrorStats {
    pub total_errors: u64,
    pub memory_errors: u64,
    pub performance_errors: u64,
    pub retryable_errors: u64,
    pub critical_errors: u64,
}

impl ErrorStats {
    /// 记录一个错误
    pub fn record_error(&mut self, error: &Error) {
        self.total_errors += 1;
        
        if error.is_memory_error() {
            self.memory_errors += 1;
        }
        
        if error.is_performance_error() {
            self.performance_errors += 1;
        }
        
        if error.is_retryable() {
            self.retryable_errors += 1;
        }
        
        if error.severity() == ErrorSeverity::Critical {
            self.critical_errors += 1;
        }
    }

    /// 获取错误率
    pub fn error_rate(&self) -> f64 {
        if self.total_errors == 0 {
            0.0
        } else {
            self.critical_errors as f64 / self.total_errors as f64
        }
    }

    /// 重置统计
    pub fn reset(&mut self) {
        *self = Default::default();
    }
}

/// 错误恢复策略
#[derive(Debug, Clone)]
pub enum RecoveryStrategy {
    /// 立即重试
    Retry { max_attempts: u32, delay_ms: u64 },
    /// 降级处理
    Fallback,
    /// 跳过处理
    Skip,
    /// 终止处理
    Abort,
}

impl Error {
    /// 获取推荐的恢复策略
    pub fn recovery_strategy(&self) -> RecoveryStrategy {
        match self {
            Error::DatabaseError(_) => RecoveryStrategy::Retry { max_attempts: 3, delay_ms: 1000 },
            Error::SearchError(_) => RecoveryStrategy::Retry { max_attempts: 2, delay_ms: 500 },
            Error::InferenceError(_) => RecoveryStrategy::Fallback,
            Error::ModelError(_) => RecoveryStrategy::Abort,
            Error::VectorizationError(_) => RecoveryStrategy::Fallback,
            Error::EntityExtractionError(_) | Error::RelationExtractionError(_) => RecoveryStrategy::Skip,
            Error::IoError(_) => RecoveryStrategy::Retry { max_attempts: 2, delay_ms: 100 },
            Error::SerializationError(_) => RecoveryStrategy::Skip,
            Error::ConfigError(_) => RecoveryStrategy::Abort,
            Error::SqlError(_) => RecoveryStrategy::Retry { max_attempts: 1, delay_ms: 0 },
            Error::GraphError(_) => RecoveryStrategy::Fallback,
            Error::TomlError(_) | Error::TomlSerError(_) => RecoveryStrategy::Abort,
            Error::RegexError(_) => RecoveryStrategy::Abort,
            Error::NetworkError(_) => RecoveryStrategy::Retry { max_attempts: 3, delay_ms: 2000 },
        }
    }
}

/// 错误上下文信息
#[derive(Debug, Clone)]
pub struct ErrorContext {
    pub operation: String,
    pub component: String,
    pub timestamp: std::time::SystemTime,
    pub additional_info: std::collections::HashMap<String, String>,
}

impl ErrorContext {
    pub fn new(operation: &str, component: &str) -> Self {
        Self {
            operation: operation.to_string(),
            component: component.to_string(),
            timestamp: std::time::SystemTime::now(),
            additional_info: std::collections::HashMap::new(),
        }
    }

    pub fn with_info(mut self, key: &str, value: &str) -> Self {
        self.additional_info.insert(key.to_string(), value.to_string());
        self
    }
}

/// 带上下文的错误
#[derive(Debug, Clone)]
pub struct ContextualError {
    pub error: Error,
    pub context: ErrorContext,
}

impl std::fmt::Display for ContextualError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}:{}] {}", self.context.component, self.context.operation, self.error)
    }
}

impl std::error::Error for ContextualError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.error)
    }
}
