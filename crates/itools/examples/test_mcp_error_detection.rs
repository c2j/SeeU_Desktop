use std::path::PathBuf;
use itools::mcp::{McpServerManager, McpServerConfig};
use itools::mcp::server_manager::TransportConfig;
use std::collections::HashMap;
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging to see the enhanced error detection
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    println!("🔍 MCP Error Detection Enhancement Demo");
    println!("=======================================");

    // Create a temporary config path for demo
    let config_path = PathBuf::from("./test_mcp_error_detection_config.json");
    
    // Initialize MCP server manager
    let mut manager = McpServerManager::new(config_path.clone());
    
    println!("📁 Initializing MCP server manager...");
    manager.initialize().await?;

    // Test 1: Server that produces stderr errors but process succeeds
    println!("\n🔴 Test 1: Server with stderr errors (should now fail correctly)");
    let filesystem_server = McpServerConfig {
        id: Uuid::new_v4(),
        name: "Filesystem Server with Invalid Path".to_string(),
        description: Some("Server that will produce ENOENT errors in stderr".to_string()),
        transport: TransportConfig::Command {
            command: "npx".to_string(),
            args: vec![
                "-y".to_string(),
                "@modelcontextprotocol/server-filesystem".to_string(),
                "/nonexistent/path/that/does/not/exist".to_string(),
            ],
            env: HashMap::new(),
        },
        enabled: true,
        auto_start: false,
        directory: "Error Detection Test".to_string(),
        metadata: HashMap::new(),
    };

    let fs_server_id = manager.add_server(filesystem_server).await?;
    println!("Added filesystem server with ID: {}", fs_server_id);

    // Test the filesystem server (should now detect stderr errors)
    println!("🔍 Testing filesystem server with invalid path...");
    match manager.test_server(fs_server_id).await {
        Ok(true) => println!("❌ UNEXPECTED: Server reported as successful despite stderr errors"),
        Ok(false) => println!("✅ CORRECT: Server correctly failed due to stderr errors"),
        Err(e) => println!("💥 Test error: {}", e),
    }

    // Test 2: Server that produces different types of errors
    println!("\n🔴 Test 2: Server with permission errors");
    let permission_server = McpServerConfig {
        id: Uuid::new_v4(),
        name: "Permission Error Server".to_string(),
        description: Some("Server that will produce permission errors".to_string()),
        transport: TransportConfig::Command {
            command: "cat".to_string(),
            args: vec!["/etc/shadow".to_string()], // This should produce permission denied
            env: HashMap::new(),
        },
        enabled: true,
        auto_start: false,
        directory: "Error Detection Test".to_string(),
        metadata: HashMap::new(),
    };

    let perm_server_id = manager.add_server(permission_server).await?;
    println!("Added permission test server with ID: {}", perm_server_id);

    // Test the permission server
    println!("🔍 Testing server with permission issues...");
    match manager.test_server(perm_server_id).await {
        Ok(true) => println!("❌ UNEXPECTED: Server reported as successful despite permission errors"),
        Ok(false) => println!("✅ CORRECT: Server correctly failed due to permission errors"),
        Err(e) => println!("💥 Test error: {}", e),
    }

    // Test 3: Valid server (should still pass)
    println!("\n🟢 Test 3: Valid server (should still pass)");
    let valid_server = McpServerConfig {
        id: Uuid::new_v4(),
        name: "Valid Echo Server".to_string(),
        description: Some("Server that should work correctly".to_string()),
        transport: TransportConfig::Command {
            command: "echo".to_string(),
            args: vec!["Hello, this should work!".to_string()],
            env: HashMap::new(),
        },
        enabled: true,
        auto_start: false,
        directory: "Error Detection Test".to_string(),
        metadata: HashMap::new(),
    };

    let valid_server_id = manager.add_server(valid_server).await?;
    println!("Added valid server with ID: {}", valid_server_id);

    // Test the valid server
    println!("🔍 Testing valid server...");
    match manager.test_server(valid_server_id).await {
        Ok(true) => println!("✅ CORRECT: Valid server passed as expected"),
        Ok(false) => println!("❌ UNEXPECTED: Valid server failed"),
        Err(e) => println!("💥 Test error: {}", e),
    }

    // Test 4: Server with warnings but no errors (should pass)
    println!("\n🟡 Test 4: Server with warnings but no errors");
    let warning_server = McpServerConfig {
        id: Uuid::new_v4(),
        name: "Warning Server".to_string(),
        description: Some("Server that produces warnings but no errors".to_string()),
        transport: TransportConfig::Command {
            command: "sh".to_string(),
            args: vec![
                "-c".to_string(),
                "echo 'Warning: This is just a warning' >&2; echo 'Success output'".to_string(),
            ],
            env: HashMap::new(),
        },
        enabled: true,
        auto_start: false,
        directory: "Error Detection Test".to_string(),
        metadata: HashMap::new(),
    };

    let warning_server_id = manager.add_server(warning_server).await?;
    println!("Added warning server with ID: {}", warning_server_id);

    // Test the warning server
    println!("🔍 Testing server with warnings...");
    match manager.test_server(warning_server_id).await {
        Ok(true) => println!("✅ CORRECT: Server with warnings passed (warnings are not errors)"),
        Ok(false) => println!("❌ UNEXPECTED: Server with warnings failed"),
        Err(e) => println!("💥 Test error: {}", e),
    }

    println!("\n🎯 Error Detection Enhancement Summary:");
    println!("======================================");
    println!("✅ Enhanced stderr analysis with error pattern detection");
    println!("✅ Distinguishes between errors and warnings");
    println!("✅ Detects common error patterns:");
    println!("   - ENOENT (file not found)");
    println!("   - EACCES/EPERM (permission denied)");
    println!("   - Error:/ERROR:/error: patterns");
    println!("   - Exception/Fatal/Panic patterns");
    println!("✅ Maintains backward compatibility for valid servers");
    
    // Display server directories to show the test results
    println!("\n📂 Test Results Summary:");
    let directories = manager.get_server_directories();
    for dir in directories {
        if dir.name == "Error Detection Test" && !dir.servers.is_empty() {
            println!("  📁 {} ({} servers)", dir.name, dir.servers.len());
            for server in &dir.servers {
                println!("    - {} ({})", server.name, if server.enabled { "enabled" } else { "disabled" });
            }
        }
    }
    
    // Clean up
    std::fs::remove_file(&config_path).ok();
    
    println!("\n✨ Error Detection Enhancement Demo completed!");
    println!("Now MCP server testing correctly identifies stderr errors and fails appropriately.");
    
    Ok(())
}
