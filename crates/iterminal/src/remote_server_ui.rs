use crate::remote_server::{AuthMethod, RemoteServer};
use crate::remote_server_manager::RemoteServerManager;
use crate::ssh_connection::{ConnectionTestResult, SshConnectionBuilder};
use eframe::egui::{self, Color32, RichText, Ui};
use std::collections::HashMap;
use std::path::PathBuf;
use uuid::Uuid;

/// 远程服务器UI状态
#[derive(Debug)]
pub struct RemoteServerUI {
    /// 服务器管理器
    pub manager: RemoteServerManager,
    /// 是否显示添加对话框
    show_add_dialog: bool,
    /// 是否显示编辑对话框
    show_edit_dialog: bool,
    /// 正在编辑的服务器
    editing_server: Option<RemoteServer>,
    /// 搜索查询
    search_query: String,
    /// 选中的服务器ID
    selected_server_id: Option<Uuid>,
    /// 是否显示密码
    show_password: bool,
    /// 连接测试状态
    test_connection_status: HashMap<Uuid, ConnectionTestStatus>,
    /// 是否显示导入/导出对话框
    show_import_export_dialog: bool,
    /// 标签过滤器
    tag_filter: String,
    /// 排序方式
    sort_by: SortBy,
    /// 是否显示已禁用的服务器
    show_disabled: bool,
    /// 是否显示SSH状态对话框
    show_ssh_status_dialog: bool,
    /// 是否显示替代方案对话框
    show_alternatives_dialog: bool,
}

/// 连接测试状态
#[derive(Debug, Clone)]
pub enum ConnectionTestStatus {
    None,
    Testing,
    Success,
    Failed(String),
}

/// 排序方式
#[derive(Debug, Clone, PartialEq)]
pub enum SortBy {
    Name,
    Host,
    LastConnected,
    ConnectionCount,
}

/// 远程服务器操作
#[derive(Debug, Clone)]
pub enum RemoteServerAction {
    Connect(Uuid),
    Edit(Uuid),
    Delete(Uuid),
    TestConnection(Uuid),
    Import,
    Export,
    ToggleEnabled(Uuid),
}

impl RemoteServerUI {
    /// 创建新的远程服务器UI
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let manager = RemoteServerManager::new()?;
        
