//! HelixDB进程管理器
//! 
//! 负责管理HelixDB进程的生命周期，包括启动、停止、监控和错误恢复

use std::process::{Child, Command, Stdio};
use std::path::PathBuf;
use std::time::{Duration, Instant};
use tokio::time::sleep;
use crate::{HelixDBConfig, HelixDBError, HelixDBStatus};

/// HelixDB进程管理器
#[derive(Debug)]
pub struct HelixDBManager {
    /// HelixDB进程句柄
    process: Option<Child>,
    /// 数据目录
    data_dir: PathBuf,
    /// 配置
    config: HelixDBConfig,
    /// 进程状态
    status: HelixDBStatus,
    /// 启动时间
    start_time: Option<Instant>,
    /// 健康检查客户端
    health_client: reqwest::Client,
    /// 重启尝试次数
    restart_attempts: u32,
}

impl HelixDBManager {
    /// 创建新的HelixDB管理器
    pub fn new(data_dir: PathBuf, config: HelixDBConfig) -> Self {
        Self {
            process: None,
            data_dir,
            config,
            status: HelixDBStatus::Stopped,
            start_time: None,
            health_client: reqwest::Client::builder()
                .timeout(Duration::from_secs(5))
                .build()
                .unwrap_or_default(),
            restart_attempts: 0,
        }
    }

    /// 启动HelixDB进程（异步后台启动）
    pub async fn start(&mut self) -> Result<(), HelixDBError> {
        if self.status == HelixDBStatus::Running {
            return Ok(());
        }

        self.status = HelixDBStatus::Starting;
        log::info!("🚀 启动HelixDB进程...");

        // 确保数据目录存在
        std::fs::create_dir_all(&self.data_dir)
            .map_err(|e| HelixDBError::IoError(format!("创建数据目录失败: {}", e)))?;

        // 检查HelixDB CLI是否可用
        self.check_helix_cli().await?;

        // 初始化HelixDB项目
        self.init_helix_project().await?;

        // 启动HelixDB进程
        let process = self.spawn_helix_process().await?;
        self.process = Some(process);
        self.start_time = Some(Instant::now());

        // 异步等待服务启动（不阻塞主线程）
        let manager_clone = self.clone_for_async();
        tokio::spawn(async move {
            if let Err(e) = manager_clone.wait_for_startup_async().await {
                log::error!("HelixDB启动失败: {}", e);
            }
        });

        log::info!("✅ HelixDB进程已启动，端口: {}", self.config.port);
        Ok(())
    }

    /// 停止HelixDB进程
    pub async fn stop(&mut self) -> Result<(), HelixDBError> {
        if self.status == HelixDBStatus::Stopped {
            return Ok(());
        }

        self.status = HelixDBStatus::Stopping;
        log::info!("🛑 停止HelixDB进程...");

        if let Some(mut process) = self.process.take() {
            // 尝试优雅停止
            if let Err(e) = process.kill() {
                log::warn!("强制停止HelixDB进程失败: {}", e);
            }

            // 等待进程退出
            match process.wait() {
                Ok(status) => {
                    log::info!("HelixDB进程已退出，状态: {:?}", status);
                }
                Err(e) => {
                    log::warn!("等待HelixDB进程退出失败: {}", e);
                }
            }
        }

        self.status = HelixDBStatus::Stopped;
        self.start_time = None;
        self.restart_attempts = 0;
        log::info!("✅ HelixDB进程已停止");

        Ok(())
    }

    /// 重启HelixDB进程
    pub async fn restart(&mut self) -> Result<(), HelixDBError> {
        log::info!("🔄 重启HelixDB进程...");
        self.stop().await?;
        sleep(Duration::from_secs(2)).await;
        self.start().await?;
        Ok(())
    }

