use std::sync::{Arc, Mutex};
use uuid::Uuid;
use log;


use crate::db_storage::DbStorageManager;
use crate::mcp_server::McpServerRecord;

/// MCP同步服务，负责将绿灯状态的MCP服务器同步到数据库
pub struct McpSyncService {
    storage: Arc<Mutex<DbStorageManager>>,
}

impl McpSyncService {
    /// 创建新的MCP同步服务
    pub fn new(storage: Arc<Mutex<DbStorageManager>>) -> Self {
        Self { storage }
    }

    /// 同步MCP服务器到数据库
    /// 参数：
    /// - server_id: 服务器UUID
    /// - name: 服务器名称
    /// - description: 服务器描述
    /// - transport_type: 传输类型 ("Command" 或 "SSE")
    /// - transport_config: 传输配置的JSON字符串
    /// - directory: 服务器目录
    /// - capabilities: 服务器能力的JSON字符串
    /// - health_status: 健康状态 ("Red", "Yellow", "Green")
    pub fn sync_server(
        &self,
        server_id: Uuid,
        name: String,
        description: Option<String>,
        transport_type: String,
        transport_config: String,
        directory: String,
        capabilities: Option<String>,
        health_status: String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let storage = self.storage.lock()
            .map_err(|e| format!("Failed to lock storage: {}", e))?;

        // 检查服务器是否已存在
        let existing_server = storage.load_mcp_server(&server_id.to_string());

        let mut server_record = match existing_server {
            Ok(mut existing) => {
                // 更新现有服务器
                existing.name = name;
                existing.description = description;
                existing.transport_type = transport_type;
                existing.transport_config = transport_config;
                existing.directory = directory;
                existing.capabilities = capabilities;
                existing.health_status = health_status.clone();
                existing.updated_at = chrono::Utc::now();
                
                // 如果状态变为绿灯，更新测试时间和成功状态
                if health_status == "Green" {
                    existing.last_test_time = Some(chrono::Utc::now());
                    existing.last_test_success = true;
                }
                
                existing
            }
            Err(_) => {
                // 创建新服务器记录
                let mut new_server = McpServerRecord::new(
                    server_id,
                    name,
                    description,
                    transport_type,
                    transport_config,
                    directory,
                );
                new_server.capabilities = capabilities;
                new_server.health_status = health_status.clone();
                
                // 如果状态为绿灯，设置测试时间和成功状态
                if health_status == "Green" {
                    new_server.last_test_time = Some(chrono::Utc::now());
                    new_server.last_test_success = true;
                }
                
                new_server
            }
        };

        // 保存到数据库
        storage.save_mcp_server(&server_record)?;

        log::info!("✅ Synced MCP server '{}' ({}) to database with status: {}", 
                  server_record.name, server_id, health_status);

        Ok(())
    }

    /// 从数据库中移除MCP服务器
    pub fn remove_server(&self, server_id: Uuid) -> Result<(), Box<dyn std::error::Error>> {
        let storage = self.storage.lock()
            .map_err(|e| format!("Failed to lock storage: {}", e))?;

        storage.delete_mcp_server(&server_id.to_string())?;

        log::info!("🗑️ Removed MCP server {} from database", server_id);

        Ok(())
    }

    /// 获取所有绿灯状态的MCP服务器
    pub fn get_green_servers(&self) -> Result<Vec<McpServerRecord>, Box<dyn std::error::Error>> {
        let storage = self.storage.lock()
            .map_err(|e| format!("Failed to lock storage: {}", e))?;

        let servers = storage.list_green_mcp_servers()?;

        log::debug!("📋 Retrieved {} green MCP servers from database", servers.len());

        Ok(servers)
    }

    /// 获取所有MCP服务器
    pub fn get_all_servers(&self) -> Result<Vec<McpServerRecord>, Box<dyn std::error::Error>> {
        let storage = self.storage.lock()
            .map_err(|e| format!("Failed to lock storage: {}", e))?;

        let servers = storage.list_mcp_servers()?;

        log::debug!("📋 Retrieved {} MCP servers from database", servers.len());

        Ok(servers)
    }

    /// 批量同步多个MCP服务器
    pub fn batch_sync_servers(
        &self,
        servers: Vec<(Uuid, String, Option<String>, String, String, String, Option<String>, String)>
    ) -> Result<usize, Box<dyn std::error::Error>> {
        let mut synced_count = 0;

        for (server_id, name, description, transport_type, transport_config, directory, capabilities, health_status) in servers {
            match self.sync_server(
                server_id,
                name.clone(),
                description,
                transport_type,
                transport_config,
                directory,
                capabilities,
                health_status.clone(),
            ) {
                Ok(_) => {
                    synced_count += 1;
                    log::debug!("✅ Synced server '{}' with status: {}", name, health_status);
                }
                Err(e) => {
                    log::error!("❌ Failed to sync server '{}': {}", name, e);
                }
            }
        }

        log::info!("🔄 Batch sync completed: {}/{} servers synced successfully", 
                  synced_count, synced_count);

        Ok(synced_count)
    }

    /// 清理数据库中不再存在的MCP服务器
    pub fn cleanup_orphaned_servers(
        &self,
        active_server_ids: Vec<Uuid>
    ) -> Result<usize, Box<dyn std::error::Error>> {
        let storage = self.storage.lock()
            .map_err(|e| format!("Failed to lock storage: {}", e))?;

        let all_servers = storage.list_mcp_servers()?;
        let mut removed_count = 0;

        for server in all_servers {
            if let Ok(server_uuid) = server.get_uuid() {
                if !active_server_ids.contains(&server_uuid) {
                    // 这个服务器在数据库中但不在活跃列表中，删除它
                    if let Err(e) = storage.delete_mcp_server(&server.id) {
                        log::error!("❌ Failed to remove orphaned server '{}': {}", server.name, e);
                    } else {
                        log::info!("🧹 Removed orphaned MCP server '{}' from database", server.name);
                        removed_count += 1;
                    }
                }
            }
        }

        if removed_count > 0 {
            log::info!("🧹 Cleanup completed: removed {} orphaned servers", removed_count);
        }

        Ok(removed_count)
    }
}

/// MCP服务器同步事件
#[derive(Debug, Clone)]
pub enum McpSyncEvent {
    ServerSynced(Uuid, String),
    ServerRemoved(Uuid),
    BatchSyncCompleted(usize),
    CleanupCompleted(usize),
    SyncError(Uuid, String),
}