        Ok(Self {
            manager,
            show_add_dialog: false,
            show_edit_dialog: false,
            editing_server: None,
            search_query: String::new(),
            selected_server_id: None,
            show_password: false,
            test_connection_status: HashMap::new(),
            show_import_export_dialog: false,
            tag_filter: String::new(),
            sort_by: SortBy::Name,
            show_disabled: true,
            show_ssh_status_dialog: false,
            show_alternatives_dialog: false,
        })
    }

    /// 渲染远程服务器管理界面
    pub fn render(&mut self, ui: &mut Ui, ctx: &egui::Context) -> Option<RemoteServerAction> {
        let mut action = None;

        ui.vertical(|ui| {
            // 标题和工具栏
            ui.horizontal(|ui| {
                ui.heading("远程服务器管理");
                ui.add_space(10.0);
                
                // 添加服务器按钮
                if ui.button("➕ 添加服务器").clicked() {
                    self.show_add_dialog = true;
                    self.editing_server = Some(RemoteServer::default());
                }
                
                // 导入/导出按钮
                if ui.button("📁 导入/导出").clicked() {
                    self.show_import_export_dialog = true;
                }
                
                // SSH状态信息按钮
                if ui.button("🔧 SSH状态").clicked() {
                    self.show_ssh_status_dialog = true;
                }

                // SSH替代方案按钮
                if ui.button("🔄 连接方案").clicked() {
                    self.show_alternatives_dialog = true;
                }

                // 保存按钮
                if self.manager.has_unsaved_changes() {
                    if ui.button("💾 保存").clicked() {
                        if let Err(e) = self.manager.save_to_file() {
                            log::error!("保存远程服务器配置失败: {}", e);
                        }
                    }
                }
            });

            ui.separator();

            // 搜索和过滤器
            ui.horizontal(|ui| {
                ui.label("搜索:");
                ui.text_edit_singleline(&mut self.search_query);
                
                ui.add_space(10.0);
                ui.label("标签:");
                ui.text_edit_singleline(&mut self.tag_filter);
                
                ui.add_space(10.0);
                ui.label("排序:");
                egui::ComboBox::from_label("")
                    .selected_text(format!("{:?}", self.sort_by))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.sort_by, SortBy::Name, "名称");
                        ui.selectable_value(&mut self.sort_by, SortBy::Host, "主机");
                        ui.selectable_value(&mut self.sort_by, SortBy::LastConnected, "最后连接");
                        ui.selectable_value(&mut self.sort_by, SortBy::ConnectionCount, "连接次数");
                    });
                
                ui.add_space(10.0);
                ui.checkbox(&mut self.show_disabled, "显示已禁用");
            });

            ui.separator();

            // 服务器列表
            if let Some(server_action) = self.render_server_list(ui) {
                action = Some(server_action);
            }
        });

        // 渲染对话框
        if self.show_add_dialog || self.show_edit_dialog {
            if let Some(dialog_action) = self.render_server_dialog(ctx) {
                action = Some(dialog_action);
            }
        }

        if self.show_import_export_dialog {
            self.render_import_export_dialog(ctx);
        }

        // 渲染SSH状态对话框
        if self.show_ssh_status_dialog {
            self.render_ssh_status_dialog(ctx);
        }

        // 渲染替代方案对话框
        if self.show_alternatives_dialog {
            self.render_alternatives_dialog(ctx);
        }

        action
    }

    /// 渲染服务器列表
    fn render_server_list(&mut self, ui: &mut Ui) -> Option<RemoteServerAction> {
        let mut action = None;

        // 获取过滤后的服务器列表
        let servers = self.get_filtered_servers();

        if servers.is_empty() {
            ui.centered_and_justified(|ui| {
                ui.label("暂无远程服务器配置");
            });
            return None;
        }

        // 表格标题
        ui.horizontal(|ui| {
            ui.label(RichText::new("名称").strong());
            ui.add_space(100.0);
            ui.label(RichText::new("连接信息").strong());
            ui.add_space(150.0);
            ui.label(RichText::new("认证").strong());
            ui.add_space(80.0);
            ui.label(RichText::new("状态").strong());
            ui.add_space(80.0);
            ui.label(RichText::new("操作").strong());
        });

        ui.separator();

        // 滚动区域 - 克隆服务器数据以避免借用问题
        let servers_data: Vec<_> = servers.iter().map(|s| (*s).clone()).collect();

        egui::ScrollArea::vertical()
            .max_height(400.0)
            .show(ui, |ui| {
                for server in servers_data {
                    if let Some(server_action) = self.render_server_row(ui, &server) {
                        action = Some(server_action);
                    }
                    ui.separator();
                }
            });

        action
    }

    /// 渲染单个服务器行
    fn render_server_row(&mut self, ui: &mut Ui, server: &RemoteServer) -> Option<RemoteServerAction> {
        let mut action = None;
        
        ui.horizontal(|ui| {
            // 服务器名称和状态指示器
            ui.vertical(|ui| {
                let name_text = if server.enabled {
                    RichText::new(&server.name)
                } else {
                    RichText::new(&server.name).color(Color32::GRAY)
                };
                ui.label(name_text);
                
                // 标签
                if !server.tags.is_empty() {
                    ui.horizontal(|ui| {
                        for tag in &server.tags {
                            ui.small(RichText::new(format!("#{}", tag)).color(Color32::BLUE));
                        }
                    });
                }
            });

            ui.add_space(20.0);

            // 连接信息
            ui.vertical(|ui| {
                ui.label(&server.get_connection_string());
                if let Some(ref desc) = server.description {
                    ui.small(RichText::new(desc).color(Color32::GRAY));
                }
                if let Some(last_connected) = server.last_connected {
                    let time_str = last_connected.format("%Y-%m-%d %H:%M").to_string();
                    ui.small(RichText::new(format!("最后连接: {}", time_str)).color(Color32::GRAY));
                }
            });

            ui.add_space(20.0);

            // 认证方式
            ui.label(server.get_auth_method_display());

            ui.add_space(20.0);

            // 连接测试状态
            match self.test_connection_status.get(&server.id) {
                Some(ConnectionTestStatus::Testing) => {
                    ui.spinner();
                    ui.small("测试中...");
                }
                Some(ConnectionTestStatus::Success) => {
                    ui.small(RichText::new("✓ 连接正常").color(Color32::GREEN));
                }
                Some(ConnectionTestStatus::Failed(msg)) => {
                    ui.small(RichText::new("✗ 连接失败").color(Color32::RED))
                        .on_hover_text(msg);
                }
                _ => {
                    ui.small("未测试");
                }
            }

            ui.add_space(20.0);

            // 操作按钮
            ui.horizontal(|ui| {
                // 连接按钮
                if server.enabled {
                    if ui.button("🔗 连接").clicked() {
                        action = Some(RemoteServerAction::Connect(server.id));
                    }
                } else {
                    ui.add_enabled(false, egui::Button::new("🔗 连接"));
                }

                // 测试连接按钮
                let (test_button_text, test_enabled) = match self.test_connection_status.get(&server.id) {
                    Some(ConnectionTestStatus::Testing) => ("⏳ 测试中", false),
                    Some(ConnectionTestStatus::Success) => ("✅ 成功", true),
                    Some(ConnectionTestStatus::Failed(_)) => ("❌ 失败", true),
                    _ => ("🔍 测试", true),
                };

                if ui.add_enabled(test_enabled, egui::Button::new(test_button_text)).clicked() {
                    log::info!("用户点击测试连接按钮，服务器: {} ({})", server.name, server.id);
                    action = Some(RemoteServerAction::TestConnection(server.id));
                }

                // 如果正在测试中，添加一个取消按钮
                if matches!(self.test_connection_status.get(&server.id), Some(ConnectionTestStatus::Testing)) {
                    if ui.small_button("✖").on_hover_text("取消测试").clicked() {
                        log::info!("用户取消连接测试，服务器: {}", server.name);
                        self.test_connection_status.remove(&server.id);
                    }
                }

                // 编辑按钮
                if ui.button("✏️ 编辑").clicked() {
                    action = Some(RemoteServerAction::Edit(server.id));
                }

                // 启用/禁用按钮
                let toggle_text = if server.enabled { "禁用" } else { "启用" };
                if ui.button(toggle_text).clicked() {
                    action = Some(RemoteServerAction::ToggleEnabled(server.id));
                }

                // 删除按钮
                if ui.button(RichText::new("🗑️ 删除").color(Color32::RED)).clicked() {
                    action = Some(RemoteServerAction::Delete(server.id));
                }
            });
        });

        action
    }

    /// 获取过滤后的服务器列表
    fn get_filtered_servers(&self) -> Vec<&RemoteServer> {
        let mut servers: Vec<&RemoteServer> = if self.search_query.is_empty() && self.tag_filter.is_empty() {
            self.manager.list_servers()
        } else {
            let mut results = Vec::new();
            
            // 搜索过滤
            if !self.search_query.is_empty() {
                results.extend(self.manager.search_servers(&self.search_query));
            } else {
                results.extend(self.manager.list_servers());
            }
            
            // 标签过滤
            if !self.tag_filter.is_empty() {
                results.retain(|server| {
                    server.tags.iter().any(|tag| tag.contains(&self.tag_filter))
                });
            }
            
            results
        };

        // 启用状态过滤
        if !self.show_disabled {
            servers.retain(|server| server.enabled);
        }

        // 排序
        match self.sort_by {
            SortBy::Name => servers.sort_by(|a, b| a.name.cmp(&b.name)),
            SortBy::Host => servers.sort_by(|a, b| a.host.cmp(&b.host)),
            SortBy::LastConnected => servers.sort_by(|a, b| b.last_connected.cmp(&a.last_connected)),
            SortBy::ConnectionCount => servers.sort_by(|a, b| b.connection_count.cmp(&a.connection_count)),
        }

        servers
    }

    /// 开始连接测试
    pub fn start_connection_test(&mut self, server_id: Uuid) {
        log::info!("收到连接测试请求，服务器ID: {}", server_id);

        // 如果已经在测试中，先清除状态
        if matches!(self.test_connection_status.get(&server_id), Some(ConnectionTestStatus::Testing)) {
            log::info!("服务器 {} 已在测试中，清除状态后重新测试", server_id);
        }

        self.test_connection_status.insert(server_id, ConnectionTestStatus::Testing);

        if let Some(server) = self.manager.get_server(server_id) {
            let server_clone = server.clone();
            let server_name = server.name.clone();

            log::info!("开始测试连接到服务器: {} ({})", server_name, server.get_connection_string());

            // 同步执行连接测试（简化实现）
            match SshConnectionBuilder::test_connection(&server_clone) {
                Ok(result) => {
                    log::info!("服务器 {} 连接测试结果: {:?}", server_name, result);
                    self.update_connection_test_result(server_id, result);
                }
                Err(e) => {
                    log::error!("服务器 {} 连接测试失败: {}", server_name, e);
                    self.update_connection_test_result(server_id, ConnectionTestResult::Failed(e));
                }
            }
        } else {
            log::error!("找不到服务器ID: {}", server_id);
            self.test_connection_status.remove(&server_id);
        }
    }

    /// 更新连接测试结果
    pub fn update_connection_test_result(&mut self, server_id: Uuid, result: ConnectionTestResult) {
        let status = match result {
            ConnectionTestResult::Success => ConnectionTestStatus::Success,
            other => ConnectionTestStatus::Failed(other.get_display_text()),
        };

        self.test_connection_status.insert(server_id, status);
    }

    /// 获取连接测试状态
    pub fn get_connection_test_status(&self, server_id: Uuid) -> Option<&ConnectionTestStatus> {
        self.test_connection_status.get(&server_id)
    }

    /// 显示添加服务器对话框
    pub fn show_add_dialog(&mut self) {
        self.show_add_dialog = true;
        self.editing_server = Some(RemoteServer::default());
    }

    /// 显示编辑服务器对话框
    pub fn show_edit_dialog(&mut self, server_id: Uuid) {
        if let Some(server) = self.manager.get_server(server_id) {
            self.show_edit_dialog = true;
            self.editing_server = Some(server.clone());
        }
    }

    /// 切换服务器启用状态
    pub fn toggle_server_enabled(&mut self, server_id: Uuid) -> Result<(), String> {
        if let Some(server) = self.manager.get_server_mut(server_id) {
            server.enabled = !server.enabled;
            Ok(())
        } else {
            Err("服务器不存在".to_string())
        }
    }

    /// 删除服务器
    pub fn delete_server(&mut self, server_id: Uuid) -> Result<(), String> {
        self.test_connection_status.remove(&server_id);
        self.manager.remove_server(server_id)
    }

    /// 渲染紧凑的服务器快速访问面板
    pub fn render_compact_panel(&mut self, ui: &mut Ui, ctx: &egui::Context) -> Option<RemoteServerAction> {
        let mut action = None;

        ui.vertical(|ui| {
            // 搜索框
            ui.horizontal(|ui| {
                ui.label("🔍");
                ui.text_edit_singleline(&mut self.search_query);
                if ui.small_button("✖").clicked() {
                    self.search_query.clear();
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.small_button("➕").on_hover_text("添加服务器").clicked() {
                        self.show_add_dialog = true;
                        self.editing_server = Some(RemoteServer::default());
                    }
                });
            });

            ui.add_space(5.0);

            // 服务器列表 - 紧凑显示
            let servers = self.get_filtered_servers();

            if servers.is_empty() {
                ui.centered_and_justified(|ui| {
                    ui.small("暂无服务器配置");
                });
            } else {
                // 克隆服务器数据以避免借用问题
                let servers_data: Vec<_> = servers.iter().take(10).map(|s| (*s).clone()).collect();
                let total_count = servers.len();

                egui::ScrollArea::vertical()
                    .max_height(200.0)
                    .show(ui, |ui| {
                        for server in servers_data {
                            if let Some(server_action) = self.render_compact_server_row(ui, &server) {
                                action = Some(server_action);
                            }
                        }

                        if total_count > 10 {
                            ui.small(format!("还有 {} 个服务器...", total_count - 10));
                        }
                    });
            }
        });

        // 渲染对话框
        if self.show_add_dialog || self.show_edit_dialog {
            if let Some(dialog_action) = self.render_server_dialog(ctx) {
                action = Some(dialog_action);
            }
        }

        action
    }

    /// 渲染紧凑的服务器行
    fn render_compact_server_row(&mut self, ui: &mut Ui, server: &RemoteServer) -> Option<RemoteServerAction> {
        let mut action = None;

        ui.horizontal(|ui| {
            // 状态指示器
            let (status_color, status_text) = if !server.enabled {
                (Color32::RED, "●")
            } else {
                match self.test_connection_status.get(&server.id) {
                    Some(ConnectionTestStatus::Testing) => (Color32::YELLOW, "⏳"),
                    Some(ConnectionTestStatus::Success) => (Color32::GREEN, "✓"),
                    Some(ConnectionTestStatus::Failed(_)) => (Color32::RED, "✗"),
                    _ => (Color32::GRAY, "●"),
                }
            };

            ui.colored_label(status_color, status_text);

            // 服务器信息
            ui.vertical(|ui| {
                let name_text = if server.enabled {
                    RichText::new(&server.name).strong()
                } else {
                    RichText::new(&server.name).color(Color32::GRAY)
                };
                ui.label(name_text);

                ui.horizontal(|ui| {
                    ui.small(format!("{}@{}", server.username, server.host));
                    if server.port != 22 {
                        ui.small(format!(":{}", server.port));
                    }
                    ui.small(format!("({})", server.get_auth_method_display()));
                });
            });

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // 操作按钮
                if server.enabled {
                    if ui.small_button("🔗").on_hover_text("连接").clicked() {
                        log::info!("紧凑面板：用户点击连接按钮，服务器: {}", server.name);
                        action = Some(RemoteServerAction::Connect(server.id));
                    }
                } else {
                    ui.add_enabled(false, egui::Button::new("🔗").small());
                }

                // 连接测试按钮
                let test_button_text = match self.test_connection_status.get(&server.id) {
                    Some(ConnectionTestStatus::Testing) => "⏳",
                    Some(ConnectionTestStatus::Success) => "✅",
                    Some(ConnectionTestStatus::Failed(_)) => "❌",
                    _ => "🔍",
                };

                if ui.small_button(test_button_text).on_hover_text("测试连接").clicked() {
                    log::info!("紧凑面板：用户点击测试连接按钮，服务器: {} ({})", server.name, server.id);
                    action = Some(RemoteServerAction::TestConnection(server.id));
                }

                if ui.small_button("✏️").on_hover_text("编辑").clicked() {
                    log::info!("紧凑面板：用户点击编辑按钮，服务器: {}", server.name);
                    action = Some(RemoteServerAction::Edit(server.id));
                }
            });
        });

        ui.separator();
        action
    }

    /// 渲染服务器添加/编辑对话框
    fn render_server_dialog(&mut self, ctx: &egui::Context) -> Option<RemoteServerAction> {
        let mut action = None;
        let mut close_dialog = false;

        if let Some(ref mut server) = self.editing_server {
            let title = if self.show_add_dialog { "添加远程服务器" } else { "编辑远程服务器" };

            egui::Window::new(title)
                .collapsible(false)
                .resizable(true)
                .default_width(500.0)
                .show(ctx, |ui| {
                    ui.vertical(|ui| {
                        // 基本信息
                        ui.group(|ui| {
                            ui.vertical(|ui| {
                                ui.label(RichText::new("基本信息").strong());

                                ui.horizontal(|ui| {
                                    ui.label("名称:");
                                    ui.text_edit_singleline(&mut server.name);
                                });

                                ui.horizontal(|ui| {
                                    ui.label("主机:");
                                    ui.text_edit_singleline(&mut server.host);
                                });

                                ui.horizontal(|ui| {
                                    ui.label("端口:");
                                    ui.add(egui::DragValue::new(&mut server.port).range(1..=65535));
                                });

                                ui.horizontal(|ui| {
                                    ui.label("用户名:");
                                    ui.text_edit_singleline(&mut server.username);
                                });

                                ui.horizontal(|ui| {
                                    ui.label("工作目录:");
                                    let mut working_dir = server.working_directory.clone().unwrap_or_default();
                                    ui.text_edit_singleline(&mut working_dir);
                                    server.working_directory = if working_dir.is_empty() { None } else { Some(working_dir) };
                                });
                            });
                        });

                        ui.add_space(10.0);

                        // 认证方式
                        ui.group(|ui| {
                            ui.vertical(|ui| {
                                ui.label(RichText::new("认证方式").strong());

                                let mut auth_type = match &server.auth_method {
                                    AuthMethod::Password(_) => 0,
                                    AuthMethod::PrivateKey { .. } => 1,
                                    AuthMethod::Agent => 2,
                                };

                                ui.horizontal(|ui| {
                                    ui.radio_value(&mut auth_type, 0, "密码");
                                    ui.radio_value(&mut auth_type, 1, "私钥");
                                    ui.radio_value(&mut auth_type, 2, "SSH Agent");
                                });

                                match auth_type {
                                    0 => {
                                        // 密码认证
                                        let mut password = match &server.auth_method {
                                            AuthMethod::Password(pwd) => pwd.clone(),
                                            _ => String::new(),
                                        };

                                        ui.horizontal(|ui| {
                                            ui.label("密码:");
                                            if self.show_password {
                                                ui.text_edit_singleline(&mut password);
                                            } else {
                                                ui.add(egui::TextEdit::singleline(&mut password).password(true));
                                            }
                                            if ui.button(if self.show_password { "隐藏" } else { "显示" }).clicked() {
                                                self.show_password = !self.show_password;
                                            }
                                        });

                                        server.auth_method = AuthMethod::Password(password);
                                    }
                                    1 => {
                                        // 私钥认证
                                        let (key_path, mut passphrase) = match &server.auth_method {
                                            AuthMethod::PrivateKey { key_path, passphrase } => {
                                                (key_path.clone(), passphrase.clone().unwrap_or_default())
                                            }
                                            _ => (PathBuf::new(), String::new()),
                                        };

                                        ui.horizontal(|ui| {
                                            ui.label("私钥文件:");
                                            ui.text_edit_singleline(&mut key_path.to_string_lossy().to_string());
                                            if ui.button("浏览").clicked() {
                                                // TODO: 实现文件选择对话框
                                            }
                                        });

                                        ui.horizontal(|ui| {
                                            ui.label("私钥密码:");
                                            if self.show_password {
                                                ui.text_edit_singleline(&mut passphrase);
                                            } else {
                                                ui.add(egui::TextEdit::singleline(&mut passphrase).password(true));
                                            }
                                            if ui.button(if self.show_password { "隐藏" } else { "显示" }).clicked() {
                                                self.show_password = !self.show_password;
                                            }
                                        });

                                        server.auth_method = AuthMethod::PrivateKey {
                                            key_path,
                                            passphrase: if passphrase.is_empty() { None } else { Some(passphrase) },
                                        };
                                    }
                                    2 => {
                                        // SSH Agent认证
                                        ui.label("使用SSH Agent进行认证");
                                        server.auth_method = AuthMethod::Agent;
                                    }
                                    _ => {}
                                }
                            });
                        });

                        ui.add_space(10.0);

                        // 其他选项
                        ui.group(|ui| {
                            ui.vertical(|ui| {
                                ui.label(RichText::new("其他选项").strong());

                                ui.horizontal(|ui| {
                                    ui.label("描述:");
                                    let mut description = server.description.clone().unwrap_or_default();
                                    ui.text_edit_multiline(&mut description);
                                    server.description = if description.is_empty() { None } else { Some(description) };
                                });

                                ui.checkbox(&mut server.enabled, "启用此服务器");
                            });
                        });

                        ui.add_space(10.0);

                        // 按钮
                        ui.horizontal(|ui| {
                            if ui.button("保存").clicked() {
                                match server.validate() {
                                    Ok(_) => {
                                        if self.show_add_dialog {
                                            if let Err(e) = self.manager.add_server(server.clone()) {
                                                log::error!("添加服务器失败: {}", e);
                                            } else {
                                                close_dialog = true;
                                            }
                                        } else {
                                            if let Err(e) = self.manager.update_server(server.clone()) {
                                                log::error!("更新服务器失败: {}", e);
                                            } else {
                                                close_dialog = true;
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        log::error!("服务器配置验证失败: {}", e);
                                    }
                                }
                            }

                            if ui.button("取消").clicked() {
                                close_dialog = true;
                            }

                            if ui.button("测试连接").clicked() {
                                action = Some(RemoteServerAction::TestConnection(server.id));
                            }
                        });
                    });
                });
        }

        if close_dialog {
            self.show_add_dialog = false;
            self.show_edit_dialog = false;
            self.editing_server = None;
            self.show_password = false;
        }

        action
    }

    /// 渲染导入/导出对话框
    fn render_import_export_dialog(&mut self, ctx: &egui::Context) {
        egui::Window::new("导入/导出配置")
            .collapsible(false)
            .resizable(true)
            .default_width(400.0)
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    ui.label("导出配置:");
                    ui.horizontal(|ui| {
                        if ui.button("导出（安全）").clicked() {
                            match self.manager.export_config(false) {
                                Ok(_config) => {
                                    // TODO: 保存到文件或复制到剪贴板
                                    log::info!("配置导出成功");
                                }
                                Err(e) => {
                                    log::error!("配置导出失败: {}", e);
                                }
                            }
                        }

                        if ui.button("导出（完整）").clicked() {
                            match self.manager.export_config(true) {
                                Ok(_config) => {
                                    // TODO: 保存到文件
                                    log::info!("完整配置导出成功");
                                }
                                Err(e) => {
                                    log::error!("完整配置导出失败: {}", e);
                                }
                            }
                        }
                    });

                    ui.separator();

                    ui.label("导入配置:");
                    if ui.button("从文件导入").clicked() {
                        // TODO: 实现文件选择和导入
                    }

                    ui.separator();

                    if ui.button("关闭").clicked() {
                        self.show_import_export_dialog = false;
                    }
                });
            });
    }

    /// 渲染SSH状态对话框
    fn render_ssh_status_dialog(&mut self, ctx: &egui::Context) {
        egui::Window::new("SSH支持状态")
            .resizable(true)
            .default_width(600.0)
            .default_height(400.0)
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    ui.label("当前系统SSH支持状态:");
                    ui.separator();

                    egui::ScrollArea::vertical()
                        .max_height(300.0)
                        .show(ui, |ui| {
                            let ssh_info = SshConnectionBuilder::get_ssh_support_info();
                            for line in ssh_info.lines() {
                                if line.starts_with("✅") {
                                    ui.colored_label(Color32::GREEN, line);
                                } else if line.starts_with("❌") {
                                    ui.colored_label(Color32::RED, line);
                                } else if line.starts_with("⚠️") {
                                    ui.colored_label(Color32::YELLOW, line);
                                } else {
                                    ui.label(line);
                                }
                            }
                        });

                    ui.separator();
                    if ui.button("关闭").clicked() {
                        self.show_ssh_status_dialog = false;
                    }
                });
            });
    }

    /// 渲染SSH替代方案对话框
    fn render_alternatives_dialog(&mut self, ctx: &egui::Context) {
        egui::Window::new("SSH连接替代方案")
            .resizable(true)
            .default_width(700.0)
            .default_height(500.0)
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    ui.label("可用的SSH连接替代方案:");
                    ui.separator();

                    egui::ScrollArea::vertical()
                        .max_height(400.0)
                        .show(ui, |ui| {
                            let full_report = crate::webssh::SshAlternativeManager::get_full_support_report();
                            for line in full_report.lines() {
                                if line.starts_with("✅") {
                                    ui.colored_label(Color32::GREEN, line);
                                } else if line.starts_with("❌") {
                                    ui.colored_label(Color32::RED, line);
                                } else if line.starts_with("⚠️") {
                                    ui.colored_label(Color32::YELLOW, line);
                                } else if line.starts_with("💡") {
                                    ui.colored_label(Color32::LIGHT_BLUE, line);
                                } else if line.starts_with("🔧") || line.starts_with("📡") || line.starts_with("🔄") {
                                    ui.label(RichText::new(line).strong());
                                } else if line.starts_with("=") {
                                    ui.separator();
                                } else if line.starts_with("•") {
                                    ui.label(RichText::new(line).strong());
                                } else if line.starts_with("  ") {
                                    ui.label(format!("    {}", line.trim()));
                                } else {
                                    ui.label(line);
                                }
                            }
                        });

                    ui.separator();
                    ui.horizontal(|ui| {
                        if ui.button("测试原生SSH").clicked() {
                            // 这里可以添加测试原生SSH的逻辑
                            log::info!("测试原生SSH连接");
                        }

                        if ui.button("关闭").clicked() {
                            self.show_alternatives_dialog = false;
                        }
                    });
                });
            });
    }
}

impl Default for RemoteServerUI {
    fn default() -> Self {
        Self::new().expect("Failed to create RemoteServerUI")
    }
}
