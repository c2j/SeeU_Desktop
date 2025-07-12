use eframe::egui;
use iterminal::state::ITerminalState;
use iterminal::remote_server::{AuthMethod, RemoteServer};

fn main() -> Result<(), eframe::Error> {
    env_logger::init();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1200.0, 800.0]),
        ..Default::default()
    };

    eframe::run_native(
        "iTerminal UI Demo",
        options,
        Box::new(|_cc| Ok(Box::new(TerminalApp::new()))),
    )
}

struct TerminalApp {
    terminal_state: ITerminalState,
}

impl TerminalApp {
    fn new() -> Self {
        let mut terminal_state = ITerminalState::new();

        // 延迟初始化并添加一些测试服务器
        if let Some(remote_ui) = terminal_state.get_remote_server_ui_mut_lazy() {
            // 测试服务器1
            let server1 = RemoteServer::new(
                "开发服务器".to_string(),
                "dev.example.com".to_string(),
                "developer".to_string(),
                AuthMethod::Agent,
            );
            
            // 测试服务器2
            let mut server2 = RemoteServer::new(
                "生产服务器".to_string(),
                "prod.example.com".to_string(),
                "admin".to_string(),
                AuthMethod::PrivateKey {
                    key_path: std::path::PathBuf::from("/Users/user/.ssh/id_rsa"),
                    passphrase: None,
                },
            );
            server2.description = Some("主要生产环境服务器".to_string());
            server2.tags = vec!["生产".to_string(), "Web".to_string()];
            
            // 测试服务器3
            let mut server3 = RemoteServer::new(
                "测试服务器".to_string(),
                "192.168.1.100".to_string(),
                "testuser".to_string(),
                AuthMethod::Password("password123".to_string()),
            );
            server3.port = 2222;
            server3.enabled = false; // 禁用状态
            server3.tags = vec!["测试".to_string(), "本地".to_string()];
            
            // 添加服务器
            if let Err(e) = remote_ui.manager.add_server(server1) {
                println!("Failed to add server1: {}", e);
            }
            if let Err(e) = remote_ui.manager.add_server(server2) {
                println!("Failed to add server2: {}", e);
            }
            if let Err(e) = remote_ui.manager.add_server(server3) {
                println!("Failed to add server3: {}", e);
            }
            
            println!("Added {} test servers", remote_ui.manager.list_servers().len());
        }
        
        Self { terminal_state }
    }
}

impl eframe::App for TerminalApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("iTerminal 远程服务器管理功能演示");
            ui.separator();
            
            ui.horizontal(|ui| {
                ui.label("功能演示:");
                if ui.button("显示/隐藏远程服务器面板").clicked() {
                    self.terminal_state.show_remote_servers = !self.terminal_state.show_remote_servers;
                    self.terminal_state.show_compact_remote_panel = true;
                }
                
                if ui.button("切换紧凑/完整视图").clicked() {
                    self.terminal_state.show_compact_remote_panel = !self.terminal_state.show_compact_remote_panel;
                }
                
                if ui.button("添加测试服务器").clicked() {
                    if let Some(remote_ui) = self.terminal_state.get_remote_server_ui_mut_lazy() {
                        remote_ui.show_add_dialog();
                    }
                }
            });
            
            ui.separator();
            
            // 状态信息
            ui.horizontal(|ui| {
                ui.label(format!("远程服务器面板: {}", 
                    if self.terminal_state.show_remote_servers { "显示" } else { "隐藏" }));
                ui.label(format!("视图模式: {}", 
                    if self.terminal_state.show_compact_remote_panel { "紧凑" } else { "完整" }));
                
                if let Some(remote_ui) = self.terminal_state.get_remote_server_ui() {
                    ui.label(format!("服务器数量: {}", remote_ui.manager.list_servers().len()));
                }
            });
            
            ui.separator();
            
            // 渲染终端界面（包含远程服务器管理）
            iterminal::render_iterminal(ui, &mut self.terminal_state);
        });
    }
}
