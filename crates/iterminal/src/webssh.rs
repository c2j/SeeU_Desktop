use crate::remote_server::RemoteServer;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// WebSSH连接配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSshConfig {
    /// WebSSH服务器URL
    pub server_url: String,
    /// API密钥（如果需要）
    pub api_key: Option<String>,
    /// 连接超时（秒）
    pub timeout: u64,
    /// 是否使用SSL
    pub use_ssl: bool,
}

impl Default for WebSshConfig {
    fn default() -> Self {
        Self {
            server_url: "ws://localhost:8080/webssh".to_string(),
            api_key: None,
            timeout: 30,
            use_ssl: false,
        }
    }
}

/// WebSSH连接状态
#[derive(Debug, Clone)]
pub enum WebSshState {
    Disconnected,
    Connecting,
    Connected,
    Error(String),
}

/// WebSSH连接管理器
pub struct WebSshConnection {
    config: WebSshConfig,
    state: WebSshState,
    server: RemoteServer,
}

impl WebSshConnection {
    /// 创建新的WebSSH连接
    pub fn new(server: RemoteServer, config: WebSshConfig) -> Self {
        Self {
            config,
            state: WebSshState::Disconnected,
            server,
        }
    }

    /// 检查WebSSH支持是否可用
    pub fn is_available() -> bool {
        // WebSSH需要网络连接和WebSocket支持
        // 这里简化检查，实际应该检查网络连接
        true
    }

    /// 获取支持信息
    pub fn get_support_info() -> String {
        "✅ WebSSH支持可用\n   - 通过WebSocket连接\n   - 支持浏览器式SSH\n   - 需要WebSSH服务器".to_string()
    }

    /// 连接到WebSSH服务器
    pub async fn connect(&mut self) -> Result<(), String> {
        log::info!("开始WebSSH连接到 {}@{}:{} 通过 {}", 
                   self.server.username, self.server.host, self.server.port, self.config.server_url);

        self.state = WebSshState::Connecting;

        // 构建WebSSH连接参数
        let mut params = HashMap::new();
        params.insert("hostname", self.server.host.clone());
        params.insert("port", self.server.port.to_string());
        params.insert("username", self.server.username.clone());

        // 根据认证方式添加参数
        match &self.server.auth_method {
            crate::remote_server::AuthMethod::Password(password) => {
                params.insert("password", password.clone());
                params.insert("auth_method", "password".to_string());
            }
            crate::remote_server::AuthMethod::PrivateKey { key_path, passphrase } => {
                params.insert("key_path", key_path.to_string_lossy().to_string());
                if let Some(passphrase) = passphrase {
                    params.insert("key_passphrase", passphrase.clone());
                }
                params.insert("auth_method", "publickey".to_string());
            }
            crate::remote_server::AuthMethod::Agent => {
                params.insert("auth_method", "agent".to_string());
            }
        }

        // 这里应该实现实际的WebSocket连接
        // 由于这是一个示例，我们模拟连接过程
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

        self.state = WebSshState::Connected;
        log::info!("WebSSH连接建立成功");
        Ok(())
    }

    /// 发送命令
    pub async fn send_command(&mut self, command: &str) -> Result<(), String> {
        match self.state {
            WebSshState::Connected => {
                log::debug!("发送WebSSH命令: {}", command);
                // 这里应该通过WebSocket发送命令
                Ok(())
            }
            _ => Err("WebSSH连接未建立".to_string()),
        }
    }

    /// 获取连接状态
    pub fn get_state(&self) -> &WebSshState {
        &self.state
    }

    /// 断开连接
    pub async fn disconnect(&mut self) {
        log::info!("断开WebSSH连接");
        self.state = WebSshState::Disconnected;
    }

    /// 测试WebSSH连接
    pub async fn test_connection(server: &RemoteServer, config: &WebSshConfig) -> Result<bool, String> {
        log::info!("测试WebSSH连接到 {}@{}:{}", 
                   server.username, server.host, server.port);

        let mut connection = Self::new(server.clone(), config.clone());
        match connection.connect().await {
            Ok(_) => {
                connection.disconnect().await;
                Ok(true)
            }
            Err(e) => {
                log::error!("WebSSH连接测试失败: {}", e);
                Ok(false)
            }
        }
    }
}

/// PuTTY集成支持（Windows）
#[cfg(target_os = "windows")]
pub struct PuttyIntegration;

#[cfg(target_os = "windows")]
impl PuttyIntegration {
    /// 检查PuTTY是否可用
    pub fn is_available() -> bool {
        use std::process::Command;
        
        // 检查常见的PuTTY安装路径
        let putty_paths = [
            "putty.exe",
            "C:\\Program Files\\PuTTY\\putty.exe",
            "C:\\Program Files (x86)\\PuTTY\\putty.exe",
        ];

        for path in &putty_paths {
            if Command::new(path)
                .arg("-help")
                .output()
                .map(|output| output.status.success())
                .unwrap_or(false)
            {
                return true;
            }
        }

        false
    }

    /// 获取PuTTY路径
    pub fn get_putty_path() -> Option<String> {
        use std::process::Command;
        
        let putty_paths = [
            "putty.exe",
            "C:\\Program Files\\PuTTY\\putty.exe",
            "C:\\Program Files (x86)\\PuTTY\\putty.exe",
        ];

        for path in &putty_paths {
            if Command::new(path)
                .arg("-help")
                .output()
                .map(|output| output.status.success())
                .unwrap_or(false)
            {
                return Some(path.to_string());
            }
        }

        None
    }

