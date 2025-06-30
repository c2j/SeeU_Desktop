use zhushoude_duckdb::*;
use duckdb::params;
use tokio;

#[tokio::test]
async fn test_index_creation_debug() {
    println!("🔄 开始索引创建调试测试...");
    
    let config = ZhushoudeConfig {
        database_path: ":memory:".to_string(),
        ..Default::default()
    };
    
    // 创建数据库管理器
    let db_manager = DatabaseManager::new(config).await.expect("Failed to create database manager");
    
    println!("✅ 数据库管理器创建成功");

    // 测试索引创建
    println!("🔄 测试索引创建...");
    match db_manager.create_indexes().await {
        Ok(_) => println!("✅ 所有索引创建成功"),
        Err(e) => println!("❌ 索引创建失败: {}", e),
    }
    
    println!("✅ 索引创建调试测试完成");
}

#[tokio::test]
async fn test_index_creation_with_timeout() {
    println!("🔄 开始带超时的索引创建测试...");
    
    let config = ZhushoudeConfig {
        database_path: ":memory:".to_string(),
        ..Default::default()
    };
    
    // 创建数据库管理器
    let db_manager = DatabaseManager::new(config).await.expect("Failed to create database manager");
    
    println!("✅ 数据库管理器创建成功");
    
    // 使用超时测试索引创建
    let timeout_duration = std::time::Duration::from_secs(5);
    
    let result = tokio::time::timeout(timeout_duration, async {
        println!("🔄 开始创建所有索引（带5秒超时）...");
        db_manager.create_indexes().await
    }).await;
    
    match result {
        Ok(Ok(())) => println!("✅ 所有索引在超时时间内创建成功"),
        Ok(Err(e)) => println!("❌ 索引创建失败: {}", e),
        Err(_) => println!("⏰ 索引创建超时（5秒）"),
    }
}
