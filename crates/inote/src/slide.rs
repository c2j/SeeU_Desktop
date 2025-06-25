use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use regex::Regex;

/// 幻灯片数据结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Slide {
    /// 幻灯片标题
    pub title: String,
    /// 幻灯片内容（Markdown格式）
    pub content: String,
    /// 幻灯片索引
    pub index: usize,
    /// 自定义CSS样式
    pub css: Option<String>,
    /// 幻灯片配置
    pub config: SlideConfig,
}

/// 幻灯片配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlideConfig {
    /// 背景色
    pub background_color: Option<String>,
    /// 文字颜色
    pub text_color: Option<String>,
    /// 字体大小
    pub font_size: Option<f32>,
    /// 对齐方式
    pub alignment: SlideAlignment,
    /// 动画效果
    pub transition: SlideTransition,
}

/// 幻灯片样式模板
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlideTemplate {
    /// 模板ID
    pub id: String,
    /// 模板名称
    pub name: String,
    /// 模板描述
    pub description: String,
    /// 背景色
    pub background_color: String,
    /// 文字颜色
    pub text_color: String,
    /// 标题颜色
    pub title_color: String,
    /// 背景图片URL（可选）
    pub background_image: Option<String>,
    /// 字体大小
    pub font_size: f32,
    /// 标题字体大小
    pub title_font_size: f32,
    /// 对齐方式
    pub alignment: SlideAlignment,
    /// 显示比例
    pub aspect_ratio: SlideAspectRatio,
    /// 动画效果
    pub transition: SlideTransition,
    /// 内边距
    pub padding: f32,
    /// 行间距
    pub line_spacing: f32,
    /// 是否为内置模板
    pub is_builtin: bool,
}

/// 幻灯片对齐方式
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SlideAlignment {
    Left,
    Center,
    Right,
    Justify,
}

/// 幻灯片切换动画
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SlideTransition {
    None,
    Fade,
    Slide,
    Zoom,
}

/// 幻灯片显示比例
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SlideAspectRatio {
    /// 16:9 宽屏
    Widescreen,
    /// 4:3 标准
    Standard,
    /// 3:2 经典
    Classic,
    /// 1:1 正方形
    Square,
    /// 自定义比例
    Custom(f32, f32),
}

impl SlideAspectRatio {
    /// 获取比例值
    pub fn ratio(&self) -> f32 {
        match self {
            SlideAspectRatio::Widescreen => 16.0 / 9.0,
            SlideAspectRatio::Standard => 4.0 / 3.0,
            SlideAspectRatio::Classic => 3.0 / 2.0,
            SlideAspectRatio::Square => 1.0,
            SlideAspectRatio::Custom(w, h) => w / h,
        }
    }

    /// 获取显示名称
    pub fn display_name(&self) -> &str {
        match self {
            SlideAspectRatio::Widescreen => "16:9 宽屏",
            SlideAspectRatio::Standard => "4:3 标准",
            SlideAspectRatio::Classic => "3:2 经典",
            SlideAspectRatio::Square => "1:1 正方形",
            SlideAspectRatio::Custom(w, h) => "自定义",
        }
    }
}

/// 幻灯片演示文稿
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlideShow {
    /// 演示文稿标题
    pub title: String,
    /// 幻灯片列表
    pub slides: Vec<Slide>,
    /// 全局配置
    pub global_config: SlideConfig,
    /// 自定义CSS
    pub custom_css: Option<String>,
    /// 使用的样式模板ID
    pub template_id: Option<String>,
}

/// 幻灯片样式管理器
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlideStyleManager {
    /// 用户自定义模板
    pub custom_templates: Vec<SlideTemplate>,
    /// 当前选中的模板ID
    pub selected_template_id: String,
}

impl Default for SlideStyleManager {
    fn default() -> Self {
        Self {
            custom_templates: Vec::new(),
            selected_template_id: "default".to_string(),
        }
    }
}

impl SlideStyleManager {
    /// 获取所有可用模板（内置 + 自定义）
    pub fn get_all_templates(&self) -> Vec<SlideTemplate> {
        let mut templates = SlideTemplate::builtin_templates();
        templates.extend(self.custom_templates.clone());
        templates
    }

    /// 根据ID获取模板
    pub fn get_template(&self, id: &str) -> Option<SlideTemplate> {
        self.get_all_templates().into_iter().find(|t| t.id == id)
    }

