use crate::config::TerminalConfig;
use crate::egui_terminal::EguiTerminalManager;
use crate::export_ui::ExportDialog;
use crate::session_history::SessionHistoryManager;
use crate::session_history_ui::SessionHistoryUI;
use crate::help_ui::TerminalHelpUI;
use crate::remote_server_ui::{RemoteServerUI, RemoteServerAction};
use crate::ssh_connection::SshConnectionBuilder;
use egui_term::BackendSettings;
use eframe::egui;
use uuid::Uuid;

/// Main state for the iTerminal module
#[derive(Debug)]
pub struct ITerminalState {
    /// egui_term based terminal manager
    pub egui_terminal_manager: EguiTerminalManager,
    /// Terminal configuration
    pub config: TerminalConfig,
    /// Export dialog state
    pub export_dialog: ExportDialog,
    /// Session history manager
    pub session_history_manager: Option<SessionHistoryManager>,
    /// Session history UI state
    pub session_history_ui: SessionHistoryUI,
    /// Terminal help UI state
    pub help_ui: TerminalHelpUI,
    /// Remote server UI state
    pub remote_server_ui: Option<RemoteServerUI>,
    /// Whether to show remote servers panel
    pub show_remote_servers: bool,
    /// Whether to show compact remote servers panel (vs full management)
    pub show_compact_remote_panel: bool,
    /// Whether to show command history
    pub show_history: bool,
    /// Search query for history
    pub history_search: String,
    /// Whether the terminal has focus
    pub has_focus: bool,
    /// Whether cursor is visible (for blinking)
    pub cursor_visible: bool,
    /// Font size scale factor
    pub font_scale: f32,
    /// Last update time for cursor blinking
    pub last_cursor_blink: std::time::Instant,
    /// Terminal history for display
    pub terminal_history: Vec<String>,
    /// Current session name
    pub current_session: String,
}

impl ITerminalState {
    /// Create a new terminal state
    pub fn new() -> Self {
        let config = TerminalConfig::load();
        let egui_terminal_manager = EguiTerminalManager::new();

        // Initialize session history manager
        let session_history_manager = match SessionHistoryManager::new() {
            Ok(manager) => {
                log::info!("Session history manager initialized successfully");
                Some(manager)
            }
            Err(e) => {
                log::warn!("Failed to initialize session history manager: {}", e);
                None
            }
        };

        // 延迟初始化远程服务器UI，避免启动时访问密钥环
        let remote_server_ui = None;

        Self {
            egui_terminal_manager,
            config,
            export_dialog: ExportDialog::default(),
            session_history_manager,
            session_history_ui: SessionHistoryUI::default(),
            help_ui: TerminalHelpUI::default(),
            remote_server_ui,
            show_remote_servers: false,
            show_compact_remote_panel: true,
            show_history: false,
            history_search: String::new(),
            has_focus: false,
            cursor_visible: true,
            font_scale: 1.0,
            last_cursor_blink: std::time::Instant::now(),
            terminal_history: Vec::new(),
            current_session: "主会话".to_string(),
        }
    }

    /// Update the terminal state
    pub fn update(&mut self) {
        // Update egui terminal manager
        self.egui_terminal_manager.update();

        // Handle cursor blinking
        if self.has_focus && self.config.cursor_blink_interval > 0 {
            let elapsed = self.last_cursor_blink.elapsed();
            if elapsed.as_millis() >= self.config.cursor_blink_interval as u128 {
                self.cursor_visible = !self.cursor_visible;
                self.last_cursor_blink = std::time::Instant::now();
            }
        } else {
            self.cursor_visible = true;
        }
    }

    /// Create a new terminal session
    pub fn create_session(&mut self, title: Option<String>, ctx: Option<&egui::Context>) -> Result<Uuid, String> {
        if let Some(ctx) = ctx {
            self.egui_terminal_manager.create_session(title, ctx)
        } else {
            let error_msg = "Cannot create egui_term session without context";
            log::error!("{}", error_msg);
            Err(error_msg.to_string())
        }
    }

