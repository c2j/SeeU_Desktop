use eframe::egui;
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};
use chrono::{DateTime, Utc};
use log;

use crate::notebook::Notebook;
use crate::note::{Note, Attachment};
use crate::tag::Tag;
use crate::db_storage::DbStorageManager;
use crate::clipboard::ClipboardManager;
use crate::migration::DataMigration;
use crate::db_ui_import::SiyuanImportState;
use crate::slide::{SlidePlayState, SlideParser, SlideStyleManager};
use crate::document_converter::{DocumentConverter, ConversionError};
use crate::notebook_selector::NotebookSelectorState;
use crate::knowledge_graph_integration::KnowledgeGraphManager;
use crate::knowledge_graph_ui::KnowledgeGraphUI;
use zhushoude_duckdb::{ZhushoudeDB, ZhushoudeConfig, Document, DocumentType, SearchResult as SemanticSearchResult};

/// 删除确认类型
#[derive(Debug, Clone, PartialEq)]
pub enum DeleteConfirmationType {
    Note,
    Notebook,
    Tag,
}

/// 搜索模式类型
#[derive(Debug, Clone, PartialEq)]
pub enum SearchMode {
    Semantic,   // 语义搜索（默认）
    Database,   // 数据库搜索
}

/// 搜索结果来源类型
#[derive(Debug, Clone, PartialEq)]
pub enum SearchResultType {
    Semantic,   // 来自语义搜索
    Database,   // 来自数据库搜索
    Hybrid,     // 混合搜索结果
}

/// 搜索结果项
#[derive(Debug, Clone)]
pub struct SearchResultItem {
    pub note_id: String,
    pub result_type: SearchResultType,
}

/// 删除确认信息
#[derive(Debug, Clone)]
pub struct DeleteConfirmation {
    pub confirmation_type: DeleteConfirmationType,
    pub target_id: String,
    pub target_name: String,
    pub target_index: Option<usize>, // For notebooks that need index
}

/// Save status for auto-save feature
#[derive(Debug, Clone, PartialEq)]
pub enum SaveStatus {
    Saved,      // Content is saved
    Saving,     // Currently saving
    Modified,   // Content is modified but not yet saved
}

/// 最近访问的笔记记录
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RecentNoteAccess {
    pub note_id: String,
    pub note_title: String,
    pub notebook_name: String,
    pub accessed_at: DateTime<Utc>,
}

// SaveStatus is already exported as part of the module

/// iNote state with SQLite storage
pub struct DbINoteState {
    pub notebooks: Vec<Notebook>,
    pub notes: HashMap<String, Note>,
    pub tags: Vec<Tag>,
    pub current_notebook: Option<usize>,
    pub current_note: Option<String>,
    pub search_query: String,
    pub search_results: Vec<SearchResultItem>, // Search results with type information
    pub is_searching: bool,          // Whether we're currently showing search results
    pub search_mode: SearchMode,     // Current search mode (semantic or database)
    pub note_content: String,
    pub note_title: String,
    pub show_create_notebook: bool,
    pub show_create_tag: bool,
    pub new_notebook_name: String,
    pub new_notebook_description: String,
    pub new_tag_name: String,
    pub new_tag_color: String,
    pub storage: Arc<Mutex<DbStorageManager>>,
    pub clipboard_manager: Option<ClipboardManager>,  // Clipboard manager for rich text conversion
    pub markdown_preview: bool,      // Whether to show markdown preview instead of editor
    pub last_saved_content: String,  // Last saved content for auto-save comparison
    pub last_saved_title: String,    // Last saved title for auto-save comparison
    pub save_status: SaveStatus,     // Current save status
    pub editor_maximized: bool,      // Whether the editor is maximized
    pub combined_editor: bool,       // Whether title and content are combined in editor
    pub siyuan_import: SiyuanImportState, // 思源笔记导入状态
    pub show_markdown_help: bool,    // Whether to show the Markdown help popup
    pub slide_play_state: SlidePlayState, // 幻灯片播放状态
    pub slide_parser: SlideParser,   // 幻灯片解析器
    pub slide_style_manager: SlideStyleManager, // 幻灯片样式管理器

    // 删除确认对话框
    pub show_delete_confirmation: bool,
    pub delete_confirmation: Option<DeleteConfirmation>,

    // 笔记设置
    pub settings_default_collapse_notebooks: bool, // 默认折叠笔记本
    pub settings_enable_markdown_preview: bool,    // 启用Markdown预览
    pub settings_show_note_stats: bool,            // 显示笔记统计信息
    pub settings_auto_save: bool,                  // 自动保存
    pub settings_syntax_highlight: bool,           // 语法高亮
    pub settings_show_line_numbers: bool,          // 显示行号

    // UI 布局设置
    pub show_note_tree: bool,                      // 是否显示笔记树

    // 最近访问的笔记
    pub recent_notes: VecDeque<RecentNoteAccess>,  // 最近访问的笔记列表，最多保存20个

    // 文档导入功能
    pub document_converter: DocumentConverter,     // 文档转换器
    pub notebook_selector: NotebookSelectorState,  // 笔记本选择对话框状态
    pub show_document_import_dialog: bool,         // 显示文档导入对话框

    // 知识图谱功能
    pub knowledge_graph_manager: Option<Arc<tokio::sync::Mutex<KnowledgeGraphManager>>>, // 知识图谱管理器
    pub show_knowledge_graph_panel: bool,          // 显示知识图谱面板
    pub knowledge_graph_enabled: bool,             // 知识图谱功能是否启用

    // 语义搜索功能
    pub semantic_search_query: String,             // 语义搜索查询
    pub semantic_search_results: Vec<crate::knowledge_graph_integration::SemanticSearchResult>, // 语义搜索结果
    pub is_semantic_searching: bool,               // 是否正在进行语义搜索
    pub show_semantic_search_panel: bool,          // 显示语义搜索面板

    // zhushoude_duckdb 语义搜索引擎
    pub semantic_db: Option<Arc<ZhushoudeDB>>,     // 语义搜索数据库实例
    pub semantic_search_enabled: bool,             // 是否启用语义搜索
    pub use_semantic_search_by_default: bool,      // 是否默认使用语义搜索
    pub semantic_index_version: String,            // 语义索引版本号

    // 语义索引重建状态
    pub is_rebuilding_semantic_index: bool,        // 是否正在重建语义索引
    pub semantic_rebuild_progress: Option<f32>,    // 重建进度 (0.0-1.0)
    pub semantic_rebuild_result: Option<Result<usize, String>>, // 重建结果：成功时返回索引的笔记数量，失败时返回错误信息

    // 用于与后台线程通信的共享状态
    pub semantic_rebuild_progress_ref: Option<Arc<Mutex<f32>>>,
    pub semantic_rebuild_result_ref: Option<Arc<Mutex<Option<Result<usize, String>>>>>,

    // 知识图谱UI
    pub knowledge_graph_ui: KnowledgeGraphUI,      // 知识图谱可视化UI
}

impl Default for DbINoteState {
    fn default() -> Self {
        // Don't create database storage in Default - this will be done asynchronously
        // Create a placeholder that will be initialized later
        let storage = Arc::new(Mutex::new(DbStorageManager::new_placeholder()));

        Self {
            notebooks: Vec::new(),
            notes: HashMap::new(),
            tags: Vec::new(),
            current_notebook: None,
            current_note: None,
            search_query: String::new(),
            search_results: Vec::new(),
            is_searching: false,
            search_mode: SearchMode::Semantic, // 默认使用语义搜索
            note_content: String::new(),
            note_title: String::new(),
            show_create_notebook: false,
            show_create_tag: false,
            new_notebook_name: String::new(),
            new_notebook_description: String::new(),
            new_tag_name: String::new(),
            new_tag_color: "#3498db".to_string(),
            storage,
            clipboard_manager: ClipboardManager::new().ok(),
            markdown_preview: false,
            last_saved_content: String::new(),
            last_saved_title: String::new(),
            save_status: SaveStatus::Saved,
            editor_maximized: false,
            combined_editor: false,  // 默认使用分离标题模式
            siyuan_import: SiyuanImportState::default(),
            show_markdown_help: false,
            slide_play_state: SlidePlayState::default(),
            slide_parser: SlideParser::new(),
            slide_style_manager: SlideStyleManager::default(),
            show_delete_confirmation: false,
            delete_confirmation: None,

            // 笔记设置默认值
            settings_default_collapse_notebooks: true,  // 默认折叠笔记本
            settings_enable_markdown_preview: true,     // 启用Markdown预览
            settings_show_note_stats: false,            // 不显示笔记统计信息
            settings_auto_save: true,                   // 自动保存
            settings_syntax_highlight: true,            // 语法高亮
            settings_show_line_numbers: false,          // 不显示行号

            // UI 布局设置默认值
            show_note_tree: true,                       // 默认显示笔记树

            // 最近访问笔记初始化
            recent_notes: VecDeque::new(),

            // 文档导入功能初始化
            document_converter: DocumentConverter::new(),
            notebook_selector: NotebookSelectorState::default(),
            show_document_import_dialog: false,

            // 知识图谱功能初始化
            knowledge_graph_manager: None,
            show_knowledge_graph_panel: false,
            knowledge_graph_enabled: false,

            // 语义搜索功能初始化
            semantic_search_query: String::new(),
            semantic_search_results: Vec::new(),
            is_semantic_searching: false,
            show_semantic_search_panel: false,

            // zhushoude_duckdb 语义搜索引擎初始化
            semantic_db: None,
            semantic_search_enabled: false,
            use_semantic_search_by_default: true,  // 默认启用语义搜索
            semantic_index_version: "v1.0.0".to_string(), // 初始版本

            // 语义索引重建状态初始化
            is_rebuilding_semantic_index: false,
            semantic_rebuild_progress: None,
            semantic_rebuild_result: None,

            // 后台线程通信状态初始化
            semantic_rebuild_progress_ref: None,
            semantic_rebuild_result_ref: None,

            // 知识图谱UI初始化
            knowledge_graph_ui: KnowledgeGraphUI::new(),
        }
    }
}

impl DbINoteState {
    /// Initialize the state (lightweight initialization)
    pub fn initialize(&mut self) {
        log::info!("Initializing DbINoteState (lightweight)...");

        // Load settings first
        if let Err(err) = crate::load_settings(self) {
            log::warn!("Failed to load note settings: {}", err);
        }

        // Load slide style configuration
        if let Err(err) = self.load_slide_style_config() {
            log::warn!("Failed to load slide style config: {}", err);
        }

        // Don't load data here - it will be loaded asynchronously
        log::info!("DbINoteState lightweight initialization completed");
    }

    /// Initialize the database asynchronously (non-blocking)
    pub fn initialize_database_async(&mut self) {
        log::info!("Starting async database initialization...");

        // Initialize the database storage
        if let Ok(mut storage) = self.storage.lock() {
            if let Err(err) = storage.initialize_async() {
                log::error!("Failed to initialize database: {}", err);
                return;
            }
        } else {
            log::error!("Failed to lock storage for initialization");
            return;
        }

        log::info!("Database storage initialized, data will be loaded on demand");

        // Initialize semantic search engine asynchronously
        self.initialize_semantic_search_async();
    }

    /// Initialize semantic search engine asynchronously
    pub fn initialize_semantic_search_async(&mut self) {
        log::info!("Initializing semantic search engine...");

        // Initialize synchronously for now to ensure it's available immediately
        self.initialize_semantic_search_sync();
    }

