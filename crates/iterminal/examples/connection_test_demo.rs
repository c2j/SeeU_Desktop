use iterminal::remote_server::{AuthMethod, RemoteServer};
use iterminal::remote_server_manager::RemoteServerManager;
use iterminal::ssh_connection::SshConnectionBuilder;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    env_logger::init();

    println!("🔧 iTerminal 连接测试功能演示");
    println!("==============================");

    // 创建远程服务器管理器
    let mut manager = RemoteServerManager::new()?;
    println!("✅ 远程服务器管理器初始化成功");

    // 创建测试服务器配置
    println!("\n📝 创建测试服务器配置...");

    // 1. SSH Agent认证服务器（测试公网服务器）
    let server1 = RemoteServer::new(
        "GitHub SSH测试".to_string(),
        "github.com".to_string(),
        "git".to_string(),
        AuthMethod::Agent,
    );

    // 2. 私钥认证服务器（测试本地私钥）
    let server2 = RemoteServer::new(
        "本地私钥测试".to_string(),
        "localhost".to_string(),
        "user".to_string(),
        AuthMethod::PrivateKey {
            key_path: PathBuf::from(format!("{}/.ssh/id_rsa", std::env::var("HOME").unwrap_or_default())),
            passphrase: None,
        },
    );

    // 3. 密码认证服务器（测试网络连接）
    let mut server3 = RemoteServer::new(
        "网络连接测试".to_string(),
        "8.8.8.8".to_string(),
        "test".to_string(),
        AuthMethod::Password("test123".to_string()),
    );
    server3.port = 22;

    // 4. 无效服务器（测试错误处理）
    let server4 = RemoteServer::new(
        "无效服务器测试".to_string(),
        "invalid.nonexistent.domain".to_string(),
        "user".to_string(),
        AuthMethod::Agent,
    );

    let test_servers = vec![server1, server2, server3, server4];

    // 添加服务器到管理器
    println!("\n➕ 添加测试服务器...");
    for server in &test_servers {
        match manager.add_server(server.clone()) {
            Ok(_) => println!("   ✅ 添加服务器: {}", server.name),
            Err(e) => println!("   ❌ 添加服务器失败: {} - {}", server.name, e),
        }
    }

    // 测试连接功能
    println!("\n🔍 开始连接测试...");
    println!("注意：某些测试可能需要几秒钟时间");

    for server in manager.list_servers() {
        println!("\n--- 测试服务器: {} ---", server.name);
        println!("连接信息: {}", server.get_connection_string());
        println!("认证方式: {}", server.get_auth_method_display());

        // 执行连接测试
        print!("测试中... ");
        std::io::Write::flush(&mut std::io::stdout()).unwrap();

        let start_time = std::time::Instant::now();
        match SshConnectionBuilder::test_connection(server) {
            Ok(result) => {
                let duration = start_time.elapsed();
                println!("完成 ({:.2}s)", duration.as_secs_f64());
                
                match result {
                    iterminal::ssh_connection::ConnectionTestResult::Success => {
                        println!("   ✅ 连接成功");
                    }
                    iterminal::ssh_connection::ConnectionTestResult::Failed(msg) => {
                        println!("   ❌ 连接失败: {}", msg);
                    }
                    iterminal::ssh_connection::ConnectionTestResult::Timeout => {
                        println!("   ⏰ 连接超时");
                    }
                    iterminal::ssh_connection::ConnectionTestResult::AuthenticationFailed => {
                        println!("   🔐 认证失败");
                    }
                    iterminal::ssh_connection::ConnectionTestResult::HostUnreachable => {
                        println!("   🚫 主机不可达");
                    }
                    iterminal::ssh_connection::ConnectionTestResult::PermissionDenied => {
                        println!("   🔒 权限被拒绝");
                    }
                }
            }
            Err(e) => {
                println!("错误");
                println!("   ❌ 测试失败: {}", e);
            }
        }
    }

    // 测试SSH客户端可用性
    println!("\n🔧 SSH客户端检查:");
    if SshConnectionBuilder::check_ssh_availability() {
        println!("   ✅ SSH客户端可用");
        if let Some(version) = SshConnectionBuilder::get_ssh_version() {
            println!("   版本: {}", version.trim());
        }
    } else {
        println!("   ❌ SSH客户端不可用");
    }

    // 显示配置保存状态
    println!("\n💾 配置状态:");
    if manager.has_unsaved_changes() {
        println!("   ⚠️  有未保存的更改");
        match manager.save_to_file() {
            Ok(_) => println!("   ✅ 配置已保存"),
            Err(e) => println!("   ❌ 保存失败: {}", e),
        }
    } else {
        println!("   ✅ 配置已是最新");
    }

    println!("\n🎉 连接测试演示完成！");
    println!("\n📋 测试总结:");
    println!("   - 测试了多种认证方式的连接");
    println!("   - 验证了错误处理机制");
    println!("   - 检查了SSH客户端可用性");
    println!("   - 确认了配置保存功能");

    Ok(())
}
