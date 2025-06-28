//! 配置模块的单元测试

use zhushoude_duckdb::config::*;
use tempfile::NamedTempFile;

#[test]
fn test_default_config() {
    let config = ZhushoudeConfig::default();
    
    assert_eq!(config.database_path, "./zhushoude.db");
    assert_eq!(config.embedding.model_name, "bge-small-zh");
    assert_eq!(config.embedding.batch_size, 16);
    assert_eq!(config.embedding.max_cache_size, 5000);
    assert_eq!(config.embedding.vector_dimension, 512);
    assert!(config.embedding.enable_chinese_optimization);
    assert!(config.embedding.normalize_vectors);
    
    assert!(config.graph.enabled);
    assert_eq!(config.graph.max_nodes, 100000);
    assert_eq!(config.graph.max_edges, 500000);
    assert!(config.graph.enable_algorithm_cache);
    
    assert_eq!(config.hybrid.semantic_weight, 0.7);
    assert_eq!(config.hybrid.graph_weight, 0.3);
    assert!(matches!(config.hybrid.fusion_strategy, FusionStrategy::WeightedAverage));
    
    assert_eq!(config.performance.memory_limit_mb, 200);
    assert!(config.performance.enable_monitoring);
    assert!(matches!(config.performance.cache_strategy, CacheStrategy::LRU));
    assert!(config.performance.thread_pool_size.is_none());
}

#[test]
fn test_lightweight_config() {
    let config = ZhushoudeConfig::lightweight();
    
    assert_eq!(config.embedding.batch_size, 8);
    assert_eq!(config.embedding.max_cache_size, 1000);
    assert_eq!(config.performance.memory_limit_mb, 120);
}

#[test]
fn test_high_performance_config() {
    let config = ZhushoudeConfig::high_performance();
    
    assert_eq!(config.embedding.batch_size, 32);
    assert_eq!(config.embedding.max_cache_size, 10000);
    assert_eq!(config.performance.memory_limit_mb, 500);
}

#[test]
fn test_config_validation() {
    let mut config = ZhushoudeConfig::default();
    
    // 有效配置应该通过验证
    assert!(config.validate().is_ok());
    
    // 无效的batch_size
    config.embedding.batch_size = 0;
    assert!(config.validate().is_err());
    config.embedding.batch_size = 16; // 恢复
    
    // 无效的cache_size
    config.embedding.max_cache_size = 0;
    assert!(config.validate().is_err());
    config.embedding.max_cache_size = 5000; // 恢复
    
    // 无效的权重
    config.hybrid.semantic_weight = 0.0;
    config.hybrid.graph_weight = 0.0;
    assert!(config.validate().is_err());
    config.hybrid.semantic_weight = 0.7; // 恢复
    config.hybrid.graph_weight = 0.3; // 恢复
    
    // 无效的内存限制
    config.performance.memory_limit_mb = 50;
    assert!(config.validate().is_err());
}

#[test]
fn test_config_serialization() {
    let config = ZhushoudeConfig::default();
    
    // 测试TOML序列化
    let toml_str = toml::to_string(&config).expect("序列化失败");
    assert!(toml_str.contains("database_path"));
    assert!(toml_str.contains("bge-small-zh"));
    
    // 测试TOML反序列化
    let deserialized: ZhushoudeConfig = toml::from_str(&toml_str).expect("反序列化失败");
    assert_eq!(config.database_path, deserialized.database_path);
    assert_eq!(config.embedding.model_name, deserialized.embedding.model_name);
}

#[test]
fn test_config_file_operations() {
    let config = ZhushoudeConfig::default();
    
    // 创建临时文件
    let temp_file = NamedTempFile::new().expect("创建临时文件失败");
    let file_path = temp_file.path();
    
    // 保存配置到文件
    config.save_to_file(file_path).expect("保存配置失败");
    
    // 从文件加载配置
    let loaded_config = ZhushoudeConfig::from_file(file_path).expect("加载配置失败");
    
    // 验证配置一致性
    assert_eq!(config.database_path, loaded_config.database_path);
    assert_eq!(config.embedding.model_name, loaded_config.embedding.model_name);
    assert_eq!(config.embedding.batch_size, loaded_config.embedding.batch_size);
}

#[test]
fn test_fusion_strategy_serialization() {
    let strategies = vec![
        FusionStrategy::WeightedAverage,
        FusionStrategy::ReciprocalRankFusion,
        FusionStrategy::MaxFusion,
    ];
    
    for strategy in strategies {
        let serialized = serde_json::to_string(&strategy).expect("序列化失败");
        let deserialized: FusionStrategy = serde_json::from_str(&serialized).expect("反序列化失败");
        
        match (&strategy, &deserialized) {
            (FusionStrategy::WeightedAverage, FusionStrategy::WeightedAverage) => {},
            (FusionStrategy::ReciprocalRankFusion, FusionStrategy::ReciprocalRankFusion) => {},
            (FusionStrategy::MaxFusion, FusionStrategy::MaxFusion) => {},
            _ => panic!("序列化/反序列化不匹配"),
        }
    }
}

#[test]
fn test_cache_strategy_serialization() {
    let strategies = vec![
        CacheStrategy::LRU,
        CacheStrategy::LFU,
        CacheStrategy::TimeExpiry,
    ];
    
    for strategy in strategies {
        let serialized = serde_json::to_string(&strategy).expect("序列化失败");
        let deserialized: CacheStrategy = serde_json::from_str(&serialized).expect("反序列化失败");
        
        match (&strategy, &deserialized) {
            (CacheStrategy::LRU, CacheStrategy::LRU) => {},
            (CacheStrategy::LFU, CacheStrategy::LFU) => {},
            (CacheStrategy::TimeExpiry, CacheStrategy::TimeExpiry) => {},
            _ => panic!("序列化/反序列化不匹配"),
        }
    }
}

#[test]
fn test_embedding_config_edge_cases() {
    let mut config = EmbeddingConfig::default();
    
    // 测试极端值
    config.batch_size = 1;
    config.max_cache_size = 1;
    config.vector_dimension = 1;
    
    let serialized = serde_json::to_string(&config).expect("序列化失败");
    let deserialized: EmbeddingConfig = serde_json::from_str(&serialized).expect("反序列化失败");
    
    assert_eq!(config.batch_size, deserialized.batch_size);
    assert_eq!(config.max_cache_size, deserialized.max_cache_size);
    assert_eq!(config.vector_dimension, deserialized.vector_dimension);
}

#[test]
fn test_performance_config_thread_pool() {
    let mut config = PerformanceConfig::default();
    
    // 测试自动检测
    assert!(config.thread_pool_size.is_none());
    
    // 测试手动设置
    config.thread_pool_size = Some(4);
    assert_eq!(config.thread_pool_size, Some(4));
    
    let serialized = serde_json::to_string(&config).expect("序列化失败");
    let deserialized: PerformanceConfig = serde_json::from_str(&serialized).expect("反序列化失败");
    
    assert_eq!(config.thread_pool_size, deserialized.thread_pool_size);
}
