use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};
use chrono::{DateTime, Utc};
use log;

use crate::notebook::Notebook;
use crate::note::Note;
use crate::tag::Tag;
use crate::db_storage::DbStorageManager;
use crate::clipboard::ClipboardManager;
use crate::db_ui_import::SiyuanImportState;
use crate::slide::{SlidePlayState, SlideParser, SlideStyleManager};
use crate::document_converter::DocumentConverter;
use crate::notebook_selector::NotebookSelectorState;

/// 笔记排序方式
#[derive(Debug, Clone, PartialEq)]
pub enum NoteSortBy {
    /// 按添加顺序（最新添加的在前）
    CreatedTime,
    /// 按更新时间（最近更新的在前）
    UpdatedTime,
}

/// 笔记视图模式
#[derive(Debug, Clone, PartialEq)]
pub enum NoteViewMode {
    /// 树状视图 - 按笔记本分组显示
    TreeView,
    /// 时间视图 - 所有笔记按时间排序的扁平列表
    TimeView,
}

/// 删除确认类型
#[derive(Debug, Clone, PartialEq)]
pub enum DeleteConfirmationType {
    Note,
    Notebook,
    Tag,
}

/// 删除确认信息
#[derive(Debug, Clone)]
pub struct DeleteConfirmation {
    pub confirmation_type: DeleteConfirmationType,
    pub item_id: String,
    pub item_name: String,
}

/// 保存状态
#[derive(Debug, Clone, PartialEq)]
pub enum SaveStatus {
    Saved,
    Modified,
    Saving,
    Error(String),
}

/// 最近访问的笔记
#[derive(Debug, Clone)]
pub struct RecentNoteAccess {
    pub note_id: String,
    pub note_title: String,
    pub accessed_at: DateTime<Utc>,
}

/// iNote state with SQLite storage
pub struct DbINoteState {
    pub notebooks: Vec<Notebook>,
    pub notes: HashMap<String, Note>,
    pub tags: Vec<Tag>,
    pub current_notebook: Option<usize>,
    pub current_note: Option<String>,
    pub search_query: String,
    pub search_results: Vec<String>,    // Search results (note IDs)
    pub is_searching: bool,          // Whether we're currently showing search results
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
    pub image_storage: crate::image_storage::ImageStorageManager, // Image storage manager
    pub markdown_preview: bool,      // Whether to show markdown preview instead of editor
    pub last_saved_content: String,  // Last saved content for auto-save comparison
    pub last_saved_title: String,    // Last saved title for auto-save comparison
    pub save_status: SaveStatus,     // Current save status
    pub editor_maximized: bool,      // Whether the editor is maximized
    pub combined_editor: bool,       // Whether title and content are combined in editor
    pub siyuan_import: SiyuanImportState, // 思源笔记导入状态
    pub show_markdown_help: bool,    // Whether to show the Markdown help popup
    pub show_search_help: bool,      // Whether to show the search help popup
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

    // 笔记排序设置
    pub note_sort_by: NoteSortBy,                  // 笔记排序方式
    pub note_view_mode: NoteViewMode,              // 笔记视图模式

    // 全局笔记列表（用于时间视图）
    pub global_notes: Vec<String>,                 // 全局笔记ID列表，按时间排序

    // 高级功能设置
    pub settings_enable_plugin_system: bool,       // 启用插件系统
    pub settings_enable_ai_integration: bool,      // 启用AI集成
    pub settings_enable_collaboration: bool,       // 启用协作功能

    // UI 布局设置
    pub show_note_tree: bool,                      // 是否显示笔记树

    // 最近访问的笔记
    pub recent_notes: VecDeque<RecentNoteAccess>,  // 最近访问的笔记列表，最多保存20个

    // 文档导入功能
    pub document_converter: DocumentConverter,     // 文档转换器
    pub notebook_selector: NotebookSelectorState,  // 笔记本选择对话框状态
    pub show_document_import_dialog: bool,         // 显示文档导入对话框

    // 性能优化相关
    pub note_content_cache: std::collections::HashMap<String, String>, // 笔记内容缓存
    pub large_note_threshold: usize,               // 大笔记阈值（字节）
    pub is_loading_note: bool,                     // 是否正在加载笔记
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
            image_storage: crate::image_storage::ImageStorageManager::default(),
            markdown_preview: false,
            last_saved_content: String::new(),
            last_saved_title: String::new(),
            save_status: SaveStatus::Saved,
            editor_maximized: false,
            combined_editor: false,  // 默认使用分离标题模式
            siyuan_import: SiyuanImportState::default(),
            show_markdown_help: false,
            show_search_help: false,
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

            // 笔记排序设置默认值
            note_sort_by: NoteSortBy::CreatedTime,       // 默认按添加顺序排序
            note_view_mode: NoteViewMode::TreeView,      // 默认树状视图

            // 全局笔记列表
            global_notes: Vec::new(),                    // 初始为空

            // 高级功能设置默认值
            settings_enable_plugin_system: false,       // 默认禁用插件系统
            settings_enable_ai_integration: true,       // 默认启用AI集成
            settings_enable_collaboration: false,       // 默认禁用协作功能

            // UI 布局设置默认值
            show_note_tree: true,                       // 默认显示笔记树

