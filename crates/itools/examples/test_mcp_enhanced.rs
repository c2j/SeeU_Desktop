use std::path::PathBuf;
use itools::mcp::{McpServerManager, McpServerConfig};
use itools::mcp::server_manager::TransportConfig;
use std::collections::HashMap;
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging to see the enhanced test output
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    println!("🧪 Enhanced MCP Server Testing Demo");
    println!("====================================");

    // Create a temporary config path for demo
    let config_path = PathBuf::from("./test_mcp_config.json");
    
    // Initialize MCP server manager
    let mut manager = McpServerManager::new(config_path.clone());
    
    println!("📁 Initializing MCP server manager...");
    manager.initialize().await?;

    // Test 1: Valid MCP server (should succeed)
    println!("\n🟢 Test 1: Testing valid MCP server");
    let valid_server = McpServerConfig {
        id: Uuid::new_v4(),
        name: "Echo Test Server".to_string(),
        description: Some("A simple echo server for testing".to_string()),
        transport: TransportConfig::Command {
            command: "echo".to_string(),
            args: vec!["Hello MCP Server!".to_string()],
            env: HashMap::new(),
        },
        enabled: true,
        auto_start: false,
        directory: "Test".to_string(),
        metadata: HashMap::new(),
    };

    let server_id = manager.add_server(valid_server).await?;
    println!("Added server with ID: {}", server_id);

    // Test the server
    println!("Testing server connection...");
    match manager.test_server(server_id).await {
        Ok(true) => println!("✅ Test passed: Server is working correctly"),
        Ok(false) => println!("❌ Test failed: Server is not responsive"),
        Err(e) => println!("💥 Test error: {}", e),
    }

    // Test 2: Invalid command (should fail)
    println!("\n🔴 Test 2: Testing invalid MCP server");
    let invalid_server = McpServerConfig {
        id: Uuid::new_v4(),
        name: "Invalid Test Server".to_string(),
        description: Some("A server with invalid command for testing".to_string()),
        transport: TransportConfig::Command {
            command: "nonexistent_command_12345".to_string(),
            args: vec!["--test".to_string()],
            env: HashMap::new(),
        },
        enabled: true,
        auto_start: false,
        directory: "Test".to_string(),
        metadata: HashMap::new(),
    };

    let invalid_server_id = manager.add_server(invalid_server).await?;
    println!("Added invalid server with ID: {}", invalid_server_id);

    // Test the invalid server
    println!("Testing invalid server connection...");
    match manager.test_server(invalid_server_id).await {
        Ok(true) => println!("❓ Unexpected: Invalid server reported as working"),
        Ok(false) => println!("✅ Expected: Invalid server correctly failed"),
        Err(e) => println!("💥 Test error: {}", e),
    }

    // Test 3: Long-running process (should succeed)
    println!("\n🟡 Test 3: Testing long-running process");
    let long_running_server = McpServerConfig {
        id: Uuid::new_v4(),
        name: "Sleep Test Server".to_string(),
        description: Some("A server that sleeps for testing".to_string()),
        transport: TransportConfig::Command {
            command: "sleep".to_string(),
            args: vec!["5".to_string()],
            env: HashMap::new(),
        },
        enabled: true,
        auto_start: false,
        directory: "Test".to_string(),
        metadata: HashMap::new(),
    };

    let sleep_server_id = manager.add_server(long_running_server).await?;
    println!("Added sleep server with ID: {}", sleep_server_id);

    // Test the sleep server
    println!("Testing sleep server connection (this should timeout gracefully)...");
    match manager.test_server(sleep_server_id).await {
        Ok(true) => println!("✅ Test passed: Sleep server is running correctly"),
        Ok(false) => println!("❌ Test failed: Sleep server failed to start"),
        Err(e) => println!("💥 Test error: {}", e),
    }

    println!("\n🎯 Enhanced Testing Summary:");
    println!("- Enhanced error logging with detailed process information");
    println!("- Timeout handling for long-running processes");
    println!("- Stdout/stderr capture for better debugging");
    println!("- Process ID logging for successful starts");
    println!("- Command validation and error reporting");
    
    // Clean up
    std::fs::remove_file(&config_path).ok();
    
    println!("\n✨ Demo completed! Check the logs above for detailed test information.");
    Ok(())
}
