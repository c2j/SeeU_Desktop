use anyhow::Result;
use eframe::egui;
use std::time::{Duration, Instant};

use super::{SidebarConfig, SidebarState, SidebarPosition, SidebarEvent};

/// 侧边栏窗口实现
pub struct SidebarWindow {
    /// 配置
    config: SidebarConfig,
    /// 当前状态
    state: SidebarState,
    /// 当前位置
    position: Option<SidebarPosition>,
    /// 窗口位置
    window_pos: egui::Pos2,
    /// 窗口大小
    window_size: egui::Vec2,
    /// 是否正在拖拽
    is_dragging: bool,
    /// 拖拽开始位置
    drag_start_pos: Option<egui::Pos2>,
    /// 最后鼠标活动时间
    last_mouse_activity: Instant,
    /// 动画开始时间
    animation_start: Option<Instant>,
    /// 动画起始位置
    animation_start_pos: Option<egui::Pos2>,
    /// 动画目标位置
    animation_target_pos: Option<egui::Pos2>,
    /// 是否可见
    visible: bool,
    /// 内容
    content: String,
}

impl SidebarWindow {
    /// 创建新的侧边栏窗口
    pub fn new(config: SidebarConfig) -> Result<Self> {
        Ok(Self {
            window_pos: egui::Pos2::new(100.0, 100.0),
            window_size: egui::Vec2::new(config.default_width, config.default_height),
            config,
            state: SidebarState::Visible,
            position: None,
            is_dragging: false,
            drag_start_pos: None,
            last_mouse_activity: Instant::now(),
            animation_start: None,
            animation_start_pos: None,
            animation_target_pos: None,
            visible: true,
            content: "侧边栏内容\n\n这是一个可拖拽的侧边栏窗口。\n\n功能特性：\n• 拖拽移动\n• 贴边吸附\n• 自动隐藏\n• 鼠标悬停显示".to_string(),
        })
    }
    
    /// 更新侧边栏状态
    pub fn update(&mut self, ctx: &egui::Context) {
        // 处理动画
        self.update_animation();
        
        // 处理自动隐藏
        if self.config.auto_hide_enabled {
            self.update_auto_hide();
        }
        
        // 请求重绘以保持动画流畅
        if self.is_animating() {
            ctx.request_repaint();
        }
    }
    
    /// 渲染侧边栏
    pub fn render(&mut self, ctx: &egui::Context) -> Option<SidebarEvent> {
        if !self.visible {
            return None;
        }

        let mut event = None;

        // 创建无边框、置顶的侧边栏窗口
        let window_response = egui::Window::new("SeeU 侧边栏")
            .id(egui::Id::new("sidebar_window"))
            .title_bar(false)  // 无标题栏
            .resizable(false)  // 禁用调整大小（我们自己处理）
            .collapsible(false)
            .scroll([false, true])  // 只允许垂直滚动
            .fixed_pos(self.window_pos)  // 使用固定位置
            .fixed_size(self.window_size)  // 使用固定大小
            .frame(egui::Frame {
                fill: ctx.style().visuals.window_fill,
                stroke: egui::Stroke::new(1.0, ctx.style().visuals.window_stroke.color),
                rounding: egui::Rounding::same(8.0),  // 圆角
                shadow: egui::Shadow::NONE,  // 阴影效果
                inner_margin: egui::Margin::same(8.0),
                outer_margin: egui::Margin::ZERO,
            })
            .show(ctx, |ui| {
                // 处理拖拽
                let response = ui.allocate_response(ui.available_size(), egui::Sense::drag());
                
                if response.drag_started() {
                    self.is_dragging = true;
                    self.drag_start_pos = Some(response.interact_pointer_pos().unwrap_or_default());
                }
                
                if response.dragged() && self.is_dragging {
                    let delta = response.drag_delta();
                    if delta != egui::Vec2::ZERO {
                        self.window_pos += delta;
                        event = Some(SidebarEvent::Dragged(delta));
                    }
                }
                
                if response.drag_stopped() {
                    self.is_dragging = false;
                    self.drag_start_pos = None;
                    
                    // 检查是否需要贴边
                    if self.config.snap_to_edge_enabled {
                        self.check_snap_to_edge(ctx);
                    }
                }
                
                // 处理鼠标进入/离开
                if response.hovered() {
                    self.last_mouse_activity = Instant::now();
                    if self.state == SidebarState::Hidden {
                        event = Some(SidebarEvent::MouseEnter);
                    }
                } else if self.state == SidebarState::Visible {
                    // 检查是否应该开始隐藏倒计时
                    if self.last_mouse_activity.elapsed() > Duration::from_millis(100) {
                        // 给一点缓冲时间
                    }
                }
                
                // 渲染内容
                ui.vertical(|ui| {
                    ui.heading("SeeU Desktop 侧边栏");
                    ui.separator();
                    
                    ui.label("状态信息：");
                    ui.label(format!("位置: {:?}", self.position));
                    ui.label(format!("状态: {:?}", self.state));
                    ui.label(format!("窗口位置: {:.1}, {:.1}", self.window_pos.x, self.window_pos.y));
                    ui.label(format!("窗口大小: {:.1} x {:.1}", self.window_size.x, self.window_size.y));
                    
                    ui.separator();
                    
                    // 控制按钮
                    ui.horizontal(|ui| {
                        if ui.button("隐藏").clicked() {
                            event = Some(SidebarEvent::Hide);
                        }
                        if ui.button("关闭").clicked() {
                            event = Some(SidebarEvent::Close);
                        }
                    });
                    
                    ui.separator();
                    
                    // 内容区域
                    ui.text_edit_multiline(&mut self.content);
                });
            });
        
        // 更新窗口位置和大小
        if let Some(response) = window_response {
            let rect = response.response.rect;
            self.window_pos = rect.min;
            self.window_size = rect.size();
        }
        
        event
    }
    
