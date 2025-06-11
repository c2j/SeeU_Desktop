use eframe::egui;

use crate::ui::{
    navigation::render_navigation,
    workspace::render_workspace,
    status_bar::render_status_bar,
    right_sidebar::render_right_sidebar,
    theme::{Theme, configure_visuals},
};

use crate::services::{
    system_service::SystemService,
};

use crate::config::{StartupConfig, StartupMetrics};

// 导入模块
use inote::db_state::DbINoteState;
use isearch::ISearchState;
use aiAssist::state::AIAssistState;
use itools::IToolsState;
use iterminal::ITerminalState;

/// Main application state
pub struct SeeUApp {
    // Global state
    pub active_module: Module,
    pub show_right_sidebar: bool,

    // Search state
    pub search_query: String,
    pub global_search_results: GlobalSearchResults,

    // Module states
    pub inote_state: DbINoteState,
    pub isearch_state: ISearchState,
    pub ai_assist_state: AIAssistState,
    pub itools_state: IToolsState,
    pub iterminal_state: ITerminalState,
    pub settings_state: crate::ui::settings::SettingsState,

    // Services
    pub system_service: SystemService,

    // Theme
    pub theme: Theme,

    // Application settings
    pub app_settings: AppSettings,

    // Command channel
    slash_command_receiver: Option<std::sync::mpsc::Receiver<AppCommand>>,

    // Startup state
    pub startup_complete: bool,
    pub startup_progress: f32,
    pub startup_message: String,
    pub startup_config: StartupConfig,
    pub startup_metrics: StartupMetrics,
}

/// Application settings
#[derive(Debug, Clone)]
pub struct AppSettings {
    pub auto_startup: bool,
    pub restore_session: bool,
    pub auto_save: bool,
    pub periodic_backup: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            auto_startup: false,
            restore_session: true,
            auto_save: true,
            periodic_backup: false,
        }
    }
}

/// Application modules
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Module {
    Home,
    Terminal,
    Files,
    DataAnalysis,
    Note,
    Search,
    ITools,
    Settings,
}

/// Application commands
#[derive(Debug, Clone)]
enum AppCommand {
    Search(String),
    InsertToNote(String),
}

/// Global search results for Home page display
#[derive(Debug, Clone, Default)]
pub struct GlobalSearchResults {
    pub query: String,
    pub inote_results: Vec<INoteSearchResult>,
    pub itools_results: Vec<IToolsSearchResult>,
    pub isearch_results: Vec<ISearchResult>,
    pub has_results: bool,
}

/// iNote search result for Home display
#[derive(Debug, Clone)]
pub struct INoteSearchResult {
    pub id: String,
    pub title: String,
    pub notebook_name: String,
    pub content_preview: String,
}

/// iTools search result for Home display
#[derive(Debug, Clone)]
pub struct IToolsSearchResult {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: String,
}

/// iSearch result for Home display (simplified from isearch::SearchResult)
#[derive(Debug, Clone)]
pub struct ISearchResult {
    pub id: String,
    pub filename: String,
    pub path: String,
    pub file_type: String,
    pub content_preview: String,
}



impl SeeUApp {
    /// Create a new instance of the application
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let startup_config = StartupConfig::default();
        let mut startup_metrics = StartupMetrics::new();

        // Configure fonts
        let mut fonts = egui::FontDefinitions::default();

        // 添加嵌入式中文字体 - 文泉驿微米黑
        let wqy_font_data = include_bytes!("../assets/fonts/wqy-microhei.ttc");
        let mut wqy_font = egui::FontData::from_static(wqy_font_data);
        wqy_font.tweak.scale = 1.0;
        wqy_font.tweak.y_offset_factor = 0.0;
        fonts.font_data.insert("wqy-microhei".to_owned(), wqy_font.into());

