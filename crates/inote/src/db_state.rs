use eframe::egui;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use chrono::Utc;
use log;

use crate::notebook::Notebook;
use crate::note::{Note, Attachment};
use crate::tag::Tag;
use crate::db_storage::DbStorageManager;
use crate::migration::DataMigration;
use crate::db_ui_import::SiyuanImportState;

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
    pub markdown_preview: bool,      // Whether to show markdown preview instead of editor
    pub last_saved_content: String,  // Last saved content for auto-save comparison
    pub last_saved_title: String,    // Last saved title for auto-save comparison
    pub save_status: SaveStatus,     // Current save status
    pub editor_maximized: bool,      // Whether the editor is maximized
    pub combined_editor: bool,       // Whether title and content are combined in editor
    pub siyuan_import: SiyuanImportState, // 思源笔记导入状态
    pub show_markdown_help: bool,    // Whether to show the Markdown help popup

    // 删除确认对话框
    pub show_delete_confirmation: bool,
    pub delete_confirmation: Option<DeleteConfirmation>,
}

impl Default for DbINoteState {
    fn default() -> Self {
        // Create storage manager
        let storage = match DbStorageManager::new() {
            Ok(storage) => Arc::new(Mutex::new(storage)),
            Err(err) => {
                log::error!("Failed to create database storage: {}", err);
                panic!("Failed to create database storage: {}", err);
            }
        };

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
            markdown_preview: false,
            last_saved_content: String::new(),
            last_saved_title: String::new(),
            save_status: SaveStatus::Saved,
            editor_maximized: false,
            combined_editor: false,  // 默认使用分离标题模式
            siyuan_import: SiyuanImportState::default(),
            show_markdown_help: false,
            show_delete_confirmation: false,
            delete_confirmation: None,
        }
    }
}

impl DbINoteState {
    /// Initialize the state
    pub fn initialize(&mut self) {
        // Migrate data from file storage if needed
        self.migrate_data();

        // Load notebooks, notes, and tags
        self.load_notebooks();
        self.load_tags();

        // Load notes for all notebooks
        self.load_all_notes();
    }

    /// Migrate data from file storage to SQLite database
    fn migrate_data(&self) {
        if let Ok(storage) = self.storage.lock() {
            let migration = DataMigration::new_with_ref(&storage);
            if let Err(err) = migration.migrate() {
                log::error!("Failed to migrate data: {}", err);
            }
        }
    }

    /// Load notebooks from storage
    fn load_notebooks(&mut self) {
        if let Ok(storage) = self.storage.lock() {
            match storage.list_notebooks() {
                Ok(notebooks) => {
                    self.notebooks = notebooks;
                }
                Err(err) => {
                    log::error!("Failed to load notebooks: {}", err);
                }
            }
        }
    }

    /// Load notes for a notebook
    fn load_notes_for_notebook(&mut self, notebook_id: &str) {
        if let Ok(storage) = self.storage.lock() {
            match storage.get_notes_for_notebook(notebook_id) {
                Ok(notes) => {
                    for note in notes {
                        self.notes.insert(note.id.clone(), note);
                    }
                    log::info!("Loaded notes for notebook {}", notebook_id);
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

    /// Load tags from storage
    fn load_tags(&mut self) {
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
        }
    }

    /// Select a note
    pub fn select_note(&mut self, note_id: &str) {
        if let Some(note) = self.notes.get(note_id) {
            self.current_note = Some(note_id.to_string());

            // 在新的UI布局中，标题始终单独显示，内容始终只包含正文
            self.note_content = note.content.clone();
            self.note_title = note.title.clone();

            // Initialize last saved content and title
            self.last_saved_content = self.note_content.clone();
            self.last_saved_title = self.note_title.clone();
            self.save_status = SaveStatus::Saved;
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
        // Save current selection
        let current_notebook = self.current_notebook;
        let current_note = self.current_note.clone();

        // Clear current state
        self.notebooks.clear();
        self.notes.clear();
        self.tags.clear();
        self.current_notebook = None;
        self.current_note = None;
        self.note_content.clear();
        self.note_title.clear();

        // Reload from database
        self.load_notebooks();
        self.load_tags();
        self.load_all_notes(); // 重要：重新加载所有笔记

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
}
