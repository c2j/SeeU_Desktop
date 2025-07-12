//! 文件编辑器状态管理

use std::collections::HashMap;
use std::path::PathBuf;
use std::time::SystemTime;

use std::sync::mpsc::{self, Receiver, Sender};

/// 文件加载状态
#[derive(Debug, Clone)]
pub enum FileLoadingState {
    /// 未加载
    NotLoading,
    /// 正在加载
    Loading {
        path: PathBuf,
        progress: f32,
    },
    /// 加载完成
    Completed,
    /// 加载失败
    Failed(String),
}

/// 异步加载消息
#[derive(Debug)]
pub enum AsyncLoadMessage {
    /// 加载进度更新
    Progress {
        path: PathBuf,
        progress: f32,
        bytes_loaded: u64,
        total_bytes: u64,
    },
    /// 加载完成
    Completed {
        path: PathBuf,
        content: String,
        elapsed: std::time::Duration,
    },
    /// 加载失败
    Failed {
        path: PathBuf,
        error: String,
    },
}


use std::ops::Range;
use crop::Rope;
use egui_ltreeview::TreeViewState;
use crate::settings::EditorSettings;
use crate::{FileEditorError, FileEditorResult};

/// 文件路径包装器，用于实现NodeId trait
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FileNodeId(pub PathBuf);

impl From<PathBuf> for FileNodeId {
    fn from(path: PathBuf) -> Self {
        Self(path)
    }
}

impl From<FileNodeId> for PathBuf {
    fn from(node_id: FileNodeId) -> Self {
        node_id.0
    }
}

impl AsRef<PathBuf> for FileNodeId {
    fn as_ref(&self) -> &PathBuf {
        &self.0
    }
}

// NodeId trait 是一个 trait alias，FileNodeId 已经实现了所需的 traits

/// 文件编辑器主状态
#[derive(Debug)]
pub struct IFileEditorState {
    /// 是否已初始化
    pub initialized: bool,

    /// 工作区根目录
    pub workspace_root: Option<PathBuf>,

    /// 文件树状态
    pub file_tree: FileTreeState,

    /// 上一帧的展开状态，用于检测展开状态变化
    pub previous_expanded_dirs: std::collections::HashSet<PathBuf>,

    /// 上次刷新时间（用于防抖）
    pub last_refresh_time: Option<std::time::Instant>,

    /// 异步加载消息接收器
    pub async_load_receiver: Option<Receiver<AsyncLoadMessage>>,
    /// 异步加载消息发送器
    pub async_load_sender: Option<Sender<AsyncLoadMessage>>,

    /// 编辑器状态
    pub editor: EditorState,

    /// UI 状态
    pub ui_state: EditorUIState,

    /// 设置
    pub settings: EditorSettings,

    /// 设置管理器
    pub settings_manager: crate::settings::FileEditorSettingsModule,

    /// 文件监控器
    pub file_watcher: Option<notify::RecommendedWatcher>,

    /// 搜索集成状态
    pub pending_open_request: Option<OpenRequest>,

    /// 错误状态
    pub last_error: Option<FileEditorError>,
}

impl IFileEditorState {
    pub fn new() -> Self {
        let mut settings_manager = crate::settings::FileEditorSettingsModule::new();
        settings_manager.load_settings();
        let settings = settings_manager.get_settings().clone();

        // 创建异步加载通道
        let (sender, receiver) = mpsc::channel();

        Self {
            initialized: false,
            workspace_root: None,
            file_tree: FileTreeState::new(),
            previous_expanded_dirs: std::collections::HashSet::new(),
            last_refresh_time: None,
            async_load_receiver: Some(receiver),
            async_load_sender: Some(sender),
            editor: EditorState::new(),
            ui_state: EditorUIState::new(),
            settings,
            settings_manager,
            file_watcher: None,
            pending_open_request: None,
            last_error: None,
        }
    }
    
    /// 确保已初始化
    pub fn ensure_initialized(&mut self) {
        if !self.initialized {
            self.initialize();
        }
    }
    
    /// 初始化编辑器
    pub fn initialize(&mut self) {
        log::info!("Initializing file editor...");

        // 优先使用上次打开的目录
        if let Some(last_dir) = self.settings_manager.get_last_opened_directory() {
            self.workspace_root = Some(last_dir.clone());
            // 设置根路径并立即加载文件树
            if let Err(e) = self.file_tree.set_root(last_dir.clone(), &self.settings) {
                log::error!("Failed to load last opened directory: {}", e);
                // 如果加载失败，清除设置
                self.workspace_root = None;
                self.file_tree.root_path = None;
            } else {
                log::info!("Successfully loaded file tree from last session: {:?}", last_dir);
            }
        } else {
            log::info!("No previous directory found, user will need to select a directory");
        }

        // 初始化文件监控
        if let Err(e) = self.setup_file_watcher() {
            log::warn!("Failed to setup file watcher: {}", e);
        }

        self.initialized = true;
        log::info!("File editor initialized successfully");
    }

    /// 检查是否是首次使用（没有设置过目录）
    pub fn is_first_time_use(&self) -> bool {
        self.settings_manager.get_last_opened_directory().is_none()
    }

    /// 设置文件监控
    fn setup_file_watcher(&mut self) -> FileEditorResult<()> {
        use notify::{Watcher, RecursiveMode, Event};
        use std::sync::mpsc;

        let (tx, _rx) = mpsc::channel();

        let mut watcher = notify::recommended_watcher(move |res: Result<Event, notify::Error>| {
            match res {
                Ok(event) => {
                    // 处理文件事件
                    log::debug!("File event: {:?}", event);
                    if let Err(e) = tx.send(event) {
                        log::error!("Failed to send file event: {}", e);
                    }
                }
                Err(e) => log::error!("File watch error: {}", e),
            }
        }).map_err(|e| FileEditorError::IoError(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Failed to create file watcher: {}", e)
        )))?;

