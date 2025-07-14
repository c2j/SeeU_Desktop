use crate::encryption::{EncryptedContainer, PasswordEncryption};
use crate::remote_server::{AuthMethod, RemoteServer};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use uuid::Uuid;

/// 远程服务器配置文件结构
#[derive(Debug, Clone, Serialize, Deserialize)]
struct RemoteServerConfig {
    /// 配置文件版本
    version: String,
    /// 创建时间
    created_at: chrono::DateTime<chrono::Utc>,
    /// 最后修改时间
    last_modified: chrono::DateTime<chrono::Utc>,
    /// 服务器列表（加密存储敏感信息）
    servers: Vec<EncryptedRemoteServer>,
}

/// 加密存储的远程服务器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
struct EncryptedRemoteServer {
    /// 基本信息（明文）
    pub id: Uuid,
    pub name: String,
    pub host: String,
    pub port: u16,
    pub username: String,
    pub working_directory: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_connected: Option<chrono::DateTime<chrono::Utc>>,
    pub connection_count: u32,
    pub tags: Vec<String>,
    pub description: Option<String>,
    pub enabled: bool,
    
    /// 认证方式类型（明文）
    pub auth_type: String,
    
    /// 加密的敏感信息
    pub encrypted_data: EncryptedContainer,
}

/// 远程服务器管理器
#[derive(Debug)]
pub struct RemoteServerManager {
    /// 服务器列表
    servers: HashMap<Uuid, RemoteServer>,
    /// 配置文件路径
    config_path: PathBuf,
    /// 加密管理器
    encryption: PasswordEncryption,
    /// 是否有未保存的更改
    has_unsaved_changes: bool,
}

impl RemoteServerManager {
    /// 创建新的远程服务器管理器
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let config_dir = dirs::config_dir()
            .ok_or("无法获取配置目录")?
            .join("seeu_desktop");
        
        std::fs::create_dir_all(&config_dir)?;
        let config_path = config_dir.join("remote_servers.json");
        
        let encryption = PasswordEncryption::new()?;
        
        let mut manager = Self {
            servers: HashMap::new(),
            config_path,
            encryption,
            has_unsaved_changes: false,
        };
        
        // 尝试加载现有配置
        if let Err(e) = manager.load_from_file() {
            log::warn!("加载远程服务器配置失败: {}，将使用空配置", e);
        }
        
