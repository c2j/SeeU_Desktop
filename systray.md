跨平台 egui 桌面应用  
需求设计书（Windows 10/11、Ubuntu 20+/UOS/Kylin、macOS 10.15+）

────────────────────  
1. 目标总览  
A. 常驻系统托盘（systray）  
B. 主窗口可「最小化 → 侧边栏」；侧边栏可拖拽到屏幕任意边缘并贴边自动隐藏  
C. 在「任意应用 / 桌面空白处」的右键菜单中插入自定义条目（全局上下文菜单钩子）  

────────────────────  
2. 技术栈与架构  
• UI 框架：egui（Rust）+ eframe（winit + glow）  
• 托盘/全局快捷键/自动启动：  
  – Windows：windows-rs  
  – Linux：libayatana-appindicator3 / libappindicator3（GTK3 兼容 UOS/Kylin）  
  – macOS：cocoa + objc2  
• 全局右键菜单：  
  – Windows：COM DLL Shell Extension（rust-com）  
  – Linux：分别给 Nautilus（Ubuntu）、Peony（UOS/Kylin）、Dolphin 写 .desktop 脚本  
  – macOS：Finder Sync Extension（Swift）通过 IPC 与 Rust 通信  
• IPC：tokio + tauri-plugin-ipc（或 unix-socket / named-pipe / XPC）  
• 打包：cargo-bundle（macOS）、cargo-deb（Linux）、cargo-wix（Windows）

────────────────────  
3. 目录结构  
```
src/
├─ main.rs            // 入口，托盘、窗口管理
├─ tray/              // 跨平台托盘实现
│  ├─ tray_windows.rs
│  ├─ tray_linux.rs
│  └─ tray_macos.rs
├─ sidebar/           // 侧边栏窗口
│  ├─ sidebar.rs
│  └─ drag_resize.rs
├─ global_menu/       // 全局右键菜单
│  ├─ win_ext/        // COM DLL
│  ├─ linux_scripts/
│  └─ macos_finder/
└─ ...                // 其余  代码
```

────────────────────  
4. 模块设计详述  
4.1 托盘（tray）  
• 统一 Trait  
```rust
trait Tray {
    fn new(icon: &[u8], menu: Vec<MenuItem>) -> Result<Self>;
    fn set_tooltip(&mut self, text: &str);
    fn shutdown(self);
}
```  
• 依赖  
  – Windows：`windows-rs` + `winapi`  
  – Linux：`ksni`（KDE）/ `libayatana-appindicator3-sys`（GTK）  
  – macOS：`tray-item` crate 或 `objc` 手动调用 NSStatusItem  

4.2 侧边栏（sidebar）  
• 窗口属性  
  – 无边框：`window.set_decorations(false)`  
  – 置顶：`window.set_always_on_top(true)`  
  – 透明：`window.set_transparent(true)`  
• 贴边逻辑  
  – 监听 `Moved` 事件，计算离屏幕边缘 < 20 px 时吸附  
  – 自动隐藏：鼠标靠近时 `set_visible(true)`，离开时 `set_visible(false)`  
• 拖拽实现  
  – egui 内使用 `response.drag_delta()` 计算偏移  
  – winit 层通过 `Window::set_outer_position` 更新坐标  

4.3 全局右键菜单（global_menu）  
• Windows  
  – COM DLL 导出 `IContextMenu`  
  – 注册表路径：  
    ```
    HKCR\Directory\Background\shellex\ContextMenuHandlers\{AppID}
    HKCR\*\shellex\ContextMenuHandlers\{AppID}
    ```  
  – DLL → Rust IPC：命名管道 `\\.\pipe\MyAppCtx`  
• Linux  
  – Ubuntu：放入 `~/.local/share/nautilus-python/extensions/myapp.py`  
  – UOS/Kylin：放入 `/usr/share/peony-python-extensions/myapp.py`  
  – 脚本通过 `glib.spawn_async()` 调 Rust 二进制传递文件路径  
• macOS  
  – Xcode 子项目 `FinderSync`（Swift）  
  – 使用 `NSXPCConnection` 与 Rust helper 通信  

4.4 生命周期与单实例  
• `single-instance` crate 保证仅运行一份  
• 托盘双击 → 若主窗口隐藏则显示；若已显示则最小化到侧边栏  

────────────────────  
5. 构建脚本示例  
```bash
# 编译 COM DLL（Windows）
cargo build -p win_ext --release
# 打包 deb
cargo deb --output target/myapp.deb
# 打包 dmg
cargo bundle --release --format osx
```

────────────────────  
6. 平台差异与限制  
| 功能点        | Windows | Linux (GTK3) | macOS |
|---------------|---------|--------------|--------|
| 托盘          | ✅      | ✅            | ✅     |
| 侧边栏拖拽    | ✅      | ✅            | ✅     |
| 全局右键菜单  | ✅ COM  | ✅ 脚本       | ✅ FinderSync |
| 安装权限      | 管理员  | 用户/系统    | 用户/系统 |
| 沙箱限制      | 无      | 无            | App Sandboxing 需关闭 |

────────────────────  
7. 开发里程碑  
M1：托盘 + 基础窗口（3 d）  
M2：侧边栏吸附 / 隐藏（5 d）  
M3：Windows COM DLL 原型（5 d）  
M4：Linux Nautilus & Peony 脚本（3 d）  
M5：macOS FinderSync 扩展（4 d）  
M6：CI/CD 打包 & 自动更新（3 d）  
M7：文档与测试（2 d）

────────────────────  
8. 风险与回退方案  
• 若 COM DLL 审核不通过 → 降级为“发送到…”菜单或后台监视剪贴板  
• UOS/Kylin 老旧 GTK 版本 → 静态链接 GTK3 或使用 Qt 托盘 fallback  
• macOS 签名失败 → 不提交 Mac App Store，仅 .dmg 发布  

────────────────────  
9. 示例代码片段  
托盘初始化（Windows）：
```rust
use windows::{Win32::UI::Shell::*, Win32::Foundation::*};
fn create_tray(hwnd: HWND) {
    let nid = NOTIFYICONDATAW {
        hWnd: hwnd,
        uID: 1,
        uFlags: NIF_MESSAGE | NIF_ICON | NIF_TIP,
        uCallbackMessage: WM_APP + 1,
        hIcon: load_icon(),
        szTip: w!("MyApp"),
        ..Default::default()
    };
    unsafe { Shell_NotifyIconW(NIM_ADD, &nid) };
}
```

egui 侧边栏拖拽：
```rust
if let Some(p) = sidebar.response.drag_delta() {
    let new_pos = window.outer_position().unwrap() + p;
    window.set_outer_position(new_pos);
}
```

────────────────────  
10. 结论  
通过「egui + 平台特定 crate + 小量原生扩展」的组合，可在三平台同时实现托盘、可拖拽侧边栏及全局右键菜单。开发顺序先完成托盘与侧边栏，再并行攻克各平台右键菜单即可快速落地。