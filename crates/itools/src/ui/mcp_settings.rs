use std::collections::HashMap;
use std::path::PathBuf;
use uuid::Uuid;
use egui::{Context, Ui, RichText, Color32, Button, TextEdit, ComboBox};
use tokio::sync::mpsc;

use crate::mcp::{
    McpServerManager, McpServerConfig,
    server_manager::TransportConfig,
    rmcp_client::{McpEvent, ServerCapabilities}
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
}

/// Dialog for testing MCP server tools
#[derive(Debug, Clone)]
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

}

/// Categories of testable items
#[derive(Debug, Clone, Copy, PartialEq)]
enum TestCategory {
    Tools,
    Resources,
    Prompts,
}



/// Result of tool testing
#[derive(Debug, Clone)]
struct ToolTestResult {
    success: bool,
    output: String,
    error: Option<String>,
    duration: std::time::Duration,
    /// STDIN content sent to the MCP server
    stdin: Option<String>,
    /// STDOUT content received from the MCP server
    stdout: Option<String>,
    /// STDERR content received from the MCP server
    stderr: Option<String>,
    /// Raw MCP request sent
    mcp_request: Option<String>,
    /// Raw MCP response received
    mcp_response: Option<String>,
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
        }
    }



    /// Initialize the MCP settings UI synchronously
    pub fn initialize_sync(&mut self) -> anyhow::Result<()> {
        // Initialize server manager synchronously
        self.server_manager.initialize_sync()?;

        // Setup event channel
        let (sender, receiver) = mpsc::unbounded_channel();
        self.event_receiver = Some(receiver);

        // Set event sender in server manager
        self.server_manager.set_event_sender(sender);

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

        ui.heading("MCP 服务器设置");
        ui.separator();

        // Toolbar
        ui.horizontal(|ui| {
            if ui.button("➕ 添加服务器").clicked() {
                self.ui_state.show_add_server = true;
                self.ui_state.add_server_json_mode = false;
                self.ui_state.add_server_json_text.clear();
                self.new_server_config = McpServerConfig::default();
                // Set default directory for JSON mode
                self.new_server_config.directory = "自定义".to_string();
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

        self.render_tool_test_dialog(ctx);
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

                    // Functionality test button
                    if ui.small_button("🧪").on_hover_text("测试服务器功能").clicked() {
                        if let Some(server_id) = self.find_server_id_by_config(config) {
                            self.test_server_tools(server_id, config);
                        }
                    }

                    // Tool testing button (always enabled for testing purposes)
                    if ui.small_button("🔧").on_hover_text("测试工具").clicked() {
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

                    // Buttons
                    ui.horizontal(|ui| {
                        if ui.button("添加服务器").clicked() {
                            let config_result = if self.ui_state.add_server_json_mode {
                                // Parse JSON input
                                self.parse_json_config()
                            } else {
                                // Use form input
                                Ok(self.new_server_config.clone())
                            };

                            match config_result {
                                Ok(config) => {
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

                        if ui.button("取消").clicked() {
                            self.ui_state.show_add_server = false;
                            self.new_server_config = McpServerConfig::default();
                            self.ui_state.add_server_json_text.clear();
                            self.ui_state.add_server_json_mode = false;
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

                        if ui.button("取消").clicked() {
                            self.ui_state.show_edit_server = false;
                            self.ui_state.edit_server_json_text.clear();
                        }
                    });
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
        if ui.button("📋 插入示例 JSON").clicked() {
            self.ui_state.add_server_json_text = self.get_example_json();
        }

        ui.collapsing("JSON 格式说明", |ui| {
            ui.label("支持两种 JSON 格式:");
            ui.separator();

            ui.label("1. 标准 MCP 格式 (推荐):");
            ui.label("• mcpServers: 服务器配置对象 (必填)");
            ui.label("  • 服务器名称: 服务器配置 (必填)");
            ui.label("    • command: 启动命令 (必填)");
            ui.label("    • args: 命令参数数组 (可选)");
            ui.label("    • env: 环境变量对象 (可选)");
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
            command: String,
            args: Option<Vec<String>>,
            env: Option<std::collections::HashMap<String, String>>,
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

        // Convert to internal format
        let transport_config = TransportConfig::Command {
            command: server_config.command,
            args: server_config.args.unwrap_or_default(),
            env: server_config.env.unwrap_or_default(),
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
        })
    }

    /// Get example JSON configuration
    fn get_example_json(&self) -> String {
        serde_json::to_string_pretty(&serde_json::json!({
            "mcpServers": {
                "filesystem": {
                    "command": "npx",
                    "args": [
                        "-y",
                        "@modelcontextprotocol/server-filesystem",
                        "/Users/username/Desktop",
                        "/Users/username/Downloads"
                    ]
                }
            }
        })).unwrap_or_default()
    }

    /// Render transport configuration editor
    fn render_transport_config_editor(&mut self, ui: &mut Ui) {
        let transport_types = ["命令行", "TCP", "Unix Socket", "WebSocket"];
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
                // Command transport
                if let TransportConfig::Command { command, args, .. } = &mut self.new_server_config.transport {
                    ui.horizontal(|ui| {
                        ui.label("命令:");
                        ui.add(TextEdit::singleline(command).desired_width(200.0));
                    });
                    ui.horizontal(|ui| {
                        ui.label("参数:");
                        let args_text = args.join(" ");
                        let mut args_input = args_text.clone();
                        ui.add(TextEdit::singleline(&mut args_input).desired_width(200.0));
                        if args_input != args_text {
                            *args = args_input.split_whitespace().map(|s| s.to_string()).collect();
                        }
                    });
                } else {
                    self.new_server_config.transport = TransportConfig::Command {
                        command: String::new(),
                        args: Vec::new(),
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
                // WebSocket transport
                if let TransportConfig::WebSocket { url } = &mut self.new_server_config.transport {
                    ui.horizontal(|ui| {
                        ui.label("URL:");
                        ui.add(TextEdit::singleline(url).desired_width(200.0));
                    });
                } else {
                    self.new_server_config.transport = TransportConfig::WebSocket {
                        url: String::new(),
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
                        self.server_capabilities.insert(server_id, capabilities);
                        self.ui_state.status_message = Some(format!("服务器 {} 能力已更新", server_id));
                        log::info!("UI cached capabilities for server: {} (total cached: {})", server_id, self.server_capabilities.len());
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
            }

            // Resources
            if !capabilities.resources.is_empty() {
                ui.horizontal(|ui| {
                    ui.label("📁 资源:");
                    ui.label(format!("{} 个", capabilities.resources.len()));
                });

                ui.indent("resources", |ui| {
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
            }

            // Prompts
            if !capabilities.prompts.is_empty() {
                ui.horizontal(|ui| {
                    ui.label("💬 提示:");
                    ui.label(format!("{} 个", capabilities.prompts.len()));
                });

                ui.indent("prompts", |ui| {
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
            }

            ui.separator();
        });
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

        // First, try to connect and get capabilities
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

        match connect_result {
            Ok(_) => {
                // Connection successful, capabilities will be fetched automatically via events

                // Check if we have capabilities now
                if let Some(capabilities) = self.server_capabilities.get(&server_id).cloned() {
                    // Open tool testing dialog
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

                    };
                    self.ui_state.tool_test_dialog = Some(dialog);

                    self.ui_state.server_status_messages.insert(
                        server_id,
                        "连接成功，工具测试对话框已打开".to_string()
                    );
                } else {
                    // Connected but no capabilities yet, show status
                    self.ui_state.server_status_messages.insert(
                        server_id,
                        "连接成功，正在获取服务器能力信息...".to_string()
                    );
                }
            }
            Err(error) => {
                // Connection failed, fall back to basic connection test
                log::warn!("Connection failed for server '{}', falling back to connection test: {}", server_name, error);
                self.test_server_connection(server_id);
            }
        }
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
                                                }
                                            }
                                        });
                                } else {
                                    ui.colored_label(Color32::GRAY, "该服务器没有可用的提示");
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
                                ui.label("测试中...");
                            }

                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if ui.button("关闭").clicked() {
                                    should_close = true;
                                }
                            });
                        });

                        // Simple test result display
                        if let Some(result) = &dialog.test_result {
                            ui.separator();
                            ui.label(RichText::new("测试结果:").strong());
                            let status_color = if result.success { Color32::GREEN } else { Color32::RED };
                            let status_text = if result.success { "✅ 成功" } else { "❌ 失败" };
                            ui.colored_label(status_color, status_text);

                            if !result.output.is_empty() {
                                ui.label("输出:");
                                ui.add(TextEdit::multiline(&mut result.output.as_str())
                                    .desired_rows(5)
                                    .desired_width(f32::INFINITY));
                            }
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
                // Set testing state
                if let Some(dialog) = &mut self.ui_state.tool_test_dialog {
                    dialog.is_testing = true;
                    dialog.test_result = None;
                }

                // Execute the test with cloned data
                let result = self.execute_tool_test_with_data(
                    server_id,
                    selected_category,
                    selected_tool_index,
                    selected_resource_index,
                    selected_prompt_index,
                    &capabilities,
                    &parameter_inputs
                );

                // Update the dialog with the result
                if let Some(dialog) = &mut self.ui_state.tool_test_dialog {
                    dialog.test_result = Some(result);
                    dialog.is_testing = false;
                }
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
                            output: String::new(),
                            error: Some("工具不存在".to_string()),
                            duration: start_time.elapsed(),
                            stdin: None,
                            stdout: None,
                            stderr: None,
                            mcp_request: None,
                            mcp_response: None,
                        }
                    }
                } else {
                    ToolTestResult {
                        success: false,
                        output: String::new(),
                        error: Some("未选择工具".to_string()),
                        duration: start_time.elapsed(),
                        stdin: None,
                        stdout: None,
                        stderr: None,
                        mcp_request: None,
                        mcp_response: None,
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
                            output: String::new(),
                            error: Some("资源不存在".to_string()),
                            duration: start_time.elapsed(),
                            stdin: None,
                            stdout: None,
                            stderr: None,
                            mcp_request: None,
                            mcp_response: None,
                        }
                    }
                } else {
                    ToolTestResult {
                        success: false,
                        output: String::new(),
                        error: Some("未选择资源".to_string()),
                        duration: start_time.elapsed(),
                        stdin: None,
                        stdout: None,
                        stderr: None,
                        mcp_request: None,
                        mcp_response: None,
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
                            output: String::new(),
                            error: Some("提示不存在".to_string()),
                            duration: start_time.elapsed(),
                            stdin: None,
                            stdout: None,
                            stderr: None,
                            mcp_request: None,
                            mcp_response: None,
                        }
                    }
                } else {
                    ToolTestResult {
                        success: false,
                        output: String::new(),
                        error: Some("未选择提示".to_string()),
                        duration: start_time.elapsed(),
                        stdin: None,
                        stdout: None,
                        stderr: None,
                        mcp_request: None,
                        mcp_response: None,
                    }
                }
            }
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

        ToolTestResult {
            success,
            output: output.clone(),
            error,
            duration: start_time.elapsed(),
            stdin: Some(request_str),
            stdout: Some(stdout_content),
            stderr: None,
            mcp_request: Some(serde_json::to_string_pretty(&mcp_request).unwrap_or_default()),
            mcp_response: Some(output),
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

        ToolTestResult {
            success,
            output: output.clone(),
            error,
            duration: start_time.elapsed(),
            stdin: Some(request_str),
            stdout: Some(stdout_content),
            stderr: None,
            mcp_request: Some(serde_json::to_string_pretty(&mcp_request).unwrap_or_default()),
            mcp_response: Some(output),
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

        ToolTestResult {
            success,
            output: output.clone(),
            error,
            duration: start_time.elapsed(),
            stdin: Some(request_str),
            stdout: Some(stdout_content),
            stderr: None,
            mcp_request: Some(serde_json::to_string_pretty(&mcp_request).unwrap_or_default()),
            mcp_response: Some(output),
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
                command: String::new(),
                args: Vec::new(),
                env: HashMap::new(),
            },
            enabled: false,
            auto_start: false,
            directory: "Custom".to_string(),
            metadata: HashMap::new(),
        }
    }
}
