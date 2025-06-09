# iSearch 目录索引统计功能增强

## 🎯 功能目标

为 iSearch 索引目录管理框中的每个目录添加详细的索引统计信息，包括已索引文件数和索引大小，让用户清楚了解每个目录的索引状态和规模。

## ✨ 新增功能

### 1. 📊 目录级别统计信息

#### 每个目录显示的信息
- **📁 目录路径** - 目录的完整路径
- **📄 文件数量** - 该目录中已索引的文件数量
- **💾 索引大小** - 该目录索引的总文件大小（MB）
- **🕒 最后索引时间** - 目录最后一次索引的时间

### 2. 🎨 界面优化

#### 搜索界面目录列表
- **分组显示** - 每个目录使用独立的分组框
- **垂直布局** - 目录信息垂直排列，信息清晰
- **统计信息** - 文件数和大小水平排列
- **时间显示** - 简化的时间格式（月-日 时:分）

#### 设置界面目录列表
- **详细信息** - 与搜索界面保持一致的显示格式
- **实时更新** - 索引完成后统计信息立即更新
- **状态指示** - 未索引目录显示"未索引"状态

## 🔧 技术实现

### 1. 数据结构扩展

#### IndexedDirectory 结构增强
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexedDirectory {
    pub path: String,
    pub last_indexed: Option<DateTime<Utc>>,
    #[serde(default)]  // 向后兼容
    pub file_count: usize,
    #[serde(default)]  // 向后兼容
    pub total_size_bytes: u64,
}
```

#### DirectoryIndexResult 新增结构
```rust
#[derive(Debug, Clone)]
pub struct DirectoryIndexResult {
    pub directory_path: String,
    pub stats: IndexStats,
}
```

### 2. 通信机制优化

#### 后台索引结果传递
```rust
// 修改通道类型以传递目录特定的结果
stats_sender: Option<Sender<DirectoryIndexResult>>,
stats_receiver: Option<Receiver<DirectoryIndexResult>>,