    /// 获取当前选中的模板
    pub fn get_selected_template(&self) -> SlideTemplate {
        self.get_template(&self.selected_template_id)
            .unwrap_or_else(|| SlideTemplate::builtin_templates()[0].clone())
    }

    /// 添加自定义模板
    pub fn add_custom_template(&mut self, template: SlideTemplate) {
        if !template.is_builtin {
            self.custom_templates.push(template);
        }
    }

    /// 删除自定义模板
    pub fn remove_custom_template(&mut self, id: &str) {
        self.custom_templates.retain(|t| t.id != id);
        // 如果删除的是当前选中的模板，切换到默认模板
        if self.selected_template_id == id {
            self.selected_template_id = "default".to_string();
        }
    }

    /// 更新自定义模板
    pub fn update_custom_template(&mut self, template: SlideTemplate) {
        if let Some(index) = self.custom_templates.iter().position(|t| t.id == template.id) {
            self.custom_templates[index] = template;
        }
    }

    /// 设置选中的模板
    pub fn set_selected_template(&mut self, id: String) {
        if self.get_template(&id).is_some() {
            self.selected_template_id = id;
        }
    }
}

impl Default for SlideConfig {
    fn default() -> Self {
        Self {
            background_color: None,
            text_color: None,
            font_size: None,
            alignment: SlideAlignment::Left,
            transition: SlideTransition::None,
        }
    }
}

impl SlideTemplate {
    /// 创建新的样式模板
    pub fn new(name: String, description: String) -> Self {
        // 生成简单的ID
        let id = format!("custom_{}", std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis());

        Self {
            id,
            name,
            description,
            background_color: "#ffffff".to_string(),
            text_color: "#333333".to_string(),
            title_color: "#1a1a1a".to_string(),
            background_image: None,
            font_size: 16.0,
            title_font_size: 24.0,
            alignment: SlideAlignment::Left,
            aspect_ratio: SlideAspectRatio::Widescreen,
            transition: SlideTransition::None,
            padding: 40.0,
            line_spacing: 1.5,
            is_builtin: false,
        }
    }

    /// 创建内置模板
    pub fn builtin(id: &str, name: &str, description: &str) -> Self {
        let mut template = Self::new(name.to_string(), description.to_string());
        template.id = id.to_string();
        template.is_builtin = true;
        template
    }

    /// 获取内置模板列表
    pub fn builtin_templates() -> Vec<SlideTemplate> {
        vec![
            {
                let mut template = SlideTemplate::builtin(
                    "default",
                    "默认样式",
                    "简洁的白色背景，适合大多数场景"
                );
                template.background_color = "#ffffff".to_string();
                template.text_color = "#333333".to_string();
                template.title_color = "#1a1a1a".to_string();
                template
            },
            {
                let mut template = SlideTemplate::builtin(
                    "dark",
                    "深色主题",
                    "深色背景，适合夜间演示"
                );
                template.background_color = "#1a1a1a".to_string();
                template.text_color = "#e0e0e0".to_string();
                template.title_color = "#ffffff".to_string();
                template
            },
            {
                let mut template = SlideTemplate::builtin(
                    "blue",
                    "蓝色商务",
                    "专业的蓝色主题，适合商务演示"
                );
                template.background_color = "#2c3e50".to_string();
                template.text_color = "#ecf0f1".to_string();
                template.title_color = "#3498db".to_string();
                template
            },
            {
                let mut template = SlideTemplate::builtin(
                    "green",
                    "自然绿色",
                    "清新的绿色主题，适合环保或自然主题"
                );
                template.background_color = "#27ae60".to_string();
                template.text_color = "#ffffff".to_string();
                template.title_color = "#f1c40f".to_string();
                template
            },
            {
                let mut template = SlideTemplate::builtin(
                    "gradient",
                    "渐变背景",
                    "现代感的渐变背景"
                );
                template.background_color = "#667eea".to_string();
                template.text_color = "#ffffff".to_string();
                template.title_color = "#f093fb".to_string();
                template.alignment = SlideAlignment::Center;
                template
            },
        ]
    }
}

impl Default for SlideShow {
    fn default() -> Self {
        Self {
            title: "未命名演示文稿".to_string(),
            slides: Vec::new(),
            global_config: SlideConfig::default(),
            custom_css: None,
            template_id: Some("default".to_string()),
        }
    }
}

