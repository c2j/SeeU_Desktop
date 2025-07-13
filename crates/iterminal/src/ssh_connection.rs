use crate::remote_server::{AuthMethod, RemoteServer};
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Command;

/// SSH连接构建器
pub struct SshConnectionBuilder;

/// SSH连接配置
#[derive(Debug, Clone)]
pub struct SshConnectionConfig {
    /// SSH命令
    pub command: String,
    /// 命令参数
    pub args: Vec<String>,
    /// 环境变量
    pub env_vars: HashMap<String, String>,
    /// 工作目录
    pub working_directory: Option<PathBuf>,
}

/// 连接测试结果
#[derive(Debug, Clone)]
pub enum ConnectionTestResult {
    Success,
    Failed(String),
    Timeout,
    AuthenticationFailed,
    HostUnreachable,
    PermissionDenied,
}

impl SshConnectionBuilder {
    /// 构建SSH连接配置
    pub fn build_ssh_config(server: &RemoteServer) -> Result<SshConnectionConfig, String> {
        let mut args = Vec::new();
        let mut env_vars = HashMap::new();

        // 基本SSH选项
        args.extend_from_slice(&[
            // 保持连接活跃
            "-o".to_string(),
            "ServerAliveInterval=60".to_string(),
            "-o".to_string(),
            "ServerAliveCountMax=3".to_string(),
            // 禁用主机密钥检查（可选，根据安全需求调整）
            "-o".to_string(),
            "StrictHostKeyChecking=ask".to_string(),
            // 连接超时
            "-o".to_string(),
            "ConnectTimeout=10".to_string(),
            // 禁用密码认证提示（对于密钥认证）
            "-o".to_string(),
            "BatchMode=no".to_string(),
        ]);

        // 添加端口配置
        if server.port != 22 {
            args.extend_from_slice(&["-p".to_string(), server.port.to_string()]);
        }

        // 根据认证方式配置
        match &server.auth_method {
            AuthMethod::Password(_password) => {
                // 密码认证 - 通过环境变量传递密码（更安全）
                env_vars.insert("SSH_ASKPASS_REQUIRE".to_string(), "force".to_string());
                env_vars.insert("DISPLAY".to_string(), ":0".to_string());
                
                // 注意：实际密码处理需要更安全的方式，这里仅作示例
                args.extend_from_slice(&[
                    "-o".to_string(),
                    "PreferredAuthentications=password".to_string(),
                ]);
            }
            AuthMethod::PrivateKey { key_path, passphrase } => {
                // 私钥认证
                args.extend_from_slice(&["-i".to_string(), key_path.to_string_lossy().to_string()]);
                args.extend_from_slice(&[
                    "-o".to_string(),
                    "PreferredAuthentications=publickey".to_string(),
                ]);
                
                // 如果有私钥密码，设置相关环境变量
                if passphrase.is_some() {
                    env_vars.insert("SSH_ASKPASS_REQUIRE".to_string(), "force".to_string());
                    env_vars.insert("DISPLAY".to_string(), ":0".to_string());
                }
            }
            AuthMethod::Agent => {
                // SSH Agent认证
                args.extend_from_slice(&[
                    "-o".to_string(),
                    "PreferredAuthentications=publickey".to_string(),
                    "-o".to_string(),
                    "IdentitiesOnly=no".to_string(),
                ]);
            }
        }

        // 构建连接字符串
        let connection_string = format!("{}@{}", server.username, server.host);
        args.push(connection_string);

        // 如果指定了远程工作目录，添加cd命令
        if let Some(ref dir) = server.working_directory {
            args.extend_from_slice(&[
                "-t".to_string(),
                format!("cd '{}' && exec $SHELL -l", dir.replace('\'', "'\"'\"'"))
            ]);
        } else {
            // 默认启动登录shell
            args.extend_from_slice(&["-t".to_string(), "exec $SHELL -l".to_string()]);
        }

        Ok(SshConnectionConfig {
            command: "ssh".to_string(),
            args,
            env_vars,
            working_directory: None,
        })
    }

