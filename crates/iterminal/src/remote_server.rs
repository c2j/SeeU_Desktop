use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

/// 远程服务器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteServer {
    /// 唯一标识符
    pub id: Uuid,
    /// 服务器显示名称
    pub name: String,
    /// 主机地址（IP或域名）
    pub host: String,
    /// SSH端口，默认22
    pub port: u16,
    /// 用户名
    pub username: String,
    /// 认证方式
    pub auth_method: AuthMethod,
    /// 密码认证方法偏好（仅在使用密码认证时有效）
    #[serde(default)]
    pub password_auth_method: PasswordAuthMethod,
    /// 远程工作目录
    pub working_directory: Option<String>,
    /// 创建时间
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// 最后连接时间
    pub last_connected: Option<chrono::DateTime<chrono::Utc>>,
    /// 连接次数统计
    pub connection_count: u32,
    /// 标签分类
    pub tags: Vec<String>,
    /// 描述信息
    pub description: Option<String>,
    /// 是否启用
    pub enabled: bool,
}

/// SSH认证方式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthMethod {
    /// 密码认证（加密存储）
    Password(String),
    /// 私钥认证
    PrivateKey {
        /// 私钥文件路径
        key_path: PathBuf,
        /// 私钥密码（加密存储，可选）
        passphrase: Option<String>,
    },
    /// SSH Agent认证
    Agent,
}

/// 密码认证方法偏好
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PasswordAuthMethod {
    /// 自动选择最佳方法
    Auto,
    /// 使用sshpass工具
    Sshpass,
    /// 使用expect工具 (Unix)
    #[cfg(unix)]
    Expect,
    /// 使用PowerShell SSH (Windows)
    #[cfg(windows)]
    PowerShell,
    /// 使用PuTTY plink (Windows)
    #[cfg(windows)]
    Plink,
    /// 交互式输入（手动输入密码）
    Interactive,
}

impl Default for PasswordAuthMethod {
    fn default() -> Self {
        Self::Auto
    }
}

impl PasswordAuthMethod {
    /// 获取所有可用的密码认证方法
    pub fn available_methods() -> Vec<Self> {
        let mut methods = vec![
            Self::Auto,
            Self::Sshpass,
            Self::Interactive,
        ];

        #[cfg(unix)]
        methods.push(Self::Expect);

        #[cfg(windows)]
        {
            methods.push(Self::PowerShell);
            methods.push(Self::Plink);
        }

        methods
    }

    /// 获取方法的显示名称
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Auto => "自动选择",
            Self::Sshpass => "sshpass工具",
            #[cfg(unix)]
            Self::Expect => "expect工具",
            #[cfg(windows)]
            Self::PowerShell => "PowerShell SSH",
            #[cfg(windows)]
            Self::Plink => "PuTTY plink",
            Self::Interactive => "交互式输入",
        }
    }

    /// 获取方法的描述
    pub fn description(&self) -> &'static str {
        match self {
            Self::Auto => "自动选择最佳的可用方法",
            Self::Sshpass => "使用sshpass工具自动输入密码",
            #[cfg(unix)]
            Self::Expect => "使用expect脚本自动输入密码",
            #[cfg(windows)]
            Self::PowerShell => "使用PowerShell进行SSH连接",
            #[cfg(windows)]
            Self::Plink => "使用PuTTY的plink工具",
            Self::Interactive => "手动输入密码（最兼容）",
        }
    }

    /// 检查方法是否可用
    pub fn is_available(&self) -> bool {
        match self {
            Self::Auto => true,
            Self::Sshpass => crate::ssh_connection::SshConnectionBuilder::check_sshpass_availability(),
            #[cfg(unix)]
            Self::Expect => crate::ssh_connection::SshConnectionBuilder::check_expect_availability(),
            #[cfg(windows)]
            Self::PowerShell => crate::ssh_connection::SshConnectionBuilder::check_powershell_ssh_availability(),
            #[cfg(windows)]
            Self::Plink => crate::ssh_connection::SshConnectionBuilder::check_plink_availability(),
            Self::Interactive => true,
        }
    }
}