/// 幻灯片解析器
pub struct SlideParser {
    /// 幻灯片分隔符正则表达式
    slide_separator: Regex,
    /// CSS块正则表达式
    css_block: Regex,
    /// 配置行正则表达式
    config_line: Regex,
}

impl Default for SlideParser {
    fn default() -> Self {
        Self::new()
    }
}

impl SlideParser {
    /// 创建新的幻灯片解析器
    pub fn new() -> Self {
        Self {
            // 匹配 --slide 或 --- 作为幻灯片分隔符
            slide_separator: Regex::new(r"^--slide|^---$").unwrap(),
            // 匹配CSS块 ```css ... ```
            css_block: Regex::new(r"```css\s*\n(.*?)\n```").unwrap(),
            // 匹配配置行，如 <!-- config: background=#fff -->
            config_line: Regex::new(r"<!--\s*config:\s*(.*?)\s*-->").unwrap(),
        }
    }

    /// 检测markdown内容是否包含幻灯片标记
    pub fn is_slideshow(&self, content: &str) -> bool {
        // 检查是否包含 --slide 标记
        if content.contains("--slide") {
            return true;
        }
        
        // 检查是否包含CSS定义（可能是幻灯片样式）
        if self.css_block.is_match(content) {
            return true;
        }
        
        // 检查是否包含多个 --- 分隔符（可能是幻灯片分隔）
        let separator_count = content.lines()
            .filter(|line| line.trim() == "---")
            .count();
        
        separator_count >= 2
    }

    /// 解析markdown内容为幻灯片演示文稿
    pub fn parse(&self, content: &str) -> Result<SlideShow, String> {
        if !self.is_slideshow(content) {
            return Err("内容不包含幻灯片标记".to_string());
        }

        let mut slideshow = SlideShow::default();
        
        // 提取全局CSS
        if let Some(css_match) = self.css_block.find(content) {
            slideshow.custom_css = Some(css_match.as_str().to_string());
        }

        // 分割幻灯片
        let slides_content = self.split_slides(content);
        
        // 解析每个幻灯片
        for (index, slide_content) in slides_content.iter().enumerate() {
            let slide = self.parse_slide(slide_content, index)?;
            slideshow.slides.push(slide);
        }

        // 如果有幻灯片，从第一个幻灯片提取标题
        if let Some(first_slide) = slideshow.slides.first() {
            if !first_slide.title.is_empty() {
                slideshow.title = first_slide.title.clone();
            }
        }

        Ok(slideshow)
    }

    /// 分割markdown内容为多个幻灯片
    fn split_slides(&self, content: &str) -> Vec<String> {
        let mut slides = Vec::new();
        let mut current_slide = String::new();
        
        for line in content.lines() {
            if self.slide_separator.is_match(line.trim()) {
                // 遇到分隔符，保存当前幻灯片
                if !current_slide.trim().is_empty() {
                    slides.push(current_slide.trim().to_string());
                }
                current_slide.clear();
            } else {
                // 添加到当前幻灯片
                current_slide.push_str(line);
                current_slide.push('\n');
            }
        }
        
        // 添加最后一个幻灯片
        if !current_slide.trim().is_empty() {
            slides.push(current_slide.trim().to_string());
        }
        
        // 如果没有找到分隔符，将整个内容作为一个幻灯片
        if slides.is_empty() && !content.trim().is_empty() {
            slides.push(content.trim().to_string());
        }
        
        slides
    }

    /// 解析单个幻灯片
    fn parse_slide(&self, content: &str, index: usize) -> Result<Slide, String> {
        let mut slide = Slide {
            title: String::new(),
            content: content.to_string(),
            index,
            css: None,
            config: SlideConfig::default(),
        };

        // 提取标题（第一个标题行）
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with('#') {
                // 提取标题文本
                slide.title = trimmed.trim_start_matches('#').trim().to_string();
                break;
            }
        }

        // 解析配置
        if let Some(config_match) = self.config_line.find(content) {
            let config_str = config_match.as_str();
            slide.config = self.parse_config(config_str)?;
        }

        // 提取CSS
        if let Some(css_match) = self.css_block.find(content) {
            slide.css = Some(css_match.as_str().to_string());
        }

