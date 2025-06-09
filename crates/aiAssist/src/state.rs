use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use crate::api::ApiService;
use once_cell::sync::Lazy;

// 全局状态，用于在UI线程和异步任务之间共享数据
static ACTIVE_REQUESTS: Lazy<Mutex<HashMap<Uuid, Arc<Mutex<StateUpdate>>>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

// 状态更新结构
pub struct StateUpdate {
    pub message_id: Uuid,
    pub content: String,
    pub is_complete: bool,
    pub error: Option<String>,
}

// 移除模型加载状态结构

/// Type for slash command callback
pub type SlashCommandCallback = Box<dyn FnMut(SlashCommand) + Send + 'static>;

/// Type for insert to note callback
pub type InsertToNoteCallback = Box<dyn FnMut(String) + Send + 'static>;

/// AI assistant state
pub struct AIAssistState {
    pub chat_messages: Vec<ChatMessage>,
    pub chat_input: String,
    pub should_focus_chat: bool,
    pub show_ai_settings: bool,
    pub show_history_dropdown: bool,
    pub show_attachment_dialog: bool,
    pub chat_sessions: Vec<ChatSession>,
    pub active_session_idx: usize,
    pub ai_settings: AISettings,
    pub api_service: Arc<ApiService>,
    pub is_sending: bool,
    pub streaming_content: String,
    pub streaming_message_id: Option<Uuid>,
    pub current_request_id: Option<Uuid>,
    pub slash_command_callback: Option<SlashCommandCallback>,
    pub insert_to_note_callback: Option<InsertToNoteCallback>,

    // 标记当前是否处于笔记视图且有打开的笔记
    pub can_insert_to_note: bool,

    // 控制@指令和Slash指令的提示框显示
    pub show_at_commands: bool,
    pub show_slash_commands: bool,

    // 存储最近的搜索查询和结果，用于 @search 引用
    pub last_search_query: Option<String>,
    pub last_search_results: Option<String>,
}

impl Default for AIAssistState {
    fn default() -> Self {
        // Create a default chat session
        let default_session = ChatSession {
            id: Uuid::new_v4(),
            name: "新对话".to_string(),
            created_at: Utc::now(),
            messages: vec![
                ChatMessage {
                    id: Uuid::new_v4(),
                    role: MessageRole::Assistant,
                    content: "你好！我是SeeU智能助手，有什么我可以帮助你的吗？".to_string(),
                    timestamp: Utc::now(),
                    attachments: vec![],
                }
            ],
        };

        Self {
            chat_messages: default_session.messages.clone(),
            chat_input: String::new(),
            should_focus_chat: false,
            show_ai_settings: false,
            show_history_dropdown: false,
            show_attachment_dialog: false,
            chat_sessions: vec![default_session],
            active_session_idx: 0,
            ai_settings: AISettings::default(),
            api_service: Arc::new(ApiService::new()),
            is_sending: false,
            streaming_content: String::new(),
            streaming_message_id: None,
            current_request_id: None,
            slash_command_callback: None,
            insert_to_note_callback: None,
            can_insert_to_note: false,
            show_at_commands: false,
            show_slash_commands: false,
            last_search_query: None,
            last_search_results: None,
        }
    }
}

impl AIAssistState {

