use anyhow::{Result, anyhow};
use futures::stream::StreamExt;
use reqwest::{Client, header};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use bytes::Bytes;
use crate::state::{AISettings, MessageRole};

/// API错误类型
#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("网络请求错误: {0}")]
    RequestError(#[from] reqwest::Error),

    #[error("JSON解析错误: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("API返回错误: {0}")]
    ApiResponseError(String),

    #[error("流式响应错误: {0}")]
    StreamError(String),
}

/// API服务
pub struct ApiService {
    client: Client,
}

/// 聊天消息请求 (OpenAI compatible)
#[derive(Debug, Clone, Serialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
}

/// 聊天请求 (OpenAI compatible)
#[derive(Debug, Serialize)]
pub struct ChatRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    pub stream: bool,
    pub temperature: f32,
    pub max_tokens: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<Tool>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<String>,
}

/// Models list request response (OpenAI compatible)
#[derive(Debug, Deserialize)]
pub struct ModelsResponse {
    pub data: Vec<Model>,
}

#[derive(Debug, Deserialize)]
pub struct Model {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub owned_by: String,
}

/// 聊天响应消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatResponseMessage {
    pub role: Option<String>,  // 可选，某些服务可能不返回
    pub content: Option<String>,  // 可选，当有tool_calls时可能为空
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,
}

/// 聊天响应 (OpenAI compatible)
/// 为了增强兼容性，只有choices字段是必需的，其他字段都是可选的
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatResponse {
    pub id: Option<String>,
    pub object: Option<String>,
    pub created: Option<u64>,
    pub model: Option<String>,
    pub choices: Vec<Choice>,  // 唯一必需的字段
    pub usage: Option<Usage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Choice {
    pub index: Option<u32>,  // 可选，某些服务可能不返回
    pub message: ChatResponseMessage,  // 必需
    pub finish_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

/// 流式聊天响应 (OpenAI compatible)
/// 为了增强兼容性，只有choices字段是必需的，其他字段都是可选的
#[derive(Debug, Deserialize)]
pub struct ChatStreamResponse {
    pub id: Option<String>,
    pub object: Option<String>,
    pub created: Option<u64>,
    pub model: Option<String>,
    pub choices: Vec<StreamChoice>,  // 唯一必需的字段
}

#[derive(Debug, Deserialize)]
pub struct StreamChoice {
    pub index: Option<u32>,  // 可选，某些服务可能不返回
    pub delta: Delta,  // 必需
    pub finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Delta {
    pub role: Option<String>,
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,
}

/// Tool definition for Function Calling (OpenAI compatible)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    #[serde(rename = "type")]
    pub tool_type: String,  // "function"
    pub function: FunctionDefinition,
}

/// Function definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionDefinition {
    pub name: String,
    pub description: Option<String>,
    pub parameters: Option<Value>,  // JSON Schema
}

/// Tool call from LLM response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    #[serde(rename = "type")]
    pub call_type: String,  // "function"
    pub function: FunctionCall,
}

/// Function call details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionCall {
    pub name: String,
    pub arguments: String,  // JSON string
}

impl ApiService {
    /// 创建新的API服务
    pub fn new() -> Self {
        let client = Client::builder()
            .build()
            .expect("Failed to create HTTP client");

        Self { client }
    }

