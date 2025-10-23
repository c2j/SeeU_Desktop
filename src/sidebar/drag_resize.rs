use eframe::egui;

/// 拖拽状态
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DragState {
    None,
    Dragging,
    ResizingLeft,
    ResizingRight,
    ResizingTop,
    ResizingBottom,
    ResizingTopLeft,
    ResizingTopRight,
    ResizingBottomLeft,
    ResizingBottomRight,
}

/// 拖拽和调整大小处理器
pub struct DragResizeHandler {
    /// 当前拖拽状态
    pub drag_state: DragState,
    /// 拖拽开始位置
    pub drag_start_pos: Option<egui::Pos2>,
    /// 拖拽开始时的窗口矩形
    pub drag_start_rect: Option<egui::Rect>,
    /// 调整大小的边界宽度
    pub resize_border_width: f32,
    /// 最小窗口大小
    pub min_size: egui::Vec2,
    /// 最大窗口大小
    pub max_size: egui::Vec2,
}

impl Default for DragResizeHandler {
    fn default() -> Self {
        Self {
            drag_state: DragState::None,
            drag_start_pos: None,
            drag_start_rect: None,
            resize_border_width: 8.0,
            min_size: egui::Vec2::new(200.0, 150.0),
            max_size: egui::Vec2::new(800.0, 600.0),
        }
    }
}

impl DragResizeHandler {
    /// 创建新的拖拽调整大小处理器
    pub fn new(min_size: egui::Vec2, max_size: egui::Vec2) -> Self {
        Self {
            min_size,
            max_size,
            ..Default::default()
        }
    }
    
    /// 处理鼠标输入并返回新的窗口矩形
    pub fn handle_input(
        &mut self,
        ui: &mut egui::Ui,
        window_rect: egui::Rect,
        mouse_pos: egui::Pos2,
        mouse_pressed: bool,
        mouse_released: bool,
    ) -> egui::Rect {
        let mut new_rect = window_rect;
        
        // 检测鼠标位置对应的操作类型
        let hover_state = self.detect_hover_state(window_rect, mouse_pos);
        
        // 设置鼠标光标
        self.set_cursor(ui, hover_state);
        
        // 处理鼠标按下
        if mouse_pressed && self.drag_state == DragState::None {
            self.drag_state = hover_state;
            self.drag_start_pos = Some(mouse_pos);
            self.drag_start_rect = Some(window_rect);
        }
        
        // 处理拖拽
        if self.drag_state != DragState::None {
            if let (Some(start_pos), Some(start_rect)) = (self.drag_start_pos, self.drag_start_rect) {
                let delta = mouse_pos - start_pos;
                new_rect = self.apply_drag_delta(start_rect, delta);
            }
        }
        
        // 处理鼠标释放
        if mouse_released {
            self.drag_state = DragState::None;
            self.drag_start_pos = None;
            self.drag_start_rect = None;
        }
        
        new_rect
    }
    
    /// 检测鼠标悬停状态
    fn detect_hover_state(&self, window_rect: egui::Rect, mouse_pos: egui::Pos2) -> DragState {
        let border = self.resize_border_width;
        
        // 检查是否在边界区域
        let left_edge = mouse_pos.x >= window_rect.left() && mouse_pos.x <= window_rect.left() + border;
        let right_edge = mouse_pos.x >= window_rect.right() - border && mouse_pos.x <= window_rect.right();
        let top_edge = mouse_pos.y >= window_rect.top() && mouse_pos.y <= window_rect.top() + border;
        let bottom_edge = mouse_pos.y >= window_rect.bottom() - border && mouse_pos.y <= window_rect.bottom();
        
        // 检查角落
        if top_edge && left_edge {
            DragState::ResizingTopLeft
        } else if top_edge && right_edge {
            DragState::ResizingTopRight
        } else if bottom_edge && left_edge {
            DragState::ResizingBottomLeft
        } else if bottom_edge && right_edge {
            DragState::ResizingBottomRight
        }
        // 检查边缘
        else if left_edge {
            DragState::ResizingLeft
        } else if right_edge {
            DragState::ResizingRight
        } else if top_edge {
            DragState::ResizingTop
        } else if bottom_edge {
            DragState::ResizingBottom
        }
        // 检查是否在窗口内部（拖拽）
        else if window_rect.contains(mouse_pos) {
            DragState::Dragging
        } else {
            DragState::None
        }
    }
    
    /// 设置鼠标光标
    fn set_cursor(&self, ui: &mut egui::Ui, hover_state: DragState) {
        let cursor = match hover_state {
            DragState::ResizingLeft | DragState::ResizingRight => egui::CursorIcon::ResizeHorizontal,
            DragState::ResizingTop | DragState::ResizingBottom => egui::CursorIcon::ResizeVertical,
            DragState::ResizingTopLeft | DragState::ResizingBottomRight => egui::CursorIcon::ResizeNwSe,
            DragState::ResizingTopRight | DragState::ResizingBottomLeft => egui::CursorIcon::ResizeNeSw,
            DragState::Dragging => egui::CursorIcon::Grab,
            DragState::None => egui::CursorIcon::Default,
        };
        
        ui.ctx().set_cursor_icon(cursor);
    }
    
