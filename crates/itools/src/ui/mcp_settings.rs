use std::collections::HashMap;
use std::path::PathBuf;
use uuid::Uuid;
use egui::{Context, Ui, RichText, Color32, Button, TextEdit, ComboBox, ScrollArea};
use tokio::sync::mpsc;


use crate::mcp::{
    McpServerManager, McpServerConfig,
    server_manager::TransportConfig,
    rmcp_client::{McpEvent, ServerCapabilities, TestResult}
};

/// MCP Settings UI component
#[derive(Debug)]
pub struct McpSettingsUi {
    /// Server manager
    server_manager: McpServerManager,
    
    /// UI state
    ui_state: McpUiState,
    
    /// Event receiver for MCP events
    event_receiver: Option<mpsc::UnboundedReceiver<McpEvent>>,
    
    /// Selected server for editing
    selected_server: Option<Uuid>,
    
    /// New server configuration being created
    new_server_config: McpServerConfig,
    
    /// Directory expansion states
    directory_expanded: HashMap<String, bool>,

    /// Track if this is the first render for auto-refresh
    first_render: bool,

    /// Server capabilities cache (server_id -> capabilities)
    server_capabilities: HashMap<Uuid, ServerCapabilities>,

    /// Pending tool test requests (server_id -> server_name)
    pending_tool_tests: HashMap<Uuid, String>,
}

/// UI state for MCP settings
#[derive(Debug)]
struct McpUiState {
    /// Show add server dialog
    show_add_server: bool,

    /// Show import dialog
    show_import_dialog: bool,

    /// Show export dialog
    show_export_dialog: bool,



    /// Show edit server dialog
    show_edit_server: bool,

    /// Edit server JSON text
    edit_server_json_text: String,

    /// Status message
    status_message: Option<String>,

    /// Error message
    error_message: Option<String>,

    /// Server-specific status messages (server_id -> message)
    server_status_messages: HashMap<Uuid, String>,

    /// Server-specific error messages (server_id -> error)
    server_error_messages: HashMap<Uuid, String>,

    /// Server test output details (server_id -> (stdout, stderr))
    server_test_outputs: HashMap<Uuid, (String, String)>,

    /// Expanded state for error details (server_id -> expanded)
    error_details_expanded: HashMap<Uuid, bool>,

    /// Import file path
    import_file_path: String,

    /// Export file path
    export_file_path: String,

    /// Selected export directory
    export_directory: Option<String>,

    /// Add server input mode: true for JSON, false for form
    add_server_json_mode: bool,

    /// JSON input text for adding server
    add_server_json_text: String,

    /// Use real rmcp client instead of mock
    use_real_rmcp: bool,

    /// Tool testing dialog state
    tool_test_dialog: Option<ToolTestDialog>,

    /// Functionality testing dialog state
    functionality_test_dialog: Option<FunctionalityTestDialog>,

    /// Show delete confirmation dialog
    show_delete_confirmation: bool,

    /// Server to be deleted (for confirmation)
    server_to_delete: Option<Uuid>,

    /// Connection test result for add server dialog
    connection_test_result: Option<TestResult>,

    /// Tested capabilities from connection test
    tested_capabilities: Option<ServerCapabilities>,
}

/// Dialog for testing MCP server tools
#[derive(Debug)]
struct ToolTestDialog {
    server_id: Uuid,
    server_name: String,
    capabilities: ServerCapabilities,
    selected_category: TestCategory,
    selected_tool_index: Option<usize>,
    selected_resource_index: Option<usize>,
    selected_prompt_index: Option<usize>,
    parameter_inputs: HashMap<String, String>,
    test_result: Option<ToolTestResult>,
    is_testing: bool,
    /// Test execution frame counter to prevent UI blocking
    test_frame_counter: u32,
    /// Current active tab in the result display
    active_tab: TestResultTab,
}

/// Dialog for testing MCP server functionality (health status testing)
#[derive(Debug)]
struct FunctionalityTestDialog {
    server_id: Uuid,
    server_name: String,
    test_result: Option<TestResult>,
    is_testing: bool,
    /// Test execution frame counter to prevent UI blocking
    test_frame_counter: u32,
    /// Current active tab in the result display
    active_tab: FunctionalityTestTab,
    /// Available tools for testing
    available_tools: Vec<crate::mcp::rmcp_client::ToolInfo>,
    /// Selected tool index for testing
    selected_tool_index: Option<usize>,
    /// Parameter inputs for the selected tool
    parameter_inputs: std::collections::HashMap<String, String>,
    /// Test execution phase
    test_phase: FunctionalityTestPhase,
}

/// Tabs for functionality test result display
#[derive(Debug, Clone, Copy, PartialEq)]
enum FunctionalityTestTab {
    Summary,
    Details,
    Output,
}

/// Phases of functionality testing
#[derive(Debug, Clone, Copy, PartialEq)]
enum FunctionalityTestPhase {
    /// Selecting tool and configuring parameters
    Setup,
    /// Executing the test
    Testing,
    /// Showing test results
    Results,
}

/// Categories of testable items
#[derive(Debug, Clone, Copy, PartialEq)]
enum TestCategory {
    Tools,
    Resources,
    Prompts,
}

/// Tabs for displaying test results
#[derive(Debug, Clone, Copy, PartialEq)]
enum TestResultTab {
    Summary,
    Request,
    Response,
}



/// Result of tool testing
#[derive(Debug, Clone)]
struct ToolTestResult {
    success: bool,
    error: Option<String>,
    duration: std::time::Duration,
    /// Summary of the test result
    summary: String,
    /// Raw MCP request sent
    request: Option<String>,
    /// Raw MCP response received
    response: Option<String>,
    /// Additional debug information
    debug_info: Option<String>,
}

impl McpSettingsUi {
    /// Create a new MCP settings UI
    pub fn new(config_path: PathBuf) -> Self {
        let server_manager = McpServerManager::new(config_path);
        
        Self {
            server_manager,
            ui_state: McpUiState::default(),
            event_receiver: None,
            selected_server: None,
            new_server_config: McpServerConfig::default(),
            directory_expanded: HashMap::new(),
            first_render: true,
            server_capabilities: HashMap::new(),
            pending_tool_tests: HashMap::new(),
        }
    }



    /// Initialize the MCP settings UI synchronously
    pub fn initialize_sync(&mut self) -> anyhow::Result<()> {
        // Initialize server manager synchronously
        self.server_manager.initialize_sync()?;

        // Setup event channel
        let (sender, receiver) = mpsc::unbounded_channel();
        self.event_receiver = Some(receiver);

        // Add event sender in server manager (不覆盖现有的发送器)
        self.server_manager.add_event_sender(sender);

        log::info!("MCP settings UI initialized synchronously with event channel");
        Ok(())
    }

    /// Render the MCP settings UI
    pub fn render(&mut self, ctx: &Context, ui: &mut Ui) {
        // Initialize if not already done
        if self.event_receiver.is_none() {
            log::info!("Initializing MCP settings UI on first render");
            if let Err(e) = self.initialize_sync() {
                log::error!("Failed to initialize MCP settings UI: {}", e);
                ui.colored_label(egui::Color32::RED, format!("初始化失败: {}", e));
                return;
            }
        }

        // Auto-refresh on first render or when explicitly requested
        if self.should_auto_refresh() {
            self.refresh_server_list();
        }

        // Process MCP events
        self.process_events();

        ui.heading("MCP Hub");
        ui.separator();

        // Toolbar
        ui.horizontal(|ui| {
            if ui.button("➕ 添加服务器").clicked() {
                self.ui_state.show_add_server = true;
                self.ui_state.add_server_json_mode = false;
                self.ui_state.add_server_json_text.clear();
                // 清理之前的测试结果
                self.ui_state.connection_test_result = None;
                self.ui_state.tested_capabilities = None;
                self.ui_state.error_message = None;

                // 创建带有合理默认值的新配置
                self.new_server_config = McpServerConfig {
                    id: uuid::Uuid::new_v4(),
                    name: "Filesystem Server".to_string(),
                    description: Some("MCP Filesystem Server - 文件系统操作".to_string()),
                    transport: TransportConfig::Command {
                        command: "npx".to_string(),
                        args: vec![
                            "-y".to_string(),
                            "@modelcontextprotocol/server-filesystem".to_string(),
                            ".".to_string(), // 使用当前目录，与Inspector一致
                        ],
                        env: HashMap::new(),
                    },
                    enabled: false,
                    auto_start: false,
                    directory: "自定义".to_string(),
                    metadata: HashMap::new(),
                    capabilities: None,
                    last_health_status: None,
                    last_test_time: None,
                    last_test_success: None,
                };
            }

            if ui.button("📁 导入配置").clicked() {
                self.ui_state.show_import_dialog = true;
            }

            if ui.button("💾 导出配置").clicked() {
                self.ui_state.show_export_dialog = true;
            }

            ui.separator();

            if ui.button("🔄 刷新列表").clicked() {
                // Refresh server list
                self.refresh_server_list();
            }

            ui.separator();

            // Real rmcp toggle
            let rmcp_text = if self.ui_state.use_real_rmcp {
                "🔧 真实 rmcp"
            } else {
                "🎭 模拟模式"
            };

            if ui.button(rmcp_text).on_hover_text("切换 rmcp 客户端模式").clicked() {
                self.ui_state.use_real_rmcp = !self.ui_state.use_real_rmcp;
                self.ui_state.status_message = Some(format!(
                    "已切换到 {} 模式",
                    if self.ui_state.use_real_rmcp { "真实 rmcp" } else { "模拟" }
                ));
            }
        });

        ui.separator();

        // Status messages
        if let Some(message) = &self.ui_state.status_message {
            ui.colored_label(Color32::GREEN, message);
        }

        if let Some(error) = &self.ui_state.error_message {
            ui.colored_label(Color32::RED, error);
        }

        // Server directory tree
        egui::ScrollArea::vertical().show(ui, |ui| {
            self.render_server_directories(ui);
        });

        // Dialogs
        self.render_add_server_dialog(ctx);
        self.render_edit_server_dialog(ctx);
        self.render_import_dialog(ctx);
        self.render_export_dialog(ctx);
        self.render_delete_confirmation_dialog(ctx);

        self.render_tool_test_dialog(ctx);
        self.render_functionality_test_dialog(ctx);
    }

    /// Render server directories in a tree structure
    fn render_server_directories(&mut self, ui: &mut Ui) {
        let directories = self.server_manager.get_server_directories();

        for directory in directories {
            let expanded = self.directory_expanded.get(&directory.name).copied().unwrap_or(false);

            ui.horizontal(|ui| {
                let expand_button = if expanded { "📂" } else { "📁" };

                // Make both icon and directory name clickable for expansion
                let icon_response = ui.button(expand_button);
                let name_response = ui.selectable_label(false, RichText::new(&directory.name).strong());

                if icon_response.clicked() || name_response.clicked() {
                    self.directory_expanded.insert(directory.name.clone(), !expanded);
                }

                ui.label(format!("({} servers)", directory.servers.len()));
            });

            if expanded {
                ui.indent("servers", |ui| {
                    for server_config in &directory.servers {
                        self.render_server_item(ui, server_config);
                    }
                });
            }

            ui.separator();
        }
    }

