# 文件编辑器功能设计文档

## 概述

参考 Lapce 编辑器，为 SeeU Desktop 添加一个高性能的文件编辑器功能。该功能将作为独立的 crate 实现，支持文本文件编辑、语法高亮、目录树浏览等功能。

## 功能需求

### 核心功能
1. **文件浏览**: 左侧目录树，支持文件夹展开/折叠
2. **文本编辑**: 基于 ROPE 数据结构的高效文本编辑器，支持大文件编辑
3. **语法高亮**: 支持常见编程语言的语法高亮
4. **多文件编辑**: 支持同时打开多个文件（标签页）
5. **文件操作**: 新建、打开、保存、另存为等基础操作
6. **搜索集成**: 从搜索结果界面直接打开文件或目录到编辑器

### 扩展功能（后续实现）
1. **LSP 支持**: 代码补全、错误检查、跳转定义等
2. **搜索替换**: 文件内搜索、跨文件搜索
3. **主题支持**: 多种编辑器主题
4. **快捷键**: 常用编辑快捷键支持

## 技术架构

### Crate 结构
```
crates/ifile_editor/
├── Cargo.toml
├── src/
│   ├── lib.rs              # 主模块入口
│   ├── state.rs            # 编辑器状态管理
│   ├── ui/
│   │   ├── mod.rs          # UI 模块
│   │   ├── main_ui.rs      # 主界面渲染
│   │   ├── file_tree.rs    # 文件树组件
│   │   ├── editor.rs       # 文本编辑器组件
│   │   └── tabs.rs         # 标签页组件
│   ├── core/
│   │   ├── mod.rs          # 核心模块
│   │   ├── file_manager.rs # 文件管理
│   │   ├── text_buffer.rs  # 文本缓冲区（基于 ROPE）
│   │   └── syntax.rs       # 语法高亮
│   ├── settings/
│   │   ├── mod.rs          # 设置模块
│   │   └── editor_settings.rs # 编辑器设置
│   └── tests/              # 单元测试
│       ├── mod.rs
│       ├── file_tree_tests.rs
│       ├── editor_tests.rs
│       └── text_buffer_tests.rs
```

### 核心依赖

#### 文本编辑核心
- **crop**: ROPE 算法实现，高效文本编辑（B-tree 基础的 UTF-8 文本 rope）
- **syntect**: 语法高亮支持
- **egui**: UI 框架

#### 文件树组件
- **egui_ltreeview**: 专业树形视图组件，支持多选、键盘导航、拖拽等功能
- **walkdir**: 目录遍历

#### 其他依赖
- **serde**: 设置序列化
- **tokio**: 异步文件操作
- **notify**: 文件变化监控

## UI 设计

### 整体布局
```
+------------------------------------------+
| 文件编辑器                                |
+----------+-------------------------------+
| 文件树   | 编辑区域                      |
| 📁 src   | ┌─ file1.rs ─┐ ┌─ file2.py ─┐|
| 📄 main  | │             │ │             │|
| 📄 lib   | │   文本编辑  │ │   文本编辑  │|
| 📁 docs  | │   区域      │ │   区域      │|
| 📄 README| │             │ │             │|
|          | └─────────────┘ └─────────────┘|
+----------+-------------------------------+
```

### 导航栏集成
在现有导航栏的 Home 图标下方添加文件编辑器图标：
- 图标: 📄📄 (双文件图标)
- 提示文本: "文件编辑"
- 对应模块: `Module::FileEditor`

## 状态管理

### 主状态结构
```rust
pub struct IFileEditorState {
    // 文件树状态
    pub file_tree: FileTreeState,

    // 打开的文件（基于 ROPE 数据结构）
    pub open_files: HashMap<PathBuf, OpenFile>,

    // 当前活动文件
    pub active_file: Option<PathBuf>,

    // 编辑器设置
    pub settings: EditorSettings,

    // UI 状态
    pub show_file_tree: bool,
    pub tree_width: f32,

    // 搜索集成状态
    pub pending_open_request: Option<OpenRequest>,
}

pub struct OpenFile {
    pub path: PathBuf,
    pub content: TextBuffer,  // 基于 crop::Rope 的高效文本缓冲区
    pub modified: bool,
    pub cursor_position: usize,
    pub scroll_position: f32,
    pub language: Option<String>,
    pub rope: crop::Rope,     // ROPE 数据结构实例
}

pub struct OpenRequest {
    pub path: PathBuf,
    pub request_type: OpenRequestType,
    pub source_module: String,
}

pub enum OpenRequestType {
    File,
    Directory,
}
```