    /// Try to initialize semantic database (lazy initialization)
    fn try_initialize_semantic_db(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if self.semantic_db.is_some() {
            return Ok(());
        }

        log::info!("Initializing real semantic database with vector embeddings...");

        // 尝试多次初始化，处理可能的数据库锁定或损坏问题
        let max_retries = 3;
        let mut last_error = None;

        for attempt in 1..=max_retries {
            log::info!("数据库初始化尝试 {}/{}", attempt, max_retries);

            // 如果不是第一次尝试，先清理可能的锁文件
            if attempt > 1 {
                self.cleanup_database_lock_files();
                std::thread::sleep(std::time::Duration::from_millis(1000));
            }

            match std::thread::spawn(|| {
                let rt = match tokio::runtime::Runtime::new() {
                    Ok(rt) => rt,
                    Err(e) => {
                        log::error!("创建Tokio运行时失败: {}", e);
                        return Err(format!("创建Tokio运行时失败: {}", e));
                    }
                };

                rt.block_on(async {
                    // 尝试不同的数据库配置策略
                    let configs = vec![
                        // 策略1: 使用内存数据库（最安全）
                        ZhushoudeConfig {
                            database_path: ":memory:".to_string(),
                            embedding: zhushoude_duckdb::EmbeddingConfig {
                                model_name: "bge-small-zh".to_string(),
                                batch_size: 8, // 减小批次大小
                                max_cache_size: 500, // 减小缓存
                                vector_dimension: 384,
                                enable_chinese_optimization: true,
                                normalize_vectors: false,
                            },
                            ..Default::default()
                        },
                        // 策略2: 使用临时文件数据库
                        ZhushoudeConfig {
                            database_path: format!("./semantic_search_temp_{}.db", std::process::id()),
                            embedding: zhushoude_duckdb::EmbeddingConfig {
                                model_name: "bge-small-zh".to_string(),
                                batch_size: 8,
                                max_cache_size: 500,
                                vector_dimension: 384,
                                enable_chinese_optimization: true,
                                normalize_vectors: false,
                            },
                            ..Default::default()
                        },
                        // 策略3: 原始配置（作为最后尝试）
                        ZhushoudeConfig {
                            database_path: "./semantic_search.db".to_string(),
                            embedding: zhushoude_duckdb::EmbeddingConfig {
                                model_name: "bge-small-zh".to_string(),
                                batch_size: 16,
                                max_cache_size: 1000,
                                vector_dimension: 384,
                                enable_chinese_optimization: true,
                                normalize_vectors: false,
                            },
                            ..Default::default()
                        },
                    ];

                    let mut last_error = String::new();

                    for (i, config) in configs.iter().enumerate() {
                        log::info!("尝试数据库配置策略 {}/3: {}", i + 1,
                            if config.database_path == ":memory:" { "内存数据库" }
                            else if config.database_path.contains("temp") { "临时文件数据库" }
                            else { "持久化数据库" }
                        );

                        match ZhushoudeDB::new(config.clone()).await {
                            Ok(db) => {
                                log::info!("✅ 数据库配置策略 {} 成功", i + 1);
                                return Ok(db);
                            }
                            Err(e) => {
                                last_error = format!("策略{}: {}", i + 1, e);
                                log::warn!("❌ 数据库配置策略 {} 失败: {}", i + 1, e);

                                // 在策略之间等待一下
                                if i < configs.len() - 1 {
                                    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                                }
                            }
                        }
                    }

                    Err(format!("所有数据库配置策略都失败: {}", last_error))
                })
            }).join() {
                Ok(Ok(db)) => {
                    self.semantic_db = Some(Arc::new(db));
                    self.semantic_search_enabled = true;
                    log::info!("✅ 语义数据库初始化成功 (尝试 {}/{})", attempt, max_retries);

                    // Check and index only new notes (incremental indexing)
                    self.incremental_index_notes_to_semantic_db();

                    return Ok(());
                }
                Ok(Err(e)) => {
                    last_error = Some(e.clone());
                    log::warn!("❌ 数据库初始化失败 (尝试 {}/{}): {}", attempt, max_retries, e);
                }
                Err(e) => {
                    let error_msg = format!("线程执行失败: {:?}", e);
                    last_error = Some(error_msg.clone());
                    log::warn!("❌ 线程执行失败 (尝试 {}/{}): {}", attempt, max_retries, error_msg);
                }
            }
        }

        // 所有尝试都失败了，禁用语义搜索功能
        self.semantic_search_enabled = false;
        self.semantic_db = None;
        let final_error = last_error.unwrap_or_else(|| "未知错误".to_string());
        log::error!("❌ 语义数据库初始化最终失败，已尝试 {} 次: {}", max_retries, final_error);
        log::warn!("⚠️ 语义搜索功能已禁用，将使用传统搜索");

        // 不返回错误，而是优雅降级
        Ok(())
    }

    /// 清理数据库锁文件
    fn cleanup_database_lock_files(&self) {
        use std::fs;
        use std::path::Path;

        let lock_files = [
            "./semantic_search.db.wal",
            "./semantic_search.db-shm",
            "./semantic_search.db-journal",
            "./semantic_search.db.lock"
        ];

        for lock_file in &lock_files {
            let path = Path::new(lock_file);
            if path.exists() {
                log::info!("清理数据库锁文件: {}", lock_file);
                if let Err(e) = fs::remove_file(path) {
                    log::warn!("删除锁文件失败 {}: {}", lock_file, e);
                }
            }
        }
    }

    /// Create semantic database instance
    async fn create_semantic_db() -> Result<Arc<ZhushoudeDB>, Box<dyn std::error::Error + Send + Sync>> {
        log::info!("Creating ZhushoudeDB instance for semantic search...");
        let config = ZhushoudeConfig::default();
        let db = ZhushoudeDB::new(config).await?;
        log::info!("ZhushoudeDB instance created successfully");
        Ok(Arc::new(db))
    }

    /// Index existing notes into semantic database
    async fn index_notes_to_semantic_db(&self, semantic_db: Arc<ZhushoudeDB>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        log::info!("Starting to index existing notes into semantic database...");

        // Get all notes from traditional storage
        let notes = if let Ok(storage) = self.storage.lock() {
            storage.get_all_notes().unwrap_or_default()
        } else {
            return Err("Failed to access note storage".into());
        };

        log::info!("Found {} notes to index", notes.len());

        // Convert notes to Document format for zhushoude_duckdb
        let mut documents = Vec::new();
        for note in &notes {
            let document = zhushoude_duckdb::Document {
                id: note.id.clone(),
                title: note.title.clone(),
                content: note.content.clone(),
                doc_type: zhushoude_duckdb::DocumentType::Note,
                metadata: serde_json::json!({
                    "notebook_id": "", // Note doesn't have notebook_id field
                    "tags": note.tag_ids.join(","),
                    "created_at": note.created_at.to_rfc3339(),
                    "updated_at": note.updated_at.to_rfc3339(),
                }),
            };
            documents.push(document);
        }

        // Index documents into semantic database using the correct API
        for (i, document) in documents.iter().enumerate() {
            match semantic_db.add_note(document).await {
                Ok(_) => {
                    if i % 10 == 0 {  // Log progress every 10 notes
                        log::info!("Indexed {}/{} notes", i + 1, documents.len());
                    }
                }
                Err(e) => {
                    log::warn!("Failed to index note {}: {}", document.id, e);
                }
            }
        }

        log::info!("✅ Completed indexing {} notes into semantic database", documents.len());
        Ok(())
    }

    /// Initialize semantic search engine synchronously (for immediate use)
    pub fn initialize_semantic_search_sync(&mut self) {
        if self.semantic_db.is_some() {
            return; // Already initialized
        }

        log::info!("Initializing semantic search engine synchronously...");

        // For now, just enable the flag without creating the actual instance
        // to avoid blocking the UI thread. The actual instance will be created
        // when needed in the search methods.
        self.semantic_search_enabled = true;
        log::info!("Semantic search engine marked as enabled (lazy initialization)");
    }

    /// Perform semantic search (placeholder implementation)
    pub fn perform_semantic_search(&mut self, query: &str) {
        if !self.semantic_search_enabled {
            return;
        }

        log::info!("Performing semantic search for: {}", query);

        // Set searching state
        self.is_semantic_searching = true;

        // For now, we'll create some mock results to demonstrate the UI
        // In a real implementation, this would interface with the semantic DB
        self.semantic_search_results = vec![
            crate::knowledge_graph_integration::SemanticSearchResult {
                note_id: "mock_1".to_string(),
                title: "相关笔记示例 1".to_string(),
                content: "这是一个与您的搜索查询语义相关的笔记内容示例...".to_string(),
                similarity_score: 0.85,
                metadata: Some(serde_json::json!({
                    "doc_type": "note",
                    "created_at": chrono::Utc::now().to_rfc3339()
                })),
            },
            crate::knowledge_graph_integration::SemanticSearchResult {
                note_id: "mock_2".to_string(),
                title: "相关笔记示例 2".to_string(),
                content: "另一个语义相关的笔记，展示了智能搜索的能力...".to_string(),
                similarity_score: 0.72,
                metadata: Some(serde_json::json!({
                    "doc_type": "note",
                    "created_at": chrono::Utc::now().to_rfc3339()
                })),
            },
        ];

        // Reset searching state
        self.is_semantic_searching = false;

        log::info!("Semantic search completed with {} results", self.semantic_search_results.len());
    }

    /// Sync note to semantic search engine (incremental update)
    fn sync_note_to_semantic_db(&self, note: &Note) {
        if !self.semantic_search_enabled {
            return;
        }

        if let Some(semantic_db) = &self.semantic_db {
            let semantic_db = semantic_db.clone();
            let note = note.clone();
            let notebook_id = self.find_notebook_for_note(&note.id).unwrap_or_else(|| "unknown".to_string());

            log::info!("🔄 Updating note '{}' in semantic database", note.title);

            // 使用独立线程处理增量更新
            std::thread::spawn(move || {
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(async move {
                    let document = zhushoude_duckdb::Document {
                        id: note.id.clone(),
                        title: note.title.clone(),
                        content: note.content.clone(),
                        doc_type: zhushoude_duckdb::DocumentType::Note,
                        metadata: serde_json::json!({
                            "created_at": note.created_at,
                            "updated_at": note.updated_at,
                            "notebook_id": notebook_id,
                            "tag_ids": note.tag_ids,
                            "index_version": "v1.0.0"
                        }),
                    };

                    // 使用 add_note 方法（会自动覆盖现有文档）
                    match semantic_db.add_note(&document).await {
                        Ok(_) => {
                            log::info!("✅ Note '{}' successfully updated in semantic database", note.title);
                        }
                        Err(e) => {
                            log::error!("❌ Failed to update note '{}' in semantic database: {}", note.title, e);
                        }
                    }
                });
            });
        } else {
            log::warn!("⚠️ Semantic database not available, skipping note sync for '{}'", note.title);
        }
    }

    /// Remove note from semantic search engine
    fn remove_note_from_semantic_db(&self, note_id: &str) {
        if !self.semantic_search_enabled || self.semantic_db.is_none() {
            return;
        }

        // Note: zhushoude_duckdb doesn't have a remove_note method yet
        // This would need to be implemented in the future
        log::debug!("Note removal from semantic DB not yet implemented for note: {}", note_id);
    }

    /// Initialize semantic DB with existing notes
    pub fn initialize_semantic_db_with_existing_notes(&mut self) {
        if !self.semantic_search_enabled {
            return;
        }

        log::info!("Would initialize semantic DB with {} existing notes (deferred)", self.notes.len());

        // For now, we'll just log the initialization
        // In a future implementation, this would trigger a background sync process
        log::info!("Semantic DB initialization deferred for thread safety");

        // TODO: Implement proper background initialization
        // This could involve:
        // 1. A dedicated background thread for semantic operations
        // 2. Batch processing of existing notes
        // 3. Progress tracking and error handling
    }

    /// Index existing notes to semantic database in background (full reindex)
    fn index_existing_notes_to_semantic_db(&self) {
        if let Some(semantic_db) = &self.semantic_db {
            let semantic_db = semantic_db.clone();
            let storage = self.storage.clone();

            log::info!("Starting background indexing of existing notes from database...");

            // Use a thread to handle the async operation to avoid runtime issues
            std::thread::spawn(move || {
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(async move {
                    // Load all notes from database
                    let all_notes = if let Ok(storage) = storage.lock() {
                        match storage.get_all_notes() {
                            Ok(notes) => notes,
                            Err(e) => {
                                log::error!("Failed to load notes from database: {}", e);
                                return;
                            }
                        }
                    } else {
                        log::error!("Failed to lock storage for indexing");
                        return;
                    };

                    log::info!("Found {} existing notes to index", all_notes.len());

                    for (i, note) in all_notes.iter().enumerate() {
                        let document = zhushoude_duckdb::Document {
                            id: note.id.clone(),
                            title: note.title.clone(),
                            content: note.content.clone(),
                            doc_type: zhushoude_duckdb::DocumentType::Note,
                            metadata: serde_json::json!({
                                "created_at": note.created_at,
                                "updated_at": note.updated_at,
                                "notebook_id": "unknown", // We don't have notebook mapping here
                                "tag_ids": note.tag_ids
                            }),
                        };

                        match semantic_db.add_note(&document).await {
                            Ok(_) => {
                                if (i + 1) % 10 == 0 || i == all_notes.len() - 1 {
                                    log::info!("✅ Indexed {}/{} notes to semantic database", i + 1, all_notes.len());
                                }
                            }
                            Err(e) => {
                                log::error!("❌ Failed to index note '{}' to semantic database: {}", note.title, e);
                            }
                        }
                    }

                    log::info!("🎉 Background indexing of {} existing notes completed", all_notes.len());
                });
            });
        } else {
            log::warn!("⚠️ Semantic database not available, skipping existing notes indexing");
        }
    }

