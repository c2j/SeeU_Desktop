# iSearch 搜索结果排序功能实现

## 🎯 功能目标

在 iSearch 搜索结果页面添加排序功能，支持按照以下条件对搜索结果进行排序：
1. **相关性** - 按搜索匹配度排序（默认）
2. **文件名** - 按文件名字母顺序排序
3. **目录名** - 按目录路径排序
4. **文件大小** - 按文件大小排序
5. **修改时间** - 按文件修改时间排序

每种排序条件都支持升序和降序两种排序方向。

## ✨ 实现功能

### 1. 🏗️ 数据结构设计

#### 排序条件枚举
```rust
/// Sort criteria for search results
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SortBy {
    Relevance,    // Default: by search score
    FileName,     // By file name
    DirectoryName, // By directory path
    FileSize,     // By file size
    ModifiedTime, // By modification time
}

impl SortBy {
    pub fn display_name(&self) -> &'static str {
        match self {
            SortBy::Relevance => "相关性",
            SortBy::FileName => "文件名",
            SortBy::DirectoryName => "目录名",
            SortBy::FileSize => "文件大小",
            SortBy::ModifiedTime => "修改时间",
        }
    }
}
```

#### 排序方向枚举
```rust
/// Sort order for search results
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SortOrder {
    Ascending,  // 升序
    Descending, // 降序
}

impl SortOrder {
    pub fn display_name(&self) -> &'static str {
        match self {
            SortOrder::Ascending => "升序",
            SortOrder::Descending => "降序",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            SortOrder::Ascending => "⬆",
            SortOrder::Descending => "⬇",
        }
    }
}
```

#### 状态字段扩展
```rust
pub struct ISearchState {
    // ... 其他字段 ...
    
    // Search result sorting
    pub sort_by: SortBy,
    pub sort_order: SortOrder,
}
```

### 2. 🔧 排序算法实现

#### 核心排序方法
```rust
/// Sort search results according to current sort settings
pub fn sort_results(&mut self) {
    match self.sort_by {
        SortBy::Relevance => {
            // Sort by search score
            match self.sort_order {
                SortOrder::Descending => {
                    self.search_results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
                }
                SortOrder::Ascending => {
                    self.search_results.sort_by(|a, b| a.score.partial_cmp(&b.score).unwrap_or(std::cmp::Ordering::Equal));
                }
            }
        }
        SortBy::FileName => {
            // Sort by file name (case-insensitive)
            match self.sort_order {
                SortOrder::Ascending => {
                    self.search_results.sort_by(|a, b| a.filename.to_lowercase().cmp(&b.filename.to_lowercase()));
                }
                SortOrder::Descending => {
                    self.search_results.sort_by(|a, b| b.filename.to_lowercase().cmp(&a.filename.to_lowercase()));
                }
            }
        }
        SortBy::DirectoryName => {
            // Sort by directory path (case-insensitive)
            match self.sort_order {
                SortOrder::Ascending => {
                    self.search_results.sort_by(|a, b| {
                        let dir_a = std::path::Path::new(&a.path).parent().unwrap_or(std::path::Path::new("")).to_string_lossy().to_lowercase();
                        let dir_b = std::path::Path::new(&b.path).parent().unwrap_or(std::path::Path::new("")).to_string_lossy().to_lowercase();
                        dir_a.cmp(&dir_b)
                    });
                }
                SortOrder::Descending => {
                    self.search_results.sort_by(|a, b| {
                        let dir_a = std::path::Path::new(&a.path).parent().unwrap_or(std::path::Path::new("")).to_string_lossy().to_lowercase();
                        let dir_b = std::path::Path::new(&b.path).parent().unwrap_or(std::path::Path::new("")).to_string_lossy().to_lowercase();
                        dir_b.cmp(&dir_a)
                    });
                }
            }
        }
        SortBy::FileSize => {
            // Sort by file size
            match self.sort_order {
                SortOrder::Ascending => {
                    self.search_results.sort_by(|a, b| a.size_bytes.cmp(&b.size_bytes));
                }
                SortOrder::Descending => {
                    self.search_results.sort_by(|a, b| b.size_bytes.cmp(&a.size_bytes));
                }
            }
        }
        SortBy::ModifiedTime => {
            // Sort by modification time
            match self.sort_order {
                SortOrder::Ascending => {
                    self.search_results.sort_by(|a, b| a.modified.cmp(&b.modified));
                }
                SortOrder::Descending => {
                    self.search_results.sort_by(|a, b| b.modified.cmp(&a.modified));
                }
            }
        }
    }
}
```

