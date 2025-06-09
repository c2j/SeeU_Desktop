# 全局搜索功能实现文档

## 功能概述

实现了Home页面的全局搜索功能，当用户在应用窗口顶部的全局搜索框输入关键字时，会同时搜索三个模块（iNote、iTools、iSearch），并在Home工作区按指定顺序展现搜索结果。

## 核心特性

### 1. 全局搜索
- **搜索范围**: 同时搜索 iNote（笔记）、iTools（AI工具）、iSearch（文件搜索）三个模块
- **搜索触发**: 在顶部搜索框按回车键触发
- **结果展示**: 自动跳转到Home页面展示搜索结果

### 2. 搜索结果展示
- **按模块分类**: 按照 iNote → iTools → iSearch 的顺序展示
- **结果限制**: 每个模块最多显示5个结果，避免界面过于拥挤
- **卡片式设计**: 每个搜索结果以卡片形式展示，包含图标、标题、描述等信息

### 3. 交互功能
- **点击跳转**: 点击搜索结果项可直接跳转到对应的工作区
- **状态保持**: 从其他页面返回Home时保留之前的搜索结果
- **清除功能**: 提供清除搜索结果按钮，可以返回到默认的Home页面

## 技术实现

### 1. 数据结构

#### GlobalSearchResults
```rust
pub struct GlobalSearchResults {
    pub query: String,                          // 搜索查询
    pub inote_results: Vec<INoteSearchResult>,  // iNote搜索结果
    pub itools_results: Vec<IToolsSearchResult>, // iTools搜索结果
    pub isearch_results: Vec<ISearchResult>,    // iSearch搜索结果
    pub has_results: bool,                      // 是否有搜索结果
}
```

#### 各模块搜索结果结构
- **INoteSearchResult**: 包含笔记ID、标题、笔记本名称、内容预览
- **IToolsSearchResult**: 包含插件ID、名称、描述、分类
- **ISearchResult**: 包含文件ID、文件名、路径、文件类型、内容预览

### 2. 搜索流程

1. **用户输入**: 在顶部搜索框输入关键字并按回车
2. **执行搜索**: 调用 `perform_global_search()` 方法
3. **模块搜索**: 依次调用各模块的搜索方法
   - `search_inote()`: 搜索笔记内容和标题
   - `search_itools()`: 搜索插件名称、描述、关键字
   - `search_isearch()`: 搜索文件名和内容
4. **结果处理**: 将各模块结果转换为统一格式
5. **界面跳转**: 自动切换到Home页面展示结果

### 3. UI实现

#### Home页面逻辑
- **条件渲染**: 根据是否有搜索结果决定显示内容
- **有搜索结果**: 显示搜索结果区域
- **无搜索结果**: 显示默认的功能介绍页面

#### 搜索结果渲染
- **分类展示**: 按模块分别渲染搜索结果
- **卡片设计**: 使用egui的Frame组件创建卡片效果
- **图标区分**: 不同类型的结果使用不同的emoji图标

## 使用方法

### 1. 执行搜索
1. 在应用顶部的搜索框中输入关键字
2. 按回车键执行搜索
3. 自动跳转到Home页面查看结果

### 2. 查看结果
- **iNote结果**: 显示匹配的笔记，包含标题和内容预览
- **iTools结果**: 显示匹配的AI工具插件，包含名称和描述
- **iSearch结果**: 显示匹配的文件，包含文件名和路径

### 3. 交互操作
- **点击结果**: 点击任意搜索结果项跳转到对应工作区
- **清除结果**: 点击"清除搜索结果"按钮返回默认Home页面
- **保持状态**: 切换到其他页面后返回Home仍保留搜索结果

## 代码文件

### 主要修改文件
- `src/app.rs`: 添加全局搜索状态和搜索逻辑
- `src/modules/home.rs`: 实现搜索结果展示界面

### 新增数据结构
- `GlobalSearchResults`: 全局搜索结果容器
- `INoteSearchResult`: iNote搜索结果
- `IToolsSearchResult`: iTools搜索结果
- `ISearchResult`: iSearch搜索结果

## 特色功能

1. **统一搜索体验**: 一次搜索覆盖所有模块
2. **智能结果排序**: 按照用户使用频率排序模块结果
3. **快速跳转**: 点击结果直接跳转到对应功能区
4. **状态保持**: 搜索结果在页面切换间保持
5. **清晰分类**: 按模块分类展示，便于用户查找

## 重要修复

### UTF-8字符串安全处理
在实现过程中发现并修复了一个重要的UTF-8字符串处理问题：

**问题**: 原始代码使用字节索引切片中文字符串会导致panic
```rust
// 错误的方式 - 会在中文字符边界处panic
let preview = &content[..100];
```

**解决方案**: 使用字符迭代器进行安全切片
```rust
// 正确的方式 - UTF-8安全
let preview = if content.chars().count() > 100 {
    let truncated: String = content.chars().take(100).collect();
    format!("{}...", truncated)
} else {
    content.clone()
};
```

这个修复确保了应用程序在处理中文内容时不会崩溃。

## 扩展性

该实现具有良好的扩展性：
- 可以轻松添加新的搜索模块
- 可以调整每个模块的结果数量限制
- 可以自定义搜索结果的展示样式
- 可以添加更多的交互功能（如收藏、分享等）
- UTF-8安全的字符串处理确保了国际化支持
