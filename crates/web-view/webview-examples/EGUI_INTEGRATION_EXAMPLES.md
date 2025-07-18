# egui + WebView Integration Examples

本目录包含了两个展示如何将 `web-view` crate 与 `egui 0.28.1` 框架集成的完整示例。

## 📁 示例文件

### 1. 基础集成示例
- **文件**: `examples/egui_embedded_webview.rs`
- **说明文档**: `examples/egui_embedded_webview_README.md`
- **功能**: 基础的egui + WebView集成

### 2. 高级集成示例
- **文件**: `examples/egui_webview_advanced.rs`
- **功能**: 多窗口管理、高级控制、状态监控

### 3. 安全集成示例
- **文件**: `examples/egui_webview_safe.rs`
- **功能**: 使用系统浏览器命令的安全集成方案

### 4. 安全嵌入式示例
- **文件**: `examples/egui_webview_embedded_safe.rs`
- **功能**: 真正的WebView嵌入，使用单线程安全操作

## 🚀 快速开始

### 运行基础示例
```bash
cargo run -p webview-examples --example egui_embedded_webview
```

### 运行高级示例
```bash
cargo run -p webview-examples --example egui_webview_advanced
```

### 运行安全示例
```bash
cargo run -p webview-examples --example egui_webview_safe
```

### 运行安全嵌入式示例
```bash
cargo run -p webview-examples --example egui_webview_embedded_safe
```

## ✨ 功能特性对比

| 功能 | 基础示例 | 高级示例 | 安全示例 | 嵌入式示例 |
|------|---------|---------|---------|------------|
| egui原生界面 | ✅ | ✅ | ✅ | ✅ |
| WebView窗口创建 | ✅ | ✅ | ✅ | ✅ |
| URL导航控制 | ✅ | ✅ | ✅ | ✅ |
| 快速链接按钮 | ✅ | ✅ | ✅ | ✅ |
| 状态显示 | ✅ | ✅ | ✅ | ✅ |
| 多窗口管理 | ❌ | ✅ | ✅ | ❌ |
| 菜单栏 | ❌ | ✅ | ✅ | ❌ |
| 高级控制 | ❌ | ✅ | ❌ | ✅ |
| 状态消息历史 | ❌ | ✅ | ✅ | ❌ |
| 窗口生命周期管理 | ❌ | ✅ | ❌ | ✅ |
| 真正的WebView嵌入 | ❌ | ❌ | ❌ | ✅ |
| JavaScript执行 | ❌ | ❌ | ❌ | ✅ |
| 线程安全 | ✅ | ✅ | ✅ | ✅ |
| 生产环境适用 | ✅ | ✅ | ✅ | ⚠️ |

## 🏗️ 技术架构

### 核心组件
- **egui**: 即时模式GUI框架，负责主界面渲染
- **web-view**: 原生WebView绑定，负责Web内容显示
- **线程模型**: WebView在独立线程中运行，避免阻塞主UI
- **状态管理**: 通过 `Arc<Mutex<T>>` 实现线程间状态共享

### 平台支持
- **macOS**: Cocoa + WebKit
- **Linux**: GTK + WebKit2
- **Windows**: Win32 + MSHTML/Edge

## 🔧 依赖配置

已在 `Cargo.toml` 中添加必要的依赖：

```toml
[dev-dependencies]
# ... 其他依赖 ...
# egui dependencies for embedded webview example
egui = "0.28.1"
eframe = "0.28.1"
env_logger = "0.10.0"
```

## 📋 兼容性验证

### ✅ 编译兼容性
- 所有示例都能成功编译
- 无版本冲突或依赖问题
- 与egui 0.28.1完全兼容

### ✅ 运行时兼容性
- egui和web-view在不同层面工作，无冲突
- 线程隔离确保稳定性
- 跨平台一致性

## 🎯 示例详解

### 基础示例 (`egui_embedded_webview.rs`)

**主要特性**:
- 简洁的单窗口界面
- URL输入和导航
- 快速链接按钮
- 实时状态显示
- 技术细节展示

**适用场景**:
- 学习egui + WebView集成基础
- 简单的Web内容展示需求
- 概念验证和原型开发

### 高级示例 (`egui_webview_advanced.rs`)

**主要特性**:
- 多窗口管理系统
- 完整的菜单栏
- 高级控制面板
- 状态消息历史
- 窗口生命周期管理
- 预设快速创建

**适用场景**:
- 复杂的多窗口应用
- 需要高级控制的场景
- 生产级应用开发参考

## 🔍 代码结构

### 基础示例结构
```rust
struct EguiWebViewApp {
    url_input: String,
    status_message: String,
    webview_manager: Arc<Mutex<WebViewManager>>,
    webview_open: bool,
}

struct WebViewManager {
    current_url: String,
    is_running: bool,
    thread_handle: Option<thread::JoinHandle<()>>,
}
```

### 高级示例结构
```rust
struct AdvancedEguiWebViewApp {
    url_input: String,
    title_input: String,
    status_messages: Vec<String>,
    webview_manager: Arc<Mutex<AdvancedWebViewManager>>,
    show_advanced: bool,
    auto_refresh_interval: u32,
}

struct AdvancedWebViewManager {
    windows: HashMap<String, WebViewWindow>,
    next_window_id: u32,
}

struct WebViewWindow {
    id: String,
    title: String,
    url: String,
    is_running: bool,
    thread_handle: Option<thread::JoinHandle<()>>,
    created_at: std::time::Instant,
}
```

## 🛠️ 扩展建议

基于这些示例，你可以进一步扩展：

1. **双向通信**: 实现egui与WebView之间的消息传递
2. **内容嵌入**: 尝试将WebView内容直接嵌入到egui界面
3. **状态持久化**: 保存和恢复浏览历史和设置
4. **高级功能**: 添加书签、历史记录、下载管理等
5. **自定义渲染**: 实现自定义的Web内容渲染

## 📚 相关资源

- [egui官方文档](https://docs.rs/egui/)
- [web-view crate文档](https://docs.rs/web-view/)
- [eframe示例](https://github.com/emilk/egui/tree/master/crates/eframe)
- [WebView原理](https://github.com/zserge/webview)

## 🤝 贡献

欢迎提交改进建议和新的集成示例！