#### 智能排序切换
```rust
/// Change sort criteria and re-sort results
pub fn set_sort_by(&mut self, sort_by: SortBy) {
    // If clicking the same sort criteria, toggle the order
    if self.sort_by == sort_by {
        self.sort_order = match self.sort_order {
            SortOrder::Ascending => SortOrder::Descending,
            SortOrder::Descending => SortOrder::Ascending,
        };
    } else {
        // Different sort criteria, use default order for that criteria
        self.sort_by = sort_by.clone();
        self.sort_order = match sort_by {
            SortBy::Relevance => SortOrder::Descending, // Higher score first
            SortBy::FileName => SortOrder::Ascending,   // A-Z
            SortBy::DirectoryName => SortOrder::Ascending, // A-Z
            SortBy::FileSize => SortOrder::Descending,  // Larger files first
            SortBy::ModifiedTime => SortOrder::Descending, // Newer files first
        };
    }

    // Re-sort the current results
    self.sort_results();

    // Save sort preferences
    self.save_search_options();
}
```

### 3. 🎨 用户界面设计

#### 排序工具栏
```rust
// Sort controls
ui.horizontal(|ui| {
    ui.label("排序：");
    
    // Sort by buttons
    let sort_options = [
        SortBy::Relevance,
        SortBy::FileName,
        SortBy::DirectoryName,
        SortBy::FileSize,
        SortBy::ModifiedTime,
    ];
    
    for sort_option in &sort_options {
        let is_current = state.sort_by == *sort_option;
        let button_text = if is_current {
            format!("{} {}", sort_option.display_name(), state.sort_order.icon())
        } else {
            sort_option.display_name().to_string()
        };
        
        let button = if is_current {
            ui.add(egui::Button::new(button_text).fill(ui.visuals().selection.bg_fill))
        } else {
            ui.button(button_text)
        };
        
        if button.clicked() {
            state.set_sort_by(sort_option.clone());
        }
    }
});
```

#### 界面布局
```
┌─────────────────────────────────────────────────┐
│ 找到 25 个结果，耗时 0.12 秒                    │
├─────────────────────────────────────────────────┤
│ 排序： [相关性 ⬇] [文件名] [目录名] [文件大小] [修改时间] │
├─────────────────────────────────────────────────┤
│ 📄 document1.pdf                                │
│ 📂 /Users/username/Documents                    │
│ 内容预览: This is a sample document...          │
│ [打开文件] [打开文件夹] [...]                   │
├─────────────────────────────────────────────────┤
│ 📝 report.docx                                  │
│ 📂 /Users/username/Work                         │
│ 内容预览: Quarterly report for...               │
│ [打开文件] [打开文件夹] [...]                   │
└─────────────────────────────────────────────────┘
```

### 4. 💾 配置持久化

#### 保存排序设置
```rust
/// Save search options to disk
pub fn save_search_options(&self) {
    let options = serde_json::json!({
        "enable_content_preview": self.enable_content_preview,
        "enable_file_type_filter": self.enable_file_type_filter,
        "search_hidden_files": self.search_hidden_files,
        "enable_file_monitoring": self.enable_file_monitoring,
        "sort_by": self.sort_by,
        "sort_order": self.sort_order,
    });
    // 保存到配置文件
}
```

#### 加载排序设置
```rust
/// Load search options from disk
fn load_search_options(&mut self) {
    // 从配置文件加载
    if let Some(sort_by) = options.get("sort_by") {
        if let Ok(sort_by_enum) = serde_json::from_value::<SortBy>(sort_by.clone()) {
            self.sort_by = sort_by_enum;
        }
    }
    if let Some(sort_order) = options.get("sort_order") {
        if let Ok(sort_order_enum) = serde_json::from_value::<SortOrder>(sort_order.clone()) {
            self.sort_order = sort_order_enum;
        }
    }
}
```

### 5. 🔄 搜索流程集成

#### 搜索后自动排序
```rust
/// Search for files
pub fn search(&mut self) {
    // ... 执行搜索逻辑 ...
    
    // Update state with results
    self.search_results = final_results;
    
    // Apply current sort settings
    self.sort_results();
    
    self.is_searching = false;
}
```

## 🎯 功能特点