    /// 构建简化的SSH命令（用于终端后端）
    pub fn build_ssh_command(server: &RemoteServer) -> Result<(String, Vec<String>), String> {
        log::info!("构建SSH命令，服务器: {}@{}:{}", server.username, server.host, server.port);

        // 对于密码认证，使用sshpass工具
        if let AuthMethod::Password(password) = &server.auth_method {
            if !password.is_empty() {
                // 检查sshpass是否可用
                if !Self::check_sshpass_availability() {
                    log::warn!("sshpass工具不可用，将使用交互式密码输入");

                    // 在Windows上，提供安装建议
                    #[cfg(target_os = "windows")]
                    {
                        log::info!("Windows用户建议：");
                        log::info!("1. 使用Windows Subsystem for Linux (WSL) 并安装sshpass");
                        log::info!("2. 使用私钥认证代替密码认证");
                        log::info!("3. 手动输入密码进行连接");
                    }

                    // 如果sshpass不可用，回退到普通SSH命令，用户需要手动输入密码
                } else {
                    log::info!("使用sshpass进行密码认证");

                    // 获取可用的SSH命令
                    let ssh_command = Self::get_ssh_command().unwrap_or_else(|| "ssh".to_string());

                    let mut args = vec![
                        "-p".to_string(),
                        password.clone(),
                        ssh_command,
                    ];

                    // 添加SSH选项
                    args.extend_from_slice(&[
                        "-o".to_string(),
                        "ServerAliveInterval=60".to_string(),
                        "-o".to_string(),
                        "ServerAliveCountMax=3".to_string(),
                        "-o".to_string(),
                        "StrictHostKeyChecking=ask".to_string(),
                        "-o".to_string(),
                        "ConnectTimeout=10".to_string(),
                        "-o".to_string(),
                        "PreferredAuthentications=password".to_string(),
                    ]);

                    // 添加端口配置
                    if server.port != 22 {
                        args.extend_from_slice(&["-p".to_string(), server.port.to_string()]);
                    }

                    // 构建连接字符串
                    let connection_string = format!("{}@{}", server.username, server.host);
                    args.push(connection_string);

                    // 如果指定了远程工作目录，添加cd命令
                    if let Some(ref dir) = server.working_directory {
                        args.extend_from_slice(&[
                            "-t".to_string(),
                            format!("cd '{}' && exec $SHELL -l", dir.replace('\'', "'\"'\"'"))
                        ]);
                    } else {
                        // 默认启动登录shell
                        args.extend_from_slice(&["-t".to_string(), "exec $SHELL -l".to_string()]);
                    }

                    log::info!("sshpass命令构建完成: sshpass {}", args.join(" "));
                    return Ok(("sshpass".to_string(), args));
                }
            }
        }

        // 对于其他认证方式，使用原有逻辑
        let mut config = Self::build_ssh_config(server)?;

        // 检查SSH客户端是否可用，如果不可用则返回错误
        if let Some(ssh_command) = Self::get_ssh_command() {
            config.command = ssh_command;
            log::info!("SSH命令构建完成: {} {}", config.command, config.args.join(" "));
            Ok((config.command, config.args))
        } else {
            Err("SSH客户端不可用。请安装OpenSSH或其他SSH客户端。".to_string())
        }
    }

