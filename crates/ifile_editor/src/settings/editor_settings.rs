//! 编辑器设置

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use chrono::{DateTime, Utc};

/// 最近访问的目录记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecentDirectory {
    pub path: String,
    pub accessed_at: DateTime<Utc>,
}

/// 最近访问的文件记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecentFile {
    pub path: String,
    pub name: String,
    pub accessed_at: DateTime<Utc>,
}

/// 编辑器设置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditorSettings {
    // 外观设置
    pub font_family: String,
    pub font_size: f32,
    pub line_height_factor: f32,
    pub theme: String,
    
    // 编辑器行为
    pub tab_size: usize,
    pub insert_spaces: bool,
    pub word_wrap: bool,
    pub show_line_numbers: bool,
    pub show_whitespace: bool,
    pub auto_indent: bool,
    
    // 文件设置
    pub auto_save: bool,
    pub auto_save_delay_ms: u64,
    pub default_encoding: String,
    pub detect_encoding: bool,
    
    // 性能设置
    pub syntax_highlighting: bool,
    pub max_file_size_mb: usize,
    pub virtual_scrolling: bool,

    // 工作区设置
    pub last_opened_directory: Option<String>,

    // 历史记录设置
    pub recent_directories: Vec<RecentDirectory>,
    pub recent_files: Vec<RecentFile>,
    pub max_recent_directories: usize,
    pub max_recent_files: usize,
}

impl Default for EditorSettings {
    fn default() -> Self {
        Self {
            font_family: "Consolas".to_string(),
            font_size: 14.0,
            line_height_factor: 1.2,
            theme: "dark".to_string(),
            tab_size: 4,
            insert_spaces: true,
            word_wrap: false,
            show_line_numbers: true,
            show_whitespace: false,
            auto_indent: true,
            auto_save: true,
            auto_save_delay_ms: 2000,
            default_encoding: "UTF-8".to_string(),
            detect_encoding: true,
            syntax_highlighting: true,
            max_file_size_mb: 10,
            virtual_scrolling: true,
            last_opened_directory: None,
            recent_directories: Vec::new(),
            recent_files: Vec::new(),
            max_recent_directories: 5,
            max_recent_files: 10,
        }
    }
}

impl EditorSettings {
    /// 计算行高
    pub fn line_height(&self) -> f32 {
        self.font_size * self.line_height_factor
    }
    
    /// 验证设置
    pub fn validate(&self) -> Result<(), String> {
        if self.font_size < 8.0 || self.font_size > 72.0 {
            return Err("字体大小必须在8-72之间".to_string());
        }
        
        if self.line_height_factor < 0.8 || self.line_height_factor > 3.0 {
            return Err("行高因子必须在0.8-3.0之间".to_string());
        }
        
        if self.tab_size == 0 || self.tab_size > 16 {
            return Err("制表符大小必须在1-16之间".to_string());
        }
        
        if self.max_file_size_mb == 0 || self.max_file_size_mb > 1024 {
            return Err("最大文件大小必须在1-1024MB之间".to_string());
        }
        
        Ok(())
    }
}

/// 文件编辑器设置模块
#[derive(Debug)]
pub struct FileEditorSettingsModule {
    settings: EditorSettings,
    temp_settings: EditorSettings,
}

impl FileEditorSettingsModule {
    pub fn new() -> Self {
        let settings = EditorSettings::default();
        Self {
            temp_settings: settings.clone(),
            settings,
        }
    }
    
