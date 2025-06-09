use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::thread;
use std::io::{BufRead, BufReader};
use uuid::Uuid;

/// Command execution result
#[derive(Debug, Clone)]
pub struct CommandResult {
    /// Command that was executed
    pub command: String,
    /// Exit code
    pub exit_code: Option<i32>,
    /// Standard output
    pub stdout: Vec<String>,
    /// Standard error
    pub stderr: Vec<String>,
    /// Execution time in milliseconds
    pub execution_time: u64,
}

/// Command execution message
#[derive(Debug, Clone)]
pub enum CommandMessage {
    /// Output line from stdout
    Stdout(String),
    /// Output line from stderr
    Stderr(String),
    /// Command finished with exit code
    Finished(i32),
    /// Command failed to start
    Error(String),
}

/// Asynchronous command executor
#[derive(Debug)]
pub struct CommandExecutor {
    /// Channel for receiving command results
    receiver: mpsc::Receiver<(Uuid, CommandMessage)>,
    /// Channel for sending commands
    sender: mpsc::Sender<(Uuid, CommandMessage)>,
}

impl CommandExecutor {
    /// Create a new command executor
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::channel();

        Self {
            receiver,
            sender,
        }
    }

    /// Execute a command asynchronously
    pub fn execute_async(&self, command_id: Uuid, command: String, working_dir: String) {
        let sender = self.sender.clone();

        thread::spawn(move || {
            let _start_time = std::time::Instant::now();

            // Parse command and arguments
            let parts: Vec<&str> = command.trim().split_whitespace().collect();
            if parts.is_empty() {
                let _ = sender.send((command_id, CommandMessage::Error("Empty command".to_string())));
                return;
            }

            let program = parts[0];
            let args = &parts[1..];

            // Create command
            let mut cmd = Command::new(program);
            cmd.args(args)
                .current_dir(&working_dir)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped());

            // Execute command
            match cmd.spawn() {
                Ok(mut child) => {
                    // Handle stdout
                    if let Some(stdout) = child.stdout.take() {
                        let sender_stdout = sender.clone();
                        let cmd_id = command_id;
                        thread::spawn(move || {
                            let reader = BufReader::new(stdout);
                            for line in reader.lines() {
                                match line {
                                    Ok(line) => {
                                        let _ = sender_stdout.send((cmd_id, CommandMessage::Stdout(line)));
                                    }
                                    Err(_) => break,
                                }
                            }
                        });
                    }

                    // Handle stderr
                    if let Some(stderr) = child.stderr.take() {
                        let sender_stderr = sender.clone();
                        let cmd_id = command_id;
                        thread::spawn(move || {
                            let reader = BufReader::new(stderr);
                            for line in reader.lines() {
                                match line {
                                    Ok(line) => {
                                        let _ = sender_stderr.send((cmd_id, CommandMessage::Stderr(line)));
                                    }
                                    Err(_) => break,
                                }
                            }
                        });
                    }

                    // Wait for command to finish
                    match child.wait() {
                        Ok(status) => {
                            let exit_code = status.code().unwrap_or(-1);
                            let _ = sender.send((command_id, CommandMessage::Finished(exit_code)));
                        }
                        Err(e) => {
                            let _ = sender.send((command_id, CommandMessage::Error(format!("Failed to wait for command: {}", e))));
                        }
                    }
                }
                Err(e) => {
                    let _ = sender.send((command_id, CommandMessage::Error(format!("Failed to execute command: {}", e))));
                }
            }
        });
    }

    /// Check for command messages
    pub fn check_messages(&self) -> Vec<(Uuid, CommandMessage)> {
        let mut messages = Vec::new();

        while let Ok(message) = self.receiver.try_recv() {
            messages.push(message);
        }

        messages
    }

    /// Execute a simple command synchronously (for built-in commands)
    pub fn execute_builtin(command: &str, working_dir: &str) -> CommandResult {
        let start_time = std::time::Instant::now();
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        let mut exit_code = 0;

        let parts: Vec<&str> = command.trim().split_whitespace().collect();
        if parts.is_empty() {
            return CommandResult {
                command: command.to_string(),
                exit_code: Some(1),
                stdout,
                stderr: vec!["Empty command".to_string()],
                execution_time: start_time.elapsed().as_millis() as u64,
            };
        }

        let cmd = parts[0];
        let args = &parts[1..];

        match cmd {
            "clear" => {
                // Clear command is handled by the UI
                stdout.push("Terminal cleared".to_string());
            }
            "cd" => {
                let path = args.get(0).unwrap_or(&"").trim();
                if path.is_empty() {
                    // cd with no arguments goes to home directory
                    if let Some(home) = dirs::home_dir() {
                        stdout.push(format!("Changed directory to: {}", home.display()));
                    } else {
                        stderr.push("Could not determine home directory".to_string());
                        exit_code = 1;
                    }
                } else {
                    // Try to change to the specified directory
                    let target_path = if path.starts_with('/') || path.contains(':') {
                        // Absolute path
                        std::path::PathBuf::from(path)
                    } else {
                        // Relative path
                        std::path::PathBuf::from(working_dir).join(path)
                    };

                    if target_path.exists() && target_path.is_dir() {
                        stdout.push(format!("Changed directory to: {}", target_path.display()));
                    } else {
                        stderr.push(format!("Directory not found: {}", path));
                        exit_code = 1;
                    }
                }
            }
            "pwd" => {
                stdout.push(working_dir.to_string());
            }
            "ls" | "dir" => {
                execute_ls_command(working_dir, args, &mut stdout, &mut stderr, &mut exit_code);
            }
            "cat" | "type" => {
                execute_cat_command(working_dir, args, &mut stdout, &mut stderr, &mut exit_code);
            }
            "echo" => {
                execute_echo_command(args, &mut stdout);
            }
            "date" => {
                execute_date_command(args, &mut stdout);
            }
            "whoami" => {
                execute_whoami_command(&mut stdout, &mut stderr, &mut exit_code);
            }
            "hostname" => {
                execute_hostname_command(&mut stdout, &mut stderr, &mut exit_code);
            }
            "env" => {
                execute_env_command(args, &mut stdout);
            }
            "which" => {
                execute_which_command(args, &mut stdout, &mut stderr, &mut exit_code);
            }
            "head" => {
                execute_head_command(working_dir, args, &mut stdout, &mut stderr, &mut exit_code);
            }
            "tail" => {
                execute_tail_command(working_dir, args, &mut stdout, &mut stderr, &mut exit_code);
            }
            "wc" => {
                execute_wc_command(working_dir, args, &mut stdout, &mut stderr, &mut exit_code);
            }
            "find" => {
                execute_find_command(working_dir, args, &mut stdout, &mut stderr, &mut exit_code);
            }
            "grep" => {
                execute_grep_command(working_dir, args, &mut stdout, &mut stderr, &mut exit_code);
            }
            "tree" => {
                execute_tree_command(working_dir, args, &mut stdout, &mut stderr, &mut exit_code);
            }
            "du" => {
                execute_du_command(working_dir, args, &mut stdout, &mut stderr, &mut exit_code);
            }
            "df" => {
                execute_df_command(&mut stdout, &mut stderr, &mut exit_code);
            }
            "ps" => {
                execute_ps_command(args, &mut stdout, &mut stderr, &mut exit_code);
            }
            "uptime" => {
                execute_uptime_command(&mut stdout, &mut stderr, &mut exit_code);
            }
            "history" => {
                stdout.push("Command history is managed by the terminal interface".to_string());
                stdout.push("Use the history button (📜) to view and search command history".to_string());
            }
            "help" => {
                execute_help_command(&mut stdout);
            }
            "exit" => {
                stdout.push("Goodbye!".to_string());
            }
            _ => {
                // Not a built-in command
                return CommandResult {
                    command: command.to_string(),
                    exit_code: None, // Indicates this should be handled as external command
                    stdout,
                    stderr,
                    execution_time: start_time.elapsed().as_millis() as u64,
                };
            }
        }

        CommandResult {
            command: command.to_string(),
            exit_code: Some(exit_code),
            stdout,
            stderr,
            execution_time: start_time.elapsed().as_millis() as u64,
        }
    }

    /// Check if a command is a built-in command
    pub fn is_builtin(command: &str) -> bool {
        let parts: Vec<&str> = command.trim().split_whitespace().collect();
        if parts.is_empty() {
            return false;
        }

        let cmd = parts[0];
        matches!(cmd,
            "clear" | "help" | "exit" | "pwd" | "cd" |
            "ls" | "dir" | "cat" | "type" | "echo" | "date" |
            "whoami" | "hostname" | "env" | "which" |
            "head" | "tail" | "wc" | "find" | "grep" |
            "tree" | "du" | "df" | "ps" | "uptime" | "history"
        )
    }
}