    /// 检查HelixDB进程健康状态
    pub async fn health_check(&mut self) -> Result<bool, HelixDBError> {
        if self.status != HelixDBStatus::Running {
            return Ok(false);
        }

        let health_url = format!("http://localhost:{}/health", self.config.port);
        
        match self.health_client
            .get(&health_url)
            .send()
            .await
        {
            Ok(response) => {
                let is_healthy = response.status().is_success();
                if !is_healthy {
                    log::warn!("HelixDB健康检查失败，状态码: {}", response.status());
                    self.status = HelixDBStatus::Error("健康检查失败".to_string());
                    
                    // 尝试自动重启
                    if self.config.auto_restart && self.restart_attempts < self.config.max_restart_attempts {
                        self.restart_attempts += 1;
                        log::info!("尝试自动重启HelixDB (第{}次)", self.restart_attempts);
                        if let Err(e) = self.restart().await {
                            log::error!("自动重启失败: {}", e);
                        }
                    }
                }
                Ok(is_healthy)
            }
            Err(e) => {
                log::warn!("HelixDB健康检查请求失败: {}", e);
                self.status = HelixDBStatus::Error(format!("健康检查请求失败: {}", e));
                Ok(false)
            }
        }
    }

    /// 获取进程状态
    pub fn status(&self) -> &HelixDBStatus {
        &self.status
    }

    /// 获取运行时间
    pub fn uptime(&self) -> Option<Duration> {
        self.start_time.map(|start| start.elapsed())
    }

    /// 获取端口号
    pub fn port(&self) -> u16 {
        self.config.port
    }

    /// 检查HelixDB CLI是否可用
    async fn check_helix_cli(&self) -> Result<(), HelixDBError> {
        let output = Command::new("helix")
            .arg("--version")
            .output()
            .map_err(|e| HelixDBError::CliNotFound(format!("HelixDB CLI未找到: {}. 请确保已安装HelixDB CLI工具", e)))?;

        if !output.status.success() {
            return Err(HelixDBError::CliNotFound("HelixDB CLI不可用".to_string()));
        }

        let version = String::from_utf8_lossy(&output.stdout);
        log::info!("检测到HelixDB CLI版本: {}", version.trim());
        Ok(())
    }

    /// 初始化HelixDB项目
    async fn init_helix_project(&self) -> Result<(), HelixDBError> {
        // 检查是否已经初始化
        let schema_file = self.data_dir.join("schema.hx");
        if schema_file.exists() {
            log::debug!("HelixDB项目已初始化");
            return Ok(());
        }

        log::info!("初始化HelixDB项目...");
        
        let output = Command::new("helix")
            .args(&["init", "--path", self.data_dir.to_str().unwrap()])
            .output()
            .map_err(|e| HelixDBError::InitError(format!("初始化失败: {}", e)))?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(HelixDBError::InitError(format!("初始化失败: {}", error)));
        }

        // 创建默认schema
        self.create_default_schema().await?;