        // 监控工作区目录
        if let Some(workspace) = &self.workspace_root {
            watcher.watch(workspace, RecursiveMode::Recursive)
                .map_err(|e| FileEditorError::IoError(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Failed to watch directory: {}", e)
                )))?;
            log::info!("File watcher setup for: {:?}", workspace);
        }

        self.file_watcher = Some(watcher);

        // TODO: 在后台线程中处理文件事件
        // 这里需要一个事件处理机制来更新文件树

        Ok(())
    }

    /// 设置文件树根目录并保存到设置
    pub fn set_file_tree_root(&mut self, path: PathBuf) -> FileEditorResult<()> {
        self.file_tree.set_root(path.clone(), &self.settings)?;
        self.workspace_root = Some(path.clone());
        self.settings_manager.update_last_opened_directory(&path);
        // 添加到最近目录历史记录
        self.settings_manager.add_recent_directory(&path);
        log::info!("Updated file tree root and saved to settings: {:?}", path);
        Ok(())
    }
    
    /// 从搜索结果打开文件
    pub fn open_file_from_search(&mut self, path: PathBuf) -> FileEditorResult<()> {
        self.ensure_initialized();
        
        // 如果是文件，打开文件并导航到其目录
        if path.is_file() {
            // 设置文件树根目录为文件的父目录
            if let Some(parent) = path.parent() {
                self.file_tree.set_root(parent.to_path_buf(), &self.settings)?;
                // 选择文件
                self.file_tree.tree_view_state.set_one_selected(FileNodeId(path.clone()));
            }
            
            // 打开文件到编辑器
            self.editor.open_file(path.clone(), &self.settings, self.async_load_sender.clone())?;
            // 添加到最近文件历史记录
            self.settings_manager.add_recent_file(&path);
        } else {
            return Err(FileEditorError::FileNotFound { 
                path: path.to_string_lossy().to_string() 
            });
        }
        
        Ok(())
    }
    
    /// 获取最后的错误并清除
    pub fn take_error(&mut self) -> Option<FileEditorError> {
        self.last_error.take()
    }

    /// 处理异步加载消息（每帧调用）
    pub fn process_async_messages(&mut self) {
        if let Some(receiver) = &self.async_load_receiver {
            // 处理所有待处理的消息
            while let Ok(message) = receiver.try_recv() {
                match message {
                    AsyncLoadMessage::Progress { path, progress, bytes_loaded, total_bytes } => {
                        // 更新加载状态
                        self.editor.loading_state = FileLoadingState::Loading {
                            path: path.clone(),
                            progress,
                        };

                        log::debug!("Loading progress for {:?}: {:.1}% ({} / {} bytes)",
                                   path, progress * 100.0, bytes_loaded, total_bytes);
                    }
                    AsyncLoadMessage::Completed { path, content, elapsed } => {
                        log::info!("Async loading completed for {:?} in {:?}", path, elapsed);

                        // 创建真正的文本缓冲区
                        let buffer = TextBuffer {
                            file_path: path.clone(),
                            rope: Rope::from(content),
                            encoding: "UTF-8".to_string(),
                            line_ending: LineEnding::Unix,
                            modified: false,
                            last_saved: SystemTime::now(),
                            read_only: false, // 加载完成后可编辑
                            cursor: Cursor::default(),
                            selection: None,
                            language: None, // TODO: 实现语言检测
                            scroll_offset: 0.0,
                            visible_lines: 0..0,
                        };

                        // 替换占位符缓冲区
                        self.editor.buffers.insert(path.clone(), buffer);
                        self.editor.loading_state = FileLoadingState::NotLoading;

                        log::info!("File buffer updated successfully: {:?}", path);
                    }
                    AsyncLoadMessage::Failed { path, error } => {
                        log::error!("Async loading failed for {:?}: {}", path, error);
                        self.editor.loading_state = FileLoadingState::Failed(error);
                    }
                }
            }
        }
    }
    
    /// 更新设置
    pub fn update_settings(&mut self, settings: EditorSettings) {
        self.settings = settings;
        // 应用设置到编辑器
        self.editor.apply_settings(&self.settings);
    }

    /// 打开文件对话框
    pub fn open_file_dialog(&mut self) {
        let initial_dir = self.workspace_root.clone()
            .or_else(|| dirs::home_dir())
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());

        // 创建文件对话框
        let file_dialog = rfd::FileDialog::new()
            .set_title("打开文件")
            .set_directory(&initial_dir)
            .add_filter("文本文件", &["txt", "md", "markdown"])
            .add_filter("代码文件", &["rs", "py", "js", "ts", "jsx", "tsx", "java", "c", "cpp", "h", "hpp"])
            .add_filter("配置文件", &["json", "toml", "yaml", "yml", "xml", "ini", "cfg"])
            .add_filter("Web文件", &["html", "htm", "css", "scss", "sass", "less"])
            .add_filter("脚本文件", &["sh", "bash", "zsh", "fish", "ps1", "bat", "cmd"])
            .add_filter("所有文件", &["*"]);

        // 显示对话框并处理结果
        if let Some(path) = file_dialog.pick_file() {
            log::info!("User selected file: {:?}", path);

            // 打开选中的文件
            if let Err(e) = self.editor.open_file(path.clone(), &self.settings, self.async_load_sender.clone()) {
                log::error!("Failed to open file: {}", e);
                self.last_error = Some(e);
            } else {
                // 添加到最近文件历史记录
                self.settings_manager.add_recent_file(&path);
                // 如果文件不在当前工作区，更新工作区到文件的父目录
                if let Some(parent) = path.parent() {
                    if self.workspace_root.as_ref() != Some(&parent.to_path_buf()) {
                        if let Err(e) = self.set_workspace(parent.to_path_buf()) {
                            log::warn!("Failed to update workspace: {}", e);
                        }
                    }
                }

                // 在文件树中选择文件
                self.file_tree.tree_view_state.set_one_selected(FileNodeId(path.clone()));
                log::info!("Successfully opened file: {:?}", path);
            }
        } else {
            log::info!("User cancelled file dialog");
        }
    }

    /// 打开文件夹对话框
    pub fn open_folder_dialog(&mut self) {
        let initial_dir = self.workspace_root.clone()
            .or_else(|| dirs::home_dir())
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());

        // 创建文件夹对话框
        let folder_dialog = rfd::FileDialog::new()
            .set_title("选择工作文件夹")
            .set_directory(&initial_dir);

        // 显示对话框并处理结果
        if let Some(path) = folder_dialog.pick_folder() {
            log::info!("User selected folder: {:?}", path);

            if let Err(e) = self.set_workspace(path) {
                log::error!("Failed to set workspace: {}", e);
                self.last_error = Some(e);
            } else {
                log::info!("Successfully set workspace");
            }
        } else {
            log::info!("User cancelled folder dialog");
        }
    }

    /// 设置工作目录
    pub fn set_workspace(&mut self, path: PathBuf) -> FileEditorResult<()> {
        if !path.exists() || !path.is_dir() {
            return Err(FileEditorError::FileNotFound {
                path: path.to_string_lossy().to_string(),
            });
        }

        self.workspace_root = Some(path.clone());
        self.file_tree.set_root(path.clone(), &self.settings)?;
        // 保存设置到文件
        self.settings_manager.update_last_opened_directory(&path);
        // 添加到最近目录历史记录
        self.settings_manager.add_recent_directory(&path);
        log::info!("Updated workspace and saved to settings: {:?}", path);
        Ok(())
    }

    /// 处理待处理的打开请求
    pub fn process_pending_open_requests(&mut self) -> FileEditorResult<()> {
        if let Some(request) = self.pending_open_request.take() {
            match request.request_type {
                OpenRequestType::File => {
                    self.open_file_from_search(request.path)?;
                }
                OpenRequestType::Directory => {
                    self.file_tree.open_from_search(request.path, true, &self.settings)?;
                }
            }
        }
        Ok(())
    }

    /// 添加打开请求（从搜索模块调用）
    pub fn request_open(&mut self, path: PathBuf, is_directory: bool, source: String) {
        let request_type = if is_directory {
            OpenRequestType::Directory
        } else {
            OpenRequestType::File
        };

        self.pending_open_request = Some(OpenRequest {
            path,
            request_type,
            source_module: source,
        });
    }

    /// 获取当前文件的状态信息（用于状态栏显示）
    pub fn get_current_file_status(&self) -> Option<FileStatusInfo> {
        if let Some(buffer) = self.editor.get_active_buffer() {
            Some(FileStatusInfo {
                file_path: buffer.file_path.clone(),
                encoding: buffer.encoding.clone(),
                language: buffer.language.clone(),
                line_count: buffer.line_count(),
                char_count: buffer.char_count(),
                byte_count: buffer.byte_count(),
                modified: buffer.modified,
                read_only: buffer.read_only,
                cursor_line: buffer.cursor.line + 1, // 1-based line number
                cursor_column: buffer.cursor.column + 1, // 1-based column number
            })
        } else {
            None
        }
    }

    /// 防抖刷新文件树（避免频繁刷新）
    pub fn refresh_file_tree_debounced(&mut self, min_interval_ms: u64) -> FileEditorResult<bool> {
        let now = std::time::Instant::now();

        // 检查是否需要防抖
        if let Some(last_refresh) = self.last_refresh_time {
            if now.duration_since(last_refresh).as_millis() < min_interval_ms as u128 {
                return Ok(false); // 跳过刷新
            }
        }

        self.file_tree.refresh(&self.settings)?;
        self.last_refresh_time = Some(now);
        Ok(true) // 执行了刷新
    }

    /// 获取当前文件信息（用于AI助手上下文）
    pub fn get_current_file_context(&self) -> Option<FileContext> {
        if let Some(buffer) = self.editor.get_active_buffer() {
            let selected_text = if let Some(selection) = &buffer.selection {
                // 获取选中的文本
                let start_byte = self.cursor_to_byte_offset(&selection.start, buffer);
                let end_byte = self.cursor_to_byte_offset(&selection.end, buffer);
                if start_byte < end_byte {
                    Some(buffer.rope.byte_slice(start_byte..end_byte).to_string())
                } else {
                    None
                }
            } else {
                None
            };

            Some(FileContext {
                file_path: buffer.file_path.clone(),
                file_name: buffer.file_path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("未知文件")
                    .to_string(),
                language: buffer.language.clone(),
                content: buffer.rope.to_string(),
                selected_text,
                cursor_line: buffer.cursor.line + 1,
                cursor_column: buffer.cursor.column + 1,
                total_lines: buffer.line_count(),
                is_modified: buffer.modified,
                is_read_only: buffer.read_only,
            })
        } else {
            None
        }
    }

    /// 将光标位置转换为字节偏移量
    fn cursor_to_byte_offset(&self, cursor: &Cursor, buffer: &TextBuffer) -> usize {
        // 简化实现：使用行和列计算字节偏移
        let mut byte_offset = 0;
        let content = buffer.rope.to_string();
        let lines: Vec<&str> = content.lines().collect();

        // 添加前面行的字节数
        for (line_idx, line) in lines.iter().enumerate() {
            if line_idx >= cursor.line {
                break;
            }
            byte_offset += line.len() + 1; // +1 for newline
        }

        // 添加当前行的列偏移
        if cursor.line < lines.len() {
            let line = lines[cursor.line];
            let char_offset = cursor.column.min(line.chars().count());
            byte_offset += line.chars().take(char_offset).map(|c| c.len_utf8()).sum::<usize>();
        }

        byte_offset.min(buffer.rope.byte_len())
    }
}