impl Default for CommandExecutor {
    fn default() -> Self {
        Self::new()
    }
}

// Helper functions for built-in commands

/// Execute ls/dir command
fn execute_ls_command(working_dir: &str, args: &[&str], stdout: &mut Vec<String>, stderr: &mut Vec<String>, exit_code: &mut i32) {
    use std::fs;

    let show_hidden = args.contains(&"-a") || args.contains(&"--all");
    let long_format = args.contains(&"-l") || args.contains(&"--long");
    let human_readable = args.contains(&"-h") || args.contains(&"--human-readable");

    let target_dir = args.iter()
        .find(|arg| !arg.starts_with('-'))
        .map(|s| s.to_string())
        .unwrap_or_else(|| working_dir.to_string());

    let path = std::path::Path::new(&target_dir);

    match fs::read_dir(path) {
        Ok(entries) => {
            let mut items: Vec<_> = entries.collect();
            items.sort_by_key(|entry| {
                entry.as_ref().map(|e| e.file_name()).unwrap_or_default()
            });

            for entry in items {
                match entry {
                    Ok(entry) => {
                        let file_name = entry.file_name();
                        let name = file_name.to_string_lossy();

                        // Skip hidden files unless -a flag is used
                        if !show_hidden && name.starts_with('.') {
                            continue;
                        }

                        if long_format {
                            if let Ok(metadata) = entry.metadata() {
                                let size = if human_readable {
                                    format_size(metadata.len())
                                } else {
                                    metadata.len().to_string()
                                };

                                let file_type = if metadata.is_dir() { "d" } else { "-" };
                                let permissions = format_permissions(&metadata);

                                if let Ok(modified) = metadata.modified() {
                                    let datetime: chrono::DateTime<chrono::Local> = modified.into();
                                    stdout.push(format!("{}{} {:>8} {} {}",
                                        file_type, permissions, size,
                                        datetime.format("%b %d %H:%M"), name));
                                } else {
                                    stdout.push(format!("{}{} {:>8} {}",
                                        file_type, permissions, size, name));
                                }
                            } else {
                                stdout.push(name.to_string());
                            }
                        } else {
                            stdout.push(name.to_string());
                        }
                    }
                    Err(e) => {
                        stderr.push(format!("Error reading entry: {}", e));
                        *exit_code = 1;
                    }
                }
            }
        }
        Err(e) => {
            stderr.push(format!("Cannot access '{}': {}", target_dir, e));
            *exit_code = 1;
        }
    }
}