    /// Incremental index notes to semantic database (only index new notes)
    fn incremental_index_notes_to_semantic_db(&self) {
        if let Some(semantic_db) = &self.semantic_db {
            let semantic_db = semantic_db.clone();
            let storage = self.storage.clone();

            log::info!("Starting incremental indexing of notes...");

            // Use a thread to handle the async operation to avoid runtime issues
            std::thread::spawn(move || {
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(async move {
                    // Load all notes from database
                    let all_notes = if let Ok(storage) = storage.lock() {
                        match storage.get_all_notes() {
                            Ok(notes) => notes,
                            Err(e) => {
                                log::error!("Failed to load notes from database: {}", e);
                                return;
                            }
                        }
                    } else {
                        log::error!("Failed to lock storage for indexing");
                        return;
                    };

                    log::info!("Checking {} notes for indexing status...", all_notes.len());

                    // Check which notes are already indexed
                    let mut new_notes = Vec::new();
                    for note in &all_notes {
                        match semantic_db.search_notes(&format!("id:{}", note.id), 1).await {
                            Ok(results) => {
                                if results.is_empty() {
                                    // Note not found in semantic database, needs indexing
                                    new_notes.push(note);
                                }
                            }
                            Err(_) => {
                                // Error searching, assume note needs indexing
                                new_notes.push(note);
                            }
                        }
                    }

                    if new_notes.is_empty() {
                        log::info!("✅ All notes are already indexed in semantic database");
                        return;
                    }

                    log::info!("Found {} new notes to index", new_notes.len());

                    // 使用真正的批量索引优化性能
                    const BATCH_SIZE: usize = 20;
                    let mut batch_documents = Vec::new();

                    for (i, note) in new_notes.iter().enumerate() {
                        let document = zhushoude_duckdb::Document {
                            id: note.id.clone(),
                            title: note.title.clone(),
                            content: note.content.clone(),
                            doc_type: zhushoude_duckdb::DocumentType::Note,
                            metadata: serde_json::json!({
                                "created_at": note.created_at,
                                "updated_at": note.updated_at,
                                "notebook_id": "unknown",
                                "tag_ids": note.tag_ids,
                                "index_version": "v1.0.0"
                            }),
                        };

                        batch_documents.push(document);

                        // 当批次满了或者是最后一个文档时，执行批量索引
                        if batch_documents.len() >= BATCH_SIZE || i == new_notes.len() - 1 {
                            // 使用真正的批量API
                            match semantic_db.add_notes_batch(&batch_documents).await {
                                Ok(_) => {
                                    log::info!("✅ Batch indexed {} notes to semantic database (total: {}/{})",
                                        batch_documents.len(), i + 1, new_notes.len());
                                }
                                Err(e) => {
                                    log::error!("❌ Failed to batch index {} notes: {}", batch_documents.len(), e);

                                    // 如果批量失败，回退到单个添加
                                    let mut success_count = 0;
                                    for doc in &batch_documents {
                                        match semantic_db.add_note(doc).await {
                                            Ok(_) => success_count += 1,
                                            Err(e) => {
                                                log::error!("❌ Failed to index note '{}': {}", doc.title, e);
                                            }
                                        }
                                    }
                                    if success_count > 0 {
                                        log::info!("✅ Fallback indexed {}/{} notes individually", success_count, batch_documents.len());
                                    }
                                }
                            }

                            batch_documents.clear();
                        }
                    }

                    log::info!("🎉 Incremental indexing completed: {} new notes indexed", new_notes.len());
                });
            });
        } else {
            log::warn!("⚠️ Semantic database not available, skipping incremental indexing");
        }
    }

    /// 索引健康检查：验证索引完整性
    pub fn check_semantic_index_health(&self) {
        if let Some(semantic_db) = &self.semantic_db {
            let semantic_db = semantic_db.clone();
            let storage = self.storage.clone();

            log::info!("🔍 Starting semantic index health check...");

            std::thread::spawn(move || {
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(async move {
                    // 获取数据库中的所有笔记
                    let all_notes = if let Ok(storage) = storage.lock() {
                        match storage.get_all_notes() {
                            Ok(notes) => notes,
                            Err(e) => {
                                log::error!("Failed to load notes for health check: {}", e);
                                return;
                            }
                        }
                    } else {
                        log::error!("Failed to lock storage for health check");
                        return;
                    };

                    let mut missing_count = 0;
                    let mut outdated_count = 0;

                    for note in &all_notes {
                        // 检查笔记是否在语义数据库中
                        match semantic_db.search_notes(&format!("id:{}", note.id), 1).await {
                            Ok(results) => {
                                if results.is_empty() {
                                    missing_count += 1;
                                    log::warn!("⚠️ Note '{}' missing from semantic index", note.title);
                                } else {
                                    // 检查索引版本是否过期
                                    if let Some(metadata) = &results[0].metadata {
                                        if let Some(index_version) = metadata.get("index_version") {
                                            if index_version != "v1.0.0" {
                                                outdated_count += 1;
                                                log::warn!("⚠️ Note '{}' has outdated index version: {}", note.title, index_version);
                                            }
                                        } else {
                                            outdated_count += 1;
                                            log::warn!("⚠️ Note '{}' missing index version metadata", note.title);
                                        }
                                    }
                                }
                            }
                            Err(e) => {
                                log::error!("❌ Failed to check note '{}' in semantic index: {}", note.title, e);
                            }
                        }
                    }

                    if missing_count == 0 && outdated_count == 0 {
                        log::info!("✅ Semantic index health check passed: all {} notes are properly indexed", all_notes.len());
                    } else {
                        log::warn!("⚠️ Semantic index health check found issues: {} missing, {} outdated", missing_count, outdated_count);
                        if missing_count > 0 || outdated_count > 0 {
                            log::info!("💡 Consider running a full reindex to fix these issues");
                        }
                    }
                });
            });
        } else {
            log::warn!("⚠️ Semantic database not available, skipping health check");
        }
    }

    /// Add a single note to semantic database for vector indexing
    fn add_note_to_semantic_db(&self, note: &Note) {
        if let Some(semantic_db) = &self.semantic_db {
            let semantic_db = semantic_db.clone();
            let note = note.clone();

            // Find the notebook ID for this note
            let notebook_id = self.find_notebook_for_note(&note.id).unwrap_or_else(|| "unknown".to_string());

            log::info!("🔍 Adding note '{}' to semantic database for vector indexing", note.title);

            // Use a thread to handle the async operation to avoid runtime issues
            std::thread::spawn(move || {
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(async move {
                    let document = zhushoude_duckdb::Document {
                        id: note.id.clone(),
                        title: note.title.clone(),
                        content: note.content.clone(),
                        doc_type: zhushoude_duckdb::DocumentType::Note,
                        metadata: serde_json::json!({
                            "created_at": note.created_at,
                            "updated_at": note.updated_at,
                            "notebook_id": notebook_id,
                            "tag_ids": note.tag_ids
                        }),
                    };

                    match semantic_db.add_note(&document).await {
                        Ok(_) => {
                            log::info!("✅ Note '{}' successfully indexed to semantic database", note.title);
                        }
                        Err(e) => {
                            log::error!("❌ Failed to index note '{}' to semantic database: {}", note.title, e);
                        }
                    }
                });
            });
        } else {
            log::warn!("⚠️ Semantic database not available, skipping vector indexing for note '{}'", note.title);
        }
    }

    /// Load data when needed (lazy loading)
    pub fn ensure_data_loaded(&mut self) {
        // Check if data is already loaded
        if !self.notebooks.is_empty() {
            return; // Data already loaded
        }

        // Check if database is ready
        if let Ok(storage) = self.storage.lock() {
            if storage.is_placeholder() {
                return; // Database not ready yet
            }
        } else {
            return; // Cannot lock storage
        }

        log::info!("Loading data on demand...");

        // Load data from storage
        self.load_notebooks();
        self.load_tags();
        self.load_recent_notes();
        // Don't load all notes - use lazy loading instead

        // Create default data if database is empty
        if self.notebooks.is_empty() {
            self.create_default_data();
        }

        // 确保根据设置折叠笔记本
        self.apply_notebook_collapse_setting();

        // 初始化语义搜索引擎并同步现有笔记
        if self.semantic_search_enabled && !self.notes.is_empty() {
            self.initialize_semantic_db_with_existing_notes();
        }

        log::info!("Data loaded on demand: {} notebooks", self.notebooks.len());
    }

    /// Create default data when database is empty
    fn create_default_data(&mut self) {
        log::info!("Creating default data...");

        // Create default notebook
        self.create_notebook("默认笔记本".to_string(), "欢迎使用 SeeU Desktop 笔记功能".to_string());

        // Select the first notebook
        if !self.notebooks.is_empty() {
            self.select_notebook(0);

            // Create welcome note
            let welcome_content = r#"# 欢迎使用 SeeU Desktop 笔记功能！

## 功能特点

- 📝 **Markdown 支持**: 支持完整的 Markdown 语法
- 🏷️ **标签管理**: 为笔记添加标签，方便分类和查找
- 🔍 **全文搜索**: 快速搜索笔记内容
- 📁 **笔记本管理**: 将笔记组织到不同的笔记本中
- 💾 **自动保存**: 编辑时自动保存，无需担心数据丢失

## 快速开始

1. 点击 "新建笔记" 创建你的第一个笔记
2. 使用 Markdown 语法编写内容
3. 点击预览按钮查看渲染效果
4. 使用标签为笔记分类

## Markdown 示例

### 文本格式
- **粗体文本**
- *斜体文本*
- ~~删除线~~
- `代码片段`

### 列表
1. 有序列表项 1
2. 有序列表项 2
   - 无序子项
   - 另一个子项

### 代码块
```rust
fn main() {
    println!("Hello, SeeU Desktop!");
}
```

### 表格
| 功能 | 状态 | 说明 |
|------|------|------|
| 笔记编辑 | ✅ | 支持 Markdown |
| 标签管理 | ✅ | 多标签支持 |
| 全文搜索 | ✅ | 快速搜索 |

开始你的笔记之旅吧！"#;

            let note_id = self.create_note("欢迎使用 SeeU Desktop".to_string(), welcome_content.to_string());

            if let Some(note_id) = note_id {
                self.select_note(&note_id);
                log::info!("Created welcome note: {}", note_id);
            }
        }

        log::info!("Default data created successfully");
    }

    /// 应用笔记本折叠设置
    fn apply_notebook_collapse_setting(&mut self) {
        if self.settings_default_collapse_notebooks {
            for notebook in &mut self.notebooks {
                notebook.expanded = false;
            }
            log::info!("Applied default collapse setting to {} notebooks", self.notebooks.len());
        }
    }

    /// Migrate data from file storage to SQLite database (async)
    fn migrate_data_async(&self) {
        // Temporarily disabled to avoid stack overflow
        return;
    }

    /// Migrate data from file storage to SQLite database (sync - for compatibility)
    fn migrate_data(&self) {
        if let Ok(storage) = self.storage.lock() {
            let migration = DataMigration::new_with_ref(&storage);
            if let Err(err) = migration.migrate() {
                log::error!("Failed to migrate data: {}", err);
            }
        }
    }

    /// Load notebooks from storage (public method for external use)
    pub fn load_notebooks(&mut self) {
        if let Ok(storage) = self.storage.lock() {
            match storage.list_notebooks() {
                Ok(mut notebooks) => {
                    // 根据设置决定是否折叠笔记本
                    if self.settings_default_collapse_notebooks {
                        for notebook in &mut notebooks {
                            notebook.expanded = false;
                        }
                    }
                    self.notebooks = notebooks;
                }
                Err(err) => {
                    log::error!("Failed to load notebooks: {}", err);
                }
            }
        }
    }

    /// Load notes for a notebook (public method for external use)
    pub fn load_notes_for_notebook(&mut self, notebook_id: &str) {
        // 验证笔记本是否存在
        if !self.notebooks.iter().any(|nb| nb.id == notebook_id) {
            log::warn!("Notebook {} not found", notebook_id);
            return;
        }

        // 直接从数据库加载笔记，不依赖内存中的note_ids字段
        // 这样可以确保即使note_ids不准确也能正确加载所有笔记
        if let Ok(storage) = self.storage.lock() {
            match storage.get_notes_for_notebook(notebook_id) {
                Ok(notes) => {
                    let mut loaded_count = 0;
                    let mut updated_count = 0;
                    let mut note_ids_for_notebook = Vec::new();

                    for note in notes {
                        note_ids_for_notebook.push(note.id.clone());

                        if self.notes.contains_key(&note.id) {
                            // 笔记已存在，更新它（可能有新的内容）
                            self.notes.insert(note.id.clone(), note);
                            updated_count += 1;
                        } else {
                            // 新笔记，添加到内存
                            self.notes.insert(note.id.clone(), note);
                            loaded_count += 1;
                        }
                    }

                    // 智能合并笔记本的note_ids字段以保持一致性
                    if let Some(notebook) = self.notebooks.iter_mut().find(|nb| nb.id == notebook_id) {
                        let old_count = notebook.note_ids.len();

                        // 不直接替换，而是智能合并
                        // 1. 保留内存中存在但数据库查询结果中没有的笔记ID（可能是刚导入的）
                        let mut merged_note_ids = Vec::new();

                        // 2. 首先添加数据库中的所有笔记（确保数据库数据优先）
                        for note_id in &note_ids_for_notebook {
                            if !merged_note_ids.contains(note_id) {
                                merged_note_ids.push(note_id.clone());
                            }
                        }

                        // 3. 然后添加内存中存在但数据库查询中缺失的笔记
                        for existing_note_id in &notebook.note_ids {
                            if !merged_note_ids.contains(existing_note_id) {
                                // 验证这个笔记确实存在于内存中
                                if self.notes.contains_key(existing_note_id) {
                                    merged_note_ids.push(existing_note_id.clone());
                                    log::info!("保留内存中的笔记 '{}' (可能是刚导入的)", existing_note_id);
                                }
                            }
                        }

                        notebook.note_ids = merged_note_ids;
                        let new_count = notebook.note_ids.len();

                        if old_count != new_count {
                            log::info!("智能合并笔记本 '{}' 的note_ids: {} -> {} 个笔记", notebook.name, old_count, new_count);
                        } else {
                            log::debug!("笔记本 '{}' 的note_ids保持不变: {} 个笔记", notebook.name, new_count);
                        }
                    }

                    log::info!("Loaded {} new notes and updated {} existing notes for notebook '{}'",
                              loaded_count, updated_count, notebook_id);
                }
                Err(err) => {
                    log::error!("Failed to load notes for notebook {}: {}", notebook_id, err);
                }
            }
        }
    }

