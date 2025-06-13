use std::path::PathBuf;
use itools::mcp::{McpServerManager, McpServerConfig};
use itools::mcp::server_manager::TransportConfig;
use std::collections::HashMap;
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging to see the enhanced test output
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    println!("🎨 MCP UI Optimizations Demo");
    println!("============================");

    // Create a temporary config path for demo
    let config_path = PathBuf::from("./test_mcp_ui_config.json");
    
    // Initialize MCP server manager
    let mut manager = McpServerManager::new(config_path.clone());
    
    println!("📁 Initializing MCP server manager...");
    manager.initialize().await?;

    // Test 1: Add a server and test it
    println!("\n🟢 Test 1: Adding and testing a valid server");
    let valid_server = McpServerConfig {
        id: Uuid::new_v4(),
        name: "Echo Test Server".to_string(),
        description: Some("A simple echo server for testing UI optimizations".to_string()),
        transport: TransportConfig::Command {
            command: "echo".to_string(),
            args: vec!["Hello from MCP Server!".to_string()],
            env: HashMap::new(),
        },
        enabled: true,
        auto_start: false,
        directory: "UI Test".to_string(),
        metadata: HashMap::new(),
    };

    let server_id = manager.add_server(valid_server.clone()).await?;
    println!("✅ Added server with ID: {}", server_id);

    // Test the server (this will demonstrate the enhanced testing with detailed logs)
    println!("🔍 Testing server connection with enhanced logging...");
    match manager.test_server(server_id).await {
        Ok(true) => println!("✅ Test passed: Server is working correctly"),
        Ok(false) => println!("❌ Test failed: Server is not responsive"),
        Err(e) => println!("💥 Test error: {}", e),
    }

    // Test 2: Update server configuration (demonstrates edit functionality)
    println!("\n✏️ Test 2: Updating server configuration");
    let mut updated_config = valid_server.clone();
    updated_config.name = "Updated Echo Server".to_string();
    updated_config.description = Some("Updated description for testing edit functionality".to_string());
    updated_config.directory = "Updated Directory".to_string();

    match manager.update_server(server_id, updated_config).await {
        Ok(()) => println!("✅ Server configuration updated successfully"),
        Err(e) => println!("❌ Failed to update server: {}", e),
    }

    // Test 3: Add an invalid server to test error handling
    println!("\n🔴 Test 3: Adding an invalid server to test error display");
    let invalid_server = McpServerConfig {
        id: Uuid::new_v4(),
        name: "Invalid Command Server".to_string(),
        description: Some("Server with invalid command for testing error display".to_string()),
        transport: TransportConfig::Command {
            command: "nonexistent_command_xyz_123".to_string(),
            args: vec!["--invalid-arg".to_string()],
            env: HashMap::new(),
        },
        enabled: true,
        auto_start: false,
        directory: "Error Test".to_string(),
        metadata: HashMap::new(),
    };

    let invalid_server_id = manager.add_server(invalid_server).await?;
    println!("Added invalid server with ID: {}", invalid_server_id);

    // Test the invalid server (this will demonstrate error handling)
    println!("🔍 Testing invalid server (should fail with detailed error info)...");
    match manager.test_server(invalid_server_id).await {
        Ok(true) => println!("❓ Unexpected: Invalid server reported as working"),
        Ok(false) => println!("✅ Expected: Invalid server correctly failed with detailed logs"),
        Err(e) => println!("💥 Test error: {}", e),
    }

    // Test 4: Test a long-running process
    println!("\n🟡 Test 4: Testing timeout handling with long-running process");
    let sleep_server = McpServerConfig {
        id: Uuid::new_v4(),
        name: "Sleep Server".to_string(),
        description: Some("Server that sleeps to test timeout handling".to_string()),
        transport: TransportConfig::Command {
            command: "sleep".to_string(),
            args: vec!["3".to_string()],
            env: HashMap::new(),
        },
        enabled: true,
        auto_start: false,
        directory: "Timeout Test".to_string(),
        metadata: HashMap::new(),
    };

    let sleep_server_id = manager.add_server(sleep_server).await?;
    println!("Added sleep server with ID: {}", sleep_server_id);

    // Test the sleep server
    println!("🔍 Testing sleep server (should handle timeout gracefully)...");
    match manager.test_server(sleep_server_id).await {
        Ok(true) => println!("✅ Sleep server test passed"),
        Ok(false) => println!("❌ Sleep server test failed"),
        Err(e) => println!("💥 Sleep server test error: {}", e),
    }

    println!("\n🎯 UI Optimizations Summary:");
    println!("============================");
    println!("1. ✏️  Edit Icon: Changed from 🔧 (settings) to ✏️ (edit) for better clarity");
    println!("2. 🔍  Test Icon: Changed from 🧪 (test tube) to 🔍 (magnifying glass) for better intuition");
    println!("3. 📍  Server-specific Messages: Status/error messages now appear under each server");
    println!("4. 🎨  Enhanced Error Handling: Detailed error logs with process information");
    println!("5. ⏱️  Timeout Management: Proper timeout handling for long-running tests");
    println!("6. 🔄  JSON Edit Support: Servers can now be edited via JSON configuration");
    
    // Display server directories to show the organization
    println!("\n📂 Server Directory Structure:");
    let directories = manager.get_server_directories();
    for dir in directories {
        if !dir.servers.is_empty() {
            println!("  📁 {} ({} servers)", dir.name, dir.servers.len());
            for server in &dir.servers {
                println!("    - {} ({})", server.name, if server.enabled { "enabled" } else { "disabled" });
            }
        }
    }
    
    // Clean up
    std::fs::remove_file(&config_path).ok();
    
    println!("\n✨ UI Optimizations Demo completed!");
    println!("The enhanced MCP settings interface now provides:");
    println!("- Better visual feedback with appropriate icons");
    println!("- Server-specific status messages");
    println!("- JSON-based configuration editing");
    println!("- Improved error handling and logging");
    
    Ok(())
}