    /// Send a message to the AI assistant
    pub fn send_message(&mut self) -> Option<SlashCommand> {
        if self.chat_input.trim().is_empty() || self.is_sending {
            return None;
        }

        // Check for slash commands
        let input = self.chat_input.trim();
        let slash_command = if input.starts_with('/') {
            self.parse_slash_command(input)
        } else {
            None
        };

        // If it's a slash command that should be handled externally, return it
        if let Some(cmd) = &slash_command {
            if matches!(cmd, SlashCommand::Search(_)) {
                // Create a user message showing the command
                let user_message = ChatMessage {
                    id: Uuid::new_v4(),
                    role: MessageRole::User,
                    content: self.chat_input.clone(),
                    timestamp: Utc::now(),
                    attachments: vec![],
                };

                // Add the message to the current session
                if let Some(session) = self.chat_sessions.get_mut(self.active_session_idx) {
                    // 检查是否是该会话的第一条用户消息
                    let is_first_user_message = session.messages.iter()
                        .filter(|msg| msg.role == MessageRole::User)
                        .count() == 0;

                    // 如果是第一条用户消息，更新会话名称为消息摘要
                    if is_first_user_message {
                        // 获取消息摘要（最多12个字符）
                        let summary = Self::get_message_summary(&user_message.content, 12);
                        session.name = summary;
                    }

                    session.messages.push(user_message.clone());
                }

                // Add the message to the current chat
                self.chat_messages.push(user_message);

                // Clear the input
                self.chat_input.clear();

                return slash_command;
            }
        }

        // Create a new user message
        let user_message = ChatMessage {
            id: Uuid::new_v4(),
            role: MessageRole::User,
            content: self.chat_input.clone(),
            timestamp: Utc::now(),
            attachments: vec![],
        };

        // Add the message to the current session
        if let Some(session) = self.chat_sessions.get_mut(self.active_session_idx) {
            // 检查是否是该会话的第一条用户消息
            let is_first_user_message = session.messages.iter()
                .filter(|msg| msg.role == MessageRole::User)
                .count() == 0;

            // 如果是第一条用户消息，更新会话名称为消息摘要
            if is_first_user_message {
                // 获取消息摘要（最多12个字符）
                let summary = Self::get_message_summary(&user_message.content, 12);
                session.name = summary;
            }

            session.messages.push(user_message.clone());
        }

        // Add the message to the current chat
        self.chat_messages.push(user_message);

        // Clear the input
        self.chat_input.clear();

        // Mark as sending
        self.is_sending = true;

        // Create a placeholder for the assistant's response
        self.create_assistant_placeholder();

        None
    }

    /// Parse slash command
    fn parse_slash_command(&self, input: &str) -> Option<SlashCommand> {
        let parts: Vec<&str> = input.splitn(2, ' ').collect();

        if parts.len() < 1 {
            return None;
        }

        match parts[0] {
            "/search" => {
                if parts.len() > 1 {
                    Some(SlashCommand::Search(parts[1].trim().to_string()))
                } else {
                    Some(SlashCommand::Search(String::new()))
                }
            },
            _ => None
        }
    }

    /// Create a placeholder for the assistant's response
    fn create_assistant_placeholder(&mut self) {
        let message_id = Uuid::new_v4();
        self.streaming_message_id = Some(message_id);
        self.streaming_content.clear();

        // Create a placeholder message
        let placeholder_message = ChatMessage {
            id: message_id,
            role: MessageRole::Assistant,
            content: "".to_string(),
            timestamp: Utc::now(),
            attachments: vec![],
        };

        // Add the placeholder to the current chat
        self.chat_messages.push(placeholder_message.clone());

        // Add the placeholder to the current session
        if let Some(session) = self.chat_sessions.get_mut(self.active_session_idx) {
            session.messages.push(placeholder_message);
        }

        // Prepare the messages for the API
        let messages = self.prepare_messages_for_api();

        // 创建一个可以在UI线程和异步任务之间共享的状态
        let state_mutex = Arc::new(Mutex::new(StateUpdate {
            message_id,
            content: String::new(),
            is_complete: false,
            error: None,
        }));

        // 克隆用于UI线程的状态
        let ui_state = state_mutex.clone();

        // 创建一个请求ID，用于在UI中标识这个请求
        let request_id = Uuid::new_v4();

        // 将请求ID存储在全局状态中，以便UI可以访问
        ACTIVE_REQUESTS.lock().unwrap().insert(request_id, ui_state);

        // Clone what we need for the background thread
        let api_service = self.api_service.clone();
        let ai_settings = self.ai_settings.clone();

        // 使用标准线程而不是tokio任务
        let state_mutex_clone = state_mutex.clone();
        let request_id_clone = request_id;

        // 使用标准线程来处理API调用
        std::thread::spawn(move || {
            // 创建一个单线程的tokio运行时
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();

            // 在运行时内执行异步任务
            rt.block_on(async {
                if ai_settings.streaming {
                    // 为闭包创建一个新的Arc克隆
                    let callback_state = state_mutex_clone.clone();

                    // Use streaming API
                    let result = api_service.send_chat_stream(
                        &ai_settings,
                        messages,
                        move |content| {
                            // 更新共享状态
                            let mut state = callback_state.lock().unwrap();
                            state.content = content;
                        },
                    ).await;

                    // 标记为完成
                    let mut state = state_mutex_clone.lock().unwrap();
                    state.is_complete = true;

                    if let Err(e) = result {
                        log::error!("Error sending chat stream: {}", e);
                        state.error = Some(format!("错误: {}", e));
                    }
                } else {
                    // Use non-streaming API
                    match api_service.send_chat(&ai_settings, messages).await {
                        Ok(response) => {
                            // 更新共享状态
                            let mut state = state_mutex_clone.lock().unwrap();
                            state.content = response;
                            state.is_complete = true;
                        },
                        Err(e) => {
                            log::error!("Error sending chat: {}", e);
                            let mut state = state_mutex_clone.lock().unwrap();
                            state.error = Some(format!("错误: {}", e));
                            state.is_complete = true;
                        }
                    }
                }

                // 请求完成后，从全局状态中移除
                ACTIVE_REQUESTS.lock().unwrap().remove(&request_id_clone);
            });
        });

        // 存储请求ID，以便UI可以检查更新
        self.current_request_id = Some(request_id);
    }