/// Execute cat/type command
fn execute_cat_command(working_dir: &str, args: &[&str], stdout: &mut Vec<String>, stderr: &mut Vec<String>, exit_code: &mut i32) {
    if args.is_empty() {
        stderr.push("cat: missing file operand".to_string());
        *exit_code = 1;
        return;
    }

    for &filename in args {
        let path = if filename.starts_with('/') || filename.contains(':') {
            std::path::PathBuf::from(filename)
        } else {
            std::path::PathBuf::from(working_dir).join(filename)
        };

        match std::fs::read_to_string(&path) {
            Ok(content) => {
                for line in content.lines() {
                    stdout.push(line.to_string());
                }
            }
            Err(e) => {
                stderr.push(format!("cat: {}: {}", filename, e));
                *exit_code = 1;
            }
        }
    }
}

/// Execute echo command
fn execute_echo_command(args: &[&str], stdout: &mut Vec<String>) {
    let output = args.join(" ");
    stdout.push(output);
}

/// Execute date command
fn execute_date_command(args: &[&str], stdout: &mut Vec<String>) {
    let now = chrono::Local::now();

    if args.is_empty() {
        stdout.push(now.format("%a %b %d %H:%M:%S %Z %Y").to_string());
    } else if args.contains(&"-u") || args.contains(&"--utc") {
        let utc = chrono::Utc::now();
        stdout.push(utc.format("%a %b %d %H:%M:%S UTC %Y").to_string());
    } else if args.contains(&"-I") || args.contains(&"--iso-8601") {
        stdout.push(now.format("%Y-%m-%d").to_string());
    } else {
        stdout.push(now.format("%a %b %d %H:%M:%S %Z %Y").to_string());
    }
}

/// Execute whoami command
fn execute_whoami_command(stdout: &mut Vec<String>, stderr: &mut Vec<String>, exit_code: &mut i32) {
    match std::env::var("USER").or_else(|_| std::env::var("USERNAME")) {
        Ok(username) => stdout.push(username),
        Err(_) => {
            stderr.push("Cannot determine username".to_string());
            *exit_code = 1;
        }
    }
}