        // 添加思源黑体 - 更好的中文显示
        let source_han_font_data = include_bytes!("../assets/fonts/SourceHanSansSC-Regular.otf");
        let mut source_han_font = egui::FontData::from_static(source_han_font_data);
        source_han_font.tweak.scale = 1.0;
        source_han_font.tweak.y_offset_factor = 0.0;
        fonts.font_data.insert("source-han-sans".to_owned(), source_han_font.into());

        // 将中文字体添加到比例字体族中
        fonts.families.get_mut(&egui::FontFamily::Proportional).unwrap()
            .insert(0, "source-han-sans".to_owned());
        fonts.families.get_mut(&egui::FontFamily::Proportional).unwrap()
            .push("wqy-microhei".to_owned());

        // 为等宽字体族配置字体
        // 使用思源黑体和文泉驿微米黑作为等宽字体
        let monospace_family = fonts.families.get_mut(&egui::FontFamily::Monospace).unwrap();
        // 保留默认的等宽字体（通常是系统自带的等宽字体）
        monospace_family.push("source-han-sans".to_owned());
        monospace_family.push("wqy-microhei".to_owned());

        // Set fonts
        cc.egui_ctx.set_fonts(fonts);
        startup_metrics.record_font_load();

        // Configure theme
        let theme = Theme::DarkModern;
        configure_visuals(&cc.egui_ctx, theme);

        // Create states
        let mut inote_state = DbINoteState::default();
        let mut isearch_state = ISearchState::default();
        let ai_assist_state = aiAssist::initialize();
        let mut itools_state = itools::initialize();
        let iterminal_state = iterminal::initialize();

        // Initialize states based on configuration
        inote_state.initialize();
        startup_metrics.record_database_init();

        isearch_state.initialize();
        startup_metrics.record_search_init();

        itools_state.initialize();
        startup_metrics.record_plugin_init();

        // Create app instance
        let mut app = Self {
            active_module: Module::Home,
            show_right_sidebar: false,
            search_query: String::new(),
            global_search_results: GlobalSearchResults::default(),
            inote_state,
            isearch_state,
            ai_assist_state,
            itools_state,
            iterminal_state,
            settings_state: crate::ui::settings::SettingsState::default(),
            system_service: SystemService::new(),
            theme,
            app_settings: AppSettings::default(),
            slash_command_receiver: None,
            startup_complete: !startup_config.show_startup_progress,
            startup_progress: 0.0,
            startup_message: "正在初始化应用程序...".to_string(),
            startup_config,
            startup_metrics,
        };

        // 设置命令通道
        let tx = app.setup_command_channel();

        // 设置回调函数
        app.setup_callbacks(tx);

        // 启动后台任务来更新启动进度
        app.start_startup_progress_tracker();

        // Record total startup time and log metrics
        app.startup_metrics.record_total();
        app.startup_metrics.log_metrics();

        // Load application settings
        app.load_app_settings();

