use std::path::PathBuf;
use itools::mcp::{McpServerManager, McpServerConfig};
use itools::mcp::server_manager::TransportConfig;
use std::collections::HashMap;
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    println!("🔧 MCP Error Details UI Enhancement Demo");
    println!("========================================");

    // Create a temporary config path for demo
    let config_path = PathBuf::from("./test_error_details_ui_config.json");
    
    // Initialize MCP server manager
    let mut manager = McpServerManager::new(config_path.clone());
    
    println!("📁 Initializing MCP server manager...");
    manager.initialize().await?;

    println!("\n🎯 Testing Error Details Collection:");
    println!("===================================");

    // Test 1: Server with stderr errors
    println!("\n🔴 Test 1: Server with stderr errors");
    let error_server = McpServerConfig {
        id: Uuid::new_v4(),
        name: "Error Server".to_string(),
        description: Some("Server that produces stderr errors".to_string()),
        transport: TransportConfig::Command {
            command: "npx".to_string(),
            args: vec![
                "-y".to_string(),
                "@modelcontextprotocol/server-filesystem".to_string(),
                "/nonexistent/path".to_string(),
            ],
            env: HashMap::new(),
        },
        enabled: true,
        auto_start: false,
        directory: "Error Details Test".to_string(),
        metadata: HashMap::new(),
    };

    let error_server_id = manager.add_server(error_server).await?;
    println!("Added error server with ID: {}", error_server_id);

    // Test the error server and get detailed results
    println!("🔍 Testing error server with detailed output capture...");
    match manager.test_server_detailed(error_server_id).await {
        Ok(test_result) => {
            println!("✅ Test completed with detailed results:");
            println!("  Success: {}", test_result.success);
            println!("  Stdout length: {} bytes", test_result.stdout.len());
            println!("  Stderr length: {} bytes", test_result.stderr.len());
            
            if !test_result.stdout.is_empty() {
                println!("  📤 Stdout preview: {}", 
                    test_result.stdout.lines().take(2).collect::<Vec<_>>().join(" | "));
            }
            
            if !test_result.stderr.is_empty() {
                println!("  📥 Stderr preview: {}", 
                    test_result.stderr.lines().take(2).collect::<Vec<_>>().join(" | "));
            }
            
            if let Some(error_msg) = &test_result.error_message {
                println!("  ❌ Error message: {}", error_msg);
            }
        }
        Err(e) => println!("💥 Test error: {}", e),
    }

    // Test 2: Server with mixed output
    println!("\n🟡 Test 2: Server with mixed stdout/stderr output");
    let mixed_server = McpServerConfig {
        id: Uuid::new_v4(),
        name: "Mixed Output Server".to_string(),
        description: Some("Server that produces both stdout and stderr".to_string()),
        transport: TransportConfig::Command {
            command: "sh".to_string(),
            args: vec![
                "-c".to_string(),
                "echo 'This is stdout output'; echo 'This is stderr output' >&2; echo 'More stdout'; echo 'Error: Something went wrong' >&2".to_string(),
            ],
            env: HashMap::new(),
        },
        enabled: true,
        auto_start: false,
        directory: "Error Details Test".to_string(),
        metadata: HashMap::new(),
    };

    let mixed_server_id = manager.add_server(mixed_server).await?;
    println!("Added mixed output server with ID: {}", mixed_server_id);

    // Test the mixed server
    println!("🔍 Testing mixed output server...");
    match manager.test_server_detailed(mixed_server_id).await {
        Ok(test_result) => {
            println!("✅ Test completed with detailed results:");
            println!("  Success: {}", test_result.success);
            println!("  Stdout ({} bytes):", test_result.stdout.len());
            for (i, line) in test_result.stdout.lines().enumerate() {
                println!("    {}: {}", i + 1, line);
            }
            println!("  Stderr ({} bytes):", test_result.stderr.len());
            for (i, line) in test_result.stderr.lines().enumerate() {
                println!("    {}: {}", i + 1, line);
            }
            
            if let Some(error_msg) = &test_result.error_message {
                println!("  ❌ Error message: {}", error_msg);
            }
        }
        Err(e) => println!("💥 Test error: {}", e),
    }

    // Test 3: Successful server
    println!("\n🟢 Test 3: Successful server");
    let success_server = McpServerConfig {
        id: Uuid::new_v4(),
        name: "Success Server".to_string(),
        description: Some("Server that succeeds with output".to_string()),
        transport: TransportConfig::Command {
            command: "echo".to_string(),
            args: vec!["Hello from successful server!".to_string()],
            env: HashMap::new(),
        },
        enabled: true,
        auto_start: false,
        directory: "Error Details Test".to_string(),
        metadata: HashMap::new(),
    };

    let success_server_id = manager.add_server(success_server).await?;
    println!("Added success server with ID: {}", success_server_id);

    // Test the success server
    println!("🔍 Testing success server...");
    match manager.test_server_detailed(success_server_id).await {
        Ok(test_result) => {
            println!("✅ Test completed with detailed results:");
            println!("  Success: {}", test_result.success);
            println!("  Stdout: {}", test_result.stdout.trim());
            println!("  Stderr: {}", if test_result.stderr.is_empty() { "(empty)" } else { &test_result.stderr });
            
            if let Some(error_msg) = &test_result.error_message {
                println!("  ❌ Error message: {}", error_msg);
            } else {
                println!("  ✅ No error message");
            }
        }
        Err(e) => println!("💥 Test error: {}", e),
    }

    println!("\n🎨 UI Enhancement Summary:");
    println!("=========================");
    println!("✅ Enhanced test result collection:");
    println!("   - Captures both stdout and stderr");
    println!("   - Provides detailed error messages");
    println!("   - Maintains success/failure status");
    
    println!("✅ UI improvements for error display:");
    println!("   - Collapsible error details section");
    println!("   - Separate stdout and stderr display");
    println!("   - Monospace font for better readability");
    println!("   - Scrollable text areas for long output");
    
    println!("✅ User experience benefits:");
    println!("   - Easy debugging with detailed output");
    println!("   - Clean interface with expandable details");
    println!("   - Clear distinction between output types");
    println!("   - Helpful for troubleshooting server issues");

    // Display server directories to show the test results
    println!("\n📂 Test Results Summary:");
    let directories = manager.get_server_directories();
    for dir in directories {
        if dir.name == "Error Details Test" && !dir.servers.is_empty() {
            println!("  📁 {} ({} servers)", dir.name, dir.servers.len());
            for server in &dir.servers {
                println!("    - {} ({})", server.name, if server.enabled { "enabled" } else { "disabled" });
            }
        }
    }
    
    // Clean up
    std::fs::remove_file(&config_path).ok();
    
    println!("\n✨ Error Details UI Enhancement Demo completed!");
    println!("Now when MCP server tests fail, users can see detailed stdout/stderr output");
    println!("in a collapsible section below the error message for easy debugging!");
    
    Ok(())
}
