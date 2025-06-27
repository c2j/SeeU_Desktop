// 语义搜索模块 - 集成到现有架构

use std::sync::Arc;
use tokio::sync::RwLock;
use std::path::PathBuf;
use dirs;

/// 语义搜索模块状态
pub struct SemanticSearchState {
    /// 嵌入式HelixDB服务
    helix_service: Option<Arc<EmbeddedHelixService>>,
    /// 混合搜索引擎
    hybrid_engine: Option<Arc<HybridSearchEngine>>,
    /// 配置
    config: SemanticSearchConfig,
    /// 初始化状态
    initialized: bool,
    /// 错误信息
    last_error: Option<String>,
    /// 搜索结果
    search_results: Vec<HybridSearchResult>,
    /// 搜索统计
    search_stats: SearchStatistics,
    /// 是否正在搜索
    is_searching: bool,
}

impl Default for SemanticSearchState {
    fn default() -> Self {
        Self {
            helix_service: None,
            hybrid_engine: None,
            config: SemanticSearchConfig::default(),
            initialized: false,
            last_error: None,
            search_results: vec![],
            search_stats: SearchStatistics {
                total_indexed_notes: 0,
                semantic_index_size: 0,
                keyword_index_size: 0,
                last_index_update: None,
            },
            is_searching: false,
        }
    }
}

impl SemanticSearchState {
    /// 初始化语义搜索服务
    pub async fn initialize(&mut self) -> Result<(), String> {
        if self.initialized {
            return Ok(());
        }

        log::info!("🔧 初始化语义搜索模块...");

        // 检查配置
        if !self.config.enabled {
            log::info!("语义搜索功能已禁用");
            return Ok(());
        }

        // 设置数据目录
        let data_dir = self.get_data_directory()?;

        // 创建向量化服务
        let embedding_service = self.create_embedding_service()?;

        // 创建HelixDB服务
        let helix_service = Arc::new(
            EmbeddedHelixService::new(data_dir, self.config.clone(), embedding_service)
                .await
                .map_err(|e| format!("创建HelixDB服务失败: {}", e))?
        );

        // 初始化HelixDB服务
        helix_service
            .initialize()
            .await
            .map_err(|e| format!("初始化HelixDB服务失败: {}", e))?;

        // 创建关键词搜索适配器（集成现有Tantivy）
        let keyword_service = self.create_keyword_service_adapter()?;

        // 创建混合搜索引擎
        let hybrid_engine = Arc::new(HybridSearchEngine::new(
            helix_service.clone(),
            keyword_service,
            self.config.search_weights.clone(),
        ));

        self.helix_service = Some(helix_service);
        self.hybrid_engine = Some(hybrid_engine);
        self.initialized = true;
        self.last_error = None;

        log::info!("✅ 语义搜索模块初始化完成");
        Ok(())
    }

    /// 关闭语义搜索服务
    pub async fn shutdown(&mut self) -> Result<(), String> {
        if !self.initialized {
            return Ok(());
        }

        log::info!("🛑 关闭语义搜索模块...");

        if let Some(helix_service) = &self.helix_service {
            helix_service
                .shutdown()
                .await
                .map_err(|e| format!("关闭HelixDB服务失败: {}", e))?;
        }

        self.helix_service = None;
        self.hybrid_engine = None;
        self.initialized = false;

        log::info!("✅ 语义搜索模块已关闭");
        Ok(())
    }

    /// 执行语义搜索
    pub async fn semantic_search(&mut self, query: &str, limit: usize) -> Result<(), String> {
        if !self.initialized {
            return Err("语义搜索服务未初始化".to_string());
        }

        let hybrid_engine = self.hybrid_engine
            .as_ref()
            .ok_or("混合搜索引擎不可用")?;

        self.is_searching = true;
        self.last_error = None;

        log::info!("🔍 执行语义搜索: {}", query);

        match hybrid_engine.hybrid_search(query, limit, None).await {
            Ok(results) => {
                self.search_results = results;
                self.is_searching = false;
                log::info!("✅ 语义搜索完成，找到 {} 个结果", self.search_results.len());
                Ok(())
            }
            Err(e) => {
                self.is_searching = false;
                let error_msg = format!("语义搜索失败: {}", e);
                self.last_error = Some(error_msg.clone());
                log::error!("{}", error_msg);
                Err(error_msg)
            }
        }
    }

