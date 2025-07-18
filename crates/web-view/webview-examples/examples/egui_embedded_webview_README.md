# egui + WebView Integration Example

这个例子演示了如何将 `web-view` crate 与 `egui 0.28.1` 框架集成，创建一个混合应用程序，结合了原生GUI控件和Web内容。

## 功能特性

- ✅ **egui原生界面**: 使用egui创建的现代化原生GUI界面
- ✅ **嵌入式WebView**: 在独立窗口中显示Web内容
- ✅ **URL导航控制**: 地址栏输入和快速链接按钮
- ✅ **窗口管理**: 实时状态显示和窗口控制
- ✅ **跨平台兼容**: 支持macOS、Linux和Windows
- ✅ **egui 0.28.1兼容**: 完全兼容最新的egui版本

## 运行示例

```bash
# 在项目根目录运行
cargo run -p webview-examples --example egui_embedded_webview
```

## 架构说明

### 技术栈
- **egui**: 即时模式GUI框架，用于主界面
- **web-view**: 原生WebView绑定，用于显示Web内容
- **线程模型**: WebView在独立线程中运行
- **状态管理**: 通过 `Arc<Mutex<T>>` 共享状态

### 平台后端
- **macOS**: Cocoa + WebKit
- **Linux**: GTK + WebKit2  
- **Windows**: Win32 + MSHTML/Edge

## 代码结构

```rust
// 主应用状态
struct EguiWebViewApp {
    url_input: String,                              // 地址栏输入
    status_message: String,                         // 状态消息
    webview_manager: Arc<Mutex<WebViewManager>>,    // WebView管理器
    webview_open: bool,                             // WebView状态
}

// WebView窗口管理器
struct WebViewManager {
    current_url: String,                            // 当前URL
    is_running: bool,                               // 运行状态
    thread_handle: Option<thread::JoinHandle<()>>,  // 线程句柄
}
```

## 使用方法

1. **启动应用**: 运行命令后会打开egui主窗口
2. **输入URL**: 在地址栏输入网址（支持自动添加https://）
3. **快速导航**: 点击预设的快速链接按钮
4. **打开WebView**: 点击"🔗 Open in WebView"按钮
5. **状态监控**: 实时查看WebView的运行状态

## 兼容性验证

### ✅ 依赖兼容性
- egui 0.28.1: 无GTK依赖，纯Rust实现
- web-view 0.7.3: 使用系统原生WebView
- 无版本冲突或ABI不兼容问题

### ✅ 运行时兼容性
- 两个库在不同层面工作，无直接冲突
- egui处理GUI渲染，web-view处理Web内容
- 通过线程隔离确保稳定性

## 扩展可能性

这个例子展示了基础的集成模式，可以进一步扩展：

1. **双向通信**: 实现egui与WebView之间的消息传递
2. **多窗口支持**: 同时管理多个WebView窗口
3. **内容嵌入**: 尝试将WebView内容直接嵌入到egui界面中
4. **状态持久化**: 保存和恢复浏览历史和设置
5. **高级控制**: 添加前进/后退、刷新、开发者工具等功能

## 注意事项

- WebView在独立线程中运行，避免阻塞主UI
- 状态同步通过Arc<Mutex<T>>实现，注意避免死锁
- 不同平台的WebView行为可能略有差异
- 确保URL格式正确，程序会自动添加https://前缀

## 相关文件

- `egui_embedded_webview.rs`: 主要示例代码
- `Cargo.toml`: 依赖配置（已添加egui和eframe）
- 其他examples: 查看更多web-view使用示例
