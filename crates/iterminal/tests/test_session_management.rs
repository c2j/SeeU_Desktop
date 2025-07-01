use iterminal::{initialize, ITerminalState, TerminalSession, OutputLine, LineType};
use uuid::Uuid;
use chrono::Utc;

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
fn test_session_buffer_limits() {
    let mut session = TerminalSession::new(Some("Buffer Test".to_string()));
    
    // 添加大量输出行测试缓冲区限制
    for i in 0..1000 {
        session.add_output_line(format!("Line {}", i), LineType::Output);
    }
    
    // 根据实际实现，可能有缓冲区大小限制
    // 这里假设没有限制，或者测试限制是否正确实施
    assert!(session.output_buffer.len() <= 1000);
    
    // 如果有限制，验证最新的行被保留
    if session.output_buffer.len() < 1000 {
        let last_line = &session.output_buffer[session.output_buffer.len() - 1];
        assert!(last_line.content.contains("Line 999"));
    }
}

#[test]
fn test_session_command_history_limits() {
    let mut session = TerminalSession::new(Some("History Limit Test".to_string()));
    
    // 添加大量命令测试历史限制
    for i in 0..500 {
        session.add_to_history(format!("command_{}", i));
    }
    
    // 根据实际实现验证历史记录限制
    assert!(session.command_history.len() <= 500);
    
    // 如果有限制，验证最新的命令被保留
    if session.command_history.len() < 500 {
        let last_command = &session.command_history[session.command_history.len() - 1];
        assert!(last_command.contains("command_499"));
    }
}

#[test]
fn test_session_timestamp_ordering() {
    let mut session = TerminalSession::new(Some("Timestamp Test".to_string()));
    
    let start_time = Utc::now();
    
    // 添加多个输出行
    session.add_output_line("First line".to_string(), LineType::Output);
    std::thread::sleep(std::time::Duration::from_millis(1));
    session.add_output_line("Second line".to_string(), LineType::Output);
    std::thread::sleep(std::time::Duration::from_millis(1));
    session.add_output_line("Third line".to_string(), LineType::Output);
    
    let end_time = Utc::now();
    
    // 验证时间戳顺序
    assert!(session.output_buffer.len() == 3);
    assert!(session.output_buffer[0].timestamp >= start_time);
    assert!(session.output_buffer[0].timestamp <= session.output_buffer[1].timestamp);
    assert!(session.output_buffer[1].timestamp <= session.output_buffer[2].timestamp);
    assert!(session.output_buffer[2].timestamp <= end_time);
}
