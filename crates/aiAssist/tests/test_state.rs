use aiAssist::state::{AIAssistState, ChatMessage, MessageRole, ChatSession, AISettings};
use uuid::Uuid;
use chrono::Utc;

#[test]
fn test_ai_settings_default_values() {
    let settings = AISettings::default();
    
    assert_eq!(settings.api_key, "");
    assert_eq!(settings.base_url, "http://localhost:11434/v1");
    assert_eq!(settings.model, "qwen2.5:7b");
    assert_eq!(settings.temperature, 0.7);
    assert_eq!(settings.max_tokens, 2000);
    assert!(settings.streaming);
}

#[test]
fn test_ai_settings_url_methods() {
    let settings = AISettings {
        base_url: "https://api.example.com/v1".to_string(),
        ..Default::default()
    };
    
    assert_eq!(settings.get_chat_url(), "https://api.example.com/v1/chat/completions");
    assert_eq!(settings.get_models_url(), "https://api.example.com/v1/models");
}

#[test]
fn test_chat_session_structure() {
    let session = ChatSession {
        id: Uuid::new_v4(),
        name: "Test Session".to_string(),
        messages: vec![],
        created_at: Utc::now(),
    };
    
    assert_eq!(session.name, "Test Session");
    assert!(session.messages.is_empty());
    assert!(session.created_at <= Utc::now());
}

#[test]
fn test_chat_message_structure() {
    let message = ChatMessage {
        id: Uuid::new_v4(),
        role: MessageRole::User,
        content: "Hello, AI!".to_string(),
        timestamp: Utc::now(),
        attachments: vec![],
        tool_calls: None,
        tool_call_results: None,
        mcp_server_info: None,
    };
    
    assert_eq!(message.role, MessageRole::User);
    assert_eq!(message.content, "Hello, AI!");
    assert!(message.attachments.is_empty());
    assert!(message.tool_calls.is_none());
}

#[test]
fn test_chat_session_message_management() {
    let mut session = ChatSession {
        id: Uuid::new_v4(),
        name: "Test Session".to_string(),
        messages: vec![],
        created_at: Utc::now(),
    };
    
    let message1 = ChatMessage {
        id: Uuid::new_v4(),
        role: MessageRole::User,
        content: "First message".to_string(),
        timestamp: Utc::now(),
        attachments: vec![],
        tool_calls: None,
        tool_call_results: None,
        mcp_server_info: None,
    };
    
    let message2 = ChatMessage {
        id: Uuid::new_v4(),
        role: MessageRole::Assistant,
        content: "Second message".to_string(),
        timestamp: Utc::now(),
        attachments: vec![],
        tool_calls: None,
        tool_call_results: None,
        mcp_server_info: None,
    };
    
    session.messages.push(message1);
    session.messages.push(message2);
    
    assert_eq!(session.messages.len(), 2);
    assert_eq!(session.messages[0].role, MessageRole::User);
    assert_eq!(session.messages[1].role, MessageRole::Assistant);
}

#[test]
fn test_message_role_variants() {
    assert_eq!(MessageRole::User, MessageRole::User);
    assert_eq!(MessageRole::Assistant, MessageRole::Assistant);
    assert_eq!(MessageRole::System, MessageRole::System);
    assert_eq!(MessageRole::SlashCommand, MessageRole::SlashCommand);
    
    // 测试不同角色不相等
    assert_ne!(MessageRole::User, MessageRole::Assistant);
    assert_ne!(MessageRole::System, MessageRole::SlashCommand);
}

#[test]
fn test_ai_assist_state_default() {
    let state = AIAssistState::default();

    // 测试默认状态
    assert_eq!(state.chat_sessions.len(), 1); // 默认创建一个会话
    assert_eq!(state.active_session_idx, 0);
    assert!(state.current_request_id.is_none());
    assert!(state.selected_mcp_server.is_none());
    assert!(state.mcp_server_capabilities.is_empty());
    assert!(state.server_names.is_empty());
    assert!(!state.is_sending);
    assert!(!state.show_ai_settings);
}
