# iFile Editor - 高性能文件编辑器模块

基于 ROPE 数据结构和 egui_ltreeview 的现代文件编辑器，支持大文件编辑、语法高亮、专业文件树浏览等功能。

## 🌟 主要特性

### 文件树功能
- ✅ **基于 egui_ltreeview 的专业树形视图**
- ✅ **多选支持**（Ctrl/Cmd + 点击）
- ✅ **键盘导航**（方向键、Enter、Shift/Ctrl 组合）
- ✅ **右键上下文菜单**
- ✅ **文件类型图标显示**
- ✅ **目录展开/折叠**
- ✅ **高性能虚拟化支持**（适合大型目录结构）
- ✅ **拖拽支持**（文件移动、复制）

### 文本编辑功能
- ✅ **ROPE 数据结构**（基于 crop crate 的高效文本编辑）
- ✅ **语法高亮**（支持多种编程语言）
- ✅ **大文件支持**（GB 级文件处理）
- ✅ **多文件编辑**（标签页管理）
- ✅ **撤销/重做**
- ✅ **搜索集成**（从搜索结果直接打开文件）

## 🚀 快速开始

### 运行示例

```bash
cd crates/ifile_editor
cargo run --example file_tree_demo
```

### 基本使用

```rust
use ifile_editor::{IFileEditorState, ui::render_file_tree};
use std::path::PathBuf;

// 创建文件编辑器状态
let mut state = IFileEditorState::new();

// 设置根目录
state.file_tree.set_root(PathBuf::from("/path/to/directory")).unwrap();

// 在 egui UI 中渲染文件树
render_file_tree(ui, &mut state);
```

## 🏗️ 架构设计

### 核心组件

```
crates/ifile_editor/
├── src/
│   ├── lib.rs              # 主模块入口
│   ├── state.rs            # 状态管理（FileTreeState, EditorState）
│   ├── ui/
│   │   ├── mod.rs          # UI 模块
│   │   ├── main_ui.rs      # 主界面渲染
│   │   ├── file_tree.rs    # 文件树组件（基于 egui_ltreeview）
│   │   ├── editor.rs       # 文本编辑器组件
│   │   └── tabs.rs         # 标签页组件
│   ├── core/
│   │   ├── mod.rs          # 核心模块
│   │   ├── file_manager.rs # 文件管理
│   │   ├── text_buffer.rs  # 文本缓冲区（基于 ROPE）
│   │   └── syntax.rs       # 语法高亮
│   ├── settings/
│   │   └── editor_settings.rs # 编辑器设置
│   └── tests/              # 单元测试
└── examples/
    └── file_tree_demo.rs   # 文件树功能演示
```

### 关键数据结构

#### FileTreeState
```rust
pub struct FileTreeState {
    pub root_path: Option<PathBuf>,
    pub tree_view_state: TreeViewState<FileNodeId>,
    pub file_entries: HashMap<PathBuf, FileEntry>,
    pub directory_children: HashMap<PathBuf, Vec<PathBuf>>,
}
```

#### FileNodeId
```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FileNodeId(pub PathBuf);
```

## 🎯 操作指南

### 文件树操作

| 操作 | 说明 |
|------|------|
| 🖱️ 单击 | 选择文件/目录 |
| 🖱️ 双击 | 激活（打开文件/切换目录展开状态） |
| 🖱️ 右键 | 显示上下文菜单 |
| ⌨️ 方向键 | 键盘导航 |
| ⌨️ Ctrl/Cmd + 点击 | 多选 |
| ⌨️ Enter | 激活选中项 |
| ⌨️ Space | 切换选择状态 |

### 文件类型图标

| 文件类型 | 图标 |
|----------|------|
| 目录 | 📁 |
| Rust (.rs) | 🦀 |
| Python (.py) | 🐍 |
| JavaScript/TypeScript | 📜 |
| Markdown (.md) | 📝 |
| JSON | 📋 |
| TOML/YAML | ⚙️ |
| HTML | 🌐 |
| CSS | 🎨 |
| 图片 | 🖼️ |
| PDF | 📕 |
| 压缩包 | 📦 |
| 其他 | 📄 |

## 🧪 测试

运行单元测试：

```bash
cargo test
```

测试覆盖范围：
- ✅ 文件树状态管理
- ✅ 目录扫描和结构构建
- ✅ 文件条目属性检测
- ✅ 文件图标识别
- ✅ 树视图状态选择
- ✅ 刷新功能
- ✅ FileNodeId 类型转换

## 🔧 技术栈

### 核心依赖
- **egui**: UI 框架
- **egui_ltreeview**: 专业树形视图组件
- **crop**: ROPE 算法实现（高效文本编辑）
- **syntect**: 语法高亮支持
- **walkdir**: 目录遍历
- **notify**: 文件变化监控

### 开发依赖
- **tempfile**: 临时文件测试
- **mockall**: 模拟依赖
- **eframe**: 示例应用框架
- **rfd**: 文件对话框

## 📈 性能特性

### ROPE 数据结构优势
- **对数时间复杂度**：插入、删除、替换操作都是 O(log n)
- **内存友好**：只需要重新分配修改的部分
- **大文件支持**：可以高效处理 GB 级别的文件
- **零拷贝切片**：获取文本片段不需要复制数据

### 文件树性能
- **虚拟化支持**：egui_ltreeview 提供虚拟化，适合大型目录结构
- **懒加载**：按需加载目录内容
- **缓存机制**：缓存目录结构和文件条目
- **增量更新**：只更新变化的部分

## 🤝 贡献

欢迎提交 Issue 和 Pull Request！

## 📄 许可证

MIT License