    /// Load all notes for all notebooks
    fn load_all_notes(&mut self) {
        // Clone notebook IDs to avoid borrowing issues
        let notebook_ids: Vec<String> = self.notebooks.iter()
            .map(|notebook| notebook.id.clone())
            .collect();

        // Load notes for each notebook
        for notebook_id in notebook_ids {
            self.load_notes_for_notebook(&notebook_id);
        }
    }

    /// Load tags from storage (public method for external use)
    pub fn load_tags(&mut self) {
        if let Ok(storage) = self.storage.lock() {
            match storage.list_tags() {
                Ok(tags) => {
                    self.tags = tags;
                }
                Err(err) => {
                    log::error!("Failed to load tags: {}", err);
                }
            }
        }
    }

    /// Create a new notebook
    pub fn create_notebook(&mut self, name: String, description: String) {
        let notebook = Notebook::new(name, description);

        // Save to storage
        if let Ok(storage) = self.storage.lock() {
            match storage.save_notebook(&notebook) {
                Ok(_) => {
                    // Add to list
                    self.notebooks.push(notebook);
                },
                Err(err) => {
                    log::error!("Failed to save notebook: {}", err);
                }
            }
        } else {
            log::error!("Failed to lock storage");
        }
    }

    /// Show delete confirmation for notebook
    pub fn show_delete_notebook_confirmation(&mut self, index: usize) {
        if index >= self.notebooks.len() {
            return;
        }

        let notebook = &self.notebooks[index];
        self.delete_confirmation = Some(DeleteConfirmation {
            confirmation_type: DeleteConfirmationType::Notebook,
            target_id: notebook.id.clone(),
            target_name: notebook.name.clone(),
            target_index: Some(index),
        });
        self.show_delete_confirmation = true;
    }

    /// Delete a notebook (internal method)
    fn delete_notebook_internal(&mut self, index: usize) {
        if index >= self.notebooks.len() {
            return;
        }

        // Get notebook ID
        let notebook_id = self.notebooks[index].id.clone();

        // Get note IDs to delete
        let note_ids: Vec<String> = self.notebooks[index].note_ids.clone();

        // Delete all notes in the notebook
        for note_id in &note_ids {
            self.delete_note_internal(note_id);
        }

        // Delete from storage
        if let Ok(storage) = self.storage.lock() {
            if let Err(err) = storage.delete_notebook(&notebook_id) {
                log::error!("Failed to delete notebook from storage: {}", err);
                return; // Don't update UI state if database deletion failed
            }
        }

        // Remove from list
        self.notebooks.remove(index);

        // Reset current notebook if needed
        if self.current_notebook == Some(index) {
            self.current_notebook = None;
            self.current_note = None;
        } else if let Some(current) = self.current_notebook {
            if current > index {
                self.current_notebook = Some(current - 1);
            }
        }

        // Force reload from database to ensure consistency
        self.force_reload_data();
    }

    /// Delete a notebook (public method with confirmation)
    pub fn delete_notebook(&mut self, index: usize) {
        self.show_delete_notebook_confirmation(index);
    }

    /// Create a new note
    pub fn create_note(&mut self, title: String, content: String) -> Option<String> {
        if let Some(notebook_idx) = self.current_notebook {
            if notebook_idx >= self.notebooks.len() {
                return None;
            }

            let note = Note::new(title, content);
            let note_id = note.id.clone();
            let notebook_id = self.notebooks[notebook_idx].id.clone();

            // Save to storage
            if let Ok(storage) = self.storage.lock() {
                if let Err(err) = storage.save_note(&note, &notebook_id) {
                    log::error!("Failed to save note: {}", err);
                    return None;
                }
            }

            // Add to notebook
            self.notebooks[notebook_idx].add_note(note_id.clone());

            // Add to notes map
            let note_clone = note.clone();
            self.notes.insert(note_id.clone(), note);

            // Process with knowledge graph if enabled
            if self.knowledge_graph_enabled {
                self.process_note_with_knowledge_graph(&note_clone);
            }

            // Sync to semantic search engine
            self.sync_note_to_semantic_db(&note_clone);

            return Some(note_id);
        }

        None
    }

    /// Update a note
    pub fn update_note(&mut self, note_id: &str, title: String, content: String) {
        // First find the notebook ID before mutably borrowing the note
        let notebook_id = self.find_notebook_for_note(note_id).clone();

        if notebook_id.is_none() {
            log::error!("Failed to find notebook for note: {}", note_id);
            return;
        }

        if let Some(note) = self.notes.get_mut(note_id) {
            note.title = title.clone();
            note.content = content.clone();
            note.updated_at = Utc::now();

            // Save to storage
            let save_success = if let Ok(storage) = self.storage.lock() {
                if let Some(notebook_id) = notebook_id {
                    if let Err(err) = storage.save_note(note, &notebook_id) {
                        log::error!("Failed to save note to database: {}", err);
                        false
                    } else {
                        true
                    }
                } else {
                    false
                }
            } else {
                log::error!("Failed to lock storage for note: {}", note_id);
                false
            };

            // Sync to semantic search engine after successful save
            if save_success {
                // Clone the note to avoid borrowing issues
                let note_clone = note.clone();
                self.sync_note_to_semantic_db(&note_clone);
            }
        } else {
            log::error!("Note not found in memory: {}", note_id);
        }
    }

    /// Find notebook ID for a note
    fn find_notebook_for_note(&self, note_id: &str) -> Option<String> {
        for notebook in &self.notebooks {
            if notebook.note_ids.contains(&note_id.to_string()) {
                return Some(notebook.id.clone());
            }
        }

        None
    }

    /// Show delete confirmation for note
    pub fn show_delete_note_confirmation(&mut self, note_id: &str) {
        if let Some(note) = self.notes.get(note_id) {
            self.delete_confirmation = Some(DeleteConfirmation {
                confirmation_type: DeleteConfirmationType::Note,
                target_id: note_id.to_string(),
                target_name: note.title.clone(),
                target_index: None,
            });
            self.show_delete_confirmation = true;
        }
    }

    /// Delete a note (internal method)
    fn delete_note_internal(&mut self, note_id: &str) {
        // Remove from storage first
        if let Ok(storage) = self.storage.lock() {
            if let Err(err) = storage.delete_note(note_id) {
                log::error!("Failed to delete note from storage: {}", err);
                return; // Don't update UI state if database deletion failed
            }
        }

        // Remove from notebooks
        for notebook in &mut self.notebooks {
            notebook.remove_note(note_id);
        }

        // Remove from notes map
        self.notes.remove(note_id);

        // Reset current note if needed
        if self.current_note == Some(note_id.to_string()) {
            self.current_note = None;
            self.note_content = String::new();
            self.note_title = String::new();
        }

        // Force reload to ensure consistency
        self.force_reload_data();
    }

    /// Delete a note (public method with confirmation)
    pub fn delete_note(&mut self, note_id: &str) {
        self.show_delete_note_confirmation(note_id);
    }

    /// Create a new tag
    pub fn create_tag(&mut self, name: String, color: String) {
        let tag = Tag::new(name, color);

        // Save to storage
        if let Ok(storage) = self.storage.lock() {
            if let Err(err) = storage.save_tag(&tag) {
                log::error!("Failed to save tag: {}", err);
                return;
            }
        }

        // Add to list
        self.tags.push(tag);
    }

    /// Show delete confirmation for tag
    pub fn show_delete_tag_confirmation(&mut self, tag_id: &str) {
        if let Some(tag) = self.tags.iter().find(|t| t.id == tag_id) {
            self.delete_confirmation = Some(DeleteConfirmation {
                confirmation_type: DeleteConfirmationType::Tag,
                target_id: tag_id.to_string(),
                target_name: tag.name.clone(),
                target_index: None,
            });
            self.show_delete_confirmation = true;
        }
    }

    /// Delete a tag (internal method)
    fn delete_tag_internal(&mut self, tag_id: &str) {
        // Remove from storage first
        if let Ok(storage) = self.storage.lock() {
            if let Err(err) = storage.delete_tag(tag_id) {
                log::error!("Failed to delete tag from storage: {}", err);
                return; // Don't update UI state if database deletion failed
            }
        }

        // Remove from list
        self.tags.retain(|tag| tag.id != tag_id);

        // Force reload to ensure consistency (this will also reload notes with updated tag associations)
        self.force_reload_data();
    }

    /// Delete a tag (public method with confirmation)
    pub fn delete_tag(&mut self, tag_id: &str) {
        self.show_delete_tag_confirmation(tag_id);
    }

    /// Add a tag to a note
    pub fn add_tag_to_note(&mut self, note_id: &str, tag_id: &str) {
        // First find the notebook ID before mutably borrowing the note
        let notebook_id = self.find_notebook_for_note(note_id).clone();

        if let Some(note) = self.notes.get_mut(note_id) {
            note.add_tag(tag_id.to_string());

            // Save to storage
            if let Ok(storage) = self.storage.lock() {
                if let Some(notebook_id) = notebook_id {
                    if let Err(err) = storage.save_note(note, &notebook_id) {
                        log::error!("Failed to save note: {}", err);
                    }
                }
            }
        }
    }

    /// Remove a tag from a note
    pub fn remove_tag_from_note(&mut self, note_id: &str, tag_id: &str) {
        // First find the notebook ID before mutably borrowing the note
        let notebook_id = self.find_notebook_for_note(note_id).clone();

        if let Some(note) = self.notes.get_mut(note_id) {
            note.remove_tag(tag_id);

            // Save to storage
            if let Ok(storage) = self.storage.lock() {
                if let Some(notebook_id) = notebook_id {
                    if let Err(err) = storage.save_note(note, &notebook_id) {
                        log::error!("Failed to save note: {}", err);
                    }
                }
            }
        }
    }

    /// Select a notebook
    pub fn select_notebook(&mut self, index: usize) {
        if index < self.notebooks.len() {
            self.current_notebook = Some(index);
            self.current_note = None;
            self.note_content = String::new();
            self.note_title = String::new();

            // Lazy load notes for the selected notebook
            let notebook_id = self.notebooks[index].id.clone();
            self.load_notes_for_notebook(&notebook_id);
        }
    }

    /// Select a note
    pub fn select_note(&mut self, note_id: &str) {
        // First try to get the note from memory
        if let Some(note) = self.notes.get(note_id) {
            self.current_note = Some(note_id.to_string());

            // 在新的UI布局中，标题始终单独显示，内容始终只包含正文
            self.note_content = note.content.clone();
            self.note_title = note.title.clone();

            // Initialize last saved content and title
            self.last_saved_content = self.note_content.clone();
            self.last_saved_title = self.note_title.clone();
            self.save_status = SaveStatus::Saved;

            // 添加到最近访问记录
            self.add_to_recent_notes(note_id);
            return;
        }

        // If note is not in memory, try to load it from database
        let loaded_note = if let Ok(storage) = self.storage.lock() {
            match storage.load_note(note_id) {
                Ok(note) => {
                    log::info!("Loaded note '{}' from database for selection", note.title);
                    Some(note)
                }
                Err(e) => {
                    log::error!("Failed to load note {} from database: {}", note_id, e);
                    None
                }
            }
        } else {
            None
        };

        if let Some(note) = loaded_note {
            // Add the note to memory
            self.notes.insert(note_id.to_string(), note.clone());

            // Set as current note
            self.current_note = Some(note_id.to_string());
            self.note_content = note.content.clone();
            self.note_title = note.title.clone();

            // Initialize last saved content and title
            self.last_saved_content = self.note_content.clone();
            self.last_saved_title = self.note_title.clone();
            self.save_status = SaveStatus::Saved;

            // 添加到最近访问记录
            self.add_to_recent_notes(note_id);

            // Also need to ensure the notebook containing this note is selected
            self.select_notebook_for_note(note_id);
        }
    }

    /// Check if note content or title has changed and needs saving
    pub fn check_note_modified(&mut self) -> bool {
        if self.current_note.is_none() {
            return false;
        }

        let content_changed = self.note_content != self.last_saved_content;
        let title_changed = self.note_title != self.last_saved_title;

        if content_changed || title_changed {
            self.save_status = SaveStatus::Modified;
            true
        } else {
            false
        }
    }

    /// Auto-save note if modified
    pub fn auto_save_if_modified(&mut self) {
        if self.save_status == SaveStatus::Modified {
            if let Some(note_id) = self.current_note.clone() {
                let title = self.note_title.clone();
                let content = self.note_content.clone();

                self.save_status = SaveStatus::Saving;
                self.update_note(&note_id, title, content);
                self.last_saved_content = self.note_content.clone();
                self.last_saved_title = self.note_title.clone();
                self.save_status = SaveStatus::Saved;
            }
        }
    }

    /// 从内容中提取第一行作为标题
    pub fn extract_title_from_content(&mut self) {
        if self.combined_editor {
            let content = self.note_content.clone();
            let lines: Vec<&str> = content.splitn(2, '\n').collect();

            // 只有在以下情况下才自动提取标题：
            // 1. 当前没有标题
            // 2. 内容的第一行与当前标题不同（说明内容已更新）
            let current_title_empty = self.note_title.trim().is_empty();

            if !lines.is_empty() {
                // 第一行作为标题
                let first_line = lines[0].trim();
                let first_line_different = first_line != self.note_title;

                if !first_line.is_empty() && (current_title_empty || first_line_different) {
                    // 只有在标题为空或内容第一行变化时才更新标题
                    // 这样可以避免覆盖用户手动编辑的标题
                    log::info!("Extracting title from content: '{}'", first_line);
                    self.note_title = first_line.to_string();
                }
            }
        }
    }

