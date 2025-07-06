use anyhow::Result;
use serde_json::{Value, json};
use uuid::Uuid;

use crate::api::{Tool, FunctionDefinition, ToolCall};

/// MCP工具信息
#[derive(Debug, Clone)]
pub struct McpToolInfo {
    pub name: String,
    pub description: Option<String>,
    pub input_schema: Option<Value>,
}

/// MCP资源信息
#[derive(Debug, Clone)]
pub struct McpResourceInfo {
    pub uri: String,
    pub name: String,
    pub description: Option<String>,
    pub mime_type: Option<String>,
}

/// MCP提示信息
#[derive(Debug, Clone)]
pub struct McpPromptInfo {
    pub name: String,
    pub description: Option<String>,
    pub arguments: Vec<McpPromptArgument>,
}

/// MCP提示参数
#[derive(Debug, Clone, PartialEq)]
pub struct McpPromptArgument {
    pub name: String,
    pub description: Option<String>,
    pub required: bool,
}

/// MCP服务器能力
#[derive(Debug, Clone)]
pub struct McpServerCapabilities {
    pub tools: Vec<McpToolInfo>,
    pub resources: Vec<McpResourceInfo>,
    pub prompts: Vec<McpPromptInfo>,
}

/// MCP工具转换器
pub struct McpToolConverter;

impl McpToolConverter {
    /// 将MCP工具转换为OpenAI Function Calling格式
    pub fn convert_mcp_tools_to_openai(capabilities: &McpServerCapabilities) -> Vec<Tool> {
        let mut tools = Vec::new();

        // 转换工具
        for tool in &capabilities.tools {
            let function_def = FunctionDefinition {
                name: tool.name.clone(),
                description: tool.description.clone(),
                parameters: tool.input_schema.clone(),
            };

            tools.push(Tool {
                tool_type: "function".to_string(),
                function: function_def,
            });
        }

        // 转换资源为工具（读取资源）
        for resource in &capabilities.resources {
            let function_def = FunctionDefinition {
                name: format!("read_resource_{}", Self::sanitize_name(&resource.name)),
                description: Some(format!(
                    "读取资源: {} ({})", 
                    resource.name, 
                    resource.description.as_deref().unwrap_or("无描述")
                )),
                parameters: Some(json!({
                    "type": "object",
                    "properties": {
                        "uri": {
                            "type": "string",
                            "description": "资源URI",
                            "default": resource.uri
                        }
                    },
                    "required": ["uri"]
                })),
            };

            tools.push(Tool {
                tool_type: "function".to_string(),
                function: function_def,
            });
        }

        // 转换提示为工具
        for prompt in &capabilities.prompts {
            let mut properties = json!({});
            let mut required = Vec::new();

            for arg in &prompt.arguments {
                properties[&arg.name] = json!({
                    "type": "string",
                    "description": arg.description.as_deref().unwrap_or("无描述")
                });

                if arg.required {
                    required.push(arg.name.clone());
                }
            }

            let function_def = FunctionDefinition {
                name: format!("get_prompt_{}", Self::sanitize_name(&prompt.name)),
                description: Some(format!(
                    "获取提示: {} ({})", 
                    prompt.name, 
                    prompt.description.as_deref().unwrap_or("无描述")
                )),
                parameters: Some(json!({
                    "type": "object",
                    "properties": properties,
                    "required": required
                })),
            };

            tools.push(Tool {
                tool_type: "function".to_string(),
                function: function_def,
            });
        }

        tools
    }

    /// 清理名称，确保符合函数名规范
    fn sanitize_name(name: &str) -> String {
        name.chars()
            .map(|c| if c.is_alphanumeric() || c == '_' { c } else { '_' })
            .collect::<String>()
            .trim_matches('_')
            .to_string()
    }

