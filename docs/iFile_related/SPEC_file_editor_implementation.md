# 文件编辑器实现规范

## 技术选型详细说明

### 1. ROPE 算法实现 - crop crate

#### 选择理由
- **性能优势**: crop 是目前 Rust 生态中最快的 UTF-8 文本 ROPE 实现
- **内存效率**: B-tree 基础结构，优化的内存使用
- **Lapce 同源**: Lapce 编辑器也使用类似的 ROPE 实现

#### 核心 API 使用
```rust
use crop::Rope;

// 创建文本缓冲区
let mut rope = Rope::from("Hello, World!");

// 插入文本
rope.insert(7, "Rust ");

// 删除文本
rope.delete(0..5);

// 获取文本片段
let slice = rope.slice(0..4);
```

### 2. 文件树组件 - egui_ltreeview

#### 选择理由
- **功能完整**: 支持多选、键盘导航、Windows 文件管理器风格
- **性能优化**: 虚拟化支持，适合大型目录结构
- **egui 原生**: 与项目 UI 框架完美集成
- **专业特性**: 拖拽支持、上下文菜单、自定义图标

#### 详细使用模式
```rust
use egui_ltreeview::{TreeView, TreeViewBuilder, NodeBuilder, TreeViewSettings};

// 创建树视图设置
let settings = TreeViewSettings {
    fill_space_horizontal: true,
    fill_space_vertical: true,
    override_indent: Some(20),
    ..Default::default()
};

// 在 UI 渲染中
let response = TreeView::new(ui.make_persistent_id("file_tree"))
    .with_settings(settings)
    .desired_width(250.0)
    .allow_multi_selection(true)
    .fallback_context_menu(|ui, nodes| {
        // 右键菜单处理
        for node in nodes {
            ui.label(format!("操作: {:?}", node));
        }
    })
    .show(ui, |builder| {
        // 构建文件树结构
        build_file_tree(builder, &file_tree_state);
    });

// 处理树视图响应
handle_tree_actions(response.actions);
```

### 3. 语法高亮 - syntect

#### 选择理由
- **语言支持广泛**: 支持 100+ 编程语言
- **主题丰富**: 内置多种高亮主题
- **性能良好**: 增量高亮支持

## 详细实现规范

### 1. Cargo.toml 配置

```toml
[package]
name = "ifile_editor"
version = "0.1.0"
edition = "2021"

[dependencies]
# UI 框架
egui = { version = "0.28.1", default-features = false, features = ["default_fonts"] }
egui_ltreeview = "0.4.1"

# 文本编辑核心
crop = "0.4.2"
syntect = "5.1"

# 文件操作
walkdir = "2.4"
notify = "6.1"
encoding_rs = "0.8"

# 异步和序列化
tokio = { version = "1.32", features = ["fs", "io-util"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# 工具库
anyhow = "1.0"
log = "0.4"
uuid = { version = "1.4", features = ["v4"] }
```

### 2. 状态管理详细设计

#### 主状态结构
```rust
#[derive(Debug)]
pub struct IFileEditorState {
    // 工作区设置
    pub workspace_root: Option<PathBuf>,
    
    // 文件树状态
    pub file_tree: FileTreeState,
    
    // 编辑器状态
    pub editor: EditorState,
    
    // UI 状态
    pub ui_state: EditorUIState,
    
    // 设置
    pub settings: EditorSettings,
    
    // 文件监控
    pub file_watcher: Option<FileWatcher>,
}

#[derive(Debug)]
pub struct EditorState {
    // 打开的文件缓冲区
    pub buffers: HashMap<PathBuf, TextBuffer>,
    
    // 标签页管理
    pub tabs: Vec<PathBuf>,
    pub active_tab: Option<usize>,
    
    // 编辑历史
    pub undo_stack: HashMap<PathBuf, UndoStack>,
}

#[derive(Debug)]
pub struct EditorUIState {
    // 布局状态
    pub show_file_tree: bool,
    pub file_tree_width: f32,
    pub editor_font_size: f32,
    
    // 交互状态
    pub file_tree_selected: Option<PathBuf>,
    pub search_query: String,
    pub show_search: bool,
}
```

