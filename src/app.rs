use eframe::egui;
use uuid::Uuid;
use anyhow::Result;

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
use crate::tray::Tray;

// 导入模块
use inote::db_state::DbINoteState;
use isearch::ISearchState;
use aiAssist::state::AIAssistState;
use aiAssist::mcp_integration::McpIntegrationManager;
use itools::IToolsState;
use iterminal::ITerminalState;
use itools::mcp::rmcp_client::McpEvent;

/// Main application state
pub struct SeeUApp {
    // Global state
    pub active_module: Module,
    pub show_right_sidebar: bool,
    /// 跟踪右侧边栏的上一次状态，用于检测打开事件
    pub prev_show_right_sidebar: bool,

    // Search state
    pub search_query: String,
    pub global_search_results: GlobalSearchResults,

    // Module states
    pub inote_state: DbINoteState,
    pub isearch_state: ISearchState,
    pub ai_assist_state: AIAssistState,
    pub itools_state: IToolsState,
    pub iterminal_state: ITerminalState,
    pub ifile_editor_state: ifile_editor::IFileEditorState,
    pub modular_settings_state: crate::ui::settings_trait::ModularSettingsState,

    // Services
    pub system_service: SystemService,

    // MCP Integration
    pub mcp_integration: McpIntegrationManager,

    // MCP Sync Service
    pub mcp_sync_service: Option<inote::mcp_sync::McpSyncService>,

    // Theme
    pub theme: Theme,

    // Application settings
    pub app_settings: AppSettings,

    // Command channel
    slash_command_receiver: Option<std::sync::mpsc::Receiver<AppCommand>>,

    // MCP event channel
    mcp_event_receiver: Option<tokio::sync::mpsc::UnboundedReceiver<McpEvent>>,

    // Startup state
    pub startup_complete: bool,
    pub startup_progress: f32,
    pub startup_message: String,
    pub startup_config: StartupConfig,
    pub startup_metrics: StartupMetrics,

    // UI state
    pub request_ui_repaint: bool,

    // MCP sync state
    pub mcp_sync_pending: bool,

    // System tray
    pub tray: Option<crate::tray::PlatformTray>,
    pub window_visible: bool,
    pub minimize_to_tray: bool,

    // Sidebar
    pub sidebar: Option<crate::sidebar::SidebarWindow>,
    pub show_sidebar: bool,
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
    FileEditor,      // 新增：文件编辑器
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
    RefreshMcpServers,
    // 新增面板互动命令
    TerminalAction(TerminalAction),
    NoteAction(NoteAction),
    EditorAction(EditorAction),
}

/// 终端操作命令
#[derive(Debug, Clone)]
pub enum TerminalAction {
    Execute(String),
    NewSession(Option<String>),
    SwitchSession(String),
    Export,
    GetOutput,
    GetHistory,
}

/// 笔记操作命令
#[derive(Debug, Clone)]
pub enum NoteAction {
    Create(String),
    Search(String),
    Open(String),
    List,
    GetCurrent,
    GetSearchResults,
}

/// 编辑器操作命令
#[derive(Debug, Clone)]
pub enum EditorAction {
    Open(String),
    Create(String),
    Save,
    List,
    GetCurrent,
    GetSelection,
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

        // Initialize inote state and semantic search
        inote_state.initialize();
        inote_state.initialize_storage_async("notes.db".to_string());

        let isearch_state = ISearchState::default();
        let mut ai_assist_state = aiAssist::initialize();
        let itools_state = itools::initialize();
        let iterminal_state = iterminal::initialize();
        let ifile_editor_state = ifile_editor::initialize();

        // Load AI assistant chat sessions
        if let Err(err) = aiAssist::load_chat_sessions(&mut ai_assist_state) {
            log::warn!("Failed to load AI assistant chat sessions: {}", err);
        }

        // Initialize states based on configuration
        // Note: We'll initialize these asynchronously to avoid blocking the UI
        // For now, just do minimal synchronous initialization
        startup_metrics.record_database_init();
        startup_metrics.record_search_init();
        startup_metrics.record_plugin_init();

        // Create MCP sync service
        let mcp_sync_service = Some(inote::mcp_sync::McpSyncService::new(inote_state.storage.clone()));