    /// 解析工具调用，确定是否为MCP工具
    pub fn parse_mcp_tool_call(tool_call: &ToolCall) -> Option<McpToolCallInfo> {
        let function_name = &tool_call.function.name;

        log::info!("🔍 解析MCP工具调用:");
        log::info!("  - 调用ID: {}", tool_call.id);
        log::info!("  - 函数名称: {}", function_name);
        log::info!("  - 函数参数: {}", tool_call.function.arguments);

        let mcp_info = if function_name.starts_with("read_resource_") {
            let original_name = function_name.strip_prefix("read_resource_").unwrap().to_string();
            log::info!("  - 识别为资源读取调用: {}", original_name);
            Some(McpToolCallInfo {
                call_type: McpCallType::ReadResource,
                original_name,
                arguments: tool_call.function.arguments.clone(),
                tool_call_id: tool_call.id.clone(),
            })
        } else if function_name.starts_with("get_prompt_") {
            let original_name = function_name.strip_prefix("get_prompt_").unwrap().to_string();
            log::info!("  - 识别为提示获取调用: {}", original_name);
            Some(McpToolCallInfo {
                call_type: McpCallType::GetPrompt,
                original_name,
                arguments: tool_call.function.arguments.clone(),
                tool_call_id: tool_call.id.clone(),
            })
        } else {
            // 直接的工具调用
            log::info!("  - 识别为直接工具调用: {}", function_name);
            Some(McpToolCallInfo {
                call_type: McpCallType::CallTool,
                original_name: function_name.clone(),
                arguments: tool_call.function.arguments.clone(),
                tool_call_id: tool_call.id.clone(),
            })
        };

        if mcp_info.is_some() {
            log::info!("✅ MCP工具调用解析成功");
        } else {
            log::warn!("❌ MCP工具调用解析失败");
        }

        mcp_info
    }
}

/// MCP工具调用信息
#[derive(Debug, Clone)]
pub struct McpToolCallInfo {
    pub call_type: McpCallType,
    pub original_name: String,
    pub arguments: String,
    pub tool_call_id: String,
}

/// MCP调用类型
#[derive(Debug, Clone)]
pub enum McpCallType {
    CallTool,
    ReadResource,
    GetPrompt,
}

/// MCP工具调用结果
#[derive(Debug, Clone)]
pub struct McpToolCallResult {
    pub tool_call_id: String,
    pub success: bool,
    pub result: Value,
    pub error: Option<String>,
}

/// MCP工具执行器
#[derive(Clone)]
pub struct McpToolExecutor {
    // 这里将来会集成实际的MCP客户端
    // 目前使用占位符实现，但会记录详细的执行信息
}

impl McpToolExecutor {
    pub fn new() -> Self {
        Self {}
    }

    /// 执行MCP工具调用
    /// 注意：这是一个占位符实现，实际的MCP客户端集成需要在主应用程序中处理
    pub async fn execute_tool_call(
        &self,
        server_id: Uuid,
        call_info: &McpToolCallInfo,
    ) -> Result<McpToolCallResult> {
        log::info!("🔧 开始执行MCP工具调用:");
        log::info!("  - 服务器ID: {}", server_id);
        log::info!("  - 调用类型: {:?}", call_info.call_type);
        log::info!("  - 工具名称: {}", call_info.original_name);
        log::info!("  - 调用ID: {}", call_info.tool_call_id);
        log::info!("  - 参数JSON: {}", call_info.arguments);

        // 模拟一些处理时间
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

        // 创建模拟结果
        let result = match call_info.call_type {
            McpCallType::CallTool => {
                json!({
                    "success": true,
                    "message": format!("工具 {} 执行成功（模拟）", call_info.original_name),
                    "tool_name": call_info.original_name,
                    "server_id": server_id.to_string(),
                    "arguments": call_info.arguments,
                    "timestamp": chrono::Utc::now().to_rfc3339()
                })
            }
            McpCallType::ReadResource => {
                json!({
                    "content": format!("这是资源 {} 的模拟内容。\n\n实际使用时，这里会包含真实的资源数据。", call_info.original_name),
                    "mime_type": "text/plain",
                    "resource_name": call_info.original_name,
                    "server_id": server_id.to_string(),
                    "timestamp": chrono::Utc::now().to_rfc3339()
                })
            }
            McpCallType::GetPrompt => {
                json!({
                    "messages": [
                        {
                            "role": "user",
                            "content": format!("这是提示 {} 的模拟内容。\n\n实际使用时，这里会包含真实的提示模板。", call_info.original_name)
                        }
                    ],
                    "prompt_name": call_info.original_name,
                    "server_id": server_id.to_string(),
                    "timestamp": chrono::Utc::now().to_rfc3339()
                })
            }
        };

        let tool_result = McpToolCallResult {
            tool_call_id: call_info.tool_call_id.clone(),
            success: true,
            result: result.clone(),
            error: None,
        };

        // 记录执行结果
        log::info!("✅ MCP工具调用执行完成:");
        log::info!("  - 调用ID: {}", tool_result.tool_call_id);
        log::info!("  - 执行状态: {}", if tool_result.success { "成功" } else { "失败" });
        if let Some(error) = &tool_result.error {
            log::error!("  - 错误信息: {}", error);
        }
        log::info!("  - 结果预览: {}",
            serde_json::to_string(&result).unwrap_or_default().chars().take(100).collect::<String>()
        );

        Ok(tool_result)
    }
}
