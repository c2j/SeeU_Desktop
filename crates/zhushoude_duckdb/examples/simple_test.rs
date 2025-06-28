//! 简单的功能测试

use zhushoude_duckdb::*;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🧪 简单功能测试");
    
    // 1. 测试数据库连接
    println!("1. 测试数据库连接...");
    let config = ZhushoudeConfig {
        database_path: ":memory:".to_string(),
        ..Default::default()
    };
    
    let db_manager = Arc::new(DatabaseManager::new(config).await?);
    println!("✅ 数据库连接成功");
    
    // 2. 测试向量索引管理器创建
    println!("2. 测试向量索引管理器...");
    let mut index_manager = VectorIndexManager::new(db_manager.clone());
    index_manager.initialize().await?;
    println!("✅ 向量索引管理器初始化成功");
    
    // 3. 测试创建简单索引
    println!("3. 测试创建索引...");
    let index_name = index_manager.create_index(
        "test_table",
        "test_column",
        IndexType::Linear,
        128
    ).await?;
    println!("✅ 创建索引成功: {}", index_name);
    
    // 4. 测试列出索引
    println!("4. 测试列出索引...");
    let indexes = index_manager.list_indexes();
    println!("✅ 找到 {} 个索引", indexes.len());
    for index in &indexes {
        println!("  - {}", index);
    }
    
    // 5. 测试获取索引信息
    println!("5. 测试获取索引信息...");
    if let Some(info) = index_manager.get_index_info(&index_name) {
        println!("✅ 索引信息: 类型={:?}, 维度={}", info.index_type, info.dimension);
    }
    
    // 6. 测试删除索引
    println!("6. 测试删除索引...");
    index_manager.drop_index(&index_name).await?;
    println!("✅ 删除索引成功");
    
    let remaining = index_manager.list_indexes();
    println!("✅ 剩余索引数量: {}", remaining.len());
    
    println!("\n🎉 所有测试通过！");
    Ok(())
}