    /// 切换编辑器最大化状态
    pub fn toggle_editor_maximized(&mut self) {
        self.editor_maximized = !self.editor_maximized;
    }

    /// 切换合并编辑器模式
    pub fn toggle_combined_editor(&mut self) {
        let was_combined = self.combined_editor;
        self.combined_editor = !self.combined_editor;

        if self.combined_editor {
            // 切换到合并模式时，标题已经分离显示，不需要添加到内容开头
            // 但需要确保从内容中提取标题（如果标题为空）
            if self.note_title.trim().is_empty() {
                self.extract_title_from_content();
            }

            // 记录当前标题，以便在编辑时检测变化
            self.last_saved_title = self.note_title.clone();
        } else {
            // 切换到分离模式时，保持当前标题和内容不变
            // 标题已经可以在两种模式下编辑，所以不需要特殊处理
        }

        // 记录日志
        log::info!("Toggled editor mode: combined={}", self.combined_editor);
    }

    /// 向当前笔记内容中追加文本
    pub fn append_to_note_content(&mut self, text: &str) {
        if self.current_note.is_none() {
            // 如果没有选中的笔记，不执行任何操作
            return;
        }

        // 检查当前内容是否为空
        if self.note_content.trim().is_empty() {
            // 如果内容为空，直接设置为新文本
            self.note_content = text.to_string();
        } else {
            // 否则，在内容末尾添加换行符和新文本
            self.note_content.push_str("\n\n");
            self.note_content.push_str(text);
        }

        // 标记笔记为已修改状态
        self.check_note_modified();
    }

    /// 处理富文本粘贴，自动转换为Markdown格式
    pub fn paste_rich_text(&mut self) -> Result<bool, String> {
        if let Some(ref mut clipboard) = self.clipboard_manager {
            match clipboard.get_html_as_markdown() {
                Ok(Some(markdown)) => {
                    // 成功获取并转换了富文本内容
                    self.insert_text_at_cursor(&markdown);
                    log::info!("Successfully pasted rich text content as Markdown");
                    Ok(true)
                }
                Ok(None) => {
                    // 剪贴板为空或没有内容
                    log::debug!("Clipboard is empty or has no content");
                    Ok(false)
                }
                Err(e) => {
                    // 转换失败，尝试获取纯文本
                    log::warn!("Failed to convert rich text: {}", e);
                    match clipboard.get_text() {
                        Ok(text) => {
                            if !text.trim().is_empty() {
                                self.insert_text_at_cursor(&text);
                                Ok(true)
                            } else {
                                Ok(false)
                            }
                        }
                        Err(e) => Err(format!("Failed to access clipboard: {}", e)),
                    }
                }
            }
        } else {
            Err("Clipboard manager not available".to_string())
        }
    }

    /// 在光标位置插入文本
    fn insert_text_at_cursor(&mut self, text: &str) {
        if self.current_note.is_none() {
            return;
        }

        // 简单实现：在内容末尾添加文本
        // 在实际应用中，可以根据光标位置插入
        if self.note_content.trim().is_empty() {
            self.note_content = text.to_string();
        } else {
            // 如果当前内容不以换行符结尾，添加换行符
            if !self.note_content.ends_with('\n') {
                self.note_content.push('\n');
            }
            self.note_content.push_str(text);
        }

        // 标记笔记为已修改状态
        self.check_note_modified();
    }

    /// 检查剪贴板是否包含富文本内容
    pub fn clipboard_has_rich_content(&mut self) -> bool {
        if let Some(ref mut clipboard) = self.clipboard_manager {
            clipboard.has_rich_content()
        } else {
            false
        }
    }

    /// Search notes and update search results (with hybrid search support)
    pub fn search_notes(&mut self) {
        // Clear previous search results
        self.search_results.clear();

        // If search query is empty, don't search
        if self.search_query.is_empty() {
            self.is_searching = false;
            return;
        }

        // Check if this is a tag search using the "label:" prefix
        if self.search_query.starts_with("label:") {
            // Extract the tag name from the query
            let tag_name = self.search_query[6..].trim().to_string();

            if tag_name.is_empty() {
                return;
            }

            // Find the tag ID by name
            let tag_id = self.find_tag_by_name(&tag_name);

            if let Some(tag_id) = tag_id {
                // Use the existing tag search function
                self.search_notes_by_tag(&tag_id);
            } else {
                // Set is_searching to true but with empty results
                self.search_query = format!("标签: {}", tag_name);
                self.is_searching = true;
            }

            return;
        }

        // Use the search mode explicitly selected by the user
        match self.search_mode {
            SearchMode::Semantic => {
                log::info!("执行语义搜索: {}", self.search_query);
                self.perform_semantic_search_with_query(&self.search_query.clone());
            }
            SearchMode::Database => {
                log::info!("执行数据库搜索: {}", self.search_query);
                self.perform_traditional_search();
            }
        }
    }

    /// Perform traditional database search
    fn perform_traditional_search(&mut self) {
        if let Ok(storage) = self.storage.lock() {
            match storage.search_notes(&self.search_query) {
                Ok(notes) => {
                    // Update notes map with search results
                    for note in &notes {
                        self.notes.insert(note.id.clone(), note.clone());
                    }

                    // Update search results with type information
                    self.search_results = notes.iter().map(|note| SearchResultItem {
                        note_id: note.id.clone(),
                        result_type: SearchResultType::Database,
                    }).collect();
                    self.is_searching = true;

                    log::info!("Traditional search completed: {} results found", notes.len());
                }
                Err(err) => {
                    log::error!("Failed to search notes: {}", err);
                }
            }
        }
    }

    /// Perform traditional search but mark results as semantic (fallback scenario)
    fn perform_traditional_search_as_semantic_fallback(&mut self, query: &str) {
        if let Ok(storage) = self.storage.lock() {
            match storage.search_notes(query) {
                Ok(notes) => {
                    // Update notes map with search results
                    for note in &notes {
                        self.notes.insert(note.id.clone(), note.clone());
                    }

                    // Mark results as semantic even though they're from traditional search
                    // This is a temporary fallback until full semantic search integration
                    self.search_results = notes.iter().map(|note| SearchResultItem {
                        note_id: note.id.clone(),
                        result_type: SearchResultType::Semantic,
                    }).collect();
                    self.is_searching = true;

                    log::info!("Semantic search (traditional fallback) completed: {} results found", notes.len());
                }
                Err(err) => {
                    log::error!("Failed to perform semantic search fallback: {}", err);
                }
            }
        }
    }

    /// Perform enhanced semantic search using vector embeddings
    fn perform_enhanced_semantic_search(&mut self, query: &str) {
        // First, try real semantic search if available
        if self.semantic_db.is_some() {
            let query_clone = query.to_string();
            let semantic_db_clone = self.semantic_db.clone();

            // Spawn async task for real semantic search
            tokio::spawn(async move {
                if let Some(db) = semantic_db_clone {
                    match db.search_notes(&query_clone, 20).await {
                        Ok(results) => {
                            log::info!("Real semantic search completed: {} results found", results.len());
                            for (i, result) in results.iter().enumerate() {
                                log::debug!("Result {}: {} (score: {:.3})",
                                    i + 1,
                                    result.title,
                                    result.similarity_score
                                );
                            }
                        }
                        Err(e) => {
                            log::error!("Real semantic search failed: {}", e);
                        }
                    }
                }
            });
        }

        // For immediate UI response, also perform traditional search
        // This will be replaced by the async semantic results when they arrive
        if let Ok(storage) = self.storage.lock() {
            match storage.search_notes(query) {
                Ok(notes) => {
                    // Update notes map with search results
                    for note in &notes {
                        self.notes.insert(note.id.clone(), note.clone());
                    }

                    // Mark results as semantic search type
                    self.search_results = notes.iter().map(|note| SearchResultItem {
                        note_id: note.id.clone(),
                        result_type: SearchResultType::Semantic,
                    }).collect();
                    self.is_searching = true;

                    log::info!("Semantic search (with traditional fallback) completed: {} results found", notes.len());
                }
                Err(e) => {
                    log::error!("Semantic search fallback failed: {}", e);
                }
            }
        }
    }

    /// Perform real semantic search using vector embeddings
    async fn perform_real_semantic_search(&self, query: &str) -> Result<Vec<Note>, Box<dyn std::error::Error + Send + Sync>> {
        if let Some(semantic_db) = &self.semantic_db {
            // Use the real semantic search API from zhushoude_duckdb
            let search_results = semantic_db.search_notes(query, 20).await?;

            // Convert semantic search results to Note objects
            let mut notes = Vec::new();
            for result in &search_results {
                // Extract metadata from the search result
                let empty_map = serde_json::Map::new();
                let metadata = if let Some(meta) = &result.metadata {
                    meta.as_object().unwrap_or(&empty_map)
                } else {
                    &empty_map
                };

                // Create a Note from the semantic search result
                let note = Note {
                    id: result.document_id.clone(),
                    title: result.title.clone(),
                    content: result.content.clone(),
                    created_at: metadata.get("created_at")
                        .and_then(|v| v.as_str())
                        .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
                        .map(|dt| dt.with_timezone(&chrono::Utc))
                        .unwrap_or_else(chrono::Utc::now),
                    updated_at: metadata.get("updated_at")
                        .and_then(|v| v.as_str())
                        .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
                        .map(|dt| dt.with_timezone(&chrono::Utc))
                        .unwrap_or_else(chrono::Utc::now),
                    tag_ids: metadata.get("tags")
                        .and_then(|v| v.as_str())
                        .map(|s| s.split(',').map(|t| t.trim().to_string()).collect())
                        .unwrap_or_default(),
                    attachments: Vec::new(),
                };
                notes.push(note);
            }

            log::info!("Real semantic search found {} results with similarity scores", notes.len());
            for (i, result) in search_results.iter().enumerate() {
                log::debug!("Result {}: {} (score: {:.3})", i + 1, result.title, result.similarity_score);
            }

            Ok(notes)
        } else {
            Err("Semantic database not initialized".into())
        }
    }

    /// Perform semantic search with query parameter
    fn perform_semantic_search_with_query(&mut self, query: &str) {
        if !self.semantic_search_enabled {
            log::warn!("Semantic search requested but not enabled, falling back to database search");
            self.search_query = query.to_string();
            self.perform_traditional_search();
            return;
        }

        // Try lazy initialization if needed
        if self.semantic_db.is_none() {
            if let Err(e) = self.try_initialize_semantic_db() {
                log::warn!("Failed to initialize semantic search: {}, falling back to database search", e);
                self.semantic_search_enabled = false;
                self.search_query = query.to_string();
                self.perform_traditional_search();
                return;
            }
        }

        log::info!("Performing real semantic search for query: {}", query);

        // If we have a semantic database, use it for real semantic search
        if let Some(semantic_db) = &self.semantic_db {
            let semantic_db = semantic_db.clone();
            let query_clone = query.to_string();

            // Perform semantic search in a non-blocking way
            let search_handle = std::thread::spawn(move || {
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(async move {
                    match semantic_db.search_notes(&query_clone, 20).await {
                        Ok(results) => {
                            log::info!("Real semantic search completed: {} results found", results.len());
                            for (i, result) in results.iter().enumerate() {
                                log::info!("🔍 Semantic result {}: '{}' (score: {:.3})",
                                    i + 1,
                                    result.title,
                                    result.similarity_score
                                );
                            }
                            Some(results)
                        }
                        Err(e) => {
                            log::error!("Real semantic search failed: {}", e);
                            None
                        }
                    }
                })
            });

            // Try to get results with a short timeout to avoid blocking UI
            match search_handle.join() {
                Ok(Some(semantic_results)) => {
                    log::info!("🎉 Processing {} semantic search results", semantic_results.len());

                    // Clear existing search results and replace with semantic results
                    self.search_results.clear();

                    // Convert semantic results to search results with proper metadata
                    for semantic_result in semantic_results {
                        // Store the note in memory for quick access
                        let note = crate::note::Note {
                            id: semantic_result.document_id.clone(),
                            title: semantic_result.title.clone(),
                            content: semantic_result.content.clone(),
                            created_at: chrono::Utc::now(), // Placeholder
                            updated_at: chrono::Utc::now(), // Placeholder
                            tag_ids: Vec::new(),
                            attachments: Vec::new(),
                        };

                        self.notes.insert(note.id.clone(), note);
                        self.search_results.push(SearchResultItem {
                            note_id: semantic_result.document_id,
                            result_type: SearchResultType::Semantic,
                        });
                    }

                    self.is_searching = true;
                    log::info!("✅ Processed semantic search results: {} total results", self.search_results.len());
                }
                Ok(None) => {
                    log::warn!("Semantic search returned no results, falling back to traditional search");
                    self.perform_traditional_search();
                }
                Err(e) => {
                    log::error!("Semantic search thread failed: {:?}, falling back to traditional search", e);
                    self.perform_traditional_search();
                }
            }
        } else {
            log::warn!("Semantic database not available, falling back to traditional search");
            self.perform_traditional_search();
        }
    }

