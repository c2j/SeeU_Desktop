use anyhow::Result;
use rmcp::{
    ServiceExt,
    transport::TokioChildProcess,
};
use tokio::process::Command;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use std::time::Duration;

async fn test_mcp_server_with_debug(name: &str, mut command: Command) -> Result<()> {
    println!("\n=== Testing {} ===", name);

    // Add debug output to capture stderr
    command.stderr(std::process::Stdio::piped());
    command.stdout(std::process::Stdio::piped());

    // Start server with timeout
    println!("Starting server...");
    let child_process = TokioChildProcess::new(&mut command)?;

    // Try to connect with detailed error reporting
    let service_result = tokio::time::timeout(
        Duration::from_secs(15),
        ().serve(child_process)
    ).await;

    match service_result {
        Ok(Ok(service)) => {
            println!("✓ Successfully connected to {}", name);

            // Get server info
            let server_info = service.peer_info();
            println!("Server info: {:#?}", server_info);

            // Try to list tools with timeout
            println!("Listing tools...");
            match tokio::time::timeout(Duration::from_secs(5), service.list_all_tools()).await {
                Ok(Ok(tools)) => {
                    println!("✓ Successfully listed tools from {}", name);
                    println!("Number of tools: {}", tools.len());
                    for tool in &tools {
                        println!("  - Tool: {}", tool.name);
                        println!("    Description: {}", tool.description);
                    }
                }
                Ok(Err(e)) => {
                    println!("✗ Failed to list tools from {}: {}", name, e);
                }
                Err(_) => {
                    println!("✗ Timeout while listing tools from {}", name);
                }
            }

            // Cancel the service
            println!("Cancelling service...");
            if let Err(e) = service.cancel().await {
                println!("Warning: Failed to cancel service {}: {}", name, e);
            }
        }
        Ok(Err(e)) => {
            println!("✗ Failed to connect to {}: {}", name, e);
        }
        Err(_) => {
            println!("✗ Timeout while connecting to {}", name);
        }
    }

    Ok(())
}

async fn test_direct_execution() -> Result<()> {
    println!("\n=== Testing Direct Node.js Execution ===");

    let mut cmd = Command::new("node");
    cmd.arg("/Volumes/Raiden_C2J/Projects/Desktop_Projects/CU/SeeU_Desktop/apps/preset_mcpservers/server-filesystem/full-mcp-entry.js")
       .arg("/tmp");

    test_mcp_server_with_debug("Direct Node.js", cmd).await?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("info,{}=debug", env!("CARGO_CRATE_NAME")).into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    println!("MCP Filesystem Server Comparison Test");
    println!("=====================================");

    // Test 1: Direct binary
    let mut cmd1 = Command::new("/Volumes/Raiden_C2J/Projects/Desktop_Projects/CU/SeeU_Desktop/apps/preset_mcpservers/server-filesystem/dist-full-mcp/mcp-filesystem-darwin-x64");
    cmd1.arg("/tmp");

    test_mcp_server_with_debug("Direct Binary", cmd1).await?;

    // Wait a bit between tests
    tokio::time::sleep(Duration::from_secs(2)).await;

    // Test 2: Direct Node.js execution (improved version)
    test_direct_execution().await?;

    // Wait a bit between tests
    tokio::time::sleep(Duration::from_secs(2)).await;

    // Test 3: NPX command
    let mut cmd3 = Command::new("npx");
    cmd3.arg("-y").arg("@modelcontextprotocol/server-filesystem").arg("/tmp");

    test_mcp_server_with_debug("NPX Command", cmd3).await?;

    println!("\n=== Test Complete ===");
    
    Ok(())
}
