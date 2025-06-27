pub mod notebook;
pub mod note;
pub mod tag;
pub mod storage;
pub mod clipboard;
pub mod db_storage;
pub mod migration;
pub mod db_state;
pub mod db_ui;
pub mod db_ui_import;
pub mod tree_ui;
pub mod markdown;
pub mod mermaid;

#[cfg(test)]
mod tests {
    use super::mermaid::MermaidDiagramType;

    #[test]
    fn test_class_diagram_detection() {
        let code = "classDiagram\n    class Animal {\n        +String name\n    }";
        let diagram_type = MermaidDiagramType::from_code(code);
        assert_eq!(diagram_type, MermaidDiagramType::ClassDiagram);
    }

    #[test]
    fn test_state_diagram_detection() {
        let code = "stateDiagram-v2\n    [*] --> Idle";
        let diagram_type = MermaidDiagramType::from_code(code);
        assert_eq!(diagram_type, MermaidDiagramType::StateDiagram);
    }

    #[test]
    fn test_git_graph_detection() {
        let code = "gitGraph\n    commit id: \"Initial\"";
        let diagram_type = MermaidDiagramType::from_code(code);
        assert_eq!(diagram_type, MermaidDiagramType::GitGraph);
    }

    #[test]
    fn test_user_journey_detection() {
        let code = "journey\n    title My working day";
        let diagram_type = MermaidDiagramType::from_code(code);
        assert_eq!(diagram_type, MermaidDiagramType::UserJourney);
    }

    #[test]
    fn test_entity_relationship_detection() {
        let code = "erDiagram\n    CUSTOMER {\n        string name\n    }";
        let diagram_type = MermaidDiagramType::from_code(code);
        assert_eq!(diagram_type, MermaidDiagramType::EntityRelationship);
    }

    #[test]
    fn test_flowchart_detection() {
        let code = "flowchart TD\n    A --> B";
        let diagram_type = MermaidDiagramType::from_code(code);
        assert_eq!(diagram_type, MermaidDiagramType::Flowchart);
    }

    #[test]
    fn test_sequence_detection() {
        let code = "sequenceDiagram\n    participant A";
        let diagram_type = MermaidDiagramType::from_code(code);
        assert_eq!(diagram_type, MermaidDiagramType::Sequence);
    }

    #[test]
    fn test_unknown_detection() {
        let code = "unknownDiagram\n    some content";
        let diagram_type = MermaidDiagramType::from_code(code);
        assert_eq!(diagram_type, MermaidDiagramType::Unknown);
    }
}
pub mod document_converter;
pub mod notebook_selector;
pub mod slide;
pub mod siyuan_import;
pub mod mcp_server;
pub mod mcp_sync;

use eframe::egui;
use notebook::Notebook;
use note::{Note, Attachment};
use tag::Tag;
use storage::StorageManager;
use db_storage::DbStorageManager;
use db_state::{DbINoteState, SaveStatus};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;
use chrono::Utc;

/// iNote state
pub struct INoteState {
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
    pub storage: Arc<Mutex<StorageManager>>,
}

impl Default for INoteState {
    fn default() -> Self {
        let storage = Arc::new(Mutex::new(StorageManager::new()));

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
        }
    }
}