    /// 应用拖拽增量
    fn apply_drag_delta(&self, start_rect: egui::Rect, delta: egui::Vec2) -> egui::Rect {
        let mut new_rect = start_rect;
        
        match self.drag_state {
            DragState::Dragging => {
                // 移动整个窗口
                new_rect = new_rect.translate(delta);
            }
            DragState::ResizingLeft => {
                // 调整左边缘
                let new_left = (start_rect.left() + delta.x).max(start_rect.right() - self.max_size.x);
                let new_left = new_left.min(start_rect.right() - self.min_size.x);
                new_rect.min.x = new_left;
            }
            DragState::ResizingRight => {
                // 调整右边缘
                let new_right = (start_rect.right() + delta.x).min(start_rect.left() + self.max_size.x);
                let new_right = new_right.max(start_rect.left() + self.min_size.x);
                new_rect.max.x = new_right;
            }
            DragState::ResizingTop => {
                // 调整上边缘
                let new_top = (start_rect.top() + delta.y).max(start_rect.bottom() - self.max_size.y);
                let new_top = new_top.min(start_rect.bottom() - self.min_size.y);
                new_rect.min.y = new_top;
            }
            DragState::ResizingBottom => {
                // 调整下边缘
                let new_bottom = (start_rect.bottom() + delta.y).min(start_rect.top() + self.max_size.y);
                let new_bottom = new_bottom.max(start_rect.top() + self.min_size.y);
                new_rect.max.y = new_bottom;
            }
            DragState::ResizingTopLeft => {
                // 调整左上角
                let new_left = (start_rect.left() + delta.x).max(start_rect.right() - self.max_size.x);
                let new_left = new_left.min(start_rect.right() - self.min_size.x);
                let new_top = (start_rect.top() + delta.y).max(start_rect.bottom() - self.max_size.y);
                let new_top = new_top.min(start_rect.bottom() - self.min_size.y);
                new_rect.min = egui::Pos2::new(new_left, new_top);
            }
            DragState::ResizingTopRight => {
                // 调整右上角
                let new_right = (start_rect.right() + delta.x).min(start_rect.left() + self.max_size.x);
                let new_right = new_right.max(start_rect.left() + self.min_size.x);
                let new_top = (start_rect.top() + delta.y).max(start_rect.bottom() - self.max_size.y);
                let new_top = new_top.min(start_rect.bottom() - self.min_size.y);
                new_rect.min.y = new_top;
                new_rect.max.x = new_right;
            }
            DragState::ResizingBottomLeft => {
                // 调整左下角
                let new_left = (start_rect.left() + delta.x).max(start_rect.right() - self.max_size.x);
                let new_left = new_left.min(start_rect.right() - self.min_size.x);
                let new_bottom = (start_rect.bottom() + delta.y).min(start_rect.top() + self.max_size.y);
                let new_bottom = new_bottom.max(start_rect.top() + self.min_size.y);
                new_rect.min.x = new_left;
                new_rect.max.y = new_bottom;
            }
            DragState::ResizingBottomRight => {
                // 调整右下角
                let new_right = (start_rect.right() + delta.x).min(start_rect.left() + self.max_size.x);
                let new_right = new_right.max(start_rect.left() + self.min_size.x);
                let new_bottom = (start_rect.bottom() + delta.y).min(start_rect.top() + self.max_size.y);
                let new_bottom = new_bottom.max(start_rect.top() + self.min_size.y);
                new_rect.max = egui::Pos2::new(new_right, new_bottom);
            }
            DragState::None => {
                // 无操作
            }
        }
        
        new_rect
    }
    
    /// 检查是否正在拖拽或调整大小
    pub fn is_active(&self) -> bool {
        self.drag_state != DragState::None
    }
    
    /// 重置状态
    pub fn reset(&mut self) {
        self.drag_state = DragState::None;
        self.drag_start_pos = None;
        self.drag_start_rect = None;
    }
}

/// 绘制调整大小的边框
pub fn draw_resize_borders(ui: &mut egui::Ui, rect: egui::Rect, border_width: f32, color: egui::Color32) {
    let painter = ui.painter();
    
    // 绘制边框
    let stroke = egui::Stroke::new(1.0, color);
    
    // 左边框
    painter.line_segment(
        [rect.left_top(), rect.left_bottom()],
        stroke,
    );
    
    // 右边框
    painter.line_segment(
        [rect.right_top(), rect.right_bottom()],
        stroke,
    );
    
    // 上边框
    painter.line_segment(
        [rect.left_top(), rect.right_top()],
        stroke,
    );
    
    // 下边框
    painter.line_segment(
        [rect.left_bottom(), rect.right_bottom()],
        stroke,
    );
    
    // 绘制调整大小的手柄
    let handle_size = border_width;
    let handle_color = color;
    
    // 角落手柄
    let corners = [
        rect.left_top(),
        rect.right_top(),
        rect.left_bottom(),
        rect.right_bottom(),
    ];
    
    for corner in corners {
        painter.circle_filled(corner, handle_size / 2.0, handle_color);
    }
    
    // 边缘中点手柄
    let edge_centers = [
        egui::Pos2::new(rect.center().x, rect.top()),
        egui::Pos2::new(rect.center().x, rect.bottom()),
        egui::Pos2::new(rect.left(), rect.center().y),
        egui::Pos2::new(rect.right(), rect.center().y),
    ];
    
    for center in edge_centers {
        painter.circle_filled(center, handle_size / 3.0, handle_color);
    }
}
