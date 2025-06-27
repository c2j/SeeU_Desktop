//! 语义搜索模块
//! 
//! 提供基于HelixDB的语义搜索功能，包括：
//! - 嵌入式HelixDB进程管理
//! - 向量化服务
//! - 混合搜索引擎
//! - 与现有搜索系统的集成

pub mod config;
pub mod embedding;
pub mod helix_manager;
pub mod helix_service;
pub mod hybrid_search;
pub mod types;
pub mod errors;

// 重新导出主要类型
pub use config::*;
pub use embedding::*;
pub use helix_manager::*;
pub use helix_service::*;
pub use types::*;
pub use errors::*;

// 重新导出混合搜索，但避免冲突
pub use hybrid_search::{
    HybridSearchEngine, KeywordSearchService, KeywordSearchResult,
    TantivyAdapter, SearchSuggestion as HybridSearchSuggestion,
    SuggestionType as HybridSuggestionType,
    SearchStatistics as HybridSearchStatistics,
};

/// 语义搜索模块的主要入口点
pub struct SemanticSearchModule {
    state: SemanticSearchState,
}

impl SemanticSearchModule {
    /// 创建新的语义搜索模块实例
    pub fn new() -> Self {
        Self {
            state: SemanticSearchState::default(),
        }
    }

    /// 获取可变状态引用
    pub fn state_mut(&mut self) -> &mut SemanticSearchState {
        &mut self.state
    }

    /// 获取状态引用
    pub fn state(&self) -> &SemanticSearchState {
        &self.state
    }
}

impl Default for SemanticSearchModule {
    fn default() -> Self {
        Self::new()
    }
}

/// 语义搜索模块状态
pub struct SemanticSearchState {
    /// 嵌入式HelixDB服务
    helix_service: Option<std::sync::Arc<EmbeddedHelixService>>,
    /// 混合搜索引擎
    hybrid_engine: Option<std::sync::Arc<HybridSearchEngine>>,
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
            search_stats: SearchStatistics::default(),
            is_searching: false,
        }
    }
}

impl SemanticSearchState {
    /// 异步初始化语义搜索服务（与主应用一同启动）
    pub async fn initialize(&mut self) -> Result<(), SemanticSearchError> {
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
        let helix_service = std::sync::Arc::new(
            EmbeddedHelixService::new(data_dir, self.config.clone(), embedding_service)
                .await?
        );

        // 异步启动HelixDB服务（后台启动，不阻塞主应用）
        let helix_service_clone = helix_service.clone();
        tokio::spawn(async move {
            if let Err(e) = helix_service_clone.initialize().await {
                log::error!("HelixDB后台初始化失败: {}", e);
            } else {
                log::info!("✅ HelixDB后台服务启动成功");
            }
        });

        // 创建混合搜索引擎（暂时不依赖HelixDB完全启动）
        let hybrid_engine = std::sync::Arc::new(HybridSearchEngine::new(
            helix_service.clone(),
            self.config.search_weights.clone(),
        ));

        self.helix_service = Some(helix_service);
        self.hybrid_engine = Some(hybrid_engine);
        self.initialized = true;
        self.last_error = None;

        log::info!("✅ 语义搜索模块初始化完成（HelixDB后台启动中）");
        Ok(())
    }

    /// 关闭语义搜索服务（与主应用一同退出）
    pub async fn shutdown(&mut self) -> Result<(), SemanticSearchError> {
        if !self.initialized {
            return Ok(());
        }

        log::info!("🛑 关闭语义搜索模块...");

        if let Some(helix_service) = &self.helix_service {
            helix_service.shutdown().await?;
        }

        self.helix_service = None;
        self.hybrid_engine = None;
        self.initialized = false;

        log::info!("✅ 语义搜索模块已关闭");
        Ok(())
    }

    /// 检查服务是否就绪
    pub async fn is_ready(&self) -> bool {
        if !self.initialized {
            return false;
        }

        if let Some(helix_service) = &self.helix_service {
            helix_service.is_healthy().await
        } else {
            false
        }
    }

    /// 执行混合搜索
    pub async fn search(&mut self, query: &str, limit: usize) -> Result<(), SemanticSearchError> {
        if !self.initialized {
            return Err(SemanticSearchError::NotInitialized);
        }

        let hybrid_engine = self.hybrid_engine
            .as_ref()
            .ok_or(SemanticSearchError::ServiceUnavailable)?;

        self.is_searching = true;
        self.last_error = None;

        log::info!("🔍 执行混合搜索: {}", query);

        match hybrid_engine.hybrid_search(query, limit, None).await {
            Ok(results) => {
                self.search_results = results;
                self.is_searching = false;
                log::info!("✅ 混合搜索完成，找到 {} 个结果", self.search_results.len());
                Ok(())
            }
            Err(e) => {
                self.is_searching = false;
                let error_msg = format!("混合搜索失败: {}", e);
                self.last_error = Some(error_msg.clone());
                log::error!("{}", error_msg);
                Err(SemanticSearchError::SearchError(error_msg))
            }
        }
    }

    /// 索引笔记
    pub async fn index_note(&mut self, note: &Note) -> Result<(), SemanticSearchError> {
        if !self.initialized {
            return Ok(()); // 如果未初始化，静默跳过
        }

        let helix_service = self.helix_service
            .as_ref()
            .ok_or(SemanticSearchError::ServiceUnavailable)?;

        helix_service.index_note(note).await?;
        log::debug!("✅ 笔记已索引: {}", note.title);
        Ok(())
    }

    /// 获取数据目录
    fn get_data_directory(&self) -> Result<std::path::PathBuf, SemanticSearchError> {
        let app_data_dir = dirs::data_dir()
            .ok_or(SemanticSearchError::ConfigError("无法获取应用数据目录".to_string()))?
            .join("SeeU_Desktop")
            .join("semantic_search");

        std::fs::create_dir_all(&app_data_dir)
            .map_err(|e| SemanticSearchError::IoError(format!("创建数据目录失败: {}", e)))?;

        Ok(app_data_dir)
    }

    /// 创建向量化服务
    fn create_embedding_service(&self) -> Result<std::sync::Arc<dyn EmbeddingService + Send + Sync>, SemanticSearchError> {
        EmbeddingServiceFactory::create_service(&self.config.embedding_config)
            .map_err(|e| SemanticSearchError::EmbeddingError(e))
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

    pub fn set_config(&mut self, config: SemanticSearchConfig) {
        self.config = config;
    }
}