    /// 获取可用模型列表 (OpenAI compatible)
    pub async fn get_models(&self, settings: &AISettings) -> Result<Vec<String>> {
        let url = settings.get_models_url();
        log::info!("Fetching models from: {}", url);

        let mut request = self.client.get(&url)
            .header(header::CONTENT_TYPE, "application/json");

        // 添加Authorization头
        if !settings.api_key.is_empty() {
            request = request.header("Authorization", format!("Bearer {}", settings.api_key));
        }

        let response = request.send().await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow!(ApiError::ApiResponseError(error_text)));
        }

        let models_response: ModelsResponse = response.json().await?;
        let models = models_response.data.into_iter().map(|m| m.id).collect();

        Ok(models)
    }

    /// 发送聊天请求并返回完整响应 (包括tool_calls)
    pub async fn send_chat_with_tools_full_response(&self, settings: &AISettings, messages: Vec<(MessageRole, String)>, tools: Option<Vec<Tool>>) -> Result<ChatResponse> {
        // 转换消息格式
        let chat_messages = messages.iter().map(|(role, content)| {
            ChatMessage {
                role: match role {
                    MessageRole::User => "user".to_string(),
                    MessageRole::Assistant => "assistant".to_string(),
                    MessageRole::System => "system".to_string(),
                    MessageRole::SlashCommand => "user".to_string(),
                },
                content: content.clone(),
                tool_calls: None,
                tool_call_id: None,
            }
        }).collect::<Vec<_>>();

        let url = settings.get_chat_url();
        log::info!("📤 发送请求到: {}", url);
        log::info!("🤖 使用模型: {}", settings.model);
        log::info!("💬 消息数量: {}", chat_messages.len());

        // 记录tools信息
        if let Some(ref tools_vec) = tools {
            log::info!("🛠️ 发送工具定义给LLM:");
            log::info!("  - 工具数量: {}", tools_vec.len());
            for (index, tool) in tools_vec.iter().enumerate() {
                log::info!("  {}. 工具名称: {}", index + 1, tool.function.name);
                log::info!("     工具描述: {}", tool.function.description.as_deref().unwrap_or("无描述"));
                if let Some(params) = &tool.function.parameters {
                    if let Some(properties) = params.get("properties") {
                        if let Some(props_obj) = properties.as_object() {
                            log::info!("     参数数量: {}", props_obj.len());
                            for (param_name, _) in props_obj.iter() {
                                log::info!("       - {}", param_name);
                            }
                        }
                    }
                }
            }
        } else {
            log::info!("🚫 未发送工具定义 (无选中的MCP服务器)");
        }

        // 创建请求
        let request = ChatRequest {
            model: settings.model.clone(),
            messages: chat_messages,
            stream: false,
            temperature: settings.temperature,
            max_tokens: settings.max_tokens,
            tools,
            tool_choice: None,
        };

        // 打印完整的请求JSON信息
        log::info!("📋 完整请求JSON:");
        match serde_json::to_string_pretty(&request) {
            Ok(request_json) => {
                for line in request_json.lines() {
                    log::info!("  {}", line);
                }
            }
            Err(e) => {
                log::error!("  Failed to serialize request to JSON: {}", e);
            }
        }

        let mut request_builder = self.client.post(&url)
            .header(header::CONTENT_TYPE, "application/json");

        // 添加Authorization头
        if !settings.api_key.is_empty() {
            request_builder = request_builder.header("Authorization", format!("Bearer {}", settings.api_key));
        }

        let response = request_builder.json(&request).send().await?;

        // 检查响应状态
        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow!(ApiError::ApiResponseError(error_text)));
        }

        // 解析响应
        let chat_response: ChatResponse = response.json().await?;

        // 打印完整的响应JSON信息
        log::info!("📥 完整响应JSON:");
        match serde_json::to_string_pretty(&chat_response) {
            Ok(response_json) => {
                for line in response_json.lines() {
                    log::info!("  {}", line);
                }
            }
            Err(e) => {
                log::error!("  Failed to serialize response to JSON: {}", e);
            }
        }

        // 检查并警告缺失的字段
        if chat_response.id.is_none() {
            log::warn!("Response missing 'id' field");
        }
        if chat_response.object.is_none() {
            log::warn!("Response missing 'object' field");
        }
        if chat_response.created.is_none() {
            log::warn!("Response missing 'created' field");
        }
        if chat_response.model.is_none() {
            log::warn!("Response missing 'model' field");
        } else {
            log::info!("Response from model: {}", chat_response.model.as_ref().unwrap());
        }

        if chat_response.choices.is_empty() {
            return Err(anyhow!(ApiError::ApiResponseError("No choices in response".to_string())));
        }

        Ok(chat_response)
    }

    /// 发送聊天请求 (OpenAI compatible)
    pub async fn send_chat(&self, settings: &AISettings, messages: Vec<(MessageRole, String)>) -> Result<String> {
        self.send_chat_with_tools(settings, messages, None).await
    }

    /// 发送聊天请求，支持工具调用 (OpenAI compatible)
    pub async fn send_chat_with_tools(&self, settings: &AISettings, messages: Vec<(MessageRole, String)>, tools: Option<Vec<Tool>>) -> Result<String> {
        // 转换消息格式
        let chat_messages = messages.iter().map(|(role, content)| {
            ChatMessage {
                role: match role {
                    MessageRole::User => "user".to_string(),
                    MessageRole::Assistant => "assistant".to_string(),
                    MessageRole::System => "system".to_string(),
                    MessageRole::SlashCommand => "user".to_string(), // Slash指令作为用户消息处理（虽然实际上不应该到达这里）
                },
                content: content.clone(),
                tool_calls: None,
                tool_call_id: None,
            }
        }).collect::<Vec<_>>();

        let url = settings.get_chat_url();
        log::info!("📤 发送请求到: {}", url);
        log::info!("🤖 使用模型: {}", settings.model);
        log::info!("💬 消息数量: {}", chat_messages.len());

        // 记录tools信息
        if let Some(ref tools_vec) = tools {
            log::info!("🛠️ 发送工具定义给LLM:");
            log::info!("  - 工具数量: {}", tools_vec.len());
            for (index, tool) in tools_vec.iter().enumerate() {
                log::info!("  {}. 工具名称: {}", index + 1, tool.function.name);
                log::info!("     工具描述: {}", tool.function.description.as_deref().unwrap_or("无描述"));
                if let Some(params) = &tool.function.parameters {
                    if let Some(properties) = params.get("properties") {
                        if let Some(props_obj) = properties.as_object() {
                            log::info!("     参数数量: {}", props_obj.len());
                            for (param_name, _) in props_obj.iter() {
                                log::info!("       - {}", param_name);
                            }
                        }
                    }
                }
            }
        } else {
            log::info!("🚫 未发送工具定义 (无选中的MCP服务器)");
        }

        // 创建请求
        let request = ChatRequest {
            model: settings.model.clone(),
            messages: chat_messages,
            stream: false,
            temperature: settings.temperature,
            max_tokens: settings.max_tokens,
            tools,
            tool_choice: None,
        };

        // 打印完整的请求JSON信息
        log::info!("📋 完整请求JSON:");
        match serde_json::to_string_pretty(&request) {
            Ok(request_json) => {
                for line in request_json.lines() {
                    log::info!("  {}", line);
                }
            }
            Err(e) => {
                log::error!("  Failed to serialize request to JSON: {}", e);
            }
        }

        let mut request_builder = self.client.post(&url)
            .header(header::CONTENT_TYPE, "application/json");

        // 添加Authorization头
        if !settings.api_key.is_empty() {
            request_builder = request_builder.header("Authorization", format!("Bearer {}", settings.api_key));
        }

        let response = request_builder.json(&request).send().await?;

        // 检查响应状态
        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow!(ApiError::ApiResponseError(error_text)));
        }

        // 解析响应
        let chat_response: ChatResponse = response.json().await?;

        // 打印完整的响应JSON信息
        log::info!("📥 完整响应JSON:");
        match serde_json::to_string_pretty(&chat_response) {
            Ok(response_json) => {
                for line in response_json.lines() {
                    log::info!("  {}", line);
                }
            }
            Err(e) => {
                log::error!("  Failed to serialize response to JSON: {}", e);
            }
        }

        // 检查并警告缺失的字段
        if chat_response.id.is_none() {
            log::warn!("Response missing 'id' field");
        }
        if chat_response.object.is_none() {
            log::warn!("Response missing 'object' field");
        }
        if chat_response.created.is_none() {
            log::warn!("Response missing 'created' field");
        }
        if chat_response.model.is_none() {
            log::warn!("Response missing 'model' field");
        } else {
            log::info!("Response from model: {}", chat_response.model.as_ref().unwrap());
        }

        if let Some(choice) = chat_response.choices.first() {
            // 检查choice的index字段
            if choice.index.is_none() {
                log::warn!("Choice missing 'index' field");
            }

            // 检查message的role字段
            if choice.message.role.is_none() {
                log::warn!("Message missing 'role' field");
            }

            Ok(choice.message.content.clone().unwrap_or_default())
        } else {
            Err(anyhow!(ApiError::ApiResponseError("No choices in response".to_string())))
        }
    }

    /// 发送流式聊天请求 (OpenAI compatible)
    pub async fn send_chat_stream(
        &self,
        settings: &AISettings,
        messages: Vec<(MessageRole, String)>,
        callback: impl Fn(String) + Send + Sync + 'static,
    ) -> Result<()> {
        self.send_chat_stream_with_tools(settings, messages, None, callback).await
    }

    /// 发送流式聊天请求，支持工具调用 (OpenAI compatible)
    pub async fn send_chat_stream_with_tools(
        &self,
        settings: &AISettings,
        messages: Vec<(MessageRole, String)>,
        tools: Option<Vec<Tool>>,
        callback: impl Fn(String) + Send + Sync + 'static,
    ) -> Result<()> {
        // 转换消息格式
        let chat_messages = messages.iter().map(|(role, content)| {
            ChatMessage {
                role: match role {
                    MessageRole::User => "user".to_string(),
                    MessageRole::Assistant => "assistant".to_string(),
                    MessageRole::System => "system".to_string(),
                    MessageRole::SlashCommand => "user".to_string(), // Slash指令作为用户消息处理（虽然实际上不应该到达这里）
                },
                content: content.clone(),
                tool_calls: None,
                tool_call_id: None,
            }
        }).collect::<Vec<_>>();

        let url = settings.get_chat_url();
        log::info!("📤 发送流式请求到: {}", url);
        log::info!("🤖 使用模型: {}", settings.model);
        log::info!("💬 消息数量: {}", chat_messages.len());

        // 记录工具信息
        if let Some(ref tools_vec) = tools {
            log::info!("🛠️ 发送工具定义给LLM:");
            log::info!("  - 工具数量: {}", tools_vec.len());
            for (i, tool) in tools_vec.iter().enumerate() {
                log::info!("  {}. 工具名称: {}", i + 1, tool.function.name);
                log::info!("     工具描述: {}", tool.function.description.as_deref().unwrap_or("无描述"));
                if let Some(params) = &tool.function.parameters {
                    if let Some(properties) = params.get("properties").and_then(|p| p.as_object()) {
                        log::info!("     参数数量: {}", properties.len());
                        for param_name in properties.keys() {
                            log::info!("       - {}", param_name);
                        }
                    }
                }
            }
        } else {
            log::info!("🚫 未发送工具定义 (无选中的MCP服务器)");
        }

        // 创建请求
        let request = ChatRequest {
            model: settings.model.clone(),
            messages: chat_messages.clone(),
            stream: true,
            temperature: settings.temperature,
            max_tokens: settings.max_tokens,
            tools: tools.clone(),
            tool_choice: if tools.is_some() { Some("auto".to_string()) } else { None },
        };

        // 打印完整的请求JSON信息
        log::info!("📋 完整请求JSON:");
        match serde_json::to_string_pretty(&request) {
            Ok(request_json) => {
                for line in request_json.lines() {
                    log::info!("  {}", line);
                }
            }
            Err(e) => {
                log::error!("  Failed to serialize request to JSON: {}", e);
            }
        }

        let mut request_builder = self.client.post(&url)
            .header(header::CONTENT_TYPE, "application/json");

        // 添加Authorization头
        if !settings.api_key.is_empty() {
            request_builder = request_builder.header("Authorization", format!("Bearer {}", settings.api_key));
        }

        let response = request_builder.json(&request).send().await?;

        // 检查响应状态
        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow!(ApiError::ApiResponseError(error_text)));
        }

        // 获取流式响应
        let stream = response.bytes_stream();
        let callback = Arc::new(callback);
        let mut content = String::new();

        self.handle_stream(stream, callback, &mut content).await
    }

    /// 处理流式响应 (OpenAI compatible)
    async fn handle_stream(
        &self,
        mut stream: impl futures::Stream<Item = reqwest::Result<Bytes>> + Unpin,
        callback: Arc<impl Fn(String) + Send + Sync + 'static>,
        content: &mut String,
    ) -> Result<()> {
        while let Some(item) = stream.next().await {
            match item {
                Ok(bytes) => {
                    let data = String::from_utf8_lossy(&bytes);
                    for line in data.lines() {
                        if line.trim().is_empty() {
                            continue;
                        }

                        // OpenAI使用SSE格式，以"data: "开头
                        if let Some(json_str) = line.strip_prefix("data: ") {
                            if json_str.trim() == "[DONE]" {
                                return Ok(());
                            }

                            match serde_json::from_str::<ChatStreamResponse>(json_str) {
                                Ok(response) => {
                                    // 检查并警告缺失的字段（只在第一次收到响应时警告）
                                    if content.is_empty() {
                                        if response.id.is_none() {
                                            log::warn!("Stream response missing 'id' field");
                                        }
                                        if response.object.is_none() {
                                            log::warn!("Stream response missing 'object' field");
                                        }
                                        if response.created.is_none() {
                                            log::warn!("Stream response missing 'created' field");
                                        }
                                        if response.model.is_none() {
                                            log::warn!("Stream response missing 'model' field");
                                        }
                                    }

                                    if let Some(choice) = response.choices.first() {
                                        if let Some(delta_content) = &choice.delta.content {
                                            content.push_str(delta_content);
                                            callback(content.clone());
                                        }

                                        if choice.finish_reason.is_some() {
                                            return Ok(());
                                        }
                                    }
                                },
                                Err(e) => {
                                    log::error!("Failed to parse stream response: {} - Line: {}", e, json_str);
                                }
                            }
                        }
                    }
                },
                Err(e) => {
                    log::error!("Stream error: {}", e);
                    return Err(anyhow!(ApiError::StreamError(e.to_string())));
                }
            }
        }
        Ok(())
    }
}