    /// 检查是否需要贴边吸附
    fn check_snap_to_edge(&mut self, ctx: &egui::Context) {
        let screen_size = ctx.screen_rect().size();
        
        if let Some(snap_position) = SidebarPosition::from_window_position(self.window_pos, screen_size) {
            let target_pos = snap_position.get_snapped_position(screen_size, self.window_size);
            self.start_animation(target_pos);
            self.position = Some(snap_position);
        }
    }
    
    /// 开始动画
    fn start_animation(&mut self, target_pos: egui::Pos2) {
        self.animation_start = Some(Instant::now());
        self.animation_start_pos = Some(self.window_pos);
        self.animation_target_pos = Some(target_pos);
    }
    
    /// 更新动画
    fn update_animation(&mut self) {
        if let (Some(start_time), Some(start_pos), Some(target_pos)) = 
            (self.animation_start, self.animation_start_pos, self.animation_target_pos) {
            
            let elapsed = start_time.elapsed();
            let duration = Duration::from_millis(self.config.animation_duration);
            
            if elapsed >= duration {
                // 动画完成
                self.window_pos = target_pos;
                self.animation_start = None;
                self.animation_start_pos = None;
                self.animation_target_pos = None;
            } else {
                // 计算当前位置
                let progress = elapsed.as_millis() as f32 / duration.as_millis() as f32;
                let eased_progress = ease_in_out_cubic(progress);
                
                self.window_pos = start_pos + (target_pos - start_pos) * eased_progress;
            }
        }
    }
    
    /// 更新自动隐藏逻辑
    fn update_auto_hide(&mut self) {
        if self.state == SidebarState::Visible && self.position.is_some() {
            let idle_time = self.last_mouse_activity.elapsed();
            if idle_time > Duration::from_millis(self.config.auto_hide_delay) {
                self.start_hide_animation();
            }
        }
    }
    
    /// 开始隐藏动画
    fn start_hide_animation(&mut self) {
        if let Some(position) = self.position {
            let screen_size = egui::Vec2::new(1920.0, 1080.0); // TODO: 获取实际屏幕大小
            let target_pos = position.get_hidden_position(
                screen_size, 
                self.window_size, 
                self.config.visible_width_when_hidden
            );
            self.start_animation(target_pos);
            self.state = SidebarState::Hiding;
        }
    }
    
    /// 开始显示动画
    fn start_show_animation(&mut self) {
        if let Some(position) = self.position {
            let screen_size = egui::Vec2::new(1920.0, 1080.0); // TODO: 获取实际屏幕大小
            let target_pos = position.get_snapped_position(screen_size, self.window_size);
            self.start_animation(target_pos);
            self.state = SidebarState::Showing;
        }
    }
    
    /// 检查是否正在动画
    fn is_animating(&self) -> bool {
        self.animation_start.is_some()
    }
    
    /// 处理事件
    pub fn handle_event(&mut self, event: SidebarEvent) {
        match event {
            SidebarEvent::MouseEnter => {
                self.last_mouse_activity = Instant::now();
                if self.state == SidebarState::Hidden {
                    self.start_show_animation();
                }
            }
            SidebarEvent::MouseLeave => {
                // 开始隐藏倒计时
            }
            SidebarEvent::Show => {
                self.visible = true;
                self.state = SidebarState::Visible;
            }
            SidebarEvent::Hide => {
                self.start_hide_animation();
            }
            SidebarEvent::Close => {
                self.visible = false;
            }
            _ => {}
        }
    }
    
    /// 获取是否可见
    pub fn is_visible(&self) -> bool {
        self.visible
    }
    
    /// 设置可见性
    pub fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }
}

/// 缓动函数：三次方缓入缓出
fn ease_in_out_cubic(t: f32) -> f32 {
    if t < 0.5 {
        4.0 * t * t * t
    } else {
        1.0 - (-2.0 * t + 2.0).powi(3) / 2.0
    }
}