### 1. 智能排序逻辑
- **默认排序** - 每种排序条件都有合理的默认排序方向
- **切换逻辑** - 点击相同排序条件时切换升序/降序
- **即时生效** - 排序更改后立即重新排列搜索结果

### 2. 用户体验优化
- **视觉反馈** - 当前排序条件高亮显示，并显示排序方向图标
- **一键操作** - 单击按钮即可切换排序条件和方向
- **持久保存** - 用户的排序偏好自动保存并在下次启动时恢复

### 3. 性能考虑
- **内存排序** - 在内存中对搜索结果进行排序，速度快
- **增量更新** - 只在需要时重新排序，避免不必要的计算
- **配置缓存** - 排序设置保存到磁盘，避免重复配置

## 📊 排序算法详解

### 1. 相关性排序
- **算法** - 按搜索引擎返回的相关性评分排序
- **默认方向** - 降序（高分在前）
- **特点** - 最相关的结果优先显示

### 2. 文件名排序
- **算法** - 按文件名字母顺序排序（忽略大小写）
- **默认方向** - 升序（A-Z）
- **特点** - 便于按文件名查找特定文件

### 3. 目录名排序
- **算法** - 按文件所在目录的路径排序（忽略大小写）
- **默认方向** - 升序（A-Z）
- **特点** - 将同一目录的文件聚集在一起

### 4. 文件大小排序
- **算法** - 按文件字节大小排序
- **默认方向** - 降序（大文件在前）
- **特点** - 便于查找大文件或小文件

### 5. 修改时间排序
- **算法** - 按文件最后修改时间排序
- **默认方向** - 降序（最新文件在前）
- **特点** - 便于查找最近修改的文件

## 🔧 技术实现亮点

### 1. 类型安全设计
- **强类型枚举** - 使用 Rust 枚举确保排序条件的类型安全
- **序列化支持** - 支持配置的序列化和反序列化
- **默认值处理** - 为所有排序选项提供合理的默认值

### 2. 智能交互设计
- **状态管理** - 排序状态与搜索状态分离，便于管理
- **用户意图识别** - 智能判断用户是要切换排序条件还是排序方向
- **即时反馈** - 排序更改后立即更新界面显示

### 3. 性能优化
- **原地排序** - 直接对搜索结果向量进行排序，避免额外内存分配
- **条件分支** - 使用 match 语句优化排序算法的选择
- **缓存友好** - 排序算法对 CPU 缓存友好

## 🎨 用户界面设计原则

### 1. 直观易用
- **清晰标识** - 排序按钮文字清晰，功能明确
- **视觉层次** - 当前排序条件通过高亮和图标突出显示
- **操作简单** - 单击即可完成排序切换

### 2. 一致性
- **设计风格** - 排序工具栏与整体界面风格一致
- **交互模式** - 与其他控件的交互方式保持一致
- **视觉元素** - 使用统一的颜色、字体、间距

### 3. 响应性
- **即时反馈** - 点击排序按钮后立即看到结果变化
- **状态同步** - 界面状态与实际排序状态保持同步
- **性能保证** - 排序操作快速完成，不阻塞界面

## 📝 总结

这次搜索结果排序功能的实现成功提升了 iSearch 的用户体验：

### 核心成就
1. **多维度排序** - 支持 5 种不同的排序条件
2. **智能交互** - 点击相同条件切换排序方向，点击不同条件切换排序类型
3. **持久化配置** - 用户的排序偏好自动保存和恢复
4. **性能优化** - 高效的内存排序算法，响应迅速

### 技术价值
- **类型安全** - 使用 Rust 强类型系统确保代码安全性
- **模块化设计** - 排序功能独立封装，易于维护和扩展
- **配置管理** - 完善的配置持久化机制

### 用户价值
- **查找效率** - 用户可以根据需要快速重新组织搜索结果
- **个性化体验** - 支持用户根据使用习惯自定义排序偏好
- **操作便利** - 简单直观的排序控制界面

现在用户在使用 iSearch 时可以享受到：
- 🔄 **灵活排序** - 5 种排序条件，升序降序任选
- ⚡ **即时生效** - 点击排序按钮立即重新排列结果
- 💾 **偏好记忆** - 排序设置自动保存，下次使用时恢复
- 🎯 **智能切换** - 点击相同条件切换方向，点击不同条件切换类型
- 📊 **多维度查看** - 按相关性、文件名、目录、大小、时间等多角度组织结果

这次功能增强让 iSearch 的搜索结果展示变得更加灵活和用户友好！
