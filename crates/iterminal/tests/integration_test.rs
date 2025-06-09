use iterminal::initialize;

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