        Ok(slide)
    }

    /// 解析配置字符串
    fn parse_config(&self, config_str: &str) -> Result<SlideConfig, String> {
        let mut config = SlideConfig::default();
        
        // 提取配置内容
        if let Some(captures) = self.config_line.captures(config_str) {
            if let Some(config_content) = captures.get(1) {
                let config_pairs: HashMap<String, String> = config_content
                    .as_str()
                    .split(',')
                    .filter_map(|pair| {
                        let parts: Vec<&str> = pair.split('=').collect();
                        if parts.len() == 2 {
                            Some((parts[0].trim().to_string(), parts[1].trim().to_string()))
                        } else {
                            None
                        }
                    })
                    .collect();

                // 解析各个配置项
                if let Some(bg) = config_pairs.get("background") {
                    config.background_color = Some(bg.clone());
                }
                
                if let Some(color) = config_pairs.get("color") {
                    config.text_color = Some(color.clone());
                }
                
                if let Some(size) = config_pairs.get("font-size") {
                    if let Ok(size_f) = size.parse::<f32>() {
                        config.font_size = Some(size_f);
                    }
                }
                
                if let Some(align) = config_pairs.get("align") {
                    config.alignment = match align.as_str() {
                        "center" => SlideAlignment::Center,
                        "right" => SlideAlignment::Right,
                        _ => SlideAlignment::Left,
                    };
                }
                
                if let Some(transition) = config_pairs.get("transition") {
                    config.transition = match transition.as_str() {
                        "fade" => SlideTransition::Fade,
                        "slide" => SlideTransition::Slide,
                        "zoom" => SlideTransition::Zoom,
                        _ => SlideTransition::None,
                    };
                }
            }
        }
        
        Ok(config)
    }
}

/// 幻灯片播放状态
#[derive(Debug, Clone)]
pub struct SlidePlayState {
    /// 当前幻灯片索引
    pub current_slide: usize,
    /// 是否处于播放模式
    pub is_playing: bool,
    /// 是否全屏显示
    pub fullscreen: bool,
    /// 演示文稿数据
    pub slideshow: Option<SlideShow>,
    /// 当前使用的样式模板
    pub current_template: Option<SlideTemplate>,
    /// 是否显示样式选择器
    pub show_style_selector: bool,
}

impl Default for SlidePlayState {
    fn default() -> Self {
        Self {
            current_slide: 0,
            is_playing: false,
            fullscreen: false,
            slideshow: None,
            current_template: None,
            show_style_selector: false,
        }
    }
}

impl SlidePlayState {
    /// 开始播放幻灯片
    pub fn start_slideshow(&mut self, slideshow: SlideShow) {
        self.slideshow = Some(slideshow);
        self.current_slide = 0;
        self.is_playing = true;
        self.fullscreen = true;
        self.show_style_selector = false;
    }

    /// 开始播放幻灯片（带样式模板）
    pub fn start_slideshow_with_template(&mut self, slideshow: SlideShow, template: SlideTemplate) {
        self.slideshow = Some(slideshow);
        self.current_template = Some(template);
        self.current_slide = 0;
        self.is_playing = true;
        self.fullscreen = true;
        self.show_style_selector = false;
    }

    /// 停止播放幻灯片
    pub fn stop_slideshow(&mut self) {
        self.is_playing = false;
        self.fullscreen = false;
        self.slideshow = None;
        self.current_template = None;
        self.current_slide = 0;
        self.show_style_selector = false;
    }

    /// 设置当前样式模板
    pub fn set_template(&mut self, template: SlideTemplate) {
        self.current_template = Some(template);
    }

    /// 切换样式选择器显示状态
    pub fn toggle_style_selector(&mut self) {
        self.show_style_selector = !self.show_style_selector;
    }

    /// 下一张幻灯片
    pub fn next_slide(&mut self) -> bool {
        if let Some(slideshow) = &self.slideshow {
            if self.current_slide < slideshow.slides.len() - 1 {
                self.current_slide += 1;
                return true;
            }
        }
        false
    }

    /// 上一张幻灯片
    pub fn previous_slide(&mut self) -> bool {
        if self.current_slide > 0 {
            self.current_slide -= 1;
            return true;
        }
        false
    }

    /// 跳转到指定幻灯片
    pub fn goto_slide(&mut self, index: usize) -> bool {
        if let Some(slideshow) = &self.slideshow {
            if index < slideshow.slides.len() {
                self.current_slide = index;
                return true;
            }
        }
        false
    }

    /// 获取当前幻灯片
    pub fn get_current_slide(&self) -> Option<&Slide> {
        self.slideshow.as_ref()?.slides.get(self.current_slide)
    }

