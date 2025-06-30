//! 知识图谱可视化UI模块
//! 
//! 提供实体关系图的可视化界面

use eframe::egui;
use crate::knowledge_graph_integration::{EntityInfo, RelationInfo, SemanticSearchResult};
use std::collections::HashMap;

/// 知识图谱可视化状态
#[derive(Default)]
pub struct KnowledgeGraphUI {
    /// 当前显示的实体
    pub current_entities: Vec<EntityInfo>,
    /// 当前显示的关系
    pub current_relations: Vec<RelationInfo>,
    /// 选中的实体
    pub selected_entity: Option<String>,
    /// 实体位置缓存
    pub entity_positions: HashMap<String, egui::Pos2>,
    /// 是否显示实体类型
    pub show_entity_types: bool,
    /// 是否显示置信度
    pub show_confidence: bool,
    /// 最小置信度阈值
    pub min_confidence: f32,
    /// 图谱缩放级别
    pub zoom_level: f32,
    /// 图谱偏移
    pub offset: egui::Vec2,
}

impl KnowledgeGraphUI {
    /// 创建新的知识图谱UI
    pub fn new() -> Self {
        Self {
            current_entities: Vec::new(),
            current_relations: Vec::new(),
            selected_entity: None,
            entity_positions: HashMap::new(),
            show_entity_types: true,
            show_confidence: true,
            min_confidence: 0.5,
            zoom_level: 1.0,
            offset: egui::Vec2::ZERO,
        }
    }
    
    /// 更新实体和关系数据
    pub fn update_data(&mut self, entities: Vec<EntityInfo>, relations: Vec<RelationInfo>) {
        self.current_entities = entities;
        self.current_relations = relations;
        
        // 重新计算实体位置
        self.calculate_entity_positions();
    }
    
    /// 计算实体位置（简单的圆形布局）
    fn calculate_entity_positions(&mut self) {
        self.entity_positions.clear();
        
        let entity_count = self.current_entities.len();
        if entity_count == 0 {
            return;
        }
        
        let center = egui::Pos2::new(300.0, 300.0);
        let radius = 150.0;
        
        for (i, entity) in self.current_entities.iter().enumerate() {
            let angle = 2.0 * std::f32::consts::PI * i as f32 / entity_count as f32;
            let x = center.x + radius * angle.cos();
            let y = center.y + radius * angle.sin();
            
            self.entity_positions.insert(entity.text.clone(), egui::Pos2::new(x, y));
        }
    }
    
    /// 渲染知识图谱
    pub fn render(&mut self, ui: &mut egui::Ui) {
        ui.heading("知识图谱");
        
        // 控制面板
        self.render_controls(ui);
        
        ui.separator();
        
        // 图谱可视化区域
        let available_size = ui.available_size();
        let (response, painter) = ui.allocate_painter(available_size, egui::Sense::click_and_drag());
        
        // 处理交互
        self.handle_interactions(&response);
        
        // 绘制图谱
        self.draw_graph(&painter, response.rect);
        
        // 实体详情面板
        if let Some(selected) = &self.selected_entity {
            self.render_entity_details(ui, selected);
        }
    }
    
