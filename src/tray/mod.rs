use anyhow::Result;

/// 托盘菜单项
#[derive(Debug, Clone)]
pub struct MenuItem {
    pub id: String,
    pub label: String,
    pub enabled: bool,
    pub separator: bool,
}

impl MenuItem {
    pub fn new(id: &str, label: &str) -> Self {
        Self {
            id: id.to_string(),
            label: label.to_string(),
            enabled: true,
            separator: false,
        }
    }

    pub fn separator() -> Self {
        Self {
            id: String::new(),
            label: String::new(),
            enabled: false,
            separator: true,
        }
    }

    pub fn disabled(mut self) -> Self {
        self.enabled = false;
        self
    }
}

/// 托盘事件
#[derive(Debug, Clone)]
pub enum TrayEvent {
    /// 左键单击
    LeftClick,
    /// 右键单击
    RightClick,
    /// 双击
    DoubleClick,
    /// 菜单项被点击
    MenuItemClick(String),
}

/// 跨平台托盘 trait
pub trait Tray {
    /// 创建新的托盘实例
    fn new(icon: &[u8], menu: Vec<MenuItem>) -> Result<Self>
    where
        Self: Sized;

    /// 设置工具提示文本
    fn set_tooltip(&mut self, text: &str) -> Result<()>;

    /// 更新菜单
    fn update_menu(&mut self, menu: Vec<MenuItem>) -> Result<()>;

    /// 设置图标
    fn set_icon(&mut self, icon: &[u8]) -> Result<()>;

    /// 获取下一个事件（非阻塞）
    fn try_recv_event(&mut self) -> Option<TrayEvent>;

    /// 关闭托盘
    fn shutdown(self) -> Result<()>;
}

// 平台特定实现
#[cfg(target_os = "windows")]
mod tray_windows;
#[cfg(target_os = "windows")]
pub use tray_windows::WindowsTray as PlatformTray;

#[cfg(target_os = "linux")]
mod tray_linux;
#[cfg(target_os = "linux")]
pub use tray_linux::LinuxTray as PlatformTray;

#[cfg(target_os = "macos")]
mod tray_macos;
#[cfg(target_os = "macos")]
pub use tray_macos::MacOSTray as PlatformTray;

/// 创建平台特定的托盘实例
pub fn create_tray(icon: &[u8], menu: Vec<MenuItem>) -> Result<PlatformTray> {
    PlatformTray::new(icon, menu)
}

/// 默认托盘菜单
pub fn default_menu() -> Vec<MenuItem> {
    vec![
        MenuItem::new("show", "显示主窗口"),
        MenuItem::new("hide", "隐藏到托盘"),
        MenuItem::separator(),
        MenuItem::new("sidebar", "显示侧边栏"),
        MenuItem::new("settings", "设置"),
        MenuItem::separator(),
        MenuItem::new("quit", "退出"),
    ]
}
