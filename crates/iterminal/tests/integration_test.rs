use iterminal::{initialize, ITerminalState, TerminalConfig, TerminalSession, OutputLine, LineType};
use uuid::Uuid;
use chrono::Utc;

#[test]
fn test_terminal_initialization() {
    let state = initialize();

    // 验证初始状态
    assert!(!state.show_settings);
    assert!(!state.show_history);
    assert_eq!(state.font_scale, 1.0);
    assert_eq!(state.session_count(), 1); // 应该有一个默认会话
}

#[test]
fn test_session_creation() {
    let mut state = initialize();

    // 创建新会话
    let session_id = state.create_session(Some("Test Session".to_string()));

    // 验证会话创建
    assert_eq!(state.session_count(), 2); // 现在应该有两个会话
    assert!(state.set_active_session(session_id)); // 应该能设置为活动会话
}

#[test]
fn test_command_execution() {
    let mut state = initialize();

    // 执行内置命令
    assert!(state.execute_command("help".to_string()));
    assert!(state.execute_command("pwd".to_string()));
    assert!(state.execute_command("clear".to_string()));
}

#[test]
fn test_session_management() {
    let mut state = initialize();

    // 创建多个会话
    let session1 = state.create_session(Some("Session 1".to_string()));
    let session2 = state.create_session(Some("Session 2".to_string()));

    assert_eq!(state.session_count(), 3); // 默认 + 2个新会话

    // 切换会话
    assert!(state.set_active_session(session1));
    assert!(state.set_active_session(session2));

    // 关闭会话
    assert!(state.close_session(session1));
    assert_eq!(state.session_count(), 2);

    // 不能关闭不存在的会话
    assert!(!state.close_session(session1));
}

#[test]
fn test_configuration() {
    let mut state = initialize();
    let config = state.get_config().clone();

    // 验证默认配置
    assert_eq!(config.font_family, "Source Code Pro");
    assert_eq!(config.font_size, 14.0);
    assert_eq!(config.scrollback_lines, 10000);
    assert!(!config.enable_bell);

    // 修改配置
    let mut new_config = config;
    new_config.font_size = 16.0;
    new_config.enable_bell = true;

    state.update_config(new_config);

    // 验证配置更新
    let updated_config = state.get_config();
    assert_eq!(updated_config.font_size, 16.0);
    assert!(updated_config.enable_bell);
}

#[test]
fn test_font_scaling() {
    let mut state = initialize();

    // 测试字体缩放
    assert_eq!(state.font_scale, 1.0);

    state.increase_font_size();
    assert_eq!(state.font_scale, 1.1);

    state.decrease_font_size();
    assert_eq!(state.font_scale, 1.0);

    state.reset_font_size();
    assert_eq!(state.font_scale, 1.0);

    // 测试边界值
    for _ in 0..50 {
        state.increase_font_size();
    }
    assert!(state.font_scale <= 3.0);

    for _ in 0..50 {
        state.decrease_font_size();
    }
    assert!(state.font_scale >= 0.5);
}

#[test]
fn test_settings_toggle() {
    let mut state = initialize();

    // 测试设置对话框切换
    assert!(!state.show_settings);

    state.toggle_settings();
    assert!(state.show_settings);

    state.toggle_settings();
    assert!(!state.show_settings);
}

#[test]
fn test_history_toggle() {
    let mut state = initialize();

    // 测试历史对话框切换
    assert!(!state.show_history);

    state.toggle_history();
    assert!(state.show_history);

    state.toggle_history();
    assert!(!state.show_history);
}

#[test]
fn test_safe_commands() {
    let mut state = initialize();

    // 测试安全的内置命令
    assert!(state.execute_command("ls".to_string()));
    assert!(state.execute_command("pwd".to_string()));
    assert!(state.execute_command("whoami".to_string()));
    assert!(state.execute_command("hostname".to_string()));
    assert!(state.execute_command("date".to_string()));
    assert!(state.execute_command("env".to_string()));
    assert!(state.execute_command("echo hello world".to_string()));
    assert!(state.execute_command("uptime".to_string()));
    assert!(state.execute_command("df".to_string()));
    assert!(state.execute_command("ps".to_string()));
}

