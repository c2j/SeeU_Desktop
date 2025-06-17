# MCP Function Calling 实现文档

## 概述

本文档描述了在SeeU Desktop AI助手中实现的MCP (Model Context Protocol) Function Calling功能。该功能允许AI助手调用MCP服务器提供的工具、资源和提示。

## 实现的功能

### 1. 核心数据结构扩展

#### API层面 (crates/aiAssist/src/api.rs)
- 扩展了 `ChatRequest` 支持 `tools` 和 `tool_choice` 字段
- 扩展了 `ChatMessage` 支持 `tool_calls` 和 `tool_call_id` 字段
- 扩展了 `ChatResponseMessage` 支持 `tool_calls` 字段
- 添加了 `Tool`, `FunctionDefinition`, `ToolCall`, `FunctionCall` 等OpenAI兼容的数据结构
- 新增了 `send_chat_with_tools_full_response` 方法返回完整响应（包括tool_calls）

#### MCP工具集成 (crates/aiAssist/src/mcp_tools.rs)
- `McpToolConverter`: 将MCP服务器能力转换为OpenAI Function Calling格式
- `McpToolExecutor`: 执行MCP工具调用（当前为占位符实现）
- `McpToolCallInfo`: 解析和存储MCP工具调用信息
- 支持三种MCP调用类型：
  - `CallTool`: 直接工具调用
  - `ReadResource`: 读取资源
  - `GetPrompt`: 获取提示

### 2. 状态管理扩展

#### AI助手状态 (crates/aiAssist/src/state.rs)
- 添加了MCP相关状态字段：
  - `selected_mcp_server`: 当前选择的MCP服务器
  - `mcp_server_capabilities`: MCP服务器能力缓存
  - `pending_tool_calls`: 待处理的工具调用
  - `show_tool_call_confirmation`: 显示工具调用确认对话框
  - `current_tool_call_batch`: 当前工具调用批次

- 新增方法：
  - `set_selected_mcp_server()`: 设置选中的MCP服务器
  - `update_mcp_server_capabilities()`: 更新MCP服务器能力
  - `approve_tool_calls()`: 批准工具调用
  - `reject_tool_calls()`: 拒绝工具调用

### 3. MCP集成管理

#### MCP集成管理器 (crates/aiAssist/src/mcp_integration.rs)
- `McpIntegrationManager`: 管理MCP服务器与AI助手的集成
- `handle_tool_calls_response()`: 处理LLM响应中的工具调用
- `execute_approved_tool_calls()`: 执行已确认的工具调用
- `format_tool_results_as_message()`: 格式化工具调用结果为消息

### 4. 用户界面增强

#### AI助手UI (crates/aiAssist/src/ui.rs)
- 添加了MCP服务器选择器下拉菜单
- 实现了工具调用确认对话框 `render_tool_call_confirmation()`
- 显示工具调用详情：工具名称、服务器、参数、调用类型
- 提供确认/拒绝工具调用的用户界面

### 5. 应用程序集成

#### 主应用 (src/app.rs)
- 添加了 `McpIntegrationManager` 到应用状态
- 实现了 `sync_mcp_servers_to_ai_assistant()` 方法同步MCP服务器信息
- 添加了 `convert_mcp_capabilities_to_ai_format()` 转换MCP能力格式
- 在更新循环中定期同步MCP服务器信息（每30秒）

#### iTools状态扩展 (crates/itools/src/state.rs)
- 添加了 `get_available_mcp_servers()` 获取可用MCP服务器列表
- 添加了 `get_mcp_server_capabilities()` 获取MCP服务器能力

## 工作流程

### 1. 初始化流程
1. 应用启动时，iTools模块初始化MCP服务器管理器
2. AI助手状态初始化MCP相关字段
3. 定期同步MCP服务器信息到AI助手

### 2. 工具调用流程
1. 用户在AI助手中选择MCP服务器
2. 用户发送消息给AI助手
3. AI助手将MCP服务器的工具转换为OpenAI Function Calling格式
4. 发送带有工具定义的请求给LLM
5. LLM返回包含tool_calls的响应
6. 系统解析tool_calls并显示确认对话框
7. 用户确认后执行工具调用
8. 将工具调用结果返回给LLM继续对话

### 3. 用户确认机制
- 所有工具调用都需要用户明确确认
- 显示详细的工具调用信息：名称、参数、服务器
- 提供批量确认/拒绝功能
- 支持查看工具调用的详细参数

## 技术特点

### 1. OpenAI兼容性
- 完全兼容OpenAI Function Calling API格式
- 支持标准的tools、tool_calls字段
- 可与任何支持Function Calling的LLM模型配合使用

### 2. 安全性
- 所有工具调用都需要用户确认
- 显示详细的调用信息供用户审查
- 支持拒绝危险的工具调用

### 3. 扩展性
- 模块化设计，易于扩展新的MCP功能
- 支持多种MCP调用类型
- 可配置的集成选项

### 4. 用户体验
- 直观的MCP服务器选择界面
- 清晰的工具调用确认对话框
- 实时的工具调用状态反馈

## 当前限制

1. **工具执行器**: 当前使用占位符实现，需要集成真实的MCP客户端
2. **流式支持**: 暂不支持流式响应中的工具调用
3. **并发控制**: 暂不支持并发工具调用
4. **错误处理**: 需要更完善的错误处理和重试机制

## 下一步计划

1. 集成真实的MCP客户端执行工具调用
2. 添加工具调用历史记录
3. 实现工具调用结果的缓存机制
4. 添加工具调用性能监控
5. 支持更复杂的工具调用场景

## 使用方法

1. 在iTools模块中配置和测试MCP服务器
2. 在AI助手中选择要使用的MCP服务器
3. 发送需要工具支持的消息给AI助手
4. 在确认对话框中审查并确认工具调用
5. 查看工具调用结果并继续对话

这个实现为SeeU Desktop提供了强大的MCP Function Calling能力，使AI助手能够与外部工具和服务进行安全、可控的交互。