#### 文本缓冲区设计
```rust
use crop::Rope;

#[derive(Debug)]
pub struct TextBuffer {
    // 文本内容
    pub rope: Rope,
    
    // 文件信息
    pub file_path: PathBuf,
    pub encoding: String,
    pub line_ending: LineEnding,
    
    // 编辑状态
    pub modified: bool,
    pub last_saved: SystemTime,
    
    // 光标和选择
    pub cursor: Cursor,
    pub selection: Option<Selection>,
    
    // 语法高亮
    pub syntax_set: Option<SyntaxReference>,
    pub highlight_cache: HighlightCache,
    
    // 视图状态
    pub scroll_offset: f32,
    pub visible_lines: Range<usize>,
}

#[derive(Debug, Clone)]
pub struct Cursor {
    pub line: usize,
    pub column: usize,
    pub byte_offset: usize,
}

#[derive(Debug, Clone)]
pub struct Selection {
    pub start: Cursor,
    pub end: Cursor,
}
```

### 3. 文件树实现规范

#### 文件树节点结构和 egui_ltreeview 集成
```rust
use egui_ltreeview::{NodeId, TreeViewBuilder, NodeBuilder};
use std::path::PathBuf;

// PathBuf 实现 NodeId trait
impl NodeId for PathBuf {
    type Id = PathBuf;

    fn id(&self) -> Self::Id {
        self.clone()
    }
}

#[derive(Debug, Clone)]
pub struct FileTreeNode {
    pub path: PathBuf,
    pub name: String,
    pub is_dir: bool,
    pub children: Vec<FileTreeNode>,
    pub icon: String,
    pub size: Option<u64>,
    pub modified: Option<SystemTime>,
}

// 构建文件树的函数
fn build_file_tree(builder: &mut TreeViewBuilder<PathBuf>, state: &FileTreeState) {
    if let Some(root) = &state.root_path {
        build_tree_recursive(builder, root, &state.tree_data, 0);
    }
}

fn build_tree_recursive(
    builder: &mut TreeViewBuilder<PathBuf>,
    current_path: &Path,
    nodes: &[FileTreeNode],
    depth: usize,
) {
    for node in nodes {
        if node.is_dir {
            // 目录节点
            builder.node(NodeBuilder::dir(node.path.clone())
                .label(&node.name)
                .icon(|ui| {
                    ui.label(&node.icon);
                })
                .default_open(depth < 2) // 默认展开前两层
                .context_menu(|ui| {
                    if ui.button("📁 新建文件夹").clicked() {
                        // 处理新建文件夹
                    }
                    if ui.button("📄 新建文件").clicked() {
                        // 处理新建文件
                    }
                }));

            // 递归构建子目录
            if !node.children.is_empty() {
                build_tree_recursive(builder, &node.path, &node.children, depth + 1);
            }

            builder.close_dir();
        } else {
            // 文件节点
            builder.node(NodeBuilder::leaf(node.path.clone())
                .label(&node.name)
                .icon(|ui| {
                    ui.label(&node.icon);
                })
                .context_menu(|ui| {
                    if ui.button("📝 编辑").clicked() {
                        // 处理文件编辑
                    }
                    if ui.button("🗑 删除").clicked() {
                        // 处理文件删除
                    }
                }));
        }
    }
}
```

#### 文件树操作接口和搜索集成
```rust
impl FileTreeState {
    pub fn new() -> Self {
        Self {
            root_path: None,
            expanded_dirs: HashSet::new(),
            selected_file: None,
            tree_data: Vec::new(),
            tree_view_state: egui_ltreeview::TreeViewState::default(),
        }
    }

    pub fn set_root(&mut self, path: PathBuf) -> Result<()> {
        self.root_path = Some(path.clone());
        self.refresh()?;
        Ok(())
    }

    pub fn refresh(&mut self) -> Result<()> {
        if let Some(root) = &self.root_path {
            self.tree_data = self.scan_directory(root)?;
        }
        Ok(())
    }

    pub fn expand_dir(&mut self, path: &Path) -> Result<()> {
        self.expanded_dirs.insert(path.to_path_buf());
        Ok(())
    }

    pub fn collapse_dir(&mut self, path: &Path) {
        self.expanded_dirs.remove(path);
    }

    pub fn select_file(&mut self, path: &Path) {
        self.selected_file = Some(path.to_path_buf());
    }

    pub fn get_file_icon(&self, path: &Path) -> &str {
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
                _ => "📄",
            }
        }
    }

    // 搜索集成：从搜索结果打开文件或目录
    pub fn open_from_search(&mut self, path: PathBuf, is_directory: bool) -> Result<()> {
        if is_directory {
            // 打开目录：设置为根目录并展开
            self.set_root(path)?;
        } else {
            // 打开文件：导航到文件所在目录并选中文件
            if let Some(parent) = path.parent() {
                self.set_root(parent.to_path_buf())?;
                self.select_file(&path);

                // 展开到文件所在的路径
                let mut current = parent;
                while let Some(p) = current.parent() {
                    self.expand_dir(p);
                    current = p;
                }
            }
        }
        Ok(())
    }

    // 处理 egui_ltreeview 的 Action
    pub fn handle_tree_actions(&mut self, actions: Vec<egui_ltreeview::Action<PathBuf>>) {
        for action in actions {
            match action {
                egui_ltreeview::Action::Activate(activate) => {
                    // 双击或回车激活文件
                    for node_id in activate.nodes {
                        if node_id.is_file() {
                            // 发送文件打开请求
                            self.request_file_open(node_id);
                        }
                    }
                }
                egui_ltreeview::Action::SetSelected(selection) => {
                    // 更新选中状态
                    if let Some(first) = selection.nodes.first() {
                        self.select_file(first);
                    }
                }
                _ => {} // 其他 action 暂时忽略
            }
        }
    }

    fn request_file_open(&self, path: PathBuf) {
        // 这里会通过事件系统通知编辑器打开文件
        log::info!("Request to open file: {:?}", path);
    }
}
```