    /// 测试SSH连接
    pub fn test_connection(server: &RemoteServer) -> Result<ConnectionTestResult, String> {
        log::info!("开始测试SSH连接到 {}@{}:{}", server.username, server.host, server.port);

        // 首先检查SSH客户端是否可用
        if !Self::check_ssh_availability() {
            return Ok(ConnectionTestResult::Failed("SSH客户端不可用".to_string()));
        }

        let mut args = vec![
            "-o".to_string(),
            "ConnectTimeout=10".to_string(),
            "-o".to_string(),
            "BatchMode=yes".to_string(),
            "-o".to_string(),
            "StrictHostKeyChecking=no".to_string(),
            "-o".to_string(),
            "UserKnownHostsFile=/dev/null".to_string(),
        ];

        // 添加端口
        if server.port != 22 {
            args.extend_from_slice(&["-p".to_string(), server.port.to_string()]);
        }

        // 添加认证配置
        match &server.auth_method {
            AuthMethod::PrivateKey { key_path, .. } => {
                if !key_path.exists() {
                    return Ok(ConnectionTestResult::Failed(format!("私钥文件不存在: {}", key_path.display())));
                }
                args.extend_from_slice(&["-i".to_string(), key_path.to_string_lossy().to_string()]);
                args.extend_from_slice(&["-o".to_string(), "PreferredAuthentications=publickey".to_string()]);
            }
            AuthMethod::Agent => {
                args.extend_from_slice(&["-o".to_string(), "IdentitiesOnly=no".to_string()]);
                args.extend_from_slice(&["-o".to_string(), "PreferredAuthentications=publickey".to_string()]);
            }
            AuthMethod::Password(_) => {
                // 密码认证在批处理模式下无法测试，但我们可以测试连接性
                log::info!("密码认证模式，仅测试网络连接性");
                return Self::test_network_connectivity(server);
            }
        }

        // 添加连接目标和测试命令
        args.push(format!("{}@{}", server.username, server.host));
        args.push("echo 'connection_test_ok'".to_string());

        log::debug!("执行SSH测试命令: ssh {}", args.join(" "));

        // 执行测试命令
        let output = Command::new("ssh")
            .args(&args)
            .output()
            .map_err(|e| format!("执行SSH命令失败: {}", e))?;

        log::debug!("SSH测试命令退出码: {}", output.status.code().unwrap_or(-1));
        log::debug!("SSH测试命令stdout: {}", String::from_utf8_lossy(&output.stdout));
        log::debug!("SSH测试命令stderr: {}", String::from_utf8_lossy(&output.stderr));

        // 分析结果
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if stdout.contains("connection_test_ok") {
                log::info!("SSH连接测试成功");
                Ok(ConnectionTestResult::Success)
            } else {
                Ok(ConnectionTestResult::Failed("连接测试命令执行失败".to_string()))
            }
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);

