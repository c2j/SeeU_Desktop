//! 错误处理模块的单元测试

use zhushoude_duckdb::errors::*;
use std::io;

#[test]
fn test_error_types() {
    // 测试各种错误类型的创建
    let config_error = Error::ConfigError("测试配置错误".to_string());
    assert!(matches!(config_error, Error::ConfigError(_)));
    
    let model_error = Error::ModelError("测试模型错误".to_string());
    assert!(matches!(model_error, Error::ModelError(_)));
    
    let inference_error = Error::InferenceError("测试推理错误".to_string());
    assert!(matches!(inference_error, Error::InferenceError(_)));
    
    let embedding_error = Error::EmbeddingError("测试向量化错误".to_string());
    assert!(matches!(embedding_error, Error::EmbeddingError(_)));
    
    let search_error = Error::SearchError("测试搜索错误".to_string());
    assert!(matches!(search_error, Error::SearchError(_)));
    
    let graph_error = Error::GraphError("测试图计算错误".to_string());
    assert!(matches!(graph_error, Error::GraphError(_)));
    
    let cache_error = Error::CacheError("测试缓存错误".to_string());
    assert!(matches!(cache_error, Error::CacheError(_)));
}

#[test]
fn test_memory_error() {
    let memory_error = Error::OutOfMemoryError {
        current_mb: 300,
        limit_mb: 200,
    };
    
    assert!(memory_error.is_memory_error());
    assert!(!memory_error.is_timeout_error());
    assert!(!memory_error.is_retryable());
    assert_eq!(memory_error.severity(), ErrorSeverity::Critical);
    
    let error_msg = format!("{}", memory_error);
    assert!(error_msg.contains("300"));
    assert!(error_msg.contains("200"));
}

#[test]
fn test_timeout_error() {
    let timeout_error = Error::TimeoutError {
        operation: "语义搜索".to_string(),
        timeout_ms: 5000,
    };
    
    assert!(!timeout_error.is_memory_error());
    assert!(timeout_error.is_timeout_error());
    assert!(timeout_error.is_retryable());
    assert_eq!(timeout_error.severity(), ErrorSeverity::Medium);
    
    let error_msg = format!("{}", timeout_error);
    assert!(error_msg.contains("语义搜索"));
    assert!(error_msg.contains("5000"));
}

#[test]
fn test_error_from_conversions() {
    // 测试从IO错误的转换
    let io_error = io::Error::new(io::ErrorKind::NotFound, "文件未找到");
    let converted_error: Error = io_error.into();
    assert!(matches!(converted_error, Error::IoError(_)));
    
    // 测试从JSON错误的转换
    let json_error = serde_json::from_str::<serde_json::Value>("invalid json");
    assert!(json_error.is_err());
    let converted_error: Error = json_error.unwrap_err().into();
    assert!(matches!(converted_error, Error::SerializationError(_)));
}

#[test]
fn test_error_context() {
    // 测试错误上下文扩展
    let result: Result<()> = Err(Error::ModelError("原始错误".to_string()));
    
    let with_context = result.with_context(|| "添加上下文信息".to_string());
    assert!(with_context.is_err());
    
    let error_msg = format!("{}", with_context.unwrap_err());
    assert!(error_msg.contains("添加上下文信息"));
    assert!(error_msg.contains("原始错误"));
    
    // 测试操作上下文
    let result: Result<()> = Err(Error::InferenceError("推理失败".to_string()));
    let with_operation = result.with_operation("向量化文本");
    assert!(with_operation.is_err());
    
    let error_msg = format!("{}", with_operation.unwrap_err());
    assert!(error_msg.contains("操作 '向量化文本' 失败"));
}

#[test]
fn test_error_severity() {
    let errors = vec![
        (Error::OutOfMemoryError { current_mb: 300, limit_mb: 200 }, ErrorSeverity::Critical),
        (Error::DatabaseError("数据库连接失败".to_string()), ErrorSeverity::High),
        (Error::ModelError("模型加载失败".to_string()), ErrorSeverity::High),
        (Error::ConfigError("配置错误".to_string()), ErrorSeverity::Medium),
        (Error::TimeoutError { operation: "搜索".to_string(), timeout_ms: 1000 }, ErrorSeverity::Medium),
        (Error::ValidationError("验证失败".to_string()), ErrorSeverity::Low),
    ];
    
    for (error, expected_severity) in errors {
        assert_eq!(error.severity(), expected_severity);
    }
}

#[test]
fn test_retryable_errors() {
    let retryable_errors = vec![
        Error::TimeoutError { operation: "搜索".to_string(), timeout_ms: 1000 },
        Error::ConcurrencyError("并发冲突".to_string()),
        Error::IoError(io::Error::new(io::ErrorKind::TimedOut, "超时")),
    ];
    
    for error in retryable_errors {
        assert!(error.is_retryable());
    }
    
    let non_retryable_errors = vec![
        Error::ConfigError("配置错误".to_string()),
        Error::ModelError("模型错误".to_string()),
        Error::ValidationError("验证错误".to_string()),
    ];
    
    for error in non_retryable_errors {
        assert!(!error.is_retryable());
    }
}

#[test]
fn test_error_display() {
    let errors = vec![
        Error::ConfigError("配置文件格式错误".to_string()),
        Error::ModelError("无法加载bge-small-zh模型".to_string()),
        Error::InferenceError("推理过程中发生错误".to_string()),
        Error::EmbeddingError("向量化失败".to_string()),
        Error::SearchError("搜索查询失败".to_string()),
        Error::GraphError("图计算错误".to_string()),
        Error::CacheError("缓存操作失败".to_string()),
        Error::UnsupportedOperation("不支持的操作类型".to_string()),
        Error::ValidationError("数据验证失败".to_string()),
        Error::ConcurrencyError("并发访问冲突".to_string()),
    ];
    
    for error in errors {
        let error_msg = format!("{}", error);
        assert!(!error_msg.is_empty());
        assert!(error_msg.len() > 5); // 确保有实际的错误信息
    }
}

#[test]
fn test_error_debug() {
    let error = Error::ModelError("测试错误".to_string());
    let debug_msg = format!("{:?}", error);
    assert!(debug_msg.contains("ModelError"));
    assert!(debug_msg.contains("测试错误"));
}

#[test]
fn test_result_type() {
    // 测试Result类型别名
    fn test_function() -> Result<String> {
        Ok("成功".to_string())
    }
    
    fn test_error_function() -> Result<String> {
        Err(Error::ConfigError("测试错误".to_string()))
    }
    
    assert!(test_function().is_ok());
    assert_eq!(test_function().unwrap(), "成功");
    
    assert!(test_error_function().is_err());
    assert!(matches!(test_error_function().unwrap_err(), Error::ConfigError(_)));
}

#[test]
fn test_error_chain() {
    // 测试错误链
    let base_error = Error::ModelError("基础错误".to_string());
    let result: Result<()> = Err(base_error);
    
    let chained_result: Result<()> = result
        .with_context(|| "第一层上下文".to_string())
        .and_then(|_| Err(Error::InferenceError("推理错误".to_string())))
        .with_context(|| "第二层上下文".to_string());
    
    assert!(chained_result.is_err());
    let error_msg = format!("{}", chained_result.unwrap_err());
    assert!(error_msg.contains("第一层上下文"));
}