### 文件树状态
```rust
pub struct FileTreeState {
    pub root_path: Option<PathBuf>,
    pub expanded_dirs: HashSet<PathBuf>,
    pub selected_file: Option<PathBuf>,
    pub tree_data: Vec<FileTreeNode>,
    pub tree_view_state: egui_ltreeview::TreeViewState<PathBuf>,  // egui_ltreeview 状态
}

pub struct FileTreeNode {
    pub path: PathBuf,
    pub name: String,
    pub is_dir: bool,
    pub children: Vec<FileTreeNode>,
    pub icon: String,
}

impl egui_ltreeview::NodeId for PathBuf {
    type Id = PathBuf;
    fn id(&self) -> Self::Id {
        self.clone()
    }
}
```

## 核心组件设计

### 1. 文件树组件 (file_tree.rs)
- 使用 `egui_ltreeview::TreeView` 实现专业文件树
- 支持多选、键盘导航（箭头键、Shift/Ctrl 组合）
- 文件夹展开/折叠，支持默认展开状态
- 文件类型图标显示和自定义图标
- 右键菜单（新建、删除、重命名等）
- 拖拽支持（文件移动、复制）
- 从搜索结果打开文件/目录的集成接口

### 2. 文本编辑器组件 (editor.rs)
- 基于 `crop::Rope` 的高效文本缓冲区（B-tree 结构）
- ROPE 数据结构优势：
  - 对数时间复杂度的插入、删除、替换操作
  - 高效的大文件处理能力
  - 内存友好的文本分块存储
  - 支持增量编辑和撤销/重做
- 语法高亮（使用 `syntect`）
- 行号显示
- 光标和选择区域管理
- 滚动条支持和虚拟滚动

### 3. 标签页组件 (tabs.rs)
- 多文件标签页管理
- 文件修改状态指示
- 标签页关闭功能
- 拖拽排序（后续实现）

### 4. 搜索集成组件 (search_integration.rs)
- 处理来自搜索模块的文件打开请求
- 支持打开单个文件到编辑器
- 支持打开目录到文件树
- 与主应用的模块切换集成

## 设置系统

### 编辑器设置
```rust
pub struct EditorSettings {
    // 外观设置
    pub font_family: String,
    pub font_size: f32,
    pub line_height: f32,
    pub theme: String,
    
    // 编辑器行为
    pub tab_size: usize,
    pub insert_spaces: bool,
    pub word_wrap: bool,
    pub show_line_numbers: bool,
    pub show_whitespace: bool,
    
    // 文件设置
    pub auto_save: bool,
    pub auto_save_delay: u64, // 秒
    pub default_encoding: String,
}
```

### 设置页面集成
- 在主设置页面添加"文件编辑器"分类
- 图标: 📝
- 包含编辑器外观、行为、文件处理等设置

## 性能优化

### 文本编辑性能
1. **ROPE 数据结构优势**:
   - 使用 `crop` crate 的 B-tree 基础 ROPE 实现
   - 对数时间复杂度的文本操作（O(log n)）
   - 高效的大文件处理（支持 GB 级文件）
   - 内存友好的分块存储（每块最大 1KB）
   - 零拷贝的文本切片操作
2. **增量渲染**: 只渲染可见区域的文本
3. **语法高亮缓存**: 缓存语法高亮结果
4. **虚拟滚动**: 大文件的虚拟滚动支持

### 文件树性能
1. **懒加载**: 按需加载目录内容
2. **文件监控**: 使用 `notify` 监控文件变化
3. **缓存机制**: 缓存目录结构
4. **egui_ltreeview 优化**: 虚拟化支持，适合大型目录结构

## 单元测试计划

### 测试覆盖范围
1. **文本缓冲区测试**: ROPE 操作、插入、删除、查找
2. **文件管理测试**: 文件读写、编码处理
3. **文件树测试**: 目录遍历、状态管理
4. **设置测试**: 配置加载、保存、验证
5. **UI 组件测试**: 基础渲染和交互测试

### 测试工具
- 标准 Rust 测试框架
- `tempfile` 用于临时文件测试
- `mockall` 用于模拟依赖

## 实现计划

### 阶段 1: 基础架构
1. 创建 crate 结构
2. 实现基础状态管理
3. 集成到导航栏

### 阶段 2: 核心功能
1. 实现文件树组件
2. 实现基础文本编辑器
3. 文件打开/保存功能

### 阶段 3: 增强功能
1. 语法高亮支持
2. 多文件标签页
3. 设置页面

### 阶段 4: 优化和测试
1. 性能优化
2. 完善单元测试
3. 用户体验优化

## 扩展性考虑

### LSP 支持准备
- 预留 LSP 客户端接口
- 设计插件化架构
- 支持语言服务器配置

### 主题系统
- 可扩展的主题框架
- 支持自定义颜色方案
- 与应用主题系统集成

这个设计为后续的 LSP 支持、高级编辑功能等扩展奠定了良好的基础。
