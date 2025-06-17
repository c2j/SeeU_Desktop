use anyhow::Result;
use serde_json::{Value, json};
use uuid::Uuid;
use std::collections::HashMap;

use crate::api::{Tool, FunctionDefinition, ToolCall, FunctionCall};

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
#[derive(Debug, Clone)]
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

        if function_name.starts_with("read_resource_") {
            Some(McpToolCallInfo {
                call_type: McpCallType::ReadResource,
                original_name: function_name.strip_prefix("read_resource_").unwrap().to_string(),
                arguments: tool_call.function.arguments.clone(),
                tool_call_id: tool_call.id.clone(),
            })
        } else if function_name.starts_with("get_prompt_") {
            Some(McpToolCallInfo {
                call_type: McpCallType::GetPrompt,
                original_name: function_name.strip_prefix("get_prompt_").unwrap().to_string(),
                arguments: tool_call.function.arguments.clone(),
                tool_call_id: tool_call.id.clone(),
            })
        } else {
            // 直接的工具调用
            Some(McpToolCallInfo {
                call_type: McpCallType::CallTool,
                original_name: function_name.clone(),
                arguments: tool_call.function.arguments.clone(),
                tool_call_id: tool_call.id.clone(),
            })
        }
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
pub struct McpToolExecutor {
    // 这里将来会集成实际的MCP客户端
}

impl McpToolExecutor {
    pub fn new() -> Self {
        Self {}
    }

    /// 执行MCP工具调用（占位符实现）
    pub async fn execute_tool_call(
        &self,
        server_id: Uuid,
        call_info: &McpToolCallInfo,
    ) -> Result<McpToolCallResult> {
        // TODO: 集成实际的MCP客户端调用
        log::info!("执行MCP工具调用: {:?}", call_info);

        // 占位符实现
        let result = match call_info.call_type {
            McpCallType::CallTool => {
                json!({
                    "success": true,
                    "message": format!("工具 {} 执行成功", call_info.original_name),
                    "arguments": call_info.arguments
                })
            }
            McpCallType::ReadResource => {
                json!({
                    "content": format!("资源 {} 的内容", call_info.original_name),
                    "mime_type": "text/plain"
                })
            }
            McpCallType::GetPrompt => {
                json!({
                    "messages": [
                        {
                            "role": "user",
                            "content": format!("提示 {} 的内容", call_info.original_name)
                        }
                    ]
                })
            }
        };

        Ok(McpToolCallResult {
            tool_call_id: call_info.tool_call_id.clone(),
            success: true,
            result,
            error: None,
        })
    }
}