    /// 获取幻灯片总数
    pub fn get_slide_count(&self) -> usize {
        self.slideshow.as_ref().map_or(0, |s| s.slides.len())
    }
}

/// 幻灯片播放UI渲染器
pub struct SlideRenderer;

impl SlideRenderer {
    /// 渲染幻灯片播放界面
    pub fn render_slideshow(ui: &mut eframe::egui::Ui, play_state: &mut SlidePlayState) -> bool {
        let mut should_close = false;

        if !play_state.is_playing || play_state.slideshow.is_none() {
            return true; // 关闭播放界面
        }

        // 检查是否播放完成
        if play_state.current_slide >= play_state.get_slide_count() {
            should_close = Self::render_completion_screen(ui, play_state);
        } else {
            // 全屏显示幻灯片
            if play_state.fullscreen {
                should_close = Self::render_fullscreen_slide(ui, play_state);
            } else {
                should_close = Self::render_windowed_slide(ui, play_state);
            }
        }

        should_close
    }

    /// 渲染播放完成界面
    fn render_completion_screen(ui: &mut eframe::egui::Ui, play_state: &mut SlidePlayState) -> bool {
        let mut should_close = false;
        let screen_rect = ui.ctx().screen_rect();

        eframe::egui::Window::new("演示完成")
            .title_bar(false)
            .resizable(false)
            .movable(false)
            .anchor(eframe::egui::Align2::CENTER_CENTER, eframe::egui::Vec2::ZERO)
            .fixed_size(screen_rect.size())
            .show(ui.ctx(), |ui| {
                // 设置背景
                let bg_color = if let Some(template) = &play_state.current_template {
                    Self::parse_color(&template.background_color).unwrap_or(eframe::egui::Color32::from_rgb(20, 20, 20))
                } else {
                    eframe::egui::Color32::from_rgb(20, 20, 20)
                };

                ui.painter().rect_filled(
                    ui.available_rect_before_wrap(),
                    eframe::egui::Rounding::ZERO,
                    bg_color,
                );

                ui.vertical_centered(|ui| {
                    ui.add_space(screen_rect.height() * 0.3);

                    // 完成图标和文字
                    ui.label(egui::RichText::new("🎉").size(80.0));
                    ui.add_space(20.0);
                    ui.label(egui::RichText::new("演示完成")
                        .size(32.0)
                        .color(eframe::egui::Color32::WHITE));

                    ui.add_space(40.0);

                    // 提示文字
                    ui.label(egui::RichText::new("按 ESC 或 空格键 退出播放")
                        .size(18.0)
                        .color(eframe::egui::Color32::LIGHT_GRAY));

                    ui.add_space(20.0);

                    // 操作按钮
                    ui.horizontal(|ui| {
                        if ui.button("🔄 重新播放").clicked() {
                            play_state.current_slide = 0;
                        }

                        if ui.button("✕ 退出").clicked() {
                            should_close = true;
                        }
                    });
                });
            });

        // 处理键盘输入
        ui.ctx().input(|i| {
            if i.key_pressed(eframe::egui::Key::Escape) || i.key_pressed(eframe::egui::Key::Space) {
                should_close = true;
            }
        });

        should_close
    }