        app
    }

    /// 设置命令通道
    fn setup_command_channel(&mut self) -> std::sync::mpsc::Sender<AppCommand> {
        // 创建一个新的通道
        let (tx, rx) = std::sync::mpsc::channel();

        // 替换现有的接收端
        self.slash_command_receiver = Some(rx);

        // 返回发送端
        tx
    }

    /// 设置所有回调函数
    fn setup_callbacks(&mut self, tx: std::sync::mpsc::Sender<AppCommand>) {
        // 设置斜杠命令回调
        let tx_clone = tx.clone();
        aiAssist::set_slash_command_callback(&mut self.ai_assist_state, move |cmd| {
            // 发送命令
            match cmd {
                aiAssist::SlashCommand::Search(query) => {
                    let _ = tx_clone.send(AppCommand::Search(query.clone()));
                }
            }
        });

        // 设置插入到笔记回调
        let tx_clone = tx.clone();
        aiAssist::set_insert_to_note_callback(&mut self.ai_assist_state, move |content| {
            // 发送插入笔记的命令
            let _ = tx_clone.send(AppCommand::InsertToNote(content));
        });
    }

    /// 启动进度跟踪器
    fn start_startup_progress_tracker(&mut self) {
        // 简化启动进度跟踪，避免复杂的异步操作
        // 设置初始完成状态
        self.startup_complete = true;
        self.startup_progress = 1.0;
        self.startup_message = "初始化完成".to_string();
    }

    /// Process any pending slash commands
    fn process_slash_commands(&mut self) {
        if let Some(rx) = &self.slash_command_receiver {
            // Try to receive all pending commands
            while let Ok(cmd) = rx.try_recv() {
                match cmd {
                    AppCommand::Search(query) => {
                        // Set the search query
                        self.search_query = query.clone();

                        // Switch to search module
                        self.active_module = Module::Search;

                        // Execute the search
                        self.isearch_state.search_query = query.clone();
                        self.isearch_state.search();

                        // 格式化搜索结果，用于 @search 引用
                        let formatted_results = self.format_search_results_for_ai();

                        // Add search reference to AI chat
                        aiAssist::add_search_reference(
                            &mut self.ai_assist_state,
                            &query,
                            self.isearch_state.search_stats.total_results
                        );

                        // Set search results for @search references
                        aiAssist::set_search_results(
                            &mut self.ai_assist_state,
                            formatted_results
                        );
                    },
                    AppCommand::InsertToNote(content) => {
                        // 检查当前是否处于笔记视图
                        if self.active_module == Module::Note {
                            // 如果当前有打开的笔记，将内容插入到笔记编辑器
                            if let Some(note_id) = self.inote_state.current_note.clone() {
                                // 获取当前笔记内容
                                let mut note_content = self.inote_state.note_content.clone();

                                // 在内容末尾添加AI回复
                                if !note_content.is_empty() {
                                    note_content.push_str("\n\n");
                                }
                                note_content.push_str(&content);

                                // 更新笔记内容
                                self.inote_state.note_content = note_content;

                                // 标记笔记为已修改
                                self.inote_state.check_note_modified();

                                log::info!("已将AI回复插入到笔记: {}", note_id);
                            } else {
                                log::warn!("无法插入AI回复：当前没有打开的笔记");
                            }
                        } else {
                            log::warn!("无法插入AI回复：当前不在笔记视图");
                        }
                    }
                }
            }
        }
    }



    /// 格式化搜索结果，用于 AI 助手中的 @search 引用
    fn format_search_results_for_ai(&self) -> String {
        if self.isearch_state.search_results.is_empty() {
            return "未找到匹配结果".to_string();
        }

        // 限制结果数量，避免超出上下文窗口
        let max_results = 5;
        let results_to_show = std::cmp::min(self.isearch_state.search_results.len(), max_results);

        let mut formatted = format!(
            "找到 {} 个结果（显示前 {} 个）：\n\n",
            self.isearch_state.search_stats.total_results,
            results_to_show
        );

        // 添加每个搜索结果
        for (i, result) in self.isearch_state.search_results.iter().take(results_to_show).enumerate() {
            formatted.push_str(&format!(
                "{}. 文件: {}\n   路径: {}\n   类型: {}\n   内容预览: {}\n\n",
                i + 1,
                result.filename,
                result.path,
                result.file_type,
                result.content_preview.replace("\n", " ")
            ));
        }

        // 如果有更多结果，添加提示
        if self.isearch_state.search_results.len() > max_results {
            formatted.push_str(&format!(
                "... 还有 {} 个结果未显示",
                self.isearch_state.search_results.len() - max_results
            ));
        }

        formatted
    }

    /// Render the search bar
    fn render_search_bar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("🔍");
            let response = ui.add(
                egui::TextEdit::singleline(&mut self.search_query)
                    .hint_text("全局搜索（支持 filetype:pdf, +必须, \"精确短语\"）...")
                    .desired_width(ui.available_width())
            );

            // Handle search input
            if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                if !self.search_query.is_empty() {
                    // Perform global search
                    self.perform_global_search();

                    // Switch to Home to show results
                    self.active_module = Module::Home;

                    log::info!("Global search query: {}", self.search_query);
                }
            }
        });
    }

    /// Perform global search across all modules
    fn perform_global_search(&mut self) {
        let query = self.search_query.clone();

        // Clear previous results
        self.global_search_results = GlobalSearchResults {
            query: query.clone(),
            inote_results: Vec::new(),
            itools_results: Vec::new(),
            isearch_results: Vec::new(),
            has_results: false,
        };

        // Search in iNote
        self.search_inote(&query);

        // Search in iTools
        self.search_itools(&query);

        // Search in iSearch
        self.search_isearch(&query);

        // Update has_results flag
        self.global_search_results.has_results =
            !self.global_search_results.inote_results.is_empty() ||
            !self.global_search_results.itools_results.is_empty() ||
            !self.global_search_results.isearch_results.is_empty();

        log::info!("Global search completed. iNote: {}, iTools: {}, iSearch: {}",
            self.global_search_results.inote_results.len(),
            self.global_search_results.itools_results.len(),
            self.global_search_results.isearch_results.len());
    }

    /// Search in iNote module
    fn search_inote(&mut self, query: &str) {
        // Set search query and perform search
        self.inote_state.search_query = query.to_string();
        self.inote_state.search_notes();

        // Convert results to global search format
        let search_results = self.inote_state.get_search_result_notes();
        self.global_search_results.inote_results = search_results.iter()
            .take(5) // Limit to 5 results for Home display
            .map(|note| {
                // Find notebook name
                let notebook_name = self.inote_state.notebooks.iter()
                    .find(|nb| nb.note_ids.contains(&note.id))
                    .map(|nb| nb.name.clone())
                    .unwrap_or_else(|| "未知笔记本".to_string());

                // Create content preview (first 100 characters, safe for UTF-8)
                let content_preview = if note.content.chars().count() > 100 {
                    let truncated: String = note.content.chars().take(100).collect();
                    format!("{}...", truncated)
                } else {
                    note.content.clone()
                };

                INoteSearchResult {
                    id: note.id.clone(),
                    title: note.title.clone(),
                    notebook_name,
                    content_preview,
                }
            })
            .collect();
    }

    /// Search in iTools module
    fn search_itools(&mut self, query: &str) {
        // Set search query in iTools
        self.itools_state.ui_state.search_query = query.to_string();

        // Get marketplace and search plugins
        let marketplace_filters = itools::plugins::marketplace::MarketplaceFilters {
            query: query.to_string(),
            category: None,
            role: None,
            permission_level: None,
            verified_only: false,
            featured_only: false,
            sort_by: itools::plugins::marketplace::SortBy::Relevance,
        };

        let plugins = self.itools_state.plugin_manager.get_marketplace()
            .search_plugins(&marketplace_filters);

        // Convert results to global search format
        self.global_search_results.itools_results = plugins.iter()
            .take(5) // Limit to 5 results for Home display
            .map(|plugin| {
                let category = plugin.plugin.metadata.categories.first()
                    .cloned()
                    .unwrap_or_else(|| "未分类".to_string());

                IToolsSearchResult {
                    id: plugin.plugin.id.to_string(),
                    name: plugin.plugin.metadata.name.clone(),
                    description: plugin.plugin.metadata.description.clone(),
                    category,
                }
            })
            .collect();
    }

    /// Search in iSearch module
    fn search_isearch(&mut self, query: &str) {
        // Set search query and perform search
        self.isearch_state.search_query = query.to_string();
        self.isearch_state.search();

        // Convert results to global search format
        self.global_search_results.isearch_results = self.isearch_state.search_results.iter()
            .take(5) // Limit to 5 results for Home display
            .map(|result| {
                ISearchResult {
                    id: result.id.clone(),
                    filename: result.filename.clone(),
                    path: result.path.clone(),
                    file_type: result.file_type.clone(),
                    content_preview: result.content_preview.clone(),
                }
            })
            .collect();
    }

    /// Change the application theme
    pub fn set_theme(&mut self, ctx: &egui::Context, new_theme: Theme) {
        self.theme = new_theme;
        configure_visuals(ctx, new_theme);
        log::info!("Theme changed to: {}", new_theme.display_name());
    }

    /// Get current theme
    pub fn get_theme(&self) -> Theme {
        self.theme
    }

    /// Save all application settings
    fn save_all_settings(&mut self) {
        log::info!("Saving all application settings before exit...");

        // Save AI assistant settings
        if let Err(err) = aiAssist::save_settings(&self.ai_assist_state) {
            log::error!("Failed to save AI assistant settings: {}", err);
        }

        // Save search module settings
        self.isearch_state.save_search_options();
        self.isearch_state.save_indexed_directories();

        // Save terminal settings
        if let Err(err) = self.iterminal_state.save_config() {
            log::error!("Failed to save terminal settings: {}", err);
        }

        // Save note module settings
        if let Err(err) = inote::save_settings(&self.inote_state) {
            log::error!("Failed to save note settings: {}", err);
        }

        // Save application-level settings
        if let Err(err) = self.save_app_settings() {
            log::error!("Failed to save application settings: {}", err);
        }

        log::info!("All settings saved successfully");
    }

    /// Save application-level settings
    pub fn save_app_settings(&self) -> Result<(), Box<dyn std::error::Error>> {
        use std::fs;
        use serde_json;

        let base_path = dirs::config_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
        let config_dir = base_path.join("seeu_desktop");
        let config_path = config_dir.join("app_settings.json");

        fs::create_dir_all(&config_dir)?;

        let settings = serde_json::json!({
            "active_module": format!("{:?}", self.active_module),
            "show_right_sidebar": self.show_right_sidebar,
            "theme": format!("{:?}", self.theme),
            "auto_startup": self.app_settings.auto_startup,
            "restore_session": self.app_settings.restore_session,
            "auto_save": self.app_settings.auto_save,
            "periodic_backup": self.app_settings.periodic_backup
        });

        let json = serde_json::to_string_pretty(&settings)?;
        fs::write(config_path, json)?;

        Ok(())
    }

    /// Load application-level settings
    fn load_app_settings(&mut self) {
        use std::fs;
        use serde_json;

        let base_path = dirs::config_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
        let config_path = base_path.join("seeu_desktop").join("app_settings.json");

        if let Ok(content) = fs::read_to_string(config_path) {
            if let Ok(settings) = serde_json::from_str::<serde_json::Value>(&content) {
                // Load active module
                if let Some(module_str) = settings.get("active_module").and_then(|v| v.as_str()) {
                    match module_str {
                        "Home" => self.active_module = Module::Home,
                        "Terminal" => self.active_module = Module::Terminal,
                        "Files" => self.active_module = Module::Files,
                        "DataAnalysis" => self.active_module = Module::DataAnalysis,
                        "Note" => self.active_module = Module::Note,
                        "Search" => self.active_module = Module::Search,
                        "ITools" => self.active_module = Module::ITools,
                        "Settings" => self.active_module = Module::Settings,
                        _ => {}
                    }
                }

                // Load sidebar state
                if let Some(show_sidebar) = settings.get("show_right_sidebar").and_then(|v| v.as_bool()) {
                    self.show_right_sidebar = show_sidebar;
                }

                // Load theme
                if let Some(theme_str) = settings.get("theme").and_then(|v| v.as_str()) {
                    match theme_str {
                        "DarkModern" => self.theme = Theme::DarkModern,
                        _ => {}
                    }
                }

                // Load app settings
                if let Some(value) = settings.get("auto_startup").and_then(|v| v.as_bool()) {
                    self.app_settings.auto_startup = value;
                }
                if let Some(value) = settings.get("restore_session").and_then(|v| v.as_bool()) {
                    self.app_settings.restore_session = value;
                }
                if let Some(value) = settings.get("auto_save").and_then(|v| v.as_bool()) {
                    self.app_settings.auto_save = value;
                }
                if let Some(value) = settings.get("periodic_backup").and_then(|v| v.as_bool()) {
                    self.app_settings.periodic_backup = value;
                }
            }
        }
    }

    /// Render startup screen
    fn render_startup_screen(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(ui.available_height() * 0.3);

                // App logo/title
                ui.heading("SeeU Desktop");
                ui.add_space(20.0);

                // Progress bar
                let progress_bar = egui::ProgressBar::new(self.startup_progress)
                    .desired_width(300.0)
                    .text(&self.startup_message);
                ui.add(progress_bar);

                ui.add_space(10.0);
                ui.label("正在加载模块和数据...");
            });
        });

        // Request repaint to update progress
        ctx.request_repaint();
    }
}

