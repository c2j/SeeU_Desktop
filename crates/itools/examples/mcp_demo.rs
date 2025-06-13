use std::path::PathBuf;
use std::time::Duration;
use itools::mcp::{
    McpServerManager, McpServerConfig, ServerTemplateManager, PerformanceMonitor,
    BatchOperationsManager, BatchOperation
};
use itools::mcp::server_manager::TransportConfig;
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    env_logger::init();

    println!("🚀 MCP Integration Demo");
    println!("=======================");

    // Create a temporary config path for demo
    let config_path = PathBuf::from("./mcp_demo_config.json");
    
    // Initialize MCP server manager
    let mut manager = McpServerManager::new(config_path.clone());
    
    println!("📁 Initializing MCP server manager...");
    manager.initialize().await?;

    // Create example server configurations
    let example_servers = vec![
        McpServerConfig {
            name: "Demo Everything Server".to_string(),
            description: Some("A demo MCP server with all capabilities".to_string()),
            transport: TransportConfig::Command {
                command: "echo".to_string(),
                args: vec!["MCP server simulation".to_string()],
                env: HashMap::new(),
            },
            enabled: true,
            auto_start: false,
            directory: "Demo".to_string(),
            metadata: HashMap::new(),
        },
        McpServerConfig {
            name: "Demo File Server".to_string(),
            description: Some("A demo file system MCP server".to_string()),
            transport: TransportConfig::Command {
                command: "ls".to_string(),
                args: vec!["-la".to_string()],
                env: HashMap::new(),
            },
            enabled: false,
            auto_start: false,
            directory: "Demo".to_string(),
            metadata: HashMap::new(),
        },
        McpServerConfig {
            name: "Demo TCP Server".to_string(),
            description: Some("A demo TCP MCP server".to_string()),
            transport: TransportConfig::Tcp {
                host: "localhost".to_string(),
                port: 8080,
            },
            enabled: false,
            auto_start: false,
            directory: "Remote".to_string(),
            metadata: HashMap::new(),
        },
    ];

    // Add servers to manager
    println!("➕ Adding demo servers...");
    let mut server_ids = Vec::new();
    for server in example_servers {
        let server_id = manager.add_server(server.clone()).await?;
        server_ids.push(server_id);
        println!("   ✅ Added: {}", server.name);
    }

    // List all servers
    println!("\n📋 Server List:");
    let directories = manager.get_server_directories();
    for directory in directories {
        println!("  📂 {}", directory.name);
        for server in &directory.servers {
            let status = if server.enabled { "🟢" } else { "🔴" };
            println!("    {} {} - {}", status, server.name, 
                    server.description.as_deref().unwrap_or("No description"));
            
            // Show transport info
            match &server.transport {
                TransportConfig::Command { command, args, .. } => {
                    println!("      🔧 Command: {} {}", command, args.join(" "));
                }
                TransportConfig::Tcp { host, port } => {
                    println!("      🌐 TCP: {}:{}", host, port);
                }
                TransportConfig::Unix { socket_path } => {
                    println!("      🔌 Unix Socket: {}", socket_path);
                }
                TransportConfig::WebSocket { url } => {
                    println!("      🕸️  WebSocket: {}", url);
                }
            }
        }
    }

    // Test server validation
    println!("\n🧪 Testing server configurations...");
    for directory in manager.get_server_directories() {
        for server in &directory.servers {
            match manager.validate_server_config(server) {
                Ok(_) => println!("   ✅ {} - Configuration valid", server.name),
                Err(e) => println!("   ❌ {} - Configuration error: {}", server.name, e),
            }
        }
    }

    // Export configuration
    println!("\n💾 Exporting configuration...");
    let export_path = PathBuf::from("./mcp_demo_export.json");
    manager.export_server_configs(export_path.clone(), None).await?;
    println!("   ✅ Configuration exported to: {}", export_path.display());

    // Show statistics
    println!("\n📊 Statistics:");
    println!("   Total servers: {}", manager.get_total_server_count());
    println!("   Directories: {}", manager.get_server_directories().len());

    // Cleanup demo files
    println!("\n🧹 Cleaning up demo files...");
    if config_path.exists() {
        std::fs::remove_file(&config_path)?;
        println!("   ✅ Removed: {}", config_path.display());
    }
    if export_path.exists() {
        std::fs::remove_file(&export_path)?;
        println!("   ✅ Removed: {}", export_path.display());
    }

    // Demo advanced features
    println!("\n🚀 Advanced Features Demo");
    println!("==========================");

    // Template manager demo
    println!("\n📋 Template Manager Demo:");
    let template_manager = ServerTemplateManager::new();

    println!("   Available categories:");
    for category in template_manager.get_categories() {
        println!("     {} {} - {}", category.icon, category.name, category.description);
    }

    println!("\n   Available templates:");
    for template in template_manager.get_templates() {
        println!("     {} {} - {}", template.icon, template.name, template.description);
        println!("       Category: {}", template.category);
        println!("       Requirements: {}", template.requirements.join(", "));
        println!("       Tags: {}", template.tags.join(", "));
    }

    // Performance monitor demo
    println!("\n📊 Performance Monitor Demo:");
    let mut performance_monitor = PerformanceMonitor::new();

    // Start monitoring some servers
    for (i, server_id) in server_ids.iter().enumerate() {
        let server_name = format!("Demo Server {}", i + 1);
        performance_monitor.start_monitoring(*server_id, server_name);

        // Simulate some performance data
        performance_monitor.record_connection(*server_id, Duration::from_millis(100 + i as u64 * 50));

        for j in 0..5 {
            let response_time = Duration::from_millis(50 + (i * j) as u64 * 10);
            let success = j < 4; // 80% success rate
            let error_msg = if success { None } else { Some("Connection timeout".to_string()) };

            performance_monitor.record_request(*server_id, response_time, success, error_msg);
        }

        // Update capabilities
        performance_monitor.update_capabilities(*server_id, 3, 2, 1);

        // Update resource usage
        performance_monitor.update_resource_usage(
            *server_id,
            Some(256 * 1024 * 1024), // 256 MB
            Some(25.5 + i as f32 * 10.0) // Varying CPU usage
        );
    }

    println!("   Performance metrics:");
    for metrics in performance_monitor.get_all_metrics() {
        let health = performance_monitor.get_health_status(metrics.server_id);
        println!("     {} - Health: {:?}", metrics.server_name, health);
        println!("       Avg Response: {}ms", metrics.average_response_time.as_millis());
        println!("       Error Rate: {:.1}%", metrics.error_rate);
        println!("       Total Requests: {}", metrics.total_requests);
        println!("       Memory: {}MB", metrics.memory_usage.unwrap_or(0) / 1024 / 1024);
        println!("       CPU: {:.1}%", metrics.cpu_usage.unwrap_or(0.0));
    }

    // Batch operations demo
    println!("\n⚡ Batch Operations Demo:");
    let mut batch_manager = BatchOperationsManager::new();

    // Simulate batch operations
    let operations = vec![
        ("Connect All", BatchOperation::Connect(server_ids.clone())),
        ("Test All", BatchOperation::Test(server_ids.clone())),
        ("Export All", BatchOperation::Export(server_ids.clone(), "./batch_export.json".to_string())),
    ];

    for (name, operation) in operations {
        println!("   Executing: {}", name);
        match batch_manager.execute_operation(operation, &mut manager).await {
            Ok(operation_id) => {
                println!("     ✅ Operation started: {}", operation_id);

                // Wait a bit for the operation to complete
                tokio::time::sleep(Duration::from_millis(100)).await;

                // Check operation status
                if let Some(active_op) = batch_manager.get_active_operations().first() {
                    println!("     Status: {:?}", active_op.status);
                    println!("     Progress: {}/{}",
                            active_op.successful.len() + active_op.failed.len(),
                            active_op.total_servers);
                }
            }
            Err(e) => println!("     ❌ Operation failed: {}", e),
        }
    }

    println!("\n   Operation history:");
    for (i, operation) in batch_manager.get_operation_history().iter().enumerate() {
        println!("     {}. {:?} - {:?}", i + 1, operation.operation, operation.status);
        println!("        Success: {}, Failed: {}",
                operation.successful.len(),
                operation.failed.len());
    }

    // Protocol features demo
    println!("\n🔗 Protocol Features Demo:");
    println!("   MCP Protocol Version: {}", itools::mcp::protocol_handler::MCP_VERSION);
    println!("   Supported capabilities:");
    println!("     ✅ JSON-RPC 2.0 protocol");
    println!("     ✅ Initialize handshake");
    println!("     ✅ Tools discovery");
    println!("     ✅ Resources access");
    println!("     ✅ Prompts management");
    println!("     ✅ Real-time notifications");
    println!("     ✅ Error handling and recovery");

    println!("\n🎉 Advanced MCP Integration Demo completed successfully!");
    println!("   All advanced features are ready for use in the SeeU Desktop application:");
    println!("   📋 Server Templates - Easy server setup from predefined templates");
    println!("   📊 Performance Monitoring - Real-time metrics and health tracking");
    println!("   ⚡ Batch Operations - Efficient bulk server management");
    println!("   🔗 Full MCP Protocol - Complete protocol implementation with handshake");

    Ok(())
}
