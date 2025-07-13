use iterminal::remote_server::{AuthMethod, RemoteServer};
use iterminal::ssh_connection::SshConnectionBuilder;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    env_logger::init();

    println!("🔧 简单SSH连接测试");
    println!("==================");

    // 测试SSH客户端可用性
    println!("1. 检查SSH客户端...");
    if SshConnectionBuilder::check_ssh_availability() {
        println!("   ✅ SSH客户端可用");
        if let Some(version) = SshConnectionBuilder::get_ssh_version() {
            println!("   版本: {}", version.trim());
        }
    } else {
        println!("   ❌ SSH客户端不可用");
        return Ok(());
    }

    // 创建测试服务器（不需要密钥环）
    println!("\n2. 创建测试服务器配置...");
    
    // 测试GitHub SSH（公开可访问）
    let github_server = RemoteServer::new(
        "GitHub SSH测试".to_string(),
        "github.com".to_string(),
        "git".to_string(),
        AuthMethod::Agent,
    );

    // 测试无效域名（测试错误处理）
    let invalid_server = RemoteServer::new(
        "无效服务器".to_string(),
        "invalid.nonexistent.domain.test".to_string(),
        "user".to_string(),
        AuthMethod::Agent,
    );

    // 测试本地连接（如果SSH服务运行）
    let localhost_server = RemoteServer::new(
        "本地SSH测试".to_string(),
        "127.0.0.1".to_string(),
        "user".to_string(),
        AuthMethod::Agent,
    );

    let test_servers = vec![
        ("GitHub SSH", github_server),
        ("无效域名", invalid_server),
        ("本地连接", localhost_server),
    ];

    println!("   ✅ 创建了 {} 个测试服务器", test_servers.len());

    // 执行连接测试
    println!("\n3. 开始连接测试...");
    
    for (name, server) in test_servers {
        println!("\n--- 测试: {} ---", name);
        println!("目标: {}", server.get_connection_string());
        println!("认证: {}", server.get_auth_method_display());

        print!("测试中... ");
        std::io::Write::flush(&mut std::io::stdout()).unwrap();

        let start_time = std::time::Instant::now();
        match SshConnectionBuilder::test_connection(&server) {
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
                        println!("   🔐 认证失败（预期结果）");
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

    // 测试密码认证的网络连接性
    println!("\n4. 测试网络连接性（密码认证模式）...");
    let password_server = RemoteServer::new(
        "网络连接测试".to_string(),
        "8.8.8.8".to_string(),
        "test".to_string(),
        AuthMethod::Password("dummy".to_string()),
    );

    print!("测试网络连接到 8.8.8.8:22... ");
    std::io::Write::flush(&mut std::io::stdout()).unwrap();

    let start_time = std::time::Instant::now();
    match SshConnectionBuilder::test_connection(&password_server) {
        Ok(result) => {
            let duration = start_time.elapsed();
            println!("完成 ({:.2}s)", duration.as_secs_f64());
            
            match result {
                iterminal::ssh_connection::ConnectionTestResult::Success => {
                    println!("   ✅ 网络连接成功（端口开放）");
                }
                iterminal::ssh_connection::ConnectionTestResult::Failed(msg) => {
                    println!("   ❌ 网络连接失败: {}", msg);
                }
                _ => {
                    println!("   ℹ️  其他结果: {:?}", result);
                }
            }
        }
        Err(e) => {
            println!("错误: {}", e);
        }
    }

    println!("\n🎉 SSH连接测试完成！");
    println!("\n📋 测试总结:");
    println!("   - 验证了SSH客户端可用性");
    println!("   - 测试了多种连接场景");
    println!("   - 验证了错误处理机制");
    println!("   - 测试了网络连接性检查");

    Ok(())
}