impl INoteState {
    /// Initialize the state
    pub fn initialize(&mut self) {
        // Load notebooks, notes, and tags
        self.load_notebooks();
        self.load_notes();
        self.load_tags();
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

    /// Load notes from storage
    fn load_notes(&mut self) {
        if let Ok(storage) = self.storage.lock() {
            // Load notes for each notebook
            for notebook in &self.notebooks {
                for note_id in &notebook.note_ids {
                    if self.notes.contains_key(note_id) {
                        continue;
                    }

                    match storage.load_note(note_id) {
                        Ok(note) => {
                            self.notes.insert(note_id.clone(), note);
                        }
                        Err(err) => {
                            log::error!("Failed to load note {}: {}", note_id, err);
                        }
                    }
                }
            }
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

    /// Delete a notebook
    pub fn delete_notebook(&mut self, index: usize) {
        if index >= self.notebooks.len() {
            return;
        }

        // Get notebook ID
        let notebook_id = self.notebooks[index].id.clone();

        // Get note IDs to delete
        let note_ids: Vec<String> = self.notebooks[index].note_ids.clone();

        // Delete all notes in the notebook
        for note_id in &note_ids {
            self.delete_note(note_id);
        }

        // Delete from storage
        if let Ok(storage) = self.storage.lock() {
            if let Err(err) = storage.delete_notebook(&notebook_id) {
                log::error!("Failed to delete notebook from storage: {}", err);
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
    }

    /// Create a new note
    pub fn create_note(&mut self, title: String, content: String) -> Option<String> {
        if let Some(notebook_idx) = self.current_notebook {
            if notebook_idx >= self.notebooks.len() {
                return None;
            }

            let note = Note::new(title, content);
            let note_id = note.id.clone();

            // Save to storage
            if let Ok(storage) = self.storage.lock() {
                if let Err(err) = storage.save_note(&note) {
                    log::error!("Failed to save note: {}", err);
                    return None;
                }
            }

            // Add to notebook
            self.notebooks[notebook_idx].add_note(note_id.clone());

            // Save notebook
            if let Ok(storage) = self.storage.lock() {
                if let Err(err) = storage.save_notebook(&self.notebooks[notebook_idx]) {
                    log::error!("Failed to save notebook: {}", err);
                }
            }

            // Add to notes map
            self.notes.insert(note_id.clone(), note);

            return Some(note_id);
        }

        None
    }

    /// Update a note
    pub fn update_note(&mut self, note_id: &str, title: String, content: String) {
        if let Some(note) = self.notes.get_mut(note_id) {
            note.title = title;
            note.content = content;
            note.updated_at = Utc::now();

            // Save to storage
            if let Ok(storage) = self.storage.lock() {
                if let Err(err) = storage.save_note(note) {
                    log::error!("Failed to save note: {}", err);
                }
            }
        }
    }

    /// Delete a note
    pub fn delete_note(&mut self, note_id: &str) {
        // Remove from notebooks
        for notebook in &mut self.notebooks {
            notebook.remove_note(note_id);

            // Save notebook
            if let Ok(storage) = self.storage.lock() {
                if let Err(err) = storage.save_notebook(notebook) {
                    log::error!("Failed to save notebook: {}", err);
                }
            }
        }

        // Delete from storage
        if let Ok(storage) = self.storage.lock() {
            if let Err(err) = storage.delete_note(note_id) {
                log::error!("Failed to delete note from storage: {}", err);
            }
        }

        // Remove from notes map
        self.notes.remove(note_id);

        // Reset current note if needed
        if self.current_note == Some(note_id.to_string()) {
            self.current_note = None;
            self.note_content = String::new();
            self.note_title = String::new();
        }
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

    /// Delete a tag
    pub fn delete_tag(&mut self, tag_id: &str) {
        // Remove tag from all notes
        for (_, note) in &mut self.notes {
            note.remove_tag(tag_id);

            // Save note
            if let Ok(storage) = self.storage.lock() {
                if let Err(err) = storage.save_note(note) {
                    log::error!("Failed to save note: {}", err);
                }
            }
        }

        // Delete from storage
        if let Ok(storage) = self.storage.lock() {
            if let Err(err) = storage.delete_tag(tag_id) {
                log::error!("Failed to delete tag from storage: {}", err);
            }
        }

        // Remove from list
        self.tags.retain(|tag| tag.id != tag_id);
    }

    /// Add a tag to a note
    pub fn add_tag_to_note(&mut self, note_id: &str, tag_id: &str) {
        if let Some(note) = self.notes.get_mut(note_id) {
            note.add_tag(tag_id.to_string());

            // Save to storage
            if let Ok(storage) = self.storage.lock() {
                if let Err(err) = storage.save_note(note) {
                    log::error!("Failed to save note: {}", err);
                }
            }
        }
    }

    /// Remove a tag from a note
    pub fn remove_tag_from_note(&mut self, note_id: &str, tag_id: &str) {
        if let Some(note) = self.notes.get_mut(note_id) {
            note.remove_tag(tag_id);

            // Save to storage
            if let Ok(storage) = self.storage.lock() {
                if let Err(err) = storage.save_note(note) {
                    log::error!("Failed to save note: {}", err);
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
            self.note_content = note.content.clone();
            self.note_title = note.title.clone();
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

        let query = self.search_query.to_lowercase();

        // Find matching notes
        let matching_notes: Vec<String> = self.notes
            .iter()
            .filter(|(_, note)| {
                note.title.to_lowercase().contains(&query) ||
                note.content.to_lowercase().contains(&query)
            })
            .map(|(id, _)| id.clone())
            .collect();

        // Update search results
        self.search_results = matching_notes;
        self.is_searching = true;
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
}

/// Render the iNote module
pub fn render_inote(ui: &mut egui::Ui, state: &mut INoteState) {
    render_inote_with_sidebar_info(ui, state, false);
}

/// Render the main iNote interface with sidebar information
pub fn render_inote_with_sidebar_info(ui: &mut egui::Ui, state: &mut INoteState, right_sidebar_open: bool) {
    // Initialize state if needed
    if state.notebooks.is_empty() && state.notes.is_empty() && state.tags.is_empty() {
        state.initialize();
    }

    // Create a default notebook if none exists
    if state.notebooks.is_empty() {
        state.create_notebook("默认笔记本".to_string(), "默认笔记本".to_string());
    }
}

/// Render the iNote module with SQLite storage
pub fn render_db_inote(ui: &mut egui::Ui, state: &mut DbINoteState) {
    render_db_inote_with_sidebar_info(ui, state, false, None, None);
}

/// Render the iNote module with SQLite storage and sidebar information
pub fn render_db_inote_with_sidebar_info(ui: &mut egui::Ui, state: &mut DbINoteState, right_sidebar_open: bool, right_sidebar_width: Option<f32>, font_family: Option<&str>) {
    // Ensure data is loaded when needed (lazy loading)
    state.ensure_data_loaded();

    // 检查是否处于幻灯片播放模式
    if state.slide_play_state.is_playing {
        let should_close = crate::slide::SlideRenderer::render_slideshow(ui, &mut state.slide_play_state);
        if should_close {
            state.stop_slideshow();
        }
        return; // 在幻灯片模式下，不渲染其他UI
    }

    // 检查是否处于全窗口最大化模式
    if state.editor_maximized && state.current_note.is_some() {
        // 全窗口最大化模式 - 只显示编辑器
        ui.vertical(|ui| {
            // 添加一个返回按钮 - 固定在顶部不随内容滚动
            ui.horizontal(|ui| {
                if ui.button("◀ 返回正常视图").clicked() {
                    state.editor_maximized = false;
                }

                // 在最大化模式下仍然显示编辑/预览切换按钮
                ui.add_space(10.0);
                if ui.button(if state.markdown_preview { "📝 编辑" } else { "👁 预览" }).clicked() {
                    state.auto_save_if_modified();
                    state.markdown_preview = !state.markdown_preview;
                }

                // 显示当前笔记标题
                if let Some(note_id) = &state.current_note {
                    if let Some(note) = state.notes.get(note_id) {
                        ui.add_space(20.0);
                        ui.heading(&note.title);
                    }
                }
            });

            ui.separator();

            // 在全屏模式下显示笔记编辑器
            crate::db_ui::render_note_editor(ui, state, font_family);
        });
    } else {
        // 正常模式
        ui.vertical(|ui| {
            // Search bar and controls
            ui.horizontal(|ui| {
                // 笔记树切换按钮
                let tree_button_text = if state.show_note_tree { "📁 隐藏树" } else { "📁 显示树" };
                if ui.button(tree_button_text).on_hover_text("切换笔记树的显示/隐藏").clicked() {
                    state.toggle_note_tree();
                }

                ui.separator();

                ui.label("🔍");
                let search_width = ui.available_width() - 50.0;
                let response = ui.add(
                    egui::TextEdit::singleline(&mut state.search_query)
                        .hint_text("搜索笔记... (支持 label:标签名)")
                        .desired_width(search_width)
                );

                if ui.button("搜索").clicked() ||
                   (response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter))) {
                    if !state.search_query.is_empty() {
                        state.search_notes();
                    }
                }

                // 添加帮助按钮，显示搜索语法
                let help_button = ui.button("?").on_hover_text(
                    "搜索语法帮助:\n\n普通搜索: 直接输入关键词\n标签搜索: label:标签名"
                );
                if help_button.clicked() {
                    // 可以在这里添加更详细的帮助对话框
                }

                // 如果正在搜索，显示返回按钮
                if state.is_searching {
                    if ui.button("◀ 返回").clicked() {
                        state.is_searching = false;
                        state.search_query.clear();
                        state.search_results.clear();
                    }
                }
            });

            ui.separator();

            // Main content layout - conditionally show tree view
            if state.show_note_tree {
                // Show tree view in left panel
                let mut side_panel = egui::SidePanel::left("tree_panel")
                    .resizable(true)
                    .default_width(250.0)
                    .max_width(400.0); // 设置最大宽度，防止被内容撑得过宽

                // 在Linux下减少间距以解决100px间隔问题
                #[cfg(target_os = "linux")]
                {
                    side_panel = side_panel.frame(egui::Frame::none().inner_margin(egui::Margin::same(0.0)));
                }

                side_panel.show_inside(ui, |ui| {
                        // 设置固定宽度布局，防止内容自动撑开
                        ui.set_width(ui.available_width());

                        // 检查是否正在搜索
                        if state.is_searching {
                            // 在侧边栏显示搜索结果
                            log::info!("Sidebar: Showing search results. Query: '{}', Results: {}",
                                state.search_query, state.search_results.len());
                            crate::db_ui::render_search_results(ui, state);
                        } else {
                            // 树状视图，整合笔记本和笔记
                            crate::tree_ui::render_tree_view(ui, state);

                            // Tags section
                            ui.add_space(10.0);
                            crate::tree_ui::render_tag_list(ui, state);
                        }
                    });
            }

            // 使用完整的中央面板，egui会自动处理侧边栏的空间分配
            // 增加右侧边距15px，使编辑区与AI助手侧边栏保持适当距离
            egui::CentralPanel::default()
                .frame(egui::Frame::none().inner_margin(egui::Margin {
                    left: 8.0,
                    right: 23.0,  // 8 + 15 = 23，增加15px右边距
                    top: 8.0,
                    bottom: 8.0,
                }))
                .show_inside(ui, |ui| {
                    render_note_content_area(ui, state, font_family);
                });
        });
    }

    // 处理对话框
    process_dialogs(ui, state);
}

/// 渲染笔记内容区域（提取的公共逻辑）
fn render_note_content_area(ui: &mut egui::Ui, state: &mut DbINoteState, font_family: Option<&str>) {
    // 检查是否正在搜索
    if state.is_searching {
        // 如果选中了搜索结果中的笔记，显示笔记编辑器
        if let Some(_note_id) = &state.current_note {
            crate::db_ui::render_note_editor(ui, state, font_family);
        } else {
            // 如果笔记树隐藏，在中央面板显示搜索结果
            if !state.show_note_tree {
                crate::db_ui::render_search_results(ui, state);
            } else {
                // 否则显示搜索结果的提示
                ui.centered_and_justified(|ui| {
                    ui.heading("请从左侧选择一个搜索结果");
                });
            }
        }
    } else if let Some(_note_id) = &state.current_note {
        // 直接显示笔记编辑器
        crate::db_ui::render_note_editor(ui, state, font_family);
    } else if state.current_notebook.is_some() {
        // 显示笔记本信息
        if let Some(notebook_idx) = state.current_notebook {
            if notebook_idx < state.notebooks.len() {
                let notebook = &state.notebooks[notebook_idx];
                ui.heading(&notebook.name);
                ui.label(&notebook.description);
                ui.separator();
                ui.label(format!("包含 {} 个笔记", notebook.note_ids.len()));

                if ui.button("+ 创建新笔记").clicked() {
                    let note_id = state.create_note("新笔记".to_string(), "".to_string());
                    if let Some(id) = note_id {
                        state.select_note(&id);
                    }
                }
            }
        }
    } else {
        // 如果笔记树隐藏，显示笔记本选择界面
        if !state.show_note_tree {
            render_notebook_selection_area(ui, state);
        } else {
            ui.centered_and_justified(|ui| {
                ui.label("选择或创建一个笔记本");
            });
        }
    }
}

/// 渲染笔记本选择区域（当笔记树隐藏时使用）
fn render_notebook_selection_area(ui: &mut egui::Ui, state: &mut DbINoteState) {
    ui.vertical_centered(|ui| {
        ui.heading("📚 笔记本管理");
        ui.add_space(20.0);

        if state.notebooks.is_empty() {
            ui.label("还没有笔记本");
            ui.add_space(10.0);
            if ui.button("+ 创建第一个笔记本").clicked() {
                state.show_create_notebook = true;
            }
        } else {
            ui.label("选择一个笔记本开始编辑：");
            ui.add_space(10.0);

            // 显示笔记本列表
            egui::ScrollArea::vertical()
                .max_height(300.0)
                .show(ui, |ui| {
                    // 克隆笔记本数据以避免借用冲突
                    let notebooks_data: Vec<(usize, String, usize, bool)> = state.notebooks.iter().enumerate()
                        .map(|(idx, nb)| (idx, nb.name.clone(), nb.note_ids.len(), state.current_notebook == Some(idx)))
                        .collect();

                    for (notebook_idx, notebook_name, note_count, is_selected) in notebooks_data {
                        ui.horizontal(|ui| {
                            if ui.selectable_label(is_selected, format!("📓 {}", notebook_name)).clicked() {
                                state.select_notebook(notebook_idx);
                                state.current_note = None;
                                // 自动展开并加载笔记
                                if let Some(nb) = state.notebooks.get_mut(notebook_idx) {
                                    if !nb.expanded {
                                        nb.expanded = true;
                                        let notebook_id = nb.id.clone();
                                        state.load_notes_for_notebook(&notebook_id);
                                    }
                                }
                            }

                            ui.label(format!("({} 个笔记)", note_count));
                        });
                    }
                });

            ui.add_space(20.0);
            if ui.button("+ 创建新笔记本").clicked() {
                state.show_create_notebook = true;
            }
        }
    });
}

/// 处理对话框的函数
fn process_dialogs(ui: &mut egui::Ui, state: &mut DbINoteState) {
    if state.show_create_notebook {
        // Create a dialog window
        let mut created = false;
        let mut closed = false;

        egui::Window::new("创建新笔记本")
            .collapsible(false)
            .resizable(false)
            .fixed_size([300.0, 150.0])
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .open(&mut state.show_create_notebook)
            .show(ui.ctx(), |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(10.0);

                    ui.horizontal(|ui| {
                        ui.label("名称:");
                        ui.add(egui::TextEdit::singleline(&mut state.new_notebook_name)
                            .desired_width(200.0)
                            .hint_text("输入笔记本名称"));
                    });

                    ui.add_space(5.0);

                    ui.horizontal(|ui| {
                        ui.label("描述:");
                        ui.add(egui::TextEdit::singleline(&mut state.new_notebook_description)
                            .desired_width(200.0)
                            .hint_text("输入笔记本描述"));
                    });

                    ui.add_space(10.0);
                    ui.separator();
                    ui.add_space(10.0);

                    ui.horizontal(|ui| {
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("创建").clicked() && !state.new_notebook_name.is_empty() {
                                created = true;
                                closed = true;
                                log::info!("Create button clicked");
                            }

                            if ui.button("取消").clicked() {
                                closed = true;
                                log::info!("Cancel button clicked");
                            }
                        });
                    });
                });
            });

        // Create notebook if button was clicked
        if created {
            log::info!("Creating notebook: {}", state.new_notebook_name);
            state.create_notebook(
                state.new_notebook_name.clone(),
                state.new_notebook_description.clone()
            );

            // Reset values
            state.new_notebook_name.clear();
            state.new_notebook_description.clear();
        }

        if closed {
            state.show_create_notebook = false;
        }
    }

    if state.show_create_tag {
        // Create a dialog window
        let mut created = false;
        let mut closed = false;

        egui::Window::new("创建新标签")
            .collapsible(false)
            .resizable(false)
            .fixed_size([300.0, 150.0])
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .open(&mut state.show_create_tag)
            .show(ui.ctx(), |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(10.0);

                    ui.horizontal(|ui| {
                        ui.label("名称:");
                        ui.add(egui::TextEdit::singleline(&mut state.new_tag_name)
                            .desired_width(200.0)
                            .hint_text("输入标签名称"));
                    });

                    ui.add_space(5.0);

                    ui.horizontal(|ui| {
                        ui.label("颜色:");
                        let mut color = hex_to_color32(&state.new_tag_color);
                        if ui.color_edit_button_srgba(&mut color).changed() {
                            state.new_tag_color = format!("#{:02x}{:02x}{:02x}", color.r(), color.g(), color.b());
                        }

                        ui.label(state.new_tag_color.clone());
                    });

                    ui.add_space(10.0);
                    ui.separator();
                    ui.add_space(10.0);

                    ui.horizontal(|ui| {
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("创建").clicked() && !state.new_tag_name.is_empty() {
                                created = true;
                                closed = true;
                                log::info!("Create tag button clicked");
                            }

                            if ui.button("取消").clicked() {
                                closed = true;
                                log::info!("Cancel tag button clicked");
                            }
                        });
                    });
                });
            });

        // Create tag if button was clicked
        if created {
            log::info!("Creating tag: {}", state.new_tag_name);
            state.create_tag(
                state.new_tag_name.clone(),
                state.new_tag_color.clone()
            );

            // Reset values
            state.new_tag_name.clear();
        }

        if closed {
            state.show_create_tag = false;
        }
    }

    // 显示删除确认对话框
    if state.show_delete_confirmation {
        if let Some(confirmation) = &state.delete_confirmation {
            let mut confirmed = false;
            let mut cancelled = false;

            let title = match confirmation.confirmation_type {
                crate::db_state::DeleteConfirmationType::Notebook => "确认删除笔记本",
                crate::db_state::DeleteConfirmationType::Note => "确认删除笔记",
                crate::db_state::DeleteConfirmationType::Tag => "确认删除标签",
            };

            let message = match confirmation.confirmation_type {
                crate::db_state::DeleteConfirmationType::Notebook => {
                    format!("您确定要删除笔记本 \"{}\" 吗？\n\n⚠️ 警告：这将同时删除笔记本中的所有笔记！\n此操作无法撤销。", confirmation.target_name)
                },
                crate::db_state::DeleteConfirmationType::Note => {
                    format!("您确定要删除笔记 \"{}\" 吗？\n\n此操作无法撤销。", confirmation.target_name)
                },
                crate::db_state::DeleteConfirmationType::Tag => {
                    format!("您确定要删除标签 \"{}\" 吗？\n\n此操作将从所有笔记中移除该标签。", confirmation.target_name)
                },
            };

            egui::Window::new(title)
                .collapsible(false)
                .resizable(false)
                .fixed_size([400.0, 200.0])
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .open(&mut state.show_delete_confirmation)
                .show(ui.ctx(), |ui| {
                    ui.vertical_centered(|ui| {
                        ui.add_space(10.0);

                        // 显示警告图标和消息
                        ui.horizontal(|ui| {
                            ui.label("⚠️");
                            ui.vertical(|ui| {
                                for line in message.lines() {
                                    ui.label(line);
                                }
                            });
                        });

                        ui.add_space(20.0);
                        ui.separator();
                        ui.add_space(10.0);

                        ui.horizontal(|ui| {
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                // 删除按钮 - 红色
                                let delete_button = ui.add(
                                    egui::Button::new("🗑 确认删除")
                                        .fill(egui::Color32::from_rgb(220, 53, 69))
                                );
                                if delete_button.clicked() {
                                    confirmed = true;
                                }

                                ui.add_space(10.0);

                                // 取消按钮
                                if ui.button("取消").clicked() {
                                    cancelled = true;
                                }
                            });
                        });
                    });
                });

            if confirmed {
                state.confirm_deletion();
            } else if cancelled {
                state.cancel_deletion();
            }
        }
    }

    // 显示文档导入对话框
    if state.show_document_import_dialog {
        render_document_import_dialog(ui, state);
    }

    // 显示笔记本选择对话框（用于文档导入）
    if state.notebook_selector.show_dialog {
        let notebooks = state.notebooks.clone();
        if let Some(import_action) = crate::notebook_selector::render_notebook_selector_dialog(
            ui.ctx(),
            &mut state.notebook_selector,
            &notebooks,
        ) {
            let (selected_notebook_id, should_edit) = match import_action {
                crate::notebook_selector::ImportAction::Import(notebook_id) => (notebook_id, false),
                crate::notebook_selector::ImportAction::ImportAndEdit(notebook_id) => (notebook_id, true),
            };

            // 导入文档
            let file_path = state.notebook_selector.file_path.clone();
            match state.import_document_as_note(&file_path, &selected_notebook_id) {
                Ok(note_id) => {
                    log::info!("Successfully imported document as note: {}", note_id);

                    // 如果选择了"导入并编辑"，选择导入的笔记
                    if should_edit {
                        state.select_note(&note_id);
                        log::info!("Selected imported note for editing: {}", note_id);
                    }

                    // 重置对话框状态
                    state.notebook_selector.import_success = true;
                    state.notebook_selector.import_in_progress = false;
                },
                Err(error) => {
                    log::error!("Failed to import document: {}", error);
                    state.notebook_selector.import_error = Some(error);
                    state.notebook_selector.import_in_progress = false;
                }
            }
        }
    }
}