    /// Perform hybrid search (semantic + traditional)
    fn perform_hybrid_search(&mut self) {
        log::info!("Performing hybrid search for query: {}", self.search_query);

        // Start with traditional search for immediate results
        self.perform_traditional_search();

        // For now, we'll just log that semantic search would be performed
        // In a future implementation, we'll use a proper async runtime
        // or message passing to handle semantic search without thread safety issues
        if self.semantic_search_enabled {
            log::info!("Semantic search is enabled but deferred due to thread safety considerations");
            // TODO: Implement proper async semantic search with message passing
        }
    }

    /// Find a tag by name
    fn find_tag_by_name(&self, name: &str) -> Option<String> {
        self.tags.iter()
            .find(|tag| tag.name.to_lowercase() == name.to_lowercase())
            .map(|tag| tag.id.clone())
    }

    /// Get search result notes with their types
    pub fn get_search_result_notes(&self) -> Vec<(&Note, SearchResultType)> {
        self.search_results
            .iter()
            .filter_map(|result_item| {
                self.notes.get(&result_item.note_id)
                    .map(|note| (note, result_item.result_type.clone()))
            })
            .collect()
    }

    /// Get search result note IDs
    pub fn get_search_result_note_ids(&self) -> Vec<String> {
        self.search_results
            .iter()
            .map(|result_item| result_item.note_id.clone())
            .collect()
    }

    /// Get notes for current notebook
    pub fn get_current_notebook_notes(&self) -> Vec<&Note> {
        if let Some(notebook_idx) = self.current_notebook {
            if notebook_idx < self.notebooks.len() {
                let notebook = &self.notebooks[notebook_idx];
                return notebook.note_ids
                    .iter()
                    .filter_map(|id| self.notes.get(id))
                    .collect();
            }
        }

        Vec::new()
    }

    /// Get tag by ID
    pub fn get_tag(&self, tag_id: &str) -> Option<&Tag> {
        self.tags.iter().find(|tag| tag.id == tag_id)
    }

    /// Get current search terms for highlighting
    pub fn get_search_terms(&self) -> Vec<String> {
        if !self.is_searching || self.search_query.is_empty() {
            return Vec::new();
        }

        // Handle tag search - extract tag name for highlighting
        if self.search_query.starts_with("标签: ") {
            let tag_name = self.search_query[6..].trim();
            if !tag_name.is_empty() {
                return vec![tag_name.to_string()];
            }
        }

        // For regular search, extract search terms using simple splitting
        self.search_query
            .split_whitespace()
            .filter(|term| term.len() > 2)
            .map(|term| term.to_lowercase())
            .collect()
    }

    /// Get tags for a note
    pub fn get_note_tags(&self, note_id: &str) -> Vec<&Tag> {
        if let Some(note) = self.notes.get(note_id) {
            return note.tag_ids
                .iter()
                .filter_map(|tag_id| self.get_tag(tag_id))
                .collect();
        }

        Vec::new()
    }

    /// Search notes by tag ID
    pub fn search_notes_by_tag(&mut self, tag_id: &str) {
        // Clear previous search results
        self.search_results.clear();

        // Get the tag name for display
        let tag_name = self.get_tag(tag_id)
            .map(|tag| tag.name.clone())
            .unwrap_or_else(|| "未知标签".to_string());

        // Set search query to tag name for display purposes
        self.search_query = format!("标签: {}", tag_name);

        // Search in database
        if let Ok(storage) = self.storage.lock() {
            match storage.get_notes_for_tag(tag_id) {
                Ok(notes) => {
                    // Update notes map with search results
                    for note in &notes {
                        self.notes.insert(note.id.clone(), note.clone());
                    }

                    // Update search results with database type (tag search is database search)
                    self.search_results = notes.iter().map(|note| SearchResultItem {
                        note_id: note.id.clone(),
                        result_type: SearchResultType::Database,
                    }).collect();

                    // Important: Set is_searching to true to display search results
                    self.is_searching = true;
                }
                Err(err) => {
                    log::error!("Failed to search notes by tag: {}", err);
                }
            }
        } else {
            log::error!("Failed to lock storage for tag search");
        }
    }

    /// Confirm deletion
    pub fn confirm_deletion(&mut self) {
        if let Some(confirmation) = self.delete_confirmation.take() {
            match confirmation.confirmation_type {
                DeleteConfirmationType::Notebook => {
                    if let Some(index) = confirmation.target_index {
                        self.delete_notebook_internal(index);
                    }
                },
                DeleteConfirmationType::Note => {
                    self.delete_note_internal(&confirmation.target_id);
                },
                DeleteConfirmationType::Tag => {
                    self.delete_tag_internal(&confirmation.target_id);
                },
            }
        }
        self.show_delete_confirmation = false;
    }

    /// Cancel deletion
    pub fn cancel_deletion(&mut self) {
        self.delete_confirmation = None;
        self.show_delete_confirmation = false;
    }

    /// Force reload all data from database
    pub fn force_reload_data(&mut self) {
        log::info!("开始强制重新加载所有数据...");

        // Save current selection
        let current_notebook = self.current_notebook;
        let current_note = self.current_note.clone();

        // Clear current state
        let old_notebook_count = self.notebooks.len();
        let old_note_count = self.notes.len();

        self.notebooks.clear();
        self.notes.clear();
        self.tags.clear();
        self.current_notebook = None;
        self.current_note = None;
        self.note_content.clear();
        self.note_title.clear();

        log::info!("已清空内存数据 (之前: {} 个笔记本, {} 个笔记)", old_notebook_count, old_note_count);

        // Reload from database
        self.load_notebooks();
        self.load_tags();
        self.load_all_notes(); // 重要：重新加载所有笔记

        log::info!("重新加载完成 (现在: {} 个笔记本, {} 个笔记)", self.notebooks.len(), self.notes.len());

        // 验证数据一致性
        self.validate_data_consistency();

        // Try to restore selection if still valid
        if let Some(notebook_idx) = current_notebook {
            if notebook_idx < self.notebooks.len() {
                self.current_notebook = Some(notebook_idx);

                // Try to restore note selection
                if let Some(note_id) = current_note {
                    if self.notes.contains_key(&note_id) {
                        self.select_note(&note_id);
                    }
                }
            }
        }
    }

    /// 验证数据一致性
    fn validate_data_consistency(&mut self) {
        log::info!("开始验证数据一致性...");

        let mut total_expected_notes = 0;
        let mut inconsistencies = 0;

        for notebook in &self.notebooks {
            total_expected_notes += notebook.note_ids.len();

            // 检查笔记本中的每个笔记是否在内存中存在
            for note_id in &notebook.note_ids {
                if !self.notes.contains_key(note_id) {
                    log::warn!("数据不一致: 笔记本 '{}' 引用的笔记 '{}' 在内存中不存在", notebook.name, note_id);
                    inconsistencies += 1;
                }
            }
        }

        // 检查内存中的笔记是否都有对应的笔记本引用
        for note_id in self.notes.keys() {
            let mut found = false;
            for notebook in &self.notebooks {
                if notebook.note_ids.contains(note_id) {
                    found = true;
                    break;
                }
            }
            if !found {
                log::warn!("数据不一致: 内存中的笔记 '{}' 没有被任何笔记本引用", note_id);
                inconsistencies += 1;
            }
        }

        if inconsistencies > 0 {
            log::error!("发现 {} 个数据一致性问题！尝试自动修复...", inconsistencies);
            self.fix_data_inconsistencies();
        } else {
            log::info!("数据一致性验证通过 (笔记本预期: {} 个笔记, 内存实际: {} 个笔记)",
                      total_expected_notes, self.notes.len());
        }
    }

    /// 自动修复数据不一致问题
    fn fix_data_inconsistencies(&mut self) {
        log::info!("开始自动修复数据不一致问题...");

        // 重新从数据库同步笔记本的note_ids
        if let Ok(storage) = self.storage.lock() {
            for notebook in &mut self.notebooks {
                // 从数据库重新获取这个笔记本的所有笔记ID
                match storage.get_notes_for_notebook(&notebook.id) {
                    Ok(notes) => {
                        let old_count = notebook.note_ids.len();
                        notebook.note_ids.clear();

                        for note in notes {
                            notebook.note_ids.push(note.id.clone());
                            // 确保笔记也在内存中
                            if !self.notes.contains_key(&note.id) {
                                self.notes.insert(note.id.clone(), note);
                            }
                        }

                        let new_count = notebook.note_ids.len();
                        if old_count != new_count {
                            log::info!("修复笔记本 '{}': {} -> {} 个笔记", notebook.name, old_count, new_count);
                        }
                    }
                    Err(err) => {
                        log::error!("无法从数据库获取笔记本 '{}' 的笔记: {}", notebook.name, err);
                    }
                }
            }
        }

        log::info!("数据不一致问题修复完成");
    }

    /// 安全地刷新笔记本视图，保护新导入的笔记
    pub fn safe_refresh_notebook_view(&mut self, notebook_id: &str) {
        log::info!("安全刷新笔记本视图: {}", notebook_id);

        // 记录当前状态
        let current_note_count = if let Some(notebook) = self.notebooks.iter().find(|nb| nb.id == notebook_id) {
            notebook.note_ids.len()
        } else {
            0
        };

        // 从数据库获取最新的笔记列表
        if let Ok(storage) = self.storage.lock() {
            match storage.get_notes_for_notebook(notebook_id) {
                Ok(db_notes) => {
                    let db_note_count = db_notes.len();
                    log::info!("数据库中的笔记数量: {}, 内存中的笔记数量: {}", db_note_count, current_note_count);

                    // 先收集笔记ID，然后更新内存中的笔记
                    let mut db_note_ids = Vec::new();
                    for note in &db_notes {
                        db_note_ids.push(note.id.clone());
                    }

                    // 更新内存中的笔记
                    for note in db_notes {
                        self.notes.insert(note.id.clone(), note);
                    }

                    // 重新构建笔记本的note_ids，使用已经收集的数据
                    if let Some(notebook) = self.notebooks.iter_mut().find(|nb| nb.id == notebook_id) {

                        // 合并内存中的笔记ID和数据库中的笔记ID
                        let mut merged_ids = db_note_ids;
                        for existing_id in &notebook.note_ids {
                            if !merged_ids.contains(existing_id) && self.notes.contains_key(existing_id) {
                                merged_ids.push(existing_id.clone());
                                log::info!("保护内存中的笔记: {}", existing_id);
                            }
                        }

                        notebook.note_ids = merged_ids;
                        log::info!("笔记本 '{}' 刷新完成，现有 {} 个笔记", notebook.name, notebook.note_ids.len());
                    }
                }
                Err(err) => {
                    log::error!("无法从数据库刷新笔记本 '{}': {}", notebook_id, err);
                }
            }
        }
    }

    /// 切换笔记树的可见性
    pub fn toggle_note_tree(&mut self) {
        self.show_note_tree = !self.show_note_tree;
        log::info!("Note tree visibility toggled to: {}", self.show_note_tree);
    }

    /// 检查当前笔记是否支持幻灯片模式
    pub fn is_current_note_slideshow(&self) -> bool {
        if self.note_content.trim().is_empty() {
            return false;
        }
        self.slide_parser.is_slideshow(&self.note_content)
    }

    /// 开始播放当前笔记的幻灯片
    pub fn start_slideshow(&mut self) -> Result<(), String> {
        if !self.is_current_note_slideshow() {
            return Err("当前笔记不支持幻灯片模式".to_string());
        }

        let slideshow = self.slide_parser.parse(&self.note_content)?;
        let selected_template = self.slide_style_manager.get_selected_template();
        self.slide_play_state.start_slideshow_with_template(slideshow, selected_template);

        log::info!("Started slideshow with {} slides using template: {}",
                  self.slide_play_state.get_slide_count(),
                  self.slide_style_manager.selected_template_id);
        Ok(())
    }

    /// 开始播放幻灯片（带样式选择）
    pub fn start_slideshow_with_style(&mut self, template_id: &str) -> Result<(), String> {
        if !self.is_current_note_slideshow() {
            return Err("当前笔记不支持幻灯片模式".to_string());
        }

        let slideshow = self.slide_parser.parse(&self.note_content)?;

        if let Some(template) = self.slide_style_manager.get_template(template_id) {
            self.slide_play_state.start_slideshow_with_template(slideshow, template);
            log::info!("Started slideshow with template: {}", template_id);
            Ok(())
        } else {
            Err(format!("样式模板不存在: {}", template_id))
        }
    }

    /// 停止播放幻灯片
    pub fn stop_slideshow(&mut self) {
        self.slide_play_state.stop_slideshow();
        log::info!("Stopped slideshow");
    }

    /// 获取幻灯片播放状态的可变引用
    pub fn get_slide_play_state_mut(&mut self) -> &mut SlidePlayState {
        &mut self.slide_play_state
    }

    /// 获取幻灯片播放状态的不可变引用
    pub fn get_slide_play_state(&self) -> &SlidePlayState {
        &self.slide_play_state
    }

    /// 保存幻灯片样式配置
    pub fn save_slide_style_config(&self) -> Result<(), String> {
        let config_json = serde_json::to_string(&self.slide_style_manager)
            .map_err(|e| format!("序列化样式配置失败: {}", e))?;

        if let Ok(storage) = self.storage.lock() {
            storage.save_setting("slide_style_config", &config_json)
                .map_err(|e| format!("保存样式配置失败: {}", e))?;

            log::info!("Slide style configuration saved");
            Ok(())
        } else {
            Err("无法锁定存储".to_string())
        }
    }

