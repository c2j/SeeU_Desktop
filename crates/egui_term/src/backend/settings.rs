use std::path::PathBuf;
use std::collections::HashMap;

/// Get the default shell for the current platform
fn get_default_shell() -> String {
    #[cfg(windows)]
    {
        // On Windows, try PowerShell first, then fall back to cmd.exe
        if std::process::Command::new("powershell").arg("-Command").arg("echo test").output().is_ok() {
            "powershell".to_string()
        } else {
            "cmd.exe".to_string()
        }
    }
    #[cfg(unix)]
    {
        // On Unix-like systems, use SHELL environment variable or fall back to /bin/bash
        std::env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string())
    }
}

/// SSH认证方式
#[derive(Debug, Clone)]
pub enum SshAuthMethod {
    Password(String),
    PrivateKey { key_path: PathBuf, passphrase: Option<String> },
    Agent,
}

/// SSH连接配置
#[derive(Debug, Clone)]
pub struct SshConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub auth_method: SshAuthMethod,
    pub remote_working_directory: Option<String>,
    pub connection_options: HashMap<String, String>,
}

/// 终端后端设置
#[derive(Debug, Clone)]
pub struct BackendSettings {
    pub shell: String,
    pub args: Vec<String>,
    pub working_directory: Option<PathBuf>,
    pub ssh_config: Option<SshConfig>,
    pub env_vars: HashMap<String, String>,
}

impl Default for BackendSettings {
    fn default() -> Self {
        Self {
            shell: get_default_shell(),
            args: vec![],
            working_directory: None,
            ssh_config: None,
            env_vars: HashMap::new(),
        }
    }
}

impl BackendSettings {
    /// 创建SSH连接的设置
    pub fn for_ssh(ssh_config: SshConfig) -> Self {
        let (shell, args) = Self::build_ssh_command(&ssh_config);

        Self {
            shell,
            args,
            working_directory: None,
            ssh_config: Some(ssh_config.clone()),
            env_vars: ssh_config.connection_options.clone(),
        }
    }

    /// 构建SSH命令
    fn build_ssh_command(ssh_config: &SshConfig) -> (String, Vec<String>) {
        let mut args = vec![
            "-o".to_string(),
            "ServerAliveInterval=60".to_string(),
            "-o".to_string(),
            "ServerAliveCountMax=3".to_string(),
            "-o".to_string(),
            "ConnectTimeout=10".to_string(),
        ];

        // 添加端口
        if ssh_config.port != 22 {
            args.extend_from_slice(&["-p".to_string(), ssh_config.port.to_string()]);
        }

        // 添加认证配置
        match &ssh_config.auth_method {
            SshAuthMethod::PrivateKey { key_path, .. } => {
                args.extend_from_slice(&["-i".to_string(), key_path.to_string_lossy().to_string()]);
                args.extend_from_slice(&[
                    "-o".to_string(),
                    "PreferredAuthentications=publickey".to_string(),
                ]);
            }
            SshAuthMethod::Agent => {
                args.extend_from_slice(&[
                    "-o".to_string(),
                    "PreferredAuthentications=publickey".to_string(),
                    "-o".to_string(),
                    "IdentitiesOnly=no".to_string(),
                ]);
            }
            SshAuthMethod::Password(_) => {
                args.extend_from_slice(&[
                    "-o".to_string(),
                    "PreferredAuthentications=password".to_string(),
                ]);
            }
        }

        // 添加连接目标
        let connection_string = format!("{}@{}", ssh_config.username, ssh_config.host);
        args.push(connection_string);

        // 添加远程命令
        if let Some(ref dir) = ssh_config.remote_working_directory {
            args.extend_from_slice(&[
                "-t".to_string(),
                format!("cd '{}' && exec $SHELL -l", dir.replace('\'', "'\"'\"'"))
            ]);
        } else {
            args.extend_from_slice(&["-t".to_string(), "exec $SHELL -l".to_string()]);
        }

        ("ssh".to_string(), args)
    }

    /// 检查是否为SSH连接
    pub fn is_ssh_connection(&self) -> bool {
        self.ssh_config.is_some()
    }

    /// 获取SSH配置
    pub fn get_ssh_config(&self) -> Option<&SshConfig> {
        self.ssh_config.as_ref()
    }

    /// 获取连接显示名称
    pub fn get_connection_display_name(&self) -> String {
        if let Some(ssh_config) = &self.ssh_config {
            if ssh_config.port == 22 {
                format!("{}@{}", ssh_config.username, ssh_config.host)
            } else {
                format!("{}@{}:{}", ssh_config.username, ssh_config.host, ssh_config.port)
            }
        } else {
            "本地终端".to_string()
        }
    }
}

impl SshConfig {
    /// 创建新的SSH配置
    pub fn new(host: String, username: String, auth_method: SshAuthMethod) -> Self {
        Self {
            host,
            port: 22,
            username,
            auth_method,
            remote_working_directory: None,
            connection_options: HashMap::new(),
        }
    }

    /// 设置端口
    pub fn with_port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    /// 设置远程工作目录
    pub fn with_remote_directory(mut self, directory: String) -> Self {
        self.remote_working_directory = Some(directory);
        self
    }

    /// 添加连接选项
    pub fn with_option(mut self, key: String, value: String) -> Self {
        self.connection_options.insert(key, value);
        self
    }

    /// 获取连接字符串
    pub fn get_connection_string(&self) -> String {
        if self.port == 22 {
            format!("{}@{}", self.username, self.host)
        } else {
            format!("{}@{}:{}", self.username, self.host, self.port)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_shell_selection() {
        let shell = get_default_shell();

        #[cfg(windows)]
        {
            // On Windows, should be either powershell or cmd.exe
            assert!(shell == "powershell" || shell == "cmd.exe",
                   "Windows shell should be powershell or cmd.exe, got: {}", shell);
        }

        #[cfg(unix)]
        {
            // On Unix, should be either from SHELL env var or /bin/bash
            assert!(shell.contains("sh") || shell.contains("bash") || shell.contains("zsh"),
                   "Unix shell should contain sh, bash, or zsh, got: {}", shell);
        }
    }

    #[test]
    fn test_backend_settings_default() {
        let settings = BackendSettings::default();

        // Should be a valid shell for the current platform
        #[cfg(windows)]
        {
            assert!(settings.shell == "powershell" || settings.shell == "cmd.exe",
                   "Windows default shell should be powershell or cmd.exe, got: {}", settings.shell);
        }

        #[cfg(unix)]
        {
            // On Unix, should be a valid shell path (either from SHELL env var or /bin/bash fallback)
            assert!(settings.shell.contains("sh") || settings.shell.contains("bash") || settings.shell.contains("zsh"),
                   "Unix default shell should contain sh, bash, or zsh, got: {}", settings.shell);

            // Should be an absolute path on Unix systems
            assert!(settings.shell.starts_with('/'),
                   "Unix shell should be an absolute path, got: {}", settings.shell);
        }
    }
}