/// 渲染文档导入对话框
fn render_document_import_dialog(ui: &mut egui::Ui, state: &mut DbINoteState) {
    let mut file_selected = false;
    let mut dialog_closed = false;
    let mut selected_file_path = String::new();

    egui::Window::new("导入文档")
        .collapsible(false)
        .resizable(false)
        .fixed_size([400.0, 300.0])
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .open(&mut state.show_document_import_dialog)
        .show(ui.ctx(), |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(10.0);
                ui.heading("选择要导入的文档");
                ui.add_space(10.0);

                ui.label("支持的文档格式：");
                ui.horizontal(|ui| {
                    ui.label("📄 PDF");
                    ui.label("📝 DOCX");
                    ui.label("📊 PPTX");
                    ui.label("📄 TXT");
                    ui.label("📝 MD");
                });

                ui.add_space(20.0);

                // 文件选择按钮
                if ui.button("📁 选择文件").clicked() {
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("支持的文档", &["pdf", "docx", "pptx", "txt", "md"])
                        .add_filter("PDF文件", &["pdf"])
                        .add_filter("Word文档", &["docx"])
                        .add_filter("PowerPoint文档", &["pptx"])
                        .add_filter("文本文件", &["txt"])
                        .add_filter("Markdown文件", &["md"])
                        .pick_file()
                    {
                        selected_file_path = path.to_string_lossy().to_string();
                        file_selected = true;
                    }
                }

                ui.add_space(20.0);
                ui.separator();
                ui.add_space(10.0);

                ui.horizontal(|ui| {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("取消").clicked() {
                            dialog_closed = true;
                        }
                    });
                });
            });
        });

    if file_selected {
        // 关闭当前对话框
        state.show_document_import_dialog = false;

        // 获取文件名
        let file_name = std::path::Path::new(&selected_file_path)
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("未知文件")
            .to_string();

        // 显示笔记本选择对话框
        state.notebook_selector.show_for_file(selected_file_path, file_name);
    }

    if dialog_closed {
        state.show_document_import_dialog = false;
    }
}

