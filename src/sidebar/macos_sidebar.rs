use anyhow::Result;
use eframe::egui;
use std::time::{Duration, Instant};

use super::{SidebarConfig, SidebarState, SidebarPosition, SidebarEvent};

/// macOS 专用侧边栏实现
/// 使用独立窗口实现真正的侧边栏效果
pub struct MacOSSidebarApp {
    /// 配置
    config: SidebarConfig,
    /// 当前状态
    state: SidebarState,
    /// 当前位置
    position: Option<SidebarPosition>,
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
    /// 内容
    content: String,
    /// 是否应该关闭
    should_close: bool,
}

impl MacOSSidebarApp {
    /// 创建新的 macOS 侧边栏应用
    pub fn new(config: SidebarConfig) -> Self {
        Self {
            config,
            state: SidebarState::Visible,
            position: Some(SidebarPosition::Right), // 默认在右侧
            is_dragging: false,
            drag_start_pos: None,
            last_mouse_activity: Instant::now(),
            animation_start: None,
            animation_start_pos: None,
            animation_target_pos: None,
            content: "SeeU Desktop 侧边栏\n\n这是一个 macOS 风格的侧边栏。\n\n功能特性：\n• 贴边吸附\n• 自动隐藏\n• 鼠标悬停显示\n• 拖拽移动\n\n您可以：\n- 拖拽移动到屏幕任意边缘\n- 鼠标离开后自动隐藏\n- 鼠标靠近时自动显示".to_string(),
            should_close: false,
        }
    }
    
    /// 检查是否应该关闭
    pub fn should_close(&self) -> bool {
        self.should_close
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
                self.state = SidebarState::Visible;
            }
            SidebarEvent::Hide => {
                self.start_hide_animation();
            }
            SidebarEvent::Close => {
                self.should_close = true;
            }
            _ => {}
        }
    }
    
    /// 开始隐藏动画
    fn start_hide_animation(&mut self) {
        if let Some(position) = self.position {
            // 获取屏幕大小（这里使用估计值，实际应该从系统获取）
            let screen_size = egui::Vec2::new(1920.0, 1080.0);
            let window_size = egui::Vec2::new(self.config.default_width, self.config.default_height);
            let target_pos = position.get_hidden_position(
                screen_size, 
                window_size, 
                self.config.visible_width_when_hidden
            );
            self.start_animation(target_pos);
            self.state = SidebarState::Hiding;
        }
    }
    
    /// 开始显示动画
    fn start_show_animation(&mut self) {
        if let Some(position) = self.position {
            let screen_size = egui::Vec2::new(1920.0, 1080.0);
            let window_size = egui::Vec2::new(self.config.default_width, self.config.default_height);
            let target_pos = position.get_snapped_position(screen_size, window_size);
            self.start_animation(target_pos);
            self.state = SidebarState::Showing;
        }
    }
    
    /// 开始动画
    fn start_animation(&mut self, target_pos: egui::Pos2) {
        self.animation_start = Some(Instant::now());
        self.animation_start_pos = Some(egui::Pos2::ZERO); // 当前位置由 viewport 管理
        self.animation_target_pos = Some(target_pos);
    }
    
    /// 更新动画
    fn update_animation(&mut self, ctx: &egui::Context) {
        if let (Some(start_time), Some(_start_pos), Some(target_pos)) = 
            (self.animation_start, self.animation_start_pos, self.animation_target_pos) {
            
            let elapsed = start_time.elapsed();
            let duration = Duration::from_millis(self.config.animation_duration);
            
            if elapsed >= duration {
                // 动画完成
                ctx.send_viewport_cmd(egui::ViewportCommand::OuterPosition(target_pos));
                self.animation_start = None;
                self.animation_start_pos = None;
                self.animation_target_pos = None;
            } else {
                // 计算当前位置
                let progress = elapsed.as_millis() as f32 / duration.as_millis() as f32;
                let eased_progress = ease_in_out_cubic(progress);
                
                // 获取当前位置
                if let Some(current_pos) = ctx.input(|i| i.viewport().outer_rect.map(|r| r.min)) {
                    let new_pos = current_pos + (target_pos - current_pos) * eased_progress;
                    ctx.send_viewport_cmd(egui::ViewportCommand::OuterPosition(new_pos));
                }
            }
        }
    }
    
    /// 更新自动隐藏逻辑
    fn update_auto_hide(&mut self) {
        if self.config.auto_hide_enabled && self.state == SidebarState::Visible && self.position.is_some() {
            let idle_time = self.last_mouse_activity.elapsed();
            if idle_time > Duration::from_millis(self.config.auto_hide_delay) {
                self.start_hide_animation();
            }
        }
    }
    
    /// 检查是否正在动画
    fn is_animating(&self) -> bool {
        self.animation_start.is_some()
    }
}

