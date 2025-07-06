use eframe::egui;
use std::collections::VecDeque;
use std::process::{Command, Stdio};
use std::io::{BufRead, BufReader};
use std::sync::{Arc, Mutex};
use std::thread;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([800.0, 600.0]),
        ..Default::default()
    };
    
    eframe::run_native(
        "Simple Terminal Example",
        options,
        Box::new(|_cc| Box::new(SimpleTerminalApp::default())),
    )
}

#[derive(Default)]
struct SimpleTerminalApp {
    terminal: SimpleTerminal,
}

impl eframe::App for SimpleTerminalApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Simple Terminal Example");
            ui.separator();
            
            self.terminal.ui(ui);
        });
    }
}

struct SimpleTerminal {
    output: Arc<Mutex<VecDeque<String>>>,
    input: String,
    command_history: Vec<String>,
    history_index: Option<usize>,
}

impl Default for SimpleTerminal {
    fn default() -> Self {
        Self {
            output: Arc::new(Mutex::new(VecDeque::new())),
            input: String::new(),
            command_history: Vec::new(),
            history_index: None,
        }
    }
}

impl SimpleTerminal {
    fn ui(&mut self, ui: &mut egui::Ui) {
        // 输出区域
        egui::ScrollArea::vertical()
            .max_height(400.0)
            .stick_to_bottom(true)
            .show(ui, |ui| {
                let output = self.output.lock().unwrap();
                for line in output.iter() {
                    ui.label(egui::RichText::new(line).font(egui::FontId::monospace(12.0)));
                }
            });
        
        ui.separator();
        
        // 输入区域
        ui.horizontal(|ui| {
            ui.label("$ ");
            
            let response = ui.text_edit_singleline(&mut self.input);
            
            if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                self.execute_command();
            }
            
            if ui.button("Execute").clicked() {
                self.execute_command();
            }
            
            if ui.button("Clear").clicked() {
                self.output.lock().unwrap().clear();
            }
        });
        
        // 处理历史记录导航
        if ui.input(|i| i.key_pressed(egui::Key::ArrowUp)) {
            self.navigate_history_up();
        }
        if ui.input(|i| i.key_pressed(egui::Key::ArrowDown)) {
            self.navigate_history_down();
        }
    }
    
    fn execute_command(&mut self) {
        if self.input.trim().is_empty() {
            return;
        }
        
        let command = self.input.trim().to_string();
        
        // 添加到历史记录
        self.command_history.push(command.clone());
        self.history_index = None;
        
        // 显示命令
        {
            let mut output = self.output.lock().unwrap();
            output.push_back(format!("$ {}", command));
            
            // 限制输出行数
            while output.len() > 1000 {
                output.pop_front();
            }
        }
        
        // 执行命令
        let output_clone = Arc::clone(&self.output);
        let cmd = command.clone();
        
        thread::spawn(move || {
            Self::run_command(&cmd, output_clone);
        });
        
        self.input.clear();
    }
    
    fn run_command(command: &str, output: Arc<Mutex<VecDeque<String>>>) {
        let parts: Vec<&str> = command.split_whitespace().collect();
        if parts.is_empty() {
            return;
        }
        
        let mut cmd = Command::new(parts[0]);
        if parts.len() > 1 {
            cmd.args(&parts[1..]);
        }
        
        match cmd.stdout(Stdio::piped()).stderr(Stdio::piped()).spawn() {
            Ok(mut child) => {
                if let Some(stdout) = child.stdout.take() {
                    let reader = BufReader::new(stdout);
                    for line in reader.lines() {
                        match line {
                            Ok(line) => {
                                let mut output = output.lock().unwrap();
                                output.push_back(line);
                                while output.len() > 1000 {
                                    output.pop_front();
                                }
                            }
                            Err(_) => break,
                        }
                    }
                }
                
                if let Some(stderr) = child.stderr.take() {
                    let reader = BufReader::new(stderr);
                    for line in reader.lines() {
                        match line {
                            Ok(line) => {
                                let mut output = output.lock().unwrap();
                                output.push_back(format!("ERROR: {}", line));
                                while output.len() > 1000 {
                                    output.pop_front();
                                }
                            }
                            Err(_) => break,
                        }
                    }
                }
                
                match child.wait() {
                    Ok(status) => {
                        let mut output = output.lock().unwrap();
                        if !status.success() {
                            output.push_back(format!("Command exited with code: {:?}", status.code()));
                        }
                    }
                    Err(e) => {
                        let mut output = output.lock().unwrap();
                        output.push_back(format!("Failed to wait for command: {}", e));
                    }
                }
            }
            Err(e) => {
                let mut output = output.lock().unwrap();
                output.push_back(format!("Failed to execute command: {}", e));
            }
        }
    }
    
    fn navigate_history_up(&mut self) {
        if self.command_history.is_empty() {
            return;
        }
        
        match self.history_index {
            None => {
                self.history_index = Some(self.command_history.len() - 1);
                self.input = self.command_history[self.history_index.unwrap()].clone();
            }
            Some(index) => {
                if index > 0 {
                    self.history_index = Some(index - 1);
                    self.input = self.command_history[self.history_index.unwrap()].clone();
                }
            }
        }
    }
    
    fn navigate_history_down(&mut self) {
        if let Some(index) = self.history_index {
            if index < self.command_history.len() - 1 {
                self.history_index = Some(index + 1);
                self.input = self.command_history[self.history_index.unwrap()].clone();
            } else {
                self.history_index = None;
                self.input.clear();
            }
        }
    }
}