/// Convert hex color string to egui::Color32
fn hex_to_color32(hex: &str) -> egui::Color32 {
    let hex = hex.trim_start_matches('#');

    if hex.len() == 6 {
        // Parse RGB
        if let (Ok(r), Ok(g), Ok(b)) = (
            u8::from_str_radix(&hex[0..2], 16),
            u8::from_str_radix(&hex[2..4], 16),
            u8::from_str_radix(&hex[4..6], 16),
        ) {
            return egui::Color32::from_rgb(r, g, b);
        }
    }

    // Default color if parsing fails
    egui::Color32::BLUE
}

/// Save note module settings
pub fn save_settings(state: &DbINoteState) -> Result<(), Box<dyn std::error::Error>> {
    use std::fs;
    use serde_json;

    let base_path = dirs::config_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
    let config_dir = base_path.join("seeu_desktop");
    let config_path = config_dir.join("inote_settings.json");

    fs::create_dir_all(&config_dir)?;

    let settings = serde_json::json!({
        "settings_default_collapse_notebooks": state.settings_default_collapse_notebooks,
        "settings_enable_markdown_preview": state.settings_enable_markdown_preview,
        "settings_show_note_stats": state.settings_show_note_stats,
        "settings_auto_save": state.settings_auto_save,
        "settings_syntax_highlight": state.settings_syntax_highlight,
        "settings_show_line_numbers": state.settings_show_line_numbers,
        "show_note_tree": state.show_note_tree
    });

    let json = serde_json::to_string_pretty(&settings)?;
    fs::write(config_path, json)?;

    log::info!("Note settings saved successfully");
    Ok(())
}