    /// 渲染全屏幻灯片
    fn render_fullscreen_slide(ui: &mut eframe::egui::Ui, play_state: &mut SlidePlayState) -> bool {
        let mut should_close = false;

        // 使用全屏区域
        let screen_rect = ui.ctx().screen_rect();

        // 创建全屏窗口，使用绝对位置
        eframe::egui::Window::new("幻灯片播放")
            .title_bar(false)
            .resizable(false)
            .movable(false)
            .anchor(eframe::egui::Align2::LEFT_TOP, eframe::egui::Vec2::ZERO)
            .fixed_pos(eframe::egui::Pos2::new(0.0, 0.0))
            .fixed_size(screen_rect.size())
            .show(ui.ctx(), |ui| {
                // 应用样式模板背景
                let bg_color = if let Some(template) = &play_state.current_template {
                    Self::parse_color(&template.background_color).unwrap_or(eframe::egui::Color32::from_rgb(20, 20, 20))
                } else {
                    eframe::egui::Color32::from_rgb(20, 20, 20)
                };

                ui.painter().rect_filled(
                    ui.available_rect_before_wrap(),
                    eframe::egui::Rounding::ZERO,
                    bg_color,
                );

                // 获取当前样式模板的对齐方式
                let alignment = if let Some(template) = &play_state.current_template {
                    log::info!("🎯 全屏播放使用模板: {} ({}), 对齐方式: {:?}",
                              template.name, template.id, template.alignment);
                    template.alignment.clone()
                } else {
                    log::warn!("⚠️ 全屏播放时没有找到当前模板，使用默认左对齐");
                    SlideAlignment::Left
                };

                // 先渲染幻灯片内容，确保内容在最上层
                Self::render_fullscreen_content(ui, play_state, &mut should_close);

                // 然后在内容之上绘制导航箭头
                // 左侧导航箭头
                let left_arrow_rect = eframe::egui::Rect::from_min_size(
                    eframe::egui::Pos2::new(20.0, screen_rect.height() * 0.5 - 30.0),
                    eframe::egui::Vec2::new(60.0, 60.0)
                );

                if play_state.current_slide > 0 {
                    let left_response = ui.allocate_rect(left_arrow_rect, eframe::egui::Sense::click());
                    if left_response.clicked() {
                        play_state.previous_slide();
                    }

                    // 绘制左箭头
                    let arrow_color = if left_response.hovered() {
                        eframe::egui::Color32::WHITE
                    } else {
                        eframe::egui::Color32::LIGHT_GRAY
                    };

                    ui.painter().text(
                        left_arrow_rect.center(),
                        eframe::egui::Align2::CENTER_CENTER,
                        "◀",
                        eframe::egui::FontId::proportional(40.0),
                        arrow_color,
                    );
                }

                // 右侧导航箭头
                let right_arrow_rect = eframe::egui::Rect::from_min_size(
                    eframe::egui::Pos2::new(screen_rect.width() - 80.0, screen_rect.height() * 0.5 - 30.0),
                    eframe::egui::Vec2::new(60.0, 60.0)
                );

                if play_state.current_slide < play_state.get_slide_count() - 1 {
                    let right_response = ui.allocate_rect(right_arrow_rect, eframe::egui::Sense::click());
                    if right_response.clicked() {
                        play_state.next_slide();
                    }

                    // 绘制右箭头
                    let arrow_color = if right_response.hovered() {
                        eframe::egui::Color32::WHITE
                    } else {
                        eframe::egui::Color32::LIGHT_GRAY
                    };

                    ui.painter().text(
                        right_arrow_rect.center(),
                        eframe::egui::Align2::CENTER_CENTER,
                        "▶",
                        eframe::egui::FontId::proportional(40.0),
                        arrow_color,
                    );
                }
            });

        // 渲染样式选择器
        if play_state.show_style_selector {
            should_close = Self::render_style_selector(ui, play_state) || should_close;
        }

        // 处理键盘快捷键
        Self::handle_keyboard_input(ui, play_state, &mut should_close);

        should_close
    }

    /// 渲染全屏内容（提取的公共方法）
    fn render_fullscreen_content(ui: &mut eframe::egui::Ui, play_state: &mut SlidePlayState, should_close: &mut bool) {
        // 顶部控制栏
        ui.horizontal(|ui| {
            // 样式选择按钮
            if ui.button("🎨 样式").clicked() {
                play_state.toggle_style_selector();
            }

            ui.with_layout(eframe::egui::Layout::right_to_left(eframe::egui::Align::Center), |ui| {
                // 关闭按钮
                if ui.button("✕ 退出").clicked() {
                    *should_close = true;
                }

                // 窗口模式按钮
                if ui.button("🗗 窗口").clicked() {
                    play_state.fullscreen = false;
                }
            });
        });

        ui.add_space(20.0);

        // 幻灯片内容区域
        Self::render_slide_content(ui, play_state);

        ui.add_space(20.0);

        // 底部导航栏
        Self::render_navigation_bar(ui, play_state);
    }