        // Create app instance
        let mut app = Self {
            active_module: Module::Home,
            show_right_sidebar: false,
            prev_show_right_sidebar: false,
            search_query: String::new(),
            global_search_results: GlobalSearchResults::default(),
            inote_state,
            isearch_state,
            ai_assist_state,
            itools_state,
            iterminal_state,
            ifile_editor_state,
            modular_settings_state: crate::ui::settings_trait::ModularSettingsState::default(),
            system_service: SystemService::new(),
            mcp_integration: McpIntegrationManager::new(),
            mcp_sync_service,
            theme,
            app_settings: AppSettings::default(),
            slash_command_receiver: None,
            mcp_event_receiver: None,
            startup_complete: false, // Always show startup progress to avoid blocking
            startup_progress: 0.0,
            startup_message: "正在初始化应用程序...".to_string(),
            startup_config,
            startup_metrics,
            request_ui_repaint: false,
            mcp_sync_pending: false,
            tray: None,
            window_visible: true,
            minimize_to_tray: true,
            sidebar: None,
            show_sidebar: false,
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
        // Note: egui 0.28.1 doesn't have Theme enum or set_theme method
        // Theme is handled through visuals instead
        let visuals = match app.theme {
            Theme::DarkModern | Theme::Dark => egui::Visuals::dark(),
            Theme::LightModern | Theme::Light => egui::Visuals::light(),
        };
        cc.egui_ctx.set_visuals(visuals);

        // Then apply our custom visuals
        configure_visuals(&cc.egui_ctx, app.theme);

        // Apply UI scale using native pixels_per_point as baseline
        let native_pixels_per_point = cc.egui_ctx.native_pixels_per_point().unwrap_or(1.0);
        cc.egui_ctx.set_pixels_per_point(native_pixels_per_point * app.app_settings.ui_scale);

        app.update_fonts(&cc.egui_ctx);

        // TEMPORARILY DISABLED: Adjust window state for DPI scaling after egui context is available
        // app.adjust_window_state_for_dpi(&cc.egui_ctx);

        // Initialize system tray
        app.initialize_tray();

        // Initialize sidebar
        app.initialize_sidebar();

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
                },
                aiAssist::SlashCommand::Terminal(term_cmd) => {
                    let action = match term_cmd {
                        aiAssist::TerminalCommand::Execute(cmd) => TerminalAction::Execute(cmd),
                        aiAssist::TerminalCommand::NewSession(title) => TerminalAction::NewSession(title),
                        aiAssist::TerminalCommand::SwitchSession(id) => TerminalAction::SwitchSession(id),
                        aiAssist::TerminalCommand::Export => TerminalAction::Export,
                    };
                    let _ = tx_clone.send(AppCommand::TerminalAction(action));
                },
                aiAssist::SlashCommand::Note(note_cmd) => {
                    let action = match note_cmd {
                        aiAssist::NoteCommand::Create(title) => NoteAction::Create(title),
                        aiAssist::NoteCommand::Search(query) => NoteAction::Search(query),
                        aiAssist::NoteCommand::Open(id) => NoteAction::Open(id),
                        aiAssist::NoteCommand::List => NoteAction::List,
                    };
                    let _ = tx_clone.send(AppCommand::NoteAction(action));
                },
                aiAssist::SlashCommand::Editor(editor_cmd) => {
                    let action = match editor_cmd {
                        aiAssist::EditorCommand::Open(path) => EditorAction::Open(path),
                        aiAssist::EditorCommand::Create(path) => EditorAction::Create(path),
                        aiAssist::EditorCommand::Save => EditorAction::Save,
                        aiAssist::EditorCommand::List => EditorAction::List,
                    };
                    let _ = tx_clone.send(AppCommand::EditorAction(action));
                },
                // 其他命令已在AI助手内部处理，不需要发送到应用层
                aiAssist::SlashCommand::Clear |
                aiAssist::SlashCommand::New |
                aiAssist::SlashCommand::Help => {
                    // 这些命令已在AI助手内部处理
                }
            }
        });

        // 设置插入到笔记回调
        let tx_clone = tx.clone();
        aiAssist::set_insert_to_note_callback(&mut self.ai_assist_state, move |content| {
            // 发送插入笔记的命令
            let _ = tx_clone.send(AppCommand::InsertToNote(content));
        });

        // 设置MCP刷新回调
        let tx_clone = tx.clone();
        aiAssist::set_mcp_refresh_callback(&mut self.ai_assist_state, move || {
            // 发送MCP刷新的命令
            let _ = tx_clone.send(AppCommand::RefreshMcpServers);
        });

        // 设置获取编辑器上下文的回调
        // 暂时返回None，@editor引用将通过其他方式实现
        aiAssist::set_get_editor_context_callback(&mut self.ai_assist_state, move || {
            // 这里我们需要一个更安全的方式来获取编辑器上下文
            // 暂时返回None，将在后续实现中完善
            None
        });

        // 注意：我们将在右侧边栏渲染时更新终端上下文
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

        // 启动后台线程进行数据库初始化（优化版本）
        std::thread::spawn(move || {
            log::info!("Starting optimized background database initialization...");
            let start_time = std::time::Instant::now();

            // 初始化笔记数据库（快速模式）
            if let Ok(mut storage) = inote_storage.lock() {
                if let Err(err) = storage.initialize_async_fast() {
                    log::error!("Failed to initialize note database: {}", err);
                } else {
                    let db_time = start_time.elapsed();
                    log::info!("Database initialization completed in {:?}", db_time);
                }
            }

            // 延迟初始化搜索索引（非阻塞）
            std::thread::spawn(move || {
                // 等待一小段时间让UI先启动
                std::thread::sleep(std::time::Duration::from_millis(500));

                log::info!("Starting search index initialization...");
                let search_start = std::time::Instant::now();

                if let Ok(indexer_lock) = isearch_indexer.lock() {
                    if let Err(err) = indexer_lock.initialize_index() {
                        log::error!("Failed to initialize search index: {}", err);
                    } else {
                        let search_time = search_start.elapsed();
                        log::info!("Search index initialization completed in {:?}", search_time);
                    }
                }
            });

            let total_time = start_time.elapsed();
            log::info!("Background initialization completed in {:?}", total_time);
        });

        // 初始化语义搜索引擎（同步方式，标记为启用）
        // Semantic search has been removed

        // 启动进度更新定时器
        self.start_progress_timer();
    }

    /// 启动进度更新定时器
    fn start_progress_timer(&mut self) {
        // 简单的进度模拟，实际应用中可以通过通道接收真实进度
        let start_time = std::time::Instant::now();

        // 设置一个合理的初始化时间（比如2秒）
        let _target_duration = std::time::Duration::from_secs(2);

        // 记录开始时间用于进度计算
        self.startup_metrics.start_time = start_time;
    }

    /// 更新启动进度
    fn update_startup_progress(&mut self) {
        let elapsed = self.startup_metrics.start_time.elapsed();
        // 减少目标时间，让启动更快
        let target_duration = std::time::Duration::from_millis(1200); // 从2秒减少到1.2秒

        // 计算进度（0.0 到 1.0）
        let progress = (elapsed.as_secs_f32() / target_duration.as_secs_f32()).min(1.0);
        self.startup_progress = progress;

        // 更新进度消息并执行相应的初始化任务
        if progress < 0.25 {
            self.startup_message = "正在初始化核心模块...".to_string();
        } else if progress < 0.5 {
            self.startup_message = "正在加载用户界面...".to_string();
        } else if progress < 0.75 {
            self.startup_message = "正在加载用户界面...".to_string();
            // 延迟字体缓存预加载，不阻塞启动
            static FONT_CACHE_SCHEDULED: std::sync::Once = std::sync::Once::new();
            FONT_CACHE_SCHEDULED.call_once(|| {
                // 延迟更长时间，让用户先看到主界面
                std::thread::spawn(|| {
                    std::thread::sleep(std::time::Duration::from_secs(2));
                    log::info!("Starting delayed font cache preloading...");
                    inote::mermaid::preload_font_cache();
                });
            });
        } else if progress < 0.9 {
            self.startup_message = "正在启动后台服务...".to_string();

            // 立即开始后台初始化，不等待动画完成
            static BACKGROUND_INIT_STARTED: std::sync::Once = std::sync::Once::new();
            BACKGROUND_INIT_STARTED.call_once(|| {
                log::info!("Starting immediate background initialization...");
                // 后台初始化将在动画完成后立即开始
            });
        } else {
            self.startup_message = "启动完成".to_string();
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

        // 初始化工具模块（但不加载MCP服务器配置）
        self.itools_state.initialize_without_mcp_loading();

        // 设置MCP事件通道
        self.setup_mcp_event_channel();

        // 现在加载MCP服务器配置（事件接收器已设置）
        self.itools_state.load_mcp_server_configurations();

        // 启动笔记模块的后台数据加载
        self.start_background_data_loading();

        // 延迟执行MCP同步，不阻塞启动
        self.schedule_delayed_mcp_sync();

        log::info!("Module initialization completed");
    }

    /// 调度延迟的MCP同步
    fn schedule_delayed_mcp_sync(&mut self) {
        log::info!("Scheduling delayed MCP synchronization...");

        // 创建一个标记，表示MCP同步需要在后台执行
        // 实际的同步将在用户首次访问AI助手时触发
        self.mcp_sync_pending = true;
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

    /// 设置MCP事件通道
    fn setup_mcp_event_channel(&mut self) {
        // 创建MCP事件通道
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        self.mcp_event_receiver = Some(rx);

        // 将发送端添加到MCP服务器管理器（不覆盖UI的事件接收器）
        if let Some(manager) = self.itools_state.get_mcp_server_manager_mut() {
            manager.add_event_sender(tx);
            log::info!("MCP事件通道已添加到主应用");
        } else {
            log::warn!("无法获取MCP服务器管理器来添加事件通道");
        }
    }

    /// 同步MCP服务器信息到AI助手
    pub fn sync_mcp_servers_to_ai_assistant(&mut self) {
        self.sync_mcp_servers_to_ai_assistant_internal(false);
    }

    pub fn sync_mcp_servers_to_ai_assistant_force(&mut self) {
        self.sync_mcp_servers_to_ai_assistant_internal(true);
    }

    fn sync_mcp_servers_to_ai_assistant_internal(&mut self, force_refresh: bool) {
        // 第一步：同步MCP服务器到数据库
        self.sync_mcp_servers_to_database(force_refresh);

        // 第二步：从数据库读取绿灯服务器并同步到AI助手
        self.load_green_servers_from_database_to_ai_assistant(force_refresh);
    }

    /// 同步MCP服务器到数据库
    fn sync_mcp_servers_to_database(&mut self, force_refresh: bool) {
        if let Some(sync_service) = &self.mcp_sync_service {
            // 获取可用的MCP服务器
            let servers = self.itools_state.get_available_mcp_servers();

            if force_refresh {
                log::info!("🔄 开始同步MCP服务器到数据库: 发现 {} 个服务器", servers.len());
            }

            let mut servers_to_sync = Vec::new();
            let mut active_server_ids = Vec::new();

            // 收集所有服务器信息
            for (server_id, server_name) in &servers {
                active_server_ids.push(*server_id);

                if let Some(manager) = self.itools_state.get_mcp_server_manager() {
                    if let Some(health_status) = manager.get_server_health_status(*server_id) {
                        // 从服务器目录中查找配置信息
                        let mut server_config = None;
                        let directories = manager.get_server_directories();
                        for dir in &directories {
                            for config in &dir.servers {
                                if config.id == *server_id {
                                    server_config = Some(config.clone());
                                    break;
                                }
                            }
                            if server_config.is_some() {
                                break;
                            }
                        }

                        if let Some(config) = server_config {
                            // 构建传输配置JSON
                            let transport_config = match &config.transport {
                                itools::mcp::server_manager::TransportConfig::Command { command, args, env } => {
                                    serde_json::json!({
                                        "type": "Command",
                                        "command": command,
                                        "args": args,
                                        "env": env
                                    }).to_string()
                                }
                                itools::mcp::server_manager::TransportConfig::Tcp { host, port } => {
                                    serde_json::json!({
                                        "type": "Tcp",
                                        "host": host,
                                        "port": port
                                    }).to_string()
                                }
                                itools::mcp::server_manager::TransportConfig::Unix { socket_path } => {
                                    serde_json::json!({
                                        "type": "Unix",
                                        "socket_path": socket_path
                                    }).to_string()
                                }
                                itools::mcp::server_manager::TransportConfig::WebSocket { url } => {
                                    serde_json::json!({
                                        "type": "WebSocket",
                                        "url": url
                                    }).to_string()
                                }
                            };

                            // 获取服务器能力 - 优先使用运行时能力，如果没有则使用配置文件中的静态能力
                            let capabilities_json = if let Some(capabilities) = self.itools_state.get_mcp_server_capabilities(*server_id) {
                                // 运行时能力：由于ToolInfo等结构体没有实现Serialize，我们需要手动构建JSON
                                log::info!("🔧 找到服务器 '{}' 的运行时能力: 工具={}, 资源={}, 提示={}",
                                    server_name, capabilities.tools.len(), capabilities.resources.len(), capabilities.prompts.len());

                                // 详细记录工具信息
                                if !capabilities.tools.is_empty() {
                                    log::info!("🛠️ 服务器 '{}' 的工具详情:", server_name);
                                    for (index, tool) in capabilities.tools.iter().enumerate() {
                                        log::info!("  {}. {} - {}",
                                            index + 1,
                                            tool.name,
                                            tool.description.as_deref().unwrap_or("无描述")
                                        );
                                    }
                                } else {
                                    log::warn!("⚠️ 服务器 '{}' 的运行时能力中没有工具！", server_name);
                                }

                                let tools_json: Vec<serde_json::Value> = capabilities.tools.iter().map(|tool| {
                                    serde_json::json!({
                                        "name": tool.name,
                                        "description": tool.description,
                                        "input_schema": tool.input_schema
                                    })
                                }).collect();

                                let resources_json: Vec<serde_json::Value> = capabilities.resources.iter().map(|resource| {
                                    serde_json::json!({
                                        "uri": resource.uri,
                                        "name": resource.name,
                                        "description": resource.description,
                                        "mime_type": resource.mime_type
                                    })
                                }).collect();

                                let prompts_json: Vec<serde_json::Value> = capabilities.prompts.iter().map(|prompt| {
                                    let args_json: Vec<serde_json::Value> = prompt.arguments.iter().map(|arg| {
                                        serde_json::json!({
                                            "name": arg.name,
                                            "description": arg.description,
                                            "required": arg.required
                                        })
                                    }).collect();

                                    serde_json::json!({
                                        "name": prompt.name,
                                        "description": prompt.description,
                                        "arguments": args_json
                                    })
                                }).collect();

                                Some(serde_json::json!({
                                    "tools": tools_json,
                                    "resources": resources_json,
                                    "prompts": prompts_json
                                }).to_string())
                            } else if let Some(static_capabilities) = &config.capabilities {
                                // 静态能力：直接使用配置文件中的JSON
                                log::debug!("🔧 使用服务器 '{}' 的静态能力配置", server_name);
                                log::debug!("📋 静态能力内容: {}", static_capabilities);
                                Some(static_capabilities.to_string())
                            } else {
                                log::warn!("⚠️ 服务器 '{}' 既没有运行时能力也没有静态能力配置", server_name);
                                log::warn!("🔍 调试信息 - 服务器状态: {:?}", health_status);

                                // 尝试手动触发能力提取
                                log::info!("🔄 尝试手动触发服务器 '{}' 的能力提取", server_name);

                                // 创建一个异步任务来执行能力提取
                                let _server_id_copy = *server_id;
                                let server_name_copy = server_name.clone();

                                // 注意：这里我们不能直接调用异步方法，因为我们在同步上下文中
                                // 我们将在下一次同步时尝试提取能力
                                log::info!("💡 将在下次MCP事件处理时尝试强制提取能力 - 服务器: '{}'", server_name_copy);

                                None
                            };

                            let health_status_str = match health_status {
                                itools::mcp::rmcp_client::ServerHealthStatus::Red => "Red",
                                itools::mcp::rmcp_client::ServerHealthStatus::Yellow => "Yellow",
                                itools::mcp::rmcp_client::ServerHealthStatus::Green => "Green",
                            }.to_string();

                            servers_to_sync.push((
                                *server_id,
                                server_name.clone(),
                                config.description.clone(),
                                "Command".to_string(), // 简化处理，实际应该根据transport类型设置
                                transport_config,
                                config.directory.clone(),
                                capabilities_json,
                                health_status_str,
                            ));
                        }
                    }
                }
            }

            // 批量同步服务器到数据库
            if !servers_to_sync.is_empty() {
                match sync_service.batch_sync_servers(servers_to_sync) {
                    Ok(synced_count) => {
                        if force_refresh {
                            log::info!("✅ 成功同步 {} 个MCP服务器到数据库", synced_count);
                        }
                    }
                    Err(e) => {
                        log::error!("❌ 批量同步MCP服务器到数据库失败: {}", e);
                    }
                }
            }

            // 清理数据库中不再存在的服务器
            if let Err(e) = sync_service.cleanup_orphaned_servers(active_server_ids) {
                log::error!("❌ 清理孤立MCP服务器失败: {}", e);
            }
        } else {
            log::warn!("⚠️ MCP同步服务不可用");
        }
    }

    /// 从数据库加载绿灯服务器到AI助手
    fn load_green_servers_from_database_to_ai_assistant(&mut self, force_refresh: bool) {
        log::debug!("🔄 开始从数据库加载绿灯服务器到AI助手 (force_refresh: {})", force_refresh);

        if let Some(sync_service) = &self.mcp_sync_service {
            // 记录当前AI助手中的服务器数量，用于检测变化
            let prev_ai_server_count = self.ai_assist_state.mcp_server_capabilities.len();
            log::debug!("📊 当前AI助手中的服务器数量: {}", prev_ai_server_count);

            // 如果是强制刷新，先清空AI助手中的MCP服务器状态
            if force_refresh {
                log::info!("🧹 清空AI助手中的MCP服务器状态，准备从数据库重新加载");
                self.ai_assist_state.mcp_server_capabilities.clear();
                self.ai_assist_state.server_names.clear();
                self.ai_assist_state.selected_mcp_server = None;
            } else {
                // 非强制刷新时，检查并移除已删除的服务器
                let current_server_ids: std::collections::HashSet<Uuid> =
                    self.itools_state.get_available_mcp_servers()
                        .into_iter()
                        .map(|(id, _)| id)
                        .collect();

                let ai_server_ids: Vec<Uuid> = self.ai_assist_state.mcp_server_capabilities.keys().cloned().collect();

                for ai_server_id in ai_server_ids {
                    if !current_server_ids.contains(&ai_server_id) {
                        log::info!("🗑️ 检测到已删除的MCP服务器，从AI助手中移除: {}", ai_server_id);
                        self.ai_assist_state.remove_mcp_server(ai_server_id);
                    }
                }
            }

            // 从数据库获取绿灯状态的服务器
            log::debug!("📋 查询数据库中的绿灯服务器...");
            match sync_service.get_green_servers() {
                Ok(green_servers) => {
                    let mut synced_count = 0;
                    let total_checked = green_servers.len();
                    log::info!("📊 数据库查询结果: 找到 {} 个绿灯状态的MCP服务器", total_checked);

                    for server_record in green_servers {
                        log::debug!("🔍 处理服务器记录: {} ({})", server_record.name, server_record.id);
                        if let Ok(server_id) = server_record.get_uuid() {
                            // 更新MCP集成管理器中的服务器名称
                            self.mcp_integration.update_server_name(server_id, server_record.name.clone());

                            // 解析服务器能力
                            if let Some(capabilities_json) = &server_record.capabilities {
                                log::debug!("🔍 解析服务器 '{}' 的能力JSON: {}", server_record.name, capabilities_json);

                                if let Ok(capabilities_value) = serde_json::from_str::<serde_json::Value>(capabilities_json) {
                                    // 转换为AI助手格式的能力
                                    let ai_capabilities = self.convert_json_capabilities_to_ai_format(&capabilities_value);

                                    log::debug!("📊 转换后的AI助手能力 - 服务器 '{}': 工具={}, 资源={}, 提示={}",
                                        server_record.name, ai_capabilities.tools.len(), ai_capabilities.resources.len(), ai_capabilities.prompts.len());

                                    // 详细记录工具信息（仅在调试模式）
                                    if !ai_capabilities.tools.is_empty() {
                                        log::debug!("🛠️ 从数据库加载的工具详情 - 服务器 '{}':", server_record.name);
                                        for (index, tool) in ai_capabilities.tools.iter().enumerate() {
                                            log::debug!("  {}. {} - {}",
                                                index + 1,
                                                tool.name,
                                                tool.description.as_deref().unwrap_or("无描述")
                                            );
                                        }
                                    } else {
                                        log::warn!("⚠️ 从数据库加载的服务器 '{}' 没有工具！", server_record.name);
                                    }

                                    self.mcp_integration.update_server_capabilities(
                                        &mut self.ai_assist_state,
                                        server_id,
                                        ai_capabilities,
                                    );
                                    synced_count += 1;

                                    if force_refresh {
                                        log::info!("✅ 从数据库加载绿灯MCP服务器 '{}' 到AI助手", server_record.name);
                                    }
                                } else {
                                    log::warn!("⚠️ 无法解析服务器 '{}' 的能力JSON: {}", server_record.name, capabilities_json);
                                }
                            } else {
                                log::warn!("⚠️ 服务器 '{}' 没有能力信息", server_record.name);

                                // 对于绿灯但没有能力信息的服务器，记录需要获取能力的服务器
                                if let Ok(_server_uuid) = server_record.get_uuid() {
                                    log::info!("⚠️ 绿灯服务器 '{}' 没有能力信息，需要手动测试以获取能力", server_record.name);
                                    // 这里可以添加到一个待处理列表，供用户手动触发测试
                                }
                            }
                        } else {
                            log::error!("❌ 无法解析服务器ID: {}", server_record.id);
                        }
                    }

                    if force_refresh {
                        log::info!("🎯 从数据库加载完成: 检查了 {} 个服务器, 加载了 {} 个绿灯MCP服务器到AI助手, AI助手现在有 {} 个可用服务器",
                            total_checked, synced_count, self.ai_assist_state.mcp_server_capabilities.len());

                        // 列出所有可用的服务器
                        if !self.ai_assist_state.mcp_server_capabilities.is_empty() {
                            let server_list: Vec<String> = self.ai_assist_state.server_names.values().cloned().collect();
                            log::info!("📋 AI助手可用的MCP服务器: {}", server_list.join(", "));
                        } else {
                            log::warn!("⚠️ 数据库中没有找到可用的绿灯MCP服务器");
                        }
                    } else {
                        let current_ai_server_count = self.ai_assist_state.mcp_server_capabilities.len();

                        // 检测状态变化
                        if current_ai_server_count != prev_ai_server_count {
                            log::info!("🔄 MCP服务器状态发生变化: AI助手中的服务器数量从 {} 变为 {}",
                                prev_ai_server_count, current_ai_server_count);

                            if current_ai_server_count > prev_ai_server_count {
                                log::info!("✅ 新增了 {} 个可用的MCP服务器", current_ai_server_count - prev_ai_server_count);
                            } else if current_ai_server_count < prev_ai_server_count {
                                log::info!("❌ 移除了 {} 个MCP服务器", prev_ai_server_count - current_ai_server_count);
                            }
                        }

                        log::debug!("MCP sync completed: checked {} servers, synced {} green servers, AI assistant now has {} servers",
                            total_checked, synced_count, current_ai_server_count);

                        if synced_count > 0 {
                            log::info!("从数据库加载了 {} 个绿灯MCP服务器到AI助手", synced_count);
                        }
                    }
                }
                Err(e) => {
                    log::error!("❌ 从数据库获取绿灯MCP服务器失败: {}", e);
                }
            }
        } else {
            log::warn!("⚠️ MCP同步服务不可用");
        }
    }

    /// 处理MCP事件
    fn process_mcp_events(&mut self) {
        // 收集所有待处理的事件
        let mut events = Vec::new();
        if let Some(receiver) = &mut self.mcp_event_receiver {
            while let Ok(event) = receiver.try_recv() {
                events.push(event);
            }
        } else {
            log::debug!("⚠️ 主应用MCP事件接收器为空");
        }

        if !events.is_empty() {
            log::info!("📥 主应用收到 {} 个MCP事件", events.len());
        }

        // 处理收集到的事件
        let mut needs_sync = false;
        let mut has_green_status_change = false;

        for event in events {
            match event {
                McpEvent::HealthStatusChanged(server_id, status) => {
                    log::info!("🔄 收到MCP服务器状态变化事件: {} -> {:?}", server_id, status);

                    // 检查是否有服务器变成绿灯状态
                    if matches!(status, itools::mcp::rmcp_client::ServerHealthStatus::Green) {
                        log::info!("🟢 MCP服务器 {} 变成绿灯状态，将立即刷新AI助手下拉框", server_id);
                        has_green_status_change = true;
                    }

                    needs_sync = true;
                }
                McpEvent::ServerConnected(server_id) => {
                    log::info!("🔗 MCP服务器已连接: {}", server_id);
                    needs_sync = true;
                }
                McpEvent::ServerDisconnected(server_id) => {
                    log::info!("🔌 MCP服务器已断开: {}", server_id);

                    // 立即从AI助手中移除断开的服务器
                    self.ai_assist_state.remove_mcp_server(server_id);

                    needs_sync = true;
                }
                McpEvent::CapabilitiesUpdated(server_id, _capabilities) => {
                    log::info!("🔧 MCP服务器能力已更新: {}", server_id);
                    needs_sync = true;
                }
                McpEvent::CapabilitiesExtracted(server_id, capabilities, capabilities_json) => {
                    log::info!("🎯 收到MCP服务器能力提取成功事件: {} - 工具:{}, 资源:{}, 提示:{}",
                        server_id, capabilities.tools.len(), capabilities.resources.len(), capabilities.prompts.len());

                    // 将测试通过的能力保存到数据库
                    if let Err(e) = self.save_extracted_capabilities_to_database(server_id, &capabilities_json) {
                        log::error!("❌ 保存提取的能力到数据库失败: {}", e);
                    } else {
                        log::info!("✅ 成功保存测试通过的能力到数据库 - 服务器: {}", server_id);
                        has_green_status_change = true; // 触发AI助手状态更新
                    }

                    needs_sync = true;
                }
                McpEvent::TestCompleted(server_id, test_result) => {
                    log::info!("🧪 MCP服务器测试完成: {} (成功: {})", server_id, test_result.success);

                    // 如果测试成功，说明服务器可能变成绿灯状态
                    if test_result.success {
                        log::info!("✅ MCP服务器 {} 测试成功，将立即刷新AI助手下拉框", server_id);
                        has_green_status_change = true;
                    }

                    needs_sync = true;
                }
                McpEvent::ServerError(server_id, error) => {
                    log::warn!("❌ MCP服务器错误: {} - {}", server_id, error);
                    needs_sync = true;
                }
            }
        }

        // 如果有任何事件需要同步，执行一次同步（包括数据库同步）
        if needs_sync {
            self.sync_mcp_servers_to_ai_assistant();

            // 如果有服务器变成绿灯状态，记录日志表示AI助手下拉框应该自动更新
            if has_green_status_change {
                log::info!("🟢 MCP服务器变成绿灯状态，AI助手下拉框将通过同步机制自动更新");
                // 标记需要重绘UI
                self.request_ui_repaint = true;
            }
        }
    }

    // /// 转换MCP服务器能力为AI助手格式
    // fn convert_mcp_capabilities_to_ai_format(
    //     &self,
    //     capabilities: &itools::mcp::rmcp_client::ServerCapabilities,
    // ) -> aiAssist::mcp_tools::McpServerCapabilities {
    //     use aiAssist::mcp_tools::{McpServerCapabilities, McpToolInfo, McpResourceInfo, McpPromptInfo, McpPromptArgument};

    //     let tools = capabilities.tools.iter().map(|tool| {
    //         McpToolInfo {
    //             name: tool.name.clone(),
    //             description: tool.description.clone(),
    //             input_schema: tool.input_schema.clone(),
    //         }
    //     }).collect();

    //     let resources = capabilities.resources.iter().map(|resource| {
    //         McpResourceInfo {
    //             uri: resource.uri.clone(),
    //             name: resource.name.clone(),
    //             description: resource.description.clone(),
    //             mime_type: resource.mime_type.clone(),
    //         }
    //     }).collect();

    //     let prompts = capabilities.prompts.iter().map(|prompt| {
    //         let arguments = prompt.arguments.iter().map(|arg| {
    //             McpPromptArgument {
    //                 name: arg.name.clone(),
    //                 description: arg.description.clone(),
    //                 required: arg.required,
    //             }
    //         }).collect();

    //         McpPromptInfo {
    //             name: prompt.name.clone(),
    //             description: prompt.description.clone(),
    //             arguments,
    //         }
    //     }).collect();

    //     McpServerCapabilities {
    //         tools,
    //         resources,
    //         prompts,
    //     }
    // }

    /// 转换JSON格式的能力为AI助手格式
    fn convert_json_capabilities_to_ai_format(
        &self,
        capabilities_json: &serde_json::Value,
    ) -> aiAssist::mcp_tools::McpServerCapabilities {
        use aiAssist::mcp_tools::{McpServerCapabilities, McpToolInfo, McpResourceInfo, McpPromptInfo, McpPromptArgument};

        let tools = capabilities_json.get("tools")
            .and_then(|t| t.as_array())
            .map(|tools_array| {
                tools_array.iter().filter_map(|tool| {
                    let name = tool.get("name")?.as_str()?.to_string();
                    let description = tool.get("description").and_then(|d| d.as_str()).unwrap_or("").to_string();
                    let input_schema = tool.get("input_schema").cloned().unwrap_or(serde_json::Value::Null);

                    Some(McpToolInfo {
                        name,
                        description: Some(description),
                        input_schema: Some(input_schema),
                    })
                }).collect()
            })
            .unwrap_or_default();

        let resources = capabilities_json.get("resources")
            .and_then(|r| r.as_array())
            .map(|resources_array| {
                resources_array.iter().filter_map(|resource| {
                    let uri = resource.get("uri")?.as_str()?.to_string();
                    let name = resource.get("name").and_then(|n| n.as_str()).unwrap_or("").to_string();
                    let description = resource.get("description").and_then(|d| d.as_str()).unwrap_or("").to_string();
                    let mime_type = resource.get("mime_type").and_then(|m| m.as_str()).map(|s| s.to_string());

                    Some(McpResourceInfo {
                        uri,
                        name,
                        description: Some(description),
                        mime_type,
                    })
                }).collect()
            })
            .unwrap_or_default();

        let prompts = capabilities_json.get("prompts")
            .and_then(|p| p.as_array())
            .map(|prompts_array| {
                prompts_array.iter().filter_map(|prompt| {
                    let name = prompt.get("name")?.as_str()?.to_string();
                    let description = prompt.get("description").and_then(|d| d.as_str()).unwrap_or("").to_string();

                    let arguments = prompt.get("arguments")
                        .and_then(|a| a.as_array())
                        .map(|args_array| {
                            args_array.iter().filter_map(|arg| {
                                let arg_name = arg.get("name")?.as_str()?.to_string();
                                let arg_description = arg.get("description").and_then(|d| d.as_str()).unwrap_or("").to_string();
                                let required = arg.get("required").and_then(|r| r.as_bool()).unwrap_or(false);

                                Some(McpPromptArgument {
                                    name: arg_name,
                                    description: Some(arg_description),
                                    required,
                                })
                            }).collect()
                        })
                        .unwrap_or_default();

                    Some(McpPromptInfo {
                        name,
                        description: Some(description),
                        arguments,
                    })
                }).collect()
            })
            .unwrap_or_default();

        McpServerCapabilities {
            tools,
            resources,
            prompts,
        }
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
        // Collect all pending commands first to avoid borrowing conflicts
        let mut commands = Vec::new();
        if let Some(rx) = &self.slash_command_receiver {
            while let Ok(cmd) = rx.try_recv() {
                commands.push(cmd);
            }
        }

        // Process collected commands
        for cmd in commands {
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
                },
                AppCommand::RefreshMcpServers => {
                    log::info!("🔄 收到MCP服务器刷新请求，立即强制同步");
                    self.sync_mcp_servers_to_ai_assistant_force();
                },
                AppCommand::TerminalAction(action) => {
                    self.handle_terminal_action(action);
                },
                AppCommand::NoteAction(action) => {
                    self.handle_note_action(action);
                },
                AppCommand::EditorAction(action) => {
                    self.handle_editor_action(action);
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

        // 转换为本地时间显示
        let local_modified = first_result.modified.with_timezone(&chrono::Local);
        let formatted = format!(
            "搜索结果（第1条，共{}条）：\n\n文件名: {}\n路径: {}\n文件类型: {}\n修改时间: {}\n文件大小: {} bytes\n\n内容:\n{}",
            self.isearch_state.search_stats.total_results,
            first_result.filename,
            first_result.path,
            first_result.file_type,
            local_modified.format("%Y-%m-%d %H:%M:%S"),
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

    /// 更新AI助手的上下文信息
    fn update_ai_assistant_contexts(&mut self) {
        // 更新终端上下文
        if let Some(session_id) = self.iterminal_state.egui_terminal_manager.get_active_session_id() {
            if let Some(session) = self.iterminal_state.egui_terminal_manager.get_sessions().get(&session_id) {
                match session.get_text_content() {
                    Ok(content) => {
                        // 限制输出长度，避免过长的内容
                        let terminal_output = if content.len() > 2000 {
                            format!("{}...\n[输出已截断，总长度: {} 字符]",
                                &content[content.len().saturating_sub(2000)..], content.len())
                        } else {
                            content
                        };
                        aiAssist::update_terminal_output(&mut self.ai_assist_state, terminal_output);
                    },
                    Err(_) => {
                        // 如果获取失败，清空终端上下文
                        aiAssist::update_terminal_output(&mut self.ai_assist_state, String::new());
                    }
                }
            }
        }

        // 更新笔记上下文
        if let Some(note_id) = &self.inote_state.current_note {
            if let Some(note) = self.inote_state.notes.get(note_id) {
                aiAssist::update_note_context(&mut self.ai_assist_state,
                    note.title.clone(), note.content.clone());
            }
        } else {
            // 清空笔记上下文
            aiAssist::clear_note_context(&mut self.ai_assist_state);
        }

        // 注意：不再自动更新编辑器上下文到AI助手
        // 编辑器上下文只有在用户显式使用@editor时才会被引用
        // 这避免了文件编辑器中打开文件时自动引用其内容的问题
    }

    /// 处理消息中的 @term 引用，获取当前终端内容
    fn process_term_references(&self, content: &str) -> String {
        if !content.contains("@term") {
            return content.to_string();
        }

        // 尝试获取当前终端内容
        let terminal_content = if let Some(session_id) = self.iterminal_state.egui_terminal_manager.get_active_session_id() {
            if let Some(session) = self.iterminal_state.egui_terminal_manager.get_sessions().get(&session_id) {
                match session.get_text_content() {
                    Ok(content) => {
                        // 限制输出长度，避免过长的内容
                        if content.len() > 2000 {
                            Some(format!("{}...\n[输出已截断，总长度: {} 字符]",
                                &content[content.len().saturating_sub(2000)..], content.len()))
                        } else {
                            Some(content)
                        }
                    },
                    Err(_) => None,
                }
            } else {
                None
            }
        } else {
            None
        };

        // 如果获取到终端内容，替换 @term；否则保持不变
        if let Some(terminal_output) = terminal_content {
            let replacement = format!("@term (当前终端输出):\n{}", terminal_output);
            content.replace("@term", &replacement)
        } else {
            // 如果没有终端内容，保持 @term 不变
            content.to_string()
        }
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
        // Search results are now stored directly in search_results field
        let search_results = &self.inote_state.search_results;
        self.global_search_results.inote_results = search_results.iter()
            .take(5) // Limit to 5 results for Home display
            .filter_map(|note_id| {
                self.inote_state.notes.get(note_id).map(|note| {
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

        // First, set the egui built-in visuals to ensure proper base styling
        let visuals = match new_theme {
            Theme::DarkModern | Theme::Dark => egui::Visuals::dark(),
            Theme::LightModern | Theme::Light => egui::Visuals::light(),
        };
        ctx.set_visuals(visuals);

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
        let visuals = match self.theme {
            Theme::DarkModern | Theme::Dark => egui::Visuals::dark(),
            Theme::LightModern | Theme::Light => egui::Visuals::light(),
        };
        ctx.set_visuals(visuals);

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
    pub fn update_fonts(&self, ctx: &egui::Context) {
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

        // Save AI assistant chat sessions
        if let Err(err) = aiAssist::save_chat_sessions(&self.ai_assist_state) {
            log::error!("Failed to save AI assistant chat sessions: {}", err);
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
            self.render_animated_splash(ui);
        });

        // Request repaint to update progress
        ctx.request_repaint();
    }

    /// Render animated splash screen with SVG animations
    fn render_animated_splash(&mut self, ui: &mut egui::Ui) {
        let available_rect = ui.available_rect_before_wrap();
        let center = available_rect.center();
        let time = ui.input(|i| i.time) as f32;

        // Background gradient
        let bg_color1 = egui::Color32::from_rgb(15, 23, 42);  // Dark blue
        // let bg_color2 = egui::Color32::from_rgb(30, 41, 59);  // Lighter blue
        ui.painter().rect_filled(
            available_rect,
            egui::Rounding::ZERO,
            bg_color1,
        );

        // Central logo area
        let logo_center = egui::Pos2::new(center.x, center.y - 80.0);

        // Main logo with pulsing animation
        let pulse_scale = 1.0 + 0.1 * (time * 2.0).sin();
        let logo_size = 80.0 * pulse_scale;
        // let logo_rect = egui::Rect::from_center_size(logo_center, egui::Vec2::splat(logo_size));

        // Draw main logo circle with gradient
        let logo_color = egui::Color32::from_rgb(59, 130, 246); // Blue
        ui.painter().circle_filled(logo_center, logo_size / 2.0, logo_color);

        // Inner glow effect
        let glow_alpha = (0.3 + 0.2 * (time * 3.0).sin()).max(0.0).min(1.0);
        let glow_color = egui::Color32::from_rgba_unmultiplied(59, 130, 246, (glow_alpha * 100.0) as u8);
        ui.painter().circle_filled(logo_center, logo_size / 2.0 + 10.0, glow_color);

        // Logo text "SeeU"
        let logo_text_color = egui::Color32::WHITE;
        ui.painter().text(
            logo_center,
            egui::Align2::CENTER_CENTER,
            "SeeU",
            egui::FontId::proportional(24.0),
            logo_text_color,
        );

        // Feature modules animation
        self.render_feature_modules_animation(ui, center, time);

        // App title
        let title_pos = egui::Pos2::new(center.x, logo_center.y + 120.0);
        ui.painter().text(
            title_pos,
            egui::Align2::CENTER_CENTER,
            "SeeU Desktop",
            egui::FontId::proportional(32.0),
            egui::Color32::WHITE,
        );

        // Subtitle with typewriter effect
        let subtitle_pos = egui::Pos2::new(center.x, title_pos.y + 40.0);
        let subtitle_text = "智能桌面应用 · AI驱动 · 高性能";
        let chars: Vec<char> = subtitle_text.chars().collect();
        let typewriter_progress = ((time * 2.0) % (chars.len() as f32 + 2.0)) as usize;
        let displayed_text = if typewriter_progress < chars.len() {
            chars[..typewriter_progress].iter().collect::<String>()
        } else {
            subtitle_text.to_string()
        };

        ui.painter().text(
            subtitle_pos,
            egui::Align2::CENTER_CENTER,
            &displayed_text,
            egui::FontId::proportional(16.0),
            egui::Color32::from_rgb(148, 163, 184),
        );

        // Progress area
        let progress_y = subtitle_pos.y + 80.0;
        self.render_animated_progress(ui, center.x, progress_y, time);
    }

    /// Render feature modules with animation
    fn render_feature_modules_animation(&mut self, ui: &mut egui::Ui, center: egui::Pos2, time: f32) {
        let modules = [
            ("📝", "笔记", egui::Color32::from_rgb(34, 197, 94)),   // Green
            ("🔍", "搜索", egui::Color32::from_rgb(249, 115, 22)),  // Orange
            ("💻", "终端", egui::Color32::from_rgb(168, 85, 247)),  // Purple
            ("📁", "文件", egui::Color32::from_rgb(236, 72, 153)),  // Pink
            ("🤖", "AI", egui::Color32::from_rgb(14, 165, 233)),    // Sky blue
        ];

        let radius = 150.0;
        let logo_center = egui::Pos2::new(center.x, center.y - 80.0);

        for (i, (icon, name, color)) in modules.iter().enumerate() {
            // Staggered appearance animation
            let delay = i as f32 * 0.3;
            let appear_progress = ((time - delay) * 2.0).max(0.0).min(1.0);

            if appear_progress > 0.0 {
                let angle = (i as f32 * 2.0 * std::f32::consts::PI / modules.len() as f32) - std::f32::consts::PI / 2.0;
                let module_pos = egui::Pos2::new(
                    logo_center.x + radius * angle.cos(),
                    logo_center.y + radius * angle.sin(),
                );

                // Scale animation
                let scale = appear_progress * (1.0 + 0.1 * (time * 3.0 + i as f32).sin());
                let module_size = 40.0 * scale;

                // Draw connection line with drawing animation
                let line_progress = ((time - delay - 0.5) * 3.0).max(0.0).min(1.0);
                if line_progress > 0.0 {
                    let line_end = egui::Pos2::new(
                        logo_center.x + (module_pos.x - logo_center.x) * line_progress,
                        logo_center.y + (module_pos.y - logo_center.y) * line_progress,
                    );
                    ui.painter().line_segment(
                        [logo_center, line_end],
                        egui::Stroke::new(2.0, egui::Color32::from_rgba_unmultiplied(255, 255, 255, 60)),
                    );
                }

                // Draw module circle
                ui.painter().circle_filled(module_pos, module_size / 2.0, *color);

                // Module icon
                ui.painter().text(
                    module_pos,
                    egui::Align2::CENTER_CENTER,
                    icon,
                    egui::FontId::proportional(20.0),
                    egui::Color32::WHITE,
                );

                // Module name
                let name_pos = egui::Pos2::new(module_pos.x, module_pos.y + module_size / 2.0 + 15.0);
                ui.painter().text(
                    name_pos,
                    egui::Align2::CENTER_CENTER,
                    name,
                    egui::FontId::proportional(12.0),
                    egui::Color32::from_rgb(148, 163, 184),
                );
            }
        }
    }

    /// Render animated progress bar
    fn render_animated_progress(&mut self, ui: &mut egui::Ui, center_x: f32, y: f32, time: f32) {
        let progress_width = 300.0;
        let progress_height = 8.0;
        let progress_rect = egui::Rect::from_min_size(
            egui::Pos2::new(center_x - progress_width / 2.0, y),
            egui::Vec2::new(progress_width, progress_height),
        );

        // Background
        ui.painter().rect_filled(
            progress_rect,
            egui::Rounding::same(4.0),
            egui::Color32::from_rgb(51, 65, 85),
        );

        // Animated progress fill with gradient
        let fill_width = progress_width * self.startup_progress;
        if fill_width > 0.0 {
            let fill_rect = egui::Rect::from_min_size(
                progress_rect.min,
                egui::Vec2::new(fill_width, progress_height),
            );

            // Gradient effect
            let gradient_offset = (time * 2.0) % 1.0;
            let base_color = egui::Color32::from_rgb(59, 130, 246);
            let highlight_color = egui::Color32::from_rgb(147, 197, 253);

            ui.painter().rect_filled(
                fill_rect,
                egui::Rounding::same(4.0),
                base_color,
            );

            // Moving highlight
            if self.startup_progress > 0.1 {
                let highlight_width = 60.0;
                let highlight_x = (fill_width - highlight_width) * gradient_offset;
                let highlight_rect = egui::Rect::from_min_size(
                    egui::Pos2::new(progress_rect.min.x + highlight_x, progress_rect.min.y),
                    egui::Vec2::new(highlight_width.min(fill_width), progress_height),
                );

                ui.painter().rect_filled(
                    highlight_rect,
                    egui::Rounding::same(4.0),
                    highlight_color,
                );
            }
        }

        // Progress text
        let text_pos = egui::Pos2::new(center_x, y + 25.0);
        ui.painter().text(
            text_pos,
            egui::Align2::CENTER_CENTER,
            &self.startup_message,
            egui::FontId::proportional(14.0),
            egui::Color32::from_rgb(148, 163, 184),
        );

        // Progress percentage
        let percentage_text = format!("{}%", (self.startup_progress * 100.0) as u32);
        let percentage_pos = egui::Pos2::new(center_x, y + 45.0);
        ui.painter().text(
            percentage_pos,
            egui::Align2::CENTER_CENTER,
            &percentage_text,
            egui::FontId::proportional(12.0),
            egui::Color32::from_rgb(100, 116, 139),
        );
    }

    /// Initialize system tray
    fn initialize_tray(&mut self) {
        // Load tray icon
        let icon_data = match crate::utils::icon::load_tray_icon() {
            Ok(data) => data,
            Err(e) => {
                log::warn!("Failed to load tray icon: {}, using default", e);
                // Use a simple default icon (1x1 transparent pixel)
                vec![0, 0, 0, 0]
            }
        };

        // Create default menu
        let menu = crate::tray::default_menu();

        // Create tray
        match crate::tray::create_tray(&icon_data, menu) {
            Ok(tray) => {
                self.tray = Some(tray);
                log::info!("System tray initialized successfully");
            }
            Err(e) => {
                log::error!("Failed to initialize system tray: {}", e);
            }
        }
    }

    /// Handle tray events
    fn handle_tray_events(&mut self, ctx: &egui::Context) {
        // Collect events first to avoid borrowing conflicts
        let mut events = Vec::new();
        if let Some(ref mut tray) = self.tray {
            while let Some(event) = tray.try_recv_event() {
                events.push(event);
            }
        }

        // Process events
        for event in events {
            match event {
                crate::tray::TrayEvent::LeftClick | crate::tray::TrayEvent::DoubleClick => {
                    self.toggle_window_visibility(ctx);
                }
                crate::tray::TrayEvent::RightClick => {
                    // Right click is handled by the tray menu
                }
                crate::tray::TrayEvent::MenuItemClick(item_id) => {
                    self.handle_tray_menu_click(&item_id, ctx);
                }
            }
        }
    }

    /// Handle tray menu item clicks
    fn handle_tray_menu_click(&mut self, item_id: &str, ctx: &egui::Context) {
        match item_id {
            "show" => {
                self.show_window(ctx);
            }
            "hide" => {
                self.hide_window(ctx);
            }
            "sidebar" => {
                self.show_sidebar = !self.show_sidebar;
                log::info!("Sidebar toggled: {}", self.show_sidebar);
            }
            "settings" => {
                self.show_window(ctx);
                self.active_module = Module::Settings;
            }
            "quit" => {
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
            _ => {
                log::debug!("Unknown tray menu item clicked: {}", item_id);
            }
        }
    }

    /// Toggle window visibility
    fn toggle_window_visibility(&mut self, ctx: &egui::Context) {
        if self.window_visible {
            self.hide_window(ctx);
        } else {
            self.show_window(ctx);
        }
    }

    /// Show the main window
    fn show_window(&mut self, ctx: &egui::Context) {
        self.window_visible = true;
        ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
        ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
    }

    /// Hide the main window to tray
    fn hide_window(&mut self, ctx: &egui::Context) {
        if self.minimize_to_tray {
            self.window_visible = false;
            ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
            // 当主窗口隐藏时，显示侧边栏
            self.show_sidebar = true;
        } else {
            ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(true));
        }
    }

    /// Initialize sidebar
    fn initialize_sidebar(&mut self) {
        let config = crate::sidebar::SidebarConfig::default();

        // 暂时在所有平台都使用内嵌侧边栏
        match crate::sidebar::create_sidebar_window(config) {
            Ok(sidebar) => {
                self.sidebar = Some(sidebar);
                log::info!("Sidebar initialized successfully");
            }
            Err(e) => {
                log::error!("Failed to initialize sidebar: {}", e);
            }
        }
    }

    /// Handle sidebar events and rendering
    fn handle_sidebar(&mut self, ctx: &egui::Context) {
        if self.show_sidebar {
            if let Some(ref mut sidebar) = self.sidebar {
                // Update sidebar
                sidebar.update(ctx);

                // Render sidebar and handle events
                if let Some(event) = sidebar.render(ctx) {
                    match event {
                        crate::sidebar::SidebarEvent::Close => {
                            self.show_sidebar = false;
                        }
                        crate::sidebar::SidebarEvent::Hide => {
                            self.show_sidebar = false;
                        }
                        _ => {
                            sidebar.handle_event(event);
                        }
                    }
                }
            }
        }
    }
}

impl eframe::App for SeeUApp {
    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        // Clean up system tray
        if let Some(tray) = self.tray.take() {
            if let Err(e) = tray.shutdown() {
                log::error!("Failed to shutdown tray: {}", e);
            }
        }

        // Save settings when the application exits
        self.save_all_settings();
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Show startup screen if not complete
        if !self.startup_complete {
            self.update_startup_progress();
            self.render_startup_screen(ctx);
            return;
        }

        // TEMPORARILY DISABLED: Update window state for saving later
        // self.update_window_state(ctx);

        // Handle tray events
        self.handle_tray_events(ctx);

        // Handle sidebar
        self.handle_sidebar(ctx);

        // Refresh system information for accurate CPU and memory usage
        self.system_service.refresh();

        // Process any pending slash commands
        self.process_slash_commands();

        // 更新AI助手的上下文信息
        self.update_ai_assistant_contexts();

        // 处理全局键盘快捷键
        self.handle_global_shortcuts(ctx);

        // Check if search module wants to navigate to settings
        if self.isearch_state.navigate_to_settings {
            self.active_module = Module::Settings;
            self.isearch_state.navigate_to_settings = false;
            // Set settings to search category
            self.modular_settings_state.selected_category = "search".to_string();
        }

        // Top panel - search bar
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            self.render_search_bar(ui);
        });

        // Bottom status bar - 放在最前面确保它总是占据全宽
        let save_status = self.inote_state.save_status.clone();
        let system_service = &self.system_service;

        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            // 获取iFile编辑器状态信息
            let ifile_status = if self.active_module == Module::FileEditor {
                self.ifile_editor_state.get_current_file_status()
            } else {
                None
            };

            render_status_bar(
                ui,
                system_service,
                &mut self.show_right_sidebar,
                self.active_module,
                save_status,
                ifile_status
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

        // 注意：不再自动设置AI助手的文件上下文
        // 文件上下文只有在用户显式使用@editor时才会被引用
        // 这避免了切换到文件编辑器模块时自动引用文件内容的问题

        // 处理待执行的工具调用
        self.process_pending_tool_execution();

        // 更新 iTools 模块
        itools::update_itools(&mut self.itools_state);

        // 处理MCP事件（事件驱动的同步）
        self.process_mcp_events();

        // 检查是否需要重绘UI（例如MCP服务器状态变化）
        if self.request_ui_repaint {
            ctx.request_repaint();
            self.request_ui_repaint = false;
        }

        // 检测右侧边栏打开事件，立即同步MCP服务器
        if self.show_right_sidebar && !self.prev_show_right_sidebar {
            log::info!("🎯 AI助手侧边栏已打开，立即强制刷新MCP服务器状态");
            self.sync_mcp_servers_to_ai_assistant_force();
        }

        // 更新右侧边栏状态跟踪
        self.prev_show_right_sidebar = self.show_right_sidebar;

        // 更新 iTerminal 模块
        iterminal::update_iterminal(&mut self.iterminal_state);

        // 定期保存笔记数据（每5秒检查一次）
        let current_time = ctx.input(|i| i.time);
        if current_time % 5.0 < 0.1 {
            if self.inote_state.save_status == inote::db_state::SaveStatus::Modified {
                self.inote_state.auto_save_if_modified();
            }
        }

        // 定期保存应用设置（每10秒检查一次，减少频率）
        if current_time % 10.0 < 0.1 {
            if let Err(err) = self.save_app_settings() {
                log::error!("Failed to save app settings: {}", err);
            }
        }

        // 让 egui 自动处理面板高度

        // 右侧边栏 (如果启用) - 放在主面板之前
        let right_sidebar_width = if self.show_right_sidebar {
            let response = egui::SidePanel::right("right_sidebar")
                .resizable(true)
                .default_width(300.0)
                .show(ctx, |ui| {
                    render_right_sidebar(ui, self);
                });
            Some(response.response.rect.width())
        } else {
            None
        };

        // 主面板 - 放在最后，确保它填充剩余空间
        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(ctx.style().visuals.window_fill))
            .show(ctx, |ui| {
                // 渲染工作区，传递右侧边栏宽度信息
                render_workspace(ui, &current_module, self, right_sidebar_width);
            });
    }



    /// Save state when the app is about to close
    fn save(&mut self, _storage: &mut dyn eframe::Storage) {
        // Save all settings before exit
        self.save_all_settings();
    }
}

impl SeeUApp {
    /// 查找服务器ID根据服务器名称
    fn find_server_id_by_name(&self, target_server_name: &str) -> Option<uuid::Uuid> {
        // 从MCP设置中查找服务器ID
        if let Some(_manager) = &self.itools_state.mcp_server_manager {
            for (server_id, server_name) in self.itools_state.get_available_mcp_servers() {
                if server_name == target_server_name {
                    return Some(server_id);
                }
            }
        }
        None
    }

    // /// 查找服务器ID根据服务器名称（备用方法）
    // fn find_server_id_by_name_alt(&self, target_server_name: &str) -> Option<uuid::Uuid> {
    //     // 从MCP服务器管理器中查找服务器ID
    //     for (server_id, server_name) in self.itools_state.get_available_mcp_servers() {
    //         if server_name == target_server_name {
    //             return Some(server_id);
    //         }
    //     }
    //     None
    // }



    /// 处理待执行的工具调用
    fn process_pending_tool_execution(&mut self) {
        if self.ai_assist_state.tool_execution_pending {
            log::info!("🚀 检测到待执行的工具调用，开始处理");

            // 重置标志
            self.ai_assist_state.tool_execution_pending = false;

            // 克隆必要的数据以避免借用冲突
            if let Some(batch) = self.ai_assist_state.current_tool_call_batch.clone() {
                if batch.user_approved {
                    log::info!("🔧 开始执行 {} 个已确认的工具调用", batch.tool_calls.len());

                    // 不再创建独立的执行开始消息，而是直接开始执行
                    log::info!("🔧 开始执行 {} 个工具调用...", batch.tool_calls.len());

                    // 执行真正的MCP工具调用
                    let _mcp_integration = self.mcp_integration.clone();
                    let mut results = Vec::new();

                    for pending_call in &batch.tool_calls {
                        log::info!("🔧 执行工具调用: {} (服务器: {})",
                            pending_call.tool_call.function.name,
                            pending_call.server_name
                        );

                        // 查找服务器ID
                        let server_id = if let Some(server_id) = self.find_server_id_by_name(&pending_call.server_name) {
                            server_id
                        } else {
                            log::error!("❌ 找不到服务器: {}", pending_call.server_name);
                            let error_result = aiAssist::mcp_tools::McpToolCallResult {
                                tool_call_id: pending_call.tool_call.id.clone(),
                                success: false,
                                result: serde_json::json!({}),
                                error: Some(format!("找不到服务器: {}", pending_call.server_name)),
                            };
                            results.push(error_result);
                            continue;
                        };

                        // 解析参数
                        let arguments = match serde_json::from_str::<serde_json::Value>(&pending_call.tool_call.function.arguments) {
                            Ok(args) => args,
                            Err(e) => {
                                log::error!("❌ 解析工具参数失败: {}", e);
                                let error_result = aiAssist::mcp_tools::McpToolCallResult {
                                    tool_call_id: pending_call.tool_call.id.clone(),
                                    success: false,
                                    result: serde_json::json!({}),
                                    error: Some(format!("参数解析失败: {}", e)),
                                };
                                results.push(error_result);
                                continue;
                            }
                        };

                        // 调用真正的MCP工具 (使用阻塞调用，因为我们在主线程中)
                        let tool_name = pending_call.tool_call.function.name.clone();

                        let call_result = if let Some(manager) = &self.itools_state.mcp_server_manager {
                            // 使用测试版本的工具调用，它会绕过健康状态检查并自动处理连接
                            if let Some(rmcp_client) = manager.get_rmcp_client() {
                                if let Ok(handle) = tokio::runtime::Handle::try_current() {
                                    // 如果已经在异步运行时中，使用spawn_blocking
                                    match handle.block_on(rmcp_client.call_tool_for_testing(server_id, &tool_name, arguments)) {
                                        Ok(response) => Ok(response),
                                        Err(e) => Err(e)
                                    }
                                } else {
                                    // 如果不在异步运行时中，创建新的运行时
                                    match tokio::runtime::Runtime::new() {
                                        Ok(rt) => {
                                            rt.block_on(rmcp_client.call_tool_for_testing(server_id, &tool_name, arguments))
                                        }
                                        Err(e) => Err(anyhow::anyhow!("无法创建异步运行时: {}", e))
                                    }
                                }
                            } else {
                                Err(anyhow::anyhow!("无法获取RMCP客户端"))
                            }
                        } else {
                            Err(anyhow::anyhow!("MCP服务器管理器未初始化"))
                        };

                        match call_result {
                            Ok(response) => {
                                log::info!("✅ 工具 {} 执行成功", tool_name);
                                let result = aiAssist::mcp_tools::McpToolCallResult {
                                    tool_call_id: pending_call.tool_call.id.clone(),
                                    success: true,
                                    result: response,
                                    error: None,
                                };
                                results.push(result);
                            }
                            Err(e) => {
                                log::error!("❌ 工具 {} 执行失败: {}", tool_name, e);
                                let error_result = aiAssist::mcp_tools::McpToolCallResult {
                                    tool_call_id: pending_call.tool_call.id.clone(),
                                    success: false,
                                    result: serde_json::json!({}),
                                    error: Some(e.to_string()),
                                };
                                results.push(error_result);
                            }
                        }
                    }

                    log::info!("✅ 工具调用执行完成，成功: {}, 失败: {}",
                        results.iter().filter(|r| r.success).count(),
                        results.iter().filter(|r| !r.success).count()
                    );

                    // 将结果转换为ToolCallResult格式
                    let tool_call_results: Vec<aiAssist::state::ToolCallResult> = results.iter().map(|r| {
                        aiAssist::state::ToolCallResult {
                            tool_call_id: r.tool_call_id.clone(),
                            result: serde_json::to_string_pretty(&r.result).unwrap_or_default(),
                            success: r.success,
                            error: r.error.clone(),
                            timestamp: chrono::Utc::now(),
                        }
                    }).collect();

                    // 找到包含工具调用的消息并添加结果
                    self.add_tool_results_to_message(&tool_call_results);

                    // 自动保存会话
                    self.ai_assist_state.auto_save_sessions();

                    log::info!("📝 工具调用结果已添加到聊天记录");

                    // 自动将工具调用结果发送给大模型获取完整反馈
                    self.send_tool_results_to_llm();

                    // 清理当前批次
                    self.ai_assist_state.current_tool_call_batch = None;
                } else {
                    log::warn!("⚠️ 工具调用批次未获得用户确认，跳过执行");
                }
            } else {
                log::warn!("⚠️ 没有找到待执行的工具调用批次");
            }
        }
    }

    /// 自动将工具调用结果发送给大模型获取完整反馈
    fn send_tool_results_to_llm(&mut self) {
        log::info!("🤖 开始自动发送工具调用结果给大模型获取完整反馈");

        // 检查是否有AI设置
        if self.ai_assist_state.ai_settings.api_key.is_empty() {
            log::warn!("⚠️ AI设置不完整，跳过自动发送");
            return;
        }

        // 准备消息历史，包含工具调用和结果
        let messages = self.ai_assist_state.prepare_messages_for_api_with_tool_results();

        if messages.is_empty() {
            log::warn!("⚠️ 没有消息历史，跳过自动发送");
            return;
        }

        log::info!("📤 准备发送 {} 条消息给大模型（包含工具调用结果）", messages.len());

        // 创建一个新的助手消息来接收大模型的反馈
        let response_message_id = uuid::Uuid::new_v4();
        let response_message = aiAssist::state::ChatMessage {
            id: response_message_id,
            role: aiAssist::state::MessageRole::Assistant,
            content: String::new(),
            timestamp: chrono::Utc::now(),
            attachments: vec![],
            tool_calls: None,
            tool_call_results: None,
            mcp_server_info: None,
        };

        // 添加到聊天记录
        self.ai_assist_state.chat_messages.push(response_message.clone());

        // 添加到当前会话
        if let Some(session) = self.ai_assist_state.chat_sessions.get_mut(self.ai_assist_state.active_session_idx) {
            session.messages.push(response_message.clone());
        }

        // 设置流式输出状态
        self.ai_assist_state.streaming_message_id = Some(response_message_id);
        self.ai_assist_state.is_sending = true;

        // 异步发送请求
        let api_service = self.ai_assist_state.api_service.clone();
        let ai_settings = self.ai_assist_state.ai_settings.clone();

        // 创建共享状态
        let state_mutex = std::sync::Arc::new(std::sync::Mutex::new(aiAssist::state::StateUpdate {
            message_id: response_message_id,
            content: String::new(),
            is_complete: false,
            error: None,
            has_function_calls: false,
            function_call_response: None,
        }));

        let ui_state = state_mutex.clone();
        let request_id = uuid::Uuid::new_v4();

        // 存储请求状态
        aiAssist::state::ACTIVE_REQUESTS.lock().unwrap().insert(request_id, ui_state);

        // 设置当前请求ID，这样UI可以检查更新
        self.ai_assist_state.current_request_id = Some(request_id);

        // 启动后台任务
        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                log::info!("🚀 开始异步发送工具调用结果给大模型");

                match api_service.send_chat(&ai_settings, messages).await {
                    Ok(response) => {
                        log::info!("✅ 大模型反馈获取成功，内容长度: {}", response.len());
                        let mut state = state_mutex.lock().unwrap();
                        state.content = response;
                        state.is_complete = true;
                    },
                    Err(e) => {
                        log::error!("❌ 获取大模型反馈失败: {}", e);
                        let mut state = state_mutex.lock().unwrap();
                        state.error = Some(format!("获取大模型反馈失败: {}", e));
                        state.is_complete = true;
                    }
                }

                // 注意：不在这里清理请求状态，让UI线程在check_for_updates中处理
                log::info!("🔔 后台任务完成，等待UI线程处理");
            });
        });
    }

    /// 将工具调用结果添加到对应的工具调用消息中
    fn add_tool_results_to_message(&mut self, tool_call_results: &[aiAssist::state::ToolCallResult]) {
        // 找到最近的包含工具调用的助手消息
        for message in self.ai_assist_state.chat_messages.iter_mut().rev() {
            if message.role == aiAssist::state::MessageRole::Assistant && message.tool_calls.is_some() {
                // 检查是否有匹配的工具调用ID
                if let Some(tool_calls) = &message.tool_calls {
                    let tool_call_ids: std::collections::HashSet<String> = tool_calls.iter().map(|tc| tc.id.clone()).collect();
                    let result_ids: std::collections::HashSet<String> = tool_call_results.iter().map(|tr| tr.tool_call_id.clone()).collect();

                    // 如果有交集，说明这些结果属于这个消息
                    if !tool_call_ids.is_disjoint(&result_ids) {
                        // 合并现有结果和新结果
                        let mut all_results = message.tool_call_results.clone().unwrap_or_default();
                        all_results.extend_from_slice(tool_call_results);
                        message.tool_call_results = Some(all_results);

                        log::info!("✅ 已将 {} 个工具调用结果添加到消息 {}", tool_call_results.len(), message.id);
                        break;
                    }
                }
            }
        }

        // 同样更新当前会话中的消息
        if let Some(session) = self.ai_assist_state.chat_sessions.get_mut(self.ai_assist_state.active_session_idx) {
            for message in session.messages.iter_mut().rev() {
                if message.role == aiAssist::state::MessageRole::Assistant && message.tool_calls.is_some() {
                    if let Some(tool_calls) = &message.tool_calls {
                        let tool_call_ids: std::collections::HashSet<String> = tool_calls.iter().map(|tc| tc.id.clone()).collect();
                        let result_ids: std::collections::HashSet<String> = tool_call_results.iter().map(|tr| tr.tool_call_id.clone()).collect();

                        if !tool_call_ids.is_disjoint(&result_ids) {
                            let mut all_results = message.tool_call_results.clone().unwrap_or_default();
                            all_results.extend_from_slice(tool_call_results);
                            message.tool_call_results = Some(all_results);
                            break;
                        }
                    }
                }
            }
        }

        // 自动保存会话
        self.ai_assist_state.auto_save_sessions();
    }

    /// 保存提取的能力到数据库
    fn save_extracted_capabilities_to_database(&self, server_id: uuid::Uuid, capabilities_json: &str) -> Result<(), Box<dyn std::error::Error>> {
        log::info!("💾 开始保存提取的能力到数据库 - 服务器: {}", server_id);

        // 首先尝试从数据库加载现有的MCP服务器记录
        let storage = self.inote_state.storage.lock()
            .map_err(|e| format!("Failed to lock storage: {}", e))?;

        // 加载现有服务器记录
        let mut server_record = match storage.load_mcp_server(&server_id.to_string()) {
            Ok(record) => record,
            Err(e) => {
                log::error!("❌ 无法加载MCP服务器记录: {} - 错误: {}", server_id, e);
                return Err(format!("无法加载MCP服务器记录: {}", e).into());
            }
        };

        // 更新能力字段
        server_record.capabilities = Some(capabilities_json.to_string());
        server_record.updated_at = chrono::Utc::now();

        // 保存更新后的记录
        match storage.save_mcp_server(&server_record) {
            Ok(()) => {
                log::info!("✅ 成功保存提取的能力到数据库 - 服务器: '{}'", server_record.name);

                // 验证保存结果
                if let Some(caps) = &server_record.capabilities {
                    match serde_json::from_str::<serde_json::Value>(caps) {
                        Ok(parsed_caps) => {
                            let tools_count = parsed_caps.get("tools")
                                .and_then(|t| t.as_array())
                                .map(|arr| arr.len())
                                .unwrap_or(0);

                            log::info!("✅ 验证成功 - 服务器 '{}' 的能力已保存到数据库，包含 {} 个工具", server_record.name, tools_count);
                        }
                        Err(e) => {
                            log::error!("❌ 验证失败 - 保存的能力JSON格式错误: {}", e);
                            return Err(format!("保存的能力JSON格式错误: {}", e).into());
                        }
                    }
                } else {
                    log::warn!("⚠️ 验证失败 - 能力字段为空");
                }

                Ok(())
            }
            Err(e) => {
                log::error!("❌ 保存MCP服务器记录失败: {}", e);
                Err(format!("保存MCP服务器记录失败: {}", e).into())
            }
        }
    }

    /// 从搜索结果打开文件到编辑器
    pub fn open_file_in_editor(&mut self, file_path: String) {
        use std::path::PathBuf;
        let path = PathBuf::from(file_path);

        log::info!("Opening file in editor: {:?}", path);

        // 切换到文件编辑器模块
        self.active_module = Module::FileEditor;

        // 打开文件
        if let Err(e) = self.ifile_editor_state.open_file_from_search(path) {
            log::error!("Failed to open file in editor: {}", e);
        }
    }

    /// 处理终端操作命令
    fn handle_terminal_action(&mut self, action: TerminalAction) {
        log::info!("处理终端操作: {:?}", action);

        // 切换到终端模块
        self.active_module = Module::Terminal;

        match action {
            TerminalAction::Execute(command) => {
                // 在当前终端会话执行命令
                log::info!("执行终端命令: {}", command);

                // 获取当前活动会话的信息
                let manager = &self.iterminal_state.egui_terminal_manager;
                let session_info = if let Some(session_id) = manager.get_active_session_id() {
                    if let Some(session) = manager.get_sessions().get(&session_id) {
                        (session_id.to_string(), session.title.clone())
                    } else {
                        ("unknown".to_string(), "Unknown Session".to_string())
                    }
                } else {
                    ("default".to_string(), "主会话".to_string())
                };

                // 向终端发送命令
                let manager = &mut self.iterminal_state.egui_terminal_manager;
                if let Err(e) = manager.send_command_to_active_session(&command) {
                    log::error!("发送命令到终端失败: {}", e);
                }

                // 更新AI助手的终端上下文
                let context = aiAssist::TerminalContext {
                    current_session_id: Some(session_info.0),
                    current_session_title: Some(session_info.1),
                    last_command: Some(command.clone()),
                    last_output: Some("命令执行中...".to_string()),
                    command_history: vec![command.clone()],
                    working_directory: None,
                };
                self.ai_assist_state.set_terminal_context(context);

                // 添加系统消息到AI助手
                self.add_terminal_system_message(&format!("已执行终端命令: {}", command));
            },
            TerminalAction::NewSession(title) => {
                // 创建新的终端会话
                log::info!("创建新终端会话: {:?}", title);

                // 需要egui上下文来创建会话，这里我们暂时记录请求
                // 实际创建会话需要在UI渲染时进行
                self.iterminal_state.pending_new_session = title.clone();

                let session_title = title.unwrap_or_else(|| "新会话".to_string());
                self.add_terminal_system_message(&format!("请求创建新终端会话: {}", session_title));
            },
            TerminalAction::SwitchSession(session_id) => {
                // 切换终端会话
                log::info!("切换到终端会话: {}", session_id);

                // 尝试解析UUID
                if let Ok(uuid) = uuid::Uuid::parse_str(&session_id) {
                    if self.iterminal_state.set_active_session(uuid) {
                        self.add_terminal_system_message(&format!("已切换到终端会话: {}", session_id));

                        // 更新AI助手上下文
                        if let Some(session) = self.iterminal_state.egui_terminal_manager.get_sessions().get(&uuid) {
                            let context = aiAssist::TerminalContext {
                                current_session_id: Some(uuid.to_string()),
                                current_session_title: Some(session.title.clone()),
                                last_command: None,
                                last_output: None,
                                command_history: vec![],
                                working_directory: None,
                            };
                            self.ai_assist_state.set_terminal_context(context);
                        }
                    } else {
                        self.add_terminal_system_message(&format!("切换终端会话失败: 会话 {} 不存在", session_id));
                    }
                } else {
                    self.add_terminal_system_message(&format!("无效的会话ID: {}", session_id));
                }
            },
            TerminalAction::Export => {
                // 导出终端内容
                log::info!("导出终端内容");

                if let Some(session_id) = self.iterminal_state.egui_terminal_manager.get_active_session_id() {
                    if let Some(session) = self.iterminal_state.egui_terminal_manager.get_sessions().get(&session_id) {
                        match session.get_text_content() {
                            Ok(content) => {
                                // 更新AI助手的终端输出
                                self.ai_assist_state.update_terminal_output(content.clone());
                                self.add_terminal_system_message(&format!("已导出终端内容 ({} 字符)", content.len()));
                            },
                            Err(e) => {
                                log::error!("导出终端内容失败: {}", e);
                                self.add_terminal_system_message(&format!("导出终端内容失败: {}", e));
                            }
                        }
                    } else {
                        self.add_terminal_system_message("导出失败: 没有活动的终端会话");
                    }
                } else {
                    self.add_terminal_system_message("导出失败: 没有活动的终端会话");
                }
            },
            TerminalAction::GetOutput => {
                // 获取终端输出
                log::info!("获取终端输出");

                if let Some(session_id) = self.iterminal_state.egui_terminal_manager.get_active_session_id() {
                    if let Some(session) = self.iterminal_state.egui_terminal_manager.get_sessions().get(&session_id) {
                        match session.get_text_content() {
                            Ok(content) => {
                                // 更新AI助手的终端输出
                                self.ai_assist_state.update_terminal_output(content);
                                self.add_terminal_system_message("已获取当前终端输出");
                            },
                            Err(e) => {
                                log::error!("获取终端输出失败: {}", e);
                                self.add_terminal_system_message(&format!("获取终端输出失败: {}", e));
                            }
                        }
                    } else {
                        self.add_terminal_system_message("获取失败: 没有活动的终端会话");
                    }
                } else {
                    self.add_terminal_system_message("获取失败: 没有活动的终端会话");
                }
            },
            TerminalAction::GetHistory => {
                // 获取命令历史
                log::info!("获取命令历史");

                // 这里可以从终端历史管理器获取历史
                // 暂时返回一个占位符
                self.add_terminal_system_message("命令历史功能正在开发中");
            },
        }
    }

    /// 添加终端系统消息到AI助手
    fn add_terminal_system_message(&mut self, message: &str) {
        use aiAssist::{ChatMessage, MessageRole};
        use chrono::Utc;
        use uuid::Uuid;

        let system_message = ChatMessage {
            id: Uuid::new_v4(),
            role: MessageRole::System,
            content: format!("🖥️ 终端: {}", message),
            timestamp: Utc::now(),
            attachments: vec![],
            tool_calls: None,
            tool_call_results: None,
            mcp_server_info: None,
        };

        self.ai_assist_state.chat_messages.push(system_message.clone());

        // Add to current session
        if let Some(session) = self.ai_assist_state.chat_sessions.get_mut(self.ai_assist_state.active_session_idx) {
            session.messages.push(system_message);
        }
    }

    /// 添加笔记系统消息到AI助手
    fn add_note_system_message(&mut self, message: &str) {
        use aiAssist::{ChatMessage, MessageRole};
        use chrono::Utc;
        use uuid::Uuid;

        let system_message = ChatMessage {
            id: Uuid::new_v4(),
            role: MessageRole::System,
            content: format!("📝 笔记: {}", message),
            timestamp: Utc::now(),
            attachments: vec![],
            tool_calls: None,
            tool_call_results: None,
            mcp_server_info: None,
        };

        self.ai_assist_state.chat_messages.push(system_message.clone());

        // Add to current session
        if let Some(session) = self.ai_assist_state.chat_sessions.get_mut(self.ai_assist_state.active_session_idx) {
            session.messages.push(system_message);
        }
    }

    /// 处理笔记操作命令
    fn handle_note_action(&mut self, action: NoteAction) {
        log::info!("处理笔记操作: {:?}", action);

        // 切换到笔记模块
        self.active_module = Module::Note;

        match action {
            NoteAction::Create(title) => {
                // 创建新笔记
                log::info!("创建新笔记: {}", title);

                // 确保有默认笔记本
                if self.inote_state.notebooks.is_empty() {
                    self.inote_state.create_notebook("默认笔记本".to_string(), "默认笔记本".to_string());
                    self.inote_state.current_notebook = Some(0);
                }

                let note_id = self.inote_state.create_note(title.clone(), "".to_string());
                if let Some(id) = note_id {
                    self.inote_state.select_note(&id);

                    // 更新AI助手的笔记上下文
                    self.ai_assist_state.update_current_note(id.clone(), title.clone(), "".to_string());
                    self.add_note_system_message(&format!("已创建新笔记: {}", title));
                } else {
                    self.add_note_system_message(&format!("创建笔记失败: {}", title));
                }
            },
            NoteAction::Search(query) => {
                // 搜索笔记
                log::info!("搜索笔记: {}", query);
                self.inote_state.search_query = query.clone();
                self.inote_state.search_notes();

                // 更新AI助手的笔记搜索结果
                let results: Vec<String> = self.inote_state.search_results.clone();
                let result_count = results.len();
                self.ai_assist_state.update_note_search_results(query.clone(), results);

                self.add_note_system_message(&format!("搜索笔记 \"{}\" 完成，找到 {} 个结果", query, result_count));
            },
            NoteAction::Open(note_id) => {
                // 打开指定笔记
                log::info!("打开笔记: {}", note_id);

                // 首先检查笔记是否存在
                if let Some(note) = self.inote_state.notes.get(&note_id) {
                    let note_title = note.title.clone();
                    let note_content = note.content.clone();

                    self.inote_state.select_note(&note_id);

                    // 更新AI助手的笔记上下文
                    self.ai_assist_state.update_current_note(
                        note_id.clone(),
                        note_title.clone(),
                        note_content
                    );

                    self.add_note_system_message(&format!("已打开笔记: {}", note_title));
                } else {
                    // 尝试选择笔记（会自动从数据库加载）
                    self.inote_state.select_note(&note_id);
                    self.add_note_system_message(&format!("正在加载笔记: {}", note_id));
                }
            },
            NoteAction::List => {
                // 列出所有笔记
                log::info!("列出所有笔记");

                let mut note_list = Vec::new();
                for notebook in &self.inote_state.notebooks {
                    for note_id in &notebook.note_ids {
                        if let Some(note) = self.inote_state.notes.get(note_id) {
                            note_list.push(format!("📝 {} ({})", note.title, notebook.name));
                        }
                    }
                }

                if note_list.is_empty() {
                    self.add_note_system_message("当前没有笔记");
                } else {
                    let list_text = note_list.join("\n");
                    self.add_note_system_message(&format!("笔记列表 ({} 个笔记):\n{}", note_list.len(), list_text));
                }
            },
            NoteAction::GetCurrent => {
                // 获取当前笔记
                log::info!("获取当前笔记");

                if let Some(note_id) = &self.inote_state.current_note {
                    if let Some(note) = self.inote_state.notes.get(note_id) {
                        self.ai_assist_state.update_current_note(
                            note_id.clone(),
                            note.title.clone(),
                            note.content.clone()
                        );
                        self.add_note_system_message(&format!("已获取当前笔记: {}", note.title));
                    } else {
                        self.add_note_system_message("当前笔记不存在");
                    }
                } else {
                    self.add_note_system_message("当前没有打开的笔记");
                }
            },
            NoteAction::GetSearchResults => {
                // 获取搜索结果
                log::info!("获取笔记搜索结果");

                let results: Vec<String> = self.inote_state.search_results.clone();
                let query = self.inote_state.search_query.clone();

                if results.is_empty() {
                    self.add_note_system_message("没有搜索结果");
                } else {
                    let mut result_list = Vec::new();
                    for note_id in &results {
                        if let Some(note) = self.inote_state.notes.get(note_id) {
                            result_list.push(format!("📝 {}", note.title));
                        }
                    }

                    let list_text = result_list.join("\n");
                    self.ai_assist_state.update_note_search_results(query.clone(), results);
                    self.add_note_system_message(&format!("搜索结果 \"{}\" ({} 个结果):\n{}", query, result_list.len(), list_text));
                }
            },
        }
    }

    /// 处理编辑器操作命令
    fn handle_editor_action(&mut self, action: EditorAction) {
        log::info!("处理编辑器操作: {:?}", action);

        // 切换到文件编辑器模块
        self.active_module = Module::FileEditor;

        match action {
            EditorAction::Open(path) => {
                // 打开文件
                log::info!("打开文件: {}", path);
                use std::path::PathBuf;
                let file_path = PathBuf::from(&path);

                // 确保文件编辑器已初始化
                self.ifile_editor_state.ensure_initialized();

                if let Err(e) = self.ifile_editor_state.open_file_from_search(file_path.clone()) {
                    log::error!("打开文件失败: {}", e);
                    self.add_editor_system_message(&format!("打开文件失败: {} - {}", path, e));
                } else {
                    // 更新AI助手的文件上下文
                    if let Some(context) = self.ifile_editor_state.get_current_file_context() {
                        let ai_context = Self::convert_file_context(context);
                        aiAssist::set_file_context(&mut self.ai_assist_state, Some(ai_context));
                        self.add_editor_system_message(&format!("已打开文件: {}", path));
                    } else {
                        self.add_editor_system_message(&format!("文件已打开，但无法获取内容: {}", path));
                    }
                }
            },
            EditorAction::Create(path) => {
                // 创建新文件
                log::info!("创建新文件: {}", path);
                use std::path::PathBuf;
                let file_path = PathBuf::from(&path);

                // 确保文件编辑器已初始化
                self.ifile_editor_state.ensure_initialized();

                // 创建新文件（创建空文件）
                match std::fs::File::create(&file_path) {
                    Ok(_) => {
                        // 文件创建成功，现在打开它
                        if let Err(e) = self.ifile_editor_state.open_file_from_search(file_path) {
                            log::error!("创建文件后打开失败: {}", e);
                            self.add_editor_system_message(&format!("创建文件失败: {} - {}", path, e));
                        } else {
                            self.add_editor_system_message(&format!("已创建并打开新文件: {}", path));

                            // 更新AI助手的文件上下文
                            if let Some(context) = self.ifile_editor_state.get_current_file_context() {
                                let ai_context = Self::convert_file_context(context);
                                aiAssist::set_file_context(&mut self.ai_assist_state, Some(ai_context));
                            }
                        }
                    },
                    Err(e) => {
                        log::error!("创建文件失败: {}", e);
                        self.add_editor_system_message(&format!("创建文件失败: {} - {}", path, e));
                    }
                }
            },
            EditorAction::Save => {
                // 保存当前文件
                log::info!("保存当前文件");

                if let Some(buffer) = self.ifile_editor_state.editor.get_active_buffer() {
                    let file_path = buffer.file_path.clone();
                    let file_name = file_path.file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("未知文件");

                    // 检查文件是否被修改
                    if buffer.modified {
                        // 执行保存操作
                        match self.ifile_editor_state.editor.save_active_file() {
                            Ok(_) => {
                                self.add_editor_system_message(&format!("已保存文件: {}", file_name));

                                // 更新AI助手的文件上下文
                                if let Some(context) = self.ifile_editor_state.get_current_file_context() {
                                    let ai_context = Self::convert_file_context(context);
                                    aiAssist::set_file_context(&mut self.ai_assist_state, Some(ai_context));
                                }
                            },
                            Err(e) => {
                                log::error!("保存文件失败: {}", e);
                                self.add_editor_system_message(&format!("保存文件失败: {} - {}", file_name, e));
                            }
                        }
                    } else {
                        self.add_editor_system_message(&format!("文件无需保存: {}", file_name));
                    }
                } else {
                    self.add_editor_system_message("没有打开的文件可以保存");
                }
            },
            EditorAction::List => {
                // 列出打开的文件
                log::info!("列出打开的文件");

                let tabs = &self.ifile_editor_state.editor.tabs;
                if tabs.is_empty() {
                    self.add_editor_system_message("当前没有打开的文件");
                } else {
                    let mut file_list = Vec::new();
                    for (index, path) in tabs.iter().enumerate() {
                        let file_name = path.file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("未知文件");

                        let is_active = self.ifile_editor_state.editor.active_tab == Some(index);
                        let is_modified = self.ifile_editor_state.editor.buffers.get(path)
                            .map(|b| b.modified)
                            .unwrap_or(false);

                        let status = if is_active { "🔸" } else { "📄" };
                        let modified_mark = if is_modified { "*" } else { "" };

                        file_list.push(format!("{} {}{}", status, file_name, modified_mark));
                    }

                    let list_text = file_list.join("\n");
                    self.add_editor_system_message(&format!("打开的文件 ({} 个):\n{}", tabs.len(), list_text));
                }
            },
            EditorAction::GetCurrent => {
                // 获取当前文件内容
                log::info!("获取当前文件内容");

                if let Some(context) = self.ifile_editor_state.get_current_file_context() {
                    let file_name = context.file_name.clone();
                    let total_lines = context.total_lines;
                    let ai_context = Self::convert_file_context(context);
                    aiAssist::set_file_context(&mut self.ai_assist_state, Some(ai_context));
                    self.add_editor_system_message(&format!("已获取当前文件内容: {} ({} 行)",
                        file_name, total_lines));
                } else {
                    self.add_editor_system_message("当前没有打开的文件");
                }
            },
            EditorAction::GetSelection => {
                // 获取选中的文本
                log::info!("获取选中的文本");

                if let Some(context) = self.ifile_editor_state.get_current_file_context() {
                    if let Some(selected_text) = &context.selected_text {
                        let text_len = selected_text.len();
                        let ai_context = Self::convert_file_context(context);
                        aiAssist::set_file_context(&mut self.ai_assist_state, Some(ai_context));
                        self.add_editor_system_message(&format!("已获取选中文本 ({} 字符)", text_len));
                    } else {
                        self.add_editor_system_message("当前没有选中的文本");
                    }
                } else {
                    self.add_editor_system_message("当前没有打开的文件");
                }
            },
        }
    }

    /// 添加编辑器系统消息到AI助手
    fn add_editor_system_message(&mut self, message: &str) {
        use aiAssist::{ChatMessage, MessageRole};
        use chrono::Utc;
        use uuid::Uuid;

        let system_message = ChatMessage {
            id: Uuid::new_v4(),
            role: MessageRole::System,
            content: format!("📄 编辑器: {}", message),
            timestamp: Utc::now(),
            attachments: vec![],
            tool_calls: None,
            tool_call_results: None,
            mcp_server_info: None,
        };

        self.ai_assist_state.chat_messages.push(system_message.clone());

        // Add to current session
        if let Some(session) = self.ai_assist_state.chat_sessions.get_mut(self.ai_assist_state.active_session_idx) {
            session.messages.push(system_message);
        }
    }

    /// 将 ifile_editor::FileContext 转换为 aiAssist::FileContext
    fn convert_file_context(context: ifile_editor::FileContext) -> aiAssist::FileContext {
        aiAssist::FileContext {
            file_path: context.file_path,
            file_name: context.file_name,
            language: context.language,
            content: context.content,
            selected_text: context.selected_text,
            cursor_line: context.cursor_line,
            cursor_column: context.cursor_column,
            total_lines: context.total_lines,
            is_modified: context.is_modified,
            is_read_only: context.is_read_only,
        }
    }

    /// 处理全局键盘快捷键
    fn handle_global_shortcuts(&mut self, ctx: &egui::Context) {
        ctx.input(|i| {
            // Ctrl+/ 或 Cmd+/ 快速切换到AI助手输入框
            let cmd_slash = if cfg!(target_os = "macos") {
                i.modifiers.mac_cmd && i.key_pressed(egui::Key::Slash)
            } else {
                i.modifiers.ctrl && i.key_pressed(egui::Key::Slash)
            };

            if cmd_slash {
                // 打开右侧边栏（如果未打开）
                if !self.show_right_sidebar {
                    self.show_right_sidebar = true;
                }
                // 设置AI助手输入框获得焦点
                self.ai_assist_state.should_focus_chat = true;
            }
        });
    }
}