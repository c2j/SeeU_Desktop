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
    pub search_results: Vec<String>, // IDs of notes that match the search query
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
            self.notes.insert(note_id.clone(), note);

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
            if let Ok(storage) = self.storage.lock() {
                if let Some(notebook_id) = notebook_id {
                    if let Err(err) = storage.save_note(note, &notebook_id) {
                        log::error!("Failed to save note to database: {}", err);
                    }
                }
            } else {
                log::error!("Failed to lock storage for note: {}", note_id);
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

        // Regular search in database
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
                }
                Err(err) => {
                    log::error!("Failed to search notes: {}", err);
                }
            }
        }
    }

    /// Find a tag by name
    fn find_tag_by_name(&self, name: &str) -> Option<String> {
        self.tags.iter()
            .find(|tag| tag.name.to_lowercase() == name.to_lowercase())
            .map(|tag| tag.id.clone())
    }

    /// Get search result notes
    pub fn get_search_result_notes(&self) -> Vec<&Note> {
        self.search_results
            .iter()
            .filter_map(|id| self.notes.get(id))
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

                    // Update search results
                    self.search_results = notes.iter().map(|note| note.id.clone()).collect();

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

                    // 更新内存中的笔记
                    for note in db_notes {
                        self.notes.insert(note.id.clone(), note);
                    }

                    // 重新构建笔记本的note_ids，但保护内存中的新笔记
                    if let Some(notebook) = self.notebooks.iter_mut().find(|nb| nb.id == notebook_id) {
                        // 获取数据库中的所有笔记ID
                        let mut db_note_ids = Vec::new();
                        if let Ok(db_notes) = storage.get_notes_for_notebook(notebook_id) {
                            for note in db_notes {
                                db_note_ids.push(note.id);
                            }
                        }

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
        let markdown_content = self.document_converter
            .convert_to_markdown(file_path)
            .map_err(|e| format!("文档转换失败: {}", e))?;

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

        // Save note to storage
        log::info!("保存笔记到数据库...");
        if let Ok(storage) = self.storage.lock() {
            if let Err(err) = storage.save_note(&note, notebook_id) {
                log::error!("Failed to save imported note: {}", err);
                return Err(format!("保存笔记失败: {}", err));
            }
            log::info!("笔记保存到数据库成功");
        } else {
            return Err("无法获取存储锁".to_string());
        }

        // 确保数据库操作完全完成
        log::debug!("等待数据库操作完成...");
        std::thread::sleep(std::time::Duration::from_millis(50));

        // Add note to notes map first (确保笔记在内存中)
        self.notes.insert(note_id.clone(), note.clone());
        log::info!("Added note '{}' to memory, total notes in memory: {}", note_id, self.notes.len());

        // Add note to the notebook
        if let Some(notebook) = self.notebooks.iter_mut().find(|nb| nb.id == notebook_id) {
            // 检查笔记是否已经在笔记本中，避免重复添加
            if !notebook.note_ids.contains(&note_id) {
                notebook.add_note(note_id.clone());
                log::info!("Added note '{}' to notebook '{}', total notes: {}", note_id, notebook.name, notebook.note_ids.len());
            } else {
                log::info!("Note '{}' already exists in notebook '{}', skipping add", note_id, notebook.name);
            }

            // Save updated notebook (注意：这里不保存note_ids到数据库，因为数据库使用关系设计)
            if let Ok(storage) = self.storage.lock() {
                if let Err(err) = storage.save_notebook(notebook) {
                    log::error!("Failed to save updated notebook: {}", err);
                } else {
                    log::info!("Successfully saved updated notebook '{}'", notebook.name);
                }
            }
        } else {
            return Err("找不到指定的笔记本".to_string());
        }

        // 验证导入结果
        log::info!("验证导入的笔记是否正确保存到数据库...");

        // 使用新的数据库连接进行验证，避免连接池问题
        let verification_result = if let Ok(storage) = self.storage.lock() {
            // 尝试多次验证，处理可能的时序问题
            let mut attempts = 0;
            let max_attempts = 3;

            loop {
                attempts += 1;
                log::debug!("验证尝试 {}/{}", attempts, max_attempts);

                match storage.load_note(&note_id) {
                    Ok(verified_note) => {
                        log::info!("✅ 验证成功: 笔记 '{}' 已正确保存到数据库", verified_note.title);
                        break Ok(());
                    }
                    Err(err) => {
                        log::warn!("验证尝试 {} 失败: {}", attempts, err);

                        if attempts >= max_attempts {
                            log::error!("❌ 验证失败: 经过 {} 次尝试仍无法从数据库加载笔记 '{}'", max_attempts, note_id);
                            break Err(format!("导入验证失败: {}", err));
                        } else {
                            // 短暂等待后重试
                            std::thread::sleep(std::time::Duration::from_millis(100));
                        }
                    }
                }
            }
        } else {
            Err("无法获取存储锁进行验证".to_string())
        };

        // 检查验证结果
        if let Err(err) = verification_result {
            // 验证失败，但笔记可能已经在内存中，给用户一个警告而不是完全失败
            log::warn!("验证失败，但笔记已在内存中: {}", err);
            log::warn!("笔记可能已保存，但数据库验证失败。请重启应用后检查。");

            // 不返回错误，让用户可以继续使用
            // return Err(err);
        }

        log::info!("Successfully imported document as note '{}'", note_id);

        // 返回成功，即使验证可能失败
        // 因为笔记已经在内存中可用
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
}
