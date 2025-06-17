use anyhow::Result;
use uuid::Uuid;
use std::collections::HashMap;
use serde_json::Value;

use crate::api::{ToolCall, ChatResponse};
use crate::mcp_tools::{
    McpToolConverter, McpToolCallInfo, McpToolCallResult, McpServerCapabilities,
    McpToolExecutor, McpCallType
};
use crate::state::{AIAssistState, PendingToolCall, ToolCallBatch};

/// MCP集成管理器
pub struct McpIntegrationManager {
    tool_executor: McpToolExecutor,
    server_names: HashMap<Uuid, String>,
}

impl McpIntegrationManager {
    pub fn new() -> Self {
        Self {
            tool_executor: McpToolExecutor::new(),
            server_names: HashMap::new(),
        }
    }

    /// 更新服务器名称映射
    pub fn update_server_name(&mut self, server_id: Uuid, name: String) {
        self.server_names.insert(server_id, name);
    }

    /// 处理LLM响应中的工具调用
    pub fn handle_tool_calls_response(
        &self,
        state: &mut AIAssistState,
        response: ChatResponse,
    ) -> Result<bool> {
        if let Some(choice) = response.choices.first() {
            if let Some(tool_calls) = &choice.message.tool_calls {
                if !tool_calls.is_empty() {
                    // 有工具调用，创建待处理批次
                    let batch_id = Uuid::new_v4();
                    let mut pending_calls = Vec::new();

                    for tool_call in tool_calls {
                        if let Some(mcp_info) = McpToolConverter::parse_mcp_tool_call(tool_call) {
                            if let Some(server_id) = state.selected_mcp_server {
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

                        state.current_tool_call_batch = Some(batch);
                        state.show_tool_call_confirmation = true;
                        return Ok(true); // 表示有工具调用需要处理
                    }
                }
            }
        }

        Ok(false) // 没有工具调用
    }

    /// 执行已确认的工具调用
    pub async fn execute_approved_tool_calls(
        &self,
        state: &mut AIAssistState,
    ) -> Result<Vec<McpToolCallResult>> {
        let mut results = Vec::new();

        if let Some(batch) = &state.current_tool_call_batch {
            if batch.user_approved {
                for pending_call in &batch.tool_calls {
                    log::info!("执行工具调用: {} on server {}", 
                              pending_call.tool_call.function.name, 
                              pending_call.server_name);

                    match self.tool_executor.execute_tool_call(
                        pending_call.server_id,
                        &pending_call.mcp_info,
                    ).await {
                        Ok(result) => {
                            log::info!("工具调用成功: {}", result.tool_call_id);
                            results.push(result);
                        }
                        Err(e) => {
                            log::error!("工具调用失败: {}", e);
                            let error_result = McpToolCallResult {
                                tool_call_id: pending_call.tool_call.id.clone(),
                                success: false,
                                result: Value::Null,
                                error: Some(e.to_string()),
                            };
                            results.push(error_result);
                        }
                    }
                }
            }
        }

        Ok(results)
    }

    /// 格式化工具调用结果为消息
    pub fn format_tool_results_as_message(&self, results: &[McpToolCallResult]) -> String {
        let mut message = String::new();
        message.push_str("工具调用结果:\n\n");

        for (index, result) in results.iter().enumerate() {
            message.push_str(&format!("{}. 工具调用 ID: {}\n", index + 1, result.tool_call_id));
            
            if result.success {
                message.push_str("   状态: ✅ 成功\n");
                message.push_str(&format!("   结果: {}\n", 
                    serde_json::to_string_pretty(&result.result).unwrap_or_default()));
            } else {
                message.push_str("   状态: ❌ 失败\n");
                if let Some(error) = &result.error {
                    message.push_str(&format!("   错误: {}\n", error));
                }
            }
            message.push('\n');
        }

        message
    }

    /// 检查是否有待执行的工具调用
    pub fn has_pending_approved_tool_calls(&self, state: &AIAssistState) -> bool {
        if let Some(batch) = &state.current_tool_call_batch {
            batch.user_approved && !batch.tool_calls.is_empty()
        } else {
            false
        }
    }

    /// 清理已完成的工具调用批次
    pub fn clear_completed_batch(&self, state: &mut AIAssistState) {
        state.current_tool_call_batch = None;
        state.show_tool_call_confirmation = false;
    }

    /// 获取可用的MCP服务器列表（从外部注入）
    pub fn get_available_servers(&self) -> Vec<(Uuid, String)> {
        self.server_names.iter().map(|(id, name)| (*id, name.clone())).collect()
    }

    /// 更新AI助手状态中的MCP服务器能力
    pub fn update_server_capabilities(
        &self,
        state: &mut AIAssistState,
        server_id: Uuid,
        capabilities: McpServerCapabilities,
    ) {
        state.update_mcp_server_capabilities(server_id, capabilities);

        // 同时更新服务器名称
        if let Some(server_name) = self.server_names.get(&server_id) {
            state.server_names.insert(server_id, server_name.clone());
        }
    }
}

/// MCP集成事件
#[derive(Debug, Clone)]
pub enum McpIntegrationEvent {
    ToolCallsDetected(Uuid),
    ToolCallsApproved(Uuid),
    ToolCallsRejected(Uuid),
    ToolCallCompleted(Uuid, String, bool),
    BatchCompleted(Uuid),
}

/// MCP集成配置
#[derive(Debug, Clone)]
pub struct McpIntegrationConfig {
    pub auto_approve_safe_tools: bool,
    pub timeout_seconds: u64,
    pub max_concurrent_calls: usize,
}

impl Default for McpIntegrationConfig {
    fn default() -> Self {
        Self {
            auto_approve_safe_tools: false,
            timeout_seconds: 30,
            max_concurrent_calls: 5,
        }
    }
}