impl Default for IFileEditorState {
    fn default() -> Self {
        Self::new()
    }
}

/// 文件加载状态


/// 编辑器状态
#[derive(Debug)]
pub struct EditorState {
    /// 打开的文件缓冲区
    pub buffers: HashMap<PathBuf, TextBuffer>,

    /// 标签页管理
    pub tabs: Vec<PathBuf>,
    pub active_tab: Option<usize>,

    /// 编辑历史
    pub undo_stack: HashMap<PathBuf, UndoStack>,

    /// 文件加载状态
    pub loading_state: FileLoadingState,
}

impl EditorState {
    pub fn new() -> Self {
        Self {
            buffers: HashMap::new(),
            tabs: Vec::new(),
            active_tab: None,
            undo_stack: HashMap::new(),
            loading_state: FileLoadingState::NotLoading,
        }
    }
    
    /// 打开文件（异步加载优化）
    pub fn open_file(&mut self, path: PathBuf, settings: &EditorSettings, async_sender: Option<Sender<AsyncLoadMessage>>) -> FileEditorResult<()> {
        // 检查文件是否已经打开
        if self.buffers.contains_key(&path) {
            // 切换到已打开的文件（快速切换）
            if let Some(index) = self.tabs.iter().position(|p| p == &path) {
                self.active_tab = Some(index);
                log::info!("Switched to already opened file: {:?}", path);
            }
            return Ok(());
        }

        // 检查文件大小，决定加载策略
        let metadata = std::fs::metadata(&path)?;
        let size_mb = metadata.len() as f64 / (1024.0 * 1024.0);

        if size_mb > 1.0 {
            // 大文件：异步加载
            log::info!("Starting async loading for large file ({:.1} MB): {:?}", size_mb, path);

            // 立即创建占位符缓冲区
            let placeholder_buffer = self.create_placeholder_buffer(&path, size_mb);
            self.buffers.insert(path.clone(), placeholder_buffer);
            self.tabs.push(path.clone());
            self.active_tab = Some(self.tabs.len() - 1);
            self.undo_stack.insert(path.clone(), UndoStack::new());

            // 设置加载状态
            self.loading_state = FileLoadingState::Loading {
                path: path.clone(),
                progress: 0.0,
            };

            // 启动异步加载
            if let Some(sender) = async_sender {
                self.start_async_loading(path, settings.clone(), sender);
            }

            Ok(())
        } else {
            // 小文件：直接同步加载
            match TextBuffer::from_file(&path, settings) {
                Ok(buffer) => {
                    self.buffers.insert(path.clone(), buffer);
                    self.tabs.push(path.clone());
                    self.active_tab = Some(self.tabs.len() - 1);
                    self.undo_stack.insert(path.clone(), UndoStack::new());
                    log::info!("Small file loaded successfully ({:.1} MB): {:?}", size_mb, path);
                    Ok(())
                }
                Err(e) => {
                    self.loading_state = FileLoadingState::Failed(e.to_string());
                    Err(e)
                }
            }
        }
    }



    /// 应用设置
    pub fn apply_settings(&mut self, settings: &EditorSettings) {
        // 更新所有缓冲区的设置
        for buffer in self.buffers.values_mut() {
            buffer.apply_settings(settings);
        }
    }
    
    /// 获取当前活动的缓冲区
    pub fn get_active_buffer(&self) -> Option<&TextBuffer> {
        if let Some(index) = self.active_tab {
            if let Some(path) = self.tabs.get(index) {
                return self.buffers.get(path);
            }
        }
        None
    }
    
