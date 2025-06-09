use anyhow::{Result, anyhow};
use futures::stream::StreamExt;
use reqwest::{Client, header};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use tokio_stream::wrappers::ReceiverStream;
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

/// 聊天消息请求
#[derive(Debug, Serialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

/// 聊天请求 (Ollama API 格式)
#[derive(Debug, Serialize)]
pub struct ChatRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    pub stream: bool,
    pub options: ChatOptions,
}

/// 聊天选项 (Ollama API 格式)
#[derive(Debug, Serialize)]
pub struct ChatOptions {
    pub temperature: f32,
    pub num_predict: u32,
}

/// 聊天响应 (Ollama API 格式)
#[derive(Debug, Deserialize)]
pub struct ChatResponse {
    pub model: String,
    pub created_at: String,
    pub message: ChatResponseMessage,
    pub done: bool,
}

/// 聊天响应消息
#[derive(Debug, Deserialize)]
pub struct ChatResponseMessage {
    pub role: String,
    pub content: String,
}

/// 流式聊天响应 (Ollama API 格式)
/// 与普通响应相同，但会分块返回
#[derive(Debug, Deserialize)]
pub struct ChatStreamResponse {
    pub model: String,
    pub created_at: String,
    pub message: ChatStreamMessage,
    pub done: bool,
}

/// 流式聊天消息
#[derive(Debug, Deserialize)]
pub struct ChatStreamMessage {
    pub role: String,
    pub content: String,
}

impl ApiService {
    /// 创建新的API服务
    pub fn new() -> Self {
        let client = Client::builder()
            .build()
            .expect("Failed to create HTTP client");

        Self { client }
    }

    /// 发送聊天请求
    pub async fn send_chat(&self, settings: &AISettings, messages: Vec<(MessageRole, String)>) -> Result<String> {
        // 转换消息格式
        let chat_messages = messages.iter().map(|(role, content)| {
            ChatMessage {
                role: match role {
                    MessageRole::User => "user".to_string(),
                    MessageRole::Assistant => "assistant".to_string(),
                    MessageRole::System => "system".to_string(),
                },
                content: content.clone(),
            }
        }).collect::<Vec<_>>();

        // 创建请求
        let request = ChatRequest {
            model: settings.model.clone(),
            messages: chat_messages,
            stream: false,
            options: ChatOptions {
                temperature: settings.temperature,
                num_predict: settings.max_tokens,
            },
        };

        // 记录请求信息
        log::info!("Sending request to: {}", settings.api_url);

        // 发送请求
        let response = self.client.post(&settings.api_url)
            .header(header::CONTENT_TYPE, "application/json")
            .header("Authorization", format!("Bearer {}", settings.api_key))
            .json(&request)
            .send()
            .await?;

        // 检查响应状态
        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow!(ApiError::ApiResponseError(error_text)));
        }

        // 解析响应
        let chat_response: ChatResponse = response.json().await?;

        // 记录响应信息
        log::info!("Response from model: {}", chat_response.model);

        // 提取回复内容
        Ok(chat_response.message.content.clone())
    }

    /// 发送流式聊天请求
    pub async fn send_chat_stream(
        &self,
        settings: &AISettings,
        messages: Vec<(MessageRole, String)>,
        callback: impl Fn(String) + Send + Sync + 'static,
    ) -> Result<()> {
        // 转换消息格式
        let chat_messages = messages.iter().map(|(role, content)| {
            ChatMessage {
                role: match role {
                    MessageRole::User => "user".to_string(),
                    MessageRole::Assistant => "assistant".to_string(),
                    MessageRole::System => "system".to_string(),
                },
                content: content.clone(),
            }
        }).collect::<Vec<_>>();

        // 创建请求
        let request = ChatRequest {
            model: settings.model.clone(),
            messages: chat_messages,
            stream: true,
            options: ChatOptions {
                temperature: settings.temperature,
                num_predict: settings.max_tokens,
            },
        };

        // 记录请求信息
        log::info!("Sending streaming request to: {}", settings.api_url);

        // 发送请求
        let response = self.client.post(&settings.api_url)
            .header(header::CONTENT_TYPE, "application/json")
            .header("Authorization", format!("Bearer {}", settings.api_key))
            .json(&request)
            .send()
            .await?;

        // 检查响应状态
        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow!(ApiError::ApiResponseError(error_text)));
        }

        // 获取流式响应
        let stream = response.bytes_stream();
        let callback = Arc::new(callback);

        // 直接处理流式响应，不使用tokio::spawn
        let mut content = String::new();

        let mut stream = stream;
        while let Some(item) = stream.next().await {
            match item {
                Ok(bytes) => {
                    // 解析SSE数据
                    let data = String::from_utf8_lossy(&bytes);
                    for line in data.lines() {
                        // Ollama API 不使用 SSE 格式，直接解析 JSON
                        match serde_json::from_str::<ChatStreamResponse>(&line) {
                            Ok(response) => {
                                // 如果是最后一个消息，标记完成
                                if response.done {
                                    return Ok(());
                                }

                                // 获取消息内容
                                let message_content = &response.message.content;

                                // 在流式模式下，Ollama 返回的是增量内容
                                content.push_str(message_content);
                                callback(content.clone());
                            },
                            Err(e) => {
                                log::error!("Failed to parse stream response: {} - Line: {}", e, line);
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