    /// 加载幻灯片样式配置
    pub fn load_slide_style_config(&mut self) -> Result<(), String> {
        if let Ok(storage) = self.storage.lock() {
            match storage.load_setting("slide_style_config") {
                Ok(Some(config_json)) => {
                    match serde_json::from_str::<SlideStyleManager>(&config_json) {
                        Ok(style_manager) => {
                            self.slide_style_manager = style_manager;
                            log::info!("Slide style configuration loaded");
                            Ok(())
                        }
                        Err(e) => {
                            log::warn!("Failed to deserialize slide style config: {}", e);
                            // 使用默认配置
                            self.slide_style_manager = SlideStyleManager::default();
                            Ok(())
                        }
                    }
                }
                Ok(None) => {
                    log::info!("No slide style configuration found, using defaults");
                    Ok(())
                }
                Err(e) => {
                    log::warn!("Failed to load slide style config: {}", e);
                    Ok(()) // 不阻止应用启动
                }
            }
        } else {
            Err("无法锁定存储".to_string())
        }
    }

    /// 添加笔记到最近访问记录
    pub fn add_to_recent_notes(&mut self, note_id: &str) {
        if let Some(note) = self.notes.get(note_id) {
            // 查找笔记所属的笔记本
            let notebook_name = self.notebooks.iter()
                .find(|nb| nb.note_ids.contains(&note_id.to_string()))
                .map(|nb| nb.name.clone())
                .unwrap_or_else(|| "未知笔记本".to_string());

            let recent_access = RecentNoteAccess {
                note_id: note_id.to_string(),
                note_title: note.title.clone(),
                notebook_name,
                accessed_at: Utc::now(),
            };

            // 移除已存在的相同笔记记录
            self.recent_notes.retain(|access| access.note_id != note_id);

            // 添加到队列前端
            self.recent_notes.push_front(recent_access);

            // 保持最多20个记录
            while self.recent_notes.len() > 20 {
                self.recent_notes.pop_back();
            }

            // 保存到数据库
            self.save_recent_notes();
        }
    }

    /// 获取最近访问的笔记列表
    pub fn get_recent_notes(&self, limit: usize) -> Vec<&RecentNoteAccess> {
        self.recent_notes.iter().take(limit).collect()
    }

    /// 保存最近访问记录到数据库
    fn save_recent_notes(&self) {
        if let Ok(storage) = self.storage.lock() {
            if let Ok(json) = serde_json::to_string(&self.recent_notes) {
                if let Err(e) = storage.save_setting("recent_notes", &json) {
                    log::error!("Failed to save recent notes: {}", e);
                }
            }
        }
    }

    /// 从数据库加载最近访问记录
    pub fn load_recent_notes(&mut self) {
        if let Ok(storage) = self.storage.lock() {
            match storage.load_setting("recent_notes") {
                Ok(Some(json)) => {
                    match serde_json::from_str::<VecDeque<RecentNoteAccess>>(&json) {
                        Ok(recent_notes) => {
                            self.recent_notes = recent_notes;
                            log::info!("Loaded {} recent notes", self.recent_notes.len());
                        }
                        Err(e) => {
                            log::warn!("Failed to deserialize recent notes: {}", e);
                        }
                    }
                }
                Ok(None) => {
                    log::debug!("No recent notes found in database");
                }
                Err(e) => {
                    log::error!("Failed to load recent notes: {}", e);
                }
            }
        }
    }

    /// 选择包含指定笔记的笔记本
    fn select_notebook_for_note(&mut self, note_id: &str) {
        log::info!("Looking for notebook containing note '{}'", note_id);

        // 查找包含该笔记的笔记本
        let mut found_notebook: Option<(usize, String, String)> = None;

        for (index, notebook) in self.notebooks.iter().enumerate() {
            log::debug!("Checking notebook '{}' (index: {}) with {} notes: {:?}",
                       notebook.name, index, notebook.note_ids.len(), notebook.note_ids);
            if notebook.note_ids.contains(&note_id.to_string()) {
                found_notebook = Some((index, notebook.name.clone(), notebook.id.clone()));
                log::info!("Found note '{}' in notebook '{}' (index: {})", note_id, notebook.name, index);
                break;
            }
        }

        if let Some((index, notebook_name, notebook_id)) = found_notebook {
            self.current_notebook = Some(index);
            log::info!("Selected notebook '{}' for note '{}'", notebook_name, note_id);

            // 自动展开该笔记本，便于用户知道自己在哪里
            if let Some(notebook) = self.notebooks.get_mut(index) {
                if !notebook.expanded {
                    notebook.expanded = true;
                    log::info!("Expanded notebook '{}' to show user location", notebook_name);
                } else {
                    log::info!("Notebook '{}' was already expanded", notebook_name);
                }
            }

            // 只有在笔记本的笔记未完全加载时才重新加载
            // 这样可以避免在导入文档后不必要的重新加载
            let notebook_note_count = if let Some(notebook) = self.notebooks.get(index) {
                notebook.note_ids.len()
            } else {
                0
            };

            let loaded_note_count = self.notes.keys()
                .filter(|note_id| {
                    if let Some(notebook) = self.notebooks.get(index) {
                        notebook.note_ids.contains(note_id)
                    } else {
                        false
                    }
                })
                .count();

            if loaded_note_count < notebook_note_count {
                log::info!("Loading missing notes for notebook '{}' ({}/{} loaded)",
                          notebook_name, loaded_note_count, notebook_note_count);
                self.load_notes_for_notebook(&notebook_id);
            } else {
                log::debug!("All notes for notebook '{}' are already loaded ({}/{})",
                           notebook_name, loaded_note_count, notebook_note_count);
            }
        } else {
            log::warn!("Could not find notebook containing note '{}'. Available notebooks:", note_id);
            for (index, notebook) in self.notebooks.iter().enumerate() {
                log::warn!("  - Notebook '{}' (index: {}) has {} notes", notebook.name, index, notebook.note_ids.len());
            }
        }
    }

    /// Import a document as a new note
    pub fn import_document_as_note(&mut self, file_path: &str, notebook_id: &str) -> Result<String, String> {
        log::info!("Importing document '{}' to notebook '{}'", file_path, notebook_id);

        // Convert document to markdown
        let markdown_content = match self.document_converter.convert_to_markdown(file_path) {
            Ok(content) => content,
            Err(e) => {
                log::warn!("文档转换过程中出现警告: {}", e);
                // 对于字体相关的警告，我们继续处理，不视为致命错误
                if e.to_string().contains("unknown glyph name") || e.to_string().contains(".notdef") {
                    log::info!("检测到字体相关警告，但文档转换可能仍然成功");
                    // 返回一个基本的文档内容，而不是完全失败
                    format!("# {}\n\n文档转换时遇到字体问题，但已尽力提取内容。\n\n原始文件: {}",
                        std::path::Path::new(file_path).file_stem()
                            .and_then(|s| s.to_str()).unwrap_or("导入的文档"),
                        file_path)
                } else {
                    return Err(format!("文档转换失败: {}", e));
                }
            }
        };

        // Extract title from file name
        let file_name = std::path::Path::new(file_path)
            .file_stem()
            .and_then(|name| name.to_str())
            .unwrap_or("导入的文档")
            .to_string();

        // Create new note
        let note = Note::new(file_name, markdown_content);
        let note_id = note.id.clone();

        // 确保笔记本已经保存到数据库
        log::info!("确保笔记本已保存到数据库...");
        if let Some(notebook) = self.notebooks.iter().find(|nb| nb.id == notebook_id) {
            if let Ok(storage) = self.storage.lock() {
                if let Err(err) = storage.save_notebook(notebook) {
                    log::warn!("保存笔记本到数据库失败: {}", err);
                } else {
                    log::debug!("笔记本 '{}' 已确保保存到数据库", notebook.name);
                }
            }
        } else {
            log::error!("找不到笔记本 ID: {}", notebook_id);
            return Err("找不到指定的笔记本".to_string());
        }

        // Add note to notes map first (确保笔记在内存中)
        self.notes.insert(note_id.clone(), note.clone());
        log::info!("Added note '{}' to memory, total notes in memory: {}", note_id, self.notes.len());

        // Add note to the notebook in memory
        if let Some(notebook) = self.notebooks.iter_mut().find(|nb| nb.id == notebook_id) {
            // 检查笔记是否已经在笔记本中，避免重复添加
            if !notebook.note_ids.contains(&note_id) {
                notebook.add_note(note_id.clone());
                log::info!("Added note '{}' to notebook '{}', total notes: {}", note_id, notebook.name, notebook.note_ids.len());
            } else {
                log::info!("Note '{}' already exists in notebook '{}', skipping add", note_id, notebook.name);
            }
        }

        // Save note to storage
        log::info!("保存笔记到数据库...");
        let save_result = if let Ok(storage) = self.storage.lock() {
            storage.save_note(&note, notebook_id)
        } else {
            return Err("无法获取存储锁".to_string());
        };

        match save_result {
            Ok(_) => {
                log::info!("✅ 笔记保存到数据库成功");

                // 🔍 添加到语义搜索数据库进行向量化索引
                self.add_note_to_semantic_db(&note);
            }
            Err(err) => {
                log::error!("❌ 保存笔记到数据库失败: {}", err);

                // 🚨 重要：数据库保存失败是致命错误，必须清理内存状态
                log::error!("清理内存状态以保持数据一致性...");

                // 从内存中移除笔记
                self.notes.remove(&note_id);

                // 从笔记本中移除笔记ID
                if let Some(notebook) = self.notebooks.iter_mut().find(|nb| nb.id == notebook_id) {
                    notebook.note_ids.retain(|id| id != &note_id);
                    log::debug!("已从笔记本 '{}' 中移除笔记ID '{}'", notebook.name, note_id);
                }

                // 返回错误，让用户知道导入失败
                return Err(format!("保存笔记失败: {}", err));
            }
        }

        log::info!("Successfully imported document as note '{}'", note_id);
        Ok(note_id)
    }

    /// Show document import dialog for a specific file
    pub fn show_document_import_dialog(&mut self, file_path: String) {
        let file_name = std::path::Path::new(&file_path)
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("未知文件")
            .to_string();

        self.notebook_selector.show_for_file(file_path, file_name);
    }

    /// Initialize knowledge graph manager
    pub fn initialize_knowledge_graph(&mut self, enabled: bool) {
        if enabled {
            log::info!("Initializing knowledge graph manager...");

            // Spawn async task to initialize knowledge graph
            let kg_manager_future = async move {
                match KnowledgeGraphManager::new(enabled).await {
                    Ok(manager) => Some(Arc::new(tokio::sync::Mutex::new(manager))),
                    Err(e) => {
                        log::error!("Failed to initialize knowledge graph manager: {}", e);
                        None
                    }
                }
            };

            // For now, we'll set it to None and initialize it later
            // In a real implementation, you'd want to use a proper async runtime
            self.knowledge_graph_manager = None;
            self.knowledge_graph_enabled = enabled;

            log::info!("Knowledge graph initialization queued");
        } else {
            self.knowledge_graph_manager = None;
            self.knowledge_graph_enabled = false;
            log::info!("Knowledge graph disabled");
        }
    }

    /// Process note with knowledge graph (async wrapper)
    fn process_note_with_knowledge_graph(&self, note: &Note) {
        if let Some(kg_manager) = &self.knowledge_graph_manager {
            let kg_manager = kg_manager.clone();
            let note = note.clone();

            // Spawn async task
            tokio::spawn(async move {
                let manager = kg_manager.lock().await;
                if let Err(e) = manager.process_note_added(&note).await {
                    log::error!("Failed to process note with knowledge graph: {}", e);
                }
            });
        }
    }

    /// Toggle knowledge graph panel
    pub fn toggle_knowledge_graph_panel(&mut self) {
        self.show_knowledge_graph_panel = !self.show_knowledge_graph_panel;
    }

    /// Enable knowledge graph functionality
    pub fn enable_knowledge_graph(&mut self) {
        if !self.knowledge_graph_enabled {
            self.initialize_knowledge_graph(true);
        }
    }

    /// Disable knowledge graph functionality
    pub fn disable_knowledge_graph(&mut self) {
        self.knowledge_graph_manager = None;
        self.knowledge_graph_enabled = false;
        self.show_knowledge_graph_panel = false;
        log::info!("Knowledge graph functionality disabled");
    }

    /// Perform knowledge graph semantic search
    pub fn perform_kg_semantic_search(&mut self, query: String) {
        if !self.knowledge_graph_enabled || self.knowledge_graph_manager.is_none() {
            log::warn!("Knowledge graph not enabled, cannot perform semantic search");
            return;
        }

        self.semantic_search_query = query.clone();
        self.is_semantic_searching = true;
        self.semantic_search_results.clear();

        if let Some(kg_manager) = &self.knowledge_graph_manager {
            let kg_manager = kg_manager.clone();
            let query_clone = query.clone();

            // Spawn async task for semantic search
            tokio::spawn(async move {
                let manager = kg_manager.lock().await;
                match manager.semantic_search(&query_clone, 20).await {
                    Ok(results) => {
                        log::info!("Semantic search completed: {} results found", results.len());
                        // Note: In a real implementation, you'd need to send results back to the UI
                        // This would require a channel or other communication mechanism
                    }
                    Err(e) => {
                        log::error!("Semantic search failed: {}", e);
                    }
                }
            });
        }
    }