    /// 获取当前活动的可变缓冲区
    pub fn get_active_buffer_mut(&mut self) -> Option<&mut TextBuffer> {
        if let Some(index) = self.active_tab {
            if let Some(path) = self.tabs.get(index).cloned() {
                return self.buffers.get_mut(&path);
            }
        }
        None
    }

    /// 保存当前活动文件
    pub fn save_active_file(&mut self) -> FileEditorResult<()> {
        if let Some(active_index) = self.active_tab {
            if let Some(path) = self.tabs.get(active_index).cloned() {
                return self.save_file(&path);
            }
        }
        Err(FileEditorError::FileNotFound {
            path: "No active file".to_string(),
        })
    }

    /// 保存指定文件
    pub fn save_file(&mut self, path: &PathBuf) -> FileEditorResult<()> {
        if let Some(buffer) = self.buffers.get_mut(path) {
            let content = buffer.rope.to_string();

            // 写入文件
            std::fs::write(path, content.as_bytes())
                .map_err(|e| FileEditorError::IoError(e))?;

            // 更新缓冲区状态
            buffer.modified = false;
            buffer.last_saved = SystemTime::now();

            log::info!("File saved: {:?}", path);
            Ok(())
        } else {
            Err(FileEditorError::FileNotFound {
                path: path.to_string_lossy().to_string(),
            })
        }
    }

    /// 另存为
    pub fn save_file_as(&mut self, old_path: &PathBuf, new_path: PathBuf) -> FileEditorResult<()> {
        if let Some(buffer) = self.buffers.get(old_path) {
            let content = buffer.rope.to_string();

            // 写入新文件
            std::fs::write(&new_path, content.as_bytes())
                .map_err(|e| FileEditorError::IoError(e))?;

            // 创建新缓冲区
            let mut new_buffer = buffer.clone();
            new_buffer.file_path = new_path.clone();
            new_buffer.modified = false;
            new_buffer.last_saved = SystemTime::now();

            // 添加到缓冲区和标签页
            self.buffers.insert(new_path.clone(), new_buffer);
            self.tabs.push(new_path.clone());
            self.active_tab = Some(self.tabs.len() - 1);

            log::info!("File saved as: {:?}", new_path);
            Ok(())
        } else {
            Err(FileEditorError::FileNotFound {
                path: old_path.to_string_lossy().to_string(),
            })
        }
    }

    /// 关闭文件
    pub fn close_file(&mut self, path: &PathBuf) -> FileEditorResult<()> {
        // 移除缓冲区
        self.buffers.remove(path);
        self.undo_stack.remove(path);

        // 移除标签页
        if let Some(index) = self.tabs.iter().position(|p| p == path) {
            self.tabs.remove(index);

            // 调整活动标签页索引
            if let Some(active_index) = self.active_tab {
                if active_index == index {
                    // 关闭的是活动标签页
                    if self.tabs.is_empty() {
                        self.active_tab = None;
                    } else if index > 0 {
                        self.active_tab = Some(index - 1);
                    } else {
                        self.active_tab = Some(0);
                    }
                } else if active_index > index {
                    // 调整索引
                    self.active_tab = Some(active_index - 1);
                }
            }
        }

        log::info!("File closed: {:?}", path);
        Ok(())
    }

    /// 创建占位符缓冲区
    fn create_placeholder_buffer(&self, path: &PathBuf, size_mb: f64) -> TextBuffer {
        let placeholder_content = format!(
            "📄 正在加载大文件...\n\n\
            文件路径: {}\n\
            文件大小: {:.1} MB\n\n\
            ⏳ 正在后台加载中，请稍候...\n\
            💡 提示：大文件加载完成后将自动刷新显示\n\n\
            🚀 使用异步加载技术，界面不会阻塞",
            path.display(),
            size_mb
        );

        TextBuffer {
            file_path: path.clone(),
            rope: Rope::from(placeholder_content),
            encoding: "UTF-8".to_string(),
            line_ending: LineEnding::Unix,
            modified: false,
            last_saved: SystemTime::now(),
            read_only: true, // 占位符期间只读
            cursor: Cursor::default(),
            selection: None,
            language: None,
            scroll_offset: 0.0,
            visible_lines: 0..0,
        }
    }

    /// 启动异步加载
    fn start_async_loading(&self, path: PathBuf, settings: EditorSettings, sender: Sender<AsyncLoadMessage>) {
        let path_clone = path.clone();

        // 在新线程中执行文件加载
        std::thread::spawn(move || {
            let start_time = std::time::Instant::now();

            match Self::load_file_async(&path_clone, &settings, &sender) {
                Ok(content) => {
                    let elapsed = start_time.elapsed();
                    log::info!("Async file loading completed in {:?}: {:?}", elapsed, path_clone);

                    // 发送完成消息
                    if let Err(e) = sender.send(AsyncLoadMessage::Completed {
                        path: path_clone,
                        content,
                        elapsed,
                    }) {
                        log::error!("Failed to send completion message: {}", e);
                    }
                }
                Err(e) => {
                    log::error!("Async file loading failed: {:?} - {}", path_clone, e);

                    // 发送失败消息
                    if let Err(send_err) = sender.send(AsyncLoadMessage::Failed {
                        path: path_clone,
                        error: e.to_string(),
                    }) {
                        log::error!("Failed to send error message: {}", send_err);
                    }
                }
            }
        });
    }

    /// 异步加载文件内容
    fn load_file_async(path: &PathBuf, _settings: &EditorSettings, sender: &Sender<AsyncLoadMessage>) -> FileEditorResult<String> {
        use std::io::{BufReader, Read};
        use std::fs::File;

        let file = File::open(path)?;
        let metadata = file.metadata()?;
        let total_size = metadata.len();
        let mut reader = BufReader::new(file);
        let mut content = String::new();

        // 分块读取，定期发送进度更新
        const CHUNK_SIZE: usize = 64 * 1024; // 64KB chunks
        let mut buffer = vec![0; CHUNK_SIZE];
        let mut bytes_loaded = 0u64;

        loop {
            match reader.read(&mut buffer) {
                Ok(0) => break, // EOF
                Ok(bytes_read) => {
                    // 转换为字符串
                    let chunk = String::from_utf8_lossy(&buffer[..bytes_read]);
                    content.push_str(&chunk);
                    bytes_loaded += bytes_read as u64;

                    // 发送进度更新
                    let progress = bytes_loaded as f32 / total_size as f32;
                    if let Err(e) = sender.send(AsyncLoadMessage::Progress {
                        path: path.clone(),
                        progress,
                        bytes_loaded,
                        total_bytes: total_size,
                    }) {
                        log::warn!("Failed to send progress update: {}", e);
                    }

                    // 每读取1MB就让出一点时间
                    if bytes_loaded % (1024 * 1024) == 0 {
                        std::thread::sleep(std::time::Duration::from_millis(1));
                    }
                }
                Err(e) => return Err(FileEditorError::IoError(e)),
            }
        }

        Ok(content)
    }
}