    /// Prepare messages for the API
    fn prepare_messages_for_api(&self) -> Vec<(MessageRole, String)> {
        // Get all messages from the current session
        let session = &self.chat_sessions[self.active_session_idx];

        // Convert to the format expected by the API, processing @search references
        session.messages.iter()
            .map(|msg| {
                let content = if msg.role == MessageRole::User {
                    // 处理用户消息中的 @search 引用
                    self.process_search_references(&msg.content)
                } else {
                    msg.content.clone()
                };

                (msg.role.clone(), content)
            })
            .collect()
    }

    /// 处理消息中的 @search 引用
    fn process_search_references(&self, content: &str) -> String {
        // 如果消息中包含 @search 并且我们有搜索结果，则替换为实际的搜索结果
        if content.contains("@search") && self.last_search_query.is_some() && self.last_search_results.is_some() {
            let query = self.last_search_query.as_ref().unwrap();
            let results = self.last_search_results.as_ref().unwrap();

            // 替换 @search 为实际的搜索结果
            let replacement = format!("@search (查询: \"{}\"):\n{}", query, results);
            content.replace("@search", &replacement)
        } else {
            content.to_string()
        }
    }

    /// Update streaming content
    pub fn update_streaming_content(&mut self, content: String) {
        self.streaming_content = content.clone();

        // Update the message content
        if let Some(message_id) = self.streaming_message_id {
            // Update in chat_messages
            for msg in &mut self.chat_messages {
                if msg.id == message_id {
                    msg.content = content.clone();
                    break;
                }
            }

            // Update in chat_sessions
            if let Some(session) = self.chat_sessions.get_mut(self.active_session_idx) {
                for msg in &mut session.messages {
                    if msg.id == message_id {
                        msg.content = content;
                        break;
                    }
                }
            }
        }
    }

    /// Complete streaming
    pub fn complete_streaming(&mut self) {
        self.is_sending = false;
        self.streaming_message_id = None;
        self.streaming_content.clear();
        self.current_request_id = None;
    }

    /// Check for updates from async tasks
    pub fn check_for_updates(&mut self) {
        // 如果没有正在进行的请求，直接返回
        if let Some(request_id) = self.current_request_id {
            // 获取全局状态
            let active_requests = ACTIVE_REQUESTS.lock().unwrap();

            // 查找当前请求
            if let Some(state_mutex) = active_requests.get(&request_id) {
                // 获取状态更新
                let state = state_mutex.lock().unwrap();

                // 更新流式内容
                if !state.content.is_empty() {
                    self.update_streaming_content(state.content.clone());
                }

                // 如果请求完成，更新状态
                if state.is_complete {
                    // 如果有错误，添加到消息中
                    if let Some(error) = &state.error {
                        self.update_streaming_content(format!("错误: {}", error));
                    }

                    // 完成流式输出
                    self.complete_streaming();
                }
            } else {
                // 如果请求不存在，可能已经完成
                self.complete_streaming();
            }
        }
    }

    /// Create a new chat session
    pub fn create_new_session(&mut self) {
        let new_session = ChatSession {
            id: Uuid::new_v4(),
            name: "新对话".to_string(), // 初始名称，会在用户发送第一条消息时更新
            created_at: Utc::now(),
            messages: vec![
                ChatMessage {
                    id: Uuid::new_v4(),
                    role: MessageRole::Assistant,
                    content: "你好！我是SeeU智能助手，有什么我可以帮助你的吗？".to_string(),
                    timestamp: Utc::now(),
                    attachments: vec![],
                }
            ],
        };

        self.chat_sessions.push(new_session);
        self.active_session_idx = self.chat_sessions.len() - 1;
        self.chat_messages = self.chat_sessions[self.active_session_idx].messages.clone();
    }