    /// 渲染窗口模式幻灯片
    fn render_windowed_slide(ui: &mut eframe::egui::Ui, play_state: &mut SlidePlayState) -> bool {
        let mut should_close = false;

        eframe::egui::Window::new("幻灯片播放")
            .resizable(true)
            .default_size([800.0, 600.0])
            .show(ui.ctx(), |ui| {
                ui.vertical(|ui| {
                    // 顶部控制栏
                    ui.horizontal(|ui| {
                        // 全屏按钮
                        if ui.button("🔍 全屏").clicked() {
                            play_state.fullscreen = true;
                        }

                        ui.with_layout(eframe::egui::Layout::right_to_left(eframe::egui::Align::Center), |ui| {
                            // 关闭按钮
                            if ui.button("✕ 关闭").clicked() {
                                should_close = true;
                            }
                        });
                    });

                    ui.separator();

                    // 幻灯片内容区域
                    Self::render_slide_content(ui, play_state);

                    ui.separator();

                    // 底部导航栏
                    Self::render_navigation_bar(ui, play_state);
                });
            });

        // 处理键盘快捷键
        Self::handle_keyboard_input(ui, play_state, &mut should_close);

        should_close
    }

    /// 渲染样式选择器
    fn render_style_selector(ui: &mut eframe::egui::Ui, play_state: &mut SlidePlayState) -> bool {
        let mut should_close = false;

        eframe::egui::Window::new("选择样式模板")
            .resizable(false)
            .collapsible(false)
            .anchor(eframe::egui::Align2::CENTER_CENTER, eframe::egui::Vec2::ZERO)
            .show(ui.ctx(), |ui| {
                ui.vertical(|ui| {
                    ui.label("选择一个样式模板:");
                    ui.add_space(10.0);

                    // 这里需要从样式管理器获取模板列表
                    // 暂时使用内置模板
                    let templates = crate::slide::SlideTemplate::builtin_templates();

                    for template in templates {
                        ui.horizontal(|ui| {
                            // 样式预览
                            let preview_rect = ui.allocate_response(
                                eframe::egui::Vec2::new(40.0, 30.0),
                                eframe::egui::Sense::hover()
                            ).rect;

                            let bg_color = Self::parse_color(&template.background_color)
                                .unwrap_or(eframe::egui::Color32::WHITE);
                            let text_color = Self::parse_color(&template.text_color)
                                .unwrap_or(eframe::egui::Color32::BLACK);

                            ui.painter().rect_filled(
                                preview_rect,
                                eframe::egui::Rounding::same(4.0),
                                bg_color,
                            );

                            ui.painter().text(
                                preview_rect.center(),
                                eframe::egui::Align2::CENTER_CENTER,
                                "Aa",
                                eframe::egui::FontId::proportional(12.0),
                                text_color,
                            );

                            // 模板信息
                            ui.vertical(|ui| {
                                ui.label(egui::RichText::new(&template.name).strong());
                                ui.label(egui::RichText::new(&template.description).small());
                            });

                            // 选择按钮
                            if ui.button("选择").clicked() {
                                play_state.set_template(template);
                                play_state.show_style_selector = false;
                            }
                        });
                        ui.add_space(5.0);
                    }

                    ui.add_space(10.0);

                    if ui.button("取消").clicked() {
                        play_state.show_style_selector = false;
                    }
                });
            });

        should_close
    }

    /// 解析颜色字符串
    fn parse_color(color_str: &str) -> Option<eframe::egui::Color32> {
        if color_str.starts_with('#') && color_str.len() == 7 {
            if let Ok(r) = u8::from_str_radix(&color_str[1..3], 16) {
                if let Ok(g) = u8::from_str_radix(&color_str[3..5], 16) {
                    if let Ok(b) = u8::from_str_radix(&color_str[5..7], 16) {
                        return Some(eframe::egui::Color32::from_rgb(r, g, b));
                    }
                }
            }
        }
        None
    }