### 4. 文本编辑器实现规范

#### 编辑器渲染接口
```rust
pub fn render_text_editor(
    ui: &mut egui::Ui,
    buffer: &mut TextBuffer,
    settings: &EditorSettings,
) -> EditorResponse {
    // 计算可见区域
    let visible_rect = ui.available_rect_before_wrap();
    let line_height = settings.line_height();
    let visible_lines = calculate_visible_lines(&visible_rect, line_height, buffer.scroll_offset);
    
    // 渲染行号
    if settings.show_line_numbers {
        render_line_numbers(ui, &visible_lines, line_height);
    }
    
    // 渲染文本内容
    render_text_content(ui, buffer, &visible_lines, settings);
    
    // 处理输入事件
    handle_input_events(ui, buffer, settings)
}

#[derive(Debug)]
pub struct EditorResponse {
    pub cursor_moved: bool,
    pub text_changed: bool,
    pub scroll_changed: bool,
    pub file_action: Option<FileAction>,
}

#[derive(Debug)]
pub enum FileAction {
    Save,
    SaveAs,
    Close,
    Reload,
}
```

#### 语法高亮实现
```rust
use syntect::{parsing::SyntaxSet, highlighting::{ThemeSet, Highlighter}};

pub struct SyntaxHighlighter {
    syntax_set: SyntaxSet,
    theme_set: ThemeSet,
    current_theme: String,
}

impl SyntaxHighlighter {
    pub fn new() -> Self {
        Self {
            syntax_set: SyntaxSet::load_defaults_newlines(),
            theme_set: ThemeSet::load_defaults(),
            current_theme: "base16-ocean.dark".to_string(),
        }
    }
    
    pub fn highlight_line(
        &self,
        line: &str,
        syntax: &SyntaxReference,
    ) -> Vec<(egui::Color32, String)> {
        // 实现行级语法高亮
        // 返回颜色和文本片段的组合
    }
}
```

### 5. 设置系统实现

#### 设置结构定义
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditorSettings {
    // 外观设置
    pub font_family: String,
    pub font_size: f32,
    pub line_height_factor: f32,
    pub theme: String,
    
    // 编辑器行为
    pub tab_size: usize,
    pub insert_spaces: bool,
    pub word_wrap: bool,
    pub show_line_numbers: bool,
    pub show_whitespace: bool,
    pub auto_indent: bool,
    
    // 文件设置
    pub auto_save: bool,
    pub auto_save_delay_ms: u64,
    pub default_encoding: String,
    pub detect_encoding: bool,
    
    // 性能设置
    pub syntax_highlighting: bool,
    pub max_file_size_mb: usize,
    pub virtual_scrolling: bool,
}

impl Default for EditorSettings {
    fn default() -> Self {
        Self {
            font_family: "Consolas".to_string(),
            font_size: 14.0,
            line_height_factor: 1.2,
            theme: "dark".to_string(),
            tab_size: 4,
            insert_spaces: true,
            word_wrap: false,
            show_line_numbers: true,
            show_whitespace: false,
            auto_indent: true,
            auto_save: true,
            auto_save_delay_ms: 2000,
            default_encoding: "UTF-8".to_string(),
            detect_encoding: true,
            syntax_highlighting: true,
            max_file_size_mb: 10,
            virtual_scrolling: true,
        }
    }
}
```

### 6. 错误处理规范

#### 错误类型定义
```rust
use anyhow::{Result, Context};