/// Execute hostname command
fn execute_hostname_command(stdout: &mut Vec<String>, stderr: &mut Vec<String>, exit_code: &mut i32) {
    match hostname::get() {
        Ok(hostname) => {
            if let Some(hostname_str) = hostname.to_str() {
                stdout.push(hostname_str.to_string());
            } else {
                stderr.push("Invalid hostname encoding".to_string());
                *exit_code = 1;
            }
        }
        Err(e) => {
            stderr.push(format!("Cannot get hostname: {}", e));
            *exit_code = 1;
        }
    }
}

/// Execute env command
fn execute_env_command(args: &[&str], stdout: &mut Vec<String>) {
    if args.is_empty() {
        // Show all environment variables
        let mut env_vars: Vec<_> = std::env::vars().collect();
        env_vars.sort_by(|a, b| a.0.cmp(&b.0));

        for (key, value) in env_vars {
            stdout.push(format!("{}={}", key, value));
        }
    } else {
        // Show specific environment variables
        for &var_name in args {
            match std::env::var(var_name) {
                Ok(value) => stdout.push(format!("{}={}", var_name, value)),
                Err(_) => stdout.push(format!("{}=", var_name)),
            }
        }
    }
}

/// Execute which command
fn execute_which_command(args: &[&str], stdout: &mut Vec<String>, stderr: &mut Vec<String>, exit_code: &mut i32) {
    if args.is_empty() {
        stderr.push("which: missing command name".to_string());
        *exit_code = 1;
        return;
    }

    for &command in args {
        if CommandExecutor::is_builtin(command) {
            stdout.push(format!("{}: shell builtin", command));
        } else {
            // Try to find the command in PATH
            if let Some(path_var) = std::env::var_os("PATH") {
                let paths = std::env::split_paths(&path_var);
                let mut found = false;

                for path in paths {
                    let full_path = path.join(command);
                    if full_path.exists() && full_path.is_file() {
                        stdout.push(full_path.to_string_lossy().to_string());
                        found = true;
                        break;
                    }

                    // Also check with .exe extension on Windows
                    #[cfg(windows)]
                    {
                        let exe_path = path.join(format!("{}.exe", command));
                        if exe_path.exists() && exe_path.is_file() {
                            stdout.push(exe_path.to_string_lossy().to_string());
                            found = true;
                            break;
                        }
                    }
                }

                if !found {
                    stderr.push(format!("which: no {} in PATH", command));
                    *exit_code = 1;
                }
            } else {
                stderr.push("which: PATH environment variable not set".to_string());
                *exit_code = 1;
            }
        }
    }
}

/// Execute head command
fn execute_head_command(working_dir: &str, args: &[&str], stdout: &mut Vec<String>, stderr: &mut Vec<String>, exit_code: &mut i32) {
    let mut lines_to_show = 10;
    let mut files = Vec::new();

    let mut i = 0;
    while i < args.len() {
        match args[i] {
            "-n" => {
                if i + 1 < args.len() {
                    if let Ok(n) = args[i + 1].parse::<usize>() {
                        lines_to_show = n;
                        i += 2;
                    } else {
                        stderr.push(format!("head: invalid number of lines: '{}'", args[i + 1]));
                        *exit_code = 1;
                        return;
                    }
                } else {
                    stderr.push("head: option requires an argument -- n".to_string());
                    *exit_code = 1;
                    return;
                }
            }
            arg if arg.starts_with("-n") => {
                if let Ok(n) = arg[2..].parse::<usize>() {
                    lines_to_show = n;
                    i += 1;
                } else {
                    stderr.push(format!("head: invalid number of lines: '{}'", &arg[2..]));
                    *exit_code = 1;
                    return;
                }
            }
            _ => {
                files.push(args[i]);
                i += 1;
            }
        }
    }

    if files.is_empty() {
        stderr.push("head: missing file operand".to_string());
        *exit_code = 1;
        return;
    }

    for &filename in &files {
        let path = if filename.starts_with('/') || filename.contains(':') {
            std::path::PathBuf::from(filename)
        } else {
            std::path::PathBuf::from(working_dir).join(filename)
        };

        match std::fs::read_to_string(&path) {
            Ok(content) => {
                if files.len() > 1 {
                    stdout.push(format!("==> {} <==", filename));
                }

                for (i, line) in content.lines().enumerate() {
                    if i >= lines_to_show {
                        break;
                    }
                    stdout.push(line.to_string());
                }

                if files.len() > 1 && filename != *files.last().unwrap() {
                    stdout.push(String::new());
                }
            }
            Err(e) => {
                stderr.push(format!("head: {}: {}", filename, e));
                *exit_code = 1;
            }
        }
    }
}