/// 文本缓冲区
#[derive(Debug, Clone)]
pub struct TextBuffer {
    /// 文本内容 (ROPE)
    pub rope: Rope,
    
    /// 文件信息
    pub file_path: PathBuf,
    pub encoding: String,
    pub line_ending: LineEnding,
    
    /// 编辑状态
    pub modified: bool,
    pub last_saved: SystemTime,
    pub read_only: bool,
    
    /// 光标和选择
    pub cursor: Cursor,
    pub selection: Option<Selection>,
    
    /// 语法高亮
    pub language: Option<String>,
    
    /// 视图状态
    pub scroll_offset: f32,
    pub visible_lines: Range<usize>,
}

impl TextBuffer {
    /// 从文件创建文本缓冲区（优化大文件加载）
    pub fn from_file(path: &PathBuf, settings: &EditorSettings) -> FileEditorResult<Self> {
        // 检查文件大小
        let metadata = std::fs::metadata(path)?;
        let size_mb = metadata.len() / (1024 * 1024);

        if size_mb > settings.max_file_size_mb as u64 {
            return Err(FileEditorError::FileTooLarge {
                size_mb: size_mb as usize,
                max_mb: settings.max_file_size_mb,
            });
        }

        // 检查文件权限
        if let Err(e) = std::fs::File::open(path) {
            return Err(FileEditorError::PermissionDenied {
                path: format!("{}: {}", path.display(), e),
            });
        }

        // 文件大小分类：
        // 小文件：≤64KB - 直接读取，最快响应
        // 中等文件：64KB-1MB - 带进度读取，平衡性能和体验
        // 大文件：>1MB - 分块读取，避免阻塞UI
        let size_kb = metadata.len() as f64 / 1024.0;

        let content = if size_kb <= 64.0 {
            // 小文件（≤64KB）：直接读取，最快响应
            log::debug!("Reading small file ({:.1} KB) directly: {:?}", size_kb, path);
            std::fs::read_to_string(path)
                .map_err(|e| {
                    log::error!("Failed to read file {:?}: {}", path, e);
                    FileEditorError::EncodingError {
                        encoding: format!("UTF-8 (IO Error: {})", e),
                    }
                })?
        } else if size_kb <= 1024.0 {
            // 中等文件（64KB-1MB）：带进度读取
            log::info!("Reading medium file ({:.1} KB) with progress: {:?}", size_kb, path);
            Self::read_file_with_progress(path, |progress| {
                log::debug!("File loading progress: {:.1}%", progress * 100.0);
            })?
        } else {
            // 大文件（>1MB）：分块读取避免阻塞
            log::info!("Reading large file ({:.1} KB) with chunked loading: {:?}", size_kb, path);
            Self::read_file_chunked(path)?
        };

        // 创建 ROPE（ROPE 本身就是高效的，支持大文件）
        let rope = Rope::from(content);

        // 检测语言
        let language = detect_language(path);

        // 检测文件权限
        let read_only = metadata.permissions().readonly() || !is_editable_file(path);

        Ok(Self {
            rope,
            file_path: path.clone(),
            encoding: "UTF-8".to_string(),
            line_ending: LineEnding::Unix,
            modified: false,
            last_saved: metadata.modified().unwrap_or(SystemTime::now()),
            read_only,
            cursor: Cursor::default(),
            selection: None,
            language,
            scroll_offset: 0.0,
            visible_lines: 0..0,
        })
    }

    /// 分块读取大文件，避免阻塞UI
    fn read_file_chunked(path: &PathBuf) -> FileEditorResult<String> {
        use std::io::{BufReader, Read};
        use std::fs::File;

        let file = File::open(path)?;
        let mut reader = BufReader::new(file);
        let mut content = String::new();

        // 分块读取，每次读取64KB
        const CHUNK_SIZE: usize = 64 * 1024;
        let mut buffer = vec![0; CHUNK_SIZE];
        let mut total_read = 0;

        loop {
            match reader.read(&mut buffer) {
                Ok(0) => break, // EOF
                Ok(n) => {
                    // 将字节转换为字符串
                    let chunk = String::from_utf8_lossy(&buffer[..n]);
                    content.push_str(&chunk);
                    total_read += n;

                    // 性能优化：每读取1MB数据就让出控制权
                    if total_read % (1024 * 1024) == 0 {
                        // 在实际应用中，这里可以使用 tokio::task::yield_now()
                        // 或者其他方式让出控制权给UI线程
                        std::thread::yield_now();
                        log::debug!("Read {} MB from file: {:?}", total_read / (1024 * 1024), path);
                    }
                }
                Err(e) => return Err(FileEditorError::IoError(e)),
            }
        }

        log::info!("Completed reading {} bytes from file: {:?}", total_read, path);
        Ok(content)
    }

    /// 优化的大文件读取（带进度回调）
    fn read_file_with_progress<F>(path: &PathBuf, progress_callback: F) -> FileEditorResult<String>
    where
        F: Fn(f32) // 进度回调，0.0 到 1.0
    {
        use std::io::{BufReader, Read, Seek, SeekFrom};
        use std::fs::File;

        let mut file = File::open(path)?;
        let file_size = file.seek(SeekFrom::End(0))? as usize;
        file.seek(SeekFrom::Start(0))?;

        let mut reader = BufReader::new(file);
        let mut content = String::with_capacity(file_size.min(10 * 1024 * 1024)); // 预分配内存，最多10MB

        // 分块读取，每次读取64KB
        const CHUNK_SIZE: usize = 64 * 1024;
        let mut buffer = vec![0; CHUNK_SIZE];
        let mut total_read = 0;

        loop {
            match reader.read(&mut buffer) {
                Ok(0) => break, // EOF
                Ok(n) => {
                    // 将字节转换为字符串
                    let chunk = String::from_utf8_lossy(&buffer[..n]);
                    content.push_str(&chunk);
                    total_read += n;

                    // 更新进度
                    let progress = if file_size > 0 {
                        (total_read as f32) / (file_size as f32)
                    } else {
                        1.0
                    };
                    progress_callback(progress);

                    // 每读取256KB就让出控制权
                    if total_read % (256 * 1024) == 0 {
                        std::thread::yield_now();
                    }
                }
                Err(e) => return Err(FileEditorError::IoError(e)),
            }
        }

        Ok(content)
    }

    /// 应用设置
    pub fn apply_settings(&mut self, _settings: &EditorSettings) {
        // 应用编辑器设置到缓冲区
        // 例如：更新语法高亮、字体等
    }
    
    /// 获取行数
    pub fn line_count(&self) -> usize {
        self.rope.lines().count()
    }

    /// 获取字节数
    pub fn byte_count(&self) -> usize {
        self.rope.bytes().count()
    }

    /// 获取字符数
    pub fn char_count(&self) -> usize {
        self.rope.chars().count()
    }

    /// 插入文本
    pub fn insert_text(&mut self, position: usize, text: &str) {
        self.rope.insert(position, text);
        self.modified = true;
        self.update_cursor_after_insert(position, text);
    }

