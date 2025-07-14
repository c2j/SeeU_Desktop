#!/usr/bin/env cargo
//! Windows SSH连接调试工具
//! 
//! 这个工具专门用于诊断Windows下的SSH连接问题
//! 
//! 运行方式:
//! ```bash
//! cargo run --example windows_ssh_debug
//! ```

use iterminal::remote_server::{RemoteServer, AuthMethod};
use iterminal::ssh_connection::SshConnectionBuilder;
use std::process::Command;

fn main() {
    // 设置日志级别为DEBUG以获取详细信息
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();

    println!("=== Windows SSH连接调试工具 ===\n");

    // 1. 检查SSH客户端可用性
    println!("1. 检查SSH客户端可用性");
    println!("===============================");
    check_ssh_clients();
    println!();

    // 2. 检查SSH支持信息
    println!("2. SSH支持信息");
    println!("================");
    let support_info = SshConnectionBuilder::get_ssh_support_info();
    println!("{}", support_info);
    println!();

    // 3. 测试SSH命令执行
    println!("3. 测试SSH命令执行");
    println!("==================");
    test_ssh_command_execution();
    println!();

    // 4. 测试连接到示例服务器
    println!("4. 测试连接到示例服务器");
    println!("========================");
    test_example_server_connection();
    println!();

    // 5. 检查网络连接性
    println!("5. 检查网络连接性");
    println!("==================");
    test_network_connectivity();
}

fn check_ssh_clients() {
    let ssh_clients = [
        ("标准SSH", "ssh"),
        ("Windows OpenSSH", "C:\\Windows\\System32\\OpenSSH\\ssh.exe"),
        ("Git Bash SSH", "C:\\Program Files\\Git\\usr\\bin\\ssh.exe"),
        ("Git Bash SSH (x86)", "C:\\Program Files (x86)\\Git\\usr\\bin\\ssh.exe"),
    ];

    for (name, command) in &ssh_clients {
        print!("{}: ", name);
        match Command::new(command).arg("-V").output() {
            Ok(output) => {
                if output.status.success() {
                    let version = String::from_utf8_lossy(&output.stderr); // SSH版本通常输出到stderr
                    println!("✅ 可用 - {}", version.lines().next().unwrap_or("").trim());
                } else {
                    println!("❌ 执行失败 - 退出码: {}", output.status.code().unwrap_or(-1));
                    if !output.stderr.is_empty() {
                        println!("   错误: {}", String::from_utf8_lossy(&output.stderr).trim());
                    }
                }
            }
            Err(e) => {
                println!("❌ 不可用 - {}", e);
            }
        }
    }

    // 检查PowerShell SSH
    print!("PowerShell SSH: ");
    match Command::new("powershell")
        .args(&["-Command", "Get-Command ssh -ErrorAction SilentlyContinue | Select-Object -ExpandProperty Source"])
        .output()
    {
        Ok(output) => {
            if output.status.success() && !output.stdout.is_empty() {
                let path_str = String::from_utf8_lossy(&output.stdout);
                let path = path_str.trim();
                println!("✅ 可用 - {}", path);
            } else {
                println!("❌ 不可用");
            }
        }
        Err(e) => {
            println!("❌ 检查失败 - {}", e);
        }
    }
}