    /// 渲染控制面板
    fn render_controls(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("显示选项:");
            ui.checkbox(&mut self.show_entity_types, "实体类型");
            ui.checkbox(&mut self.show_confidence, "置信度");
        });
        
        ui.horizontal(|ui| {
            ui.label("最小置信度:");
            ui.add(egui::Slider::new(&mut self.min_confidence, 0.0..=1.0).step_by(0.1));
        });
        
        ui.horizontal(|ui| {
            ui.label("缩放:");
            ui.add(egui::Slider::new(&mut self.zoom_level, 0.5..=2.0).step_by(0.1));
        });
        
        ui.horizontal(|ui| {
            if ui.button("重置视图").clicked() {
                self.reset_view();
            }
            if ui.button("重新布局").clicked() {
                self.calculate_entity_positions();
            }
        });
    }
    
    /// 处理用户交互
    fn handle_interactions(&mut self, response: &egui::Response) {
        // 处理拖拽
        if response.dragged() {
            self.offset += response.drag_delta();
        }
        
        // 处理点击选择实体
        if response.clicked() {
            if let Some(pos) = response.interact_pointer_pos() {
                self.selected_entity = self.find_entity_at_position(pos);
            }
        }
    }
    
    /// 绘制图谱
    fn draw_graph(&self, painter: &egui::Painter, rect: egui::Rect) {
        // 绘制背景
        painter.rect_filled(rect, 0.0, egui::Color32::from_gray(240));
        
        // 应用变换
        let transform = |pos: egui::Pos2| -> egui::Pos2 {
            let scaled = egui::Pos2::new(pos.x * self.zoom_level, pos.y * self.zoom_level);
            scaled + self.offset + rect.min.to_vec2()
        };
        
        // 绘制关系边
        self.draw_relations(painter, &transform);
        
        // 绘制实体节点
        self.draw_entities(painter, &transform);
    }
    
    /// 绘制关系边
    fn draw_relations(&self, painter: &egui::Painter, transform: &dyn Fn(egui::Pos2) -> egui::Pos2) {
        for relation in &self.current_relations {
            if relation.confidence < self.min_confidence as f64 {
                continue;
            }
            
            if let (Some(start_pos), Some(end_pos)) = (
                self.entity_positions.get(&relation.subject.text),
                self.entity_positions.get(&relation.object.text),
            ) {
                let start = transform(*start_pos);
                let end = transform(*end_pos);
                
                // 绘制箭头线
                let stroke = egui::Stroke::new(2.0, egui::Color32::from_gray(100));
                painter.line_segment([start, end], stroke);
                
                // 绘制箭头头部
                self.draw_arrow_head(painter, start, end);
                
                // 绘制关系标签
                if self.show_confidence {
                    let mid_point = egui::Pos2::new((start.x + end.x) / 2.0, (start.y + end.y) / 2.0);
                    let label = format!("{} ({:.2})", relation.relation_type, relation.confidence);
                    painter.text(
                        mid_point,
                        egui::Align2::CENTER_CENTER,
                        label,
                        egui::FontId::default(),
                        egui::Color32::BLACK,
                    );
                }
            }
        }
    }
    
    /// 绘制箭头头部
    fn draw_arrow_head(&self, painter: &egui::Painter, start: egui::Pos2, end: egui::Pos2) {
        let direction = (end - start).normalized();
        let perpendicular = egui::Vec2::new(-direction.y, direction.x);
        
        let arrow_length = 10.0;
        let arrow_width = 5.0;
        
        let tip = end - direction * 20.0; // 箭头距离节点的距离
        let left = tip - direction * arrow_length + perpendicular * arrow_width;
        let right = tip - direction * arrow_length - perpendicular * arrow_width;
        
        let points = vec![tip, left, right];
        painter.add(egui::Shape::convex_polygon(
            points,
            egui::Color32::from_gray(100),
            egui::Stroke::NONE,
        ));
    }
    
    /// 绘制实体节点
    fn draw_entities(&self, painter: &egui::Painter, transform: &dyn Fn(egui::Pos2) -> egui::Pos2) {
        for entity in &self.current_entities {
            if entity.confidence < self.min_confidence as f64 {
                continue;
            }
            
            if let Some(pos) = self.entity_positions.get(&entity.text) {
                let screen_pos = transform(*pos);
                
                // 选择颜色
                let color = if Some(&entity.text) == self.selected_entity.as_ref() {
                    egui::Color32::from_rgb(100, 150, 255) // 选中状态
                } else {
                    self.get_entity_color(&entity.entity_type)
                };
                
                // 绘制节点圆圈
                let radius = 20.0;
                painter.circle_filled(screen_pos, radius, color);
                painter.circle_stroke(screen_pos, radius, egui::Stroke::new(2.0, egui::Color32::BLACK));
                
                // 绘制实体文本
                let text_pos = screen_pos + egui::Vec2::new(0.0, radius + 10.0);
                let display_text = if self.show_entity_types {
                    format!("{}\n({})", entity.text, entity.entity_type)
                } else {
                    entity.text.clone()
                };
                
                painter.text(
                    text_pos,
                    egui::Align2::CENTER_TOP,
                    display_text,
                    egui::FontId::default(),
                    egui::Color32::BLACK,
                );
                
                // 显示置信度
                if self.show_confidence {
                    let confidence_text = format!("{:.2}", entity.confidence);
                    painter.text(
                        screen_pos,
                        egui::Align2::CENTER_CENTER,
                        confidence_text,
                        egui::FontId::monospace(10.0),
                        egui::Color32::WHITE,
                    );
                }
            }
        }
    }
    
    /// 获取实体类型对应的颜色
    fn get_entity_color(&self, entity_type: &str) -> egui::Color32 {
        match entity_type {
            "PERSON" => egui::Color32::from_rgb(255, 182, 193),      // 浅粉色
            "LOCATION" => egui::Color32::from_rgb(144, 238, 144),    // 浅绿色
            "ORGANIZATION" => egui::Color32::from_rgb(173, 216, 230), // 浅蓝色
            "TIME" => egui::Color32::from_rgb(255, 218, 185),        // 桃色
            "CONCEPT" => egui::Color32::from_rgb(221, 160, 221),     // 梅红色
            "TECHNOLOGY" => egui::Color32::from_rgb(255, 165, 0),    // 橙色
            "PRODUCT" => egui::Color32::from_rgb(255, 255, 0),       // 黄色
            _ => egui::Color32::from_rgb(192, 192, 192),             // 灰色
        }
    }
    
    /// 查找指定位置的实体
    fn find_entity_at_position(&self, pos: egui::Pos2) -> Option<String> {
        for entity in &self.current_entities {
            if let Some(entity_pos) = self.entity_positions.get(&entity.text) {
                let distance = (pos - *entity_pos).length();
                if distance <= 20.0 { // 节点半径
                    return Some(entity.text.clone());
                }
            }
        }
        None
    }
    
    /// 渲染实体详情
    fn render_entity_details(&self, ui: &mut egui::Ui, entity_text: &str) {
        ui.separator();
        ui.heading("实体详情");
        
        if let Some(entity) = self.current_entities.iter().find(|e| e.text == *entity_text) {
            ui.label(format!("实体: {}", entity.text));
            ui.label(format!("类型: {}", entity.entity_type));
            ui.label(format!("置信度: {:.3}", entity.confidence));
            ui.label(format!("位置: {}-{}", entity.start, entity.end));
            
            ui.separator();
            ui.label("相关关系:");
            
            for relation in &self.current_relations {
                if relation.subject.text == *entity_text || relation.object.text == *entity_text {
                    let relation_text = if relation.subject.text == *entity_text {
                        format!("{} -> {} ({})", relation.subject.text, relation.object.text, relation.relation_type)
                    } else {
                        format!("{} -> {} ({})", relation.subject.text, relation.object.text, relation.relation_type)
                    };
                    ui.label(format!("  • {}", relation_text));
                }
            }
        }
    }
    
    /// 重置视图
    fn reset_view(&mut self) {
        self.zoom_level = 1.0;
        self.offset = egui::Vec2::ZERO;
        self.selected_entity = None;
    }
}