            // 最近访问笔记初始化
            recent_notes: VecDeque::new(),

            // 文档导入功能初始化
            document_converter: DocumentConverter::new(),
            notebook_selector: NotebookSelectorState::default(),
            show_document_import_dialog: false,

            // 性能优化初始化
            note_content_cache: std::collections::HashMap::new(),
            large_note_threshold: 1024 * 1024, // 1MB
            is_loading_note: false,
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

    /// Search notes and update search results
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

        // Perform traditional database search
        log::info!("执行数据库搜索: {}", self.search_query);
        self.perform_traditional_search();
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

                    // Update search results
                    self.search_results = notes.iter().map(|note| note.id.clone()).collect();
                    self.is_searching = true;

                    log::info!("Traditional search completed: {} results found", notes.len());
                }
                Err(err) => {
                    log::error!("Failed to search notes: {}", err);
                }
            }
        }
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

        if let Ok(storage) = self.storage.lock() {
            match storage.get_notes_for_tag(tag_id) {
                Ok(notes) => {
                    // Update notes map with search results
                    for note in &notes {
                        self.notes.insert(note.id.clone(), note.clone());
                    }

                    // Update search results
                    self.search_results = notes.iter().map(|note| note.id.clone()).collect();
                    self.is_searching = true;

                    log::info!("Tag search completed: {} results found for tag '{}'", notes.len(), tag_name);
                }
                Err(err) => {
                    log::error!("Failed to search notes by tag: {}", err);
                }
            }
        }
    }

    /// Find tag by name
    fn find_tag_by_name(&self, tag_name: &str) -> Option<String> {
        self.tags.iter()
            .find(|tag| tag.name.eq_ignore_ascii_case(tag_name))
            .map(|tag| tag.id.clone())
    }

    /// Get tag by ID
    pub fn get_tag(&self, tag_id: &str) -> Option<&Tag> {
        self.tags.iter().find(|tag| tag.id == tag_id)
    }

    /// Load slide style configuration
    fn load_slide_style_config(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Implementation for loading slide style config
        Ok(())
    }

    /// Initialize database storage asynchronously
    pub fn initialize_storage_async(&mut self, _db_path: String) {
        log::info!("Initializing database storage asynchronously...");

        let storage_clone = self.storage.clone();

        std::thread::spawn(move || {
            match DbStorageManager::new() {
                Ok(storage_manager) => {
                    if let Ok(mut storage) = storage_clone.lock() {
                        *storage = storage_manager;
                        log::info!("✅ Database storage initialized successfully");
                    }
                }
                Err(e) => {
                    log::error!("❌ Failed to initialize database storage: {}", e);
                }
            }
        });
    }

    /// Load data on demand (called when UI needs data)
    pub fn load_data_on_demand(&mut self) {
        if self.notebooks.is_empty() {
            log::info!("Loading data on demand...");

            // Load notebooks
            if let Ok(storage) = self.storage.lock() {
                match storage.list_notebooks() {
                    Ok(notebooks) => {
                        self.notebooks = notebooks;
                        log::info!("Loaded {} notebooks", self.notebooks.len());
                    }
                    Err(e) => {
                        log::error!("Failed to load notebooks: {}", e);
                    }
                }

                // Load tags
                match storage.list_tags() {
                    Ok(tags) => {
                        self.tags = tags;
                        log::info!("Loaded {} tags", self.tags.len());
                    }
                    Err(e) => {
                        log::error!("Failed to load tags: {}", e);
                    }
                }
            }

            // Apply notebook collapse setting
            self.apply_notebook_collapse_setting();

            // Load recent notes
            if let Ok(storage) = self.storage.lock() {
                match storage.load_recent_notes(20) {
                    Ok(recent_notes) => {
                        self.recent_notes.clear();
                        for note in recent_notes {
                            self.recent_notes.push_back(note);
                        }
                        log::info!("Loaded {} recent notes from database", self.recent_notes.len());
                    }
                    Err(e) => {
                        log::error!("Failed to load recent notes from database: {}", e);
                    }
                }
            }

            log::info!("Data loaded on demand: {} notebooks", self.notebooks.len());
        }
    }

    /// Apply notebook collapse setting
    fn apply_notebook_collapse_setting(&mut self) {
        if self.settings_default_collapse_notebooks {
            // Implementation for collapsing notebooks
        }
    }

    /// Create a new note
    pub fn create_note(&mut self, title: String, content: String) -> Option<String> {
        if let Some(notebook_idx) = self.current_notebook {
            if notebook_idx < self.notebooks.len() {
                let notebook_id = self.notebooks[notebook_idx].id.clone();

                // Create a new note
                let note = crate::note::Note {
                    id: uuid::Uuid::new_v4().to_string(),
                    title: title.clone(),
                    content: content.clone(),
                    created_at: chrono::Utc::now(),
                    updated_at: chrono::Utc::now(),
                    tag_ids: Vec::new(),
                    attachments: Vec::new(),
                };

                if let Ok(storage) = self.storage.lock() {
                    match storage.save_note(&note, &notebook_id) {
                        Ok(_) => {
                            let note_id = note.id.clone();

                            // 更新内存中的笔记
                            self.notes.insert(note_id.clone(), note);

                            // 更新笔记本的note_ids
                            self.notebooks[notebook_idx].add_note(note_id.clone());

                            // 保存更新后的笔记本到数据库
                            if let Err(e) = storage.save_notebook(&self.notebooks[notebook_idx]) {
                                log::warn!("保存更新后的笔记本失败: {}", e);
                            }

                            log::info!("Created new note: {}", title);
                            return Some(note_id);
                        }
                        Err(e) => {
                            log::error!("Failed to create note: {}", e);
                        }
                    }
                }
            }
        }
        None
    }

    /// Select a note
    pub fn select_note(&mut self, note_id: &str) {
        // 在切换笔记前，先保存当前笔记的修改
        if self.current_note.is_some() && self.save_status == SaveStatus::Modified {
            log::info!("Saving current note before switching to new note");
            self.auto_save_if_modified();
        }

        if let Some(note) = self.notes.get(note_id).cloned() {
            // 首先找到包含这个笔记的笔记本并选择它
            for (notebook_idx, notebook) in self.notebooks.iter().enumerate() {
                if notebook.note_ids.contains(&note_id.to_string()) {
                    self.current_notebook = Some(notebook_idx);
                    log::info!("Auto-selected notebook: {} for note: {}", notebook.name, note.title);
                    break;
                }
            }

            self.current_note = Some(note_id.to_string());

            // 检查是否是大文件
            let content_size = note.content.len();
            if content_size > self.large_note_threshold {
                log::info!("Loading large note: {} ({} bytes)", note_id, content_size);
                self.load_large_note_async(note_id, &note.title, &note.content);
            } else {
                // 小文件直接加载
                self.load_note_content_immediate(note_id, &note.content, &note.title);
            }

            log::info!("Selected note: {}", note.title);
        } else {
            log::warn!("Note not found in memory: {}. Attempting to load from database.", note_id);
            // 如果笔记不在内存中，尝试从数据库加载
            self.load_note_from_database(note_id);
        }
    }

    /// Load note content immediately (for small notes)
    fn load_note_content_immediate(&mut self, note_id: &str, content: &str, title: &str) {
        self.note_content = content.to_string();
        self.note_title = title.to_string();
        self.last_saved_content = content.to_string();
        self.last_saved_title = title.to_string();
        self.save_status = SaveStatus::Saved;
        self.is_loading_note = false;

        // 缓存内容
        self.note_content_cache.insert(note_id.to_string(), content.to_string());

        // Add to recent notes
        self.add_to_recent_notes(note_id, title);
    }

    /// Load a note from database when it's not in memory
    fn load_note_from_database(&mut self, note_id: &str) {
        // 首先从数据库加载笔记
        let note_result = {
            if let Ok(storage) = self.storage.lock() {
                storage.load_note(note_id)
            } else {
                log::error!("Failed to acquire storage lock");
                return;
            }
        };

        match note_result {
            Ok(note) => {
                log::info!("Loaded note from database: {}", note.title);

                // 找到包含这个笔记的笔记本
                for (notebook_idx, notebook) in self.notebooks.iter_mut().enumerate() {
                    if notebook.note_ids.contains(&note_id.to_string()) {
                        self.current_notebook = Some(notebook_idx);

                        // 确保笔记本已展开
                        notebook.expanded = true;

                        log::info!("Auto-selected and expanded notebook: {} for note: {}", notebook.name, note.title);
                        break;
                    }
                }

                // 将笔记添加到内存中
                self.notes.insert(note.id.clone(), note.clone());

                // 选择这个笔记
                self.current_note = Some(note.id.clone());

                // 加载内容
                let content_size = note.content.len();
                if content_size > self.large_note_threshold {
                    log::info!("Loading large note from database: {} ({} bytes)", note.id, content_size);
                    self.load_large_note_async(&note.id, &note.title, &note.content);
                } else {
                    self.load_note_content_immediate(&note.id, &note.content, &note.title);
                }
            }
            Err(e) => {
                log::error!("Failed to load note from database: {}", e);
            }
        }
    }

    /// Load large note asynchronously
    fn load_large_note_async(&mut self, note_id: &str, title: &str, content: &str) {
        // 首先检查缓存
        if let Some(cached_content) = self.note_content_cache.get(note_id).cloned() {
            log::info!("Loading large note from cache: {}", note_id);
            self.load_note_content_immediate(note_id, &cached_content, title);
            return;
        }

        // 设置加载状态
        self.is_loading_note = true;
        self.note_title = title.to_string();

        // 分块处理大文件内容
        let chunk_size = 64 * 1024; // 64KB chunks
        if content.len() > chunk_size {
            log::info!("Processing large note in chunks: {} bytes", content.len());

            // 先显示前面的内容
            let preview_content = format!("{}\n\n[正在加载剩余内容...]", &content[..chunk_size]);
            self.note_content = preview_content;

            // 立即加载完整内容（优化后的版本）
            self.load_note_content_immediate(note_id, content, title);
        } else {
            self.load_note_content_immediate(note_id, content, title);
        }
    }

    /// Add note to recent notes list
    fn add_to_recent_notes(&mut self, note_id: &str, note_title: &str) {
        let accessed_at = Utc::now();
        let access = RecentNoteAccess {
            note_id: note_id.to_string(),
            note_title: note_title.to_string(),
            accessed_at,
        };

        // Remove existing entry if present
        self.recent_notes.retain(|item| item.note_id != note_id);

        // Add to front
        self.recent_notes.push_front(access);

        // Keep only last 20 items
        while self.recent_notes.len() > 20 {
            self.recent_notes.pop_back();
        }

        // Save to database
        if let Ok(storage) = self.storage.lock() {
            if let Err(e) = storage.save_recent_note(note_id, note_title, &accessed_at) {
                log::error!("Failed to save recent note to database: {}", e);
            }
        }
    }

    /// Toggle note tree visibility
    pub fn toggle_note_tree(&mut self) {
        self.show_note_tree = !self.show_note_tree;
    }

    /// Find notebook for note
    pub fn find_notebook_for_note(&self, _note_id: &str) -> Option<String> {
        // For now, return the first notebook's ID if available
        self.notebooks.first().map(|nb| nb.id.clone())
    }

    /// Ensure data is loaded
    pub fn ensure_data_loaded(&mut self) {
        self.load_data_on_demand();
    }

    /// Get current notebook notes
    pub fn get_current_notebook_notes(&self) -> Vec<(String, String)> {
        if let Some(notebook_idx) = self.current_notebook {
            if let Some(notebook) = self.notebooks.get(notebook_idx) {
                return notebook.note_ids.iter()
                    .filter_map(|note_id| {
                        self.notes.get(note_id).map(|note| (note_id.clone(), note.title.clone()))
                    })
                    .collect();
            }
        }
        Vec::new()
    }

    /// Select notebook
    pub fn select_notebook(&mut self, notebook_idx: usize) {
        self.current_notebook = Some(notebook_idx);
        self.current_note = None;
    }

    /// Load notes for notebook
    pub fn load_notes_for_notebook(&mut self, notebook_id: &str) -> Result<(), String> {
        if let Ok(storage) = self.storage.lock() {
            match storage.get_notes_for_notebook_sorted(notebook_id, &self.note_sort_by) {
                Ok(notes) => {
                    log::debug!("重新加载笔记本 {} 的 {} 个笔记", notebook_id, notes.len());

                    // 更新笔记本的note_ids以反映新的排序
                    if let Some(notebook) = self.notebooks.iter_mut().find(|nb| nb.id == notebook_id) {
                        notebook.note_ids.clear();
                        for note in &notes {
                            notebook.note_ids.push(note.id.clone());
                        }
                    }

                    // 更新notes HashMap
                    for note in notes {
                        self.notes.insert(note.id.clone(), note);
                    }
                    Ok(())
                },
                Err(e) => {
                    log::error!("从数据库加载笔记本笔记失败: {}", e);
                    Err(format!("从数据库加载笔记本笔记失败: {}", e))
                }
            }
        } else {
            Err("无法获取数据库连接".to_string())
        }
    }

    /// Auto save if modified
    pub fn auto_save_if_modified(&mut self) {
        if self.note_content != self.last_saved_content || self.note_title != self.last_saved_title {
            if let Some(note_id) = self.current_note.clone() {
                log::info!("Auto-saving note: {} (title: '{}', content length: {})",
                          note_id, self.note_title, self.note_content.len());

                self.save_status = SaveStatus::Saving;

                // 实际保存到数据库
                let title = self.note_title.clone();
                let content = self.note_content.clone();
                self.update_note(&note_id, title, content);

                // Update cache when saving
                self.note_content_cache.insert(note_id.clone(), self.note_content.clone());

                // 更新保存状态
                self.last_saved_content = self.note_content.clone();
                self.last_saved_title = self.note_title.clone();
                self.save_status = SaveStatus::Saved;

                log::info!("Auto-save completed for note: {}", note_id);
            } else {
                log::warn!("Auto-save triggered but no current note selected");
            }
        }
    }

    /// Update a note
    pub fn update_note(&mut self, note_id: &str, title: String, content: String) {
        // 先获取笔记本ID，避免借用冲突
        let notebook_id = self.get_notebook_id_for_note(note_id);

        if let Some(note) = self.notes.get_mut(note_id) {
            note.title = title;
            note.content = content;
            note.updated_at = chrono::Utc::now();

            // 保存到数据库
            if let Ok(storage) = self.storage.lock() {
                if let Some(nb_id) = notebook_id {
                    if let Err(err) = storage.save_note(note, &nb_id) {
                        log::error!("Failed to save note to database: {}", err);
                        self.save_status = SaveStatus::Error(err.to_string());
                        return;
                    }
                } else {
                    log::error!("Cannot find notebook for note: {}", note_id);
                    self.save_status = SaveStatus::Error("Cannot find notebook for note".to_string());
                    return;
                }
            } else {
                log::error!("Cannot get database connection");
                self.save_status = SaveStatus::Error("Cannot get database connection".to_string());
                return;
            }

            log::debug!("Successfully updated note: {} in database", note_id);
        } else {
            log::error!("Note not found in memory: {}", note_id);
            self.save_status = SaveStatus::Error("Note not found in memory".to_string());
        }
    }

    /// Get notebook ID for a note
    fn get_notebook_id_for_note(&self, note_id: &str) -> Option<String> {
        for notebook in &self.notebooks {
            if notebook.note_ids.contains(&note_id.to_string()) {
                return Some(notebook.id.clone());
            }
        }
        None
    }

    /// Check if note is modified
    pub fn check_note_modified(&mut self) {
        if self.note_content != self.last_saved_content || self.note_title != self.last_saved_title {
            self.save_status = SaveStatus::Modified;
        } else {
            self.save_status = SaveStatus::Saved;
        }
    }

    /// Toggle editor maximized
    pub fn toggle_editor_maximized(&mut self) {
        self.editor_maximized = !self.editor_maximized;
    }

    /// Delete note
    pub fn delete_note(&mut self, note_id: &str) {
        // 从所有笔记本中移除该笔记
        for notebook in &mut self.notebooks {
            if notebook.note_ids.contains(&note_id.to_string()) {
                notebook.remove_note(note_id);

                // 保存更新后的笔记本到数据库
                if let Ok(storage) = self.storage.lock() {
                    if let Err(e) = storage.save_notebook(notebook) {
                        log::warn!("保存更新后的笔记本失败: {}", e);
                    }
                }
            }
        }

        // 从内存中移除笔记
        self.notes.remove(note_id);

        // 从数据库中删除笔记
        if let Ok(storage) = self.storage.lock() {
            if let Err(e) = storage.delete_note(note_id) {
                log::error!("从数据库删除笔记失败: {}", e);
            }
        }

        // 如果当前选中的是被删除的笔记，清除选择
        if self.current_note.as_ref().map_or(false, |id| id == note_id) {
            self.current_note = None;
            self.note_title.clear();
            self.note_content.clear();
            self.last_saved_title.clear();
            self.last_saved_content.clear();
            self.save_status = SaveStatus::Saved;
        }
    }

    /// Delete tag
    pub fn delete_tag(&mut self, tag_id: &str) {
        self.tags.retain(|tag| tag.id != tag_id);
        if let Ok(storage) = self.storage.lock() {
            let _ = storage.delete_tag(tag_id);
        }
    }

    /// Delete notebook
    pub fn delete_notebook(&mut self, notebook_idx: usize) {
        if notebook_idx < self.notebooks.len() {
            let notebook = self.notebooks.remove(notebook_idx);
            if let Ok(storage) = self.storage.lock() {
                let _ = storage.delete_notebook(&notebook.id);
            }
        }
    }

    /// Create notebook
    pub fn create_notebook(&mut self, name: String, description: String) -> Option<String> {
        // Calculate the next sort_order (highest current sort_order + 1)
        let next_sort_order = self.notebooks.iter()
            .map(|nb| nb.sort_order)
            .max()
            .unwrap_or(0) + 1;

        // Create a new notebook
        let notebook = crate::notebook::Notebook {
            id: uuid::Uuid::new_v4().to_string(),
            name: name.clone(),
            description: description.clone(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            note_ids: Vec::new(),
            expanded: true,
            sort_order: next_sort_order,
        };

        if let Ok(storage) = self.storage.lock() {
            match storage.save_notebook(&notebook) {
                Ok(_) => {
                    let notebook_id = notebook.id.clone();
                    self.notebooks.push(notebook);
                    log::info!("Created new notebook: {}", name);
                    return Some(notebook_id);
                }
                Err(e) => {
                    log::error!("Failed to create notebook: {}", e);
                }
            }
        }
        None
    }

    /// Move notebook up in the list
    pub fn move_notebook_up(&mut self, notebook_idx: usize) -> bool {
        if notebook_idx == 0 || notebook_idx >= self.notebooks.len() {
            return false; // Can't move up if it's already at the top or invalid index
        }

        // Swap sort_order values with the previous notebook
        let current_sort_order = self.notebooks[notebook_idx].sort_order;
        let prev_sort_order = self.notebooks[notebook_idx - 1].sort_order;

        self.notebooks[notebook_idx].sort_order = prev_sort_order;
        self.notebooks[notebook_idx - 1].sort_order = current_sort_order;

        // Update both notebooks in the database
        if let Ok(storage) = self.storage.lock() {
            if let Err(e) = storage.save_notebook(&self.notebooks[notebook_idx]) {
                log::error!("Failed to save notebook after moving up: {}", e);
                return false;
            }
            if let Err(e) = storage.save_notebook(&self.notebooks[notebook_idx - 1]) {
                log::error!("Failed to save notebook after moving up: {}", e);
                return false;
            }
        }

        // Swap the notebooks in the vector
        self.notebooks.swap(notebook_idx, notebook_idx - 1);

        // Update current_notebook index if needed
        if let Some(current) = self.current_notebook {
            if current == notebook_idx {
                self.current_notebook = Some(notebook_idx - 1);
            } else if current == notebook_idx - 1 {
                self.current_notebook = Some(notebook_idx);
            }
        }

        true
    }

    /// Move notebook down in the list
    pub fn move_notebook_down(&mut self, notebook_idx: usize) -> bool {
        if notebook_idx >= self.notebooks.len() - 1 {
            return false; // Can't move down if it's already at the bottom or invalid index
        }

        // Swap sort_order values with the next notebook
        let current_sort_order = self.notebooks[notebook_idx].sort_order;
        let next_sort_order = self.notebooks[notebook_idx + 1].sort_order;

        self.notebooks[notebook_idx].sort_order = next_sort_order;
        self.notebooks[notebook_idx + 1].sort_order = current_sort_order;

        // Update both notebooks in the database
        if let Ok(storage) = self.storage.lock() {
            if let Err(e) = storage.save_notebook(&self.notebooks[notebook_idx]) {
                log::error!("Failed to save notebook after moving down: {}", e);
                return false;
            }
            if let Err(e) = storage.save_notebook(&self.notebooks[notebook_idx + 1]) {
                log::error!("Failed to save notebook after moving down: {}", e);
                return false;
            }
        }

        // Swap the notebooks in the vector
        self.notebooks.swap(notebook_idx, notebook_idx + 1);

        // Update current_notebook index if needed
        if let Some(current) = self.current_notebook {
            if current == notebook_idx {
                self.current_notebook = Some(notebook_idx + 1);
            } else if current == notebook_idx + 1 {
                self.current_notebook = Some(notebook_idx);
            }
        }

        true
    }

    /// Create tag
    pub fn create_tag(&mut self, name: String, color: String) -> Option<String> {
        // Create a new tag
        let tag = crate::tag::Tag {
            id: uuid::Uuid::new_v4().to_string(),
            name: name.clone(),
            color: color.clone(),
            created_at: chrono::Utc::now(),
        };

        if let Ok(storage) = self.storage.lock() {
            match storage.save_tag(&tag) {
                Ok(_) => {
                    let tag_id = tag.id.clone();
                    self.tags.push(tag);
                    log::info!("Created new tag: {}", name);
                    return Some(tag_id);
                }
                Err(e) => {
                    log::error!("Failed to create tag: {}", e);
                }
            }
        }
        None
    }

    /// Get note tags
    pub fn get_note_tags(&self, note_id: &str) -> Vec<crate::tag::Tag> {
        if let Some(note) = self.notes.get(note_id) {
            note.tag_ids.iter()
                .filter_map(|tag_id| self.tags.iter().find(|tag| tag.id == *tag_id).cloned())
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Add tag to note
    pub fn add_tag_to_note(&mut self, note_id: &str, tag_id: &str) {
        let notebook_id = self.find_notebook_for_note(note_id).unwrap_or_default();
        if let Some(note) = self.notes.get_mut(note_id) {
            if !note.tag_ids.contains(&tag_id.to_string()) {
                note.tag_ids.push(tag_id.to_string());
                // Save to database
                if let Ok(storage) = self.storage.lock() {
                    let _ = storage.save_note(note, &notebook_id);
                }
            }
        }
    }

    /// Remove tag from note
    pub fn remove_tag_from_note(&mut self, note_id: &str, tag_id: &str) {
        let notebook_id = self.find_notebook_for_note(note_id).unwrap_or_default();
        if let Some(note) = self.notes.get_mut(note_id) {
            note.tag_ids.retain(|id| id != tag_id);
            // Save to database
            if let Ok(storage) = self.storage.lock() {
                let _ = storage.save_note(note, &notebook_id);
            }
        }
    }

    /// Get search terms
    pub fn get_search_terms(&self) -> Vec<String> {
        self.search_query.split_whitespace().map(|s| s.to_string()).collect()
    }

    /// Is current note slideshow
    pub fn is_current_note_slideshow(&self) -> bool {
        // 检查当前正在编辑的内容，而不是保存的内容
        if self.current_note.is_some() {
            return self.check_slideshow_format(&self.note_content);
        }
        false
    }

    /// Check if content is in slideshow format
    pub fn check_slideshow_format(&self, content: &str) -> bool {
        // 检查是否包含幻灯片分隔符
        let has_slide_separators = self.has_slide_separators(content);

        // 检查是否包含CSS样式块（用于自定义幻灯片样式）
        let has_css_styles = self.has_css_styles(content);

        // 检查是否包含幻灯片配置
        let has_slide_config = self.has_slide_config(content);

        // 满足任一条件即可认为是幻灯片格式
        has_slide_separators || has_css_styles || has_slide_config
    }

    /// Check if content has slide separators
    fn has_slide_separators(&self, content: &str) -> bool {
        let lines: Vec<&str> = content.lines().collect();
        let mut separator_count = 0;

        for line in lines {
            let trimmed = line.trim();
            // 检查标准的幻灯片分隔符
            if trimmed == "---" || trimmed == "--slide" || trimmed.starts_with("---slide") {
                separator_count += 1;
            }
        }

        // 至少需要一个分隔符才能构成幻灯片
        separator_count >= 1
    }

    /// Check if content has CSS styles for slides
    fn has_css_styles(&self, content: &str) -> bool {
        // 检查是否包含CSS样式块
        content.contains("<style>") && content.contains("</style>") ||
        content.contains("```css") ||
        content.contains(".slide") && (content.contains("{") && content.contains("}"))
    }

    /// Check if content has slide configuration
    fn has_slide_config(&self, content: &str) -> bool {
        // 检查是否包含幻灯片配置标记
        content.contains("slide-config:") ||
        content.contains("slideshow:") ||
        content.contains("presentation:") ||
        // 检查YAML front matter中的幻灯片配置
        (content.starts_with("---\n") && content.contains("slide:")) ||
        (content.starts_with("---\n") && content.contains("presentation:"))
    }

    /// Clear note content cache
    pub fn clear_note_cache(&mut self) {
        self.note_content_cache.clear();
        log::info!("Note content cache cleared");
    }

    /// Get cache size
    pub fn get_cache_size(&self) -> usize {
        self.note_content_cache.len()
    }

    /// Remove note from cache
    pub fn remove_from_cache(&mut self, note_id: &str) {
        self.note_content_cache.remove(note_id);
    }

    /// Check if note is in cache
    pub fn is_note_cached(&self, note_id: &str) -> bool {
        self.note_content_cache.contains_key(note_id)
    }

    /// Start slideshow
    pub fn start_slideshow(&mut self) -> Result<(), String> {
        if self.current_note.is_some() {
            // Parse the current editing content into a slideshow
            let slideshow = self.slide_parser.parse(&self.note_content)
                .map_err(|e| format!("Failed to parse slideshow: {}", e))?;
            self.slide_play_state.start_slideshow(slideshow);
            return Ok(());
        }
        Err("No note selected".to_string())
    }

    /// Stop slideshow
    pub fn stop_slideshow(&mut self) {
        self.slide_play_state.stop_slideshow();
    }

    /// Save slide style config
    pub fn save_slide_style_config(&mut self) -> Result<(), String> {
        // Implementation for saving slide style config
        Ok(())
    }

    /// Clipboard has rich content
    pub fn clipboard_has_rich_content(&mut self) -> bool {
        if let Some(clipboard) = &mut self.clipboard_manager {
            clipboard.has_rich_content()
        } else {
            false
        }
    }

    /// Paste rich text
    pub fn paste_rich_text(&mut self) -> Result<String, String> {
        if let Some(clipboard) = &mut self.clipboard_manager {
            clipboard.paste_as_markdown().map_err(|e| e.to_string())
        } else {
            Err("Clipboard not available".to_string())
        }
    }

    /// Append to note content
    pub fn append_to_note_content(&mut self, content: &str) {
        self.note_content.push_str(content);
        self.check_note_modified();
    }

    /// Force reload data
    pub fn force_reload_data(&mut self) {
        self.notebooks.clear();
        self.notes.clear();
        self.tags.clear();
        self.load_data_on_demand();
    }

    /// Confirm deletion
    pub fn confirm_deletion(&mut self) {
        if let Some(confirmation) = self.delete_confirmation.clone() {
            match confirmation.confirmation_type {
                DeleteConfirmationType::Note => {
                    self.delete_note(&confirmation.item_id);
                }
                DeleteConfirmationType::Notebook => {
                    if let Some(idx) = self.notebooks.iter().position(|nb| nb.id == confirmation.item_id) {
                        self.delete_notebook(idx);
                    }
                }
                DeleteConfirmationType::Tag => {
                    self.delete_tag(&confirmation.item_id);
                }
            }
        }
        self.cancel_deletion();
    }

    /// Cancel deletion
    pub fn cancel_deletion(&mut self) {
        self.show_delete_confirmation = false;
        self.delete_confirmation = None;
    }

    /// Import document as note
    pub fn import_document_as_note(&mut self, file_path: &str, selected_notebook_id: &str) -> Result<String, String> {
        log::info!("开始导入文档: {}", file_path);

        // 检查文件是否存在
        let path = std::path::Path::new(file_path);
        if !path.exists() {
            return Err(format!("文件不存在: {}", file_path));
        }

        // 检查文件格式是否支持
        if !DocumentConverter::is_supported_format(file_path) {
            return Err(format!("不支持的文件格式: {}", file_path));
        }

        // 提取文件名作为标题
        let title = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Imported Document")
            .to_string();

        // 使用文档转换器转换文档内容
        let content = match self.document_converter.convert_to_markdown(file_path) {
            Ok(markdown_content) => {
                log::info!("成功转换文档为Markdown格式: {}", file_path);
                markdown_content
            },
            Err(e) => {
                log::error!("文档转换失败: {}", e);
                // 如果转换失败，创建一个包含错误信息的内容
                format!("# {}\n\n⚠️ 文档导入失败\n\n**原始文件路径**: {}\n\n**错误信息**: {}\n\n---\n\n*请检查文件格式是否正确，或尝试重新导入。*",
                    title, file_path, e)
            }
        };

        // 创建笔记
        let note = Note {
            id: uuid::Uuid::new_v4().to_string(),
            title: title.clone(),
            content: content.clone(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            tag_ids: Vec::new(),
            attachments: Vec::new(),
        };

        // 保存到数据库
        let result = if let Ok(storage) = self.storage.lock() {
            match storage.save_note(&note, selected_notebook_id) {
                Ok(_) => {
                    let note_id = note.id.clone();

                    // 验证保存是否成功
                    match storage.load_note(&note_id) {
                        Ok(saved_note) => {
                            log::info!("文档导入成功，笔记ID: {}", note_id);

                            // 更新内存状态
                            self.notes.insert(note_id.clone(), saved_note);

                            Ok(note_id)
                        },
                        Err(e) => {
                            log::error!("验证保存的笔记失败: {}", e);
                            Err(format!("保存验证失败: {}", e))
                        }
                    }
                },
                Err(e) => {
                    log::error!("保存笔记到数据库失败: {}", e);
                    Err(format!("保存失败: {}", e))
                }
            }
        } else {
            Err("无法获取数据库连接".to_string())
        };

        // 在释放锁之后更新笔记本的note_ids并重新加载笔记列表以确保一致性
        if let Ok(note_id) = &result {
            // 找到对应的笔记本并更新其note_ids
            if let Some(notebook) = self.notebooks.iter_mut().find(|nb| nb.id == selected_notebook_id) {
                if !notebook.note_ids.contains(note_id) {
                    notebook.add_note(note_id.clone()); // 使用 add_note 将新笔记插入到第一条之前
                    log::debug!("已将笔记 {} 添加到笔记本 {} 的note_ids中（插入到第一条之前）", note_id, selected_notebook_id);

                    // 保存更新后的笔记本到数据库
                    if let Ok(storage) = self.storage.lock() {
                        if let Err(e) = storage.save_notebook(notebook) {
                            log::warn!("保存更新后的笔记本失败: {}", e);
                        } else {
                            log::debug!("成功保存更新后的笔记本到数据库");
                        }
                    }
                }
            } else {
                log::warn!("未找到ID为 {} 的笔记本", selected_notebook_id);
            }

            // 重新加载笔记本的笔记列表以确保一致性
            if let Err(e) = self.load_notes_for_notebook(selected_notebook_id) {
                log::warn!("重新加载笔记本笔记列表失败: {}", e);
            }
        }

        result
    }

    /// Get recent notes
    pub fn get_recent_notes(&self, limit: usize) -> Vec<RecentNoteAccess> {
        self.recent_notes.iter().take(limit).cloned().collect()
    }

    /// Check if clipboard has image data
    pub fn clipboard_has_image(&mut self) -> bool {
        if let Some(ref mut clipboard) = self.clipboard_manager {
            clipboard.has_image()
        } else {
            false
        }
    }

    /// Paste image from clipboard and return markdown text
    pub fn paste_image(&mut self) -> Result<String, String> {
        if let Some(ref mut clipboard) = self.clipboard_manager {
            match clipboard.get_image() {
                Ok(Some(image_data)) => {
                    // Save image to storage
                    match self.image_storage.save_image(&image_data) {
                        Ok(relative_path) => {
                            // Return markdown image syntax
                            Ok(format!("![图片]({})", relative_path))
                        }
                        Err(e) => {
                            Err(format!("Failed to save image: {}", e))
                        }
                    }
                }
                Ok(None) => {
                    Err("No image data in clipboard".to_string())
                }
                Err(e) => {
                    Err(format!("Failed to get image from clipboard: {}", e))
                }
            }
        } else {
            Err("Clipboard manager not available".to_string())
        }
    }

    /// Insert image at cursor position in note content
    pub fn insert_image_at_cursor(&mut self, image_markdown: &str) {
        // For now, just append to the end of the content
        // In a more sophisticated implementation, we would track cursor position
        if !self.note_content.is_empty() && !self.note_content.ends_with('\n') {
            self.note_content.push('\n');
        }
        self.note_content.push_str(image_markdown);
        self.note_content.push('\n');
    }

    /// Paste image and insert into note
    pub fn paste_and_insert_image(&mut self) -> Result<(), String> {
        let image_markdown = self.paste_image()?;
        self.insert_image_at_cursor(&image_markdown);
        log::info!("Image pasted and inserted into note");
        Ok(())
    }

    /// 设置笔记排序方式
    pub fn set_note_sort_by(&mut self, sort_by: NoteSortBy) {
        if self.note_sort_by != sort_by {
            self.note_sort_by = sort_by;
            // 重新加载当前笔记本的笔记以应用新的排序
            if let Some(notebook_idx) = self.current_notebook {
                if notebook_idx < self.notebooks.len() {
                    let notebook_id = self.notebooks[notebook_idx].id.clone();
                    self.load_notes_for_notebook(&notebook_id);
                }
            }
        }
    }

    /// 获取当前笔记排序方式
    pub fn get_note_sort_by(&self) -> &NoteSortBy {
        &self.note_sort_by
    }

    /// 设置笔记视图模式
    pub fn set_note_view_mode(&mut self, view_mode: NoteViewMode) {
        if self.note_view_mode != view_mode {
            self.note_view_mode = view_mode;
            // 如果切换到时间视图，加载全局笔记列表
            if self.note_view_mode == NoteViewMode::TimeView {
                self.load_global_notes();
            }
        }
    }

    /// 获取当前笔记视图模式
    pub fn get_note_view_mode(&self) -> &NoteViewMode {
        &self.note_view_mode
    }

    /// 加载全局笔记列表（按时间排序）
    pub fn load_global_notes(&mut self) {
        if let Ok(storage) = self.storage.lock() {
            match storage.get_all_notes_sorted(&self.note_sort_by) {
                Ok(notes) => {
                    log::debug!("加载全局笔记列表: {} 个笔记", notes.len());

                    // 更新全局笔记ID列表
                    self.global_notes.clear();
                    for note in &notes {
                        self.global_notes.push(note.id.clone());
                    }

                    // 更新notes HashMap
                    for note in notes {
                        self.notes.insert(note.id.clone(), note);
                    }
                },
                Err(e) => {
                    log::error!("加载全局笔记列表失败: {}", e);
                }
            }
        }
    }
}