// 后台线程发送结果时包含目录路径
let result = DirectoryIndexResult {
    directory_path: directory_clone.path.clone(),
    stats,
};
let _ = sender.send(result);
```

#### 结果处理优化
```rust
pub fn check_reindex_results(&mut self) {
    while let Ok(result) = receiver.try_recv() {
        // 更新全局统计
        self.index_stats.total_files += result.stats.total_files;
        self.index_stats.total_size_bytes += result.stats.total_size_bytes;
        
        // 更新特定目录的统计
        for directory in &mut self.indexed_directories {
            if directory.path == result.directory_path {
                directory.file_count = result.stats.total_files;
                directory.total_size_bytes = result.stats.total_size_bytes;
                directory.last_indexed = Some(Utc::now());
                break;
            }
        }
    }
}
```

### 3. 界面渲染增强

#### 搜索界面目录列表
```rust
ui.group(|ui| {
    ui.vertical(|ui| {
        // Directory path
        if ui.selectable_label(is_selected, format!("📁 {}", directory.path)).clicked() {
            state.selected_directory = Some(i);
        }
        
        // Directory stats
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new(format!("📄 {} 个文件", directory.file_count)).small().weak());
            ui.label(egui::RichText::new(format!("💾 {:.1} MB", directory.total_size_bytes as f64 / (1024.0 * 1024.0))).small().weak());
        });
        
        // Last indexed time
        if let Some(last_indexed) = directory.last_indexed {
            ui.label(egui::RichText::new(format!("🕒 {}", last_indexed.format("%m-%d %H:%M"))).small().weak());
        } else {
            ui.label(egui::RichText::new("🕒 未索引").small().weak());
        }
    });
});
```

#### 设置界面目录列表
```rust
ui.group(|ui| {
    ui.vertical(|ui| {
        // Directory path
        if ui.selectable_label(is_selected, format!("📁 {}", directory.path)).clicked() {
            app.isearch_state.selected_directory = Some(i);
        }
        
        // Directory stats in a horizontal layout
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new(format!("📄 {} 个文件", directory.file_count)).small().weak());
            ui.label(egui::RichText::new(format!("💾 {:.1} MB", directory.total_size_bytes as f64 / (1024.0 * 1024.0))).small().weak());
        });
        
        // Last indexed time
        if let Some(last_indexed) = directory.last_indexed {
            ui.label(egui::RichText::new(format!("🕒 最后索引: {}", last_indexed.format("%m-%d %H:%M"))).small().weak());
        } else {
            ui.label(egui::RichText::new("🕒 未索引").small().weak());
        }
    });
});
```

### 4. 向后兼容性

#### 序列化兼容性
```rust
// 使用 #[serde(default)] 确保旧数据能正常加载
#[serde(default)]
pub file_count: usize,
#[serde(default)]
pub total_size_bytes: u64,
```

#### 数据迁移
- 现有目录自动获得默认值（0个文件，0字节）
- 重新索引后会更新为正确的统计信息
- 不影响现有的索引数据和配置

## 📊 界面设计

### 搜索界面目录列表布局
```
┌─────────────────────────────────────────────────┐
│ 📁 /Users/username/Documents                    │
│ 📄 1,234 个文件    💾 567.8 MB                  │
│ 🕒 12-15 14:30                                  │
└─────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────┐
│ 📁 /Users/username/Projects                     │
│ 📄 2,567 个文件    💾 1.2 GB                    │
│ 🕒 12-15 16:45                                  │
└─────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────┐
│ 📁 /Users/username/Downloads                    │
│ 📄 0 个文件        💾 0.0 MB                    │
│ 🕒 未索引                                       │
└─────────────────────────────────────────────────┘
```

### 设置界面目录列表布局
```
已索引的目录：
┌─────────────────────────────────────────────────┐
│ 📁 /Users/username/Documents                    │
│ 📄 1,234 个文件    💾 567.8 MB                  │
│ 🕒 最后索引: 12-15 14:30                        │
└─────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────┐
│ 📁 /Users/username/Projects                     │
│ 📄 2,567 个文件    💾 1.2 GB                    │
│ 🕒 最后索引: 12-15 16:45                        │
└─────────────────────────────────────────────────┘
```

## 🎯 用户体验提升

### 信息透明度
- ✅ **目录规模可见** - 用户可以清楚看到每个目录的文件数量
- ✅ **存储占用明确** - 显示每个目录的索引大小
- ✅ **索引状态清晰** - 区分已索引和未索引的目录
- ✅ **时间信息准确** - 显示每个目录的最后索引时间

### 管理便利性
- ✅ **目录对比** - 可以比较不同目录的索引规模
- ✅ **状态监控** - 实时了解索引进度和完成状态
- ✅ **决策支持** - 基于统计信息决定是否需要重新索引
- ✅ **存储管理** - 了解哪些目录占用更多索引空间

### 界面一致性
- ✅ **统一显示** - 搜索界面和设置界面显示格式一致
- ✅ **视觉清晰** - 使用图标和颜色区分不同类型的信息
- ✅ **布局合理** - 信息层次分明，易于阅读
- ✅ **响应及时** - 索引完成后统计信息立即更新

## 🔍 功能对比

### 增强前
```
📁 /Users/username/Documents
🕒 最后索引: 12-15 14:30

📁 /Users/username/Projects  
🕒 未索引
```

### 增强后
```
┌─────────────────────────────────────────────────┐
│ 📁 /Users/username/Documents                    │
│ 📄 1,234 个文件    💾 567.8 MB                  │
│ 🕒 12-15 14:30                                  │
└─────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────┐
│ 📁 /Users/username/Projects                     │
│ 📄 0 个文件        💾 0.0 MB                    │
│ 🕒 未索引                                       │
└─────────────────────────────────────────────────┘
```

## 📝 总结

这次功能增强成功为 iSearch 添加了详细的目录级别统计信息：

### 核心成就
1. **信息丰富化** - 每个目录显示文件数、大小、索引时间
2. **界面优化** - 使用分组和图标提升视觉效果
3. **数据准确性** - 实时更新目录特定的统计信息
4. **向后兼容** - 现有数据无缝迁移到新格式

### 技术价值
- **数据结构扩展** - 合理的结构设计支持新功能
- **通信机制优化** - 目录特定的结果传递机制
- **界面一致性** - 搜索和设置界面统一的显示格式
- **兼容性保证** - 不影响现有用户的数据和配置

### 用户价值
- **透明度提升** - 清楚了解每个目录的索引状况
- **管理便利** - 基于统计信息进行目录管理决策
- **状态监控** - 实时掌握索引进度和完成状态
- **存储意识** - 了解不同目录的存储占用情况

现在用户可以在目录管理界面看到：
- 📊 **详细统计** - 每个目录的文件数和大小
- 🕒 **时间信息** - 最后索引时间或未索引状态
- 🎨 **清晰布局** - 分组显示，信息层次分明
- ⚡ **实时更新** - 索引完成后立即显示最新统计

这次增强让 iSearch 的目录管理功能更加专业和实用，为用户提供了全面的索引状态监控能力！