    /// Close a terminal session
    pub fn close_session(&mut self, session_id: Uuid) -> bool {
        self.egui_terminal_manager.close_session(session_id)
    }

    /// Set the active session
    pub fn set_active_session(&mut self, session_id: Uuid) -> bool {
        self.egui_terminal_manager.set_active_session(session_id)
    }

    /// Toggle history dialog
    pub fn toggle_history(&mut self) {
        self.show_history = !self.show_history;
    }

    /// Toggle remote servers panel
    pub fn toggle_remote_servers(&mut self) {
        self.show_remote_servers = !self.show_remote_servers;
    }

    /// Create SSH session from remote server
    pub fn create_ssh_session(&mut self, server_id: Uuid, ctx: &egui::Context) -> Result<Uuid, String> {
        log::info!("开始创建SSH会话，服务器ID: {}", server_id);

        // First, get the server info and build the command
        let (shell, args, title) = if let Some(ref remote_ui) = self.remote_server_ui {
            if let Some(server) = remote_ui.manager.get_server(server_id) {
                log::info!("找到服务器配置: {}@{}:{}", server.username, server.host, server.port);
                let (shell, args) = SshConnectionBuilder::build_ssh_command(server)?;
                log::info!("构建SSH命令: {} {}", shell, args.join(" "));
                let title = format!("SSH: {}", server.get_display_name());
                (shell, args, title)
            } else {
                log::error!("服务器配置不存在，ID: {}", server_id);
                return Err("服务器配置不存在".to_string());
            }
        } else {
            log::error!("远程服务器管理器未初始化");
            return Err("远程服务器管理器未初始化".to_string());
        };

        // Create the session
        let backend_settings = BackendSettings {
            shell: shell.clone(),
            args: args.clone(),
            working_directory: None,
            ssh_config: None, // SSH config is embedded in the command
            env_vars: std::collections::HashMap::new(),
        };

        log::info!("创建终端会话，shell: {}, args: {:?}", shell, args);
        let session_id = self.create_session_with_settings(Some(title), ctx, backend_settings)?;
        log::info!("SSH会话创建成功，会话ID: {}", session_id);

        // Update connection statistics
        if let Some(ref mut remote_ui) = self.remote_server_ui {
            if let Err(e) = remote_ui.manager.update_connection_stats(server_id) {
                log::warn!("Failed to update connection stats: {}", e);
            }
        }

        Ok(session_id)
    }

    /// Create session with custom backend settings
    pub fn create_session_with_settings(
        &mut self,
        title: Option<String>,
        ctx: &egui::Context,
        settings: BackendSettings
    ) -> Result<Uuid, String> {
        log::info!("创建带有自定义设置的终端会话: shell={}, args={:?}", settings.shell, settings.args);

        // Create session with SSH command and arguments
        self.egui_terminal_manager.create_session_with_command(
            title,
            ctx,
            &settings.shell,
            &settings.args
        )
    }