fn test_ssh_command_execution() {
    if let Some(ssh_cmd) = SshConnectionBuilder::get_ssh_command() {
        println!("使用SSH命令: {}", ssh_cmd);
        
        // 测试基本的SSH参数
        let test_args = [
            "-V",
            "-o ConnectTimeout=5",
            "-o StrictHostKeyChecking=no",
        ];

        for arg in &test_args {
            print!("测试参数 '{}': ", arg);
            match Command::new(&ssh_cmd).arg(arg).output() {
                Ok(output) => {
                    if output.status.success() || arg == &"-V" {
                        println!("✅ 支持");
                    } else {
                        println!("❌ 不支持 - 退出码: {}", output.status.code().unwrap_or(-1));
                        if !output.stderr.is_empty() {
                            println!("   错误: {}", String::from_utf8_lossy(&output.stderr).trim());
                        }
                    }
                }
                Err(e) => {
                    println!("❌ 执行失败 - {}", e);
                }
            }
        }

        // 测试Windows特定参数
        #[cfg(target_os = "windows")]
        {
            println!("\n测试Windows特定参数:");
            let windows_args = [
                "-o UserKnownHostsFile=NUL",
                "-o PasswordAuthentication=no",
            ];

            for arg in &windows_args {
                print!("测试参数 '{}': ", arg);
                match Command::new(&ssh_cmd).args(&["-o", "ConnectTimeout=1", arg, "test@localhost", "echo test"]).output() {
                    Ok(output) => {
                        // 对于这种测试，我们主要关心参数是否被接受，而不是连接是否成功
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        if stderr.contains("Bad configuration option") || stderr.contains("unknown option") {
                            println!("❌ 不支持");
                        } else {
                            println!("✅ 支持");
                        }
                    }
                    Err(e) => {
                        println!("❌ 执行失败 - {}", e);
                    }
                }
            }
        }
    } else {
        println!("❌ 未找到可用的SSH命令");
    }
}

fn test_example_server_connection() {
    // 创建一个测试服务器配置
    let test_server = RemoteServer::new(
        "Test Server".to_string(),
        "example.com".to_string(),
        "testuser".to_string(),
        AuthMethod::Password("testpass".to_string()),
    );

    println!("测试服务器: {}@{}:{}", test_server.username, test_server.host, test_server.port);
    
    // 测试连接
    match SshConnectionBuilder::test_connection(&test_server) {
        Ok(result) => {
            println!("连接测试结果: {:?}", result);
            println!("显示文本: {}", result.get_display_text());
            println!("是否成功: {}", result.is_success());
        }
        Err(e) => {
            println!("连接测试失败: {}", e);
        }
    }
}

fn test_network_connectivity() {
    use std::net::{TcpStream, ToSocketAddrs};
    use std::time::Duration;

    let test_hosts = [
        ("Google DNS", "8.8.8.8:53"),
        ("Cloudflare DNS", "1.1.1.1:53"),
        ("GitHub SSH", "github.com:22"),
        ("Example.com SSH", "example.com:22"),
    ];

    for (name, addr_str) in &test_hosts {
        print!("{}: ", name);
        
        match addr_str.to_socket_addrs() {
            Ok(mut addrs) => {
                if let Some(addr) = addrs.next() {
                    match TcpStream::connect_timeout(&addr, Duration::from_secs(5)) {
                        Ok(_) => {
                            println!("✅ 连接成功");
                        }
                        Err(e) => {
                            match e.kind() {
                                std::io::ErrorKind::TimedOut => println!("⏰ 连接超时"),
                                std::io::ErrorKind::ConnectionRefused => println!("🚫 连接被拒绝"),
                                _ => println!("❌ 连接失败: {}", e),
                            }
                        }
                    }
                } else {
                    println!("❌ 无法解析地址");
                }
            }
            Err(e) => {
                println!("❌ DNS解析失败: {}", e);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ssh_availability() {
        // 这个测试确保SSH可用性检查不会崩溃
        let available = SshConnectionBuilder::check_ssh_availability();
        println!("SSH可用性: {}", available);
        
        if available {
            let ssh_cmd = SshConnectionBuilder::get_ssh_command();
            assert!(ssh_cmd.is_some());
            println!("SSH命令: {:?}", ssh_cmd);
        }
    }

    #[test]
    fn test_connection_test_with_invalid_server() {
        let invalid_server = RemoteServer::new(
            "Invalid Server".to_string(),
            "invalid.nonexistent.domain".to_string(),
            "testuser".to_string(),
            AuthMethod::Password("testpass".to_string()),
        );

        match SshConnectionBuilder::test_connection(&invalid_server) {
            Ok(result) => {
                // 应该返回失败结果，而不是错误
                assert!(!result.is_success());
                println!("预期的失败结果: {}", result.get_display_text());
            }
            Err(e) => {
                println!("连接测试错误: {}", e);
            }
        }
    }
}