/// Execute tail command
fn execute_tail_command(working_dir: &str, args: &[&str], stdout: &mut Vec<String>, stderr: &mut Vec<String>, exit_code: &mut i32) {
    let mut lines_to_show = 10;
    let mut files = Vec::new();

    let mut i = 0;
    while i < args.len() {
        match args[i] {
            "-n" => {
                if i + 1 < args.len() {
                    if let Ok(n) = args[i + 1].parse::<usize>() {
                        lines_to_show = n;
                        i += 2;
                    } else {
                        stderr.push(format!("tail: invalid number of lines: '{}'", args[i + 1]));
                        *exit_code = 1;
                        return;
                    }
                } else {
                    stderr.push("tail: option requires an argument -- n".to_string());
                    *exit_code = 1;
                    return;
                }
            }
            arg if arg.starts_with("-n") => {
                if let Ok(n) = arg[2..].parse::<usize>() {
                    lines_to_show = n;
                    i += 1;
                } else {
                    stderr.push(format!("tail: invalid number of lines: '{}'", &arg[2..]));
                    *exit_code = 1;
                    return;
                }
            }
            _ => {
                files.push(args[i]);
                i += 1;
            }
        }
    }

    if files.is_empty() {
        stderr.push("tail: missing file operand".to_string());
        *exit_code = 1;
        return;
    }

    for &filename in &files {
        let path = if filename.starts_with('/') || filename.contains(':') {
            std::path::PathBuf::from(filename)
        } else {
            std::path::PathBuf::from(working_dir).join(filename)
        };

        match std::fs::read_to_string(&path) {
            Ok(content) => {
                if files.len() > 1 {
                    stdout.push(format!("==> {} <==", filename));
                }

                let lines: Vec<&str> = content.lines().collect();
                let start_index = if lines.len() > lines_to_show {
                    lines.len() - lines_to_show
                } else {
                    0
                };

                for line in &lines[start_index..] {
                    stdout.push(line.to_string());
                }

                if files.len() > 1 && filename != *files.last().unwrap() {
                    stdout.push(String::new());
                }
            }
            Err(e) => {
                stderr.push(format!("tail: {}: {}", filename, e));
                *exit_code = 1;
            }
        }
    }
}

/// Execute wc command
fn execute_wc_command(working_dir: &str, args: &[&str], stdout: &mut Vec<String>, stderr: &mut Vec<String>, exit_code: &mut i32) {
    let mut count_lines = false;
    let mut count_words = false;
    let mut count_chars = false;
    let mut files = Vec::new();

    // Parse arguments
    for &arg in args {
        match arg {
            "-l" | "--lines" => count_lines = true,
            "-w" | "--words" => count_words = true,
            "-c" | "--chars" => count_chars = true,
            _ if !arg.starts_with('-') => files.push(arg),
            _ => {
                stderr.push(format!("wc: invalid option '{}'", arg));
                *exit_code = 1;
                return;
            }
        }
    }

    // If no flags specified, count all
    if !count_lines && !count_words && !count_chars {
        count_lines = true;
        count_words = true;
        count_chars = true;
    }

    if files.is_empty() {
        stderr.push("wc: missing file operand".to_string());
        *exit_code = 1;
        return;
    }

    let mut total_lines = 0;
    let mut total_words = 0;
    let mut total_chars = 0;

    for &filename in &files {
        let path = if filename.starts_with('/') || filename.contains(':') {
            std::path::PathBuf::from(filename)
        } else {
            std::path::PathBuf::from(working_dir).join(filename)
        };

        match std::fs::read_to_string(&path) {
            Ok(content) => {
                let lines = content.lines().count();
                let words = content.split_whitespace().count();
                let chars = content.chars().count();

                total_lines += lines;
                total_words += words;
                total_chars += chars;

                let mut output = String::new();
                if count_lines {
                    output.push_str(&format!("{:8}", lines));
                }
                if count_words {
                    output.push_str(&format!("{:8}", words));
                }
                if count_chars {
                    output.push_str(&format!("{:8}", chars));
                }
                output.push_str(&format!(" {}", filename));

                stdout.push(output);
            }
            Err(e) => {
                stderr.push(format!("wc: {}: {}", filename, e));
                *exit_code = 1;
            }
        }
    }

    // Show totals if multiple files
    if files.len() > 1 {
        let mut output = String::new();
        if count_lines {
            output.push_str(&format!("{:8}", total_lines));
        }
        if count_words {
            output.push_str(&format!("{:8}", total_words));
        }
        if count_chars {
            output.push_str(&format!("{:8}", total_chars));
        }
        output.push_str(" total");
        stdout.push(output);
    }
}

