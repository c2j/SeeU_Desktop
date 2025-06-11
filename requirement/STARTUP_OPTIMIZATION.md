# SeeU Desktop 启动性能优化

## 问题分析

在存在较多笔记、搜索索引的情况下，应用程序启动比较慢。经过代码分析，发现以下主要性能瓶颈：

### 1. 笔记模块启动时同步加载所有数据
- `DbINoteState::initialize()` 在启动时同步加载所有笔记本、标签和笔记
- `load_all_notes()` 会遍历所有笔记本并加载每个笔记本的所有笔记
- 每个笔记还要单独查询标签和附件信息

### 2. 搜索模块启动时初始化索引
- `ISearchState::initialize()` 在启动时初始化搜索索引
- 启动文件监控器监听所有已索引目录

### 3. 数据迁移在启动时同步执行
- `migrate_data()` 在每次启动时都会检查并执行数据迁移

### 4. 插件管理器启动时加载所有插件
- 扫描和加载已安装的插件

## 优化方案

### 1. 实现懒加载机制

#### 笔记模块优化
- **修改**: `crates/inote/src/db_state.rs`
- **变更**:
  - `initialize()` 方法不再加载所有笔记，只加载笔记本和标签
  - 添加 `load_notes_for_notebook()` 懒加载机制
  - 在 `select_notebook()` 时才加载对应笔记本的笔记
  - 添加重复加载检查，避免重复加载已加载的笔记

```rust
// 启动时只加载基础数据
pub fn initialize(&mut self) {
    self.migrate_data_async();  // 异步迁移
    self.load_notebooks();      // 只加载笔记本
    self.load_tags();          // 只加载标签
    // 不再加载所有笔记
}

// 选择笔记本时才加载笔记
pub fn select_notebook(&mut self, index: usize) {
    // ... 选择逻辑
    let notebook_id = self.notebooks[index].id.clone();
    self.load_notes_for_notebook(&notebook_id);  // 懒加载
}
```

### 2. 异步初始化

#### 搜索模块优化
- **修改**: `crates/isearch/src/lib.rs`
- **变更**:
  - 搜索索引初始化改为异步执行
  - 文件监控器启动改为异步执行

```rust
// 异步初始化搜索索引
std::thread::spawn(move || {
    if let Ok(indexer_lock) = indexer_clone.lock() {
        if let Err(e) = indexer_lock.initialize_index() {
            log::error!("Failed to initialize search index: {}", e);
        }
    }
});
```

#### 插件管理器优化
- **修改**: `crates/itools/src/plugins/manager.rs`
- **变更**:
  - 插件加载改为异步执行

### 3. 数据库性能优化

#### 添加数据库索引
- **修改**: `crates/inote/src/db_storage.rs`
- **新增**: `create_indexes()` 方法
- **索引**:
  - `notes.notebook_id` - 加速按笔记本查询
  - `notes.updated_at` - 加速排序
  - `note_tags.note_id` 和 `note_tags.tag_id` - 加速标签查询
  - `attachments.note_id` - 加速附件查询
  - 全文搜索索引 - 使用 SQLite FTS5

```sql
-- 性能索引
CREATE INDEX idx_notes_notebook_id ON notes(notebook_id);
CREATE INDEX idx_notes_updated_at ON notes(updated_at);
CREATE INDEX idx_note_tags_note_id ON note_tags(note_id);
CREATE INDEX idx_note_tags_tag_id ON note_tags(tag_id);

-- 全文搜索索引
CREATE VIRTUAL TABLE notes_fts USING fts5(
    id UNINDEXED, title, content,
    content='notes', content_rowid='rowid'
);
```

### 4. 启动配置系统

#### 新增配置模块
- **新增**: `src/config/startup.rs`
- **功能**:
  - 可配置的启动行为
  - 启动性能指标收集
  - 启动进度显示

```rust
pub struct StartupConfig {
    pub lazy_load_notes: bool,           // 启用懒加载
    pub async_initialization: bool,      // 启用异步初始化
    pub show_startup_progress: bool,     // 显示启动进度
    pub background_migration: bool,      // 后台数据迁移
    pub background_indexing: bool,       // 后台索引构建
}
```

### 5. 启动进度指示

#### 启动画面
- **修改**: `src/app.rs`
- **新增**:
  - 启动进度跟踪
  - 启动画面渲染
  - 性能指标记录

```rust
// 启动画面显示
fn render_startup_screen(&mut self, ctx: &egui::Context) {
    // 显示应用logo、进度条和加载信息
}
```

## 性能提升效果

### 预期改进
1. **启动时间减少 60-80%**
   - 不再同步加载所有笔记数据
   - 异步执行耗时操作

2. **内存使用优化**
   - 按需加载笔记数据
   - 减少启动时的内存峰值

3. **用户体验改善**
   - 启动画面提供视觉反馈
   - 应用更快响应用户操作

### 配置选项
用户可以通过配置文件 `~/.config/seeu_desktop/startup.toml` 调整启动行为：

```toml
lazy_load_notes = true
async_initialization = true
show_startup_progress = true
background_migration = true
background_indexing = true
startup_timeout_seconds = 30
```

## 兼容性说明

- 所有优化都是向后兼容的
- 现有数据不受影响
- 用户可以选择禁用某些优化功能

## 后续优化建议

1. **增量索引更新** - 只更新变更的文件
2. **缓存机制** - 缓存常用查询结果
3. **分页加载** - 大量数据分页显示
4. **预加载策略** - 智能预测用户需要的数据
