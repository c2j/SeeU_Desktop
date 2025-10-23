use anyhow::Result;
use eframe::egui;

pub mod sidebar;
pub mod drag_resize;

#[cfg(target_os = "macos")]
pub mod macos_sidebar;

pub use sidebar::SidebarWindow;

#[cfg(target_os = "macos")]
pub use macos_sidebar::{MacOSSidebarApp, create_macos_sidebar_window};

/// 侧边栏位置
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SidebarPosition {
    Left,
    Right,
    Top,
    Bottom,
}

impl SidebarPosition {
    /// 获取贴边阈值（像素）
    pub fn snap_threshold() -> f32 {
        20.0
    }
    
    /// 根据窗口位置确定应该贴边的位置
    pub fn from_window_position(pos: egui::Pos2, screen_size: egui::Vec2) -> Option<Self> {
        let threshold = Self::snap_threshold();
        
        // 检查是否靠近屏幕边缘
        if pos.x <= threshold {
            Some(SidebarPosition::Left)
        } else if pos.x >= screen_size.x - threshold {
            Some(SidebarPosition::Right)
        } else if pos.y <= threshold {
            Some(SidebarPosition::Top)
        } else if pos.y >= screen_size.y - threshold {
            Some(SidebarPosition::Bottom)
        } else {
            None
        }
    }
    
    /// 获取贴边后的位置
    pub fn get_snapped_position(&self, screen_size: egui::Vec2, window_size: egui::Vec2) -> egui::Pos2 {
        match self {
            SidebarPosition::Left => egui::Pos2::new(0.0, (screen_size.y - window_size.y) / 2.0),
            SidebarPosition::Right => egui::Pos2::new(screen_size.x - window_size.x, (screen_size.y - window_size.y) / 2.0),
            SidebarPosition::Top => egui::Pos2::new((screen_size.x - window_size.x) / 2.0, 0.0),
            SidebarPosition::Bottom => egui::Pos2::new((screen_size.x - window_size.x) / 2.0, screen_size.y - window_size.y),
        }
    }
    
    /// 获取隐藏位置（部分隐藏，只露出一小部分）
    pub fn get_hidden_position(&self, screen_size: egui::Vec2, window_size: egui::Vec2, visible_width: f32) -> egui::Pos2 {
        match self {
            SidebarPosition::Left => egui::Pos2::new(-window_size.x + visible_width, (screen_size.y - window_size.y) / 2.0),
            SidebarPosition::Right => egui::Pos2::new(screen_size.x - visible_width, (screen_size.y - window_size.y) / 2.0),
            SidebarPosition::Top => egui::Pos2::new((screen_size.x - window_size.x) / 2.0, -window_size.y + visible_width),
            SidebarPosition::Bottom => egui::Pos2::new((screen_size.x - window_size.x) / 2.0, screen_size.y - visible_width),
        }
    }
}

/// 侧边栏状态
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SidebarState {
    /// 隐藏状态
    Hidden,
    /// 显示状态
    Visible,
    /// 正在显示动画
    Showing,
    /// 正在隐藏动画
    Hiding,
}

/// 侧边栏配置
#[derive(Debug, Clone)]
pub struct SidebarConfig {
    /// 默认宽度
    pub default_width: f32,
    /// 默认高度
    pub default_height: f32,
    /// 最小宽度
    pub min_width: f32,
    /// 最小高度
    pub min_height: f32,
    /// 最大宽度
    pub max_width: f32,
    /// 最大高度
    pub max_height: f32,
    /// 自动隐藏延迟（毫秒）
    pub auto_hide_delay: u64,
    /// 隐藏时可见的宽度
    pub visible_width_when_hidden: f32,
    /// 动画持续时间（毫秒）
    pub animation_duration: u64,
    /// 是否启用自动隐藏
    pub auto_hide_enabled: bool,
    /// 是否启用贴边吸附
    pub snap_to_edge_enabled: bool,
}

impl Default for SidebarConfig {
    fn default() -> Self {
        Self {
            default_width: 300.0,
            default_height: 600.0,
            min_width: 200.0,
            min_height: 400.0,
            max_width: 800.0,
            max_height: 1200.0,
            auto_hide_delay: 1000, // 1秒
            visible_width_when_hidden: 5.0,
            animation_duration: 200, // 200毫秒
            auto_hide_enabled: true,
            snap_to_edge_enabled: true,
        }
    }
}

/// 侧边栏事件
#[derive(Debug, Clone)]
pub enum SidebarEvent {
    /// 窗口被拖拽
    Dragged(egui::Vec2),
    /// 窗口大小改变
    Resized(egui::Vec2),
    /// 鼠标进入
    MouseEnter,
    /// 鼠标离开
    MouseLeave,
    /// 请求显示
    Show,
    /// 请求隐藏
    Hide,
    /// 请求关闭
    Close,
}

/// 创建侧边栏窗口
pub fn create_sidebar_window(config: SidebarConfig) -> Result<SidebarWindow> {
    SidebarWindow::new(config)
}

/// 检测鼠标是否在屏幕边缘附近
pub fn is_mouse_near_edge(mouse_pos: egui::Pos2, screen_size: egui::Vec2, threshold: f32) -> Option<SidebarPosition> {
    if mouse_pos.x <= threshold {
        Some(SidebarPosition::Left)
    } else if mouse_pos.x >= screen_size.x - threshold {
        Some(SidebarPosition::Right)
    } else if mouse_pos.y <= threshold {
        Some(SidebarPosition::Top)
    } else if mouse_pos.y >= screen_size.y - threshold {
        Some(SidebarPosition::Bottom)
    } else {
        None
    }
}
