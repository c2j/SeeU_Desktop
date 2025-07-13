use crate::remote_server::{AuthMethod, RemoteServer};
use ssh2::{Session, Channel};
use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

/// 原生SSH连接状态
#[derive(Debug, Clone)]
pub enum NativeSshState {
    Disconnected,
    Connecting,
    Connected,
    Error(String),
}

/// 原生SSH连接管理器
pub struct NativeSshConnection {
    session: Option<Session>,
    channel: Option<Channel>,
    state: Arc<Mutex<NativeSshState>>,
    server: RemoteServer,
}

impl NativeSshConnection {
    /// 创建新的原生SSH连接
    pub fn new(server: RemoteServer) -> Self {
        Self {
            session: None,
            channel: None,
            state: Arc::new(Mutex::new(NativeSshState::Disconnected)),
            server,
        }
    }

    /// 检查原生SSH支持是否可用
    pub fn is_available() -> bool {
        // ssh2 crate总是可用的，因为它是纯Rust实现
        true
    }

    /// 获取支持信息
    pub fn get_support_info() -> String {
        format!(
            "✅ 原生SSH支持可用 (ssh2 crate v{})\n   - 不依赖外部SSH客户端\n   - 纯Rust实现\n   - 支持所有平台",
            env!("CARGO_PKG_VERSION")
        )
    }

    /// 连接到远程服务器
    pub fn connect(&mut self) -> Result<(), String> {
        log::info!("开始原生SSH连接到 {}@{}:{}", 
                   self.server.username, self.server.host, self.server.port);

        // 更新状态为连接中
        {
            let mut state = self.state.lock().unwrap();
            *state = NativeSshState::Connecting;
        }

        // 建立TCP连接
        let tcp = TcpStream::connect(format!("{}:{}", self.server.host, self.server.port))
            .map_err(|e| format!("TCP连接失败: {}", e))?;

        // 创建SSH会话
        let mut session = Session::new()
            .map_err(|e| format!("创建SSH会话失败: {}", e))?;

        session.set_tcp_stream(tcp);
        session.handshake()
            .map_err(|e| format!("SSH握手失败: {}", e))?;

        // 根据认证方式进行认证
        match &self.server.auth_method {
            AuthMethod::Password(password) => {
                log::info!("使用密码认证");
                session.userauth_password(&self.server.username, password)
                    .map_err(|e| format!("密码认证失败: {}", e))?;
            }
            AuthMethod::PrivateKey { key_path, passphrase } => {
                log::info!("使用私钥认证: {}", key_path.display());
                
                if let Some(passphrase) = passphrase {
                    session.userauth_pubkey_file(
                        &self.server.username,
                        None,
                        key_path,
                        Some(passphrase)
                    ).map_err(|e| format!("私钥认证失败: {}", e))?;
                } else {
                    session.userauth_pubkey_file(
                        &self.server.username,
                        None,
                        key_path,
                        None
                    ).map_err(|e| format!("私钥认证失败: {}", e))?;
                }
            }
            AuthMethod::Agent => {
                log::info!("使用SSH Agent认证");
                session.userauth_agent(&self.server.username)
                    .map_err(|e| format!("SSH Agent认证失败: {}", e))?;
            }
        }

        // 验证认证是否成功
        if !session.authenticated() {
            return Err("SSH认证失败".to_string());
        }

        log::info!("SSH认证成功");

        // 创建shell通道
        let mut channel = session.channel_session()
            .map_err(|e| format!("创建SSH通道失败: {}", e))?;

        // 请求PTY
        channel.request_pty("xterm", None, None)
            .map_err(|e| format!("请求PTY失败: {}", e))?;

        // 启动shell
        channel.shell()
            .map_err(|e| format!("启动shell失败: {}", e))?;

        // 如果指定了工作目录，切换到该目录
        if let Some(ref dir) = self.server.working_directory {
            let cd_command = format!("cd '{}'\n", dir.replace('\'', "'\"'\"'"));
            channel.write_all(cd_command.as_bytes())
                .map_err(|e| format!("切换工作目录失败: {}", e))?;
        }

        // 保存会话和通道
        self.session = Some(session);
        self.channel = Some(channel);

        // 更新状态为已连接
        {
            let mut state = self.state.lock().unwrap();
            *state = NativeSshState::Connected;
        }

        log::info!("原生SSH连接建立成功");
        Ok(())
    }

    /// 发送命令到远程服务器
    pub fn send_command(&mut self, command: &str) -> Result<(), String> {
        if let Some(ref mut channel) = self.channel {
            let command_with_newline = format!("{}\n", command);
            channel.write_all(command_with_newline.as_bytes())
                .map_err(|e| format!("发送命令失败: {}", e))?;
            Ok(())
        } else {
            Err("SSH连接未建立".to_string())
        }
    }