impl eframe::App for SeeUApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Show startup screen if not complete
        if !self.startup_complete {
            self.render_startup_screen(ctx);
            return;
        }

        // Refresh system information for accurate CPU and memory usage
        self.system_service.refresh();

        // Process any pending slash commands
        self.process_slash_commands();

        // Check if search module wants to navigate to settings
        if self.isearch_state.navigate_to_settings {
            self.active_module = Module::Settings;
            self.isearch_state.navigate_to_settings = false;
            // Set settings to search category
            self.settings_state.current_category = crate::ui::settings::SettingsCategory::Search;
        }

        // Top panel - search bar
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            self.render_search_bar(ui);
        });

        // Bottom status bar - 放在最前面确保它总是占据全宽
        let save_status = self.inote_state.save_status.clone();
        let system_service = &self.system_service;

        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            render_status_bar(
                ui,
                system_service,
                &mut self.show_right_sidebar,
                self.active_module,
                save_status
            );
        });

        // Left navigation panel
        egui::SidePanel::left("nav_panel")
            .resizable(false)
            .default_width(48.0)
            .show(ctx, |ui| {
                render_navigation(ui, &mut self.active_module);
            });

        // 保存当前模块，供后续使用
        let current_module = self.active_module.clone();

        // 更新 AI 助手的状态 - 检查当前是否处于笔记视图且有打开的笔记
        let can_insert_to_note = self.active_module == Module::Note && self.inote_state.current_note.is_some();
        aiAssist::update_can_insert_to_note(&mut self.ai_assist_state, can_insert_to_note);

        // 更新 iTools 模块
        itools::update_itools(&mut self.itools_state);

        // 更新 iTerminal 模块
        iterminal::update_iterminal(&mut self.iterminal_state);

        // 定期保存笔记数据（每5秒检查一次）
        if ctx.input(|i| i.time) % 5.0 < 0.1 {
            if self.inote_state.save_status == inote::db_state::SaveStatus::Modified {
                self.inote_state.auto_save_if_modified();
            }
        }

        // 让 egui 自动处理面板高度

        // 右侧边栏 (如果启用) - 放在主面板之前
        if self.show_right_sidebar {
            egui::SidePanel::right("right_sidebar")
                .resizable(true)
                .default_width(300.0)
                .show(ctx, |ui| {
                    render_right_sidebar(ui, self);
                });
        }

        // 主面板 - 放在最后，确保它填充剩余空间
        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(ctx.style().visuals.window_fill))
            .show(ctx, |ui| {
                // 渲染工作区，让 egui 自动处理高度
                render_workspace(ui, &current_module, self);
            });
    }

    /// Save state when the app is about to close
    fn save(&mut self, _storage: &mut dyn eframe::Storage) {
        // Save all settings before exit
        self.save_all_settings();
    }
}