    /// 索引笔记
    pub async fn index_note(&mut self, note: &Note) -> Result<(), String> {
        if !self.initialized {
            return Ok(()); // 如果未初始化，静默跳过
        }

        let helix_service = self.helix_service
            .as_ref()
            .ok_or("HelixDB服务不可用")?;

        helix_service
            .index_note(note)
            .await
            .map_err(|e| format!("索引笔记失败: {}", e))?;

        log::debug!("✅ 笔记已索引: {}", note.title);
        Ok(())
    }

    /// 批量索引笔记
    pub async fn index_notes(&mut self, notes: &[Note]) -> Result<(), String> {
        if !self.initialized {
            return Ok(());
        }

        let helix_service = self.helix_service
            .as_ref()
            .ok_or("HelixDB服务不可用")?;

        helix_service
            .index_notes(notes)
            .await
            .map_err(|e| format!("批量索引笔记失败: {}", e))?;

        log::info!("✅ 批量索引完成: {} 条笔记", notes.len());
        Ok(())
    }

    /// 更新配置
    pub async fn update_config(&mut self, new_config: SemanticSearchConfig) -> Result<(), String> {
        let need_restart = self.config.helix_config != new_config.helix_config ||
                          self.config.embedding_config != new_config.embedding_config;

        self.config = new_config;

        if need_restart && self.initialized {
            log::info!("配置已更改，重启语义搜索服务...");
            self.shutdown().await?;
            self.initialize().await?;
        }

        Ok(())
    }

    /// 获取服务状态
    pub async fn get_status(&self) -> ServiceStatus {
        if !self.initialized {
            return ServiceStatus::Stopped;
        }

        if let Some(helix_service) = &self.helix_service {
            if helix_service.is_healthy().await {
                ServiceStatus::Running
            } else {
                ServiceStatus::Error
            }
        } else {
            ServiceStatus::Stopped
        }
    }

    /// 获取数据目录
    fn get_data_directory(&self) -> Result<PathBuf, String> {
        let app_data_dir = dirs::data_dir()
            .ok_or("无法获取应用数据目录")?
            .join("SeeU_Desktop")
            .join("semantic_search");

        std::fs::create_dir_all(&app_data_dir)
            .map_err(|e| format!("创建数据目录失败: {}", e))?;

        Ok(app_data_dir)
    }

    /// 创建向量化服务
    fn create_embedding_service(&self) -> Result<Arc<dyn EmbeddingService + Send + Sync>, String> {
        EmbeddingServiceFactory::create_service(&self.config.embedding_config)
            .map_err(|e| format!("创建向量化服务失败: {}", e))
    }

    /// 创建关键词搜索适配器
    fn create_keyword_service_adapter(&self) -> Result<Arc<dyn KeywordSearchService + Send + Sync>, String> {
        // TODO: 实现Tantivy适配器
        // 这里需要将现有的isearch模块包装成KeywordSearchService接口
        Err("关键词搜索适配器尚未实现".to_string())
    }

    // Getter方法
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }

    pub fn is_searching(&self) -> bool {
        self.is_searching
    }

    pub fn get_search_results(&self) -> &[HybridSearchResult] {
        &self.search_results
    }

    pub fn get_last_error(&self) -> Option<&String> {
        self.last_error.as_ref()
    }

    pub fn get_config(&self) -> &SemanticSearchConfig {
        &self.config
    }

    pub fn get_search_stats(&self) -> &SearchStatistics {
        &self.search_stats
    }
}

/// 服务状态
#[derive(Debug, Clone, PartialEq)]
pub enum ServiceStatus {
    Stopped,
    Starting,
    Running,
    Error,
}

impl Default for SemanticSearchConfig {
    fn default() -> Self {
        Self {
            enabled: false, // 默认禁用，需要用户手动启用
            helix_config: HelixDBConfig {
                database_path: "".to_string(), // 将在初始化时设置
                connection_timeout: 6969,
                query_timeout: 30,
            },
            embedding_config: EmbeddingConfig::default(),
            search_weights: SearchWeights::default(),
        }
    }
}

// 重新导出类型
pub use crate::{
    EmbeddedHelixService, HybridSearchEngine, HybridSearchResult,
    SemanticSearchConfig, HelixDBConfig, EmbeddingConfig, SearchWeights,
    EmbeddingService, EmbeddingServiceFactory, KeywordSearchService,
    Note, SearchStatistics,
};