#[test]
fn test_file_commands() {
    let mut state = initialize();

    // 测试文件相关命令（使用简单的参数避免长时间运行）
    assert!(state.execute_command("ls -l".to_string()));
    assert!(state.execute_command("ls -a".to_string()));
    assert!(state.execute_command("ls -h".to_string()));
    // 注意：tree 和 find 命令可能在大目录中运行时间较长，在实际使用中需要注意
}

#[test]
fn test_command_with_arguments() {
    let mut state = initialize();

    // 测试带参数的命令
    assert!(state.execute_command("echo 'Hello, SeeU Terminal!'".to_string()));
    assert!(state.execute_command("date -u".to_string()));
    assert!(state.execute_command("date -I".to_string()));
    assert!(state.execute_command("ps -a".to_string()));
    assert!(state.execute_command("env PATH".to_string()));
    assert!(state.execute_command("which ls".to_string()));
}

#[test]
fn test_help_command() {
    let mut state = initialize();

    // 测试帮助命令
    assert!(state.execute_command("help".to_string()));

    // 验证帮助命令执行后有输出
    if let Some(session) = state.terminal_manager.get_active_session() {
        assert!(!session.output_buffer.is_empty());

        // 检查是否包含帮助信息
        let output_text: String = session.output_buffer
            .iter()
            .map(|line| line.content.as_str())
            .collect::<Vec<_>>()
            .join("\n");

        assert!(output_text.contains("安全终端"));
        assert!(output_text.contains("文件和目录操作"));
        assert!(output_text.contains("系统信息"));
        assert!(output_text.contains("安全说明"));
    }
}

#[test]
fn test_terminal_config_creation() {
    let config = TerminalConfig::default();

    assert_eq!(config.font_family, "Source Code Pro");
    assert_eq!(config.font_size, 14.0);
    assert_eq!(config.scrollback_lines, 10000);
    assert!(!config.enable_bell);
    assert_eq!(config.cursor_blink_rate, 500);
    assert_eq!(config.tab_width, 4);
    assert!(config.auto_scroll);
    assert!(config.show_line_numbers);
}

#[test]
fn test_terminal_config_modification() {
    let mut config = TerminalConfig::default();

    // 修改配置
    config.font_family = "Monaco".to_string();
    config.font_size = 16.0;
    config.scrollback_lines = 5000;
    config.enable_bell = true;
    config.cursor_blink_rate = 1000;
    config.tab_width = 8;
    config.auto_scroll = false;
    config.show_line_numbers = false;

    // 验证修改
    assert_eq!(config.font_family, "Monaco");
    assert_eq!(config.font_size, 16.0);
    assert_eq!(config.scrollback_lines, 5000);
    assert!(config.enable_bell);
    assert_eq!(config.cursor_blink_rate, 1000);
    assert_eq!(config.tab_width, 8);
    assert!(!config.auto_scroll);
    assert!(!config.show_line_numbers);
}

#[test]
fn test_terminal_session_creation() {
    let session = TerminalSession::new(Some("Test Session".to_string()));

    assert!(!session.id.to_string().is_empty());
    assert_eq!(session.name, "Test Session");
    assert!(session.output_buffer.is_empty());
    assert!(session.command_history.is_empty());
    assert_eq!(session.current_directory, std::env::current_dir().unwrap());
    assert!(session.created_at <= Utc::now());
    assert!(session.is_active);
}

#[test]
fn test_terminal_session_default_name() {
    let session = TerminalSession::new(None);

    assert!(session.name.starts_with("会话"));
    assert!(session.is_active);
}

#[test]
fn test_output_line_creation() {
    let line = OutputLine {
        content: "Hello, Terminal!".to_string(),
        line_type: LineType::Output,
        timestamp: Utc::now(),
    };

    assert_eq!(line.content, "Hello, Terminal!");
    assert_eq!(line.line_type, LineType::Output);
    assert!(line.timestamp <= Utc::now());
}