    /// 删除文本
    pub fn delete_text(&mut self, range: std::ops::Range<usize>) {
        if range.start < range.end && range.end <= self.rope.byte_len() {
            self.rope.delete(range.clone());
            self.modified = true;
            self.update_cursor_after_delete(range);
        }
    }

    /// 替换文本
    pub fn replace_text(&mut self, range: std::ops::Range<usize>, text: &str) {
        if range.start < range.end && range.end <= self.rope.byte_len() {
            self.rope.replace(range, text);
            self.modified = true;
            self.update_cursor_after_replace_simple(text.len());
        }
    }

    /// 获取指定范围的文本
    pub fn get_text_range(&self, range: std::ops::Range<usize>) -> String {
        if range.start < range.end && range.end <= self.rope.byte_len() {
            self.rope.byte_slice(range).to_string()
        } else {
            String::new()
        }
    }

    /// 获取指定行的文本
    pub fn get_line_text(&self, line_index: usize) -> String {
        if line_index < self.line_count() {
            self.rope.line(line_index).to_string()
        } else {
            String::new()
        }
    }

    /// 更新插入后的光标位置
    fn update_cursor_after_insert(&mut self, position: usize, text: &str) {
        if self.cursor.byte_offset >= position {
            self.cursor.byte_offset += text.len();
            self.update_cursor_line_column();
        }
    }

    /// 更新删除后的光标位置
    fn update_cursor_after_delete(&mut self, range: std::ops::Range<usize>) {
        if self.cursor.byte_offset >= range.end {
            self.cursor.byte_offset -= range.len();
        } else if self.cursor.byte_offset > range.start {
            self.cursor.byte_offset = range.start;
        }
        self.update_cursor_line_column();
    }



    /// 根据字节偏移更新行列位置
    pub fn update_cursor_line_column(&mut self) {
        let text = self.rope.byte_slice(0..self.cursor.byte_offset).to_string();
        let mut line = 0;
        let mut column = 0;

        for ch in text.chars() {
            if ch == '\n' {
                line += 1;
                column = 0;
            } else {
                column += 1;
            }
        }

        self.cursor.line = line;
        self.cursor.column = column;
    }

    /// 设置光标位置（按字节偏移）
    pub fn set_cursor_position(&mut self, byte_offset: usize) {
        self.cursor.byte_offset = byte_offset.min(self.rope.byte_len());
        self.update_cursor_line_column();
    }

    /// 简化的替换后光标更新
    fn update_cursor_after_replace_simple(&mut self, _new_text_len: usize) {
        // 简化实现：将光标移动到替换文本的末尾
        self.update_cursor_line_column();
    }

    /// 设置光标位置（按行列）
    pub fn set_cursor_line_column(&mut self, line: usize, column: usize) {
        let line = line.min(self.line_count().saturating_sub(1));
        let line_text = self.get_line_text(line);
        let column = column.min(line_text.chars().count());

        // 计算字节偏移
        let mut byte_offset = 0;
        for i in 0..line {
            byte_offset += self.get_line_text(i).len();
            if i < line {
                byte_offset += 1; // 换行符
            }
        }

        // 添加列偏移
        let line_chars: Vec<char> = line_text.chars().collect();
        for i in 0..column {
            if i < line_chars.len() {
                byte_offset += line_chars[i].len_utf8();
            }
        }

        self.cursor.byte_offset = byte_offset;
        self.cursor.line = line;
        self.cursor.column = column;
    }

    /// 获取行高估算值（用于滚动计算）
    pub fn get_line_height_estimate(&self) -> f32 {
        // 基于字体大小估算行高
        // 通常行高 = 字体大小 * 1.2 到 1.5
        16.0 // 默认行高，可以根据实际字体大小调整
    }
}

/// 光标位置
#[derive(Debug, Clone, Copy, Default)]
pub struct Cursor {
    pub line: usize,
    pub column: usize,
    pub byte_offset: usize,
}

/// 选择区域
#[derive(Debug, Clone)]
pub struct Selection {
    pub start: Cursor,
    pub end: Cursor,
}

/// 行结束符类型
#[derive(Debug, Clone)]
pub enum LineEnding {
    Unix,    // \n
    Windows, // \r\n
    Mac,     // \r
}

/// 撤销栈
#[derive(Debug)]
pub struct UndoStack {
    pub operations: Vec<EditOperation>,
    pub current_index: usize,
}

impl UndoStack {
    pub fn new() -> Self {
        Self {
            operations: Vec::new(),
            current_index: 0,
        }
    }
}

/// 编辑操作
#[derive(Debug, Clone)]
pub struct EditOperation {
    pub operation_type: OperationType,
    pub range: Range<usize>,
    pub text: String,
    pub cursor_before: Cursor,
    pub cursor_after: Cursor,
}

/// 操作类型
#[derive(Debug, Clone)]
pub enum OperationType {
    Insert,
    Delete,
    Replace,
}

/// 文件树状态
pub struct FileTreeState {
    pub root_path: Option<PathBuf>,
    pub tree_view_state: TreeViewState<FileNodeId>,
    pub file_entries: HashMap<PathBuf, FileEntry>,
    pub directory_children: HashMap<PathBuf, Vec<PathBuf>>,
}

impl std::fmt::Debug for FileTreeState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FileTreeState")
            .field("root_path", &self.root_path)
            .field("file_entries", &self.file_entries.len())
            .field("directory_children", &self.directory_children.len())
            .finish()
    }
}

impl FileTreeState {
    pub fn new() -> Self {
        Self {
            root_path: None,
            tree_view_state: TreeViewState::default(),
            file_entries: HashMap::new(),
            directory_children: HashMap::new(),
        }
    }

    /// 设置根目录
    pub fn set_root(&mut self, path: PathBuf, settings: &EditorSettings) -> FileEditorResult<()> {
        self.root_path = Some(path.clone());
        self.refresh(settings)?;
        Ok(())
    }

    /// 刷新文件树
    pub fn refresh(&mut self, settings: &EditorSettings) -> FileEditorResult<()> {
        if let Some(root) = self.root_path.clone() {
            self.scan_directory_recursive(&root, settings)?;
        }
        Ok(())
    }



    /// 扫描目录并构建文件条目映射（非递归，懒加载）
    fn scan_directory_recursive(&mut self, path: &PathBuf, settings: &EditorSettings) -> FileEditorResult<()> {
        self.file_entries.clear();
        self.directory_children.clear();

        // 只扫描根目录，不递归扫描子目录
        self.scan_single_directory(path, settings)?;
        Ok(())
    }