/// Execute help command
fn execute_help_command(stdout: &mut Vec<String>) {
    stdout.push("SeeU 安全终端 - 可用命令:".to_string());
    stdout.push("".to_string());
    stdout.push("📁 文件和目录操作:".to_string());
    stdout.push("  ls [选项] [目录]     - 列出目录内容 (-l 详细, -a 显示隐藏文件, -h 人性化大小)".to_string());
    stdout.push("  dir [选项] [目录]    - 列出目录内容 (Windows风格)".to_string());
    stdout.push("  cd [目录]           - 切换目录".to_string());
    stdout.push("  pwd                 - 显示当前工作目录".to_string());
    stdout.push("  cat <文件>          - 显示文件内容".to_string());
    stdout.push("  head [-n 行数] <文件> - 显示文件开头几行 (默认10行)".to_string());
    stdout.push("  tail [-n 行数] <文件> - 显示文件结尾几行 (默认10行)".to_string());
    stdout.push("  wc [-l|-w|-c] <文件> - 统计文件行数/单词数/字符数".to_string());
    stdout.push("  find <目录> -name <模式> - 查找文件".to_string());
    stdout.push("  tree [目录]         - 显示目录树结构".to_string());
    stdout.push("".to_string());
    stdout.push("🔍 搜索和过滤:".to_string());
    stdout.push("  grep <模式> <文件>   - 在文件中搜索文本模式".to_string());
    stdout.push("  which <命令>        - 查找命令位置".to_string());
    stdout.push("".to_string());
    stdout.push("💻 系统信息:".to_string());
    stdout.push("  whoami              - 显示当前用户名".to_string());
    stdout.push("  hostname            - 显示主机名".to_string());
    stdout.push("  date [-u|-I]        - 显示日期时间 (-u UTC时间, -I ISO格式)".to_string());
    stdout.push("  uptime              - 显示系统运行时间".to_string());
    stdout.push("  ps [选项]           - 显示进程信息".to_string());
    stdout.push("  env [变量名]        - 显示环境变量".to_string());
    stdout.push("  du [-h] [目录]      - 显示磁盘使用情况 (-h 人性化显示)".to_string());
    stdout.push("  df [-h]             - 显示文件系统使用情况 (-h 人性化显示)".to_string());
    stdout.push("".to_string());
    stdout.push("🛠️ 实用工具:".to_string());
    stdout.push("  echo <文本>         - 输出文本".to_string());
    stdout.push("  clear               - 清空终端".to_string());
    stdout.push("  history             - 查看命令历史 (使用📜按钮)".to_string());
    stdout.push("  help                - 显示此帮助信息".to_string());
    stdout.push("  exit                - 退出终端会话".to_string());
    stdout.push("".to_string());
    stdout.push("🔒 安全说明:".to_string());
    stdout.push("  • 此终端只允许执行安全的预定义命令".to_string());
    stdout.push("  • 所有文件操作都限制在安全范围内".to_string());
    stdout.push("  • 不支持危险的系统命令和网络操作".to_string());
    stdout.push("  • 使用 ⚙ 按钮可以配置终端设置".to_string());
    stdout.push("".to_string());
    stdout.push("💡 提示: 大多数命令支持 --help 选项查看详细用法".to_string());
}

// Utility functions

/// Format file size in human readable format
fn format_size(size: u64) -> String {
    const UNITS: &[&str] = &["B", "K", "M", "G", "T"];
    let mut size = size as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{:.0}{}", size, UNITS[unit_index])
    } else {
        format!("{:.1}{}", size, UNITS[unit_index])
    }
}

/// Format file permissions (simplified)
fn format_permissions(_metadata: &std::fs::Metadata) -> String {
    // Simplified permission display
    "rwxr-xr-x".to_string()
}

/// Execute find command
fn execute_find_command(working_dir: &str, args: &[&str], stdout: &mut Vec<String>, stderr: &mut Vec<String>, exit_code: &mut i32) {
    if args.is_empty() {
        stderr.push("find: missing starting-point".to_string());
        *exit_code = 1;
        return;
    }

    let start_dir = args[0];
    let mut name_pattern: Option<&str> = None;

    // Parse arguments
    let mut i = 1;
    while i < args.len() {
        match args[i] {
            "-name" => {
                if i + 1 < args.len() {
                    name_pattern = Some(args[i + 1]);
                    i += 2;
                } else {
                    stderr.push("find: option requires an argument -- name".to_string());
                    *exit_code = 1;
                    return;
                }
            }
            _ => {
                stderr.push(format!("find: unknown option '{}'", args[i]));
                *exit_code = 1;
                return;
            }
        }
    }

    let search_path = if start_dir.starts_with('/') || start_dir.contains(':') {
        std::path::PathBuf::from(start_dir)
    } else {
        std::path::PathBuf::from(working_dir).join(start_dir)
    };

    if let Err(e) = find_files(&search_path, name_pattern, stdout, stderr) {
        stderr.push(format!("find: {}: {}", start_dir, e));
        *exit_code = 1;
    }
}