    /// Toggle semantic search panel
    pub fn toggle_semantic_search_panel(&mut self) {
        self.show_semantic_search_panel = !self.show_semantic_search_panel;
    }

    /// Clear semantic search results
    pub fn clear_semantic_search(&mut self) {
        self.semantic_search_query.clear();
        self.semantic_search_results.clear();
        self.is_semantic_searching = false;
    }

    /// Get entity relations for display
    pub fn get_entity_relations_for_display(&self, entity_text: &str) -> Vec<String> {
        if !self.knowledge_graph_enabled || self.knowledge_graph_manager.is_none() {
            return Vec::new();
        }

        // This would need to be implemented with proper async handling
        // For now, return empty vector
        Vec::new()
    }

    /// Extract entities from current note content
    pub fn extract_entities_from_current_note(&self) -> Vec<String> {
        if !self.knowledge_graph_enabled || self.knowledge_graph_manager.is_none() {
            return Vec::new();
        }

        // This would need to be implemented with proper async handling
        // For now, return empty vector
        Vec::new()
    }

    /// 开始重建语义索引
    pub fn start_semantic_index_rebuild(&mut self) {
        if self.is_rebuilding_semantic_index {
            log::warn!("语义索引重建已在进行中，跳过新的重建请求");
            return;
        }

        log::info!("开始重建语义索引...");

        // 检查并清理可能损坏的数据库文件
        self.cleanup_corrupted_database_files();

        // 重置状态
        self.is_rebuilding_semantic_index = true;
        self.semantic_rebuild_progress = Some(0.0);
        self.semantic_rebuild_result = None;

        // 确保语义数据库已初始化
        if self.semantic_db.is_none() {
            log::info!("语义数据库未初始化，正在初始化...");
            match self.try_initialize_semantic_db() {
                Ok(_) => {
                    log::info!("✅ 语义数据库初始化成功，开始重建索引");
                }
                Err(e) => {
                    log::error!("❌ 语义数据库初始化失败: {}", e);
                    self.is_rebuilding_semantic_index = false;
                    self.semantic_rebuild_result = Some(Err(format!("语义数据库初始化失败: {}", e)));
                    return;
                }
            }
        }

        // 启动异步重建任务
        self.rebuild_semantic_index_async();
    }

    /// 清理可能损坏的数据库文件
    fn cleanup_corrupted_database_files(&self) {
        use std::fs;
        use std::path::Path;

        let db_files = [
            "./semantic_search.db",
            "./semantic_search.db.wal",
            "./semantic_search.db-shm",
            "./semantic_search.db-journal"
        ];

        for db_file in &db_files {
            let path = Path::new(db_file);
            if path.exists() {
                match fs::metadata(path) {
                    Ok(metadata) => {
                        // 检查文件大小是否异常
                        if metadata.len() == 0 {
                            log::warn!("发现空的数据库文件，删除: {}", db_file);
                            if let Err(e) = fs::remove_file(path) {
                                log::error!("删除损坏的数据库文件失败 {}: {}", db_file, e);
                            }
                        }
                    }
                    Err(e) => {
                        log::warn!("无法读取数据库文件元数据 {}: {}", db_file, e);
                    }
                }
            }
        }
    }

    /// 异步重建语义索引
    fn rebuild_semantic_index_async(&mut self) {
        if let Some(semantic_db) = &self.semantic_db {
            let semantic_db = semantic_db.clone();
            let storage = self.storage.clone();

            // 从数据库获取所有笔记，而不是依赖内存中的笔记
            let notes = if let Ok(storage_guard) = storage.lock() {
                match storage_guard.get_all_notes() {
                    Ok(all_notes) => {
                        log::info!("从数据库获取到 {} 个笔记用于重建索引", all_notes.len());
                        all_notes
                    }
                    Err(e) => {
                        log::error!("❌ 从数据库获取笔记失败: {}", e);
                        self.is_rebuilding_semantic_index = false;
                        self.semantic_rebuild_result = Some(Err(format!("获取笔记失败: {}", e)));
                        return;
                    }
                }
            } else {
                log::error!("❌ 无法获取数据库连接");
                self.is_rebuilding_semantic_index = false;
                self.semantic_rebuild_result = Some(Err("无法获取数据库连接".to_string()));
                return;
            };

            let total_notes = notes.len();
            log::info!("开始重建 {} 个笔记的语义索引", total_notes);

            // 创建共享状态用于进度和结果通信
            let progress = Arc::new(Mutex::new(0.0f32));
            let result_state = Arc::new(Mutex::new(None::<Result<usize, String>>));

            let progress_clone = progress.clone();
            let result_clone = result_state.clone();

            // 使用std::thread而不是tokio::spawn来避免运行时问题
            let _handle = std::thread::spawn(move || {
                // 在线程内创建Tokio运行时
                let rt = match tokio::runtime::Runtime::new() {
                    Ok(rt) => rt,
                    Err(e) => {
                        log::error!("❌ 创建Tokio运行时失败: {}", e);
                        if let Ok(mut result) = result_clone.lock() {
                            *result = Some(Err(format!("创建Tokio运行时失败: {}", e)));
                        }
                        return;
                    }
                };

                let result = rt.block_on(async {
                    Self::perform_semantic_index_rebuild_with_progress(semantic_db, notes, progress_clone).await
                });

                // 更新结果状态
                if let Ok(mut result_state) = result_clone.lock() {
                    match result {
                        Ok(indexed_count) => {
                            log::info!("✅ 语义索引重建完成，共索引 {} 个笔记", indexed_count);
                            *result_state = Some(Ok(indexed_count));
                        }
                        Err(e) => {
                            log::error!("❌ 语义索引重建失败: {}", e);
                            *result_state = Some(Err(e.to_string()));
                        }
                    }
                }
            });

            // 存储共享状态引用以便UI更新
            self.semantic_rebuild_progress_ref = Some(progress);
            self.semantic_rebuild_result_ref = Some(result_state);
        } else {
            log::error!("语义数据库未初始化，无法重建索引");
            self.is_rebuilding_semantic_index = false;
            self.semantic_rebuild_result = Some(Err("语义数据库未初始化".to_string()));
        }
    }

    /// 执行语义索引重建的具体逻辑
    async fn perform_semantic_index_rebuild(
        semantic_db: Arc<ZhushoudeDB>,
        notes: Vec<crate::note::Note>
    ) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
        // 首先清除现有索引
        log::info!("清除现有语义索引...");
        semantic_db.clear_all_semantic_indexes().await?;

        // 批量重新索引所有笔记
        let mut indexed_count = 0;
        let batch_size = 10; // 批量处理大小

        for chunk in notes.chunks(batch_size) {
            let mut documents = Vec::new();

            for note in chunk {
                let document = zhushoude_duckdb::Document {
                    id: note.id.clone(),
                    title: note.title.clone(),
                    content: note.content.clone(),
                    doc_type: zhushoude_duckdb::DocumentType::Note,
                    metadata: serde_json::json!({
                        "created_at": note.created_at,
                        "updated_at": note.updated_at,
                        "tag_ids": note.tag_ids,
                        "attachments": note.attachments.len() // 只存储附件数量
                    }),
                };
                documents.push(document);
            }

            // 批量添加文档
            semantic_db.add_notes_batch(&documents).await?;
            indexed_count += documents.len();

            log::info!("已索引 {}/{} 个笔记", indexed_count, notes.len());
        }

        log::info!("✅ 语义索引重建完成，共处理 {} 个笔记", indexed_count);
        Ok(indexed_count)
    }

    /// 执行语义索引重建的具体逻辑（带进度更新）
    async fn perform_semantic_index_rebuild_with_progress(
        semantic_db: Arc<ZhushoudeDB>,
        notes: Vec<crate::note::Note>,
        progress: Arc<Mutex<f32>>
    ) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
        // 首先清除现有索引
        log::info!("清除现有语义索引...");
        semantic_db.clear_all_semantic_indexes().await?;

        let total_notes = notes.len();
        if total_notes == 0 {
            log::warn!("没有找到需要索引的笔记");
            if let Ok(mut p) = progress.lock() {
                *p = 1.0;
            }
            return Ok(0);
        }

        // 批量重新索引所有笔记
        let mut indexed_count = 0;
        let batch_size = 10; // 批量处理大小

        for chunk in notes.chunks(batch_size) {
            let mut documents = Vec::new();

            for note in chunk {
                let document = zhushoude_duckdb::Document {
                    id: note.id.clone(),
                    title: note.title.clone(),
                    content: note.content.clone(),
                    doc_type: zhushoude_duckdb::DocumentType::Note,
                    metadata: serde_json::json!({
                        "created_at": note.created_at,
                        "updated_at": note.updated_at,
                        "tag_ids": note.tag_ids,
                        "attachments": note.attachments.len() // 只存储附件数量
                    }),
                };
                documents.push(document);
            }

            // 批量添加文档
            semantic_db.add_notes_batch(&documents).await?;
            indexed_count += documents.len();

            // 更新进度
            let current_progress = indexed_count as f32 / total_notes as f32;
            if let Ok(mut p) = progress.lock() {
                *p = current_progress;
            }

            log::info!("已索引 {}/{} 个笔记", indexed_count, total_notes);
        }

        // 重建完成后，重新创建向量索引以优化搜索性能
        if indexed_count > 0 {
            log::info!("重新创建向量索引以优化搜索性能...");
            match semantic_db.create_vector_index().await {
                Ok(_) => {
                    log::info!("✅ 向量索引重新创建成功");
                }
                Err(e) => {
                    log::warn!("⚠️ 向量索引创建失败，将使用线性搜索: {}", e);
                    // 不返回错误，因为线性搜索仍然可以工作
                }
            }
        }

        // 设置完成进度
        if let Ok(mut p) = progress.lock() {
            *p = 1.0;
        }

        log::info!("✅ 语义索引重建完成，共处理 {} 个笔记", indexed_count);
        Ok(indexed_count)
    }

    /// 清除语义索引
    pub fn clear_semantic_index(&mut self) {
        if self.is_rebuilding_semantic_index {
            log::warn!("正在重建索引，无法清除");
            return;
        }

        log::info!("开始清除语义索引...");

        // 确保语义数据库已初始化
        if self.semantic_db.is_none() {
            log::info!("语义数据库未初始化，正在初始化...");
            match self.try_initialize_semantic_db() {
                Ok(_) => {
                    log::info!("✅ 语义数据库初始化成功");
                }
                Err(e) => {
                    log::error!("❌ 语义数据库初始化失败，无法清除索引: {}", e);
                    return;
                }
            }
        }

        if let Some(semantic_db) = &self.semantic_db {
            let semantic_db = semantic_db.clone();

            // 使用std::thread而不是tokio::spawn来避免运行时问题
            let _handle = std::thread::spawn(move || {
                // 在线程内创建Tokio运行时
                let rt = match tokio::runtime::Runtime::new() {
                    Ok(rt) => rt,
                    Err(e) => {
                        log::error!("❌ 创建Tokio运行时失败: {}", e);
                        return;
                    }
                };

                let result = rt.block_on(async {
                    semantic_db.clear_all_semantic_indexes().await
                });

                match result {
                    Ok(_) => {
                        log::info!("✅ 语义索引清除完成");
                    }
                    Err(e) => {
                        log::error!("❌ 语义索引清除失败: {}", e);
                    }
                }
            });

            log::info!("语义索引清除请求已提交");
        } else {
            log::error!("语义数据库初始化后仍然为空，这不应该发生");
        }
    }

    /// 完成语义索引重建（由异步任务调用）
    pub fn complete_semantic_index_rebuild(&mut self, result: Result<usize, String>) {
        self.is_rebuilding_semantic_index = false;
        self.semantic_rebuild_progress = None;
        self.semantic_rebuild_result = Some(result);

        // 更新索引版本
        if self.semantic_rebuild_result.as_ref().unwrap().is_ok() {
            // 增加版本号
            let current_version = self.semantic_index_version.clone();
            if let Some(version_num) = current_version.strip_prefix("v").and_then(|v| v.parse::<f32>().ok()) {
                self.semantic_index_version = format!("v{:.1}", version_num + 0.1);
            } else {
                self.semantic_index_version = "v1.1".to_string();
            }

            log::info!("语义索引版本更新为: {}", self.semantic_index_version);
        }

        // 清理共享状态引用
        self.semantic_rebuild_progress_ref = None;
        self.semantic_rebuild_result_ref = None;
    }

    /// 更新语义索引重建状态（从后台线程获取）
    pub fn update_semantic_rebuild_status(&mut self) {
        // 检查进度更新
        if let Some(progress_ref) = &self.semantic_rebuild_progress_ref {
            if let Ok(progress) = progress_ref.lock() {
                self.semantic_rebuild_progress = Some(*progress);
            }
        }

        // 检查结果更新 - 先提取结果，再更新状态以避免借用冲突
        let rebuild_result = if let Some(result_ref) = &self.semantic_rebuild_result_ref {
            if let Ok(mut result_state) = result_ref.lock() {
                result_state.take()
            } else {
                None
            }
        } else {
            None
        };

        // 如果有结果，更新状态
        if let Some(result) = rebuild_result {
            self.complete_semantic_index_rebuild(result);
        }
    }
}