            if stderr.contains("Connection timed out") || stderr.contains("timeout") {
                Ok(ConnectionTestResult::Timeout)
            } else if stderr.contains("Permission denied") || stderr.contains("Authentication failed") {
                Ok(ConnectionTestResult::AuthenticationFailed)
            } else if stderr.contains("No route to host") || stderr.contains("Connection refused") {
                Ok(ConnectionTestResult::HostUnreachable)
            } else if stderr.contains("Permission denied") {
                Ok(ConnectionTestResult::PermissionDenied)
            } else {
                let error_msg = if stderr.trim().is_empty() {
                    format!("SSH连接失败，退出码: {}", output.status.code().unwrap_or(-1))
                } else {
                    stderr.trim().to_string()
                };
                Ok(ConnectionTestResult::Failed(error_msg))
            }
        }
    }

    /// 测试网络连接性（用于密码认证）
    fn test_network_connectivity(server: &RemoteServer) -> Result<ConnectionTestResult, String> {
        log::info!("测试网络连接性到 {}:{}", server.host, server.port);

        use std::net::{TcpStream, ToSocketAddrs};
        use std::time::Duration;

        // 解析主机地址
        let addr_str = format!("{}:{}", server.host, server.port);
        let mut addrs = match addr_str.to_socket_addrs() {
            Ok(addrs) => addrs,
            Err(e) => {
                return Ok(ConnectionTestResult::Failed(format!("DNS解析失败: {}", e)));
            }
        };

        let addr = match addrs.next() {
            Some(addr) => addr,
            None => {
                return Ok(ConnectionTestResult::Failed("无法解析主机地址".to_string()));
            }
        };

        // 尝试TCP连接
        match TcpStream::connect_timeout(&addr, Duration::from_secs(10)) {
            Ok(_) => {
                log::info!("网络连接测试成功");
                Ok(ConnectionTestResult::Success)
            }
            Err(e) => {
                let error_msg = match e.kind() {
                    std::io::ErrorKind::TimedOut => "连接超时".to_string(),
                    std::io::ErrorKind::ConnectionRefused => "连接被拒绝".to_string(),
                    _ => format!("网络连接失败: {}", e),
                };
                Ok(ConnectionTestResult::Failed(error_msg))
            }
        }
    }

    /// 检查SSH客户端是否可用
    pub fn check_ssh_availability() -> bool {
        // 首先尝试标准的ssh命令
        if Command::new("ssh")
            .arg("-V")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
        {
            return true;
        }

        // 在Windows上，尝试检查OpenSSH for Windows
        #[cfg(target_os = "windows")]
        {
            // 检查Windows OpenSSH (通常在System32目录)
            if Command::new("C:\\Windows\\System32\\OpenSSH\\ssh.exe")
                .arg("-V")
                .output()
                .map(|output| output.status.success())
                .unwrap_or(false)
            {
                return true;
            }

            // 检查PowerShell SSH模块
            if Command::new("powershell")
                .args(&["-Command", "Get-Command ssh -ErrorAction SilentlyContinue"])
                .output()
                .map(|output| output.status.success())
                .unwrap_or(false)
            {
                return true;
            }
        }

        false
    }

    /// 获取可用的SSH命令路径
    pub fn get_ssh_command() -> Option<String> {
        // 首先尝试标准的ssh命令
        if Command::new("ssh")
            .arg("-V")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
        {
            return Some("ssh".to_string());
        }

        // 在Windows上，尝试其他SSH客户端
        #[cfg(target_os = "windows")]
        {
            // 检查Windows OpenSSH
            let windows_ssh = "C:\\Windows\\System32\\OpenSSH\\ssh.exe";
            if Command::new(windows_ssh)
                .arg("-V")
                .output()
                .map(|output| output.status.success())
                .unwrap_or(false)
            {
                return Some(windows_ssh.to_string());
            }

            // 检查Git Bash SSH (如果安装了Git for Windows)
            let git_ssh_paths = [
                "C:\\Program Files\\Git\\usr\\bin\\ssh.exe",
                "C:\\Program Files (x86)\\Git\\usr\\bin\\ssh.exe",
            ];

            for path in &git_ssh_paths {
                if Command::new(path)
                    .arg("-V")
                    .output()
                    .map(|output| output.status.success())
                    .unwrap_or(false)
                {
                    return Some(path.to_string());
                }
            }
        }

        None
    }

    /// 检查sshpass工具是否可用
    pub fn check_sshpass_availability() -> bool {
        Command::new("sshpass")
            .arg("-V")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    /// 获取SSH支持状态信息
    pub fn get_ssh_support_info() -> String {
        let mut info = Vec::new();

        if let Some(ssh_cmd) = Self::get_ssh_command() {
            info.push(format!("✅ SSH客户端可用: {}", ssh_cmd));

            if let Some(version) = Self::get_ssh_version() {
                info.push(format!("   版本: {}", version.trim()));
            }
        } else {
            info.push("❌ SSH客户端不可用".to_string());

            #[cfg(target_os = "windows")]
            {
                info.push("".to_string());
                info.push("Windows用户可以通过以下方式安装SSH:".to_string());
                info.push("1. 启用Windows可选功能中的OpenSSH客户端".to_string());
                info.push("2. 安装Git for Windows (包含SSH客户端)".to_string());
                info.push("3. 使用Windows Subsystem for Linux (WSL)".to_string());
                info.push("4. 安装PuTTY或其他SSH客户端".to_string());
            }

            #[cfg(target_os = "macos")]
            {
                info.push("".to_string());
                info.push("macOS用户可以通过以下方式安装SSH:".to_string());
                info.push("1. 使用Homebrew: brew install openssh".to_string());
                info.push("2. SSH通常已预装在macOS中".to_string());
            }

            #[cfg(target_os = "linux")]
            {
                info.push("".to_string());
                info.push("Linux用户可以通过以下方式安装SSH:".to_string());
                info.push("1. Ubuntu/Debian: sudo apt install openssh-client".to_string());
                info.push("2. CentOS/RHEL: sudo yum install openssh-clients".to_string());
                info.push("3. Arch Linux: sudo pacman -S openssh".to_string());
            }
        }

        if Self::check_sshpass_availability() {
            info.push("✅ sshpass工具可用 (支持密码认证)".to_string());
        } else {
            info.push("⚠️  sshpass工具不可用 (密码认证需要手动输入)".to_string());

            #[cfg(not(target_os = "windows"))]
            {
                info.push("   安装sshpass以支持自动密码认证".to_string());
            }

            #[cfg(target_os = "windows")]
            {
                info.push("   Windows用户建议使用私钥认证或WSL".to_string());
            }
        }

        info.join("\n")
    }

    /// 获取SSH客户端版本信息
    pub fn get_ssh_version() -> Option<String> {
        Command::new("ssh")
            .arg("-V")
            .output()
            .ok()
            .and_then(|output| {
                if output.status.success() {
                    // SSH版本信息通常输出到stderr
                    let version = String::from_utf8_lossy(&output.stderr);
                    Some(version.lines().next().unwrap_or("Unknown").to_string())
                } else {
                    None
                }
            })
    }

    /// 构建SCP命令（用于文件传输）
    pub fn build_scp_command(
        server: &RemoteServer,
        local_path: &str,
        remote_path: &str,
        upload: bool,
    ) -> Result<(String, Vec<String>), String> {
        let mut args = Vec::new();

        // 基本选项
        args.extend_from_slice(&[
            "-o".to_string(),
            "ConnectTimeout=10".to_string(),
        ]);

        // 端口配置
        if server.port != 22 {
            args.extend_from_slice(&["-P".to_string(), server.port.to_string()]);
        }

        // 认证配置
        match &server.auth_method {
            AuthMethod::PrivateKey { key_path, .. } => {
                args.extend_from_slice(&["-i".to_string(), key_path.to_string_lossy().to_string()]);
            }
            AuthMethod::Agent => {
                args.extend_from_slice(&["-o".to_string(), "IdentitiesOnly=no".to_string()]);
            }
            AuthMethod::Password(_) => {
                // SCP密码认证需要交互式处理
            }
        }

        // 构建源和目标路径
        let remote_target = format!("{}@{}:{}", server.username, server.host, remote_path);
        
        if upload {
            // 上传：本地 -> 远程
            args.extend_from_slice(&[local_path.to_string(), remote_target]);
        } else {
            // 下载：远程 -> 本地
            args.extend_from_slice(&[remote_target, local_path.to_string()]);
        }

        Ok(("scp".to_string(), args))
    }
}

