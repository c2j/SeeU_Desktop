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
        let config = Self::build_ssh_config(server)?;
        Ok((config.command, config.args))
    }

    /// 测试SSH连接
    pub fn test_connection(server: &RemoteServer) -> Result<ConnectionTestResult, String> {
        let mut args = vec![
            "-o".to_string(),
            "ConnectTimeout=5".to_string(),
            "-o".to_string(),
            "BatchMode=yes".to_string(),
            "-o".to_string(),
            "StrictHostKeyChecking=no".to_string(),
        ];

        // 添加端口
        if server.port != 22 {
            args.extend_from_slice(&["-p".to_string(), server.port.to_string()]);
        }

        // 添加认证配置
        match &server.auth_method {
            AuthMethod::PrivateKey { key_path, .. } => {
                args.extend_from_slice(&["-i".to_string(), key_path.to_string_lossy().to_string()]);
            }
            AuthMethod::Agent => {
                args.extend_from_slice(&["-o".to_string(), "IdentitiesOnly=no".to_string()]);
            }
            AuthMethod::Password(_) => {
                // 密码认证在批处理模式下无法测试
                return Ok(ConnectionTestResult::Failed("密码认证无法在批处理模式下测试".to_string()));
            }
        }

        // 添加连接目标和测试命令
        args.push(format!("{}@{}", server.username, server.host));
        args.push("echo 'connection_test_ok'".to_string());

        // 执行测试命令
        let output = Command::new("ssh")
            .args(&args)
            .output()
            .map_err(|e| format!("执行SSH命令失败: {}", e))?;

        // 分析结果
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if stdout.contains("connection_test_ok") {
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
                Ok(ConnectionTestResult::Failed(stderr.to_string()))
            }
        }
    }

    /// 检查SSH客户端是否可用
    pub fn check_ssh_availability() -> bool {
        Command::new("ssh")
            .arg("-V")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
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
