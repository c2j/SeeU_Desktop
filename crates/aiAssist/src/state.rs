use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use crate::api::{ApiService, ToolCall, ChatResponse};
use crate::mcp_tools::{McpToolCallInfo, McpToolCallResult, McpServerCapabilities};
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
    pub has_function_calls: bool,
    pub function_call_response: Option<crate::api::ChatResponse>,
}

// 移除模型加载状态结构

/// Type for slash command callback
pub type SlashCommandCallback = Box<dyn FnMut(SlashCommand) + Send + 'static>;

/// Type for insert to note callback
pub type InsertToNoteCallback = Box<dyn FnMut(String) + Send + 'static>;

/// Type for MCP refresh callback
pub type McpRefreshCallback = Box<dyn FnMut() + Send + 'static>;

/// Command menu state for smart command suggestions
#[derive(Debug, Clone)]
pub struct CommandMenuState {
    pub is_visible: bool,
    pub menu_type: CommandMenuType,
    pub selected_index: usize,
    pub cursor_position: Option<egui::Pos2>,
    pub trigger_position: usize, // Position in text where @ or / was typed
}

/// Type of command menu
#[derive(Debug, Clone, PartialEq)]
pub enum CommandMenuType {
    None,
    AtCommands,
    SlashCommands,
}

/// Available @ commands
#[derive(Debug, Clone)]
pub struct AtCommand {
    pub name: &'static str,
    pub description: &'static str,
    pub insert_text: &'static str,
}

/// Available slash commands
#[derive(Debug, Clone)]
pub struct SlashCommandInfo {
    pub name: &'static str,
    pub description: &'static str,
    pub insert_text: &'static str,
}