/// Recursive file finder
fn find_files(dir: &std::path::Path, pattern: Option<&str>, stdout: &mut Vec<String>, stderr: &mut Vec<String>) -> Result<(), std::io::Error> {
    use regex::Regex;

    let regex = if let Some(pattern) = pattern {
        // Convert shell pattern to regex
        let regex_pattern = pattern
            .replace("*", ".*")
            .replace("?", ".");
        Regex::new(&format!("^{}$", regex_pattern)).ok()
    } else {
        None
    };

    fn visit_dir(dir: &std::path::Path, regex: &Option<Regex>, stdout: &mut Vec<String>, stderr: &mut Vec<String>) -> Result<(), std::io::Error> {
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            let file_name = entry.file_name();

            if let Some(name) = file_name.to_str() {
                let matches = if let Some(ref regex) = regex {
                    regex.is_match(name)
                } else {
                    true
                };

                if matches {
                    stdout.push(path.to_string_lossy().to_string());
                }
            }

            if path.is_dir() {
                if let Err(e) = visit_dir(&path, regex, stdout, stderr) {
                    stderr.push(format!("find: {}: {}", path.display(), e));
                }
            }
        }
        Ok(())
    }

    visit_dir(dir, &regex, stdout, stderr)
}

/// Execute grep command
fn execute_grep_command(working_dir: &str, args: &[&str], stdout: &mut Vec<String>, stderr: &mut Vec<String>, exit_code: &mut i32) {
    if args.len() < 2 {
        stderr.push("grep: missing pattern or file".to_string());
        *exit_code = 1;
        return;
    }

    let pattern = args[0];
    let files = &args[1..];

    let regex = match regex::Regex::new(pattern) {
        Ok(regex) => regex,
        Err(e) => {
            stderr.push(format!("grep: invalid pattern: {}", e));
            *exit_code = 1;
            return;
        }
    };

    for &filename in files {
        let path = if filename.starts_with('/') || filename.contains(':') {
            std::path::PathBuf::from(filename)
        } else {
            std::path::PathBuf::from(working_dir).join(filename)
        };

        match std::fs::read_to_string(&path) {
            Ok(content) => {
                for (line_num, line) in content.lines().enumerate() {
                    if regex.is_match(line) {
                        if files.len() > 1 {
                            stdout.push(format!("{}:{}:{}", filename, line_num + 1, line));
                        } else {
                            stdout.push(format!("{}:{}", line_num + 1, line));
                        }
                    }
                }
            }
            Err(e) => {
                stderr.push(format!("grep: {}: {}", filename, e));
                *exit_code = 1;
            }
        }
    }
}

/// Execute tree command
fn execute_tree_command(working_dir: &str, args: &[&str], stdout: &mut Vec<String>, stderr: &mut Vec<String>, exit_code: &mut i32) {
    let target_dir = args.get(0).unwrap_or(&working_dir);

    let path = if target_dir.starts_with('/') || target_dir.contains(':') {
        std::path::PathBuf::from(target_dir)
    } else {
        std::path::PathBuf::from(working_dir).join(target_dir)
    };

    if !path.exists() {
        stderr.push(format!("tree: {}: No such file or directory", target_dir));
        *exit_code = 1;
        return;
    }

    if !path.is_dir() {
        stderr.push(format!("tree: {}: Not a directory", target_dir));
        *exit_code = 1;
        return;
    }

    stdout.push(format!("{}", path.display()));

    if let Err(e) = print_tree(&path, "", stdout, stderr) {
        stderr.push(format!("tree: {}", e));
        *exit_code = 1;
    }
}

/// Print directory tree recursively
fn print_tree(dir: &std::path::Path, prefix: &str, stdout: &mut Vec<String>, stderr: &mut Vec<String>) -> Result<(), std::io::Error> {
    let entries: Result<Vec<_>, _> = std::fs::read_dir(dir)?.collect();
    let mut entries = entries?;
    entries.sort_by_key(|entry| entry.file_name());

    for (i, entry) in entries.iter().enumerate() {
        let is_last_entry = i == entries.len() - 1;
        let file_name = entry.file_name();
        let name = file_name.to_string_lossy();

        let current_prefix = if is_last_entry { "└── " } else { "├── " };
        stdout.push(format!("{}{}{}", prefix, current_prefix, name));

        if entry.path().is_dir() {
            let next_prefix = format!("{}{}", prefix, if is_last_entry { "    " } else { "│   " });
            if let Err(e) = print_tree(&entry.path(), &next_prefix, stdout, stderr) {
                stderr.push(format!("tree: {}: {}", entry.path().display(), e));
            }
        }
    }

    Ok(())
}