    /// 扫描单个目录（非递归）
    fn scan_single_directory(&mut self, path: &PathBuf, settings: &EditorSettings) -> FileEditorResult<()> {
        if !path.exists() {
            return Ok(());
        }

        // 性能优化：如果不启用懒加载，限制扫描深度
        if !settings.lazy_loading {
            log::warn!("Lazy loading is disabled, this may impact performance for large directories");
        }

        let start_time = std::time::Instant::now();
        let entries = std::fs::read_dir(path)?;
        let mut children = Vec::new();
        let mut processed_count = 0;

        for entry in entries {
            let entry = entry?;
            let entry_path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();

            // 根据设置决定是否跳过隐藏文件
            if name.starts_with('.') && !settings.show_hidden_files {
                continue;
            }

            // 跳过特定的隐藏目录（即使设置显示隐藏文件也跳过这些）
            if self.should_skip_directory(&name) {
                continue;
            }

            processed_count += 1;

            // 性能优化：对于大目录，限制一次性加载的文件数量
            if processed_count > 1000 {
                log::warn!("Directory {} contains more than 1000 items, truncating for performance", path.display());
                break;
            }

            let is_dir = entry_path.is_dir();
            let metadata = entry.metadata().ok();

            let file_entry = FileEntry {
                path: entry_path.clone(),
                name: name.clone(),
                is_dir,
                icon: self.get_file_icon(&entry_path).to_string(),
                size: metadata.as_ref().and_then(|m| if !is_dir { Some(m.len()) } else { None }),
                modified: metadata.and_then(|m| m.modified().ok()),
            };

            self.file_entries.insert(entry_path.clone(), file_entry);
            children.push(entry_path.clone());

            // 不递归扫描子目录，实现懒加载
            // 子目录将在用户展开时按需加载
        }

        // 排序：目录在前，文件在后，按名称排序
        children.sort_by(|a, b| {
            let a_entry = self.file_entries.get(a);
            let b_entry = self.file_entries.get(b);

            match (a_entry, b_entry) {
                (Some(a_entry), Some(b_entry)) => {
                    match (a_entry.is_dir, b_entry.is_dir) {
                        (true, false) => std::cmp::Ordering::Less,
                        (false, true) => std::cmp::Ordering::Greater,
                        _ => a_entry.name.cmp(&b_entry.name),
                    }
                }
                _ => std::cmp::Ordering::Equal,
            }
        });

        self.directory_children.insert(path.clone(), children);

        let elapsed = start_time.elapsed();
        if elapsed.as_millis() > 100 {
            log::warn!("Directory scan took {}ms for {} items in {:?}",
                      elapsed.as_millis(), processed_count, path);
        } else {
            log::debug!("Directory scan completed in {}ms for {} items in {:?}",
                       elapsed.as_millis(), processed_count, path);
        }

        Ok(())
    }

    /// 按需加载目录的子项（懒加载）
    pub fn load_directory_children(&mut self, path: &PathBuf, settings: &EditorSettings) -> FileEditorResult<()> {
        // 如果已经加载过，直接返回
        if self.directory_children.contains_key(path) {
            return Ok(());
        }

        self.scan_single_directory(path, settings)?;
        Ok(())
    }

    /// 获取目录的子项
    pub fn get_children(&self, path: &PathBuf) -> Vec<PathBuf> {
        self.directory_children.get(path).cloned().unwrap_or_default()
    }

    /// 判断是否应该跳过特定目录（即使显示隐藏文件也跳过）
    fn should_skip_directory(&self, name: &str) -> bool {
        // 这些目录通常很大且不需要编辑，即使用户选择显示隐藏文件也跳过
        matches!(name,
            ".git" |
            "node_modules" |
            ".vscode" |
            ".idea" |
            "__pycache__" |
            ".pytest_cache" |
            "target" |      // Rust build directory
            "build" |       // Common build directory
            "dist" |        // Distribution directory
            ".next" |       // Next.js build directory
            ".nuxt" |       // Nuxt.js build directory
            "coverage" |    // Test coverage directory
            ".nyc_output"   // NYC coverage directory
        )
    }

    /// 获取文件条目
    pub fn get_file_entry(&self, path: &PathBuf) -> Option<&FileEntry> {
        self.file_entries.get(path)
    }

    /// 获取当前展开的目录列表
    pub fn get_expanded_directories(&self) -> std::collections::HashSet<PathBuf> {
        let mut expanded_dirs = std::collections::HashSet::new();

        // 遍历所有目录条目，检查哪些是展开状态的
        for (path, entry) in self.file_entries.iter() {
            if entry.is_dir {
                let _node_id = FileNodeId(path.clone());
                // 使用一个间接方法来检测展开状态：
                // 如果目录有已加载的子项，并且在TreeViewState中有状态记录，则认为它可能是展开的
                if self.directory_children.contains_key(path) {
                    expanded_dirs.insert(path.clone());
                }
            }
        }

        expanded_dirs
    }

    /// 获取根目录的子项
    pub fn get_root_children(&self) -> Vec<PathBuf> {
        if let Some(root) = &self.root_path {
            self.get_children(root)
        } else {
            Vec::new()
        }
    }

    /// 获取文件图标
    pub fn get_file_icon(&self, path: &PathBuf) -> &str {
        if path.is_dir() {
            "📁"
        } else {
            match path.extension().and_then(|s| s.to_str()) {
                Some("rs") => "🦀",
                Some("py") => "🐍",
                Some("js") | Some("ts") => "📜",
                Some("md") => "📝",
                Some("txt") => "📄",
                Some("json") => "📋",
                Some("toml") | Some("yaml") | Some("yml") => "⚙️",
                Some("html") => "🌐",
                Some("css") => "🎨",
                Some("png") | Some("jpg") | Some("jpeg") | Some("gif") => "🖼️",
                Some("pdf") => "📕",
                Some("zip") | Some("tar") | Some("gz") => "📦",
                _ => "📄",
            }
        }
    }

    /// 从搜索结果打开文件或目录
    pub fn open_from_search(&mut self, path: PathBuf, is_directory: bool, settings: &EditorSettings) -> FileEditorResult<()> {
        if is_directory {
            // 打开目录：设置为根目录
            self.set_root(path, settings)?;
        } else {
            // 打开文件：导航到文件所在目录并选中文件
            if let Some(parent) = path.parent() {
                self.set_root(parent.to_path_buf(), settings)?;

                // 选中文件
                self.tree_view_state.set_one_selected(FileNodeId(path.clone()));

                // 展开到文件所在的路径
                let mut current = parent;
                while let Some(p) = current.parent() {
                    self.tree_view_state.expand_node(&FileNodeId(p.to_path_buf()));
                    current = p;
                }
            }
        }
        Ok(())
    }
}

/// 文件条目
#[derive(Debug, Clone)]
pub struct FileEntry {
    pub path: PathBuf,
    pub name: String,
    pub is_dir: bool,
    pub icon: String,
    pub size: Option<u64>,
    pub modified: Option<SystemTime>,
}

/// 搜索集成的打开请求
#[derive(Debug, Clone)]
pub struct OpenRequest {
    pub path: PathBuf,
    pub request_type: OpenRequestType,
    pub source_module: String,
}

/// 打开请求类型
#[derive(Debug, Clone)]
pub enum OpenRequestType {
    File,
    Directory,
}