    /// 渲染设置UI
    pub fn render_settings(&mut self, ui: &mut egui::Ui) -> bool {
        let mut changed = false;
        
        ui.heading("📝 文件编辑器设置");
        ui.separator();
        
        // 外观设置
        ui.collapsing("🎨 外观设置", |ui| {
            ui.horizontal(|ui| {
                ui.label("字体族:");
                if ui.text_edit_singleline(&mut self.temp_settings.font_family).changed() {
                    changed = true;
                }
            });
            
            ui.horizontal(|ui| {
                ui.label("字体大小:");
                if ui.add(egui::Slider::new(&mut self.temp_settings.font_size, 8.0..=72.0).suffix("px")).changed() {
                    changed = true;
                }
            });
            
            ui.horizontal(|ui| {
                ui.label("行高因子:");
                if ui.add(egui::Slider::new(&mut self.temp_settings.line_height_factor, 0.8..=3.0)).changed() {
                    changed = true;
                }
            });
            
            ui.horizontal(|ui| {
                ui.label("主题:");
                egui::ComboBox::from_label("")
                    .selected_text(&self.temp_settings.theme)
                    .show_ui(ui, |ui| {
                        if ui.selectable_value(&mut self.temp_settings.theme, "dark".to_string(), "深色").changed() {
                            changed = true;
                        }
                        if ui.selectable_value(&mut self.temp_settings.theme, "light".to_string(), "浅色").changed() {
                            changed = true;
                        }
                    });
            });
        });
        
        // 编辑器行为设置
        ui.collapsing("⚙️ 编辑器行为", |ui| {
            ui.horizontal(|ui| {
                ui.label("制表符大小:");
                if ui.add(egui::Slider::new(&mut self.temp_settings.tab_size, 1..=16)).changed() {
                    changed = true;
                }
            });
            
            if ui.checkbox(&mut self.temp_settings.insert_spaces, "使用空格代替制表符").changed() {
                changed = true;
            }
            
            if ui.checkbox(&mut self.temp_settings.word_wrap, "自动换行").changed() {
                changed = true;
            }
            
            if ui.checkbox(&mut self.temp_settings.show_line_numbers, "显示行号").changed() {
                changed = true;
            }
            
            if ui.checkbox(&mut self.temp_settings.show_whitespace, "显示空白字符").changed() {
                changed = true;
            }
            
            if ui.checkbox(&mut self.temp_settings.auto_indent, "自动缩进").changed() {
                changed = true;
            }
        });
        
        // 文件设置
        ui.collapsing("📁 文件设置", |ui| {
            if ui.checkbox(&mut self.temp_settings.auto_save, "自动保存").changed() {
                changed = true;
            }
            
            if self.temp_settings.auto_save {
                ui.horizontal(|ui| {
                    ui.label("自动保存延迟:");
                    let mut delay_seconds = self.temp_settings.auto_save_delay_ms as f32 / 1000.0;
                    if ui.add(egui::Slider::new(&mut delay_seconds, 0.5..=10.0).suffix("秒")).changed() {
                        self.temp_settings.auto_save_delay_ms = (delay_seconds * 1000.0) as u64;
                        changed = true;
                    }
                });
            }
            
            ui.horizontal(|ui| {
                ui.label("默认编码:");
                egui::ComboBox::from_label("")
                    .selected_text(&self.temp_settings.default_encoding)
                    .show_ui(ui, |ui| {
                        if ui.selectable_value(&mut self.temp_settings.default_encoding, "UTF-8".to_string(), "UTF-8").changed() {
                            changed = true;
                        }
                        if ui.selectable_value(&mut self.temp_settings.default_encoding, "GBK".to_string(), "GBK").changed() {
                            changed = true;
                        }
                        if ui.selectable_value(&mut self.temp_settings.default_encoding, "GB2312".to_string(), "GB2312").changed() {
                            changed = true;
                        }
                    });
            });
            
            if ui.checkbox(&mut self.temp_settings.detect_encoding, "自动检测编码").changed() {
                changed = true;
            }
        });
        
        // 性能设置
        ui.collapsing("🚀 性能设置", |ui| {
            if ui.checkbox(&mut self.temp_settings.syntax_highlighting, "语法高亮").changed() {
                changed = true;
            }
            
            ui.horizontal(|ui| {
                ui.label("最大文件大小:");
                if ui.add(egui::Slider::new(&mut self.temp_settings.max_file_size_mb, 1..=1024).suffix("MB")).changed() {
                    changed = true;
                }
            });
            
            if ui.checkbox(&mut self.temp_settings.virtual_scrolling, "虚拟滚动").changed() {
                changed = true;
            }
        });
        
        ui.separator();
        
        // 操作按钮
        ui.horizontal(|ui| {
            if ui.button("💾 保存设置").clicked() {
                if let Err(e) = self.temp_settings.validate() {
                    log::error!("设置验证失败: {}", e);
                } else {
                    self.settings = self.temp_settings.clone();
                    self.save_settings();
                    changed = true;
                }
            }
            
            if ui.button("🔄 重置设置").clicked() {
                self.temp_settings = EditorSettings::default();
                changed = true;
            }
            
            if ui.button("↩️ 恢复更改").clicked() {
                self.temp_settings = self.settings.clone();
                changed = true;
            }
        });
        
        changed
    }
    
    /// 获取当前设置
    pub fn get_settings(&self) -> &EditorSettings {
        &self.settings
    }
    
    /// 保存设置到文件
    pub fn save_settings(&self) {
        if let Err(e) = self.save_settings_to_file() {
            log::error!("Failed to save editor settings: {}", e);
        } else {
            log::info!("Editor settings saved successfully");
        }
    }