impl RemoteServer {
    /// 创建新的远程服务器配置
    pub fn new(name: String, host: String, username: String, auth_method: AuthMethod) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            host,
            port: 22,
            username,
            auth_method,
            password_auth_method: PasswordAuthMethod::default(),
            working_directory: None,
            created_at: chrono::Utc::now(),
            last_connected: None,
            connection_count: 0,
            tags: Vec::new(),
            description: None,
            enabled: true,
        }
    }

    /// 更新最后连接时间并增加连接计数
    pub fn update_connection_stats(&mut self) {
        self.last_connected = Some(chrono::Utc::now());
        self.connection_count += 1;
    }

    /// 获取连接字符串（用于显示）
    pub fn get_connection_string(&self) -> String {
        if self.port == 22 {
            format!("{}@{}", self.username, self.host)
        } else {
            format!("{}@{}:{}", self.username, self.host, self.port)
        }
    }

    /// 获取显示名称（如果没有自定义名称则使用连接字符串）
    pub fn get_display_name(&self) -> &str {
        if self.name.trim().is_empty() {
            &self.host
        } else {
            &self.name
        }
    }

    /// 验证服务器配置是否有效
    pub fn validate(&self) -> Result<(), String> {
        if self.host.trim().is_empty() {
            return Err("主机地址不能为空".to_string());
        }

        if self.username.trim().is_empty() {
            return Err("用户名不能为空".to_string());
        }

        if self.port == 0 || self.port > 65535 {
            return Err("端口号必须在1-65535之间".to_string());
        }

        match &self.auth_method {
            AuthMethod::Password(pwd) => {
                if pwd.trim().is_empty() {
                    return Err("密码不能为空".to_string());
                }
            }
            AuthMethod::PrivateKey { key_path, .. } => {
                if !key_path.exists() {
                    return Err(format!("私钥文件不存在: {}", key_path.display()));
                }
            }
            AuthMethod::Agent => {
                // SSH Agent认证无需额外验证
            }
        }

        Ok(())
    }

    /// 检查是否匹配搜索查询
    pub fn matches_search(&self, query: &str) -> bool {
        if query.trim().is_empty() {
            return true;
        }

        let query = query.to_lowercase();
        
        self.name.to_lowercase().contains(&query)
            || self.host.to_lowercase().contains(&query)
            || self.username.to_lowercase().contains(&query)
            || self.tags.iter().any(|tag| tag.to_lowercase().contains(&query))
            || self.description.as_ref().map_or(false, |desc| desc.to_lowercase().contains(&query))
    }

    /// 添加标签
    pub fn add_tag(&mut self, tag: String) {
        if !tag.trim().is_empty() && !self.tags.contains(&tag) {
            self.tags.push(tag);
        }
    }

    /// 移除标签
    pub fn remove_tag(&mut self, tag: &str) {
        self.tags.retain(|t| t != tag);
    }

    /// 获取认证方式的显示名称
    pub fn get_auth_method_display(&self) -> &'static str {
        match &self.auth_method {
            AuthMethod::Password(_) => "密码",
            AuthMethod::PrivateKey { .. } => "私钥",
            AuthMethod::Agent => "SSH Agent",
        }
    }
}

impl AuthMethod {
    /// 检查认证方式是否需要密码输入
    pub fn requires_password_input(&self) -> bool {
        matches!(self, AuthMethod::Password(_))
    }

    /// 检查认证方式是否需要私钥文件
    pub fn requires_private_key(&self) -> bool {
        matches!(self, AuthMethod::PrivateKey { .. })
    }

    /// 获取私钥路径（如果是私钥认证）
    pub fn get_private_key_path(&self) -> Option<&PathBuf> {
        match self {
            AuthMethod::PrivateKey { key_path, .. } => Some(key_path),
            _ => None,
        }
    }

    /// 检查是否有私钥密码
    pub fn has_passphrase(&self) -> bool {
        match self {
            AuthMethod::PrivateKey { passphrase, .. } => passphrase.is_some(),
            _ => false,
        }
    }
}

impl Default for RemoteServer {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            name: String::new(),
            host: String::new(),
            port: 22,
            username: String::new(),
            auth_method: AuthMethod::Agent,
            password_auth_method: PasswordAuthMethod::default(),
            working_directory: None,
            created_at: chrono::Utc::now(),
            last_connected: None,
            connection_count: 0,
            tags: Vec::new(),
            description: None,
            enabled: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_remote_server_creation() {
        let server = RemoteServer::new(
            "Test Server".to_string(),
            "192.168.1.100".to_string(),
            "user".to_string(),
            AuthMethod::Agent,
        );

        assert_eq!(server.name, "Test Server");
        assert_eq!(server.host, "192.168.1.100");
        assert_eq!(server.username, "user");
        assert_eq!(server.port, 22);
        assert!(server.enabled);
        assert_eq!(server.connection_count, 0);
    }

    #[test]
    fn test_connection_string() {
        let mut server = RemoteServer::default();
        server.username = "testuser".to_string();
        server.host = "example.com".to_string();

        assert_eq!(server.get_connection_string(), "testuser@example.com");

        server.port = 2222;
        assert_eq!(server.get_connection_string(), "testuser@example.com:2222");
    }

    #[test]
    fn test_search_matching() {
        let mut server = RemoteServer::default();
        server.name = "Production Server".to_string();
        server.host = "prod.example.com".to_string();
        server.username = "admin".to_string();
        server.tags = vec!["production".to_string(), "web".to_string()];

        assert!(server.matches_search("prod"));
        assert!(server.matches_search("Production"));
        assert!(server.matches_search("admin"));
        assert!(server.matches_search("web"));
        assert!(!server.matches_search("test"));
    }

    #[test]
    fn test_validation() {
        let mut server = RemoteServer::default();
        
        // 空主机地址应该失败
        assert!(server.validate().is_err());
        
        server.host = "example.com".to_string();
        // 空用户名应该失败
        assert!(server.validate().is_err());
        
        server.username = "user".to_string();
        // 有效配置应该成功
        assert!(server.validate().is_ok());
        
        // 无效端口应该失败
        server.port = 0;
        assert!(server.validate().is_err());

        // 注意：u16的最大值是65535，所以我们不能直接设置70000
        // 这里我们测试端口0的情况就足够了，因为端口范围已经由类型系统保证
    }
}
