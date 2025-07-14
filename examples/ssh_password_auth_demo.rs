#!/usr/bin/env cargo
//! SSH密码认证方法演示
//! 
//! 这个示例展示了如何使用不同的SSH密码认证方法
//! 
//! 运行方式:
//! ```bash
//! cargo run --example ssh_password_auth_demo
//! ```

use iterminal::remote_server::{RemoteServer, AuthMethod, PasswordAuthMethod};
use iterminal::ssh_connection::SshConnectionBuilder;

fn main() {
    println!("=== SSH密码认证方法演示 ===\n");

    // 创建一个测试服务器配置
    let mut server = RemoteServer::new(
        "Demo Server".to_string(),
        "example.com".to_string(),
        "demo_user".to_string(),
        AuthMethod::Password("demo_password".to_string()),
    );

    println!("1. 检查SSH工具可用性");
    println!("=========================");
    check_tool_availability();
    println!();

    println!("2. 演示不同的密码认证方法");
    println!("===============================");

    // 演示所有可用的密码认证方法
    let methods = PasswordAuthMethod::available_methods();
    for method in &methods {
        println!("方法: {}", method.display_name());
        println!("描述: {}", method.description());
        println!("可用性: {}", if method.is_available() { "✅ 可用" } else { "❌ 不可用" });
        
        if method.is_available() {
            server.password_auth_method = method.clone();
            demonstrate_method(&server);
        }
        println!();
    }

    println!("3. SSH支持信息");
    println!("================");
    let support_info = SshConnectionBuilder::get_ssh_support_info();
    println!("{}", support_info);
}

fn check_tool_availability() {
    println!("sshpass: {}", if SshConnectionBuilder::check_sshpass_availability() { "✅" } else { "❌" });
    
    #[cfg(unix)]
    println!("expect: {}", if SshConnectionBuilder::check_expect_availability() { "✅" } else { "❌" });
    
    #[cfg(windows)]
    {
        println!("PowerShell SSH: {}", if SshConnectionBuilder::check_powershell_ssh_availability() { "✅" } else { "❌" });
        println!("PuTTY plink: {}", if SshConnectionBuilder::check_plink_availability() { "✅" } else { "❌" });
    }
}

fn demonstrate_method(server: &RemoteServer) {
    if let AuthMethod::Password(password) = &server.auth_method {
        match SshConnectionBuilder::try_password_authentication_methods(server, password) {
            Ok(Some((command, args))) => {
                println!("  生成的命令: {} {}", command, args.join(" "));
            }
            Ok(None) => {
                println!("  将使用交互式密码输入");
            }
            Err(e) => {
                println!("  错误: {}", e);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_auth_methods() {
        let methods = PasswordAuthMethod::available_methods();
        assert!(!methods.is_empty(), "应该有可用的密码认证方法");
        
        // Auto和Interactive应该总是可用
        assert!(methods.contains(&PasswordAuthMethod::Auto));
        assert!(methods.contains(&PasswordAuthMethod::Interactive));
    }

    #[test]
    fn test_server_configuration() {
        let mut server = RemoteServer::new(
            "Test Server".to_string(),
            "test.example.com".to_string(),
            "testuser".to_string(),
            AuthMethod::Password("testpass".to_string()),
        );

        // 测试不同的密码认证方法
        for method in PasswordAuthMethod::available_methods() {
            server.password_auth_method = method.clone();
            
            if let AuthMethod::Password(password) = &server.auth_method {
                let result = SshConnectionBuilder::try_password_authentication_methods(&server, password);
                
                match method {
                    PasswordAuthMethod::Interactive => {
                        // 交互式方法应该总是返回None
                        assert!(result.is_ok());
                        assert!(result.unwrap().is_none());
                    }
                    _ => {
                        // 其他方法的结果取决于工具可用性
                        if method.is_available() {
                            assert!(result.is_ok());
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn test_auto_selection() {
        let server = RemoteServer::new(
            "Auto Test Server".to_string(),
            "auto.example.com".to_string(),
            "autouser".to_string(),
            AuthMethod::Password("autopass".to_string()),
        );

        if let AuthMethod::Password(password) = &server.auth_method {
            let result = SshConnectionBuilder::try_auto_password_authentication(&server, password);
            
            // 自动选择应该总是成功（至少回退到交互式）
            assert!(result.is_ok());
        }
    }

    #[test]
    fn test_ssh_support_info() {
        let info = SshConnectionBuilder::get_ssh_support_info();
        
        // 支持信息应该包含基本的SSH客户端检查
        assert!(info.contains("SSH客户端"));
        assert!(!info.is_empty());
    }

    #[test]
    fn test_method_properties() {
        for method in PasswordAuthMethod::available_methods() {
            // 每个方法都应该有显示名称和描述
            assert!(!method.display_name().is_empty());
            assert!(!method.description().is_empty());
            
            // 可用性检查应该是一致的
            let is_available1 = method.is_available();
            let is_available2 = method.is_available();
            assert_eq!(is_available1, is_available2);
        }
    }
}