    /// 渲染幻灯片内容
    fn render_slide_content(ui: &mut eframe::egui::Ui, play_state: &SlidePlayState) {
        if let Some(slide) = play_state.get_current_slide() {
            // 获取样式模板的对齐方式
            let alignment = if let Some(template) = &play_state.current_template {
                log::info!("🎯 渲染幻灯片内容 - 模板: {} ({}), 对齐方式: {:?}",
                          template.name, template.id, template.alignment);
                template.alignment.clone()
            } else {
                log::warn!("⚠️ 渲染幻灯片内容时没有找到当前模板，使用默认左对齐");
                SlideAlignment::Left
            };

            // 创建滚动区域
            eframe::egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    // 应用样式模板
                    Self::apply_template_style(ui, play_state);

                    // 根据对齐方式渲染内容
                    match alignment {
                        SlideAlignment::Center => {
                            ui.vertical_centered(|ui| {
                                Self::render_slide_title_and_content(ui, slide, play_state);
                            });
                        },
                        SlideAlignment::Right => {
                            ui.with_layout(eframe::egui::Layout::top_down(eframe::egui::Align::Max), |ui| {
                                Self::render_slide_title_and_content(ui, slide, play_state);
                            });
                        },
                        SlideAlignment::Left | SlideAlignment::Justify => {
                            ui.with_layout(eframe::egui::Layout::top_down(eframe::egui::Align::Min), |ui| {
                                Self::render_slide_title_and_content(ui, slide, play_state);
                            });
                        },
                    }
                });
        } else {
            ui.centered_and_justified(|ui| {
                ui.label("无法加载幻灯片内容");
            });
        }
    }

    /// 渲染幻灯片标题和内容（提取的公共方法）
    fn render_slide_title_and_content(ui: &mut eframe::egui::Ui, slide: &Slide, play_state: &SlidePlayState) {
        log::info!("🎯 渲染幻灯片标题和内容 - 标题: '{}'", slide.title);

        // 渲染幻灯片标题
        if !slide.title.is_empty() {
            let title_color = if let Some(template) = &play_state.current_template {
                Self::parse_color(&template.title_color).unwrap_or(eframe::egui::Color32::WHITE)
            } else {
                eframe::egui::Color32::WHITE
            };

            // 直接渲染标题，对齐方式由外层控制
            ui.label(egui::RichText::new(&slide.title)
                .size(32.0)
                .color(title_color)
                .strong());
            ui.add_space(20.0);
        }

        // 渲染幻灯片内容
        crate::markdown::render_markdown(ui, &slide.content);
    }

    /// 应用样式模板
    fn apply_template_style(ui: &mut eframe::egui::Ui, play_state: &SlidePlayState) {
        if let Some(template) = &play_state.current_template {
            // 设置文字颜色
            if let Some(text_color) = Self::parse_color(&template.text_color) {
                ui.style_mut().visuals.override_text_color = Some(text_color);
            }

            // 设置字体大小（如果需要的话）
            // 注意：egui的字体大小通常在具体的文本渲染时设置
        }
    }

    /// 渲染导航栏
    fn render_navigation_bar(ui: &mut eframe::egui::Ui, play_state: &mut SlidePlayState) {
        ui.horizontal(|ui| {
            // 上一张按钮
            let prev_enabled = play_state.current_slide > 0;
            ui.add_enabled_ui(prev_enabled, |ui| {
                if ui.button("◀ 上一张").clicked() {
                    play_state.previous_slide();
                }
            });

            // 进度指示器
            let current = play_state.current_slide + 1;
            let total = play_state.get_slide_count();
            ui.label(format!("{} / {}", current, total));

            // 进度条
            let progress = if total > 0 {
                current as f32 / total as f32
            } else {
                0.0
            };

            ui.add(eframe::egui::ProgressBar::new(progress)
                .desired_width(200.0)
                .show_percentage());

            // 下一张按钮
            let next_enabled = play_state.current_slide < play_state.get_slide_count().saturating_sub(1);
            ui.add_enabled_ui(next_enabled, |ui| {
                if ui.button("下一张 ▶").clicked() {
                    play_state.next_slide();
                }
            });
        });
    }



    /// 处理键盘输入
    fn handle_keyboard_input(ui: &mut eframe::egui::Ui, play_state: &mut SlidePlayState, should_close: &mut bool) {
        let ctx = ui.ctx();

        // 检查键盘输入
        ctx.input(|i| {
            // 右箭头或空格键：下一张
            if i.key_pressed(eframe::egui::Key::ArrowRight) || i.key_pressed(eframe::egui::Key::Space) {
                play_state.next_slide();
            }

            // 左箭头：上一张
            if i.key_pressed(eframe::egui::Key::ArrowLeft) {
                play_state.previous_slide();
            }

            // ESC键：退出
            if i.key_pressed(eframe::egui::Key::Escape) {
                *should_close = true;
            }

            // F11：切换全屏
            if i.key_pressed(eframe::egui::Key::F11) {
                play_state.fullscreen = !play_state.fullscreen;
            }

            // Home键：第一张
            if i.key_pressed(eframe::egui::Key::Home) {
                play_state.goto_slide(0);
            }

            // End键：最后一张
            if i.key_pressed(eframe::egui::Key::End) {
                let last_index = play_state.get_slide_count().saturating_sub(1);
                play_state.goto_slide(last_index);
            }
        });
    }
}