    /// 读取输出
    pub fn read_output(&mut self) -> Result<String, String> {
        if let Some(ref mut channel) = self.channel {
            let mut buffer = [0; 4096];
            match channel.read(&mut buffer) {
                Ok(0) => Ok(String::new()), // 没有数据
                Ok(n) => {
                    let output = String::from_utf8_lossy(&buffer[..n]);
                    Ok(output.to_string())
                }
                Err(e) => Err(format!("读取输出失败: {}", e)),
            }
        } else {
            Err("SSH连接未建立".to_string())
        }
    }

    /// 获取连接状态
    pub fn get_state(&self) -> NativeSshState {
        self.state.lock().unwrap().clone()
    }

    /// 断开连接
    pub fn disconnect(&mut self) {
        log::info!("断开原生SSH连接");

        if let Some(mut channel) = self.channel.take() {
            let _ = channel.close();
        }

        if let Some(session) = self.session.take() {
            let _ = session.disconnect(None, "User disconnected", None);
        }

        // 更新状态
        {
            let mut state = self.state.lock().unwrap();
            *state = NativeSshState::Disconnected;
        }
    }

    /// 测试连接
    pub fn test_connection(server: &RemoteServer) -> Result<bool, String> {
        log::info!("测试原生SSH连接到 {}@{}:{}", 
                   server.username, server.host, server.port);

        let mut connection = Self::new(server.clone());
        match connection.connect() {
            Ok(_) => {
                // 发送测试命令
                if let Err(e) = connection.send_command("echo 'test_ok'") {
                    log::warn!("发送测试命令失败: {}", e);
                }

                // 等待一下让命令执行
                thread::sleep(Duration::from_millis(100));

                // 尝试读取输出
                match connection.read_output() {
                    Ok(output) => {
                        log::info!("测试命令输出: {}", output);
                    }
                    Err(e) => {
                        log::warn!("读取测试输出失败: {}", e);
                    }
                }

                connection.disconnect();
                Ok(true)
            }
            Err(e) => {
                log::error!("原生SSH连接测试失败: {}", e);
                Ok(false)
            }
        }
    }
}

impl Drop for NativeSshConnection {
    fn drop(&mut self) {
        self.disconnect();
    }
}

/// SSH连接方式枚举
#[derive(Debug, Clone)]
pub enum SshConnectionMethod {
    /// 使用外部SSH客户端（原有方式）
    External,
    /// 使用原生SSH库（ssh2 crate）
    Native,
    /// 自动选择最佳方式
    Auto,
}

impl Default for SshConnectionMethod {
    fn default() -> Self {
        Self::Auto
    }
}

/// SSH连接方式管理器
pub struct SshConnectionMethodManager;

impl SshConnectionMethodManager {
    /// 获取推荐的连接方式
    pub fn get_recommended_method() -> SshConnectionMethod {
        // 如果外部SSH客户端可用，优先使用外部客户端（更成熟）
        if crate::ssh_connection::SshConnectionBuilder::check_ssh_availability() {
            SshConnectionMethod::External
        } else {
            // 如果外部SSH不可用，使用原生实现
            SshConnectionMethod::Native
        }
    }

    /// 获取所有可用的连接方式
    pub fn get_available_methods() -> Vec<(SshConnectionMethod, String, bool)> {
        let mut methods = Vec::new();

        // 外部SSH客户端
        let external_available = crate::ssh_connection::SshConnectionBuilder::check_ssh_availability();
        methods.push((
            SshConnectionMethod::External,
            "外部SSH客户端".to_string(),
            external_available,
        ));

        // 原生SSH
        let native_available = NativeSshConnection::is_available();
        methods.push((
            SshConnectionMethod::Native,
            "原生SSH (ssh2)".to_string(),
            native_available,
        ));

        methods
    }

    /// 获取连接方式的详细信息
    pub fn get_method_info(method: &SshConnectionMethod) -> String {
        match method {
            SshConnectionMethod::External => {
                if crate::ssh_connection::SshConnectionBuilder::check_ssh_availability() {
                    format!("✅ 外部SSH客户端\n{}", 
                           crate::ssh_connection::SshConnectionBuilder::get_ssh_support_info())
                } else {
                    "❌ 外部SSH客户端不可用".to_string()
                }
            }
            SshConnectionMethod::Native => {
                format!("✅ 原生SSH支持\n{}", NativeSshConnection::get_support_info())
            }
            SshConnectionMethod::Auto => {
                let recommended = Self::get_recommended_method();
                format!("🔄 自动选择: {:?}\n{}", recommended, Self::get_method_info(&recommended))
            }
        }
    }
}
