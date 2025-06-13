use std::path::PathBuf;
use itools::mcp::{McpServerManager, McpServerConfig};
use itools::mcp::server_manager::TransportConfig;
use std::collections::HashMap;
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    println!("🎨 MCP Test Icon Comparison Demo");
    println!("================================");

    // Create a temporary config path for demo
    let config_path = PathBuf::from("./test_icon_comparison_config.json");
    
    // Initialize MCP server manager
    let mut manager = McpServerManager::new(config_path.clone());
    
    println!("📁 Initializing MCP server manager...");
    manager.initialize().await?;

    println!("\n🎯 Test Icon Options Comparison:");
    println!("================================");
    
    // Show different icon options with their meanings
    let icon_options = vec![
        ("🔍", "放大镜", "搜索/查找", "❌ 容易与搜索功能混淆"),
        ("⚡", "闪电", "快速测试/执行", "✅ 推荐：直观表示快速执行"),
        ("🎯", "靶心", "精准测试/验证", "✅ 很好：目标明确的测试"),
        ("🔬", "显微镜", "检测/分析", "✅ 不错：科学检测的感觉"),
        ("🚀", "火箭", "启动测试", "✅ 动感：表示启动/运行"),
        ("⚙️", "齿轮", "运行/测试", "⚠️ 可能与设置功能混淆"),
        ("📊", "图表", "测试结果/分析", "⚠️ 更像结果展示而非测试"),
        ("🔄", "循环箭头", "重新测试", "⚠️ 更像刷新而非测试"),
        ("✅", "对勾", "验证/检查", "⚠️ 可能与成功状态混淆"),
        ("🧪", "试管", "实验/测试", "⚠️ 原图标，不够直观"),
    ];

    for (icon, name, meaning, evaluation) in &icon_options {
        println!("  {} {} - {} | {}", icon, name, meaning, evaluation);
    }

    // Test with the new lightning icon
    println!("\n⚡ Testing with Lightning Icon (New Choice):");
    println!("===========================================");
    
    let test_server = McpServerConfig {
        id: Uuid::new_v4(),
        name: "Lightning Test Server".to_string(),
        description: Some("Testing with the new lightning icon".to_string()),
        transport: TransportConfig::Command {
            command: "echo".to_string(),
            args: vec!["⚡ Lightning fast test!".to_string()],
            env: HashMap::new(),
        },
        enabled: true,
        auto_start: false,
        directory: "Icon Test".to_string(),
        metadata: HashMap::new(),
    };

    let server_id = manager.add_server(test_server).await?;
    println!("Added test server with ID: {}", server_id);

    // Test the server
    println!("⚡ Running lightning-fast test...");
    match manager.test_server(server_id).await {
        Ok(true) => println!("✅ Lightning test passed! ⚡ is working great!"),
        Ok(false) => println!("❌ Lightning test failed"),
        Err(e) => println!("💥 Test error: {}", e),
    }

    println!("\n🎨 Icon Usage Context Examples:");
    println!("===============================");
    println!("在MCP设置界面中，用户会看到：");
    println!("  📁 服务器列表");
    println!("    ├─ 📄 Server Name");
    println!("    └─ 按钮组：");
    println!("       ├─ ✏️  编辑配置");
    println!("       ├─ ⚡  测试连接  ← 新的测试图标");
    println!("       └─ 🔌  连接/断开");
    println!("");
    println!("⚡ 的优势：");
    println!("  ✅ 直观表示'快速执行/测试'");
    println!("  ✅ 在软件界面中常用于'运行/执行'");
    println!("  ✅ 视觉上醒目，容易识别");
    println!("  ✅ 不会与其他功能混淆");
    println!("  ✅ 暗示测试的快速性和即时性");

    println!("\n🔄 其他备选方案：");
    println!("================");
    println!("如果你觉得 ⚡ 不够满意，以下是其他推荐：");
    println!("  🎯 靶心 - 表示精准测试/验证目标");
    println!("  🔬 显微镜 - 表示深入检测/分析");
    println!("  🚀 火箭 - 表示启动/运行测试");
    
    // Clean up
    std::fs::remove_file(&config_path).ok();
    
    println!("\n✨ Icon Comparison Demo completed!");
    println!("当前选择：⚡ (闪电) - 快速测试执行");
    println!("如果你有其他偏好，我们可以轻松更换！");
    
    Ok(())
}