        Ok(manager)
    }

    /// 添加服务器
    pub fn add_server(&mut self, server: RemoteServer) -> Result<(), String> {
        server.validate()?;

        // 检查是否已存在相同的服务器
        for existing in self.servers.values() {
            if existing.host == server.host
                && existing.port == server.port
                && existing.username == server.username {
                return Err("已存在相同的服务器配置".to_string());
            }
        }

        let server_name = server.name.clone();
        self.servers.insert(server.id, server);
        self.has_unsaved_changes = true;
        log::info!("添加远程服务器: {} (总数: {})", server_name, self.servers.len());

        // 自动保存配置
        if let Err(e) = self.save_to_file() {
            log::error!("自动保存配置失败: {}", e);
        }

        Ok(())
    }

    /// 更新服务器
    pub fn update_server(&mut self, server: RemoteServer) -> Result<(), String> {
        server.validate()?;
        
        if !self.servers.contains_key(&server.id) {
            return Err("服务器不存在".to_string());
        }
        
        let server_id = server.id;
        self.servers.insert(server.id, server);
        self.has_unsaved_changes = true;
        log::info!("更新远程服务器: {}", server_id);
        Ok(())
    }

    /// 删除服务器
    pub fn remove_server(&mut self, id: Uuid) -> Result<(), String> {
        if self.servers.remove(&id).is_some() {
            self.has_unsaved_changes = true;
            log::info!("删除远程服务器: {}", id);
            Ok(())
        } else {
            Err("服务器不存在".to_string())
        }
    }

    /// 获取服务器
    pub fn get_server(&self, id: Uuid) -> Option<&RemoteServer> {
        self.servers.get(&id)
    }

    /// 获取可变服务器引用
    pub fn get_server_mut(&mut self, id: Uuid) -> Option<&mut RemoteServer> {
        if self.servers.contains_key(&id) {
            self.has_unsaved_changes = true;
        }
        self.servers.get_mut(&id)
    }

    /// 获取所有服务器列表
    pub fn list_servers(&self) -> Vec<&RemoteServer> {
        let mut servers: Vec<&RemoteServer> = self.servers.values().collect();
        servers.sort_by(|a, b| a.name.cmp(&b.name));
        servers
    }

    /// 搜索服务器
    pub fn search_servers(&self, query: &str) -> Vec<&RemoteServer> {
        let mut results: Vec<&RemoteServer> = self.servers
            .values()
            .filter(|server| server.matches_search(query))
            .collect();
        
        results.sort_by(|a, b| a.name.cmp(&b.name));
        results
    }

    /// 按标签过滤服务器
    pub fn filter_by_tag(&self, tag: &str) -> Vec<&RemoteServer> {
        let mut results: Vec<&RemoteServer> = self.servers
            .values()
            .filter(|server| server.tags.contains(&tag.to_string()))
            .collect();
        
        results.sort_by(|a, b| a.name.cmp(&b.name));
        results
    }

    /// 获取所有标签
    pub fn get_all_tags(&self) -> Vec<String> {
        let mut tags: Vec<String> = self.servers
            .values()
            .flat_map(|server| server.tags.iter())
            .cloned()
            .collect();
        
        tags.sort();
        tags.dedup();
        tags
    }

    /// 更新服务器连接统计
    pub fn update_connection_stats(&mut self, id: Uuid) -> Result<(), String> {
        if let Some(server) = self.servers.get_mut(&id) {
            server.update_connection_stats();
            self.has_unsaved_changes = true;
            Ok(())
        } else {
            Err("服务器不存在".to_string())
        }
    }

    /// 检查是否有未保存的更改
    pub fn has_unsaved_changes(&self) -> bool {
        self.has_unsaved_changes
    }

    /// 保存到文件
    pub fn save_to_file(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let encrypted_servers = self.encrypt_servers()?;
        
        let config = RemoteServerConfig {
            version: "1.0".to_string(),
            created_at: chrono::Utc::now(),
            last_modified: chrono::Utc::now(),
            servers: encrypted_servers,
        };
        
        let json_content = serde_json::to_string_pretty(&config)?;
        std::fs::write(&self.config_path, json_content)?;
        
        self.has_unsaved_changes = false;
        log::info!("远程服务器配置已保存到: {}", self.config_path.display());
        Ok(())
    }

    /// 从文件加载
    pub fn load_from_file(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if !self.config_path.exists() {
            log::info!("配置文件不存在，使用空配置");
            return Ok(());
        }
        
        let json_content = std::fs::read_to_string(&self.config_path)?;
        let config: RemoteServerConfig = serde_json::from_str(&json_content)?;
        
        self.servers = self.decrypt_servers(&config.servers)?;
        self.has_unsaved_changes = false;
        
        log::info!("从配置文件加载了 {} 个远程服务器", self.servers.len());
        Ok(())
    }

    /// 导出配置（不包含敏感信息）
    pub fn export_config(&self, include_sensitive: bool) -> Result<String, Box<dyn std::error::Error>> {
        if include_sensitive {
            // 导出完整配置（需要用户确认）
            let encrypted_servers = self.encrypt_servers()?;
            let config = RemoteServerConfig {
                version: "1.0".to_string(),
                created_at: chrono::Utc::now(),
                last_modified: chrono::Utc::now(),
                servers: encrypted_servers,
            };
            serde_json::to_string_pretty(&config).map_err(Into::into)
        } else {
            // 导出不包含敏感信息的配置
            let safe_servers: Vec<_> = self.servers.values().map(|server| {
                serde_json::json!({
                    "name": server.name,
                    "host": server.host,
                    "port": server.port,
                    "username": server.username,
                    "working_directory": server.working_directory,
                    "tags": server.tags,
                    "description": server.description,
                    "auth_type": server.get_auth_method_display(),
                })
            }).collect();
            
            let config = serde_json::json!({
                "version": "1.0",
                "export_type": "safe",
                "exported_at": chrono::Utc::now(),
                "servers": safe_servers
            });
            
            serde_json::to_string_pretty(&config).map_err(Into::into)
        }
    }

    /// 获取统计信息
    pub fn get_statistics(&self) -> HashMap<String, serde_json::Value> {
        let mut stats = HashMap::new();
        
        stats.insert("total_servers".to_string(), serde_json::Value::Number(self.servers.len().into()));
        stats.insert("enabled_servers".to_string(), 
            serde_json::Value::Number(self.servers.values().filter(|s| s.enabled).count().into()));
        
        let total_connections: u32 = self.servers.values().map(|s| s.connection_count).sum();
        stats.insert("total_connections".to_string(), serde_json::Value::Number(total_connections.into()));
        
        let auth_methods: HashMap<String, usize> = self.servers.values()
            .map(|s| s.get_auth_method_display())
            .fold(HashMap::new(), |mut acc, method| {
                *acc.entry(method.to_string()).or_insert(0) += 1;
                acc
            });
        
        stats.insert("auth_methods".to_string(), serde_json::to_value(auth_methods).unwrap_or_default());
        
        stats
    }

    /// 加密服务器列表
    fn encrypt_servers(&self) -> Result<Vec<EncryptedRemoteServer>, Box<dyn std::error::Error>> {
        let mut encrypted_servers = Vec::new();

        for server in self.servers.values() {
            let mut encrypted_data = EncryptedContainer::new();
            let auth_type;

            // 根据认证方式加密敏感信息
            match &server.auth_method {
                AuthMethod::Password(password) => {
                    auth_type = "password".to_string();
                    let encrypted_password = self.encryption.encrypt(password)?;
                    encrypted_data.add_encrypted_field("password".to_string(), encrypted_password);
                }
                AuthMethod::PrivateKey { key_path, passphrase } => {
                    auth_type = "private_key".to_string();
                    encrypted_data.add_plain_field(
                        "key_path".to_string(),
                        serde_json::Value::String(key_path.to_string_lossy().to_string())
                    );

                    if let Some(passphrase) = passphrase {
                        let encrypted_passphrase = self.encryption.encrypt(passphrase)?;
                        encrypted_data.add_encrypted_field("passphrase".to_string(), encrypted_passphrase);
                    }
                }
                AuthMethod::Agent => {
                    auth_type = "agent".to_string();
                    // SSH Agent认证无需存储敏感信息
                }
            }

            let encrypted_server = EncryptedRemoteServer {
                id: server.id,
                name: server.name.clone(),
                host: server.host.clone(),
                port: server.port,
                username: server.username.clone(),
                working_directory: server.working_directory.clone(),
                created_at: server.created_at,
                last_connected: server.last_connected,
                connection_count: server.connection_count,
                tags: server.tags.clone(),
                description: server.description.clone(),
                enabled: server.enabled,
                auth_type,
                encrypted_data,
            };

            encrypted_servers.push(encrypted_server);
        }

        Ok(encrypted_servers)
    }

    /// 解密服务器列表
    fn decrypt_servers(&self, encrypted_servers: &[EncryptedRemoteServer]) -> Result<HashMap<Uuid, RemoteServer>, Box<dyn std::error::Error>> {
        let mut servers = HashMap::new();

        for encrypted_server in encrypted_servers {
            let auth_method = match encrypted_server.auth_type.as_str() {
                "password" => {
                    if let Some(encrypted_password) = encrypted_server.encrypted_data.get_encrypted_field("password") {
                        let password = self.encryption.decrypt(encrypted_password)?;
                        AuthMethod::Password(password)
                    } else {
                        return Err("密码认证服务器缺少密码字段".into());
                    }
                }
                "private_key" => {
                    let key_path = if let Some(path_value) = encrypted_server.encrypted_data.get_plain_field("key_path") {
                        if let Some(path_str) = path_value.as_str() {
                            PathBuf::from(path_str)
                        } else {
                            return Err("私钥路径格式错误".into());
                        }
                    } else {
                        return Err("私钥认证服务器缺少私钥路径".into());
                    };

                    let passphrase = if let Some(encrypted_passphrase) = encrypted_server.encrypted_data.get_encrypted_field("passphrase") {
                        Some(self.encryption.decrypt(encrypted_passphrase)?)
                    } else {
                        None
                    };

                    AuthMethod::PrivateKey { key_path, passphrase }
                }
                "agent" => AuthMethod::Agent,
                _ => {
                    log::warn!("未知的认证类型: {}，跳过该服务器", encrypted_server.auth_type);
                    continue;
                }
            };

            let server = RemoteServer {
                id: encrypted_server.id,
                name: encrypted_server.name.clone(),
                host: encrypted_server.host.clone(),
                port: encrypted_server.port,
                username: encrypted_server.username.clone(),
                auth_method,
                password_auth_method: crate::remote_server::PasswordAuthMethod::default(),
                working_directory: encrypted_server.working_directory.clone(),
                created_at: encrypted_server.created_at,
                last_connected: encrypted_server.last_connected,
                connection_count: encrypted_server.connection_count,
                tags: encrypted_server.tags.clone(),
                description: encrypted_server.description.clone(),
                enabled: encrypted_server.enabled,
            };

            servers.insert(server.id, server);
        }

        Ok(servers)
    }
}