    /// Switch to a different chat session
    pub fn switch_session(&mut self, idx: usize) {
        if idx < self.chat_sessions.len() {
            self.active_session_idx = idx;
            self.chat_messages = self.chat_sessions[idx].messages.clone();
        }
    }

    /// Set the slash command callback
    pub fn set_slash_command_callback<F>(&mut self, callback: F)
    where
        F: FnMut(SlashCommand) + Send + 'static,
    {
        self.slash_command_callback = Some(Box::new(callback));
    }

    /// Set the insert to note callback
    pub fn set_insert_to_note_callback<F>(&mut self, callback: F)
    where
        F: FnMut(String) + Send + 'static,
    {
        self.insert_to_note_callback = Some(Box::new(callback));
    }

    /// Add a search result reference to the current chat
    pub fn add_search_reference(&mut self, query: &str, result_count: usize) {
        // 存储最近的搜索查询
        self.last_search_query = Some(query.to_string());

        // Create a system message with search results reference
        let system_message = ChatMessage {
            id: Uuid::new_v4(),
            role: MessageRole::System,
            content: format!("搜索结果: 找到 {} 个匹配 \"{}\" 的结果。可以使用 @search 引用这些结果。", result_count, query),
            timestamp: Utc::now(),
            attachments: vec![],
        };

        // Add the message to the current chat
        self.chat_messages.push(system_message.clone());

        // Add the message to the current session
        if let Some(session) = self.chat_sessions.get_mut(self.active_session_idx) {
            session.messages.push(system_message);
        }
    }

    /// 设置搜索结果内容
    pub fn set_search_results(&mut self, results: String) {
        self.last_search_results = Some(results);
    }

    /// 更新是否可以插入到笔记的状态
    pub fn update_can_insert_to_note(&mut self, can_insert: bool) {
        self.can_insert_to_note = can_insert;
    }

    /// 获取消息摘要，截取指定长度的字符
    fn get_message_summary(message: &str, max_length: usize) -> String {
        let trimmed = message.trim();
        if trimmed.is_empty() {
            return "新对话".to_string();
        }

        // 获取消息的前N个字符作为摘要
        let chars: Vec<char> = trimmed.chars().collect();
        let summary: String = chars.iter().take(max_length).collect();

        // 如果原消息比摘要长，添加省略号
        if chars.len() > max_length {
            format!("{}...", summary)
        } else {
            summary
        }
    }
}

/// Chat message
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChatMessage {
    pub id: Uuid,
    pub role: MessageRole,
    pub content: String,
    pub timestamp: DateTime<Utc>,
    pub attachments: Vec<Attachment>,
}

/// Message role
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum MessageRole {
    User,
    Assistant,
    System,
}

/// Chat session
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChatSession {
    pub id: Uuid,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub messages: Vec<ChatMessage>,
}

/// Attachment
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Attachment {
    pub id: Uuid,
    pub name: String,
    pub attachment_type: AttachmentType,
    pub content: String,
}

/// Attachment type
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum AttachmentType {
    Image,
    File,
    CodeSnippet,
}

/// Slash command
#[derive(Clone, Debug, PartialEq)]
pub enum SlashCommand {
    Search(String),
}

// 移除ProviderType，统一使用OpenAI compatible格式

/// AI settings
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AISettings {
    pub base_url: String,
    pub api_key: String,
    pub model: String,
    pub temperature: f32,
    pub max_tokens: u32,
    pub streaming: bool,
}

impl AISettings {
    /// Get the chat endpoint URL (OpenAI compatible)
    pub fn get_chat_url(&self) -> String {
        let base = self.base_url.trim_end_matches('/');
        // 如果base_url已经以/v1结尾，直接添加/chat/completions
        // 否则先添加/v1再添加/chat/completions
        if base.ends_with("/v1") {
            format!("{}/chat/completions", base)
        } else {
            format!("{}/v1/chat/completions", base)
        }
    }

    /// Get the models endpoint URL (OpenAI compatible)
    pub fn get_models_url(&self) -> String {
        let base = self.base_url.trim_end_matches('/');
        if base.ends_with("/v1") {
            format!("{}/models", base)
        } else {
            format!("{}/v1/models", base)
        }
    }
}

impl Default for AISettings {
    fn default() -> Self {
        Self {
            base_url: "http://localhost:11434/v1".to_string(),
            api_key: "".to_string(),
            model: "qwen2.5:7b".to_string(),
            temperature: 0.7,
            max_tokens: 2000,
            streaming: true,
        }
    }
}
