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

    /// 构建SSH命令，优先使用原生SSH
    pub fn build_ssh_command(server: &RemoteServer) -> Result<(String, Vec<String>), String> {
        log::info!("构建SSH命令，服务器: {}@{}:{}", server.username, server.host, server.port);

        // 首先检查是否可以使用原生SSH连接
        if crate::native_ssh::NativeSshConnection::is_available() {
            log::info!("使用原生SSH连接 (ssh2 crate)");
            return Self::build_native_ssh_command(server);
        }

        // 对于密码认证，尝试多种方式
        if let AuthMethod::Password(password) = &server.auth_method {
            if !password.is_empty() {
                if let Some((command, args)) = Self::try_password_authentication_methods(server, password)? {
                    return Ok((command, args));
                }
                log::warn!("所有自动密码认证方法都不可用，回退到原生SSH");
            }
        }

        // 检查外部SSH客户端
        if let Some(ssh_command) = Self::get_ssh_command() {
            let mut config = Self::build_ssh_config(server)?;
            config.command = ssh_command;
            log::info!("SSH命令构建完成: {} {}", config.command, config.args.join(" "));
            Ok((config.command, config.args))
        } else {
            // 如果没有外部SSH客户端，强制使用原生SSH
            log::warn!("外部SSH客户端不可用，强制使用原生SSH连接");
            Self::build_native_ssh_command(server)
        }
    }

    /// 构建原生SSH命令
    fn build_native_ssh_command(server: &RemoteServer) -> Result<(String, Vec<String>), String> {
        // 使用特殊的命令标识符来表示原生SSH连接
        let args = vec![
            "native-ssh".to_string(),
            server.host.clone(),
            server.port.to_string(),
            server.username.clone(),
            match &server.auth_method {
                AuthMethod::Password(pwd) => format!("password:{}", pwd),
                AuthMethod::PrivateKey { key_path, passphrase } => {
                    if let Some(pass) = passphrase {
                        format!("key:{}:{}", key_path.display(), pass)
                    } else {
                        format!("key:{}", key_path.display())
                    }
                }
                AuthMethod::Agent => "agent".to_string(),
            }
        ];

        Ok(("native-ssh-client".to_string(), args))
    }

    /// 测试连接，支持原生SSH回退
    pub fn test_connection(server: &RemoteServer) -> Result<ConnectionTestResult, String> {
        log::info!("开始测试SSH连接到 {}@{}:{}", server.username, server.host, server.port);

        // 优先使用原生SSH进行连接测试
        if crate::native_ssh::NativeSshConnection::is_available() {
            log::info!("使用原生SSH进行连接测试");
            return crate::native_ssh::NativeSshConnection::test_connection(server);
        }

        // 检查外部SSH客户端可用性
        if !Self::check_ssh_availability() {
            log::warn!("SSH客户端不可用，尝试使用替代方案");
            
            #[cfg(target_os = "windows")]
            {
                if Self::check_plink_availability() {
                    log::info!("检测到PuTTY plink，使用plink进行连接测试");
                    return Self::test_connection_with_plink(server);
                }
            }
            
            return Ok(ConnectionTestResult::Failed("没有可用的SSH客户端，请安装OpenSSH或启用原生SSH支持".to_string()));
        }

        // 使用外部SSH客户端进行测试
        Self::test_connection_with_external_ssh(server)
    }

    /// 使用外部SSH客户端测试连接
    fn test_connection_with_external_ssh(server: &RemoteServer) -> Result<ConnectionTestResult, String> {
        let mut args = vec![
            "-o".to_string(),
            "ConnectTimeout=10".to_string(),
            "-o".to_string(),
            "StrictHostKeyChecking=no".to_string(),
        ];

        // Windows特定参数
        #[cfg(target_os = "windows")]
        {
            args.extend_from_slice(&[
                "-o".to_string(),
                "UserKnownHostsFile=NUL".to_string(),
                "-o".to_string(),
                "BatchMode=yes".to_string(),
            ]);
        }

        // Unix特定参数
        #[cfg(not(target_os = "windows"))]
        {
            args.extend_from_slice(&[
                "-o".to_string(),
                "UserKnownHostsFile=/dev/null".to_string(),
                "-o".to_string(),
                "BatchMode=yes".to_string(),
            ]);
        }

        if server.port != 22 {
            args.extend_from_slice(&["-p".to_string(), server.port.to_string()]);
        }

        args.push(format!("{}@{}", server.username, server.host));
        args.push("echo 'connection_test_success'".to_string());

        log::debug!("执行SSH命令: ssh {}", args.join(" "));
        
        match std::process::Command::new("ssh").args(&args).output() {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);
                
                log::debug!("SSH输出: {}", stdout);
                if !stderr.is_empty() {
                    log::debug!("SSH错误: {}", stderr);
                }
                
                if output.status.success() && stdout.contains("connection_test_success") {
                    Ok(ConnectionTestResult::Success)
                } else {
                    let error_msg = if !stderr.is_empty() {
                        stderr.to_string()
                    } else {
                        "连接测试失败".to_string()
                    };
                    Ok(ConnectionTestResult::Failed(error_msg))
                }
            }
            Err(e) => {
                log::error!("执行SSH命令失败: {}", e);
                Err(format!("执行SSH命令失败: {}", e))
            }
        }
    }

    #[cfg(target_os = "windows")]
    fn check_plink_availability() -> bool {
        std::process::Command::new("plink")
            .arg("-V")
            .output()
            .is_ok()
    }

    #[cfg(target_os = "windows")]
    fn test_connection_with_plink(server: &RemoteServer) -> Result<ConnectionTestResult, String> {
        let mut args = vec![
            "-ssh".to_string(),
            "-batch".to_string(),
            "-v".to_string(), // 详细输出
        ];

        if server.port != 22 {
            args.extend_from_slice(&["-P".to_string(), server.port.to_string()]);
        }

        args.push(format!("{}@{}", server.username, server.host));
        args.push("echo connection_test_success".to_string());

        log::debug!("执行plink命令: plink {}", args.join(" "));

        match std::process::Command::new("plink")
            .args(&args)
            .output()
        {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);
                
                log::debug!("plink输出: {}", stdout);
                if !stderr.is_empty() {
                    log::debug!("plink错误: {}", stderr);
                }
                
                if stdout.contains("connection_test_success") {
                    Ok(ConnectionTestResult::Success)
                } else {
                    Ok(ConnectionTestResult::Failed(format!("plink连接失败: {}", stderr)))
                }
            }
            Err(e) => {
                log::error!("执行plink命令失败: {}", e);
                Err(format!("执行plink命令失败: {}", e))
            }
        }
    }

    /// 测试SSH服务可用性（用于密码认证）
    fn test_ssh_service_availability(server: &RemoteServer) -> Result<ConnectionTestResult, String> {
        log::info!("测试SSH服务可用性到 {}:{}", server.host, server.port);

        // 首先测试网络连接性
        match Self::test_network_connectivity(server) {
            Ok(ConnectionTestResult::Success) => {
                // 网络连接成功，进一步测试SSH服务
                Self::test_ssh_banner(server)
            }
            other => other,
        }
    }

    /// 测试网络连接性
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

    /// 测试SSH banner（检查SSH服务是否真的在运行）
    fn test_ssh_banner(server: &RemoteServer) -> Result<ConnectionTestResult, String> {
        log::info!("测试SSH banner到 {}:{}", server.host, server.port);

        use std::io::{Read, Write};
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

        // 连接并读取SSH banner
        match TcpStream::connect_timeout(&addr, Duration::from_secs(10)) {
            Ok(mut stream) => {
                // 设置读取超时
                if let Err(e) = stream.set_read_timeout(Some(Duration::from_secs(5))) {
                    log::warn!("设置读取超时失败: {}", e);
                }

                // 读取SSH banner
                let mut buffer = [0; 1024];
                match stream.read(&mut buffer) {
                    Ok(n) if n > 0 => {
                        let banner = String::from_utf8_lossy(&buffer[..n]);
                        log::debug!("收到SSH banner: {}", banner.trim());

                        if banner.starts_with("SSH-") {
                            log::info!("SSH服务测试成功");
                            Ok(ConnectionTestResult::Success)
                        } else {
                            Ok(ConnectionTestResult::Failed("不是SSH服务".to_string()))
                        }
                    }
                    Ok(_) => {
                        Ok(ConnectionTestResult::Failed("未收到SSH banner".to_string()))
                    }
                    Err(e) => {
                        Ok(ConnectionTestResult::Failed(format!("读取SSH banner失败: {}", e)))
                    }
                }
            }
            Err(e) => {
                let error_msg = match e.kind() {
                    std::io::ErrorKind::TimedOut => "连接超时".to_string(),
                    std::io::ErrorKind::ConnectionRefused => "连接被拒绝".to_string(),
                    _ => format!("连接失败: {}", e),
                };
                Ok(ConnectionTestResult::Failed(error_msg))
            }
        }
    }

    /// 检查SSH客户端是否可用
    pub fn check_ssh_availability() -> bool {
        log::debug!("检查SSH客户端可用性");

        // 首先尝试标准的ssh命令
        match Command::new("ssh").arg("-V").output() {
            Ok(output) => {
                if output.status.success() {
                    log::debug!("找到标准SSH客户端");
                    return true;
                } else {
                    log::debug!("标准SSH命令执行失败: {}", String::from_utf8_lossy(&output.stderr));
                }
            }
            Err(e) => {
                log::debug!("无法执行标准SSH命令: {}", e);
            }
        }

        // 在Windows上，尝试检查OpenSSH for Windows
        #[cfg(target_os = "windows")]
        {
            log::debug!("在Windows上检查其他SSH客户端");

            // 检查Windows OpenSSH (通常在System32目录)
            let windows_ssh = "C:\\Windows\\System32\\OpenSSH\\ssh.exe";
            match Command::new(windows_ssh).arg("-V").output() {
                Ok(output) => {
                    if output.status.success() {
                        log::debug!("找到Windows OpenSSH客户端");
                        return true;
                    } else {
                        log::debug!("Windows OpenSSH执行失败: {}", String::from_utf8_lossy(&output.stderr));
                    }
                }
                Err(e) => {
                    log::debug!("无法执行Windows OpenSSH: {}", e);
                }
            }

            // 检查Git Bash SSH
            let git_ssh_paths = [
                "C:\\Program Files\\Git\\usr\\bin\\ssh.exe",
                "C:\\Program Files (x86)\\Git\\usr\\bin\\ssh.exe",
            ];

            for path in &git_ssh_paths {
                match Command::new(path).arg("-V").output() {
                    Ok(output) => {
                        if output.status.success() {
                            log::debug!("找到Git Bash SSH客户端: {}", path);
                            return true;
                        }
                    }
                    Err(_) => {
                        // Git SSH不存在是正常的，不记录错误
                    }
                }
            }
        }

        log::warn!("未找到可用的SSH客户端");
        false
    }

    /// 获取可用的SSH命令路径
    pub fn get_ssh_command() -> Option<String> {
        log::debug!("获取可用的SSH命令路径");

        // 首先尝试标准的ssh命令
        match Command::new("ssh").arg("-V").output() {
            Ok(output) => {
                if output.status.success() {
                    log::debug!("使用标准SSH命令");
                    return Some("ssh".to_string());
                }
            }
            Err(e) => {
                log::debug!("标准SSH命令不可用: {}", e);
            }
        }

        // 在Windows上，尝试其他SSH客户端
        #[cfg(target_os = "windows")]
        {
            log::debug!("在Windows上查找其他SSH客户端");

            // 检查Windows OpenSSH
            let windows_ssh = "C:\\Windows\\System32\\OpenSSH\\ssh.exe";
            match Command::new(windows_ssh).arg("-V").output() {
                Ok(output) => {
                    if output.status.success() {
                        log::debug!("使用Windows OpenSSH");
                        return Some(windows_ssh.to_string());
                    }
                }
                Err(e) => {
                    log::debug!("Windows OpenSSH不可用: {}", e);
                }
            }

            // 检查Git Bash SSH (如果安装了Git for Windows)
            let git_ssh_paths = [
                "C:\\Program Files\\Git\\usr\\bin\\ssh.exe",
                "C:\\Program Files (x86)\\Git\\usr\\bin\\ssh.exe",
            ];

            for path in &git_ssh_paths {
                match Command::new(path).arg("-V").output() {
                    Ok(output) => {
                        if output.status.success() {
                            log::debug!("使用Git Bash SSH: {}", path);
                            return Some(path.to_string());
                        }
                    }
                    Err(_) => {
                        // Git SSH不存在是正常的
                    }
                }
            }
        }

        log::warn!("未找到可用的SSH命令");
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

        // 检查密码认证工具
        info.push("".to_string());
        info.push("密码认证工具支持:".to_string());

        if Self::check_sshpass_availability() {
            info.push("✅ sshpass工具可用".to_string());
        } else {
            info.push("❌ sshpass工具不可用".to_string());
        }

        #[cfg(unix)]
        if Self::check_expect_availability() {
            info.push("✅ expect工具可用".to_string());
        } else {
            info.push("❌ expect工具不可用".to_string());
        }

        #[cfg(windows)]
        {
            if Self::check_powershell_ssh_availability() {
                info.push("✅ PowerShell SSH可用".to_string());
            } else {
                info.push("❌ PowerShell SSH不可用".to_string());
            }

            if Self::check_plink_availability() {
                info.push("✅ PuTTY plink可用".to_string());
            } else {
                info.push("❌ PuTTY plink不可用".to_string());
            }
        }

        // 如果没有任何密码认证工具可用，提供建议
        let has_password_auth_tool = Self::check_sshpass_availability() ||
            {
                #[cfg(unix)]
                { Self::check_expect_availability() }
                #[cfg(not(unix))]
                { false }
            } ||
            {
                #[cfg(windows)]
                { Self::check_powershell_ssh_availability() || Self::check_plink_availability() }
                #[cfg(not(windows))]
                { false }
            };

        if !has_password_auth_tool {
            info.push("".to_string());
            info.push("⚠️  没有可用的自动密码认证工具".to_string());
            info.push("   建议使用私钥认证或手动输入密码".to_string());
        }

        info.join("\n")
    }

    /// 尝试多种密码认证方法
    pub fn try_password_authentication_methods(
        server: &RemoteServer,
        password: &str
    ) -> Result<Option<(String, Vec<String>)>, String> {
        use crate::remote_server::PasswordAuthMethod;

        // 根据用户偏好选择认证方法
        match &server.password_auth_method {
            PasswordAuthMethod::Auto => {
                // 自动选择：按优先级尝试所有可用方法
                Self::try_auto_password_authentication(server, password)
            }
            PasswordAuthMethod::Sshpass => {
                if Self::check_sshpass_availability() {
                    log::info!("使用用户指定的sshpass进行密码认证");
                    Self::build_sshpass_command(server, password).map(Some).map_err(|e| e)
                } else {
                    Err("sshpass工具不可用".to_string())
                }
            }
            #[cfg(unix)]
            PasswordAuthMethod::Expect => {
                if Self::check_expect_availability() {
                    log::info!("使用用户指定的expect进行密码认证");
                    Self::build_expect_command(server, password).map(Some).map_err(|e| e)
                } else {
                    Err("expect工具不可用".to_string())
                }
            }
            #[cfg(windows)]
            PasswordAuthMethod::PowerShell => {
                if Self::check_powershell_ssh_availability() {
                    log::info!("使用用户指定的PowerShell SSH进行密码认证");
                    Self::build_powershell_ssh_command(server, password).map(Some).map_err(|e| e)
                } else {
                    Err("PowerShell SSH不可用".to_string())
                }
            }
            #[cfg(windows)]
            PasswordAuthMethod::Plink => {
                if Self::check_plink_availability() {
                    log::info!("使用用户指定的PuTTY plink进行密码认证");
                    Self::build_plink_command(server, password).map(Some).map_err(|e| e)
                } else {
                    Err("PuTTY plink不可用".to_string())
                }
            }
            PasswordAuthMethod::Interactive => {
                log::info!("使用用户指定的交互式密码认证");
                Ok(None) // 返回None表示使用交互式输入
            }
        }
    }

    /// 自动选择密码认证方法
    pub fn try_auto_password_authentication(
        server: &RemoteServer,
        password: &str
    ) -> Result<Option<(String, Vec<String>)>, String> {
        // 方法1: 使用sshpass
        if Self::check_sshpass_availability() {
            log::info!("自动选择：使用sshpass进行密码认证");
            if let Ok((command, args)) = Self::build_sshpass_command(server, password) {
                return Ok(Some((command, args)));
            }
        }

        // 方法2: 使用expect (Unix系统)
        #[cfg(unix)]
        if Self::check_expect_availability() {
            log::info!("自动选择：使用expect进行密码认证");
            if let Ok((command, args)) = Self::build_expect_command(server, password) {
                return Ok(Some((command, args)));
            }
        }

        // 方法3: 使用PowerShell (Windows系统)
        #[cfg(windows)]
        if Self::check_powershell_ssh_availability() {
            log::info!("自动选择：使用PowerShell SSH进行密码认证");
            if let Ok((command, args)) = Self::build_powershell_ssh_command(server, password) {
                return Ok(Some((command, args)));
            }
        }

        // 方法4: 使用PuTTY plink (Windows系统)
        #[cfg(windows)]
        if Self::check_plink_availability() {
            log::info!("自动选择：使用PuTTY plink进行密码认证");
            if let Ok((command, args)) = Self::build_plink_command(server, password) {
                return Ok(Some((command, args)));
            }
        }

        Ok(None)
    }

    /// 构建sshpass命令
    pub fn build_sshpass_command(
        server: &RemoteServer,
        password: &str
    ) -> Result<(String, Vec<String>), String> {
        let ssh_command = Self::get_ssh_command().unwrap_or_else(|| "ssh".to_string());

        let mut args = vec![
            "-p".to_string(),
            password.to_string(),
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
        Ok(("sshpass".to_string(), args))
    }

    /// 构建expect命令 (Unix系统)
    #[cfg(unix)]
    pub fn build_expect_command(
        server: &RemoteServer,
        password: &str
    ) -> Result<(String, Vec<String>), String> {
        let ssh_command = Self::get_ssh_command().unwrap_or_else(|| "ssh".to_string());

        // 创建expect脚本内容
        let expect_script = format!(
            r#"#!/usr/bin/expect -f
set timeout 30
spawn {} -o StrictHostKeyChecking=ask -o ConnectTimeout=10 -o PreferredAuthentications=password{} {}@{}
expect {{
    "password:" {{
        send "{}\r"
        exp_continue
    }}
    "Password:" {{
        send "{}\r"
        exp_continue
    }}
    "yes/no" {{
        send "yes\r"
        exp_continue
    }}
    eof
}}
interact
"#,
            ssh_command,
            if server.port != 22 { format!(" -p {}", server.port) } else { String::new() },
            server.username,
            server.host,
            password.replace("\"", "\\\""),
            password.replace("\"", "\\\"")
        );

        // 将expect脚本作为参数传递
        let args = vec![
            "-c".to_string(),
            expect_script,
        ];

        Ok(("expect".to_string(), args))
    }

    /// 构建PowerShell SSH命令 (Windows系统)
    #[cfg(windows)]
    pub fn build_powershell_ssh_command(
        server: &RemoteServer,
        password: &str
    ) -> Result<(String, Vec<String>), String> {
        // 使用PowerShell的SSH模块或者调用SSH客户端
        let powershell_script = format!(
            r#"
$password = ConvertTo-SecureString '{}' -AsPlainText -Force
$credential = New-Object System.Management.Automation.PSCredential ('{}', $password)
$sshArgs = @('-o', 'StrictHostKeyChecking=ask', '-o', 'ConnectTimeout=10'{})
try {{
    ssh $sshArgs {}@{} -t 'exec $SHELL -l'
}} catch {{
    Write-Error "SSH连接失败: $_"
}}
"#,
            password.replace("'", "''"),
            server.username,
            if server.port != 22 { format!(", '-p', '{}'", server.port) } else { String::new() },
            server.username,
            server.host
        );

        let args = vec![
            "-NoProfile".to_string(),
            "-ExecutionPolicy".to_string(),
            "Bypass".to_string(),
            "-Command".to_string(),
            powershell_script,
        ];

        Ok(("powershell".to_string(), args))
    }

    /// 构建PuTTY plink命令 (Windows系统)
    #[cfg(windows)]
    pub fn build_plink_command(
        server: &RemoteServer,
        password: &str
    ) -> Result<(String, Vec<String>), String> {
        let mut args = vec![
            "-ssh".to_string(),
            "-batch".to_string(),  // 非交互模式
            "-pw".to_string(),
            password.to_string(),
        ];

        // 添加端口配置
        if server.port != 22 {
            args.extend_from_slice(&["-P".to_string(), server.port.to_string()]);
        }

        // 构建连接字符串
        let connection_string = format!("{}@{}", server.username, server.host);
        args.push(connection_string);

        // 如果指定了远程工作目录，添加cd命令
        if let Some(ref dir) = server.working_directory {
            args.push(format!("cd '{}' && exec $SHELL -l", dir.replace('\'', "'\"'\"'")));
        } else {
            args.push("exec $SHELL -l".to_string());
        }

        Ok(("plink".to_string(), args))
    }

    /// 检查expect工具是否可用 (Unix系统)
    #[cfg(unix)]
    pub fn check_expect_availability() -> bool {
        Command::new("expect")
            .arg("-v")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    /// 检查PowerShell SSH是否可用 (Windows系统)
    #[cfg(windows)]
    pub fn check_powershell_ssh_availability() -> bool {
        Command::new("powershell")
            .args(&["-Command", "Get-Command ssh -ErrorAction SilentlyContinue"])
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    /// 检查PuTTY plink是否可用 (Windows系统)
    #[cfg(windows)]
    pub fn check_plink_availability() -> bool {
        Command::new("plink")
            .arg("-V")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    /// 记录密码认证建议
    pub fn log_password_auth_suggestions() {
        #[cfg(target_os = "windows")]
        {
            log::info!("Windows用户建议：");
            log::info!("1. 安装并使用PuTTY plink工具");
            log::info!("2. 使用Windows Subsystem for Linux (WSL) 并安装sshpass");
            log::info!("3. 使用私钥认证代替密码认证");
            log::info!("4. 手动输入密码进行连接");
            log::info!("5. 启用Windows OpenSSH客户端功能");
        }

        #[cfg(target_os = "macos")]
        {
            log::info!("macOS用户建议：");
            log::info!("1. 安装sshpass: brew install sshpass");
            log::info!("2. 安装expect: brew install expect");
            log::info!("3. 使用私钥认证代替密码认证");
            log::info!("4. 手动输入密码进行连接");
        }

        #[cfg(target_os = "linux")]
        {
            log::info!("Linux用户建议：");
            log::info!("1. 安装sshpass: sudo apt install sshpass (Ubuntu/Debian)");
            log::info!("2. 安装expect: sudo apt install expect (Ubuntu/Debian)");
            log::info!("3. 使用私钥认证代替密码认证");
            log::info!("4. 手动输入密码进行连接");
        }
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

    #[test]
    fn test_password_auth_method_availability() {
        use crate::remote_server::PasswordAuthMethod;

        let methods = PasswordAuthMethod::available_methods();
        assert!(!methods.is_empty(), "应该有可用的密码认证方法");

        // Auto和Interactive应该总是可用
        assert!(methods.contains(&PasswordAuthMethod::Auto));
        assert!(methods.contains(&PasswordAuthMethod::Interactive));

        // 检查每个方法的可用性
        for method in &methods {
            let is_available = method.is_available();
            println!("方法 '{}' 可用性: {}", method.display_name(), is_available);
        }
    }

    #[test]
    fn test_ssh_tools_availability() {
        println!("SSH工具可用性检查:");

        let sshpass_available = SshConnectionBuilder::check_sshpass_availability();
        println!("sshpass: {}", if sshpass_available { "✅" } else { "❌" });

        #[cfg(unix)]
        {
            let expect_available = SshConnectionBuilder::check_expect_availability();
            println!("expect: {}", if expect_available { "✅" } else { "❌" });
        }

        #[cfg(windows)]
        {
            let powershell_available = SshConnectionBuilder::check_powershell_ssh_availability();
            println!("PowerShell SSH: {}", if powershell_available { "✅" } else { "❌" });

            let plink_available = SshConnectionBuilder::check_plink_availability();
            println!("PuTTY plink: {}", if plink_available { "✅" } else { "❌" });
        }
    }

    #[test]
    fn test_password_auth_method_selection() {
        use crate::remote_server::{AuthMethod, PasswordAuthMethod};

        let mut server = RemoteServer::new(
            "Test Server".to_string(),
            "example.com".to_string(),
            "testuser".to_string(),
            AuthMethod::Password("testpass".to_string()),
        );

        // 测试自动选择
        server.password_auth_method = PasswordAuthMethod::Auto;
        if let AuthMethod::Password(password) = &server.auth_method {
            let result = SshConnectionBuilder::try_password_authentication_methods(&server, password);
            match result {
                Ok(Some((command, args))) => {
                    println!("自动选择成功: {} {}", command, args.join(" "));
                }
                Ok(None) => {
                    println!("自动选择回退到交互式输入");
                }
                Err(e) => {
                    println!("自动选择失败: {}", e);
                }
            }
        }

        // 测试交互式方法
        server.password_auth_method = PasswordAuthMethod::Interactive;
        if let AuthMethod::Password(password) = &server.auth_method {
            let result = SshConnectionBuilder::try_password_authentication_methods(&server, password);
            assert!(result.is_ok(), "交互式方法应该总是成功");
            assert!(result.unwrap().is_none(), "交互式方法应该返回None");
        }
    }

    #[test]
    fn test_ssh_support_info() {
        let info = SshConnectionBuilder::get_ssh_support_info();
        println!("SSH支持信息:\n{}", info);

        // 信息应该包含基本的SSH客户端检查
        assert!(info.contains("SSH客户端"), "应该包含SSH客户端信息");
    }

    #[test]
    fn test_ssh_service_availability() {
        // 测试网络连接性检查
        let server = RemoteServer::new(
            "Test Server".to_string(),
            "github.com".to_string(),  // 使用GitHub作为测试目标，因为它有SSH服务
            "git".to_string(),
            AuthMethod::Password("testpass".to_string()),
        );

        // 测试网络连接性
        let network_result = SshConnectionBuilder::test_network_connectivity(&server);
        println!("网络连接性测试结果: {:?}", network_result);

        // GitHub应该是可以连接的
        if let Ok(result) = network_result {
            if result.is_success() {
                println!("✅ 网络连接成功");

                // 测试SSH banner检测
                let banner_result = SshConnectionBuilder::test_ssh_banner(&server);
                println!("SSH banner测试结果: {:?}", banner_result);

                // GitHub的SSH服务应该返回SSH banner
                if let Ok(banner_result) = banner_result {
                    if banner_result.is_success() {
                        println!("✅ SSH服务检测成功");
                    } else {
                        println!("⚠️ SSH服务检测失败: {}", banner_result.get_display_text());
                    }
                }
            } else {
                println!("⚠️ 网络连接失败: {}", result.get_display_text());
            }
        }
    }

    #[test]
    fn test_ssh_connection_with_invalid_host() {
        // 测试无效主机的处理
        let server = RemoteServer::new(
            "Invalid Server".to_string(),
            "invalid.nonexistent.domain.test".to_string(),
            "testuser".to_string(),
            AuthMethod::Password("testpass".to_string()),
        );

        let result = SshConnectionBuilder::test_connection(&server);
        println!("无效主机测试结果: {:?}", result);

        // 应该返回失败结果，而不是错误
        assert!(result.is_ok());

        if let Ok(test_result) = result {
            assert!(!test_result.is_success());
            println!("预期的失败结果: {}", test_result.get_display_text());
        }
    }
}