    /// Render a single server item
    fn render_server_item(&mut self, ui: &mut Ui, config: &McpServerConfig) {
        ui.vertical(|ui| {
            // Main server row
            ui.horizontal(|ui| {
                // Health status indicator (traffic light system)
                if let Some(server_id) = self.find_server_id_by_config(config) {
                    let health_status = self.server_manager.get_server_health_status(server_id);
                    let (health_color, health_icon, health_tooltip) = match health_status {
                        Some(crate::mcp::rmcp_client::ServerHealthStatus::Red) => {
                            (Color32::RED, "🔴", "红灯：服务器配置已添加/修改，需要测试")
                        }
                        Some(crate::mcp::rmcp_client::ServerHealthStatus::Yellow) => {
                            (Color32::YELLOW, "🟡", "黄灯：服务器已连接，需要测试功能")
                        }
                        Some(crate::mcp::rmcp_client::ServerHealthStatus::Green) => {
                            (Color32::GREEN, "🟢", "绿灯：服务器已测试通过，可以使用")
                        }
                        None => {
                            (Color32::GRAY, "⚪", "未知状态")
                        }
                    };
                    ui.colored_label(health_color, health_icon).on_hover_text(health_tooltip);
                } else {
                    ui.colored_label(Color32::GRAY, "⚪").on_hover_text("未知状态");
                }

                // Connection status indicator
                let status_color = if config.enabled {
                    Color32::GREEN
                } else {
                    Color32::GRAY
                };
                ui.colored_label(status_color, "●");

                // Server name and description
                ui.vertical(|ui| {
                    ui.label(RichText::new(&config.name).strong());
                    if let Some(desc) = &config.description {
                        ui.label(RichText::new(desc).small().color(Color32::GRAY));
                    }

                    // Show transport type
                    let transport_info = match &config.transport {
                        TransportConfig::Command { command, args, .. } => {
                            format!("📟 stdio: {} {}", command, args.join(" "))
                        }
                        TransportConfig::Tcp { host, port } => {
                            format!("🌐 tcp: {}:{}", host, port)
                        }
                        TransportConfig::Unix { socket_path } => {
                            format!("🔌 unix: {}", socket_path)
                        }
                        TransportConfig::WebSocket { url } => {
                            format!("🌍 sse: {}", url)
                        }
                    };
                    ui.label(RichText::new(transport_info).small().color(Color32::DARK_GRAY));
                });

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // Action buttons
                    if ui.small_button("✏️").on_hover_text("编辑配置").clicked() {
                        // Find server ID and show edit dialog
                        if let Some(server_id) = self.find_server_id_by_config(config) {
                            self.selected_server = Some(server_id);
                            self.ui_state.show_edit_server = true;
                            // Load current config as JSON
                            self.ui_state.edit_server_json_text = serde_json::to_string_pretty(config).unwrap_or_default();
                        }
                    }

                    // Functionality test button (for health status)
                    if ui.small_button("🧪").on_hover_text("测试服务器功能 (影响健康状态)").clicked() {
                        if let Some(server_id) = self.find_server_id_by_config(config) {
                            self.test_server_functionality(server_id);
                        }
                    }

                    // Tool testing button (for individual tool testing)
                    if ui.small_button("🔧").on_hover_text("测试工具 (不影响健康状态)").clicked() {
                        if let Some(server_id) = self.find_server_id_by_config(config) {
                            self.test_server_tools(server_id, config);
                        }
                    }

                    let connect_button = if config.enabled {
                        Button::new("🔌").fill(Color32::from_rgb(100, 200, 100))
                    } else {
                        Button::new("⚡").fill(Color32::from_rgb(200, 100, 100))
                    };

                    if ui.add(connect_button).on_hover_text("连接/断开").clicked() {
                        // Toggle server connection
                        if let Some(server_id) = self.find_server_id_by_config(config) {
                            self.toggle_server_connection(server_id, config.enabled);
                        }
                    }
                });
            });

            // Server-specific status/error messages
            if let Some(server_id) = self.find_server_id_by_config(config) {
                if let Some(status_msg) = self.ui_state.server_status_messages.get(&server_id) {
                    ui.indent("server_status", |ui| {
                        ui.colored_label(Color32::GREEN, format!("✅ {}", status_msg));

                        // Show collapsible success details if available
                        if let Some((stdout, stderr)) = self.ui_state.server_test_outputs.get(&server_id) {
                            if !stdout.is_empty() || !stderr.is_empty() {
                                let expanded = self.ui_state.error_details_expanded.get(&server_id).copied().unwrap_or(false);

                                ui.horizontal(|ui| {
                                    if ui.small_button(if expanded { "🔽 隐藏详情" } else { "🔼 显示详情" }).clicked() {
                                        self.ui_state.error_details_expanded.insert(server_id, !expanded);
                                    }
                                    ui.label(egui::RichText::new("(点击查看测试输出)").color(egui::Color32::GRAY));
                                });

                                if expanded {
                                    ui.separator();

                                    // Show stdout if not empty
                                    if !stdout.is_empty() {
                                        ui.label(egui::RichText::new("标准输出 (stdout):").strong().color(egui::Color32::DARK_GREEN));
                                        egui::ScrollArea::vertical()
                                            .max_height(100.0)
                                            .show(ui, |ui| {
                                                ui.add(
                                                    egui::TextEdit::multiline(&mut stdout.as_str())
                                                        .desired_width(f32::INFINITY)
                                                        .font(egui::TextStyle::Monospace)
                                                        .desired_rows(5)
                                                );
                                            });
                                        ui.add_space(5.0);
                                    }

                                    // Show stderr if not empty (but with different color for success case)
                                    if !stderr.is_empty() {
                                        ui.label(egui::RichText::new("错误输出 (stderr):").strong().color(egui::Color32::DARK_BLUE));
                                        egui::ScrollArea::vertical()
                                            .max_height(100.0)
                                            .show(ui, |ui| {
                                                ui.add(
                                                    egui::TextEdit::multiline(&mut stderr.as_str())
                                                        .desired_width(f32::INFINITY)
                                                        .font(egui::TextStyle::Monospace)
                                                        .desired_rows(5)
                                                );
                                            });
                                    }

                                    ui.separator();
                                }
                            }
                        }
                    });
                }

                if let Some(error_msg) = self.ui_state.server_error_messages.get(&server_id) {
                    ui.indent("server_error", |ui| {
                        ui.colored_label(Color32::RED, format!("❌ {}", error_msg));

                        // Show collapsible error details if available
                        if let Some((stdout, stderr)) = self.ui_state.server_test_outputs.get(&server_id) {
                            if !stdout.is_empty() || !stderr.is_empty() {
                                let expanded = self.ui_state.error_details_expanded.get(&server_id).copied().unwrap_or(false);

                                ui.horizontal(|ui| {
                                    if ui.small_button(if expanded { "🔽 隐藏详情" } else { "🔼 显示详情" }).clicked() {
                                        self.ui_state.error_details_expanded.insert(server_id, !expanded);
                                    }
                                    ui.label(egui::RichText::new("(点击查看测试输出)").color(egui::Color32::GRAY));
                                });

                                if expanded {
                                    ui.separator();

                                    // Show stdout if not empty
                                    if !stdout.is_empty() {
                                        ui.label(egui::RichText::new("标准输出 (stdout):").strong());
                                        egui::ScrollArea::vertical()
                                            .max_height(100.0)
                                            .show(ui, |ui| {
                                                ui.add(
                                                    egui::TextEdit::multiline(&mut stdout.as_str())
                                                        .desired_width(f32::INFINITY)
                                                        .font(egui::TextStyle::Monospace)
                                                        .desired_rows(5)
                                                );
                                            });
                                        ui.add_space(5.0);
                                    }

                                    // Show stderr if not empty
                                    if !stderr.is_empty() {
                                        ui.label(egui::RichText::new("错误输出 (stderr):").strong().color(egui::Color32::LIGHT_RED));
                                        egui::ScrollArea::vertical()
                                            .max_height(100.0)
                                            .show(ui, |ui| {
                                                ui.add(
                                                    egui::TextEdit::multiline(&mut stderr.as_str())
                                                        .desired_width(f32::INFINITY)
                                                        .font(egui::TextStyle::Monospace)
                                                        .desired_rows(5)
                                                );
                                            });
                                    }

                                    ui.separator();
                                }
                            }
                        }
                    });
                }

                // Show server capabilities if available (regardless of enabled status)
                if let Some(capabilities) = self.server_capabilities.get(&server_id).cloned() {
                    log::debug!("Rendering capabilities for server {}: {} tools, {} resources, {} prompts",
                               server_id, capabilities.tools.len(), capabilities.resources.len(), capabilities.prompts.len());
                    self.render_server_capabilities(ui, &capabilities);
                } else {
                    log::debug!("No capabilities found for server {} in UI cache", server_id);
                }
            }
        });
    }

    /// Render add server dialog
    fn render_add_server_dialog(&mut self, ctx: &Context) {
        if !self.ui_state.show_add_server {
            return;
        }

        egui::Window::new("添加 MCP 服务器")
            .collapsible(false)
            .resizable(true)
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    // Input mode toggle
                    ui.horizontal(|ui| {
                        ui.label("输入模式:");
                        ui.radio_value(&mut self.ui_state.add_server_json_mode, false, "表单输入");
                        ui.radio_value(&mut self.ui_state.add_server_json_mode, true, "JSON 输入");
                    });

                    ui.separator();

                    if self.ui_state.add_server_json_mode {
                        // JSON input mode
                        self.render_json_input_mode(ui);
                    } else {
                        // Form input mode
                        self.render_form_input_mode(ui);
                    }

                    ui.separator();

                    // 显示连接测试结果
                    if let Some(test_result) = &self.ui_state.connection_test_result {
                        ui.group(|ui| {
                            ui.vertical(|ui| {
                                if test_result.success {
                                    ui.colored_label(Color32::GREEN, "✅ 连接测试成功");
                                    if let Some(capabilities) = &self.ui_state.tested_capabilities {
                                        ui.label(format!("🔧 工具: {}", capabilities.tools.len()));
                                        ui.label(format!("📁 资源: {}", capabilities.resources.len()));
                                        ui.label(format!("💬 提示: {}", capabilities.prompts.len()));
                                    }
                                } else {
                                    ui.colored_label(Color32::RED, "❌ 连接测试失败");
                                    if let Some(error) = &test_result.error_message {
                                        ui.label(RichText::new(error).color(Color32::RED).small());
                                    }
                                }
                            });
                        });
                        ui.separator();
                    }

                    // Buttons
                    ui.horizontal(|ui| {
                        // 连接测试按钮
                        if ui.button("🔗 连接").clicked() {
                            self.test_connection_before_add();
                        }

                        ui.separator();

                        // 只有连接测试成功后才能添加服务器
                        let can_add = self.ui_state.connection_test_result.is_some() &&
                                     self.ui_state.connection_test_result.as_ref().unwrap().success;

                        ui.add_enabled_ui(can_add, |ui| {
                            if ui.button("添加服务器").clicked() {
                            let config_result = if self.ui_state.add_server_json_mode {
                                // Parse JSON input
                                self.parse_json_config()
                            } else {
                                // Use form input
                                Ok(self.new_server_config.clone())
                            };

                            match config_result {
                                Ok(mut config) => {
                                    // 如果连接测试成功，将能力信息添加到配置中
                                    log::info!("🔍 检查连接测试结果...");
                                    if let Some(test_result) = &self.ui_state.connection_test_result {
                                        log::info!("📊 连接测试结果存在，成功状态: {}", test_result.success);
                                        if test_result.success {
                                            if let Some(capabilities) = &self.ui_state.tested_capabilities {
                                                log::info!("🎯 测试能力信息存在 - 工具:{}, 资源:{}, 提示:{}",
                                                    capabilities.tools.len(), capabilities.resources.len(), capabilities.prompts.len());
                                                // 将能力信息序列化并保存到配置中
                                                match serde_json::to_value(capabilities) {
                                                    Ok(capabilities_json) => {
                                                        config.capabilities = Some(capabilities_json.clone());
                                                        log::info!("✅ 将测试获取的能力信息添加到服务器配置中 - 工具:{}, 资源:{}, 提示:{}",
                                                            capabilities.tools.len(), capabilities.resources.len(), capabilities.prompts.len());
                                                        log::info!("📄 序列化后的能力JSON: {}", serde_json::to_string_pretty(&capabilities_json).unwrap_or_default());
                                                    }
                                                    Err(e) => {
                                                        log::error!("❌ 序列化能力信息失败: {}", e);
                                                    }
                                                }
                                            } else {
                                                log::warn!("⚠️ 连接测试成功但没有能力信息");
                                            }
                                        } else {
                                            log::warn!("⚠️ 连接测试失败，不添加能力信息");
                                        }
                                    } else {
                                        log::warn!("⚠️ 没有连接测试结果");
                                    }

                                    // Validate configuration
                                    match self.server_manager.validate_server_config(&config) {
                                        Ok(()) => {
                                            // Add server using runtime
                                            let result = if let Ok(handle) = tokio::runtime::Handle::try_current() {
                                                handle.block_on(self.server_manager.add_server(config))
                                                    .map_err(|e| e.to_string())
                                            } else {
                                                match tokio::runtime::Runtime::new() {
                                                    Ok(rt) => {
                                                        rt.block_on(self.server_manager.add_server(config))
                                                            .map_err(|e| e.to_string())
                                                    }
                                                    Err(e) => {
                                                        Err(format!("无法创建异步运行时: {}", e))
                                                    }
                                                }
                                            };

                                            match result {
                                                Ok(server_id) => {
                                                    self.ui_state.status_message = Some(format!("服务器添加成功: {}", server_id));
                                                    self.ui_state.error_message = None;
                                                    self.ui_state.show_add_server = false;
                                                    self.new_server_config = McpServerConfig::default();
                                                    self.ui_state.add_server_json_text.clear();
                                                    // 清理连接测试状态
                                                    self.ui_state.connection_test_result = None;
                                                    self.ui_state.tested_capabilities = None;
                                                }
                                                Err(e) => {
                                                    self.ui_state.error_message = Some(format!("添加服务器失败: {}", e));
                                                }
                                            }
                                        }
                                        Err(e) => {
                                            self.ui_state.error_message = Some(format!("配置验证失败: {}", e));
                                        }
                                    }
                                }
                                Err(e) => {
                                    self.ui_state.error_message = Some(format!("JSON 解析失败: {}", e));
                                }
                            }
                        }
                        });

                        if ui.button("取消").clicked() {
                            self.ui_state.show_add_server = false;
                            self.new_server_config = McpServerConfig::default();
                            self.ui_state.add_server_json_text.clear();
                            self.ui_state.add_server_json_mode = false;
                            // 清理连接测试状态
                            self.ui_state.connection_test_result = None;
                            self.ui_state.tested_capabilities = None;
                        }
                    });
                });
            });
    }

    /// Render edit server dialog
    fn render_edit_server_dialog(&mut self, ctx: &Context) {
        if !self.ui_state.show_edit_server {
            return;
        }

        egui::Window::new("编辑 MCP 服务器")
            .collapsible(false)
            .resizable(true)
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    ui.label("编辑服务器 JSON 配置:");
                    ui.separator();

                    // JSON text area
                    egui::ScrollArea::vertical()
                        .max_height(400.0)
                        .show(ui, |ui| {
                            ui.add(
                                TextEdit::multiline(&mut self.ui_state.edit_server_json_text)
                                    .desired_width(f32::INFINITY)
                                    .desired_rows(20)
                                    .font(egui::TextStyle::Monospace)
                            );
                        });

                    ui.separator();

                    // Buttons
                    ui.horizontal(|ui| {
                        if ui.button("保存更改").clicked() {
                            if let Some(server_id) = self.selected_server {
                                match serde_json::from_str::<McpServerConfig>(&self.ui_state.edit_server_json_text) {
                                    Ok(mut updated_config) => {
                                        // Preserve the original server ID
                                        updated_config.id = server_id;

                                        // Update server using runtime
                                        let result = if let Ok(handle) = tokio::runtime::Handle::try_current() {
                                            handle.block_on(self.server_manager.update_server(server_id, updated_config))
                                                .map_err(|e| e.to_string())
                                        } else {
                                            match tokio::runtime::Runtime::new() {
                                                Ok(rt) => {
                                                    rt.block_on(self.server_manager.update_server(server_id, updated_config))
                                                        .map_err(|e| e.to_string())
                                                }
                                                Err(e) => Err(format!("无法创建异步运行时: {}", e))
                                            }
                                        };

                                        match result {
                                            Ok(()) => {
                                                self.ui_state.server_status_messages.insert(server_id, "服务器配置更新成功".to_string());
                                                self.ui_state.server_error_messages.remove(&server_id);
                                                self.ui_state.show_edit_server = false;
                                                self.ui_state.edit_server_json_text.clear();
                                            }
                                            Err(e) => {
                                                self.ui_state.server_error_messages.insert(server_id, format!("更新失败: {}", e));
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        if let Some(server_id) = self.selected_server {
                                            self.ui_state.server_error_messages.insert(server_id, format!("JSON 解析失败: {}", e));
                                        }
                                    }
                                }
                            }
                        }

                        ui.separator();

                        // Delete button with warning color
                        if ui.add(Button::new("🗑️ 删除服务器").fill(Color32::from_rgb(200, 100, 100))).clicked() {
                            if let Some(server_id) = self.selected_server {
                                self.ui_state.server_to_delete = Some(server_id);
                                self.ui_state.show_delete_confirmation = true;
                            }
                        }

                        if ui.button("取消").clicked() {
                            self.ui_state.show_edit_server = false;
                            self.ui_state.edit_server_json_text.clear();
                        }
                    });
                });
            });
    }

    /// Render delete confirmation dialog
    fn render_delete_confirmation_dialog(&mut self, ctx: &Context) {
        if !self.ui_state.show_delete_confirmation {
            return;
        }

        egui::Window::new("确认删除")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    ui.add_space(10.0);

                    // Warning icon and message
                    ui.horizontal(|ui| {
                        ui.colored_label(Color32::RED, "⚠️");
                        ui.label(RichText::new("确认删除服务器").strong().color(Color32::RED));
                    });

                    ui.add_space(10.0);

                    // Get server name for confirmation
                    let server_name = if let Some(server_id) = self.ui_state.server_to_delete {
                        self.get_server_name_by_id(server_id).unwrap_or("未知服务器".to_string())
                    } else {
                        "未知服务器".to_string()
                    };

                    ui.label(format!("您确定要删除服务器 \"{}\" 吗？", server_name));
                    ui.colored_label(Color32::YELLOW, "⚠️ 此操作无法撤销！");

                    ui.add_space(15.0);

                    // Buttons
                    ui.horizontal(|ui| {
                        // Delete button (red)
                        if ui.add(Button::new("🗑️ 确认删除").fill(Color32::from_rgb(200, 50, 50))).clicked() {
                            if let Some(server_id) = self.ui_state.server_to_delete {
                                self.delete_server(server_id);
                            }
                            self.ui_state.show_delete_confirmation = false;
                            self.ui_state.server_to_delete = None;
                            self.ui_state.show_edit_server = false;
                            self.ui_state.edit_server_json_text.clear();
                        }

                        ui.add_space(10.0);

                        // Cancel button (gray)
                        if ui.add(Button::new("取消").fill(Color32::from_rgb(120, 120, 120))).clicked() {
                            self.ui_state.show_delete_confirmation = false;
                            self.ui_state.server_to_delete = None;
                        }
                    });

                    ui.add_space(10.0);
                });
            });
    }

    /// Render form input mode for adding server
    fn render_form_input_mode(&mut self, ui: &mut Ui) {
        // Server name
        ui.horizontal(|ui| {
            ui.label("名称:");
            ui.add(TextEdit::singleline(&mut self.new_server_config.name).desired_width(200.0));
        });

        // Description
        ui.horizontal(|ui| {
            ui.label("描述:");
            let mut desc = self.new_server_config.description.clone().unwrap_or_default();
            ui.add(TextEdit::singleline(&mut desc).desired_width(200.0));
            self.new_server_config.description = if desc.is_empty() { None } else { Some(desc) };
        });

        // Directory
        ui.horizontal(|ui| {
            ui.label("目录:");
            ui.add(TextEdit::singleline(&mut self.new_server_config.directory).desired_width(200.0));
        });

        ui.separator();

        // Transport configuration
        ui.label("传输配置:");
        self.render_transport_config_editor(ui);

        ui.separator();

        // Options
        ui.checkbox(&mut self.new_server_config.enabled, "启用");
        ui.checkbox(&mut self.new_server_config.auto_start, "自动启动");
    }

    /// Render JSON input mode for adding server
    fn render_json_input_mode(&mut self, ui: &mut Ui) {
        // Additional metadata fields for JSON mode
        ui.horizontal(|ui| {
            ui.label("目录:");
            ui.add(TextEdit::singleline(&mut self.new_server_config.directory).desired_width(200.0));
        });

        ui.horizontal(|ui| {
            ui.label("描述:");
            let mut desc = self.new_server_config.description.clone().unwrap_or_default();
            ui.add(TextEdit::singleline(&mut desc).desired_width(200.0));
            self.new_server_config.description = if desc.is_empty() { None } else { Some(desc) };
        });

        ui.separator();

        ui.label("JSON 配置:");
        ui.label("请输入标准 MCP 服务器配置 JSON:");

        // JSON text area
        egui::ScrollArea::vertical()
            .max_height(250.0)
            .show(ui, |ui| {
                ui.add(
                    TextEdit::multiline(&mut self.ui_state.add_server_json_text)
                        .desired_width(f32::INFINITY)
                        .desired_rows(12)
                        .font(egui::TextStyle::Monospace)
                );
            });

        ui.separator();

        // Additional options
        ui.checkbox(&mut self.new_server_config.enabled, "启用");
        ui.checkbox(&mut self.new_server_config.auto_start, "自动启动");

        ui.separator();

        // Example JSON
        ui.horizontal(|ui| {
            if ui.button("📋 stdio 示例").clicked() {
                self.ui_state.add_server_json_text = self.get_example_json();
            }
            if ui.button("🌐 SSE 示例").clicked() {
                self.ui_state.add_server_json_text = self.get_sse_example_json();
            }
        });

        ui.collapsing("JSON 格式说明", |ui| {
            ui.label("支持两种 JSON 格式:");
            ui.separator();

            ui.label("1. 标准 MCP 格式 (推荐):");
            ui.label("• mcpServers: 服务器配置对象 (必填)");
            ui.label("  • 服务器名称: 服务器配置 (必填)");

            ui.label("    stdio 传输类型:");
            ui.label("      • command: 启动命令 (必填)");
            ui.label("      • args: 命令参数数组 (可选)");
            ui.label("      • env: 环境变量对象 (可选)");

            ui.label("    SSE 传输类型:");
            ui.label("      • transport: \"sse\" (必填)");
            ui.label("      • url: SSE 端点 URL (必填)");
            ui.label("      • env: 环境变量对象 (可选)");

            ui.label("• 目录、描述、启用状态通过上方表单字段设置");

            ui.separator();

            ui.label("2. 内部格式 (兼容性):");
            ui.label("• name: 服务器名称 (必填)");
            ui.label("• description: 服务器描述 (可选)");
            ui.label("• directory: 目录名称 (可选)");
            ui.label("• transport: 传输配置 (必填)");
            ui.label("• enabled: 是否启用 (可选)");
            ui.label("• auto_start: 是否自动启动 (可选)");
            ui.label("• 内部格式会忽略上方表单字段");
        });
    }

    /// Parse JSON configuration
    fn parse_json_config(&self) -> Result<McpServerConfig, String> {
        if self.ui_state.add_server_json_text.trim().is_empty() {
            return Err("JSON 配置不能为空".to_string());
        }

        // Try to parse as standard MCP format first
        if let Ok(mcp_config) = self.parse_standard_mcp_format() {
            return Ok(mcp_config);
        }

        // Fallback to internal format for backward compatibility
        match serde_json::from_str::<McpServerConfig>(&self.ui_state.add_server_json_text) {
            Ok(mut config) => {
                // Ensure the config has a unique ID
                if config.id == Uuid::nil() {
                    config.id = Uuid::new_v4();
                }
                Ok(config)
            }
            Err(e) => Err(format!("JSON 解析错误: {}。请使用标准 MCP 格式或内部格式。", e))
        }
    }

    /// Parse standard MCP configuration format
    fn parse_standard_mcp_format(&self) -> Result<McpServerConfig, String> {
        #[derive(serde::Deserialize)]
        struct StandardMcpConfig {
            #[serde(rename = "mcpServers")]
            mcp_servers: std::collections::HashMap<String, StandardServerConfig>,
        }

        #[derive(serde::Deserialize)]
        struct StandardServerConfig {
            // stdio transport
            command: Option<String>,
            args: Option<Vec<String>>,
            env: Option<std::collections::HashMap<String, String>>,

            // sse transport
            transport: Option<String>,
            url: Option<String>,
        }

        let standard_config: StandardMcpConfig = serde_json::from_str(&self.ui_state.add_server_json_text)
            .map_err(|e| format!("标准 MCP 格式解析失败: {}", e))?;

        if standard_config.mcp_servers.is_empty() {
            return Err("mcpServers 对象不能为空".to_string());
        }

        if standard_config.mcp_servers.len() > 1 {
            return Err("当前只支持添加单个服务器，请在 mcpServers 中只包含一个服务器配置".to_string());
        }

        let (server_name, server_config) = standard_config.mcp_servers.into_iter().next().unwrap();

        // Determine transport type and create appropriate config
        let transport_config = if let Some(transport) = &server_config.transport {
            match transport.as_str() {
                "sse" => {
                    if let Some(url) = &server_config.url {
                        TransportConfig::WebSocket {
                            url: url.clone(),
                        }
                    } else {
                        return Err("SSE 传输类型需要提供 url 字段".to_string());
                    }
                }
                "stdio" => {
                    if let Some(command) = &server_config.command {
                        TransportConfig::Command {
                            command: command.clone(),
                            args: server_config.args.unwrap_or_default(),
                            env: server_config.env.unwrap_or_default(),
                        }
                    } else {
                        return Err("stdio 传输类型需要提供 command 字段".to_string());
                    }
                }
                _ => {
                    return Err(format!("不支持的传输类型: {}。支持的类型: stdio, sse", transport));
                }
            }
        } else if let Some(command) = &server_config.command {
            // Backward compatibility: if no transport specified but command exists, assume stdio
            TransportConfig::Command {
                command: command.clone(),
                args: server_config.args.unwrap_or_default(),
                env: server_config.env.unwrap_or_default(),
            }
        } else {
            return Err("必须指定 transport 字段或提供 command 字段".to_string());
        };

        Ok(McpServerConfig {
            id: Uuid::new_v4(),
            name: server_name,
            description: self.new_server_config.description.clone(),
            transport: transport_config,
            enabled: self.new_server_config.enabled,
            auto_start: self.new_server_config.auto_start,
            directory: if self.new_server_config.directory.is_empty() {
                "导入".to_string()
            } else {
                self.new_server_config.directory.clone()
            },
            metadata: HashMap::new(),
            capabilities: None,
            last_health_status: None,
            last_test_time: None,
            last_test_success: None,
        })
    }

    /// Get example JSON configuration
    fn get_example_json(&self) -> String {
        // Provide examples for both stdio and SSE transport types
        serde_json::to_string_pretty(&serde_json::json!({
            "mcpServers": {
                "filesystem": {
                    "command": "npx",
                    "args": [
                        "-y",
                        "@modelcontextprotocol/server-filesystem",
                        "/Users/username/Desktop",
                        "/Users/username/Downloads"
                    ],
                    "env": {
                        "NODE_ENV": "production"
                    }
                }
            }
        })).unwrap_or_default()
    }

    /// Get SSE example JSON configuration
    fn get_sse_example_json(&self) -> String {
        serde_json::to_string_pretty(&serde_json::json!({
            "mcpServers": {
                "photos": {
                    "transport": "sse",
                    "url": "http://localhost:3001/sse",
                    "env": {
                        "TRANSPORT": "sse"
                    }
                }
            }
        })).unwrap_or_default()
    }

    /// Render transport configuration editor
    fn render_transport_config_editor(&mut self, ui: &mut Ui) {
        let transport_types = ["命令行 (stdio)", "TCP", "Unix Socket", "SSE"];
        let mut current_type = match &self.new_server_config.transport {
            TransportConfig::Command { .. } => 0,
            TransportConfig::Tcp { .. } => 1,
            TransportConfig::Unix { .. } => 2,
            TransportConfig::WebSocket { .. } => 3,
        };

        ui.horizontal(|ui| {
            ui.label("类型:");
            ComboBox::from_id_source("transport_type")
                .selected_text(transport_types[current_type])
                .show_ui(ui, |ui| {
                    for (i, &transport_type) in transport_types.iter().enumerate() {
                        ui.selectable_value(&mut current_type, i, transport_type);
                    }
                });
        });

        // Render appropriate editor based on type
        match current_type {
            0 => {
                // Command transport (stdio)
                if let TransportConfig::Command { command, args, env } = &mut self.new_server_config.transport {
                    ui.horizontal(|ui| {
                        ui.label("命令:");
                        ui.add(TextEdit::singleline(command).hint_text("例如: npx").desired_width(200.0));
                    });
                    ui.horizontal(|ui| {
                        ui.label("参数:");
                        let args_text = args.join(" ");
                        let mut args_input = args_text.clone();
                        ui.add(TextEdit::singleline(&mut args_input)
                            .hint_text("例如: -y @modelcontextprotocol/server-filesystem /path/to/directory")
                            .desired_width(400.0));
                        if args_input != args_text {
                            *args = args_input.split_whitespace().map(|s| s.to_string()).collect();
                        }
                    });

                    // 添加重要提示
                    ui.label("💡 重要提示:");
                    ui.label("• filesystem 服务器需要指定允许访问的目录路径");
                    ui.label("• 示例: -y @modelcontextprotocol/server-filesystem /Users/username/Desktop");
                    ui.label("• 可以指定多个目录，用空格分隔");

                    // 快速设置按钮
                    ui.horizontal(|ui| {
                        if ui.button("📁 Filesystem 模板").clicked() {
                            *command = "npx".to_string();
                            *args = vec![
                                "-y".to_string(),
                                "@modelcontextprotocol/server-filesystem".to_string(),
                                ".".to_string(), // 使用当前目录
                            ];
                        }
                        if ui.button("🌐 Git 模板").clicked() {
                            *command = "uvx".to_string();
                            *args = vec!["mcp-server-git".to_string()];
                        }
                        if ui.button("🔍 Everything 模板").clicked() {
                            *command = "npx".to_string();
                            *args = vec![
                                "-y".to_string(),
                                "@modelcontextprotocol/server-everything".to_string(),
                            ];
                        }
                    });

                    // Environment variables editor
                    ui.collapsing("环境变量", |ui| {
                        let mut env_to_remove = Vec::new();
                        let mut env_to_update = Vec::new();
                        let mut env_entries: Vec<_> = env.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
                        env_entries.sort_by_key(|(k, _)| k.clone());

                        for (key, value) in env_entries {
                            ui.horizontal(|ui| {
                                ui.label(&key);
                                ui.label("=");
                                let mut value_clone = value.clone();
                                ui.add(TextEdit::singleline(&mut value_clone).desired_width(150.0));
                                if value_clone != value {
                                    env_to_update.push((key.clone(), value_clone));
                                }
                                if ui.button("🗑").clicked() {
                                    env_to_remove.push(key.clone());
                                }
                            });
                        }

                        // Apply updates
                        for (key, value) in env_to_update {
                            env.insert(key, value);
                        }

                        for key in env_to_remove {
                            env.remove(&key);
                        }

                        // Add new environment variable
                        ui.horizontal(|ui| {
                            ui.label("添加环境变量:");
                            let mut new_key = String::new();
                            let mut new_value = String::new();
                            ui.add(TextEdit::singleline(&mut new_key).hint_text("变量名").desired_width(100.0));
                            ui.label("=");
                            ui.add(TextEdit::singleline(&mut new_value).hint_text("值").desired_width(100.0));
                            if ui.button("➕").clicked() && !new_key.is_empty() {
                                env.insert(new_key, new_value);
                            }
                        });
                    });
                } else {
                    self.new_server_config.transport = TransportConfig::Command {
                        command: "npx".to_string(),
                        args: vec![
                            "-y".to_string(),
                            "@modelcontextprotocol/server-filesystem".to_string(),
                            ".".to_string(), // 使用当前目录
                        ],
                        env: HashMap::new(),
                    };
                }
            }
            1 => {
                // TCP transport
                if let TransportConfig::Tcp { host, port } = &mut self.new_server_config.transport {
                    ui.horizontal(|ui| {
                        ui.label("主机:");
                        ui.add(TextEdit::singleline(host).desired_width(150.0));
                    });
                    ui.horizontal(|ui| {
                        ui.label("端口:");
                        ui.add(egui::DragValue::new(port).range(1..=65535));
                    });
                } else {
                    self.new_server_config.transport = TransportConfig::Tcp {
                        host: "localhost".to_string(),
                        port: 8080,
                    };
                }
            }
            2 => {
                // Unix socket transport
                if let TransportConfig::Unix { socket_path } = &mut self.new_server_config.transport {
                    ui.horizontal(|ui| {
                        ui.label("Socket 路径:");
                        ui.add(TextEdit::singleline(socket_path).desired_width(200.0));
                    });
                } else {
                    self.new_server_config.transport = TransportConfig::Unix {
                        socket_path: String::new(),
                    };
                }
            }
            3 => {
                // SSE transport
                if let TransportConfig::WebSocket { url } = &mut self.new_server_config.transport {
                    ui.horizontal(|ui| {
                        ui.label("SSE URL:");
                        ui.add(TextEdit::singleline(url).desired_width(200.0));
                    });
                    ui.label("💡 提示: SSE URL 通常以 /sse 结尾，例如 http://localhost:3001/sse");
                } else {
                    self.new_server_config.transport = TransportConfig::WebSocket {
                        url: "http://localhost:3001/sse".to_string(),
                    };
                }
            }
            _ => {}
        }
    }



    /// Render import dialog
    fn render_import_dialog(&mut self, ctx: &Context) {
        if !self.ui_state.show_import_dialog {
            return;
        }

        egui::Window::new("导入 MCP 服务器")
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    ui.label("选择要导入的配置文件:");

                    ui.horizontal(|ui| {
                        ui.add(TextEdit::singleline(&mut self.ui_state.import_file_path).desired_width(300.0));
                        if ui.button("浏览").clicked() {
                            // Open file dialog
                            if let Some(path) = rfd::FileDialog::new()
                                .add_filter("JSON", &["json"])
                                .set_title("选择 MCP 配置文件")
                                .pick_file()
                            {
                                self.ui_state.import_file_path = path.to_string_lossy().to_string();
                            }
                        }
                    });

                    ui.separator();

                    ui.horizontal(|ui| {
                        if ui.button("导入").clicked() {
                            // Import servers
                            if !self.ui_state.import_file_path.is_empty() {
                                self.import_server_configs();
                            } else {
                                self.ui_state.error_message = Some("请选择要导入的文件".to_string());
                            }
                        }

                        if ui.button("取消").clicked() {
                            self.ui_state.show_import_dialog = false;
                            self.ui_state.import_file_path.clear();
                        }
                    });
                });
            });
    }

    /// Render export dialog
    fn render_export_dialog(&mut self, ctx: &Context) {
        if !self.ui_state.show_export_dialog {
            return;
        }

        egui::Window::new("导出 MCP 服务器")
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    ui.label("导出服务器配置:");

                    // Directory selection
                    ui.horizontal(|ui| {
                        ui.label("目录:");
                        ComboBox::from_id_source("export_directory")
                            .selected_text(self.ui_state.export_directory.as_deref().unwrap_or("全部"))
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut self.ui_state.export_directory, None, "全部");
                                for directory in self.server_manager.get_server_directories() {
                                    ui.selectable_value(&mut self.ui_state.export_directory, Some(directory.name.clone()), &directory.name);
                                }
                            });
                    });

                    // File path
                    ui.horizontal(|ui| {
                        ui.add(TextEdit::singleline(&mut self.ui_state.export_file_path).desired_width(300.0));
                        if ui.button("浏览").clicked() {
                            // Open save file dialog
                            if let Some(path) = rfd::FileDialog::new()
                                .add_filter("JSON", &["json"])
                                .set_title("保存 MCP 配置文件")
                                .set_file_name("mcp_servers.json")
                                .save_file()
                            {
                                self.ui_state.export_file_path = path.to_string_lossy().to_string();
                            }
                        }
                    });

                    ui.separator();

                    ui.horizontal(|ui| {
                        if ui.button("导出").clicked() {
                            // Export servers
                            if !self.ui_state.export_file_path.is_empty() {
                                self.export_server_configs();
                            } else {
                                self.ui_state.error_message = Some("请选择要导出到的文件".to_string());
                            }
                        }

                        if ui.button("取消").clicked() {
                            self.ui_state.show_export_dialog = false;
                            self.ui_state.export_file_path.clear();
                        }
                    });
                });
            });
    }



    /// Process MCP events
    fn process_events(&mut self) {
        if let Some(receiver) = &mut self.event_receiver {
            while let Ok(event) = receiver.try_recv() {
                match event {
                    McpEvent::ServerConnected(server_id) => {
                        self.ui_state.status_message = Some(format!("服务器 {} 已连接", server_id));
                        self.ui_state.error_message = None;
                    }
                    McpEvent::ServerDisconnected(server_id) => {
                        self.ui_state.status_message = Some(format!("服务器 {} 已断开", server_id));
                    }
                    McpEvent::ServerError(server_id, error) => {
                        self.ui_state.error_message = Some(format!("服务器 {} 错误: {}", server_id, error));
                        self.ui_state.status_message = None;
                    }
                    McpEvent::CapabilitiesUpdated(server_id, capabilities) => {
                        // Store capabilities in UI cache
                        log::info!("UI received capabilities for server {}: {} tools, {} resources, {} prompts",
                                  server_id, capabilities.tools.len(), capabilities.resources.len(), capabilities.prompts.len());
                        self.server_capabilities.insert(server_id, capabilities.clone());
                        self.ui_state.status_message = Some(format!("服务器 {} 能力已更新", server_id));
                        log::info!("UI cached capabilities for server: {} (total cached: {})", server_id, self.server_capabilities.len());

                        // Check if there's a pending tool test request for this server
                        if let Some(server_name) = self.pending_tool_tests.remove(&server_id) {
                            log::info!("🎯 检测到待处理的工具测试请求，自动打开工具测试对话框: {} ({})", server_name, server_id);

                            // Open tool testing dialog
                            let dialog = ToolTestDialog {
                                server_id,
                                server_name: server_name.clone(),
                                capabilities,
                                selected_category: TestCategory::Tools,
                                selected_tool_index: None,
                                selected_resource_index: None,
                                selected_prompt_index: None,
                                parameter_inputs: HashMap::new(),
                                test_result: None,
                                is_testing: false,
                                test_frame_counter: 0,
                                active_tab: TestResultTab::Summary,
                            };
                            self.ui_state.tool_test_dialog = Some(dialog);

                            // Update status message
                            self.ui_state.server_status_messages.insert(
                                server_id,
                                "连接成功，工具测试对话框已打开".to_string()
                            );
                        }
                    }
                    McpEvent::HealthStatusChanged(server_id, health_status) => {
                        log::info!("UI received HealthStatusChanged event for {}: {:?}", server_id, health_status);
                        // Health status is managed by the server manager, no need to store in UI
                        match health_status {
                            crate::mcp::rmcp_client::ServerHealthStatus::Red => {
                                self.ui_state.status_message = Some(format!("服务器 {} 状态：需要测试 🔴", server_id));
                            }
                            crate::mcp::rmcp_client::ServerHealthStatus::Yellow => {
                                self.ui_state.status_message = Some(format!("服务器 {} 状态：已连接，需要功能测试 🟡", server_id));
                            }
                            crate::mcp::rmcp_client::ServerHealthStatus::Green => {
                                self.ui_state.status_message = Some(format!("服务器 {} 状态：测试通过，可以使用 🟢", server_id));
                            }
                        }
                    }
                    McpEvent::TestCompleted(server_id, test_result) => {
                        log::info!("UI received TestCompleted event for {}: success={}", server_id, test_result.success);
                        if test_result.success {
                            self.ui_state.status_message = Some(format!("服务器 {} 功能测试通过 ✅", server_id));
                            self.ui_state.error_message = None;
                        } else {
                            let error_msg = test_result.error_message.unwrap_or("功能测试失败".to_string());
                            self.ui_state.error_message = Some(format!("服务器 {} 测试失败: {}", server_id, error_msg));
                            self.ui_state.status_message = None;
                        }

                        // Store test output for display
                        self.ui_state.server_test_outputs.insert(
                            server_id,
                            (test_result.stdout, test_result.stderr)
                        );
                    }
                    McpEvent::CapabilitiesExtracted(server_id, capabilities, _capabilities_json) => {
                        log::info!("🎯 MCP设置页面收到能力提取成功事件: {} - 工具:{}, 资源:{}, 提示:{}",
                            server_id, capabilities.tools.len(), capabilities.resources.len(), capabilities.prompts.len());

                        // 更新UI缓存中的能力信息
                        self.server_capabilities.insert(server_id, capabilities.clone());

                        // 显示成功消息
                        self.ui_state.status_message = Some(format!("服务器 {} 能力已成功提取并保存到数据库 💾", server_id));
                        self.ui_state.error_message = None;

                        // Check if there's a pending tool test request for this server
                        if let Some(server_name) = self.pending_tool_tests.remove(&server_id) {
                            log::info!("🎯 检测到待处理的工具测试请求 (能力提取完成)，自动打开工具测试对话框: {} ({})", server_name, server_id);

                            // Open tool testing dialog
                            let dialog = ToolTestDialog {
                                server_id,
                                server_name: server_name.clone(),
                                capabilities,
                                selected_category: TestCategory::Tools,
                                selected_tool_index: None,
                                selected_resource_index: None,
                                selected_prompt_index: None,
                                parameter_inputs: HashMap::new(),
                                test_result: None,
                                is_testing: false,
                                test_frame_counter: 0,
                                active_tab: TestResultTab::Summary,
                            };
                            self.ui_state.tool_test_dialog = Some(dialog);

                            // Update status message
                            self.ui_state.server_status_messages.insert(
                                server_id,
                                "能力提取成功，工具测试对话框已打开".to_string()
                            );
                        }
                    }
                }
            }
        }
    }



    /// Find server ID by configuration (helper method)
    fn find_server_id_by_config(&self, config: &McpServerConfig) -> Option<Uuid> {
        // Now we can use the ID directly from the configuration
        Some(config.id)
    }

    /// Toggle server connection
    fn toggle_server_connection(&mut self, server_id: Uuid, currently_enabled: bool) {
        let result = if currently_enabled {
            // Disconnect server
            self.server_capabilities.remove(&server_id);
            if let Ok(handle) = tokio::runtime::Handle::try_current() {
                handle.block_on(self.server_manager.disconnect_server(server_id))
                    .map_err(|e| e.to_string())
            } else {
                match tokio::runtime::Runtime::new() {
                    Ok(rt) => {
                        rt.block_on(self.server_manager.disconnect_server(server_id))
                            .map_err(|e| e.to_string())
                    }
                    Err(e) => Err(format!("无法创建异步运行时: {}", e))
                }
            }
        } else {
            // Connect server and fetch capabilities
            let connect_result = if let Ok(handle) = tokio::runtime::Handle::try_current() {
                handle.block_on(self.server_manager.connect_server(server_id))
                    .map_err(|e| e.to_string())
            } else {
                match tokio::runtime::Runtime::new() {
                    Ok(rt) => {
                        rt.block_on(self.server_manager.connect_server(server_id))
                            .map_err(|e| e.to_string())
                    }
                    Err(e) => Err(format!("无法创建异步运行时: {}", e))
                }
            };

            // Capabilities will be fetched automatically via events after connection

            connect_result
        };

        match result {
            Ok(()) => {
                let message = if currently_enabled {
                    "服务器已断开连接"
                } else {
                    "服务器已连接，已获取能力信息"
                };
                self.ui_state.status_message = Some(message.to_string());
                self.ui_state.error_message = None;
            }
            Err(e) => {
                let action = if currently_enabled { "断开连接" } else { "连接" };
                self.ui_state.error_message = Some(format!("{}失败: {}", action, e));
            }
        }
    }

    /// Test server connection
    fn test_server_connection(&mut self, server_id: Uuid) {
        // Clear previous messages and outputs for this server
        self.ui_state.server_status_messages.remove(&server_id);
        self.ui_state.server_error_messages.remove(&server_id);
        self.ui_state.server_test_outputs.remove(&server_id);

        // Get server name for better logging
        let server_name = self.server_manager.get_server_directories()
            .iter()
            .flat_map(|dir| &dir.servers)
            .find(|server| server.id == server_id)
            .map(|server| server.name.clone())
            .unwrap_or_else(|| format!("Server {}", server_id));

        log::info!("Starting detailed test for MCP server: {}", server_name);

        let start_time = std::time::Instant::now();
        let result = if let Ok(handle) = tokio::runtime::Handle::try_current() {
            handle.block_on(self.server_manager.test_server_detailed(server_id))
                .map_err(|e| e.to_string())
        } else {
            match tokio::runtime::Runtime::new() {
                Ok(rt) => {
                    rt.block_on(self.server_manager.test_server_detailed(server_id))
                        .map_err(|e| e.to_string())
                }
                Err(e) => Err(format!("无法创建异步运行时: {}", e))
            }
        };

        let test_duration = start_time.elapsed();

        match result {
            Ok(test_result) => {
                // Store the detailed output for debugging
                self.ui_state.server_test_outputs.insert(server_id, (test_result.stdout.clone(), test_result.stderr.clone()));

                if test_result.success {
                    let success_msg = format!("测试成功 (耗时: {:.2}秒)", test_duration.as_secs_f64());
                    log::info!("Server '{}' test completed successfully - server is responsive and can be started", server_name);

                    self.ui_state.server_status_messages.insert(server_id, success_msg);
                } else {
                    let failure_msg = if let Some(error_msg) = &test_result.error_message {
                        format!("测试失败: {} (耗时: {:.2}秒)", error_msg, test_duration.as_secs_f64())
                    } else {
                        format!("测试失败 - 进程无法启动或响应 (耗时: {:.2}秒)", test_duration.as_secs_f64())
                    };
                    log::error!("Server '{}' test failed - check the server configuration and ensure the command is available", server_name);

                    self.ui_state.server_error_messages.insert(server_id, failure_msg);
                }
            }
            Err(e) => {
                let error_msg = format!("测试错误: {} (耗时: {:.2}秒)", e, test_duration.as_secs_f64());
                log::error!("Server '{}' test encountered an error - check logs for detailed error information", server_name);

                self.ui_state.server_error_messages.insert(server_id, error_msg);
                // Store empty outputs for consistency
                self.ui_state.server_test_outputs.insert(server_id, (String::new(), e));
            }
        }
    }

    /// Check if auto-refresh should be performed
    fn should_auto_refresh(&mut self) -> bool {
        if self.first_render {
            self.first_render = false;
            true
        } else {
            false
        }
    }

    /// Refresh server list
    fn refresh_server_list(&mut self) {
        // Clear existing server capabilities to avoid stale data
        self.server_capabilities.clear();

        let result = if let Ok(handle) = tokio::runtime::Handle::try_current() {
            handle.block_on(self.server_manager.initialize())
                .map_err(|e| e.to_string())
        } else {
            match tokio::runtime::Runtime::new() {
                Ok(rt) => {
                    rt.block_on(self.server_manager.initialize())
                        .map_err(|e| e.to_string())
                }
                Err(e) => Err(format!("无法创建异步运行时: {}", e))
            }
        };

        match result {
            Ok(()) => {
                self.ui_state.status_message = Some("服务器列表已刷新".to_string());
                self.ui_state.error_message = None;
            }
            Err(e) => {
                self.ui_state.error_message = Some(format!("刷新失败: {}", e));
            }
        }
    }

    /// Import server configurations
    fn import_server_configs(&mut self) {
        let file_path = std::path::PathBuf::from(&self.ui_state.import_file_path);

        let result = if let Ok(handle) = tokio::runtime::Handle::try_current() {
            handle.block_on(self.server_manager.import_server_config(file_path))
                .map_err(|e| e.to_string())
        } else {
            match tokio::runtime::Runtime::new() {
                Ok(rt) => {
                    rt.block_on(self.server_manager.import_server_config(file_path))
                        .map_err(|e| e.to_string())
                }
                Err(e) => Err(format!("无法创建异步运行时: {}", e))
            }
        };

        match result {
            Ok(server_ids) => {
                self.ui_state.status_message = Some(format!("成功导入 {} 个服务器", server_ids.len()));
                self.ui_state.error_message = None;
                self.ui_state.show_import_dialog = false;
                self.ui_state.import_file_path.clear();
            }
            Err(e) => {
                self.ui_state.error_message = Some(format!("导入失败: {}", e));
            }
        }
    }

    /// Export server configurations
    fn export_server_configs(&mut self) {
        let file_path = std::path::PathBuf::from(&self.ui_state.export_file_path);
        let directory = self.ui_state.export_directory.clone();

        let result = if let Ok(handle) = tokio::runtime::Handle::try_current() {
            handle.block_on(self.server_manager.export_server_configs(file_path, directory))
                .map_err(|e| e.to_string())
        } else {
            match tokio::runtime::Runtime::new() {
                Ok(rt) => {
                    rt.block_on(self.server_manager.export_server_configs(file_path, directory))
                        .map_err(|e| e.to_string())
                }
                Err(e) => Err(format!("无法创建异步运行时: {}", e))
            }
        };

        match result {
            Ok(()) => {
                self.ui_state.status_message = Some("服务器配置导出成功".to_string());
                self.ui_state.error_message = None;
                self.ui_state.show_export_dialog = false;
                self.ui_state.export_file_path.clear();
            }
            Err(e) => {
                self.ui_state.error_message = Some(format!("导出失败: {}", e));
            }
        }
    }

    /// Render server capabilities information
    fn render_server_capabilities(&mut self, ui: &mut Ui, capabilities: &ServerCapabilities) {
        ui.indent("capabilities", |ui| {
            ui.separator();
            ui.label(RichText::new("🔧 服务器能力").strong().color(Color32::BLUE));

            // Tools
            if !capabilities.tools.is_empty() {
                ui.horizontal(|ui| {
                    ui.label("🛠️ 工具:");
                    ui.label(format!("{} 个", capabilities.tools.len()));
                });

                ui.indent("tools", |ui| {
                    // 为工具列表添加滚动区域，支持长列表和长描述
                    ScrollArea::vertical()
                        .id_source("server_capabilities_tools")
                        .max_height(150.0)  // 限制最大高度为150像素
                        .show(ui, |ui| {
                            for tool in &capabilities.tools {
                                ui.horizontal(|ui| {
                                    ui.label("  •");
                                    ui.label(&tool.name);
                                    if let Some(desc) = &tool.description {
                                        ui.label(RichText::new(format!("- {}", desc)).color(Color32::GRAY));
                                    }
                                });
                            }
                        });
                });
            }

            // Resources
            if !capabilities.resources.is_empty() {
                ui.horizontal(|ui| {
                    ui.label("📁 资源:");
                    ui.label(format!("{} 个", capabilities.resources.len()));
                });

                ui.indent("resources", |ui| {
                    // 为资源列表添加滚动区域，支持长列表和长描述
                    ScrollArea::vertical()
                        .id_source("server_capabilities_resources")
                        .max_height(150.0)  // 限制最大高度为150像素
                        .show(ui, |ui| {
                            for resource in &capabilities.resources {
                                ui.horizontal(|ui| {
                                    ui.label("  •");
                                    ui.label(&resource.name);
                                    if let Some(desc) = &resource.description {
                                        ui.label(RichText::new(format!("- {}", desc)).color(Color32::GRAY));
                                    }
                                });
                            }
                        });
                });
            }

            // Prompts
            if !capabilities.prompts.is_empty() {
                ui.horizontal(|ui| {
                    ui.label("💬 提示:");
                    ui.label(format!("{} 个", capabilities.prompts.len()));
                });

                ui.indent("prompts", |ui| {
                    // 为提示列表添加滚动区域，支持长列表和长描述
                    ScrollArea::vertical()
                        .id_source("server_capabilities_prompts")
                        .max_height(150.0)  // 限制最大高度为150像素
                        .show(ui, |ui| {
                            for prompt in &capabilities.prompts {
                                ui.horizontal(|ui| {
                                    ui.label("  •");
                                    ui.label(&prompt.name);
                                    if let Some(desc) = &prompt.description {
                                        ui.label(RichText::new(format!("- {}", desc)).color(Color32::GRAY));
                                    }
                                });
                            }
                        });
                });
            }

            ui.separator();
        });
    }

    /// Test server functionality and update health status (NON-BLOCKING VERSION)
    fn test_server_functionality(&mut self, server_id: Uuid) {
        // Clear previous messages and outputs for this server
        self.ui_state.server_status_messages.remove(&server_id);
        self.ui_state.server_error_messages.remove(&server_id);
        self.ui_state.server_test_outputs.remove(&server_id);

        // Get server name for better logging
        let server_name = self.server_manager.get_server_directories()
            .iter()
            .flat_map(|dir| &dir.servers)
            .find(|server| server.id == server_id)
            .map(|server| server.name.clone())
            .unwrap_or_else(|| format!("Server {}", server_id));

        log::info!("🧪 开始功能测试 - 服务器: '{}' ({})", server_name, server_id);

        // Get available tools for the server
        let available_tools = self.server_manager.get_server_capabilities(server_id)
            .map(|caps| caps.tools)
            .unwrap_or_default();

        log::info!("🔧 服务器 '{}' 可用工具数量: {}", server_name, available_tools.len());

        // Create and show functionality test dialog
        let dialog = FunctionalityTestDialog {
            server_id,
            server_name: server_name.clone(),
            test_result: None,
            is_testing: false,
            test_frame_counter: 0,
            active_tab: FunctionalityTestTab::Summary,
            available_tools,
            selected_tool_index: None,
            parameter_inputs: std::collections::HashMap::new(),
            test_phase: FunctionalityTestPhase::Setup,
        };
        self.ui_state.functionality_test_dialog = Some(dialog);

        // Show immediate feedback
        self.ui_state.server_status_messages.insert(
            server_id,
            "功能测试对话框已打开，正在执行测试...".to_string()
        );

        log::info!("📝 功能测试对话框已创建 - 服务器: '{}'", server_name);
    }





    /// Test server tools - combined functionality that prioritizes tool testing
    fn test_server_tools(&mut self, server_id: Uuid, config: &McpServerConfig) {
        // Clear previous messages and outputs for this server
        self.ui_state.server_status_messages.remove(&server_id);
        self.ui_state.server_error_messages.remove(&server_id);
        self.ui_state.server_test_outputs.remove(&server_id);

        // Get server name for better logging
        let server_name = config.name.clone();
        log::info!("Testing tools for server: {} ({})", server_name, server_id);

        // Show immediate status message
        self.ui_state.server_status_messages.insert(
            server_id,
            "正在连接服务器并获取能力信息...".to_string()
        );

        // Record pending tool test request
        self.pending_tool_tests.insert(server_id, server_name.clone());

        // Check if we already have capabilities
        if let Some(capabilities) = self.server_capabilities.get(&server_id).cloned() {
            // Open tool testing dialog immediately
            let dialog = ToolTestDialog {
                server_id,
                server_name: config.name.clone(),
                capabilities,
                selected_category: TestCategory::Tools,
                selected_tool_index: None,
                selected_resource_index: None,
                selected_prompt_index: None,
                parameter_inputs: HashMap::new(),
                test_result: None,
                is_testing: false,
                test_frame_counter: 0,
                active_tab: TestResultTab::Summary,
            };
            self.ui_state.tool_test_dialog = Some(dialog);

            self.ui_state.server_status_messages.insert(
                server_id,
                "工具测试对话框已打开".to_string()
            );
            return;
        }

        // Since we can't clone McpServerManager, we'll use a different approach
        // We'll trigger the connection through the existing event system
        log::info!("🔄 触发后台连接服务器: {} ({})", server_name, server_id);

        // Try to connect using the existing synchronous method but with better error handling
        let connect_result = if let Ok(handle) = tokio::runtime::Handle::try_current() {
            // Use a very short timeout to avoid blocking UI
            match std::thread::spawn(move || {
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(async {
                    // This is a placeholder - we'll rely on the event system to handle the actual connection
                    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                    Ok(())
                })
            }).join() {
                Ok(result) => result.map_err(|e: anyhow::Error| e.to_string()),
                Err(_) => Err("连接线程失败".to_string()),
            }
        } else {
            Err("无法获取异步运行时".to_string())
        };

        match connect_result {
            Ok(_) => {
                // Show status that we're waiting for the connection to complete
                self.ui_state.server_status_messages.insert(
                    server_id,
                    "连接请求已发送，等待服务器响应...".to_string()
                );
            }
            Err(e) => {
                self.ui_state.server_error_messages.insert(
                    server_id,
                    format!("连接请求失败: {}", e)
                );
            }
        }

        // The connection is now running in background
        // Results will be handled through MCP events and capability updates
        // If we already have capabilities, we can open the dialog immediately
        // Otherwise, we wait for the background connection to complete
    }



    /// Render tool testing dialog
    fn render_tool_test_dialog(&mut self, ctx: &Context) {
        let mut should_close = false;
        let mut should_execute_test = false;

        if let Some(dialog) = &mut self.ui_state.tool_test_dialog {
            let server_name = dialog.server_name.clone();

            egui::Window::new(format!("测试工具 - {}", server_name))
                .collapsible(false)
                .resizable(true)
                .default_width(600.0)
                .default_height(500.0)
                .show(ctx, |ui| {
                    ui.vertical(|ui| {
                        // Category selection
                        ui.horizontal(|ui| {
                            ui.label("测试类别:");
                            ui.radio_value(&mut dialog.selected_category, TestCategory::Tools, "🛠️ 工具");
                            ui.radio_value(&mut dialog.selected_category, TestCategory::Resources, "📁 资源");
                            ui.radio_value(&mut dialog.selected_category, TestCategory::Prompts, "💬 提示");
                        });

                        ui.separator();

                        // Simplified tool selection
                        match dialog.selected_category {
                            TestCategory::Tools => {
                                ui.label("选择工具:");
                                if !dialog.capabilities.tools.is_empty() {
                                    ComboBox::from_id_source("tool_selection")
                                        .selected_text(
                                            dialog.selected_tool_index
                                                .and_then(|i| dialog.capabilities.tools.get(i))
                                                .map(|tool| tool.name.as_str())
                                                .unwrap_or("选择工具...")
                                        )
                                        .show_ui(ui, |ui| {
                                            for (index, tool) in dialog.capabilities.tools.iter().enumerate() {
                                                if ui.selectable_label(dialog.selected_tool_index == Some(index), &tool.name).clicked() {
                                                    dialog.selected_tool_index = Some(index);
                                                    // Clear parameter inputs when tool changes
                                                    dialog.parameter_inputs.clear();
                                                }
                                            }
                                        });
                                } else {
                                    ui.colored_label(Color32::GRAY, "该服务器没有可用的工具");
                                }
                            }
                            TestCategory::Resources => {
                                ui.label("选择资源:");
                                if !dialog.capabilities.resources.is_empty() {
                                    ComboBox::from_id_source("resource_selection")
                                        .selected_text(
                                            dialog.selected_resource_index
                                                .and_then(|i| dialog.capabilities.resources.get(i))
                                                .map(|resource| resource.name.as_str())
                                                .unwrap_or("选择资源...")
                                        )
                                        .show_ui(ui, |ui| {
                                            for (index, resource) in dialog.capabilities.resources.iter().enumerate() {
                                                if ui.selectable_label(dialog.selected_resource_index == Some(index), &resource.name).clicked() {
                                                    dialog.selected_resource_index = Some(index);
                                                    // Clear parameter inputs when resource changes
                                                    dialog.parameter_inputs.clear();
                                                }
                                            }
                                        });
                                } else {
                                    ui.colored_label(Color32::GRAY, "该服务器没有可用的资源");
                                }
                            }
                            TestCategory::Prompts => {
                                ui.label("选择提示:");
                                if !dialog.capabilities.prompts.is_empty() {
                                    ComboBox::from_id_source("prompt_selection")
                                        .selected_text(
                                            dialog.selected_prompt_index
                                                .and_then(|i| dialog.capabilities.prompts.get(i))
                                                .map(|prompt| prompt.name.as_str())
                                                .unwrap_or("选择提示...")
                                        )
                                        .show_ui(ui, |ui| {
                                            for (index, prompt) in dialog.capabilities.prompts.iter().enumerate() {
                                                if ui.selectable_label(dialog.selected_prompt_index == Some(index), &prompt.name).clicked() {
                                                    dialog.selected_prompt_index = Some(index);
                                                    // Clear parameter inputs when prompt changes
                                                    dialog.parameter_inputs.clear();
                                                }
                                            }
                                        });
                                } else {
                                    ui.colored_label(Color32::GRAY, "该服务器没有可用的提示");
                                }
                            }
                        }

                        // Parameter input section
                        match dialog.selected_category {
                            TestCategory::Tools => {
                                if let Some(selected_tool_index) = dialog.selected_tool_index {
                                    if let Some(tool) = dialog.capabilities.tools.get(selected_tool_index) {
                                        ui.separator();
                                        ui.label(RichText::new("参数设置:").strong());

                                        // 为参数输入区域添加滚动区域，防止参数过多时挤出按钮
                                        ScrollArea::vertical()
                                            .id_source(format!("tool_test_parameters_{}", selected_tool_index))
                                            .max_height(200.0)  // 限制最大高度为200像素
                                            .show(ui, |ui| {
                                                // Parse inputSchema to generate parameter inputs
                                                if let Some(schema_value) = &tool.input_schema {
                                                    if let Some(schema) = schema_value.as_object() {
                                                        if let Some(properties) = schema.get("properties").and_then(|p| p.as_object()) {
                                                        let required_fields = schema.get("required")
                                                            .and_then(|r| r.as_array())
                                                            .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect::<std::collections::HashSet<_>>())
                                                            .unwrap_or_default();

                                                        for (param_name, param_schema) in properties {
                                                    // Inline parameter input rendering to avoid borrowing issues
                                                    ui.horizontal(|ui| {
                                                        let is_required = required_fields.contains(param_name.as_str());
                                                        let label_text = if is_required {
                                                            format!("{}*:", param_name)
                                                        } else {
                                                            format!("{}:", param_name)
                                                        };

                                                        ui.label(label_text);

                                                        // Get current value or default
                                                        let current_value = dialog.parameter_inputs.get(param_name).cloned().unwrap_or_default();
                                                        let mut input_value = current_value.clone();

                                                        // Generate input field based on parameter type
                                                        let param_type = param_schema.get("type").and_then(|t| t.as_str()).unwrap_or("string");
                                                        let description = param_schema.get("description").and_then(|d| d.as_str()).unwrap_or("");

                                                        match param_type {
                                                            "boolean" => {
                                                                let mut bool_value = input_value == "true";
                                                                if ui.checkbox(&mut bool_value, "").changed() {
                                                                    input_value = bool_value.to_string();
                                                                }
                                                            }
                                                            "integer" | "number" => {
                                                                ui.add(TextEdit::singleline(&mut input_value)
                                                                    .hint_text(description)
                                                                    .desired_width(150.0));
                                                            }
                                                            "array" => {
                                                                ui.add(TextEdit::singleline(&mut input_value)
                                                                    .hint_text("JSON数组格式，如: [\"item1\", \"item2\"]")
                                                                    .desired_width(200.0));
                                                            }
                                                            "object" => {
                                                                ui.add(TextEdit::singleline(&mut input_value)
                                                                    .hint_text("JSON对象格式，如: {\"key\": \"value\"}")
                                                                    .desired_width(200.0));
                                                            }
                                                            _ => { // string or default
                                                                ui.add(TextEdit::singleline(&mut input_value)
                                                                    .hint_text(description)
                                                                    .desired_width(150.0));
                                                            }
                                                        }

                                                        // Update parameter inputs if changed
                                                        if input_value != current_value {
                                                            dialog.parameter_inputs.insert(param_name.clone(), input_value);
                                                        }

                                                        // Show required indicator
                                                        if is_required {
                                                            ui.colored_label(Color32::RED, "*");
                                                        }
                                                    });

                                                    // Show parameter description if available
                                                    let description = param_schema.get("description").and_then(|d| d.as_str()).unwrap_or("");
                                                    if !description.is_empty() {
                                                        ui.indent("param_desc", |ui| {
                                                            ui.colored_label(Color32::GRAY, format!("💡 {}", description));
                                                        });
                                                    }
                                                }
                                                        } else {
                                                            ui.colored_label(Color32::GRAY, "该工具无需参数");
                                                        }
                                                    } else {
                                                        ui.colored_label(Color32::GRAY, "该工具无需参数");
                                                    }
                                                } else {
                                                    ui.colored_label(Color32::GRAY, "该工具无需参数");
                                                }
                                            });
                                    }
                                }
                            }
                            TestCategory::Prompts => {
                                if let Some(selected_prompt_index) = dialog.selected_prompt_index {
                                    if let Some(prompt) = dialog.capabilities.prompts.get(selected_prompt_index) {
                                        let arguments = &prompt.arguments;
                                        if !arguments.is_empty() {
                                                ui.separator();
                                                ui.label(RichText::new("参数设置:").strong());

                                                // 为提示参数输入区域添加滚动区域，防止参数过多时挤出按钮
                                                ScrollArea::vertical()
                                                    .id_source(format!("prompt_test_parameters_{}", selected_prompt_index))
                                                    .max_height(200.0)  // 限制最大高度为200像素
                                                    .show(ui, |ui| {
                                                        for arg in arguments {
                                                    ui.horizontal(|ui| {
                                                        let is_required = arg.required;
                                                        let label_text = if is_required {
                                                            format!("{}*:", arg.name)
                                                        } else {
                                                            format!("{}:", arg.name)
                                                        };

                                                        ui.label(label_text);

                                                        // Get current value or default
                                                        let current_value = dialog.parameter_inputs.get(&arg.name).cloned().unwrap_or_default();
                                                        let mut input_value = current_value.clone();

                                                        let hint_text = arg.description.as_deref().unwrap_or("");
                                                        ui.add(TextEdit::singleline(&mut input_value)
                                                            .hint_text(hint_text)
                                                            .desired_width(200.0));

                                                        // Update parameter inputs if changed
                                                        if input_value != current_value {
                                                            dialog.parameter_inputs.insert(arg.name.clone(), input_value);
                                                        }

                                                        // Show required indicator
                                                        if is_required {
                                                            ui.colored_label(Color32::RED, "*");
                                                        }
                                                    });

                                                    // Show parameter description if available
                                                    if let Some(desc) = &arg.description {
                                                        if !desc.is_empty() {
                                                            ui.indent("param_desc", |ui| {
                                                                ui.colored_label(Color32::GRAY, format!("💡 {}", desc));
                                                            });
                                                        }
                                                    }
                                                        }
                                                    });
                                        } else {
                                            ui.separator();
                                            ui.colored_label(Color32::GRAY, "该提示无需参数");
                                        }
                                    }
                                }
                            }
                            TestCategory::Resources => {
                                // Resources typically don't need parameters, but we can add support if needed
                                if dialog.selected_resource_index.is_some() {
                                    ui.separator();
                                    ui.colored_label(Color32::GRAY, "资源访问通常无需额外参数");
                                }
                            }
                        }

                        ui.separator();

                        // Simple test button
                        ui.horizontal(|ui| {
                            let can_test = match dialog.selected_category {
                                TestCategory::Tools => dialog.selected_tool_index.is_some(),
                                TestCategory::Resources => dialog.selected_resource_index.is_some(),
                                TestCategory::Prompts => dialog.selected_prompt_index.is_some(),
                            };

                            if ui.add_enabled(can_test && !dialog.is_testing, egui::Button::new("🧪 执行测试")).clicked() {
                                should_execute_test = true;
                            }

                            if dialog.is_testing {
                                ui.spinner();
                                let progress_text = if dialog.test_frame_counter <= 10 {
                                    format!("准备测试中... ({}/10)", dialog.test_frame_counter)
                                } else {
                                    "执行测试中...".to_string()
                                };
                                ui.label(progress_text);
                            }

                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if ui.button("关闭").clicked() {
                                    should_close = true;
                                }
                            });
                        });

                        // Tab-based test result display
                        if let Some(result) = &dialog.test_result {
                            ui.separator();
                            ui.label(RichText::new("测试结果:").strong());

                            // Status indicator
                            let status_color = if result.success { Color32::GREEN } else { Color32::RED };
                            let status_text = if result.success { "✅ 成功" } else { "❌ 失败" };
                            ui.colored_label(status_color, status_text);

                            // Tab buttons
                            ui.horizontal(|ui| {
                                if ui.selectable_label(dialog.active_tab == TestResultTab::Summary, "概要").clicked() {
                                    dialog.active_tab = TestResultTab::Summary;
                                }
                                if ui.selectable_label(dialog.active_tab == TestResultTab::Request, "Request").clicked() {
                                    dialog.active_tab = TestResultTab::Request;
                                }
                                if ui.selectable_label(dialog.active_tab == TestResultTab::Response, "Response").clicked() {
                                    dialog.active_tab = TestResultTab::Response;
                                }
                            });

                            ui.separator();

                            // Tab content with scrollable area
                            ScrollArea::vertical()
                                .max_height(300.0)
                                .show(ui, |ui| {
                                    match dialog.active_tab {
                                        TestResultTab::Summary => {
                                            ui.label(RichText::new("概要信息:").strong());
                                            ui.label(&result.summary);

                                            if let Some(error) = &result.error {
                                                ui.colored_label(Color32::RED, format!("错误: {}", error));
                                            }

                                            ui.label(format!("执行时间: {:?}", result.duration));

                                            if let Some(debug_info) = &result.debug_info {
                                                ui.separator();
                                                ui.label(RichText::new("调试信息:").strong());
                                                ui.add(TextEdit::multiline(&mut debug_info.as_str())
                                                    .desired_rows(8)
                                                    .desired_width(f32::INFINITY)
                                                    .code_editor());
                                            }
                                        }
                                        TestResultTab::Request => {
                                            ui.label(RichText::new("MCP 请求:").strong());
                                            if let Some(request) = &result.request {
                                                ui.add(TextEdit::multiline(&mut request.as_str())
                                                    .desired_rows(15)
                                                    .desired_width(f32::INFINITY)
                                                    .code_editor());
                                            } else {
                                                ui.label("无请求数据");
                                            }
                                        }
                                        TestResultTab::Response => {
                                            ui.label(RichText::new("MCP 响应:").strong());
                                            if let Some(response) = &result.response {
                                                ui.add(TextEdit::multiline(&mut response.as_str())
                                                    .desired_rows(15)
                                                    .desired_width(f32::INFINITY)
                                                    .code_editor());
                                            } else {
                                                ui.label("无响应数据");
                                            }
                                        }
                                    }
                                });
                        }
                    });
                });
        }

        // Execute test outside of the UI closure to avoid borrowing issues
        if should_execute_test {
            // Clone the dialog data we need for the test first
            let test_data = if let Some(dialog) = &self.ui_state.tool_test_dialog {
                Some((
                    dialog.server_id,
                    dialog.selected_category,
                    dialog.selected_tool_index,
                    dialog.selected_resource_index,
                    dialog.selected_prompt_index,
                    dialog.capabilities.clone(),
                    dialog.parameter_inputs.clone(),
                ))
            } else {
                None
            };

            if let Some((server_id, selected_category, selected_tool_index, selected_resource_index, selected_prompt_index, capabilities, parameter_inputs)) = test_data {
                // Set testing state and start frame-based test execution
                if let Some(dialog) = &mut self.ui_state.tool_test_dialog {
                    dialog.is_testing = true;
                    dialog.test_result = None;
                    dialog.test_frame_counter = 0;
                }
            }
        }

        // Execute test in chunks to avoid UI blocking
        let test_execution_data = if let Some(dialog) = &mut self.ui_state.tool_test_dialog {
            if dialog.is_testing && dialog.test_frame_counter < 15 {
                dialog.test_frame_counter += 1;

                // Show progress indicator during the delay
                if dialog.test_frame_counter <= 10 {
                    // Still preparing, don't execute yet - give more time for UI to update
                    None
                } else {
                    // Execute test after enough frames to allow UI to update
                    Some((
                        dialog.server_id,
                        dialog.selected_category,
                        dialog.selected_tool_index,
                        dialog.selected_resource_index,
                        dialog.selected_prompt_index,
                        dialog.capabilities.clone(),
                        dialog.parameter_inputs.clone(),
                    ))
                }
            } else {
                None
            }
        } else {
            None
        };

        // Execute test outside of the dialog borrow
        if let Some(test_data) = test_execution_data {
            // Execute the test with cloned data
            let result = self.execute_tool_test_with_data(
                test_data.0,
                test_data.1,
                test_data.2,
                test_data.3,
                test_data.4,
                &test_data.5,
                &test_data.6
            );

            // Update the dialog with the result
            if let Some(dialog) = &mut self.ui_state.tool_test_dialog {
                dialog.test_result = Some(result);
                dialog.is_testing = false;
                dialog.test_frame_counter = 0;
            }
        }

        if should_close {
            self.ui_state.tool_test_dialog = None;
        }
    }











    /// Execute tool test with separate data to avoid borrowing issues
    fn execute_tool_test_with_data(
        &mut self,
        server_id: Uuid,
        selected_category: TestCategory,
        selected_tool_index: Option<usize>,
        selected_resource_index: Option<usize>,
        selected_prompt_index: Option<usize>,
        capabilities: &ServerCapabilities,
        parameter_inputs: &HashMap<String, String>
    ) -> ToolTestResult {
        let start_time = std::time::Instant::now();

        // Execute real MCP test using actual server manager
        match selected_category {
            TestCategory::Tools => {
                if let Some(index) = selected_tool_index {
                    if let Some(tool) = capabilities.tools.get(index) {
                        self.execute_real_tool_test(server_id, &tool.name, parameter_inputs)
                    } else {
                        ToolTestResult {
                            success: false,
                            error: Some("工具不存在".to_string()),
                            duration: start_time.elapsed(),
                            summary: "工具不存在".to_string(),
                            request: None,
                            response: None,
                            debug_info: None,
                        }
                    }
                } else {
                    ToolTestResult {
                        success: false,
                        error: Some("未选择工具".to_string()),
                        duration: start_time.elapsed(),
                        summary: "未选择工具".to_string(),
                        request: None,
                        response: None,
                        debug_info: None,
                    }
                }
            }
            TestCategory::Resources => {
                if let Some(index) = selected_resource_index {
                    if let Some(resource) = capabilities.resources.get(index) {
                        self.execute_real_resource_test(server_id, &resource.uri, parameter_inputs)
                    } else {
                        ToolTestResult {
                            success: false,
                            error: Some("资源不存在".to_string()),
                            duration: start_time.elapsed(),
                            summary: "资源不存在".to_string(),
                            request: None,
                            response: None,
                            debug_info: None,
                        }
                    }
                } else {
                    ToolTestResult {
                        success: false,
                        error: Some("未选择资源".to_string()),
                        duration: start_time.elapsed(),
                        summary: "未选择资源".to_string(),
                        request: None,
                        response: None,
                        debug_info: None,
                    }
                }
            }
            TestCategory::Prompts => {
                if let Some(index) = selected_prompt_index {
                    if let Some(prompt) = capabilities.prompts.get(index) {
                        self.execute_real_prompt_test(server_id, &prompt.name, parameter_inputs)
                    } else {
                        ToolTestResult {
                            success: false,
                            error: Some("提示不存在".to_string()),
                            duration: start_time.elapsed(),
                            summary: "提示不存在".to_string(),
                            request: None,
                            response: None,
                            debug_info: None,
                        }
                    }
                } else {
                    ToolTestResult {
                        success: false,
                        error: Some("未选择提示".to_string()),
                        duration: start_time.elapsed(),
                        summary: "未选择提示".to_string(),
                        request: None,
                        response: None,
                        debug_info: None,
                    }
                }
            }
        }
    }

    /// Render parameter input field based on schema
    fn render_parameter_input(
        &self,
        ui: &mut egui::Ui,
        parameter_inputs: &mut HashMap<String, String>,
        param_name: &str,
        param_schema: &serde_json::Value,
        required_fields: &std::collections::HashSet<&str>
    ) {
        ui.horizontal(|ui| {
            let is_required = required_fields.contains(param_name);
            let label_text = if is_required {
                format!("{}*:", param_name)
            } else {
                format!("{}:", param_name)
            };

            ui.label(label_text);

            // Get current value or default
            let current_value = parameter_inputs.get(param_name).cloned().unwrap_or_default();
            let mut input_value = current_value.clone();

            // Generate input field based on parameter type
            let param_type = param_schema.get("type").and_then(|t| t.as_str()).unwrap_or("string");
            let description = param_schema.get("description").and_then(|d| d.as_str()).unwrap_or("");

            match param_type {
                "boolean" => {
                    let mut bool_value = input_value == "true";
                    if ui.checkbox(&mut bool_value, "").changed() {
                        input_value = bool_value.to_string();
                    }
                }
                "integer" | "number" => {
                    ui.add(TextEdit::singleline(&mut input_value)
                        .hint_text(description)
                        .desired_width(150.0));
                }
                "array" => {
                    ui.add(TextEdit::singleline(&mut input_value)
                        .hint_text("JSON数组格式，如: [\"item1\", \"item2\"]")
                        .desired_width(200.0));
                }
                "object" => {
                    ui.add(TextEdit::singleline(&mut input_value)
                        .hint_text("JSON对象格式，如: {\"key\": \"value\"}")
                        .desired_width(200.0));
                }
                _ => { // string or default
                    ui.add(TextEdit::singleline(&mut input_value)
                        .hint_text(description)
                        .desired_width(150.0));
                }
            }

            // Update parameter inputs if changed
            if input_value != current_value {
                parameter_inputs.insert(param_name.to_string(), input_value);
            }

            // Show required indicator
            if is_required {
                ui.colored_label(Color32::RED, "*");
            }
        });

        // Show parameter description if available
        let description = param_schema.get("description").and_then(|d| d.as_str()).unwrap_or("");
        if !description.is_empty() {
            ui.indent("param_desc", |ui| {
                ui.colored_label(Color32::GRAY, format!("💡 {}", description));
            });
        }
    }





    /// Execute real tool test using simplified MCP protocol (following git_stdio.rs pattern)
    fn execute_real_tool_test(&mut self, server_id: Uuid, tool_name: &str, parameters: &HashMap<String, String>) -> ToolTestResult {
        let start_time = std::time::Instant::now();

        // Convert parameters to JSON object format expected by MCP
        let arguments = if parameters.is_empty() {
            serde_json::Value::Object(serde_json::Map::new())
        } else {
            // Convert HashMap<String, String> to JSON object with proper value types
            let mut json_map = serde_json::Map::new();
            for (key, value) in parameters {
                // Smart type conversion based on value content
                let json_value = if value.trim().is_empty() {
                    serde_json::Value::String(value.clone())
                } else if value == "true" || value == "false" {
                    // Boolean values
                    serde_json::Value::Bool(value == "true")
                } else if let Ok(int_val) = value.parse::<i64>() {
                    // Integer values
                    serde_json::Value::Number(serde_json::Number::from(int_val))
                } else if let Ok(float_val) = value.parse::<f64>() {
                    // Float values
                    if let Some(num) = serde_json::Number::from_f64(float_val) {
                        serde_json::Value::Number(num)
                    } else {
                        serde_json::Value::String(value.clone())
                    }
                } else if value.starts_with('{') || value.starts_with('[') {
                    // Try to parse as JSON object or array
                    serde_json::from_str::<serde_json::Value>(value)
                        .unwrap_or_else(|_| serde_json::Value::String(value.clone()))
                } else {
                    // Default to string
                    serde_json::Value::String(value.clone())
                };
                json_map.insert(key.clone(), json_value);
            }
            serde_json::Value::Object(json_map)
        };

        // Create MCP request for logging
        let mcp_request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/call",
            "params": {
                "name": tool_name,
                "arguments": arguments
            }
        });

        let request_str = serde_json::to_string_pretty(&mcp_request).unwrap_or_default();

        // Log the MCP request
        log::info!("Executing tool test: {} on server {} with arguments: {:?}", tool_name, server_id, arguments);
        log::debug!("MCP Request: {}", request_str);

        // Use simplified testing method (creates fresh rmcp service like git_stdio.rs)
        let call_result = if let Ok(handle) = tokio::runtime::Handle::try_current() {
            handle.block_on(self.server_manager.call_tool_for_testing(server_id, tool_name, arguments.clone()))
        } else {
            match tokio::runtime::Runtime::new() {
                Ok(rt) => {
                    rt.block_on(self.server_manager.call_tool_for_testing(server_id, tool_name, arguments.clone()))
                }
                Err(e) => Err(anyhow::anyhow!("无法创建异步运行时: {}", e))
            }
        };

        // Process result (simplified, no complex retry logic)
        let (success, output, error, stdout_content) = match call_result {
            Ok(real_response) => {
                // Real MCP server response
                let response = serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": 1,
                    "result": real_response
                });
                let stdout_msg = format!("Tool '{}' executed successfully on server {}", tool_name, server_id);
                log::info!("✅ {}", stdout_msg);
                (
                    true,
                    serde_json::to_string_pretty(&response).unwrap_or_default(),
                    None,
                    stdout_msg
                )
            },
            Err(e) => {
                // Tool call failed
                let error_msg = format!("Tool '{}' failed on server {}: {}", tool_name, server_id, e);
                log::error!("❌ {}", error_msg);

                let response = serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": 1,
                    "error": {
                        "code": -1,
                        "message": error_msg.clone(),
                        "data": {
                            "tool": tool_name,
                            "server_id": server_id.to_string(),
                            "arguments": arguments
                        }
                    }
                });

                (
                    false,
                    serde_json::to_string_pretty(&response).unwrap_or_default(),
                    Some(error_msg.clone()),
                    error_msg
                )
            }
        };

        let summary = if success {
            format!("✅ 工具 '{}' 执行成功 (耗时: {:?})", tool_name, start_time.elapsed())
        } else {
            format!("❌ 工具 '{}' 执行失败 (耗时: {:?})", tool_name, start_time.elapsed())
        };

        ToolTestResult {
            success,
            error,
            duration: start_time.elapsed(),
            summary,
            request: Some(request_str),
            response: Some(output),
            debug_info: Some(stdout_content),
        }
    }

    /// Execute real resource test using MCP protocol
    fn execute_real_resource_test(&mut self, server_id: Uuid, uri: &str, _parameters: &HashMap<String, String>) -> ToolTestResult {
        let start_time = std::time::Instant::now();

        // Create MCP request
        let mcp_request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "resources/read",
            "params": {
                "uri": uri
            }
        });

        let request_str = serde_json::to_string_pretty(&mcp_request).unwrap_or_default();

        log::info!("Executing real resource test: {} on server {}", uri, server_id);
        log::debug!("MCP Request STDIN: {}", request_str);

        // Try to call the real MCP server
        let call_result = if let Ok(handle) = tokio::runtime::Handle::try_current() {
            handle.block_on(self.server_manager.read_resource(server_id, uri))
        } else {
            match tokio::runtime::Runtime::new() {
                Ok(rt) => {
                    rt.block_on(self.server_manager.read_resource(server_id, uri))
                }
                Err(e) => Err(anyhow::anyhow!("无法创建异步运行时: {}", e))
            }
        };

        let (success, output, error, stdout_content) = match call_result {
            Ok(real_response) => {
                // Real MCP server response
                let response = serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": 2,
                    "result": real_response
                });
                let stdout_msg = format!("Resource '{}' accessed successfully on server {}", uri, server_id);
                log::info!("{}", stdout_msg);
                (
                    true,
                    serde_json::to_string_pretty(&response).unwrap_or_default(),
                    None,
                    stdout_msg
                )
            },
            Err(e) => {
                // Resource call failed
                let error_msg = format!("Resource '{}' failed on server {}: {}", uri, server_id, e);
                log::error!("❌ {}", error_msg);

                let response = serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": 2,
                    "error": {
                        "code": -1,
                        "message": error_msg.clone(),
                        "data": {
                            "uri": uri,
                            "server_id": server_id.to_string()
                        }
                    }
                });

                (
                    false,
                    serde_json::to_string_pretty(&response).unwrap_or_default(),
                    Some(error_msg.clone()),
                    error_msg
                )
            }
        };

        // Log the response
        log::debug!("MCP Response STDOUT: {}", stdout_content);

        let summary = if success {
            format!("✅ 资源 '{}' 访问成功 (耗时: {:?})", uri, start_time.elapsed())
        } else {
            format!("❌ 资源 '{}' 访问失败 (耗时: {:?})", uri, start_time.elapsed())
        };

        ToolTestResult {
            success,
            error,
            duration: start_time.elapsed(),
            summary,
            request: Some(request_str),
            response: Some(output),
            debug_info: Some(stdout_content),
        }
    }

    /// Execute real prompt test using MCP protocol
    fn execute_real_prompt_test(&mut self, server_id: Uuid, prompt_name: &str, parameters: &HashMap<String, String>) -> ToolTestResult {
        let start_time = std::time::Instant::now();

        // Create MCP request with proper argument formatting
        let arguments = if parameters.is_empty() {
            None
        } else {
            // Convert HashMap<String, String> to JSON object with proper value types
            let mut json_map = serde_json::Map::new();
            for (key, value) in parameters {
                // Try to parse the value as JSON first, fallback to string
                let json_value = serde_json::from_str::<serde_json::Value>(value)
                    .unwrap_or_else(|_| serde_json::Value::String(value.clone()));
                json_map.insert(key.clone(), json_value);
            }
            Some(serde_json::Value::Object(json_map))
        };

        let mcp_request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 3,
            "method": "prompts/get",
            "params": {
                "name": prompt_name,
                "arguments": arguments
            }
        });

        let request_str = serde_json::to_string_pretty(&mcp_request).unwrap_or_default();

        log::info!("Executing real prompt test: {} on server {} with arguments: {:?}", prompt_name, server_id, arguments);
        log::debug!("MCP Request STDIN: {}", request_str);

        // Try to call the real MCP server
        let call_result = if let Ok(handle) = tokio::runtime::Handle::try_current() {
            handle.block_on(self.server_manager.get_prompt(server_id, prompt_name, arguments.clone()))
        } else {
            match tokio::runtime::Runtime::new() {
                Ok(rt) => {
                    rt.block_on(self.server_manager.get_prompt(server_id, prompt_name, arguments.clone()))
                }
                Err(e) => Err(anyhow::anyhow!("无法创建异步运行时: {}", e))
            }
        };

        let params_str = if parameters.is_empty() {
            "{}".to_string()
        } else {
            serde_json::to_string(parameters).unwrap_or_default()
        };

        let (success, output, error, stdout_content) = match call_result {
            Ok(real_response) => {
                // Real MCP server response
                let response = serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": 3,
                    "result": real_response
                });
                let stdout_msg = format!("Prompt '{}' executed successfully on server {}", prompt_name, server_id);
                log::info!("{}", stdout_msg);
                (
                    true,
                    serde_json::to_string_pretty(&response).unwrap_or_default(),
                    None,
                    stdout_msg
                )
            },
            Err(e) => {
                // Prompt call failed
                let error_msg = format!("Prompt '{}' failed on server {}: {}", prompt_name, server_id, e);
                log::error!("❌ {}", error_msg);

                let response = serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": 3,
                    "error": {
                        "code": -1,
                        "message": error_msg.clone(),
                        "data": {
                            "prompt": prompt_name,
                            "server_id": server_id.to_string(),
                            "arguments": arguments
                        }
                    }
                });

                (
                    false,
                    serde_json::to_string_pretty(&response).unwrap_or_default(),
                    Some(error_msg.clone()),
                    error_msg
                )
            }
        };

        // Log the response
        log::debug!("MCP Response STDOUT: {}", stdout_content);

        let summary = if success {
            format!("✅ 提示 '{}' 执行成功 (耗时: {:?})", prompt_name, start_time.elapsed())
        } else {
            format!("❌ 提示 '{}' 执行失败 (耗时: {:?})", prompt_name, start_time.elapsed())
        };

        ToolTestResult {
            success,
            error,
            duration: start_time.elapsed(),
            summary,
            request: Some(request_str),
            response: Some(output),
            debug_info: Some(stdout_content),
        }
    }

    /// Render functionality testing dialog
    fn render_functionality_test_dialog(&mut self, ctx: &Context) {
        let mut should_close = false;
        let mut should_execute_test = false;
        let mut should_complete_test = false;
        let mut should_update_parameters = false;
        let mut should_update_status_to_green = false;

        if let Some(dialog) = &mut self.ui_state.functionality_test_dialog {
            let server_name = dialog.server_name.clone();
            let server_id = dialog.server_id;

            // Handle test progress
            if dialog.is_testing {
                dialog.test_frame_counter += 1;

                // Auto-complete test after 120 frames (about 2 seconds at 60fps)
                if dialog.test_frame_counter >= 120 {
                    should_complete_test = true;
                }
            }

            egui::Window::new(format!("🧪 服务器功能测试 - {}", server_name))
                .collapsible(false)
                .resizable(true)
                .default_width(700.0)
                .default_height(500.0)
                .show(ctx, |ui| {
                    ui.vertical(|ui| {
                        // Header
                        ui.label(RichText::new("选择工具并配置参数来测试服务器功能").strong());
                        ui.separator();

                        // Phase-based content
                        let test_phase = dialog.test_phase;
                        match test_phase {
                            FunctionalityTestPhase::Setup => {
                                Self::render_test_setup_phase_static(ui, dialog, &mut should_execute_test, &mut should_update_parameters);
                            }
                            FunctionalityTestPhase::Testing => {
                                Self::render_test_execution_phase_static(ui, dialog);
                            }
                            FunctionalityTestPhase::Results => {
                                if Self::render_test_results_phase_static(ui, dialog) {
                                    // 用户点击了更新服务器状态为绿灯按钮
                                    should_update_status_to_green = true;
                                }
                            }
                        }
                        // Bottom buttons
                        ui.separator();
                        ui.horizontal(|ui| {
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if ui.button("关闭").clicked() {
                                    should_close = true;
                                }
                            });
                        });
                    });
                });
        }

        // Handle actions outside of the UI closure to avoid borrowing issues
        if should_execute_test {
            if let Some(dialog) = &mut self.ui_state.functionality_test_dialog {
                dialog.is_testing = true;
                dialog.test_frame_counter = 0;
                dialog.test_phase = FunctionalityTestPhase::Testing;
                log::info!("🧪 开始功能测试 - 服务器: '{}'", dialog.server_name);
            }
        }

        if should_update_parameters {
            if let Some(dialog) = &mut self.ui_state.functionality_test_dialog {
                Self::update_tool_parameters_static(dialog);
            }
        }

        // Auto-complete test when counter reaches threshold
        if should_complete_test {
            if let Some(dialog) = &mut self.ui_state.functionality_test_dialog {
                let server_id = dialog.server_id;
                self.execute_functionality_test(server_id);
            }
        }

        if should_close {
            self.ui_state.functionality_test_dialog = None;
        }

        // Handle status update to green
        if should_update_status_to_green {
            if let Some(dialog) = &self.ui_state.functionality_test_dialog {
                let server_id = dialog.server_id;
                log::info!("🟢 用户请求将服务器状态更新为绿灯: {}", server_id);

                // 执行状态更新
                let result = if let Ok(handle) = tokio::runtime::Handle::try_current() {
                    handle.block_on(async {
                        self.server_manager.update_server_status_to_green(server_id).await
                    })
                } else {
                    match tokio::runtime::Runtime::new() {
                        Ok(rt) => {
                            rt.block_on(async {
                                self.server_manager.update_server_status_to_green(server_id).await
                            })
                        }
                        Err(e) => {
                            Err(anyhow::anyhow!("无法创建异步运行时: {}", e))
                        }
                    }
                };

                match result {
                    Ok(()) => {
                        log::info!("✅ 服务器状态已成功更新为绿灯: {}", server_id);
                        self.ui_state.status_message = Some("✅ 服务器状态已更新为绿灯".to_string());
                    }
                    Err(e) => {
                        log::error!("❌ 更新服务器状态失败: {}", e);
                        self.ui_state.error_message = Some(format!("更新服务器状态失败: {}", e));
                    }
                }
            }
        }
    }

    /// Execute functionality test for a server
    fn execute_functionality_test(&mut self, server_id: Uuid) {
        // Get server name and selected tool info
        let server_name = self.server_manager.get_server_directories()
            .iter()
            .flat_map(|dir| &dir.servers)
            .find(|server| server.id == server_id)
            .map(|server| server.name.clone())
            .unwrap_or_else(|| format!("Server {}", server_id));

        // Get test configuration from dialog
        let (tool_name, parameters) = if let Some(dialog) = &self.ui_state.functionality_test_dialog {
            if let Some(tool_index) = dialog.selected_tool_index {
                if let Some(tool) = dialog.available_tools.get(tool_index) {
                    (tool.name.clone(), dialog.parameter_inputs.clone())
                } else {
                    log::error!("❌ 无效的工具索引: {}", tool_index);
                    (String::new(), std::collections::HashMap::new())
                }
            } else {
                log::error!("❌ 未选择测试工具");
                (String::new(), std::collections::HashMap::new())
            }
        } else {
            log::error!("❌ 功能测试对话框不存在");
            (String::new(), std::collections::HashMap::new())
        };

        if tool_name.is_empty() {
            let test_result = TestResult {
                success: false,
                error_message: Some("未选择测试工具".to_string()),
                stdout: String::new(),
                stderr: "请先选择要测试的工具".to_string(),
            };

            if let Some(dialog) = &mut self.ui_state.functionality_test_dialog {
                dialog.test_result = Some(test_result);
                dialog.is_testing = false;
                dialog.test_frame_counter = 0;
                dialog.test_phase = FunctionalityTestPhase::Results;
            }
            return;
        }

        log::info!("🧪 执行真实功能测试 - 服务器: '{}', 工具: '{}', 参数: {:?}", server_name, tool_name, parameters);

        // Execute the actual tool test
        let test_result = self.execute_tool_test(server_id, &tool_name, &parameters);

        log::info!("✅ 功能测试完成 - 服务器: '{}', 工具: '{}' - 成功: {}", server_name, tool_name, test_result.success);

        // Update the dialog with the result
        if let Some(dialog) = &mut self.ui_state.functionality_test_dialog {
            dialog.test_result = Some(test_result);
            dialog.is_testing = false;
            dialog.test_frame_counter = 0;
            dialog.test_phase = FunctionalityTestPhase::Results;
        }
    }

    /// Execute a tool test with the given parameters
    fn execute_tool_test(&mut self, server_id: Uuid, tool_name: &str, parameters: &std::collections::HashMap<String, String>) -> TestResult {
        // Convert parameters to JSON
        let mut json_params = serde_json::Map::new();
        for (key, value) in parameters {
            // Try to parse as JSON first, otherwise treat as string
            let json_value = if value.trim().starts_with('{') || value.trim().starts_with('[') || value.trim().starts_with('"') {
                serde_json::from_str(value).unwrap_or_else(|_| serde_json::Value::String(value.clone()))
            } else if value.parse::<f64>().is_ok() {
                serde_json::Value::Number(serde_json::Number::from_f64(value.parse().unwrap()).unwrap())
            } else if value.parse::<bool>().is_ok() {
                serde_json::Value::Bool(value.parse().unwrap())
            } else {
                serde_json::Value::String(value.clone())
            };
            json_params.insert(key.clone(), json_value);
        }

        let parameters_json = serde_json::Value::Object(json_params);

        // Execute the tool test using the server manager
        let result = if let Ok(handle) = tokio::runtime::Handle::try_current() {
            handle.block_on(async {
                match self.server_manager.call_tool_for_testing(server_id, tool_name, parameters_json).await {
                    Ok(response) => TestResult {
                        success: true,
                        error_message: None,
                        stdout: format!("工具调用成功 - {}\n响应: {}", tool_name, serde_json::to_string_pretty(&response).unwrap_or_default()),
                        stderr: String::new(),
                    },
                    Err(e) => TestResult {
                        success: false,
                        error_message: Some(e.to_string()),
                        stdout: String::new(),
                        stderr: e.to_string(),
                    }
                }
            })
        } else {
            match tokio::runtime::Runtime::new() {
                Ok(rt) => {
                    rt.block_on(async {
                        match self.server_manager.call_tool_for_testing(server_id, tool_name, parameters_json).await {
                            Ok(response) => TestResult {
                                success: true,
                                error_message: None,
                                stdout: format!("工具调用成功 - {}\n响应: {}", tool_name, serde_json::to_string_pretty(&response).unwrap_or_default()),
                                stderr: String::new(),
                            },
                            Err(e) => TestResult {
                                success: false,
                                error_message: Some(e.to_string()),
                                stdout: String::new(),
                                stderr: e.to_string(),
                            }
                        }
                    })
                }
                Err(e) => TestResult {
                    success: false,
                    error_message: Some(format!("无法创建异步运行时: {}", e)),
                    stdout: String::new(),
                    stderr: format!("无法创建异步运行时: {}", e),
                }
            }
        };

        result
    }

    /// Render tool parameters input fields
    fn render_tool_parameters_static(ui: &mut egui::Ui, input_schema: &serde_json::Value, parameter_inputs: &mut std::collections::HashMap<String, String>) {
        if let Some(properties) = input_schema.get("properties").and_then(|p| p.as_object()) {
            for (param_name, param_schema) in properties {
                let param_type = param_schema.get("type")
                    .and_then(|t| t.as_str())
                    .unwrap_or("string");

                let description = param_schema.get("description")
                    .and_then(|d| d.as_str())
                    .unwrap_or("");

                ui.horizontal(|ui| {
                    ui.label(format!("{}:", param_name));

                    let current_value = parameter_inputs.get(param_name).cloned().unwrap_or_default();
                    let mut new_value = current_value;

                    match param_type {
                        "boolean" => {
                            let mut bool_value = new_value.parse::<bool>().unwrap_or(false);
                            if ui.checkbox(&mut bool_value, "").changed() {
                                new_value = bool_value.to_string();
                            }
                        }
                        "number" | "integer" => {
                            ui.add(TextEdit::singleline(&mut new_value)
                                .hint_text("输入数字..."));
                        }
                        _ => {
                            ui.add(TextEdit::singleline(&mut new_value)
                                .hint_text(if description.is_empty() { "输入参数值..." } else { description }));
                        }
                    }

                    parameter_inputs.insert(param_name.clone(), new_value);
                });

                if !description.is_empty() {
                    ui.label(RichText::new(description).italics().color(Color32::GRAY));
                }
            }
        } else {
            ui.label("此工具不需要参数");
        }
    }

    /// Render the setup phase of functionality testing
    fn render_test_setup_phase_static(ui: &mut egui::Ui, dialog: &mut FunctionalityTestDialog, should_execute_test: &mut bool, should_update_parameters: &mut bool) {
        ui.label("🔧 选择要测试的工具:");
        ui.separator();

        if dialog.available_tools.is_empty() {
            ui.colored_label(Color32::YELLOW, "⚠️ 此服务器没有可用的工具进行测试");
            ui.label("请确保服务器已正确配置并且状态为绿灯");
            return;
        }

        // Tool selection
        ui.horizontal(|ui| {
            ui.label("工具:");
            ComboBox::from_id_source("functionality_test_tool_selection")
                .selected_text(
                    dialog.selected_tool_index
                        .and_then(|i| dialog.available_tools.get(i))
                        .map(|tool| tool.name.as_str())
                        .unwrap_or("选择工具...")
                )
                .show_ui(ui, |ui| {
                    for (index, tool) in dialog.available_tools.iter().enumerate() {
                        let is_selected = dialog.selected_tool_index == Some(index);
                        if ui.selectable_label(is_selected, &tool.name).clicked() {
                            dialog.selected_tool_index = Some(index);
                            *should_update_parameters = true;
                        }
                    }
                });
        });

        // Show tool description if selected
        if let Some(tool_index) = dialog.selected_tool_index {
            if let Some(tool) = dialog.available_tools.get(tool_index) {
                ui.separator();
                ui.label(RichText::new("工具描述:").strong());

                // 为工具描述添加滚动区域，支持长描述
                let description = tool.description.as_deref().unwrap_or("无描述");
                ScrollArea::vertical()
                    .id_source(format!("tool_description_{}", tool_index))
                    .max_height(120.0)  // 限制最大高度为120像素
                    .show(ui, |ui| {
                        ui.label(description);
                    });

                // Parameter inputs
                if let Some(input_schema) = &tool.input_schema {
                    ui.separator();
                    ui.label(RichText::new("参数配置:").strong());

                    // 为参数输入区域添加滚动区域，防止参数过多时挤出按钮
                    ScrollArea::vertical()
                        .id_source(format!("tool_parameters_{}", tool_index))
                        .max_height(200.0)  // 限制最大高度为200像素
                        .show(ui, |ui| {
                            Self::render_tool_parameters_static(ui, input_schema, &mut dialog.parameter_inputs);
                        });
                }

                // Test button
                ui.separator();
                ui.horizontal(|ui| {
                    if ui.add_enabled(true, egui::Button::new("🧪 执行测试")).clicked() {
                        *should_execute_test = true;
                    }
                    ui.label("点击执行测试以验证工具功能");
                });
            }
        }
    }

    /// Render the testing execution phase
    fn render_test_execution_phase_static(ui: &mut egui::Ui, dialog: &FunctionalityTestDialog) {
        ui.vertical_centered(|ui| {
            ui.add_space(20.0);
            ui.spinner();
            ui.add_space(10.0);
            ui.label(RichText::new("正在执行功能测试...").strong());

            if let Some(tool_index) = dialog.selected_tool_index {
                if let Some(tool) = dialog.available_tools.get(tool_index) {
                    ui.label(format!("测试工具: {}", tool.name));
                }
            }

            let progress_text = if dialog.test_frame_counter <= 60 {
                format!("准备测试环境... ({}/60)", dialog.test_frame_counter)
            } else {
                format!("执行工具调用... ({}/120)", dialog.test_frame_counter)
            };
            ui.label(progress_text);
            ui.add_space(20.0);
        });
    }

    /// Render the test results phase
    fn render_test_results_phase_static(ui: &mut egui::Ui, dialog: &mut FunctionalityTestDialog) -> bool {
        let mut update_button_clicked = false;

        if let Some(result) = &dialog.test_result {
            // Status indicator
            let status_color = if result.success { Color32::GREEN } else { Color32::RED };
            let status_text = if result.success { "✅ 测试成功" } else { "❌ 测试失败" };
            ui.colored_label(status_color, RichText::new(status_text).strong());

            ui.separator();

            // Tab buttons
            ui.horizontal(|ui| {
                if ui.selectable_label(dialog.active_tab == FunctionalityTestTab::Summary, "📋 概要").clicked() {
                    dialog.active_tab = FunctionalityTestTab::Summary;
                }
                if ui.selectable_label(dialog.active_tab == FunctionalityTestTab::Details, "📄 详情").clicked() {
                    dialog.active_tab = FunctionalityTestTab::Details;
                }
                if ui.selectable_label(dialog.active_tab == FunctionalityTestTab::Output, "📤 输出").clicked() {
                    dialog.active_tab = FunctionalityTestTab::Output;
                }
            });

            ui.separator();

            // Tab content with scrollable area
            ScrollArea::vertical()
                .max_height(300.0)
                .show(ui, |ui| {
                    match dialog.active_tab {
                        FunctionalityTestTab::Summary => {
                            ui.label(RichText::new("测试概要:").strong());
                            ui.label(format!("状态: {}", if result.success { "✅ 通过" } else { "❌ 失败" }));

                            if let Some(tool_index) = dialog.selected_tool_index {
                                if let Some(tool) = dialog.available_tools.get(tool_index) {
                                    ui.label(format!("测试工具: {}", tool.name));
                                }
                            }

                            if let Some(error) = &result.error_message {
                                ui.colored_label(Color32::RED, format!("错误: {}", error));
                            }

                            // If test was successful, show option to update server status
                            if result.success {
                                ui.separator();
                                ui.label(RichText::new("✅ 功能测试通过！").strong().color(Color32::GREEN));
                                ui.label("服务器功能正常，可以将状态更新为绿灯");

                                // 使用一个标志来表示用户点击了更新按钮
                                // 实际的更新逻辑将在主方法中处理，以避免借用检查问题
                                if ui.button("🟢 更新服务器状态为绿灯").clicked() {
                                    log::info!("🟢 用户请求将服务器状态更新为绿灯");
                                    update_button_clicked = true;
                                }
                            }
                        }
                        FunctionalityTestTab::Details => {
                            ui.label(RichText::new("详细信息:").strong());
                            if !result.stdout.is_empty() {
                                ui.add(TextEdit::multiline(&mut result.stdout.as_str())
                                    .desired_rows(10)
                                    .desired_width(f32::INFINITY)
                                    .code_editor());
                            } else {
                                ui.label("无详细信息");
                            }
                        }
                        FunctionalityTestTab::Output => {
                            ui.label(RichText::new("错误输出:").strong());
                            if !result.stderr.is_empty() {
                                ui.add(TextEdit::multiline(&mut result.stderr.as_str())
                                    .desired_rows(10)
                                    .desired_width(f32::INFINITY)
                                    .code_editor());
                            } else {
                                ui.label("无错误输出");
                            }
                        }
                    }
                });
        }

        update_button_clicked
    }

    /// Update tool parameters when tool selection changes
    fn update_tool_parameters_static(dialog: &mut FunctionalityTestDialog) {
        dialog.parameter_inputs.clear();

        if let Some(tool_index) = dialog.selected_tool_index {
            if let Some(tool) = dialog.available_tools.get(tool_index) {
                if let Some(input_schema) = &tool.input_schema {
                    // Initialize parameter inputs with default values
                    if let Some(properties) = input_schema.get("properties").and_then(|p| p.as_object()) {
                        for (param_name, _param_schema) in properties {
                            dialog.parameter_inputs.insert(param_name.clone(), String::new());
                        }
                    }
                }
            }
        }
    }

    /// Delete a server
    fn delete_server(&mut self, server_id: Uuid) {
        let result = if let Ok(handle) = tokio::runtime::Handle::try_current() {
            handle.block_on(self.server_manager.remove_server(server_id))
                .map_err(|e| e.to_string())
        } else {
            match tokio::runtime::Runtime::new() {
                Ok(rt) => {
                    rt.block_on(self.server_manager.remove_server(server_id))
                        .map_err(|e| e.to_string())
                }
                Err(e) => Err(format!("无法创建异步运行时: {}", e))
            }
        };

        match result {
            Ok(()) => {
                self.ui_state.status_message = Some("服务器删除成功".to_string());
                self.ui_state.server_status_messages.remove(&server_id);
                self.ui_state.server_error_messages.remove(&server_id);
                self.ui_state.server_test_outputs.remove(&server_id);
                self.ui_state.error_details_expanded.remove(&server_id);

                log::info!("Server {} deleted successfully", server_id);

                // 触发MCP事件，通知主应用同步AI助手状态
                // 这将通过事件系统自动触发AI助手的MCP服务器列表更新
                log::info!("🔄 服务器删除完成，将通过MCP事件系统自动同步AI助手状态");
            }
            Err(e) => {
                self.ui_state.error_message = Some(format!("删除服务器失败: {}", e));
                log::error!("Failed to delete server {}: {}", server_id, e);
            }
        }
    }

    /// Get server name by ID
    fn get_server_name_by_id(&self, server_id: Uuid) -> Option<String> {
        for directory in self.server_manager.get_server_directories() {
            for server in &directory.servers {
                if server.id == server_id {
                    return Some(server.name.clone());
                }
            }
        }
        None
    }

    /// Test connection before adding server
    fn test_connection_before_add(&mut self) {
        log::info!("🔗 开始测试连接 - 准备添加服务器");

        // 获取当前配置
        let config = if self.ui_state.add_server_json_mode {
            match self.parse_json_config() {
                Ok(config) => config,
                Err(e) => {
                    self.ui_state.error_message = Some(format!("JSON 解析失败: {}", e));
                    return;
                }
            }
        } else {
            self.new_server_config.clone()
        };

        // 验证配置
        if let Err(e) = self.server_manager.validate_server_config(&config) {
            self.ui_state.error_message = Some(format!("配置验证失败: {}", e));
            return;
        }

        // 清理之前的测试结果
        self.ui_state.connection_test_result = None;
        self.ui_state.tested_capabilities = None;
        self.ui_state.error_message = None;
        self.ui_state.status_message = Some("正在测试连接...".to_string());

        // 异步执行连接测试 - 使用简化的方法避免借用检查问题
        let config_clone = config.clone();

        // 直接在当前线程中执行，使用更短的超时机制快速fallback
        let (result, temp_server_id) = if let Ok(rt) = tokio::runtime::Runtime::new() {
            rt.block_on(async {
                // 添加总体超时 - 45秒（给足够时间进行连接和查询）
                match tokio::time::timeout(
                    tokio::time::Duration::from_secs(45),
                    async {
                        // 临时添加服务器配置到客户端
                        if let Some(client) = self.server_manager.get_rmcp_client_mut() {
                            let temp_server_id = client.add_server_config(config_clone.clone());

                            // 先连接服务器 - 增加连接超时（20秒）以匹配rmcp超时
                            let connect_result = tokio::time::timeout(
                                tokio::time::Duration::from_secs(20),
                                client.connect_server(temp_server_id)
                            ).await;

                            match connect_result {
                                Ok(Ok(())) => {
                                    log::info!("✅ 服务器连接成功，开始查询能力");
                                }
                                Ok(Err(e)) => {
                                    log::warn!("连接服务器失败: {}", e);
                                    return (Err(format!("连接服务器失败: {}", e)), Some(temp_server_id));
                                }
                                Err(_) => {
                                    log::warn!("连接服务器超时");
                                    return (Err("连接服务器超时 (20秒)".to_string()), Some(temp_server_id));
                                }
                            }

                            // 测试连接并获取能力 - 增加查询超时（15秒）
                            let test_result = tokio::time::timeout(
                                tokio::time::Duration::from_secs(15),
                                client.query_server_capabilities(temp_server_id)
                            ).await;

                            match test_result {
                                Ok(Ok(())) => {
                                    // 获取测试结果
                                    if let Some(capabilities) = client.get_server_capabilities(temp_server_id) {
                                        log::info!("✅ 连接测试成功 - 工具:{}, 资源:{}, 提示:{}",
                                            capabilities.tools.len(), capabilities.resources.len(), capabilities.prompts.len());

                                        (Ok(capabilities), Some(temp_server_id))
                                    } else {
                                        (Err("无法获取服务器能力信息".to_string()), Some(temp_server_id))
                                    }
                                }
                                Ok(Err(e)) => {
                                    (Err(format!("查询能力失败: {}", e)), Some(temp_server_id))
                                }
                                Err(_) => {
                                    (Err("查询能力超时 (15秒)".to_string()), Some(temp_server_id))
                                }
                            }
                        } else {
                            (Err("无法获取RMCP客户端".to_string()), None)
                        }
                    }
                ).await {
                    Ok(result) => result,
                    Err(_) => (Err("连接测试总体超时 (45秒)".to_string()), None)
                }
            })
        } else {
            (Err("无法创建异步运行时".to_string()), None)
        };

        // 处理测试结果
        match result {
            Ok(capabilities) => {
                self.ui_state.connection_test_result = Some(TestResult {
                    success: true,
                    error_message: None,
                    stdout: format!("成功获取能力信息:\n- 工具: {}\n- 资源: {}\n- 提示: {}",
                        capabilities.tools.len(), capabilities.resources.len(), capabilities.prompts.len()),
                    stderr: String::new(),
                });
                self.ui_state.tested_capabilities = Some(capabilities);
                self.ui_state.status_message = Some("✅ 连接测试成功！现在可以添加服务器。".to_string());
                self.ui_state.error_message = None;
            }
            Err(e) => {
                self.ui_state.connection_test_result = Some(TestResult {
                    success: false,
                    error_message: Some(e.clone()),
                    stdout: String::new(),
                    stderr: e.clone(),
                });
                self.ui_state.tested_capabilities = None;
                self.ui_state.status_message = None;
                self.ui_state.error_message = Some(format!("❌ 连接测试失败: {}", e));
            }
        }

        // 清理临时服务器配置
        if let (Some(client), Some(temp_id)) = (self.server_manager.get_rmcp_client_mut(), temp_server_id) {
            let _ = client.remove_server_config(temp_id);
        }
    }
}