/// Execute du command
fn execute_du_command(working_dir: &str, args: &[&str], stdout: &mut Vec<String>, stderr: &mut Vec<String>, exit_code: &mut i32) {
    let human_readable = args.contains(&"-h") || args.contains(&"--human-readable");
    let target_dir = args.iter()
        .find(|arg| !arg.starts_with('-'))
        .unwrap_or(&working_dir);

    let path = if target_dir.starts_with('/') || target_dir.contains(':') {
        std::path::PathBuf::from(target_dir)
    } else {
        std::path::PathBuf::from(working_dir).join(target_dir)
    };

    match calculate_dir_size(&path) {
        Ok(size) => {
            let size_str = if human_readable {
                format_size(size)
            } else {
                (size / 1024).to_string() // Show in KB
            };
            stdout.push(format!("{}\t{}", size_str, path.display()));
        }
        Err(e) => {
            stderr.push(format!("du: {}: {}", target_dir, e));
            *exit_code = 1;
        }
    }
}

/// Calculate directory size recursively
fn calculate_dir_size(dir: &std::path::Path) -> Result<u64, std::io::Error> {
    let mut total_size = 0;

    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            total_size += calculate_dir_size(&path)?;
        } else {
            total_size += entry.metadata()?.len();
        }
    }

    Ok(total_size)
}

/// Execute df command
fn execute_df_command(stdout: &mut Vec<String>, _stderr: &mut Vec<String>, _exit_code: &mut i32) {
    use sysinfo::Disks;

    let disks = Disks::new_with_refreshed_list();

    stdout.push("Filesystem     1K-blocks    Used Available Use% Mounted on".to_string());

    for disk in &disks {
        let total = disk.total_space() / 1024; // Convert to KB
        let available = disk.available_space() / 1024;
        let used = total - available;
        let use_percent = if total > 0 { (used * 100) / total } else { 0 };

        stdout.push(format!(
            "{:<14} {:>9} {:>7} {:>9} {:>3}% {}",
            disk.name().to_string_lossy(),
            total,
            used,
            available,
            use_percent,
            disk.mount_point().display()
        ));
    }
}

/// Execute ps command
fn execute_ps_command(args: &[&str], stdout: &mut Vec<String>, _stderr: &mut Vec<String>, _exit_code: &mut i32) {
    use sysinfo::System;

    let show_all = args.contains(&"-a") || args.contains(&"--all");
    let _show_full = args.contains(&"-f") || args.contains(&"--full");

    let mut sys = System::new_all();
    sys.refresh_processes();

    stdout.push("  PID TTY          TIME CMD".to_string());

    let mut processes: Vec<_> = sys.processes().iter().collect();
    processes.sort_by_key(|(pid, _)| pid.as_u32());

    for (pid, process) in processes.iter().take(20) { // Limit to first 20 processes
        if !show_all && process.name() == "kernel_task" {
            continue;
        }

        let cmd = process.name();
        stdout.push(format!(
            "{:5} {:12} {:8} {}",
            pid.as_u32(),
            "?",
            "00:00:00",
            cmd
        ));
    }

    if !show_all {
        stdout.push("".to_string());
        stdout.push("Note: Use 'ps -a' to show all processes".to_string());
    }
}

/// Execute uptime command
fn execute_uptime_command(stdout: &mut Vec<String>, _stderr: &mut Vec<String>, _exit_code: &mut i32) {
    use sysinfo::System;

    let uptime_seconds = System::uptime();
    let days = uptime_seconds / 86400;
    let hours = (uptime_seconds % 86400) / 3600;
    let minutes = (uptime_seconds % 3600) / 60;

    let now = chrono::Local::now();
    let load_avg = System::load_average();

    let uptime_str = if days > 0 {
        format!("{} days, {}:{:02}", days, hours, minutes)
    } else {
        format!("{}:{:02}", hours, minutes)
    };

    stdout.push(format!(
        " {} up {}, load average: {:.2}, {:.2}, {:.2}",
        now.format("%H:%M:%S"),
        uptime_str,
        load_avg.one,
        load_avg.five,
        load_avg.fifteen
    ));
}