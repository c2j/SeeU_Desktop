use std::collections::HashMap;
use std::path::PathBuf;
use uuid::Uuid;
use egui::{Context, Ui, RichText, Color32, Button, TextEdit, ComboBox};
use tokio::sync::mpsc;

use crate::mcp::{
    McpServerManager, McpServerConfig,
    server_manager::{ServerDirectory, TransportConfig},
    rmcp_client::{ConnectionStatus, McpEvent}
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

    /// Show server details
    show_server_details: bool,

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
        }
    }

    /// Initialize the MCP settings UI
    pub async fn initialize(&mut self) -> anyhow::Result<()> {
        self.server_manager.initialize().await?;
        
        // Setup event channel
        let (sender, receiver) = mpsc::unbounded_channel();
        self.event_receiver = Some(receiver);
        
        // TODO: Set event sender in server manager
        // self.server_manager.set_event_sender(sender);
        
        Ok(())
    }

    /// Render the MCP settings UI
    pub fn render(&mut self, ctx: &Context, ui: &mut Ui) {
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
        self.render_server_details_dialog(ctx);
    }

    /// Render server directories in a tree structure
    fn render_server_directories(&mut self, ui: &mut Ui) {
        let directories = self.server_manager.get_server_directories();

        for directory in directories {
            let expanded = self.directory_expanded.get(&directory.name).copied().unwrap_or(false);
            
            ui.horizontal(|ui| {
                let expand_button = if expanded { "📂" } else { "📁" };
                if ui.button(expand_button).clicked() {
                    self.directory_expanded.insert(directory.name.clone(), !expanded);
                }

                ui.label(RichText::new(&directory.name).strong());
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
                // Status indicator
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

                    if ui.small_button("🚀").on_hover_text("测试连接").clicked() {
                        // Test server connection
                        if let Some(server_id) = self.find_server_id_by_config(config) {
                            self.test_server_connection(server_id);
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
        let transport_types_en = ["Command", "TCP", "Unix Socket", "WebSocket"];
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

        // Update transport config based on selection
        match current_type {
            0 => self.render_command_transport_editor(ui),
            1 => self.render_tcp_transport_editor(ui),
            2 => self.render_unix_transport_editor(ui),
            3 => self.render_websocket_transport_editor(ui),
            _ => {}
        }
    }

    /// Render command transport editor
    fn render_command_transport_editor(&mut self, ui: &mut Ui) {
        if let TransportConfig::Command { command, args, env } = &mut self.new_server_config.transport {
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

    /// Render TCP transport editor
    fn render_tcp_transport_editor(&mut self, ui: &mut Ui) {
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

    /// Render Unix socket transport editor
    fn render_unix_transport_editor(&mut self, ui: &mut Ui) {
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

    /// Render WebSocket transport editor
    fn render_websocket_transport_editor(&mut self, ui: &mut Ui) {
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

    /// Render server details dialog
    fn render_server_details_dialog(&mut self, ctx: &Context) {
        if !self.ui_state.show_server_details {
            return;
        }

        egui::Window::new("服务器详情")
            .collapsible(false)
            .resizable(true)
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    if let Some(server_id) = self.selected_server {
                        if let Some(info) = self.server_manager.get_server_info(server_id) {
                            ui.label(format!("名称: {}", info.name));
                            if let Some(desc) = &info.description {
                                ui.label(format!("描述: {}", desc));
                            }
                            ui.label(format!("状态: {:?}", info.status));

                            if let Some(capabilities) = &info.capabilities {
                                ui.separator();
                                ui.label("服务器能力:");
                                ui.label(format!("工具: {}", capabilities.tools.len()));
                                ui.label(format!("资源: {}", capabilities.resources.len()));
                                ui.label(format!("提示: {}", capabilities.prompts.len()));
                            }
                        }
                    }

                    ui.separator();

                    if ui.button("关闭").clicked() {
                        self.ui_state.show_server_details = false;
                    }
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
                    McpEvent::CapabilitiesUpdated(server_id, _capabilities) => {
                        self.ui_state.status_message = Some(format!("服务器 {} 能力已更新", server_id));
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
            // Connect server
            if let Ok(handle) = tokio::runtime::Handle::try_current() {
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
            }
        };

        match result {
            Ok(()) => {
                let message = if currently_enabled {
                    "服务器已断开连接"
                } else {
                    "服务器已连接"
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

    /// Refresh server list
    fn refresh_server_list(&mut self) {
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
}

impl Default for McpUiState {
    fn default() -> Self {
        Self {
            show_add_server: false,
            show_import_dialog: false,
            show_export_dialog: false,
            show_server_details: false,
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