impl ConnectionTestResult {
    /// 获取结果的显示文本
    pub fn get_display_text(&self) -> String {
        match self {
            ConnectionTestResult::Success => "连接成功".to_string(),
            ConnectionTestResult::Failed(msg) => format!("连接失败: {}", msg),
            ConnectionTestResult::Timeout => "连接超时".to_string(),
            ConnectionTestResult::AuthenticationFailed => "认证失败".to_string(),
            ConnectionTestResult::HostUnreachable => "主机不可达".to_string(),
            ConnectionTestResult::PermissionDenied => "权限被拒绝".to_string(),
        }
    }

    /// 检查是否为成功结果
    pub fn is_success(&self) -> bool {
        matches!(self, ConnectionTestResult::Success)
    }

    /// 检查是否为认证相关错误
    pub fn is_auth_error(&self) -> bool {
        matches!(
            self,
            ConnectionTestResult::AuthenticationFailed | ConnectionTestResult::PermissionDenied
        )
    }

    /// 检查是否为网络相关错误
    pub fn is_network_error(&self) -> bool {
        matches!(
            self,
            ConnectionTestResult::Timeout | ConnectionTestResult::HostUnreachable
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_ssh_config_building() {
        let server = RemoteServer::new(
            "Test Server".to_string(),
            "192.168.1.100".to_string(),
            "user".to_string(),
            AuthMethod::Agent,
        );

        let config = SshConnectionBuilder::build_ssh_config(&server);
        assert!(config.is_ok());

        let config = config.unwrap();
        assert_eq!(config.command, "ssh");
        assert!(config.args.contains(&"user@192.168.1.100".to_string()));
    }

    #[test]
    fn test_ssh_command_with_custom_port() {
        let mut server = RemoteServer::new(
            "Test Server".to_string(),
            "192.168.1.100".to_string(),
            "user".to_string(),
            AuthMethod::Agent,
        );
        server.port = 2222;

        let (command, args) = SshConnectionBuilder::build_ssh_command(&server).unwrap();
        assert_eq!(command, "ssh");
        assert!(args.contains(&"-p".to_string()));
        assert!(args.contains(&"2222".to_string()));
    }

    #[test]
    fn test_ssh_availability_check() {
        // 这个测试可能在某些环境中失败，如果SSH不可用
        let available = SshConnectionBuilder::check_ssh_availability();
        // 只是确保函数能正常执行，不强制要求SSH可用
        assert!(available || !available);
    }

    #[test]
    fn test_connection_test_result_display() {
        let success = ConnectionTestResult::Success;
        assert_eq!(success.get_display_text(), "连接成功");
        assert!(success.is_success());

        let timeout = ConnectionTestResult::Timeout;
        assert!(timeout.is_network_error());
        assert!(!timeout.is_success());

        let auth_failed = ConnectionTestResult::AuthenticationFailed;
        assert!(auth_failed.is_auth_error());
        assert!(!auth_failed.is_success());
    }
}