impl Default for McpUiState {
    fn default() -> Self {
        Self {
            show_add_server: false,
            show_import_dialog: false,
            show_export_dialog: false,

            show_edit_server: false,
            edit_server_json_text: String::new(),
            status_message: None,
            error_message: None,
            server_status_messages: HashMap::new(),
            server_error_messages: HashMap::new(),
            server_test_outputs: HashMap::new(),
            error_details_expanded: HashMap::new(),
            import_file_path: String::new(),
            export_file_path: String::new(),
            export_directory: None,
            add_server_json_mode: false,
            add_server_json_text: String::new(),
            use_real_rmcp: false,
            tool_test_dialog: None,
            functionality_test_dialog: None,
            show_delete_confirmation: false,
            server_to_delete: None,
            connection_test_result: None,
            tested_capabilities: None,
        }
    }
}

impl Default for McpServerConfig {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            name: String::new(),
            description: None,
            transport: TransportConfig::Command {
                command: "npx".to_string(),
                args: vec![
                    "-y".to_string(),
                    "@modelcontextprotocol/server-filesystem".to_string(),
                    ".".to_string(), // 使用当前目录，与Inspector一致
                ],
                env: HashMap::new(),
            },
            enabled: false,
            auto_start: false,
            directory: "Custom".to_string(),
            metadata: HashMap::new(),
            capabilities: None,
            last_health_status: None,
            last_test_time: None,
            last_test_success: None,
        }
    }
}
