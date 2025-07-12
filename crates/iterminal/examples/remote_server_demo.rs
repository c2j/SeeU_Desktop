use iterminal::remote_server::{AuthMethod, RemoteServer};
use iterminal::remote_server_manager::RemoteServerManager;
use iterminal::ssh_connection::SshConnectionBuilder;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    env_logger::init();

    println!("🚀 iTerminal 远程服务器管理功能演示");
    println!("=====================================");

    // 创建远程服务器管理器
    let mut manager = RemoteServerManager::new()?;
    println!("✅ 远程服务器管理器初始化成功");

    // 创建测试服务器配置
    let mut test_servers = Vec::new();

    // 1. SSH Agent认证服务器
    let server1 = RemoteServer::new(
        "开发服务器".to_string(),
        "dev.example.com".to_string(),
        "developer".to_string(),
        AuthMethod::Agent,
    );
    test_servers.push(server1);

    // 2. 私钥认证服务器
    let server2 = RemoteServer::new(
        "生产服务器".to_string(),
        "prod.example.com".to_string(),
        "admin".to_string(),
        AuthMethod::PrivateKey {
            key_path: PathBuf::from("/Users/user/.ssh/id_rsa"),
            passphrase: Some("secret123".to_string()),
        },
    );
    test_servers.push(server2);

    // 3. 密码认证服务器
    let mut server3 = RemoteServer::new(
        "测试服务器".to_string(),
        "192.168.1.100".to_string(),
        "testuser".to_string(),
        AuthMethod::Password("password123".to_string()),
    );
    server3.port = 2222;
    server3.working_directory = Some("/home/testuser/projects".to_string());
    server3.description = Some("用于测试的本地虚拟机".to_string());
    server3.tags = vec!["测试".to_string(), "本地".to_string()];
    test_servers.push(server3);

    // 添加服务器到管理器
    println!("\n📝 添加测试服务器...");
    for server in test_servers {
        match manager.add_server(server.clone()) {
            Ok(_) => println!("✅ 添加服务器: {}", server.name),
            Err(e) => println!("❌ 添加服务器失败: {} - {}", server.name, e),
        }
    }

    // 显示服务器列表
    println!("\n📋 服务器列表:");
    let servers = manager.list_servers();
    for server in &servers {
        println!("  🖥️  {} ({})", server.name, server.get_connection_string());
        println!("      认证: {} | 端口: {}", server.get_auth_method_display(), server.port);
        if let Some(ref desc) = server.description {
            println!("      描述: {}", desc);
        }
        if !server.tags.is_empty() {
            println!("      标签: {}", server.tags.join(", "));
        }
        println!();
    }

    // 测试搜索功能
    println!("🔍 搜索测试:");
    let search_results = manager.search_servers("测试");
    println!("  搜索 '测试' 找到 {} 个结果", search_results.len());
    for server in search_results {
        println!("    - {}", server.name);
    }

    // 测试标签过滤
    let tag_results = manager.filter_by_tag("测试");
    println!("  标签 '测试' 找到 {} 个结果", tag_results.len());

    // 测试SSH命令构建
    println!("\n🔧 SSH命令构建测试:");
    for server in &servers {
        match SshConnectionBuilder::build_ssh_command(server) {
            Ok((command, args)) => {
                println!("  {} -> {} {}", server.name, command, args.join(" "));
            }
            Err(e) => {
                println!("  {} -> 错误: {}", server.name, e);
            }
        }
    }

    // 检查SSH可用性
    println!("\n🔍 SSH客户端检查:");
    if SshConnectionBuilder::check_ssh_availability() {
        println!("✅ SSH客户端可用");
        if let Some(version) = SshConnectionBuilder::get_ssh_version() {
            println!("   版本: {}", version);
        }
    } else {
        println!("❌ SSH客户端不可用");
    }

    // 显示统计信息
    println!("\n📊 统计信息:");
    let stats = manager.get_statistics();
    for (key, value) in stats {
        println!("  {}: {}", key, value);
    }

    // 保存配置
    println!("\n💾 保存配置...");
    match manager.save_to_file() {
        Ok(_) => println!("✅ 配置保存成功"),
        Err(e) => println!("❌ 配置保存失败: {}", e),
    }

    // 导出配置（安全模式）
    println!("\n📤 导出配置（安全模式）...");
    match manager.export_config(false) {
        Ok(config) => {
            println!("✅ 配置导出成功");
            println!("导出内容预览:");
            let lines: Vec<&str> = config.lines().take(10).collect();
            for line in lines {
                println!("  {}", line);
            }
            if config.lines().count() > 10 {
                println!("  ...");
            }
        }
        Err(e) => println!("❌ 配置导出失败: {}", e),
    }

    println!("\n🎉 演示完成！");
    Ok(())
}