        log::info!("✅ HelixDB项目初始化完成");
        Ok(())
    }

    /// 创建默认schema
    async fn create_default_schema(&self) -> Result<(), HelixDBError> {
        let schema_content = r#"
// 笔记节点定义
SCHEMA Note {
    id: String,
    title: String,
    content: String,
    embedding: Vector<1536>,
    notebook_id: String,
    created_at: DateTime,
    updated_at: DateTime
}

// 笔记本节点定义
SCHEMA Notebook {
    id: String,
    name: String,
    description: String,
    embedding: Vector<1536>
}

// 标签节点定义
SCHEMA Tag {
    id: String,
    name: String,
    embedding: Vector<1536>,
    usage_count: I64
}

// 添加笔记
QUERY addNote(id: String, title: String, content: String, embedding: Vector<1536>, notebook_id: String) =>
    note <- AddN<Note({
        id: id,
        title: title,
        content: content,
        embedding: embedding,
        notebook_id: notebook_id,
        created_at: NOW(),
        updated_at: NOW()
    })
    RETURN note

// 语义搜索笔记
QUERY searchNotes(query_embedding: Vector<1536>, limit: I64) =>
    notes <- N<Note>::VECTOR_SEARCH(embedding, query_embedding, limit)
    RETURN notes

// 查找相似笔记
QUERY findSimilarNotes(note_id: String, limit: I64) =>
    target_note <- N<Note::WHERE(_::{id}::EQ(note_id))
    similar_notes <- N<Note>::VECTOR_SEARCH(embedding, target_note.embedding, limit)
    RETURN similar_notes

// 删除笔记
QUERY deleteNote(note_id: String) =>
    note <- N<Note::WHERE(_::{id}::EQ(note_id))
    DELETE note
    RETURN "deleted"
"#;

        let schema_file = self.data_dir.join("schema.hx");
        std::fs::write(&schema_file, schema_content)
            .map_err(|e| HelixDBError::IoError(format!("写入schema文件失败: {}", e)))?;

        log::info!("✅ 默认schema文件已创建");
        Ok(())
    }

    /// 启动HelixDB进程
    async fn spawn_helix_process(&self) -> Result<Child, HelixDBError> {
        log::info!("启动HelixDB进程，数据目录: {:?}", self.data_dir);

        let process = Command::new("helix")
            .args(&["deploy", "--path", self.data_dir.to_str().unwrap()])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| HelixDBError::ProcessError(format!("启动进程失败: {}", e)))?;

        log::info!("HelixDB进程已启动，PID: {:?}", process.id());
        Ok(process)
    }

    /// 异步等待服务启动
    async fn wait_for_startup_async(&self) -> Result<(), HelixDBError> {
        let timeout = Duration::from_secs(self.config.startup_timeout);
        let start = Instant::now();

        log::info!("等待HelixDB服务启动...");

        while start.elapsed() < timeout {
            if self.check_service_ready().await? {
                log::info!("✅ HelixDB服务已就绪");
                return Ok(());
            }

            sleep(Duration::from_millis(500)).await;
        }

        Err(HelixDBError::StartupTimeout("HelixDB启动超时".to_string()))
    }

    /// 检查服务是否就绪
    async fn check_service_ready(&self) -> Result<bool, HelixDBError> {
        let health_url = format!("http://localhost:{}/health", self.config.port);
        
        match self.health_client
            .get(&health_url)
            .send()
            .await
        {
            Ok(response) => Ok(response.status().is_success()),
            Err(_) => Ok(false), // 连接失败说明服务还未就绪
        }
    }

    /// 为异步操作克隆必要的数据
    fn clone_for_async(&self) -> AsyncHelixDBManager {
        AsyncHelixDBManager {
            config: self.config.clone(),
            health_client: self.health_client.clone(),
        }
    }
}

/// 用于异步操作的轻量级管理器
#[derive(Clone)]
struct AsyncHelixDBManager {
    config: HelixDBConfig,
    health_client: reqwest::Client,
}

impl AsyncHelixDBManager {
    async fn wait_for_startup_async(&self) -> Result<(), HelixDBError> {
        let timeout = Duration::from_secs(self.config.startup_timeout);
        let start = Instant::now();

        while start.elapsed() < timeout {
            if self.check_service_ready().await? {
                return Ok(());
            }
            sleep(Duration::from_millis(500)).await;
        }

        Err(HelixDBError::StartupTimeout("HelixDB启动超时".to_string()))
    }

    async fn check_service_ready(&self) -> Result<bool, HelixDBError> {
        let health_url = format!("http://localhost:{}/health", self.config.port);
        
        match self.health_client.get(&health_url).send().await {
            Ok(response) => Ok(response.status().is_success()),
            Err(_) => Ok(false),
        }
    }
}

impl Drop for HelixDBManager {
    fn drop(&mut self) {
        if let Some(mut process) = self.process.take() {
            log::info!("应用退出，停止HelixDB进程...");
            let _ = process.kill();
        }
    }
}