#[test]
fn test_line_type_variants() {
    // 测试所有行类型变体
    assert_eq!(LineType::Input, LineType::Input);
    assert_eq!(LineType::Output, LineType::Output);
    assert_eq!(LineType::Error, LineType::Error);
    assert_eq!(LineType::System, LineType::System);

    // 测试不同类型不相等
    assert_ne!(LineType::Input, LineType::Output);
    assert_ne!(LineType::Error, LineType::System);
}

#[test]
fn test_session_command_history() {
    let mut session = TerminalSession::new(Some("History Test".to_string()));

    // 添加命令到历史
    session.add_to_history("ls -la".to_string());
    session.add_to_history("pwd".to_string());
    session.add_to_history("echo hello".to_string());

    assert_eq!(session.command_history.len(), 3);
    assert_eq!(session.command_history[0], "ls -la");
    assert_eq!(session.command_history[1], "pwd");
    assert_eq!(session.command_history[2], "echo hello");
}

#[test]
fn test_session_output_buffer() {
    let mut session = TerminalSession::new(Some("Output Test".to_string()));

    // 添加输出行
    session.add_output_line("Command executed".to_string(), LineType::Output);
    session.add_output_line("Error occurred".to_string(), LineType::Error);
    session.add_output_line("System message".to_string(), LineType::System);

    assert_eq!(session.output_buffer.len(), 3);
    assert_eq!(session.output_buffer[0].content, "Command executed");
    assert_eq!(session.output_buffer[0].line_type, LineType::Output);
    assert_eq!(session.output_buffer[1].content, "Error occurred");
    assert_eq!(session.output_buffer[1].line_type, LineType::Error);
    assert_eq!(session.output_buffer[2].content, "System message");
    assert_eq!(session.output_buffer[2].line_type, LineType::System);
}

#[test]
fn test_session_directory_management() {
    let mut session = TerminalSession::new(Some("Directory Test".to_string()));
    let original_dir = session.current_directory.clone();

    // 测试目录设置
    let new_dir = std::path::PathBuf::from("/tmp");
    session.set_current_directory(new_dir.clone());
    assert_eq!(session.current_directory, new_dir);

    // 恢复原始目录
    session.set_current_directory(original_dir.clone());
    assert_eq!(session.current_directory, original_dir);
}

#[test]
fn test_session_active_state() {
    let mut session = TerminalSession::new(Some("Active Test".to_string()));

    assert!(session.is_active);

    session.set_active(false);
    assert!(!session.is_active);

    session.set_active(true);
    assert!(session.is_active);
}

#[test]
fn test_multiple_sessions_management() {
    let mut state = initialize();

    // 创建多个会话
    let session1_id = state.create_session(Some("Session 1".to_string()));
    let session2_id = state.create_session(Some("Session 2".to_string()));
    let session3_id = state.create_session(Some("Session 3".to_string()));

    assert_eq!(state.session_count(), 4); // 默认 + 3个新会话

    // 测试会话切换
    assert!(state.set_active_session(session1_id));
    assert!(state.set_active_session(session2_id));
    assert!(state.set_active_session(session3_id));

    // 测试获取会话信息
    if let Some(session) = state.get_session(session1_id) {
        assert_eq!(session.name, "Session 1");
    } else {
        panic!("Session 1 should exist");
    }

    // 测试关闭会话
    assert!(state.close_session(session2_id));
    assert_eq!(state.session_count(), 3);

    // 确认会话已被删除
    assert!(state.get_session(session2_id).is_none());
}

#[test]
fn test_command_validation() {
    let mut state = initialize();

    // 测试有效命令
    assert!(state.is_command_safe("ls"));
    assert!(state.is_command_safe("pwd"));
    assert!(state.is_command_safe("echo hello"));
    assert!(state.is_command_safe("date"));
    assert!(state.is_command_safe("whoami"));

    // 测试可能不安全的命令（根据实现可能会被阻止）
    // 注意：这些测试取决于具体的安全策略实现
    assert!(!state.is_command_safe("rm -rf /"));
    assert!(!state.is_command_safe("sudo rm"));
    assert!(!state.is_command_safe("chmod 777"));
}