impl eframe::App for MacOSSidebarApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        // 更新动画
        self.update_animation(ctx);
        
        // 更新自动隐藏
        self.update_auto_hide();
        
        // 请求重绘以保持动画流畅
        if self.is_animating() {
            ctx.request_repaint();
        }
        
        // 设置窗口属性
        ctx.send_viewport_cmd(egui::ViewportCommand::Decorations(false)); // 无边框
        ctx.send_viewport_cmd(egui::ViewportCommand::WindowLevel(egui::WindowLevel::AlwaysOnTop)); // 置顶
        
        // 主界面
        egui::CentralPanel::default()
            .frame(egui::Frame {
                fill: ctx.style().visuals.window_fill,
                stroke: egui::Stroke::new(1.0, ctx.style().visuals.window_stroke.color),
                rounding: egui::Rounding::same(12.0),
                shadow: egui::Shadow::NONE,
                inner_margin: egui::Margin::same(16.0),
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
                        // 移动窗口
                        if let Some(current_pos) = ctx.input(|i| i.viewport().outer_rect.map(|r| r.min)) {
                            let new_pos = current_pos + delta;
                            ctx.send_viewport_cmd(egui::ViewportCommand::OuterPosition(new_pos));
                        }
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
                }
                
                // 渲染内容
                ui.vertical(|ui| {
                    ui.heading("🚀 SeeU Desktop");
                    ui.separator();
                    
                    ui.label("📊 状态信息：");
                    ui.label(format!("位置: {:?}", self.position));
                    ui.label(format!("状态: {:?}", self.state));
                    
                    ui.separator();
                    
                    // 控制按钮
                    ui.horizontal(|ui| {
                        if ui.button("🔽 隐藏").clicked() {
                            self.handle_event(SidebarEvent::Hide);
                        }
                        if ui.button("❌ 关闭").clicked() {
                            self.handle_event(SidebarEvent::Close);
                        }
                    });
                    
                    ui.separator();
                    
                    // 内容区域
                    egui::ScrollArea::vertical()
                        .max_height(400.0)
                        .show(ui, |ui| {
                            ui.text_edit_multiline(&mut self.content);
                        });
                });
            });
    }
    
    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        log::info!("macOS sidebar window closed");
    }
}

impl MacOSSidebarApp {
    /// 检查是否需要贴边吸附
    fn check_snap_to_edge(&mut self, ctx: &egui::Context) {
        if let Some(current_rect) = ctx.input(|i| i.viewport().outer_rect) {
            let screen_size = egui::Vec2::new(1920.0, 1080.0); // TODO: 获取实际屏幕大小
            
            if let Some(snap_position) = SidebarPosition::from_window_position(current_rect.min, screen_size) {
                let target_pos = snap_position.get_snapped_position(screen_size, current_rect.size());
                self.start_animation(target_pos);
                self.position = Some(snap_position);
            }
        }
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

/// 创建 macOS 侧边栏窗口
pub fn create_macos_sidebar_window(_config: SidebarConfig) -> Result<()> {
    // 在 macOS 下，我们暂时使用日志记录，实际的独立窗口实现需要更复杂的设置
    log::info!("macOS sidebar window creation requested - using integrated sidebar for now");

    // TODO: 实现真正的独立窗口侧边栏
    // 这需要使用 macOS 特定的 API 或者更复杂的多窗口管理

    Ok(())
}
