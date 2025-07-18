# egui + WebView Integration Troubleshooting

## 常见问题和解决方案

### 1. Trace Trap 错误

**问题**: 运行示例时出现 `zsh: trace trap` 错误

**原因**: 
- 线程安全问题：在多线程环境中使用WebView可能导致内存访问冲突
- 平台特定问题：某些平台上的WebView实现对线程有严格要求
- 资源竞争：多个WebView实例同时创建可能导致资源冲突

**解决方案**:

#### 方案1: 使用安全示例 (推荐)
```bash
cargo run -p webview-examples --example egui_webview_safe
```

这个示例使用系统浏览器命令而不是嵌入式WebView，避免了线程安全问题。

#### 方案2: 使用基础示例
```bash
cargo run -p webview-examples --example egui_embedded_webview
```

基础示例使用更简单的单窗口模式，减少了出错的可能性。

#### 方案3: 平台特定解决方案

**macOS**:
```bash
# 确保在主线程运行
export RUST_BACKTRACE=1
cargo run -p webview-examples --example egui_webview_safe
```

**Linux**:
```bash
# 确保有正确的显示环境
export DISPLAY=:0
cargo run -p webview-examples --example egui_webview_safe
```

**Windows**:
```bash
# 使用Edge WebView2 (如果可用)
cargo run -p webview-examples --example egui_webview_safe --features edge
```

### 2. 编译错误

**问题**: 缺少依赖或版本冲突

**解决方案**:
```bash
# 清理并重新构建
cargo clean
cargo build -p webview-examples

# 更新依赖
cargo update
```

### 3. WebView 不显示

**问题**: WebView窗口没有出现

**可能原因和解决方案**:

1. **权限问题** (macOS):
   ```bash
   # 给予应用网络访问权限
   # 在系统偏好设置 > 安全性与隐私 > 隐私 > 网络访问 中添加应用
   ```

2. **防火墙阻止** (所有平台):
   - 检查防火墙设置
   - 临时禁用防火墙测试

3. **URL格式错误**:
   - 确保URL包含协议 (http:// 或 https://)
   - 检查URL是否可访问

### 4. 性能问题

**问题**: 应用响应缓慢或卡顿

**解决方案**:

1. **减少重绘频率**:
   ```rust
   // 在egui update方法中
   ctx.request_repaint_after(std::time::Duration::from_millis(100));
   ```

2. **限制并发WebView数量**:
   ```rust
   const MAX_WEBVIEWS: usize = 5;
   ```

3. **使用系统浏览器** (推荐):
   使用 `egui_webview_safe.rs` 示例

### 5. 内存泄漏

**问题**: 长时间运行后内存使用持续增长

**解决方案**:

1. **正确清理资源**:
   ```rust
   impl Drop for WebViewManager {
       fn drop(&mut self) {
           self.close_all_windows();
       }
   }
   ```

2. **限制历史记录**:
   ```rust
   if self.status_messages.len() > 10 {
       self.status_messages.remove(0);
   }
   ```

### 6. 跨平台兼容性

**问题**: 在某些平台上无法运行

**解决方案**:

1. **检查平台特定依赖**:
   - Linux: 确保安装了 `webkit2gtk-4.0-dev`
   - Windows: 确保有 WebView2 运行时
   - macOS: 系统自带 WebKit

2. **使用条件编译**:
   ```rust
   #[cfg(target_os = "macos")]
   fn open_url_macos(url: &str) {
       std::process::Command::new("open").arg(url).spawn().ok();
   }
   ```

## 最佳实践

### 1. 生产环境建议

- ✅ 使用 `egui_webview_safe.rs` 作为参考
- ✅ 使用系统浏览器命令而不是嵌入式WebView
- ✅ 实现适当的错误处理
- ✅ 限制并发窗口数量
- ✅ 定期清理资源

### 2. 开发环境建议

- ✅ 启用详细日志: `RUST_LOG=debug`
- ✅ 使用 `RUST_BACKTRACE=1` 获取详细错误信息
- ✅ 定期测试不同平台
- ✅ 监控内存使用情况

### 3. 调试技巧

1. **启用详细日志**:
   ```bash
   RUST_LOG=debug cargo run -p webview-examples --example egui_webview_safe
   ```

2. **使用调试器**:
   ```bash
   rust-gdb target/debug/examples/egui_webview_safe
   ```

3. **内存分析**:
   ```bash
   valgrind --tool=memcheck target/debug/examples/egui_webview_safe
   ```

## 获取帮助

如果问题仍然存在：

1. 检查 [egui GitHub Issues](https://github.com/emilk/egui/issues)
2. 检查 [web-view GitHub Issues](https://github.com/Boscop/web-view/issues)
3. 在项目仓库中创建新的 Issue，包含：
   - 操作系统和版本
   - Rust 版本 (`rustc --version`)
   - 完整的错误信息
   - 重现步骤

## 相关资源

- [egui 官方文档](https://docs.rs/egui/)
- [web-view 官方文档](https://docs.rs/web-view/)
- [WebView 平台支持](https://github.com/zserge/webview#platform-support)
