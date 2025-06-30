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
    pub fn initialize_storage_async(&mut self, db_path: String) {
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
                            self.notes.insert(note_id.clone(), note);
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
        if let Some(note) = self.notes.get(note_id).cloned() {
            self.current_note = Some(note_id.to_string());
            self.note_title = note.title.clone();
            self.note_content = note.content.clone();
            self.last_saved_title = note.title.clone();
            self.last_saved_content = note.content.clone();
            self.save_status = SaveStatus::Saved;

            // Add to recent notes
            self.add_to_recent_notes(note_id, &note.title);

            log::info!("Selected note: {}", note.title);
        }
    }

    /// Add note to recent notes list
    fn add_to_recent_notes(&mut self, note_id: &str, note_title: &str) {
        let access = RecentNoteAccess {
            note_id: note_id.to_string(),
            note_title: note_title.to_string(),
            accessed_at: Utc::now(),
        };

        // Remove existing entry if present
        self.recent_notes.retain(|item| item.note_id != note_id);

        // Add to front
        self.recent_notes.push_front(access);

        // Keep only last 20 items
        while self.recent_notes.len() > 20 {
            self.recent_notes.pop_back();
        }
    }

    /// Toggle note tree visibility
    pub fn toggle_note_tree(&mut self) {
        self.show_note_tree = !self.show_note_tree;
    }

    /// Find notebook for note
    pub fn find_notebook_for_note(&self, note_id: &str) -> Option<String> {
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
    pub fn load_notes_for_notebook(&mut self, notebook_id: &str) {
        if let Ok(storage) = self.storage.lock() {
            if let Ok(notes) = storage.get_notes_for_notebook(notebook_id) {
                for note in notes {
                    self.notes.insert(note.id.clone(), note);
                }
            }
        }
    }

    /// Auto save if modified
    pub fn auto_save_if_modified(&mut self) {
        if self.note_content != self.last_saved_content || self.note_title != self.last_saved_title {
            // Implement auto save logic here
            self.save_status = SaveStatus::Saving;
            // For now, just mark as saved
            self.last_saved_content = self.note_content.clone();
            self.last_saved_title = self.note_title.clone();
            self.save_status = SaveStatus::Saved;
        }
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
        self.notes.remove(note_id);
        if let Ok(storage) = self.storage.lock() {
            let _ = storage.delete_note(note_id);
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
        // Create a new notebook
        let notebook = crate::notebook::Notebook {
            id: uuid::Uuid::new_v4().to_string(),
            name: name.clone(),
            description: description.clone(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            note_ids: Vec::new(),
            expanded: true,
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
        if let Some(note_id) = &self.current_note {
            if let Some(note) = self.notes.get(note_id) {
                return note.content.contains("---") && note.content.lines().any(|line| line.trim() == "---");
            }
        }
        false
    }

    /// Start slideshow
    pub fn start_slideshow(&mut self) -> Result<(), String> {
        if let Some(note_id) = &self.current_note {
            if let Some(note) = self.notes.get(note_id) {
                // Parse the note content into a slideshow
                let slideshow = self.slide_parser.parse(&note.content)
                    .map_err(|e| format!("Failed to parse slideshow: {}", e))?;
                self.slide_play_state.start_slideshow(slideshow);
                return Ok(());
            }
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
        // Implementation for importing document as note
        // For now, create a simple note
        let title = std::path::Path::new(file_path)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Imported Document")
            .to_string();

        let content = format!("Imported from: {}", file_path);

        if let Some(note_id) = self.create_note(title, content) {
            Ok(note_id)
        } else {
            Err("Failed to create note".to_string())
        }
    }

    /// Get recent notes
    pub fn get_recent_notes(&self, limit: usize) -> Vec<RecentNoteAccess> {
        self.recent_notes.iter().take(limit).cloned().collect()
    }
}