impl Default for RemoteServerManager {
    fn default() -> Self {
        Self::new().expect("Failed to create RemoteServerManager")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_server_manager_creation() {
        let manager = RemoteServerManager::new();
        assert!(manager.is_ok());
    }

    #[test]
    fn test_add_and_get_server() {
        let mut manager = RemoteServerManager::new().unwrap();

        let server = RemoteServer::new(
            "Test Server".to_string(),
            "192.168.1.100".to_string(),
            "user".to_string(),
            AuthMethod::Agent,
        );

        let server_id = server.id;
        assert!(manager.add_server(server).is_ok());

        let retrieved = manager.get_server(server_id);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name, "Test Server");
    }

    #[test]
    fn test_duplicate_server_rejection() {
        let mut manager = RemoteServerManager::new().unwrap();

        let server1 = RemoteServer::new(
            "Server 1".to_string(),
            "192.168.1.100".to_string(),
            "user".to_string(),
            AuthMethod::Agent,
        );

        let server2 = RemoteServer::new(
            "Server 2".to_string(),
            "192.168.1.100".to_string(),
            "user".to_string(),
            AuthMethod::Agent,
        );

        assert!(manager.add_server(server1).is_ok());
        assert!(manager.add_server(server2).is_err());
    }

    #[test]
    fn test_search_servers() {
        let mut manager = RemoteServerManager::new().unwrap();

        let server1 = RemoteServer::new(
            "Production Server".to_string(),
            "prod.example.com".to_string(),
            "admin".to_string(),
            AuthMethod::Agent,
        );

        let server2 = RemoteServer::new(
            "Development Server".to_string(),
            "dev.example.com".to_string(),
            "developer".to_string(),
            AuthMethod::Agent,
        );

        manager.add_server(server1).unwrap();
        manager.add_server(server2).unwrap();

        let results = manager.search_servers("prod");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "Production Server");

        let results = manager.search_servers("example");
        assert_eq!(results.len(), 2);
    }
}