/// Load note module settings
pub fn load_settings(state: &mut DbINoteState) -> Result<(), Box<dyn std::error::Error>> {
    use std::fs;
    use serde_json;

    let base_path = dirs::config_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
    let config_path = base_path.join("seeu_desktop").join("inote_settings.json");

    if let Ok(content) = fs::read_to_string(config_path) {
        if let Ok(settings) = serde_json::from_str::<serde_json::Value>(&content) {
            if let Some(value) = settings.get("settings_default_collapse_notebooks").and_then(|v| v.as_bool()) {
                state.settings_default_collapse_notebooks = value;
            }
            if let Some(value) = settings.get("settings_enable_markdown_preview").and_then(|v| v.as_bool()) {
                state.settings_enable_markdown_preview = value;
            }
            if let Some(value) = settings.get("settings_show_note_stats").and_then(|v| v.as_bool()) {
                state.settings_show_note_stats = value;
            }
            if let Some(value) = settings.get("settings_auto_save").and_then(|v| v.as_bool()) {
                state.settings_auto_save = value;
            }
            if let Some(value) = settings.get("settings_syntax_highlight").and_then(|v| v.as_bool()) {
                state.settings_syntax_highlight = value;
            }
            if let Some(value) = settings.get("settings_show_line_numbers").and_then(|v| v.as_bool()) {
                state.settings_show_line_numbers = value;
            }
            if let Some(value) = settings.get("show_note_tree").and_then(|v| v.as_bool()) {
                state.show_note_tree = value;
            }

            log::info!("Note settings loaded successfully");
        }
    }

    Ok(())
}