impl Default for CommandMenuState {
    fn default() -> Self {
        Self {
            is_visible: false,
            menu_type: CommandMenuType::None,
            selected_index: 0,
            cursor_position: None,
            trigger_position: 0,
        }
    }
}

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
    pub mcp_refresh_callback: Option<McpRefreshCallback>,

    // 标记当前是否处于笔记视图且有打开的笔记
    pub can_insert_to_note: bool,

    // 控制@指令和Slash指令的提示框显示
    pub show_at_commands: bool,
    pub show_slash_commands: bool,

    // 智能指令菜单状态
    pub command_menu: CommandMenuState,

    // MCP相关状态
    pub selected_mcp_server: Option<Uuid>,
    pub mcp_server_capabilities: HashMap<Uuid, McpServerCapabilities>,
    pub server_names: HashMap<Uuid, String>,
    pub pending_tool_calls: Vec<PendingToolCall>,
    pub show_tool_call_confirmation: bool,
    pub current_tool_call_batch: Option<ToolCallBatch>,

    /// 待处理的Function Call响应
    pub pending_function_call_response: Option<crate::api::ChatResponse>,

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
                    tool_calls: None,
                    tool_call_results: None,
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
            mcp_refresh_callback: None,
            can_insert_to_note: false,
            show_at_commands: false,
            show_slash_commands: false,
            command_menu: CommandMenuState::default(),
            selected_mcp_server: None,
            mcp_server_capabilities: HashMap::new(),
            server_names: HashMap::new(),
            pending_tool_calls: Vec::new(),
            show_tool_call_confirmation: false,
            current_tool_call_batch: None,
            pending_function_call_response: None,
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

        // Handle slash commands
        if let Some(cmd) = &slash_command {
            match cmd {
                SlashCommand::Search(_) => {
                    // Create a slash command message showing the command
                    let user_message = ChatMessage {
                        id: Uuid::new_v4(),
                        role: MessageRole::SlashCommand,
                        content: self.chat_input.clone(),
                        timestamp: Utc::now(),
                        attachments: vec![],
                        tool_calls: None,
                        tool_call_results: None,
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
                },
                SlashCommand::Clear => {
                    // Create a slash command message showing the command
                    let clear_message = ChatMessage {
                        id: Uuid::new_v4(),
                        role: MessageRole::SlashCommand,
                        content: self.chat_input.clone(),
                        timestamp: Utc::now(),
                        attachments: vec![],
                        tool_calls: None,
                        tool_call_results: None,
                    };

                    self.chat_messages.push(clear_message.clone());

                    // Add to current session
                    if let Some(session) = self.chat_sessions.get_mut(self.active_session_idx) {
                        session.messages.push(clear_message);
                    }

                    self.clear_current_session();
                    self.chat_input.clear();
                    return None;
                },
                SlashCommand::New => {
                    // Create a slash command message showing the command
                    let new_message = ChatMessage {
                        id: Uuid::new_v4(),
                        role: MessageRole::SlashCommand,
                        content: self.chat_input.clone(),
                        timestamp: Utc::now(),
                        attachments: vec![],
                        tool_calls: None,
                        tool_call_results: None,
                    };

                    self.chat_messages.push(new_message.clone());

                    // Add to current session
                    if let Some(session) = self.chat_sessions.get_mut(self.active_session_idx) {
                        session.messages.push(new_message);
                    }

                    self.create_new_session();
                    self.chat_input.clear();
                    return None;
                },
                SlashCommand::Help => {
                    // Create a slash command message showing the command
                    let command_message = ChatMessage {
                        id: Uuid::new_v4(),
                        role: MessageRole::SlashCommand,
                        content: self.chat_input.clone(),
                        timestamp: Utc::now(),
                        attachments: vec![],
                        tool_calls: None,
                        tool_call_results: None,
                    };

                    self.chat_messages.push(command_message.clone());

                    // Add to current session
                    if let Some(session) = self.chat_sessions.get_mut(self.active_session_idx) {
                        session.messages.push(command_message);
                    }

                    // Add a help message to the current chat
                    let help_message = ChatMessage {
                        id: Uuid::new_v4(),
                        role: MessageRole::System,
                        content: "可用的斜杠命令:\n/search [查询] - 执行搜索\n/clear - 清空当前会话\n/new - 创建新会话\n/help - 显示此帮助信息".to_string(),
                        timestamp: Utc::now(),
                        attachments: vec![],
                        tool_calls: None,
                        tool_call_results: None,
                    };

                    self.chat_messages.push(help_message.clone());

                    // Add to current session
                    if let Some(session) = self.chat_sessions.get_mut(self.active_session_idx) {
                        session.messages.push(help_message);
                    }

                    self.chat_input.clear();
                    self.auto_save_sessions();
                    return None;
                }
            }
        }

        // Create a new user message
        let user_message = ChatMessage {
            id: Uuid::new_v4(),
            role: MessageRole::User,
            content: self.chat_input.clone(),
            timestamp: Utc::now(),
            attachments: vec![],
            tool_calls: None,
            tool_call_results: None,
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

        // Auto-save sessions after adding a message
        self.auto_save_sessions();

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
            "/clear" => Some(SlashCommand::Clear),
            "/new" => Some(SlashCommand::New),
            "/help" => Some(SlashCommand::Help),
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
            tool_calls: None,
            tool_call_results: None,
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
            has_function_calls: false,
            function_call_response: None,
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

        // 检查是否有选中的MCP服务器，如果有则准备工具
        let tools = if let Some(server_id) = self.selected_mcp_server {
            if let Some(capabilities) = self.mcp_server_capabilities.get(&server_id) {
                let server_name = self.server_names.get(&server_id)
                    .cloned()
                    .unwrap_or_else(|| format!("服务器 {}", server_id.to_string().chars().take(8).collect::<String>()));

                log::info!("🔧 AI助手 - 选中MCP服务器: {}", server_name);
                log::info!("📋 MCP服务器能力统计:");
                log::info!("  - 工具数量: {}", capabilities.tools.len());
                log::info!("  - 资源数量: {}", capabilities.resources.len());
                log::info!("  - 提示数量: {}", capabilities.prompts.len());

                if !capabilities.tools.is_empty() {
                    log::info!("🛠️ 可用工具列表:");
                    for (index, tool) in capabilities.tools.iter().enumerate() {
                        log::info!("  {}. {} - {}",
                            index + 1,
                            tool.name,
                            tool.description.as_deref().unwrap_or("无描述")
                        );
                    }
                }

                let converted_tools = crate::mcp_tools::McpToolConverter::convert_mcp_tools_to_openai(capabilities);
                log::info!("✅ 已将 {} 个MCP工具转换为OpenAI Function Calling格式", converted_tools.len());

                Some(converted_tools)
            } else {
                log::warn!("⚠️ 选中的MCP服务器 {} 没有找到能力信息", server_id);
                None
            }
        } else {
            log::debug!("💡 未选择MCP服务器，将不使用工具调用功能");
            None
        };

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

                    // Use streaming API with tools support
                    let result = api_service.send_chat_stream_with_tools(
                        &ai_settings,
                        messages,
                        tools,
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
                    // Use non-streaming API with tools support
                    if tools.is_some() {
                        // 使用支持工具的API
                        match api_service.send_chat_with_tools_full_response(&ai_settings, messages, tools).await {
                            Ok(response) => {
                                // 检查是否有工具调用
                                if let Some(choice) = response.choices.first() {
                                    if let Some(tool_calls) = &choice.message.tool_calls {
                                        // 有工具调用，存储响应以便主线程处理
                                        log::info!("🎯 LLM响应包含工具调用:");
                                        log::info!("  - 工具调用数量: {}", tool_calls.len());

                                        for (index, tool_call) in tool_calls.iter().enumerate() {
                                            log::info!("  {}. 工具调用ID: {}", index + 1, tool_call.id);
                                            log::info!("     函数名称: {}", tool_call.function.name);
                                            log::info!("     函数参数: {}", tool_call.function.arguments);
                                        }

                                        let mut state = state_mutex_clone.lock().unwrap();
                                        // 存储完整的响应，包含文本内容和工具调用
                                        state.content = choice.message.content.clone().unwrap_or_default();
                                        state.is_complete = true;

                                        // 标记有待处理的Function Call响应
                                        // 这个标记会在主线程的check_for_updates中被检测到
                                        state.has_function_calls = true;
                                        state.function_call_response = Some(response.clone());
                                    } else {
                                        // 普通响应
                                        let mut state = state_mutex_clone.lock().unwrap();
                                        state.content = choice.message.content.clone().unwrap_or_default();
                                        state.is_complete = true;
                                    }
                                } else {
                                    let mut state = state_mutex_clone.lock().unwrap();
                                    state.error = Some("响应中没有选择".to_string());
                                    state.is_complete = true;
                                }
                            },
                            Err(e) => {
                                log::error!("Error sending chat with tools: {}", e);
                                let mut state = state_mutex_clone.lock().unwrap();
                                state.error = Some(format!("错误: {}", e));
                                state.is_complete = true;
                            }
                        }
                    } else {
                        // 普通API调用
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
        // 过滤掉Slash指令消息，因为它们不应该发送给LLM
        session.messages.iter()
            .filter(|msg| msg.role != MessageRole::SlashCommand)
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

            // 替换 @search 为实际的搜索结果（第一条结果的详细内容）
            let replacement = format!("@search (查询: \"{}\" 的第一条结果):\n{}", query, results);
            content.replace("@search", &replacement)
        } else if content.contains("@search") {
            // 如果没有搜索结果，提示用户先进行搜索
            content.replace("@search", "@search (请先使用 /search 命令进行搜索)")
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

        // Auto-save sessions after completing a response
        self.auto_save_sessions();
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

                    // 检查是否有Function Call响应需要处理
                    if state.has_function_calls {
                        if let Some(response) = &state.function_call_response {
                            self.handle_function_call_response(response.clone());
                        }
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
                    tool_calls: None,
                    tool_call_results: None,
                }
            ],
        };

        self.chat_sessions.push(new_session);
        self.active_session_idx = self.chat_sessions.len() - 1;
        self.chat_messages = self.chat_sessions[self.active_session_idx].messages.clone();

        // Auto-save sessions after creating a new session
        self.auto_save_sessions();
    }

    /// Switch to a different chat session
    pub fn switch_session(&mut self, idx: usize) {
        if idx < self.chat_sessions.len() {
            self.active_session_idx = idx;
            self.chat_messages = self.chat_sessions[idx].messages.clone();

            // Auto-save sessions after switching
            self.auto_save_sessions();
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

    /// Set the MCP refresh callback
    pub fn set_mcp_refresh_callback<F>(&mut self, callback: F)
    where
        F: FnMut() + Send + 'static,
    {
        self.mcp_refresh_callback = Some(Box::new(callback));
    }

    /// Add a search result reference to the current chat
    pub fn add_search_reference(&mut self, query: &str, result_count: usize) {
        // 存储最近的搜索查询
        self.last_search_query = Some(query.to_string());

        // Create a system message with search results reference
        let system_message = ChatMessage {
            id: Uuid::new_v4(),
            role: MessageRole::System,
            content: if result_count > 0 {
                format!("搜索完成: 找到 {} 个匹配 \"{}\" 的结果。使用 @search 可以引用第一条搜索结果的详细内容。", result_count, query)
            } else {
                format!("搜索完成: 未找到匹配 \"{}\" 的结果。", query)
            },
            timestamp: Utc::now(),
            attachments: vec![],
            tool_calls: None,
            tool_call_results: None,
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

    /// Save current chat sessions to persistent storage
    pub fn save_sessions(&self) -> Result<(), Box<dyn std::error::Error>> {
        crate::save_chat_sessions(self)
    }

    /// Auto-save sessions after important operations
    pub fn auto_save_sessions(&self) {
        if let Err(err) = self.save_sessions() {
            log::error!("Failed to auto-save chat sessions: {}", err);
        }
    }

    /// Delete a chat session
    pub fn delete_session(&mut self, idx: usize) {
        if idx < self.chat_sessions.len() && self.chat_sessions.len() > 1 {
            self.chat_sessions.remove(idx);

            // Adjust active session index
            if self.active_session_idx >= self.chat_sessions.len() {
                self.active_session_idx = self.chat_sessions.len() - 1;
            } else if self.active_session_idx > idx {
                self.active_session_idx -= 1;
            }

            // Update current chat messages
            if let Some(active_session) = self.chat_sessions.get(self.active_session_idx) {
                self.chat_messages = active_session.messages.clone();
            }

            // Auto-save after deletion
            self.auto_save_sessions();
        }
    }

    /// Rename a chat session
    pub fn rename_session(&mut self, idx: usize, new_name: String) {
        if idx < self.chat_sessions.len() {
            self.chat_sessions[idx].name = new_name;

            // Auto-save after renaming
            self.auto_save_sessions();
        }
    }

    /// Clear current chat session (keep only the initial assistant message)
    pub fn clear_current_session(&mut self) {
        if let Some(session) = self.chat_sessions.get_mut(self.active_session_idx) {
            // Keep only the initial assistant message
            let initial_message = ChatMessage {
                id: Uuid::new_v4(),
                role: MessageRole::Assistant,
                content: "你好！我是SeeU智能助手，有什么我可以帮助你的吗？".to_string(),
                timestamp: Utc::now(),
                attachments: vec![],
                tool_calls: None,
                tool_call_results: None,
            };

            session.messages = vec![initial_message.clone()];
            self.chat_messages = vec![initial_message];

            // Auto-save after clearing
            self.auto_save_sessions();
        }
    }

    /// Get session count
    pub fn get_session_count(&self) -> usize {
        self.chat_sessions.len()
    }

    /// Get current session name
    pub fn get_current_session_name(&self) -> String {
        if let Some(session) = self.chat_sessions.get(self.active_session_idx) {
            session.name.clone()
        } else {
            "未知会话".to_string()
        }
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

    /// 检查输入变化并更新指令菜单状态
    pub fn update_command_menu(&mut self, cursor_pos: Option<egui::Pos2>) {
        let input = &self.chat_input;

        // 检查是否应该显示指令菜单
        if let Some(trigger_pos) = self.find_command_trigger(input) {
            let trigger_char = input.chars().nth(trigger_pos).unwrap_or(' ');

            // 只有在菜单不可见或者触发位置改变时才更新
            if !self.command_menu.is_visible || self.command_menu.trigger_position != trigger_pos {
                self.command_menu.is_visible = true;
                self.command_menu.trigger_position = trigger_pos;
                self.command_menu.cursor_position = cursor_pos;
                self.command_menu.selected_index = 0;

                match trigger_char {
                    '@' => self.command_menu.menu_type = CommandMenuType::AtCommands,
                    '/' => self.command_menu.menu_type = CommandMenuType::SlashCommands,
                    _ => self.command_menu.menu_type = CommandMenuType::None,
                }
            } else {
                // 只更新光标位置
                self.command_menu.cursor_position = cursor_pos;
            }
        } else {
            self.command_menu.is_visible = false;
            self.command_menu.menu_type = CommandMenuType::None;
        }
    }

    /// 查找指令触发位置（@ 或 / 在单词开头）
    fn find_command_trigger(&self, input: &str) -> Option<usize> {
        let chars: Vec<char> = input.chars().collect();

        // 从后往前查找最近的 @ 或 /
        for (i, &ch) in chars.iter().enumerate().rev() {
            if ch == '@' {
                // @ 指令可以出现在任何位置（前面是空格或开头）
                let is_word_start = i == 0 || chars[i - 1].is_whitespace();

                if is_word_start {
                    // 检查后面的字符（如果有的话）
                    let after_chars = &chars[i + 1..];

                    // 如果后面没有字符，或者后面只有字母、数字、下划线，且长度合理
                    if after_chars.is_empty() ||
                       (after_chars.iter().all(|&c| c.is_alphanumeric() || c == '_') && after_chars.len() <= 10) {
                        return Some(i);
                    }
                }
            } else if ch == '/' {
                // / 指令只能出现在输入框的第一个字符
                if i == 0 {
                    // 检查后面的字符（如果有的话）
                    let after_chars = &chars[i + 1..];

                    // 如果后面没有字符，或者后面只有字母、数字、下划线，且长度合理
                    if after_chars.is_empty() ||
                       (after_chars.iter().all(|&c| c.is_alphanumeric() || c == '_') && after_chars.len() <= 10) {
                        return Some(i);
                    }
                }
            }
        }

        None
    }

    /// 处理指令菜单的键盘输入
    pub fn handle_command_menu_input(&mut self, key: egui::Key) -> bool {
        if !self.command_menu.is_visible {
            return false;
        }

        match key {
            egui::Key::ArrowUp => {
                let max_items = self.get_command_menu_items().len();
                if max_items > 0 {
                    self.command_menu.selected_index =
                        if self.command_menu.selected_index == 0 {
                            max_items - 1
                        } else {
                            self.command_menu.selected_index - 1
                        };
                }
                true
            },
            egui::Key::ArrowDown => {
                let max_items = self.get_command_menu_items().len();
                if max_items > 0 {
                    self.command_menu.selected_index =
                        (self.command_menu.selected_index + 1) % max_items;
                }
                true
            },
            egui::Key::Enter | egui::Key::Tab => {
                self.apply_selected_command_to_input();
                true
            },
            egui::Key::Escape => {
                self.command_menu.is_visible = false;
                true
            },
            _ => false
        }
    }

    /// 获取当前菜单的指令项目
    fn get_command_menu_items(&self) -> Vec<String> {
        match self.command_menu.menu_type {
            CommandMenuType::AtCommands => {
                vec![
                    "@search".to_string(),
                    "@date".to_string(),
                    "@time".to_string(),
                    "@user".to_string(),
                ]
            },
            CommandMenuType::SlashCommands => {
                vec![
                    "/search".to_string(),
                    "/clear".to_string(),
                    "/help".to_string(),
                    "/new".to_string(),
                ]
            },
            CommandMenuType::None => vec![],
        }
    }

    /// 将选中的指令插入到输入框（不执行）
    pub fn apply_selected_command_to_input(&mut self) {
        let items = self.get_command_menu_items();
        if self.command_menu.selected_index < items.len() {
            let selected_command = &items[self.command_menu.selected_index];

            // 替换触发字符和后续文本
            let mut chars: Vec<char> = self.chat_input.chars().collect();
            let trigger_pos = self.command_menu.trigger_position;

            // 找到要替换的范围（从触发位置到下一个空格或结尾）
            let mut end_pos = trigger_pos + 1;
            while end_pos < chars.len() && !chars[end_pos].is_whitespace() {
                end_pos += 1;
            }

            // 替换文本
            chars.splice(trigger_pos..end_pos, selected_command.chars());
            self.chat_input = chars.into_iter().collect();

            // 对于需要参数的slash命令，添加空格
            if selected_command.starts_with('/') && selected_command != "/clear" && selected_command != "/help" && selected_command != "/new" {
                self.chat_input.push(' ');
            }
        }

        // 隐藏菜单，但保持输入框焦点
        self.command_menu.is_visible = false;
        self.command_menu.menu_type = CommandMenuType::None;
        self.should_focus_chat = true; // 确保输入框保持焦点
    }

    /// 设置选中的MCP服务器
    pub fn set_selected_mcp_server(&mut self, server_id: Option<Uuid>) {
        let old_server = self.selected_mcp_server;
        self.selected_mcp_server = server_id;

        // 记录服务器选择变化
        match (old_server, server_id) {
            (None, None) => {
                // 没有变化，不记录
            },
            (None, Some(new_id)) => {
                let server_name = self.server_names.get(&new_id)
                    .cloned()
                    .unwrap_or_else(|| format!("服务器 {}", new_id.to_string().chars().take(8).collect::<String>()));
                log::info!("🎯 AI助手 - 选择MCP服务器: {}", server_name);

                if let Some(capabilities) = self.mcp_server_capabilities.get(&new_id) {
                    log::info!("📋 服务器能力: 工具:{} 资源:{} 提示:{}",
                        capabilities.tools.len(),
                        capabilities.resources.len(),
                        capabilities.prompts.len()
                    );
                }
            },
            (Some(old_id), None) => {
                let old_server_name = self.server_names.get(&old_id)
                    .cloned()
                    .unwrap_or_else(|| format!("服务器 {}", old_id.to_string().chars().take(8).collect::<String>()));
                log::info!("🚫 AI助手 - 取消选择MCP服务器: {}", old_server_name);
            },
            (Some(old_id), Some(new_id)) if old_id != new_id => {
                let old_server_name = self.server_names.get(&old_id)
                    .cloned()
                    .unwrap_or_else(|| format!("服务器 {}", old_id.to_string().chars().take(8).collect::<String>()));
                let new_server_name = self.server_names.get(&new_id)
                    .cloned()
                    .unwrap_or_else(|| format!("服务器 {}", new_id.to_string().chars().take(8).collect::<String>()));
                log::info!("🔄 AI助手 - 切换MCP服务器: {} -> {}", old_server_name, new_server_name);

                if let Some(capabilities) = self.mcp_server_capabilities.get(&new_id) {
                    log::info!("📋 新服务器能力: 工具:{} 资源:{} 提示:{}",
                        capabilities.tools.len(),
                        capabilities.resources.len(),
                        capabilities.prompts.len()
                    );
                }
            },
            _ => {
                // 相同服务器，不记录
            }
        }
    }

    /// 获取选中的MCP服务器
    pub fn get_selected_mcp_server(&self) -> Option<Uuid> {
        self.selected_mcp_server
    }

    /// 检查并记录MCP服务器选择变化（用于UI更新后调用）
    pub fn check_mcp_server_selection_change(&mut self, previous_selection: Option<Uuid>) {
        if previous_selection != self.selected_mcp_server {
            match (previous_selection, self.selected_mcp_server) {
                (None, Some(new_id)) => {
                    let server_name = self.server_names.get(&new_id)
                        .cloned()
                        .unwrap_or_else(|| format!("服务器 {}", new_id.to_string().chars().take(8).collect::<String>()));
                    log::info!("🎯 AI助手 - 用户选择MCP服务器: {}", server_name);

                    if let Some(capabilities) = self.mcp_server_capabilities.get(&new_id) {
                        log::info!("📋 服务器能力: 工具:{} 资源:{} 提示:{}",
                            capabilities.tools.len(),
                            capabilities.resources.len(),
                            capabilities.prompts.len()
                        );
                    }
                },
                (Some(old_id), None) => {
                    let old_server_name = self.server_names.get(&old_id)
                        .cloned()
                        .unwrap_or_else(|| format!("服务器 {}", old_id.to_string().chars().take(8).collect::<String>()));
                    log::info!("🚫 AI助手 - 用户取消选择MCP服务器: {}", old_server_name);
                },
                (Some(old_id), Some(new_id)) if old_id != new_id => {
                    let old_server_name = self.server_names.get(&old_id)
                        .cloned()
                        .unwrap_or_else(|| format!("服务器 {}", old_id.to_string().chars().take(8).collect::<String>()));
                    let new_server_name = self.server_names.get(&new_id)
                        .cloned()
                        .unwrap_or_else(|| format!("服务器 {}", new_id.to_string().chars().take(8).collect::<String>()));
                    log::info!("🔄 AI助手 - 用户切换MCP服务器: {} -> {}", old_server_name, new_server_name);

                    if let Some(capabilities) = self.mcp_server_capabilities.get(&new_id) {
                        log::info!("📋 新服务器能力: 工具:{} 资源:{} 提示:{}",
                            capabilities.tools.len(),
                            capabilities.resources.len(),
                            capabilities.prompts.len()
                        );
                    }
                },
                _ => {
                    // 没有实际变化
                }
            }
        }
    }

    /// 更新MCP服务器能力
    pub fn update_mcp_server_capabilities(&mut self, server_id: Uuid, capabilities: McpServerCapabilities) {
        self.mcp_server_capabilities.insert(server_id, capabilities);
    }

    /// 获取MCP服务器能力
    pub fn get_mcp_server_capabilities(&self, server_id: Uuid) -> Option<&McpServerCapabilities> {
        self.mcp_server_capabilities.get(&server_id)
    }

    /// 移除MCP服务器
    pub fn remove_mcp_server(&mut self, server_id: Uuid) {
        self.mcp_server_capabilities.remove(&server_id);
        self.server_names.remove(&server_id);

        // 如果当前选中的服务器被删除，清除选择
        if self.selected_mcp_server == Some(server_id) {
            self.selected_mcp_server = None;
            log::info!("🔄 当前选中的MCP服务器已被删除，已清除选择: {}", server_id);
        }

        log::info!("🗑️ 已从AI助手中移除MCP服务器: {}", server_id);
    }

    /// 处理工具调用确认
    pub fn approve_tool_calls(&mut self) {
        if let Some(batch) = &mut self.current_tool_call_batch {
            batch.user_approved = true;
            log::info!("用户确认执行 {} 个工具调用", batch.tool_calls.len());

            // 开始执行工具调用
            self.execute_approved_tool_calls();
        }
        self.show_tool_call_confirmation = false;
    }

    /// 拒绝工具调用
    pub fn reject_tool_calls(&mut self) {
        if let Some(batch) = &self.current_tool_call_batch {
            log::info!("用户拒绝执行 {} 个工具调用", batch.tool_calls.len());

            // 创建拒绝消息并添加到聊天中
            let reject_message = ChatMessage {
                id: Uuid::new_v4(),
                role: MessageRole::System,
                content: "用户拒绝了工具调用请求。".to_string(),
                timestamp: Utc::now(),
                attachments: vec![],
                tool_calls: None,
                tool_call_results: None,
            };

            self.chat_messages.push(reject_message.clone());

            // 添加到当前会话
            if let Some(session) = self.chat_sessions.get_mut(self.active_session_idx) {
                session.messages.push(reject_message);
            }
        }

        self.current_tool_call_batch = None;
        self.show_tool_call_confirmation = false;
        self.auto_save_sessions();
    }

    /// 执行已确认的工具调用
    fn execute_approved_tool_calls(&mut self) {
        if let Some(batch) = self.current_tool_call_batch.clone() {
            if !batch.user_approved {
                log::warn!("尝试执行未确认的工具调用");
                return;
            }

            log::info!("开始执行 {} 个已确认的工具调用", batch.tool_calls.len());

            // 创建执行开始的系统消息
            let start_message = ChatMessage {
                id: Uuid::new_v4(),
                role: MessageRole::System,
                content: format!("🔧 开始执行 {} 个工具调用...", batch.tool_calls.len()),
                timestamp: Utc::now(),
                attachments: vec![],
                tool_calls: None,
                tool_call_results: None,
            };

            self.chat_messages.push(start_message.clone());

            // 添加到当前会话
            if let Some(session) = self.chat_sessions.get_mut(self.active_session_idx) {
                session.messages.push(start_message);
            }

            // TODO: 实现实际的MCP工具调用执行
            // 这里需要调用MCP客户端来执行工具
            // 暂时创建一个占位符消息
            let placeholder_message = ChatMessage {
                id: Uuid::new_v4(),
                role: MessageRole::System,
                content: "⏳ 工具执行中，请稍候...".to_string(),
                timestamp: Utc::now(),
                attachments: vec![],
                tool_calls: None,
                tool_call_results: None,
            };

            self.chat_messages.push(placeholder_message.clone());

            // 添加到当前会话
            if let Some(session) = self.chat_sessions.get_mut(self.active_session_idx) {
                session.messages.push(placeholder_message);
            }

            // 清理当前批次
            self.current_tool_call_batch = None;
            self.auto_save_sessions();
        }
    }

    /// 添加工具调用结果
    pub fn add_tool_call_result(&mut self, tool_call_id: String, result: McpToolCallResult) {
        if let Some(batch) = &mut self.current_tool_call_batch {
            batch.results.insert(tool_call_id, result);
        }
    }

    /// 处理Function Call响应
    pub fn handle_function_call_response(&mut self, response: crate::api::ChatResponse) {
        log::info!("处理Function Call响应");

        if let Some(choice) = response.choices.first() {
            if let Some(tool_calls) = &choice.message.tool_calls {
                log::info!("检测到 {} 个工具调用", tool_calls.len());

                // 更新当前消息，添加tool_calls
                if let Some(message_id) = self.streaming_message_id {
                    // 更新chat_messages中的消息
                    for msg in &mut self.chat_messages {
                        if msg.id == message_id {
                            msg.tool_calls = Some(tool_calls.clone());
                            break;
                        }
                    }

                    // 更新chat_sessions中的消息
                    if let Some(session) = self.chat_sessions.get_mut(self.active_session_idx) {
                        for msg in &mut session.messages {
                            if msg.id == message_id {
                                msg.tool_calls = Some(tool_calls.clone());
                                break;
                            }
                        }
                    }
                }

                // 存储待处理的响应
                self.pending_function_call_response = Some(response.clone());

                // 创建工具调用批次并显示确认对话框
                self.create_tool_call_batch_from_response(response);
            }
        }
    }

    /// 从响应创建工具调用批次
    fn create_tool_call_batch_from_response(&mut self, response: crate::api::ChatResponse) {
        if let Some(choice) = response.choices.first() {
            if let Some(tool_calls) = &choice.message.tool_calls {
                let batch_id = Uuid::new_v4();
                let mut pending_calls = Vec::new();
                let tool_calls_len = tool_calls.len(); // 提前获取长度

                for tool_call in tool_calls {
                    // 解析MCP工具调用信息
                    if let Some(mcp_info) = crate::mcp_tools::McpToolConverter::parse_mcp_tool_call(tool_call) {
                        if let Some(server_id) = self.selected_mcp_server {
                            let server_name = self.server_names
                                .get(&server_id)
                                .cloned()
                                .unwrap_or_else(|| format!("服务器 {}", server_id));

                            pending_calls.push(PendingToolCall {
                                tool_call: tool_call.clone(),
                                mcp_info,
                                server_id,
                                server_name,
                            });
                        }
                    }
                }

                if !pending_calls.is_empty() {
                    // 创建工具调用批次
                    let batch = ToolCallBatch {
                        id: batch_id,
                        tool_calls: pending_calls,
                        original_response: response,
                        results: HashMap::new(),
                        user_approved: false,
                    };

                    self.current_tool_call_batch = Some(batch);
                    self.show_tool_call_confirmation = true;

                    log::info!("创建了包含 {} 个工具调用的批次", tool_calls_len);
                }
            }
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<crate::api::ToolCall>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_results: Option<Vec<ToolCallResult>>,
}

impl ChatMessage {
    /// 格式化时间戳为用户友好的格式
    pub fn format_timestamp(&self) -> String {
        use chrono::Local;

        // 转换为本地时间
        let local_time = self.timestamp.with_timezone(&Local::now().timezone());
        let now = Local::now();

        // 计算时间差
        let duration = now.signed_duration_since(local_time);

        if duration.num_seconds() < 60 {
            "刚刚".to_string()
        } else if duration.num_minutes() < 60 {
            format!("{}分钟前", duration.num_minutes())
        } else if duration.num_hours() < 24 {
            format!("{}小时前", duration.num_hours())
        } else if duration.num_days() < 7 {
            format!("{}天前", duration.num_days())
        } else {
            // 超过一周显示具体日期时间
            local_time.format("%m-%d %H:%M").to_string()
        }
    }
}

/// Message role
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum MessageRole {
    User,
    Assistant,
    System,
    SlashCommand, // 新增：用于标识Slash指令消息
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

/// Tool call result
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ToolCallResult {
    pub tool_call_id: String,
    pub result: String,
    pub success: bool,
    pub error: Option<String>,
}

/// 待处理的工具调用
#[derive(Clone, Debug)]
pub struct PendingToolCall {
    pub tool_call: ToolCall,
    pub mcp_info: McpToolCallInfo,
    pub server_id: Uuid,
    pub server_name: String,
}

/// 工具调用批次
#[derive(Clone, Debug)]
pub struct ToolCallBatch {
    pub id: Uuid,
    pub tool_calls: Vec<PendingToolCall>,
    pub original_response: ChatResponse,
    pub results: HashMap<String, McpToolCallResult>,
    pub user_approved: bool,
}

/// Slash command
#[derive(Clone, Debug, PartialEq)]
pub enum SlashCommand {
    Search(String),
    Clear,
    New,
    Help,
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
