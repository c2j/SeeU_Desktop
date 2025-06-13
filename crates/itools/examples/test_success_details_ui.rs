use std::path::PathBuf;
use itools::mcp::{McpServerManager, McpServerConfig};
use itools::mcp::server_manager::TransportConfig;
use std::collections::HashMap;
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    println!("🎉 MCP Success Details UI Enhancement Demo");
    println!("==========================================");

    // Create a temporary config path for demo
    let config_path = PathBuf::from("./test_success_details_ui_config.json");
    
    // Initialize MCP server manager
    let mut manager = McpServerManager::new(config_path.clone());
    
    println!("📁 Initializing MCP server manager...");
    manager.initialize().await?;

    println!("\n🎯 Testing Success Details Display:");
    println!("==================================");

    // Test 1: Simple successful server with stdout
    println!("\n🟢 Test 1: Simple successful server with stdout");
    let simple_server = McpServerConfig {
        id: Uuid::new_v4(),
        name: "Simple Success Server".to_string(),
        description: Some("Server that produces simple stdout output".to_string()),
        transport: TransportConfig::Command {
            command: "echo".to_string(),
            args: vec!["Hello from MCP server! This is a successful test.".to_string()],
            env: HashMap::new(),
        },
        enabled: true,
        auto_start: false,
        directory: "Success Details Test".to_string(),
        metadata: HashMap::new(),
    };

    let simple_server_id = manager.add_server(simple_server).await?;
    println!("Added simple server with ID: {}", simple_server_id);

    // Test the simple server
    println!("🔍 Testing simple success server...");
    match manager.test_server_detailed(simple_server_id).await {
        Ok(test_result) => {
            println!("✅ Test completed with detailed results:");
            println!("  Success: {}", test_result.success);
            println!("  Stdout: '{}'", test_result.stdout.trim());
            println!("  Stderr: '{}'", test_result.stderr.trim());
            
            if let Some(error_msg) = &test_result.error_message {
                println!("  ❌ Error message: {}", error_msg);
            } else {
                println!("  ✅ No error message - clean success!");
            }
        }
        Err(e) => println!("💥 Test error: {}", e),
    }

    // Test 2: Server with both stdout and stderr (but no errors)
    println!("\n🟡 Test 2: Server with both stdout and stderr (no errors)");
    let mixed_success_server = McpServerConfig {
        id: Uuid::new_v4(),
        name: "Mixed Output Success Server".to_string(),
        description: Some("Server that produces both stdout and stderr but succeeds".to_string()),
        transport: TransportConfig::Command {
            command: "sh".to_string(),
            args: vec![
                "-c".to_string(),
                "echo 'Server started successfully'; echo 'Loading configuration...' >&2; echo 'Ready to serve requests'; echo 'Listening on port 8080' >&2".to_string(),
            ],
            env: HashMap::new(),
        },
        enabled: true,
        auto_start: false,
        directory: "Success Details Test".to_string(),
        metadata: HashMap::new(),
    };

    let mixed_success_server_id = manager.add_server(mixed_success_server).await?;
    println!("Added mixed success server with ID: {}", mixed_success_server_id);

    // Test the mixed success server
    println!("🔍 Testing mixed output success server...");
    match manager.test_server_detailed(mixed_success_server_id).await {
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
            } else {
                println!("  ✅ No error message - clean success with informational stderr!");
            }
        }
        Err(e) => println!("💥 Test error: {}", e),
    }

    // Test 3: Server with verbose output
    println!("\n🔊 Test 3: Server with verbose output");
    let verbose_server = McpServerConfig {
        id: Uuid::new_v4(),
        name: "Verbose Success Server".to_string(),
        description: Some("Server that produces verbose output".to_string()),
        transport: TransportConfig::Command {
            command: "sh".to_string(),
            args: vec![
                "-c".to_string(),
                r#"
                echo "=== MCP Server Startup ==="
                echo "Version: 1.0.0"
                echo "Build: 2024-06-13"
                echo "Initializing components..."
                echo "✓ Configuration loaded"
                echo "✓ Database connected"
                echo "✓ API endpoints registered"
                echo "✓ Security policies applied"
                echo "Server ready to accept connections"
                echo "=== Startup Complete ==="
                "#.to_string(),
            ],
            env: HashMap::new(),
        },
        enabled: true,
        auto_start: false,
        directory: "Success Details Test".to_string(),
        metadata: HashMap::new(),
    };

    let verbose_server_id = manager.add_server(verbose_server).await?;
    println!("Added verbose server with ID: {}", verbose_server_id);

    // Test the verbose server
    println!("🔍 Testing verbose success server...");
    match manager.test_server_detailed(verbose_server_id).await {
        Ok(test_result) => {
            println!("✅ Test completed with detailed results:");
            println!("  Success: {}", test_result.success);
            println!("  Stdout ({} bytes, {} lines):", test_result.stdout.len(), test_result.stdout.lines().count());
            
            // Show first few lines as preview
            let lines: Vec<&str> = test_result.stdout.lines().collect();
            for (i, line) in lines.iter().take(3).enumerate() {
                println!("    {}: {}", i + 1, line);
            }
            if lines.len() > 3 {
                println!("    ... ({} more lines)", lines.len() - 3);
            }
            
            println!("  Stderr: '{}'", if test_result.stderr.is_empty() { "(empty)" } else { &test_result.stderr });
            
            if let Some(error_msg) = &test_result.error_message {
                println!("  ❌ Error message: {}", error_msg);
            } else {
                println!("  ✅ No error message - verbose but successful!");
            }
        }
        Err(e) => println!("💥 Test error: {}", e),
    }

    // Test 4: Server that exits quickly but successfully
    println!("\n⚡ Test 4: Server that exits quickly but successfully");
    let quick_server = McpServerConfig {
        id: Uuid::new_v4(),
        name: "Quick Exit Success Server".to_string(),
        description: Some("Server that exits quickly with success".to_string()),
        transport: TransportConfig::Command {
            command: "sh".to_string(),
            args: vec![
                "-c".to_string(),
                "echo 'Quick test completed successfully'; exit 0".to_string(),
            ],
            env: HashMap::new(),
        },
        enabled: true,
        auto_start: false,
        directory: "Success Details Test".to_string(),
        metadata: HashMap::new(),
    };

    let quick_server_id = manager.add_server(quick_server).await?;
    println!("Added quick exit server with ID: {}", quick_server_id);

    // Test the quick server
    println!("🔍 Testing quick exit success server...");
    match manager.test_server_detailed(quick_server_id).await {
        Ok(test_result) => {
            println!("✅ Test completed with detailed results:");
            println!("  Success: {}", test_result.success);
            println!("  Stdout: '{}'", test_result.stdout.trim());
            println!("  Stderr: '{}'", test_result.stderr.trim());
            
            if let Some(error_msg) = &test_result.error_message {
                println!("  ❌ Error message: {}", error_msg);
            } else {
                println!("  ✅ No error message - quick and successful!");
            }
        }
        Err(e) => println!("💥 Test error: {}", e),
    }

    println!("\n🎨 Success Details UI Enhancement Summary:");
    println!("=========================================");
    println!("✅ Enhanced success result display:");
    println!("   - Shows collapsible details for successful tests too");
    println!("   - Displays stdout output with green styling");
    println!("   - Shows stderr output with blue styling (informational)");
    println!("   - Maintains clean interface with expandable sections");
    
    println!("✅ User experience benefits:");
    println!("   - Users can verify server is actually working");
    println!("   - Debug information available for both success and failure");
    println!("   - Consistent interface for all test results");
    println!("   - Easy to distinguish between error stderr and info stderr");
    
    println!("✅ Visual improvements:");
    println!("   - Green color for successful stdout");
    println!("   - Blue color for informational stderr in success cases");
    println!("   - Red color for error stderr in failure cases");
    println!("   - Consistent collapsible behavior");

    // Display server directories to show the test results
    println!("\n📂 Test Results Summary:");
    let directories = manager.get_server_directories();
    for dir in directories {
        if dir.name == "Success Details Test" && !dir.servers.is_empty() {
            println!("  📁 {} ({} servers)", dir.name, dir.servers.len());
            for server in &dir.servers {
                println!("    - {} ({})", server.name, if server.enabled { "enabled" } else { "disabled" });
            }
        }
    }
    
    // Clean up
    std::fs::remove_file(&config_path).ok();
    
    println!("\n✨ Success Details UI Enhancement Demo completed!");
    println!("Now users can see detailed output for both successful and failed MCP server tests!");
    println!("This provides complete transparency and helps users understand what their servers are doing.");
    
    Ok(())
}