    /// 从文件加载设置
    pub fn load_settings(&mut self) {
        match self.load_settings_from_file() {
            Ok(settings) => {
                self.settings = settings;
                self.temp_settings = self.settings.clone();
                log::info!("Editor settings loaded successfully");
            }
            Err(e) => {
                log::warn!("Failed to load editor settings, using defaults: {}", e);
                self.settings = EditorSettings::default();
                self.temp_settings = self.settings.clone();
            }
        }
    }

    /// 获取配置文件路径
    fn get_config_file_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
        let config_dir = dirs::config_dir()
            .ok_or("Failed to get config directory")?;

        let app_config_dir = config_dir.join("seeu_desktop").join("ifile_editor");
        std::fs::create_dir_all(&app_config_dir)?;

        Ok(app_config_dir.join("settings.json"))
    }

    /// 保存设置到文件
    fn save_settings_to_file(&self) -> Result<(), Box<dyn std::error::Error>> {
        let config_path = Self::get_config_file_path()?;
        let json = serde_json::to_string_pretty(&self.settings)?;
        std::fs::write(config_path, json)?;
        Ok(())
    }

    /// 从文件加载设置
    fn load_settings_from_file(&self) -> Result<EditorSettings, Box<dyn std::error::Error>> {
        let config_path = Self::get_config_file_path()?;

        if !config_path.exists() {
            return Err("Config file does not exist".into());
        }

        let json = std::fs::read_to_string(config_path)?;
        let settings: EditorSettings = serde_json::from_str(&json)?;
        Ok(settings)
    }

    /// 更新上次打开的目录
    pub fn update_last_opened_directory(&mut self, path: &PathBuf) {
        self.settings.last_opened_directory = Some(path.to_string_lossy().to_string());
        self.temp_settings.last_opened_directory = self.settings.last_opened_directory.clone();
        self.save_settings();
    }

    /// 获取上次打开的目录
    pub fn get_last_opened_directory(&self) -> Option<PathBuf> {
        self.settings.last_opened_directory
            .as_ref()
            .and_then(|s| Some(PathBuf::from(s)))
            .filter(|p| p.exists() && p.is_dir())
    }

    /// 添加最近访问的目录
    pub fn add_recent_directory(&mut self, path: &PathBuf) {
        let path_str = path.to_string_lossy().to_string();

        // 移除已存在的相同路径
        self.settings.recent_directories.retain(|dir| dir.path != path_str);
        self.temp_settings.recent_directories.retain(|dir| dir.path != path_str);

        // 添加到前面
        let recent_dir = RecentDirectory {
            path: path_str,
            accessed_at: Utc::now(),
        };

        self.settings.recent_directories.insert(0, recent_dir.clone());
        self.temp_settings.recent_directories.insert(0, recent_dir);

        // 保持最大数量限制
        let max_dirs = self.settings.max_recent_directories;
        if self.settings.recent_directories.len() > max_dirs {
            self.settings.recent_directories.truncate(max_dirs);
        }
        if self.temp_settings.recent_directories.len() > max_dirs {
            self.temp_settings.recent_directories.truncate(max_dirs);
        }

        self.save_settings();
    }

    /// 添加最近访问的文件
    pub fn add_recent_file(&mut self, path: &PathBuf) {
        let path_str = path.to_string_lossy().to_string();
        let file_name = path.file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("Unknown")
            .to_string();

        // 移除已存在的相同路径
        self.settings.recent_files.retain(|file| file.path != path_str);
        self.temp_settings.recent_files.retain(|file| file.path != path_str);

        // 添加到前面
        let recent_file = RecentFile {
            path: path_str,
            name: file_name,
            accessed_at: Utc::now(),
        };

        self.settings.recent_files.insert(0, recent_file.clone());
        self.temp_settings.recent_files.insert(0, recent_file);

        // 保持最大数量限制
        let max_files = self.settings.max_recent_files;
        if self.settings.recent_files.len() > max_files {
            self.settings.recent_files.truncate(max_files);
        }
        if self.temp_settings.recent_files.len() > max_files {
            self.temp_settings.recent_files.truncate(max_files);
        }

        self.save_settings();
    }

    /// 获取最近访问的目录
    pub fn get_recent_directories(&self, limit: usize) -> Vec<RecentDirectory> {
        self.settings.recent_directories
            .iter()
            .take(limit)
            .filter(|dir| PathBuf::from(&dir.path).exists() && PathBuf::from(&dir.path).is_dir())
            .cloned()
            .collect()
    }

    /// 获取最近访问的文件
    pub fn get_recent_files(&self, limit: usize) -> Vec<RecentFile> {
        self.settings.recent_files
            .iter()
            .take(limit)
            .filter(|file| PathBuf::from(&file.path).exists() && PathBuf::from(&file.path).is_file())
            .cloned()
            .collect()
    }
}

// 注意：SettingsModule trait的实现将在主应用中通过适配器完成