/// 文件状态信息（用于状态栏显示）
#[derive(Debug, Clone)]
pub struct FileStatusInfo {
    pub file_path: PathBuf,
    pub encoding: String,
    pub language: Option<String>,
    pub line_count: usize,
    pub char_count: usize,
    pub byte_count: usize,
    pub modified: bool,
    pub read_only: bool,
    pub cursor_line: usize,
    pub cursor_column: usize,
}

/// 文件上下文信息（用于AI助手）
#[derive(Debug, Clone)]
pub struct FileContext {
    pub file_path: PathBuf,
    pub file_name: String,
    pub language: Option<String>,
    pub content: String,
    pub selected_text: Option<String>,
    pub cursor_line: usize,
    pub cursor_column: usize,
    pub total_lines: usize,
    pub is_modified: bool,
    pub is_read_only: bool,
}

/// 撤销操作类型
#[derive(Debug, Clone)]
pub enum UndoOperation {
    Insert {
        position: usize,
        text: String,
    },
    Delete {
        position: usize,
        text: String,
    },
    Replace {
        position: usize,
        old_text: String,
        new_text: String,
    },
}

/// UI 状态
#[derive(Debug)]
pub struct EditorUIState {
    /// 布局状态
    pub show_file_tree: bool,
    pub file_tree_width: f32,
    pub editor_font_size: f32,

    /// 交互状态
    pub file_tree_selected: Option<PathBuf>,
    pub search_query: String,
    pub show_search: bool,

    /// 查找替换状态
    pub show_find_replace: bool,
    pub find_query: String,
    pub replace_query: String,

    /// 编辑器选项状态
    pub word_wrap: bool,
    pub show_line_numbers: bool,
    pub auto_save: bool,

    /// 保存确认对话框状态
    pub show_save_confirmation: bool,
    pub save_confirmation_files: Vec<std::path::PathBuf>,

    /// 目录选择状态
    pub show_directory_picker: bool,
    pub directory_input: String,

    /// 文件操作对话框状态
    pub show_new_file_dialog: bool,
    pub show_new_folder_dialog: bool,
    pub show_rename_dialog: bool,
    pub show_delete_confirmation: bool,
    pub new_file_name: String,
    pub new_folder_name: String,
    pub rename_new_name: String,
    pub operation_target_path: Option<std::path::PathBuf>,

    /// 上下文菜单状态
    pub show_context_menu: bool,
    pub context_menu_path: Option<std::path::PathBuf>,
    pub context_menu_is_dir: bool,
}

impl EditorUIState {
    pub fn new() -> Self {
        let state = Self {
            show_file_tree: true,
            file_tree_width: 250.0,
            editor_font_size: 14.0,
            file_tree_selected: None,
            search_query: String::new(),
            show_search: false,
            show_find_replace: false,
            find_query: String::new(),
            replace_query: String::new(),
            word_wrap: true,
            show_line_numbers: true,
            auto_save: false,
            show_save_confirmation: false,
            save_confirmation_files: Vec::new(),
            show_directory_picker: false,
            directory_input: String::new(),
            show_new_file_dialog: false,
            show_new_folder_dialog: false,
            show_rename_dialog: false,
            show_delete_confirmation: false,
            new_file_name: String::new(),
            new_folder_name: String::new(),
            rename_new_name: String::new(),
            operation_target_path: None,
            show_context_menu: false,
            context_menu_path: None,
            context_menu_is_dir: false,
        };

        log::debug!("WORD_WRAP_DIAGNOSIS: EditorUIState initialized with word_wrap={}", state.word_wrap);
        state
    }
}

// NodeId implementation removed due to egui version compatibility

/// 检测文件语言
fn detect_language(path: &PathBuf) -> Option<String> {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| match ext.to_lowercase().as_str() {
            "rs" => "rust",
            "py" => "python",
            "js" | "ts" => "javascript",
            "md" => "markdown",
            "json" => "json",
            "toml" => "toml",
            "yaml" | "yml" => "yaml",
            "html" => "html",
            "css" => "css",
            "xml" => "xml",
            "c" => "c",
            "cpp" | "cc" | "cxx" => "cpp",
            "java" => "java",
            "go" => "go",
            "php" => "php",
            "rb" => "ruby",
            "sh" | "bash" => "bash",
            _ => "text",
        })
        .map(|s| s.to_string())
}

/// 判断文件是否可编辑
pub fn is_editable_file(path: &PathBuf) -> bool {
    if let Some(extension) = path.extension().and_then(|ext| ext.to_str()) {
        match extension.to_lowercase().as_str() {
            // 文本文件
            "txt" | "md" | "markdown" | "rst" | "log" => true,

            // 程序源码
            "rs" | "py" | "js" | "ts" | "jsx" | "tsx" | "vue" | "svelte" => true,
            "c" | "cpp" | "cc" | "cxx" | "h" | "hpp" | "hxx" => true,
            "java" | "kt" | "scala" | "go" | "php" | "rb" | "swift" => true,
            "cs" | "fs" | "vb" | "dart" | "lua" | "perl" | "r" => true,

            // 配置文件
            "json" | "toml" | "yaml" | "yml" | "ini" | "cfg" | "conf" => true,
            "xml" | "html" | "htm" | "css" | "scss" | "sass" | "less" => true,

            // 脚本文件
            "sh" | "bash" | "zsh" | "fish" | "ps1" | "bat" | "cmd" => true,
            "sql" | "dockerfile" | "makefile" => true,

            // 数据文件
            "csv" | "tsv" | "jsonl" | "ndjson" => true,

            // 二进制文件（不可编辑）
            "exe" | "dll" | "so" | "dylib" | "bin" | "obj" | "o" => false,
            "zip" | "tar" | "gz" | "bz2" | "xz" | "7z" | "rar" => false,
            "jpg" | "jpeg" | "png" | "gif" | "bmp" | "svg" | "ico" => false,
            "mp3" | "mp4" | "avi" | "mov" | "wav" | "flac" => false,
            "pdf" | "doc" | "docx" | "xls" | "xlsx" | "ppt" | "pptx" => false,

            // 默认：小文件可编辑，大文件不可编辑
            _ => {
                if let Ok(metadata) = std::fs::metadata(path) {
                    metadata.len() < 10 * 1024 * 1024 // 10MB以下的文件默认可编辑
                } else {
                    false
                }
            }
        }
    } else {
        // 没有扩展名的文件，检查是否是常见的可编辑文件
        if let Some(filename) = path.file_name().and_then(|name| name.to_str()) {
            match filename.to_lowercase().as_str() {
                "readme" | "license" | "changelog" | "makefile" | "dockerfile" => true,
                "gitignore" | "gitattributes" | "editorconfig" => true,
                _ => {
                    // 检查文件大小，小文件默认可编辑
                    if let Ok(metadata) = std::fs::metadata(path) {
                        metadata.len() < 1024 * 1024 // 1MB以下的无扩展名文件默认可编辑
                    } else {
                        false
                    }
                }
            }
        } else {
            false
        }
    }
}
