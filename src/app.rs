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

/// Window state for saving and restoring
#[derive(Debug, Clone)]
pub struct WindowState {
    pub width: f32,
    pub height: f32,
    pub x: f32,
    pub y: f32,
    pub maximized: bool,
}

impl Default for WindowState {
    fn default() -> Self {
        Self {
            width: 1280.0,
            height: 720.0,
            x: 100.0,
            y: 100.0,
            maximized: false,
        }
    }
}

/// Application settings
#[derive(Debug, Clone)]
pub struct AppSettings {
    pub auto_startup: bool,
    pub restore_session: bool,
    pub auto_save: bool,
    pub periodic_backup: bool,
    // Font settings
    pub font_size: f32,
    pub font_family: String,
    // UI scale settings
    pub ui_scale: f32,
    // Window state settings
    pub window_state: WindowState,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            auto_startup: false,
            restore_session: true,
            auto_save: true,
            periodic_backup: false,
            font_size: 14.0,
            font_family: "Default".to_string(),
            ui_scale: 1.0,
            window_state: WindowState::default(),
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
        // Note: We'll initialize these asynchronously to avoid blocking the UI
        // For now, just do minimal synchronous initialization
        startup_metrics.record_database_init();
        startup_metrics.record_search_init();
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
            startup_complete: false, // Always show startup progress to avoid blocking
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

        // Apply loaded settings
        // First set the egui built-in theme
        let egui_theme = match app.theme {
            Theme::DarkModern | Theme::Dark => egui::Theme::Dark,
            Theme::LightModern | Theme::Light => egui::Theme::Light,
        };
        cc.egui_ctx.set_theme(egui_theme);

        // Then apply our custom visuals
        configure_visuals(&cc.egui_ctx, app.theme);

        // Apply UI scale using native pixels_per_point as baseline
        let native_pixels_per_point = cc.egui_ctx.native_pixels_per_point().unwrap_or(1.0);
        cc.egui_ctx.set_pixels_per_point(native_pixels_per_point * app.app_settings.ui_scale);

        app.update_fonts(&cc.egui_ctx);

        // TEMPORARILY DISABLED: Adjust window state for DPI scaling after egui context is available
        // app.adjust_window_state_for_dpi(&cc.egui_ctx);

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
        // 启动异步初始化任务
        self.startup_complete = false;
        self.startup_progress = 0.0;
        self.startup_message = "正在初始化应用程序...".to_string();

        // 记录启动时间
        self.startup_metrics.start_time = std::time::Instant::now();