#[derive(Debug, thiserror::Error)]
pub enum FileEditorError {
    #[error("文件不存在: {path}")]
    FileNotFound { path: String },
    
    #[error("文件过大: {size_mb}MB (最大: {max_mb}MB)")]
    FileTooLarge { size_mb: usize, max_mb: usize },
    
    #[error("编码错误: {encoding}")]
    EncodingError { encoding: String },
    
    #[error("权限错误: {path}")]
    PermissionDenied { path: String },
    
    #[error("IO 错误: {0}")]
    IoError(#[from] std::io::Error),
}
```

### 7. 搜索集成规范

#### 搜索模块接口
```rust
// 在搜索结果中添加"在编辑器中打开"按钮
pub trait SearchIntegration {
    fn open_in_editor(&self, path: PathBuf, is_directory: bool);
}

// 搜索结果操作扩展
impl ISearchState {
    pub fn open_in_file_editor(&mut self, path: &str, is_directory: bool) {
        // 发送打开请求到文件编辑器
        let open_request = FileEditorOpenRequest {
            path: PathBuf::from(path),
            is_directory,
            source: "search".to_string(),
        };

        // 通过应用状态传递请求
        self.pending_file_editor_request = Some(open_request);
    }
}

// 在搜索结果 UI 中添加编辑器按钮
fn render_search_result_actions(ui: &mut egui::Ui, result: &SearchResult, state: &mut ISearchState) {
    ui.horizontal(|ui| {
        // 现有按钮
        if ui.button("📂 打开文件夹").clicked() {
            state.open_folder(&result.path);
        }

        if ui.button("📄 打开文件").clicked() {
            state.open_file(&result.path);
        }

        // 新增：在编辑器中打开
        if ui.button("📝 编辑器中打开").clicked() {
            state.open_in_file_editor(&result.path, false);
        }

        // 对于目录，添加在编辑器中浏览
        if result.path.ends_with('/') || std::path::Path::new(&result.path).is_dir() {
            if ui.button("📁 编辑器中浏览").clicked() {
                state.open_in_file_editor(&result.path, true);
            }
        }
    });
}
```

#### 应用级集成处理
```rust
// 在主应用中处理文件编辑器打开请求
impl SeeUApp {
    pub fn handle_file_editor_requests(&mut self) {
        // 检查搜索模块的请求
        if let Some(request) = self.isearch_state.pending_file_editor_request.take() {
            self.process_file_editor_request(request);
        }

        // 检查其他模块的请求...
    }

    fn process_file_editor_request(&mut self, request: FileEditorOpenRequest) {
        // 切换到文件编辑器模块
        self.active_module = Module::FileEditor;

        // 处理打开请求
        if request.is_directory {
            self.ifile_editor_state.file_tree.open_from_search(request.path, true)
                .unwrap_or_else(|e| log::error!("Failed to open directory: {}", e));
        } else {
            // 打开文件
            self.ifile_editor_state.open_file_from_search(request.path)
                .unwrap_or_else(|e| log::error!("Failed to open file: {}", e));
        }
    }
}

#[derive(Debug, Clone)]
pub struct FileEditorOpenRequest {
    pub path: PathBuf,
    pub is_directory: bool,
    pub source: String,
}
```

### 8. 单元测试规范

#### 测试模块结构
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_text_buffer_creation() {
        // 测试文本缓冲区创建
    }

    #[test]
    fn test_rope_operations() {
        let mut rope = crop::Rope::from("Hello, World!");
        rope.insert(7, "Rust ");
        assert_eq!(rope.to_string(), "Hello, Rust World!");

        rope.delete(0..6);
        assert_eq!(rope.to_string(), "Rust World!");
    }

    #[test]
    fn test_file_tree_navigation() {
        // 测试文件树导航
    }

    #[test]
    fn test_syntax_highlighting() {
        // 测试语法高亮
    }

    #[test]
    fn test_settings_serialization() {
        // 测试设置序列化
    }

    #[test]
    fn test_search_integration() {
        // 测试搜索集成功能
        let mut state = IFileEditorState::new();
        let test_path = PathBuf::from("/test/file.rs");

        state.file_tree.open_from_search(test_path.clone(), false).unwrap();
        assert_eq!(state.file_tree.selected_file, Some(test_path));
    }

    #[test]
    fn test_egui_ltreeview_integration() {
        // 测试 egui_ltreeview 集成
    }
}
```

这个实现规范为开发团队提供了详细的技术指导，确保代码质量和一致性。