    /// Handle remote server actions
    pub fn handle_remote_server_action(&mut self, action: RemoteServerAction, ctx: &egui::Context) -> Result<(), String> {
        match action {
            RemoteServerAction::Connect(server_id) => {
                match self.create_ssh_session(server_id, ctx) {
                    Ok(session_id) => {
                        self.set_active_session(session_id);
                        log::info!("SSH session created: {}", session_id);
                        Ok(())
                    }
                    Err(e) => {
                        log::error!("Failed to create SSH session: {}", e);
                        Err(e)
                    }
                }
            }
            RemoteServerAction::Edit(server_id) => {
                if let Some(ref mut remote_ui) = self.remote_server_ui {
                    remote_ui.show_edit_dialog(server_id);
                }
                Ok(())
            }
            RemoteServerAction::Delete(server_id) => {
                if let Some(ref mut remote_ui) = self.remote_server_ui {
                    remote_ui.delete_server(server_id)?;
                }
                Ok(())
            }
            RemoteServerAction::TestConnection(server_id) => {
                log::info!("处理连接测试动作，服务器ID: {}", server_id);
                if let Some(ref mut remote_ui) = self.remote_server_ui {
                    remote_ui.start_connection_test(server_id);
                } else {
                    log::error!("远程服务器UI未初始化，无法执行连接测试");
                }
                Ok(())
            }
            RemoteServerAction::ToggleEnabled(server_id) => {
                if let Some(ref mut remote_ui) = self.remote_server_ui {
                    remote_ui.toggle_server_enabled(server_id)?;
                }
                Ok(())
            }
            RemoteServerAction::Import => {
                // TODO: Implement import functionality
                log::info!("Import action triggered");
                Ok(())
            }
            RemoteServerAction::Export => {
                // TODO: Implement export functionality
                log::info!("Export action triggered");
                Ok(())
            }
        }
    }

    /// Get remote server UI reference
    pub fn get_remote_server_ui(&self) -> Option<&RemoteServerUI> {
        self.remote_server_ui.as_ref()
    }

    /// Get mutable remote server UI reference
    pub fn get_remote_server_ui_mut(&mut self) -> Option<&mut RemoteServerUI> {
        self.remote_server_ui.as_mut()
    }

    /// Check if remote servers are available
    pub fn has_remote_servers(&self) -> bool {
        self.remote_server_ui.is_some()
    }

    /// 按需初始化远程服务器UI（延迟初始化）
    pub fn ensure_remote_server_ui(&mut self) -> bool {
        if self.remote_server_ui.is_none() {
            log::info!("Initializing remote server UI on demand");
            match RemoteServerUI::new() {
                Ok(ui) => {
                    log::info!("Remote server UI initialized successfully");
                    self.remote_server_ui = Some(ui);
                    true
                }
                Err(e) => {
                    log::error!("Failed to initialize remote server UI: {}", e);
                    false
                }
            }
        } else {
            true
        }
    }

    /// 获取远程服务器UI，如果未初始化则尝试初始化
    pub fn get_remote_server_ui_lazy(&mut self) -> Option<&RemoteServerUI> {
        if self.ensure_remote_server_ui() {
            self.remote_server_ui.as_ref()
        } else {
            None
        }
    }

    /// 获取可变远程服务器UI，如果未初始化则尝试初始化
    pub fn get_remote_server_ui_mut_lazy(&mut self) -> Option<&mut RemoteServerUI> {
        if self.ensure_remote_server_ui() {
            self.remote_server_ui.as_mut()
        } else {
            None
        }
    }

    /// Save current configuration
    pub fn save_config(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.config.save()
    }

    /// Update configuration
    pub fn update_config(&mut self, config: TerminalConfig) {
        self.config = config;
    }

    /// Get current configuration
    pub fn get_config(&self) -> &TerminalConfig {
        &self.config
    }

    /// Increase font size
    pub fn increase_font_size(&mut self) {
        self.font_scale = (self.font_scale + 0.1).min(3.0);
    }

    /// Decrease font size
    pub fn decrease_font_size(&mut self) {
        self.font_scale = (self.font_scale - 0.1).max(0.5);
    }

    /// Reset font size
    pub fn reset_font_size(&mut self) {
        self.font_scale = 1.0;
    }

    /// Get the number of sessions
    pub fn session_count(&self) -> usize {
        self.egui_terminal_manager.session_count()
    }

    /// Get the egui terminal manager
    pub fn get_egui_terminal_manager(&self) -> &EguiTerminalManager {
        &self.egui_terminal_manager
    }

    /// Get the egui terminal manager (mutable)
    pub fn get_egui_terminal_manager_mut(&mut self) -> &mut EguiTerminalManager {
        &mut self.egui_terminal_manager
    }
}

impl Default for ITerminalState {
    fn default() -> Self {
        Self::new()
    }
}
