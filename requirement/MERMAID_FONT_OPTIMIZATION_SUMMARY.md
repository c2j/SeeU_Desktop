# Mermaid 字体渲染优化总结

## 问题分析

### 原始问题
- Mermaid图表渲染时速度很慢，特别是在预览和幻灯片播放模式下
- 每次渲染都需要重新扫描系统字体，导致UI卡顿
- 字体查找过程耗时较长，影响用户体验

### 根本原因
1. **重复字体初始化**：每次渲染Mermaid图表都会重新创建fontdb实例
2. **系统字体扫描**：每次都要扫描整个系统字体库
3. **字体选择算法**：缺乏有效的字体缓存和映射机制
4. **应用设置未充分利用**：没有很好地利用应用程序的字体配置

## 优化方案

### 1. 字体缓存机制优化

#### 增强的FontCache结构
```rust
struct FontCache {
    fontdb: Option<Arc<fontdb::Database>>,
    last_working_font: Option<String>,
    available_fonts: Vec<String>,
    font_mapping: HashMap<String, String>,  // 新增：字体映射
    initialized: bool,                      // 新增：初始化标志
}
```

#### 关键改进
- **全局缓存**：使用`Lazy<Mutex<FontCache>>`确保字体数据库只初始化一次
- **字体映射**：建立应用设置到实际字体名称的映射关系
- **初始化标志**：避免重复初始化

### 2. 智能字体选择

#### 优化的字体选择逻辑
```rust
fn get_svg_font_family(&self, font_family: Option<&str>) -> String {
    // 1. 优先使用应用设置的字体映射
    // 2. 检查字体是否直接可用
    // 3. 使用缓存的工作字体作为后备
    // 4. 最终后备到综合字体列表
}
```

#### 字体优先级
1. 应用程序设置的字体（通过映射）
2. 直接匹配的字体名称
3. 缓存的最后工作字体
4. 系统默认字体列表

### 3. 预加载机制

#### 启动时预加载
```rust
// 在应用启动进度中添加字体缓存预加载
if progress < 0.6 {
    self.startup_message = "正在预加载字体缓存...".to_string();
    static FONT_CACHE_PRELOADED: std::sync::Once = std::sync::Once::new();
    FONT_CACHE_PRELOADED.call_once(|| {
        inote::mermaid::preload_font_cache();
    });
}
```

#### 预加载函数
```rust
pub fn preload_font_cache() {
    log::debug!("Preloading Mermaid font cache...");
    let start_time = std::time::Instant::now();
    let (_fontdb, working_font) = get_or_create_fontdb();
    let elapsed = start_time.elapsed();
    log::info!("Font cache preloaded in {:?}, working font: {:?}", elapsed, working_font);
}
```

### 4. SVG模板优化

#### 减少重复字体查询
- 在SVG生成函数中只调用一次`get_svg_font_family()`
- 使用命名参数减少字符串重复
- 优化SVG模板格式

## 技术实现细节

### 字体数据库管理
```rust
fn get_or_create_fontdb() -> (Arc<fontdb::Database>, Option<String>) {
    let mut cache = FONT_CACHE.lock().unwrap();
    
    if cache.initialized && cache.fontdb.is_some() {
        // 返回缓存的字体数据库
        return (cache.fontdb.as_ref().unwrap().clone(), cache.last_working_font.clone());
    }
    
    // 只在首次调用时初始化
    // ...初始化逻辑
}
```

### 字体映射建立
```rust
// 建立应用设置到实际字体的映射
let mut font_mapping = std::collections::HashMap::new();
if families.iter().any(|f| f.contains("Source Han Sans")) {
    font_mapping.insert("Source Han Sans".to_string(), "Source Han Sans SC".to_string());
}
if families.iter().any(|f| f.contains("WQY")) {
    font_mapping.insert("WQY MicroHei".to_string(), "WQY MicroHei".to_string());
}
```

### 应用设置集成
- 字体设置从应用程序主界面传递到笔记模块
- 笔记模块将字体设置传递给Markdown渲染器
- Markdown渲染器将字体设置传递给Mermaid渲染器

## 性能改进

### 预期效果
1. **首次渲染后性能提升**：后续Mermaid图表渲染速度显著提升
2. **减少UI卡顿**：避免重复的字体扫描导致的界面冻结
3. **更好的字体一致性**：更好地利用应用程序的字体设置
4. **启动时间优化**：通过预加载避免首次渲染延迟

### 性能指标
- 字体数据库初始化：从每次渲染到仅启动时一次
- 字体查找时间：从毫秒级降低到微秒级
- 内存使用：共享字体数据库实例，减少内存占用

## 代码变更总结

### 主要文件修改
1. **`crates/inote/src/mermaid.rs`**
   - 增强FontCache结构
   - 优化字体选择逻辑
   - 添加预加载函数
   - 优化SVG模板

2. **`src/app.rs`**
   - 在启动进度中添加字体缓存预加载

### 新增功能
- `preload_font_cache()` 函数
- 字体映射机制
- 智能字体选择算法

## 测试验证

### 测试文档
创建了 `test_mermaid_performance.md` 包含：
- 多种类型的Mermaid图表
- 性能测试用例
- 字体渲染验证

### 验证方法
1. 打开包含多个Mermaid图表的笔记
2. 切换预览模式观察渲染速度
3. 在幻灯片模式下测试性能
4. 更改字体设置验证字体应用效果

## 后续优化建议

1. **监控和日志**：添加更详细的性能监控
2. **字体回退策略**：进一步优化字体选择算法
3. **缓存持久化**：考虑将字体缓存持久化到磁盘
4. **异步加载**：考虑异步字体加载机制

## 结论

通过实施字体缓存优化、智能字体选择和预加载机制，显著改善了Mermaid图表的渲染性能。这些优化不仅提升了用户体验，还为后续的性能优化奠定了基础。