        // 启动异步初始化
        self.start_async_initialization();
    }

    /// 启动异步初始化
    fn start_async_initialization(&mut self) {
        // 启动真正的异步初始化
        let inote_storage = self.inote_state.storage.clone();
        let isearch_indexer = self.isearch_state.get_indexer();

        // 启动后台线程进行数据库初始化
        std::thread::spawn(move || {
            log::info!("Starting background database initialization...");

            // 初始化笔记数据库
            if let Ok(mut storage) = inote_storage.lock() {
                if let Err(err) = storage.initialize_async() {
                    log::error!("Failed to initialize note database: {}", err);
                }
            }

            // 初始化搜索索引
            if let Ok(indexer_lock) = isearch_indexer.lock() {
                if let Err(err) = indexer_lock.initialize_index() {
                    log::error!("Failed to initialize search index: {}", err);
                }
            }

            log::info!("Background initialization completed");
        });

        // 启动进度更新定时器
        self.start_progress_timer();
    }

    /// 启动进度更新定时器
    fn start_progress_timer(&mut self) {
        // 简单的进度模拟，实际应用中可以通过通道接收真实进度
        let start_time = std::time::Instant::now();

        // 设置一个合理的初始化时间（比如2秒）
        let target_duration = std::time::Duration::from_secs(2);

        // 记录开始时间用于进度计算
        self.startup_metrics.start_time = start_time;
    }

    /// 更新启动进度
    fn update_startup_progress(&mut self) {
        let elapsed = self.startup_metrics.start_time.elapsed();
        let target_duration = std::time::Duration::from_secs(2);

        // 计算进度（0.0 到 1.0）
        let progress = (elapsed.as_secs_f32() / target_duration.as_secs_f32()).min(1.0);
        self.startup_progress = progress;

        // 更新进度消息
        if progress < 0.3 {
            self.startup_message = "正在初始化笔记模块...".to_string();
        } else if progress < 0.6 {
            self.startup_message = "正在初始化搜索索引...".to_string();
        } else if progress < 0.9 {
            self.startup_message = "正在启动文件监控...".to_string();
        } else {
            self.startup_message = "初始化完成".to_string();
        }

        // 当进度达到100%时，完成启动
        if progress >= 1.0 {
            self.startup_complete = true;

            // 现在安全地初始化各个模块的UI状态
            self.finalize_module_initialization();
        }
    }

    /// 完成模块初始化
    fn finalize_module_initialization(&mut self) {
        // 只进行轻量级的UI初始化，避免阻塞操作

        // 初始化搜索模块的UI状态（轻量级操作）
        self.isearch_state.load_indexed_directories();
        self.isearch_state.load_search_options();

        // 初始化工具模块（轻量级操作）
        self.itools_state.initialize();

        // 启动笔记模块的后台数据加载
        self.start_background_data_loading();

        log::info!("Module initialization completed");
    }

    /// 启动后台数据加载
    fn start_background_data_loading(&mut self) {
        let inote_storage = self.inote_state.storage.clone();

        std::thread::spawn(move || {
            log::info!("Starting background data loading...");

            // 等待数据库初始化完成
            std::thread::sleep(std::time::Duration::from_millis(100));

            // 在后台加载数据
            if let Ok(storage) = inote_storage.lock() {
                if !storage.is_placeholder() {
                    // 数据库已经初始化，可以安全地加载数据
                    log::info!("Database is ready, loading data in background");
                } else {
                    log::warn!("Database is still placeholder, skipping data loading");
                }
            }

            log::info!("Background data loading completed");
        });
    }

    /// Adjust window state for DPI scaling
    /// This converts saved physical pixel coordinates to logical coordinates
    fn adjust_window_state_for_dpi(&mut self, ctx: &egui::Context) {
        let pixels_per_point = ctx.pixels_per_point();

        // If DPI scaling is not 1.0, we need to convert saved physical coordinates to logical coordinates
        if (pixels_per_point - 1.0).abs() > 0.01 {
            log::info!("Adjusting window state for DPI scaling: {:.2}", pixels_per_point);

            // Convert physical pixels back to logical coordinates for window creation
            let original_width = self.app_settings.window_state.width;
            let original_height = self.app_settings.window_state.height;
            let original_x = self.app_settings.window_state.x;
            let original_y = self.app_settings.window_state.y;

            // Convert from physical pixels to logical coordinates
            self.app_settings.window_state.width = original_width / pixels_per_point;
            self.app_settings.window_state.height = original_height / pixels_per_point;
            self.app_settings.window_state.x = original_x / pixels_per_point;
            self.app_settings.window_state.y = original_y / pixels_per_point;

            log::info!("Window state adjusted: {}x{} -> {}x{}, pos ({},{}) -> ({},{})",
                      original_width, original_height,
                      self.app_settings.window_state.width, self.app_settings.window_state.height,
                      original_x, original_y,
                      self.app_settings.window_state.x, self.app_settings.window_state.y);
        } else {
            log::info!("No DPI adjustment needed (DPI scale: {:.2})", pixels_per_point);
        }
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
    /// 只返回第一条搜索结果的详细内容
    fn format_search_results_for_ai(&self) -> String {
        if self.isearch_state.search_results.is_empty() {
            return "未找到匹配结果".to_string();
        }

        // 只获取第一条搜索结果
        let first_result = &self.isearch_state.search_results[0];

        // 尝试获取更完整的文件内容
        let detailed_content = self.get_detailed_file_content(&first_result.path, &first_result.file_type);

        let formatted = format!(
            "搜索结果（第1条，共{}条）：\n\n文件名: {}\n路径: {}\n文件类型: {}\n修改时间: {}\n文件大小: {} bytes\n\n内容:\n{}",
            self.isearch_state.search_stats.total_results,
            first_result.filename,
            first_result.path,
            first_result.file_type,
            first_result.modified.format("%Y-%m-%d %H:%M:%S"),
            first_result.size_bytes,
            detailed_content
        );

        formatted
    }

    /// 获取文件的详细内容，用于@search引用
    fn get_detailed_file_content(&self, file_path: &str, file_type: &str) -> String {
        // 对于可预览的文件类型，尝试读取更多内容
        if self.is_text_file(file_type) {
            match std::fs::read_to_string(file_path) {
                Ok(content) => {
                    // 限制内容长度，避免超出上下文窗口（约2000字符）
                    if content.len() > 2000 {
                        let truncated: String = content.chars().take(2000).collect();
                        format!("{}...\n\n[内容已截断，完整内容请查看原文件]", truncated)
                    } else {
                        content
                    }
                },
                Err(_) => {
                    // 如果无法读取文件，返回原有的预览内容
                    format!("无法读取文件内容，预览: {}",
                        self.isearch_state.search_results[0].content_preview)
                }
            }
        } else {
            // 对于非文本文件，返回文件类型说明和预览
            format!("这是一个 {} 文件，无法显示文本内容。\n预览信息: {}",
                file_type,
                self.isearch_state.search_results[0].content_preview)
        }
    }

    /// 判断是否为文本文件
    fn is_text_file(&self, file_type: &str) -> bool {
        matches!(file_type.to_lowercase().as_str(),
            "txt" | "md" | "markdown" | "rs" | "py" | "js" | "ts" | "html" | "css" |
            "json" | "xml" | "yaml" | "yml" | "toml" | "ini" | "cfg" | "conf" |
            "log" | "csv" | "sql" | "sh" | "bat" | "ps1" | "php" | "java" | "c" |
            "cpp" | "h" | "hpp" | "go" | "rb" | "pl" | "swift" | "kt" | "scala" |
            "clj" | "hs" | "elm" | "dart" | "vue" | "jsx" | "tsx" | "svelte")
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

        // First, set the egui built-in theme to ensure proper base styling
        let egui_theme = match new_theme {
            Theme::DarkModern | Theme::Dark => egui::Theme::Dark,
            Theme::LightModern | Theme::Light => egui::Theme::Light,
        };
        ctx.set_theme(egui_theme);

        // Then apply our custom visuals on top
        configure_visuals(ctx, new_theme);

        // Force UI to repaint to ensure all elements update
        ctx.request_repaint();

        // Save settings immediately after theme change
        if let Err(err) = self.save_app_settings() {
            log::error!("Failed to save theme settings: {}", err);
        }

        log::info!("Theme changed to: {}", new_theme.display_name());
    }

    /// Get current theme
    pub fn get_theme(&self) -> Theme {
        self.theme
    }

    /// Set font size
    pub fn set_font_size(&mut self, ctx: &egui::Context, font_size: f32) {
        self.app_settings.font_size = font_size.clamp(8.0, 32.0);
        self.update_fonts(ctx);

        // Save settings immediately
        if let Err(err) = self.save_app_settings() {
            log::error!("Failed to save font settings: {}", err);
        }

        log::info!("Font size changed to: {}", self.app_settings.font_size);
    }

    /// Set font family
    pub fn set_font_family(&mut self, ctx: &egui::Context, font_family: String) {
        self.app_settings.font_family = font_family;
        self.update_fonts(ctx);

        // Save settings immediately
        if let Err(err) = self.save_app_settings() {
            log::error!("Failed to save font family settings: {}", err);
        }

        log::info!("Font family changed to: {}", self.app_settings.font_family);
    }

    /// Set UI scale
    pub fn set_ui_scale(&mut self, ctx: &egui::Context, ui_scale: f32) {
        self.app_settings.ui_scale = ui_scale.clamp(0.5, 3.0);

        // Get the native pixels_per_point as baseline
        let native_pixels_per_point = ctx.native_pixels_per_point().unwrap_or(1.0);

        // Calculate the actual pixels_per_point based on our scale
        let target_pixels_per_point = native_pixels_per_point * self.app_settings.ui_scale;

        // Apply the scaling
        ctx.set_pixels_per_point(target_pixels_per_point);

        // Force UI to repaint to ensure the scale change takes effect
        ctx.request_repaint();

        // Save settings immediately
        if let Err(err) = self.save_app_settings() {
            log::error!("Failed to save UI scale settings: {}", err);
        }

        log::info!("UI scale changed to: {}x (native: {:.2}, target: {:.2})",
                  self.app_settings.ui_scale, native_pixels_per_point, target_pixels_per_point);
    }

    /// Reset appearance settings to default
    pub fn reset_appearance_to_default(&mut self, ctx: &egui::Context) {
        // Reset theme to default
        self.theme = Theme::DarkModern;

        // Set egui built-in theme
        let egui_theme = match self.theme {
            Theme::DarkModern | Theme::Dark => egui::Theme::Dark,
            Theme::LightModern | Theme::Light => egui::Theme::Light,
        };
        ctx.set_theme(egui_theme);

        // Apply custom visuals
        configure_visuals(ctx, self.theme);

        // Reset font settings to default
        self.app_settings.font_size = 14.0;
        self.app_settings.font_family = "Default".to_string();

        // Reset UI scale to default
        self.app_settings.ui_scale = 1.0;
        let native_pixels_per_point = ctx.native_pixels_per_point().unwrap_or(1.0);
        ctx.set_pixels_per_point(native_pixels_per_point * self.app_settings.ui_scale);

        // Update fonts with default settings
        self.update_fonts(ctx);

        // Force UI to repaint
        ctx.request_repaint();

        // Save settings immediately
        if let Err(err) = self.save_app_settings() {
            log::error!("Failed to save default appearance settings: {}", err);
        }

        log::info!("Appearance settings reset to default");
    }

    /// Update fonts based on current settings
    fn update_fonts(&self, ctx: &egui::Context) {
        let mut fonts = egui::FontDefinitions::default();

        // 添加嵌入式中文字体 - 文泉驿微米黑
        let wqy_font_data = include_bytes!("../assets/fonts/wqy-microhei.ttc");
        let mut wqy_font = egui::FontData::from_static(wqy_font_data);
        wqy_font.tweak.scale = self.app_settings.font_size / 14.0; // Scale relative to default
        wqy_font.tweak.y_offset_factor = 0.0;
        fonts.font_data.insert("wqy-microhei".to_owned(), wqy_font.into());

        // 添加思源黑体 - 更好的中文显示
        let source_han_font_data = include_bytes!("../assets/fonts/SourceHanSansSC-Regular.otf");
        let mut source_han_font = egui::FontData::from_static(source_han_font_data);
        source_han_font.tweak.scale = self.app_settings.font_size / 14.0; // Scale relative to default
        source_han_font.tweak.y_offset_factor = 0.0;
        fonts.font_data.insert("source-han-sans".to_owned(), source_han_font.into());

        // 将中文字体添加到比例字体族中
        fonts.families.get_mut(&egui::FontFamily::Proportional).unwrap()
            .insert(0, "source-han-sans".to_owned());
        fonts.families.get_mut(&egui::FontFamily::Proportional).unwrap()
            .push("wqy-microhei".to_owned());

        // 为等宽字体族配置字体
        let monospace_family = fonts.families.get_mut(&egui::FontFamily::Monospace).unwrap();
        monospace_family.push("source-han-sans".to_owned());
        monospace_family.push("wqy-microhei".to_owned());

        // Set fonts
        ctx.set_fonts(fonts);
        ctx.request_repaint();
    }

    /// Update window state from current frame
    fn update_window_state(&mut self, ctx: &egui::Context) {
        // Only update window state after startup is complete to avoid capturing transient states
        if !self.startup_complete {
            return;
        }

        // Wait 2 seconds after startup before tracking window state changes
        // This allows the window to stabilize after initial creation
        if self.startup_metrics.start_time.elapsed().as_secs() < 2 {
            return;
        }

        // Get viewport info from input state
        ctx.input(|i| {
            let viewport_info = i.viewport();

            // Get the pixels per point (DPI scaling factor)
            let pixels_per_point = ctx.pixels_per_point();

            // Use inner_rect for both size and position to be consistent with startup
            // This ensures we save the same type of measurements we use when restoring
            if let Some(inner_rect) = viewport_info.inner_rect {
                let old_width = self.app_settings.window_state.width;
                let old_height = self.app_settings.window_state.height;
                let old_x = self.app_settings.window_state.x;
                let old_y = self.app_settings.window_state.y;

                // Convert egui logical coordinates to physical pixels for consistent storage
                // egui uses logical coordinates, but we want to store physical coordinates
                self.app_settings.window_state.width = inner_rect.width() * pixels_per_point;
                self.app_settings.window_state.height = inner_rect.height() * pixels_per_point;

                // For position, try to use outer_rect if available for better accuracy
                if let Some(outer_rect) = viewport_info.outer_rect {
                    self.app_settings.window_state.x = outer_rect.min.x * pixels_per_point;
                    self.app_settings.window_state.y = outer_rect.min.y * pixels_per_point;
                } else {
                    self.app_settings.window_state.x = inner_rect.min.x * pixels_per_point;
                    self.app_settings.window_state.y = inner_rect.min.y * pixels_per_point;
                }

                // Log significant window changes for debugging
                let size_changed = (old_width - self.app_settings.window_state.width).abs() > 5.0 ||
                                 (old_height - self.app_settings.window_state.height).abs() > 5.0;
                let pos_changed = (old_x - self.app_settings.window_state.x).abs() > 5.0 ||
                                (old_y - self.app_settings.window_state.y).abs() > 5.0;

                if size_changed || pos_changed {
                    log::info!("Window state changed: size {}x{} -> {}x{}, pos ({},{}) -> ({},{}) [DPI: {:.2}, logical: {}x{} at ({},{}))]",
                              old_width, old_height, self.app_settings.window_state.width, self.app_settings.window_state.height,
                              old_x, old_y, self.app_settings.window_state.x, self.app_settings.window_state.y, pixels_per_point,
                              inner_rect.width(), inner_rect.height(),
                              if let Some(outer_rect) = viewport_info.outer_rect { outer_rect.min.x } else { inner_rect.min.x },
                              if let Some(outer_rect) = viewport_info.outer_rect { outer_rect.min.y } else { inner_rect.min.y });
                }
            } else {
                log::warn!("No inner_rect available for window state update");
            }

            // Get window maximized state
            if let Some(maximized) = viewport_info.maximized {
                if self.app_settings.window_state.maximized != maximized {
                    log::info!("Window maximized state changed: {} -> {}",
                              self.app_settings.window_state.maximized, maximized);
                    self.app_settings.window_state.maximized = maximized;
                }
            }
        });
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
            // Note: We don't save active_module to ensure users always start from Home
            "show_right_sidebar": self.show_right_sidebar,
            "theme": self.theme.to_string(),
            "auto_startup": self.app_settings.auto_startup,
            "restore_session": self.app_settings.restore_session,
            "auto_save": self.app_settings.auto_save,
            "periodic_backup": self.app_settings.periodic_backup,
            "font_size": self.app_settings.font_size,
            "font_family": self.app_settings.font_family,
            "ui_scale": self.app_settings.ui_scale,
            "window_state": {
                "width": self.app_settings.window_state.width,
                "height": self.app_settings.window_state.height,
                "x": self.app_settings.window_state.x,
                "y": self.app_settings.window_state.y,
                "maximized": self.app_settings.window_state.maximized
            }
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
                // Always start from Home page for better user experience
                // Note: We don't restore the active module to ensure users always start from the main page
                self.active_module = Module::Home;

                // Load sidebar state
                if let Some(show_sidebar) = settings.get("show_right_sidebar").and_then(|v| v.as_bool()) {
                    self.show_right_sidebar = show_sidebar;
                }

                // Load theme
                if let Some(theme_str) = settings.get("theme").and_then(|v| v.as_str()) {
                    self.theme = Theme::from_string(theme_str);
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
                if let Some(value) = settings.get("font_size").and_then(|v| v.as_f64()) {
                    self.app_settings.font_size = value as f32;
                }
                if let Some(value) = settings.get("font_family").and_then(|v| v.as_str()) {
                    self.app_settings.font_family = value.to_string();
                }
                if let Some(value) = settings.get("ui_scale").and_then(|v| v.as_f64()) {
                    self.app_settings.ui_scale = value as f32;
                }
                if let Some(value) = settings.get("periodic_backup").and_then(|v| v.as_bool()) {
                    self.app_settings.periodic_backup = value;
                }

                // Load window state
                if let Some(window_state) = settings.get("window_state") {
                    if let Some(value) = window_state.get("width").and_then(|v| v.as_f64()) {
                        self.app_settings.window_state.width = value as f32;
                    }
                    if let Some(value) = window_state.get("height").and_then(|v| v.as_f64()) {
                        self.app_settings.window_state.height = value as f32;
                    }
                    if let Some(value) = window_state.get("x").and_then(|v| v.as_f64()) {
                        self.app_settings.window_state.x = value as f32;
                    }
                    if let Some(value) = window_state.get("y").and_then(|v| v.as_f64()) {
                        self.app_settings.window_state.y = value as f32;
                    }
                    if let Some(value) = window_state.get("maximized").and_then(|v| v.as_bool()) {
                        self.app_settings.window_state.maximized = value;
                    }
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
    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        // Save settings when the application exits
        self.save_all_settings();
    }

    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        // Show startup screen if not complete
        if !self.startup_complete {
            self.update_startup_progress();
            self.render_startup_screen(ctx);
            return;
        }

        // TEMPORARILY DISABLED: Update window state for saving later
        // self.update_window_state(ctx);

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

        // 定期保存应用设置（每10秒检查一次，减少频率）
        if ctx.input(|i| i.time) % 10.0 < 0.1 {
            if let Err(err) = self.save_app_settings() {
                log::error!("Failed to save app settings: {}", err);
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