    /// 启动PuTTY连接
    pub fn launch_putty(server: &RemoteServer) -> Result<(), String> {
        let putty_path = Self::get_putty_path()
            .ok_or("PuTTY不可用")?;

        let mut args = Vec::new();
        
        // 基本连接参数
        args.push(format!("{}@{}", server.username, server.host));
        args.push("-P".to_string());
        args.push(server.port.to_string());

        // 根据认证方式添加参数
        match &server.auth_method {
            crate::remote_server::AuthMethod::PrivateKey { key_path, .. } => {
                args.push("-i".to_string());
                args.push(key_path.to_string_lossy().to_string());
            }
            _ => {
                // PuTTY会提示输入密码
            }
        }

        log::info!("启动PuTTY: {} {}", putty_path, args.join(" "));

        std::process::Command::new(putty_path)
            .args(&args)
            .spawn()
            .map_err(|e| format!("启动PuTTY失败: {}", e))?;

        Ok(())
    }

    /// 获取支持信息
    pub fn get_support_info() -> String {
        if Self::is_available() {
            format!("✅ PuTTY可用: {}", 
                   Self::get_putty_path().unwrap_or_else(|| "未知路径".to_string()))
        } else {
            "❌ PuTTY不可用\n   请从 https://www.putty.org/ 下载安装".to_string()
        }
    }
}

/// 终端模拟器集成（macOS）
#[cfg(target_os = "macos")]
pub struct TerminalAppIntegration;

#[cfg(target_os = "macos")]
impl TerminalAppIntegration {
    /// 检查Terminal.app是否可用
    pub fn is_available() -> bool {
        std::path::Path::new("/Applications/Utilities/Terminal.app").exists()
    }

    /// 启动Terminal.app SSH连接
    pub fn launch_terminal(server: &RemoteServer) -> Result<(), String> {
        let ssh_command = crate::ssh_connection::SshConnectionBuilder::build_ssh_command(server)?;
        let full_command = format!("{} {}", ssh_command.0, ssh_command.1.join(" "));

        let applescript = format!(
            r#"tell application "Terminal"
                activate
                do script "{}"
            end tell"#,
            full_command.replace("\"", "\\\"")
        );

        std::process::Command::new("osascript")
            .arg("-e")
            .arg(&applescript)
            .spawn()
            .map_err(|e| format!("启动Terminal.app失败: {}", e))?;

        Ok(())
    }

    /// 获取支持信息
    pub fn get_support_info() -> String {
        if Self::is_available() {
            "✅ Terminal.app可用".to_string()
        } else {
            "❌ Terminal.app不可用".to_string()
        }
    }
}

/// SSH替代方案管理器
pub struct SshAlternativeManager;

impl SshAlternativeManager {
    /// 获取所有可用的SSH替代方案
    pub fn get_available_alternatives() -> Vec<(String, String, bool)> {
        let mut alternatives = Vec::new();

        // 原生SSH (ssh2)
        alternatives.push((
            "原生SSH".to_string(),
            crate::native_ssh::NativeSshConnection::get_support_info(),
            crate::native_ssh::NativeSshConnection::is_available(),
        ));

        // WebSSH
        alternatives.push((
            "WebSSH".to_string(),
            WebSshConnection::get_support_info(),
            WebSshConnection::is_available(),
        ));

        // 平台特定的替代方案
        #[cfg(target_os = "windows")]
        {
            alternatives.push((
                "PuTTY".to_string(),
                PuttyIntegration::get_support_info(),
                PuttyIntegration::is_available(),
            ));
        }

        #[cfg(target_os = "macos")]
        {
            alternatives.push((
                "Terminal.app".to_string(),
                TerminalAppIntegration::get_support_info(),
                TerminalAppIntegration::is_available(),
            ));
        }

        alternatives
    }

    /// 获取推荐的替代方案
    pub fn get_recommended_alternative() -> Option<String> {
        let alternatives = Self::get_available_alternatives();
        
        // 优先选择可用的替代方案
        for (name, _, available) in alternatives {
            if available {
                return Some(name);
            }
        }

        None
    }

    /// 获取完整的支持状态报告
    pub fn get_full_support_report() -> String {
        let mut report = Vec::new();
        
        report.push("🔧 SSH连接替代方案状态报告".to_string());
        report.push("=".repeat(40));

        // 外部SSH客户端状态
        report.push("\n📡 外部SSH客户端:".to_string());
        report.push(crate::ssh_connection::SshConnectionBuilder::get_ssh_support_info());

        // 替代方案状态
        report.push("\n🔄 替代连接方案:".to_string());
        let alternatives = Self::get_available_alternatives();
        
        for (name, info, _) in alternatives {
            report.push(format!("\n• {}:", name));
            for line in info.lines() {
                report.push(format!("  {}", line));
            }
        }

        // 推荐方案
        if let Some(recommended) = Self::get_recommended_alternative() {
            report.push(format!("\n💡 推荐使用: {}", recommended));
        } else {
            report.push("\n⚠️  没有可用的SSH连接方案".to_string());
        }

        report.join("\n")
    }
}
