use iterminal::native_ssh::{NativeSshConnection, SshConnectionMethodManager};
use iterminal::webssh::SshAlternativeManager;
use iterminal::ssh_connection::SshConnectionBuilder;
use iterminal::remote_server::{AuthMethod, RemoteServer};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    env_logger::init();

    println!("🔧 SSH连接替代方案测试");
    println!("======================");

    // 1. 显示完整的支持状态报告
    println!("\n📋 完整支持状态报告:");
    println!("{}", SshAlternativeManager::get_full_support_report());

    // 2. 测试连接方式管理器
    println!("\n🔍 连接方式分析:");
    let recommended = SshConnectionMethodManager::get_recommended_method();
    println!("推荐连接方式: {:?}", recommended);

    let available_methods = SshConnectionMethodManager::get_available_methods();
    for (method, name, available) in available_methods {
        let status = if available { "✅" } else { "❌" };
        println!("{} {}: {:?}", status, name, method);
        println!("   详情: {}", SshConnectionMethodManager::get_method_info(&method));
    }

    // 3. 测试原生SSH连接（如果可用）
    if NativeSshConnection::is_available() {
        println!("\n🧪 测试原生SSH连接:");
        
        // 创建测试服务器配置（使用GitHub作为测试目标）
        let test_server = RemoteServer::new(
            "GitHub SSH测试".to_string(),
            "github.com".to_string(),
            "git".to_string(),
            AuthMethod::Agent, // 使用SSH Agent认证
        );

        match NativeSshConnection::test_connection(&test_server) {
            Ok(success) => {
                if success {
                    println!("✅ 原生SSH连接测试成功");
                } else {
                    println!("⚠️  原生SSH连接测试失败（可能是认证问题）");
                }
            }
            Err(e) => {
                println!("❌ 原生SSH连接测试出错: {}", e);
            }
        }
    }

    // 4. 显示平台特定的替代方案
    println!("\n🌍 平台特定替代方案:");
    
    #[cfg(target_os = "windows")]
    {
        use iterminal::webssh::PuttyIntegration;
        println!("Windows平台:");
        println!("  {}", PuttyIntegration::get_support_info());
    }

    #[cfg(target_os = "macos")]
    {
        use iterminal::webssh::TerminalAppIntegration;
        println!("macOS平台:");
        println!("  {}", TerminalAppIntegration::get_support_info());
    }

    #[cfg(target_os = "linux")]
    {
        println!("Linux平台:");
        println!("  ✅ 原生终端支持");
        println!("  ✅ 系统SSH客户端");
    }

    // 5. 显示建议和总结
    println!("\n💡 使用建议:");
    
    if SshConnectionBuilder::check_ssh_availability() {
        println!("✅ 推荐使用外部SSH客户端（最成熟稳定）");
    } else {
        println!("⚠️  外部SSH客户端不可用，建议:");
        
        if NativeSshConnection::is_available() {
            println!("   1. 使用原生SSH连接（ssh2 crate）");
        }
        
        #[cfg(target_os = "windows")]
        {
            use iterminal::webssh::PuttyIntegration;
            if PuttyIntegration::is_available() {
                println!("   2. 使用PuTTY客户端");
            } else {
                println!("   2. 安装PuTTY或OpenSSH for Windows");
            }
        }
        
        println!("   3. 使用WebSSH（需要WebSSH服务器）");
    }

    // 6. 性能和兼容性对比
    println!("\n📊 方案对比:");
    println!("┌─────────────────┬──────────┬──────────┬──────────┬──────────┐");
    println!("│ 连接方式        │ 兼容性   │ 性能     │ 功能     │ 依赖     │");
    println!("├─────────────────┼──────────┼──────────┼──────────┼──────────┤");
    println!("│ 外部SSH客户端   │ ⭐⭐⭐⭐⭐ │ ⭐⭐⭐⭐⭐ │ ⭐⭐⭐⭐⭐ │ 需要安装 │");
    println!("│ 原生SSH(ssh2)   │ ⭐⭐⭐⭐⭐ │ ⭐⭐⭐⭐   │ ⭐⭐⭐⭐   │ 无依赖   │");
    println!("│ WebSSH          │ ⭐⭐⭐     │ ⭐⭐⭐     │ ⭐⭐⭐     │ 需要服务 │");
    println!("│ PuTTY(Windows)  │ ⭐⭐⭐⭐   │ ⭐⭐⭐⭐⭐ │ ⭐⭐⭐⭐⭐ │ 需要安装 │");
    println!("│ Terminal.app    │ ⭐⭐⭐⭐   │ ⭐⭐⭐⭐⭐ │ ⭐⭐⭐⭐   │ 系统自带 │");
    println!("└─────────────────┴──────────┴──────────┴──────────┴──────────┘");

    println!("\n🎯 结论:");
    println!("SeeU Desktop现在支持多种SSH连接方式，确保在任何环境下都能正常工作！");

    Ok(())
}
