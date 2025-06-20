# AI助手MCP Server选择持久化功能

## 功能概述

本次功能实现了AI助手中MCP Server选择的持久化，确保在发送消息时记录所选中的MCP Server，并在工具调用显示中增加MCP Server信息，这样后续执行或再次执行时不需要重新选择MCP下拉框。

## 功能详情

### 1. MCP Server信息持久化

#### 数据结构增强
在 `ChatMessage` 结构中新增了 `mcp_server_info` 字段：

```rust
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
    /// MCP Server信息，用于记录工具调用时使用的MCP Server
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mcp_server_info: Option<McpServerInfo>,
}

/// MCP Server信息
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct McpServerInfo {
    pub server_id: Uuid,
    pub server_name: String,
}
```

#### 消息发送时记录MCP Server
在 `handle_function_call_response` 方法中，当创建包含工具调用的消息时，自动记录当前选中的MCP Server信息：

```rust
// 获取当前选中的MCP Server信息
let mcp_server_info = if let Some(server_id) = self.selected_mcp_server {
    let server_name = self.server_names.get(&server_id)
        .cloned()
        .unwrap_or_else(|| format!("服务器 {}", server_id.to_string().chars().take(8).collect::<String>()));
    Some(McpServerInfo {
        server_id,
        server_name,
    })
} else {
    None
};

// 创建工具调用消息时包含MCP Server信息
let tool_call_message = ChatMessage {
    // ... 其他字段
    mcp_server_info,
};
```

### 2. 工具调用显示增强

#### UI显示MCP Server信息
在工具调用显示中，工具名称后面会显示所属的MCP Server：

```rust
// 工具名称，包含MCP Server信息
let tool_display = if let Some(mcp_info) = mcp_server_info {
    format!("{}#工具: {} ({})", index + 1, &tool_call.function.name, &mcp_info.server_name)
} else {
    format!("{}#工具: {}", index + 1, &tool_call.function.name)
};
```

**显示效果**：
- 有MCP Server信息：`1#工具: search_files (文件搜索服务器)`
- 无MCP Server信息：`1#工具: search_files`

### 3. 工具执行逻辑优化

#### 工具执行请求结构
新增了 `ToolCallExecutionRequest` 结构来传递工具调用和MCP Server信息：

```rust
/// 工具调用执行请求
#[derive(Clone, Debug)]
pub struct ToolCallExecutionRequest {
    pub tool_call: crate::api::ToolCall,
    pub mcp_server_info: Option<crate::state::McpServerInfo>,
}
```

#### 执行时优先使用记录的MCP Server
修改了 `execute_single_tool_call` 方法，优先使用消息中记录的MCP Server信息：

```rust
pub fn execute_single_tool_call(&mut self, tool_call: &crate::api::ToolCall, mcp_server_info: Option<&McpServerInfo>) {
    // 优先使用传入的MCP Server信息，如果没有则使用当前选中的
    let (server_id, server_name) = if let Some(mcp_info) = mcp_server_info {
        log::info!("📡 使用消息中记录的MCP服务器: {}", mcp_info.server_name);
        (mcp_info.server_id, mcp_info.server_name.clone())
    } else if let Some(server_id) = self.selected_mcp_server {
        let server_name = self.server_names.get(&server_id)
            .cloned()
            .unwrap_or_else(|| format!("服务器 {}", server_id));
        log::info!("📡 使用当前选中的MCP服务器: {}", server_name);
        (server_id, server_name)
    } else {
        log::warn!("❌ 未选择MCP服务器，无法执行工具调用");
        return;
    };
    // ... 执行逻辑
}
```

## 技术实现

### 修改的文件
1. `crates/aiAssist/src/state.rs` - 数据结构和状态管理
2. `crates/aiAssist/src/ui.rs` - UI显示和交互逻辑
3. `crates/aiAssist/src/tests.rs` - 测试用例更新

### 核心修改

#### 1. 数据结构扩展
- 在 `ChatMessage` 中添加 `mcp_server_info` 字段
- 新增 `McpServerInfo` 结构体
- 新增 `ToolCallExecutionRequest` 结构体

#### 2. 消息创建逻辑
- 在所有 `ChatMessage` 创建处添加 `mcp_server_info` 字段
- 在工具调用消息创建时记录当前选中的MCP Server

#### 3. UI渲染逻辑
- 修改 `render_tool_calls_in_message` 函数签名，接收MCP Server信息
- 在工具名称显示中包含MCP Server名称
- 修改工具执行请求的数据流

#### 4. 工具执行逻辑
- 修改工具执行方法，支持使用记录的MCP Server信息
- 实现MCP Server信息的优先级：记录的信息 > 当前选中的服务器

## 用户体验改进

### 1. 操作简化
- **首次执行**：用户需要选择MCP Server下拉框
- **再次执行**：自动使用之前记录的MCP Server，无需重新选择
- **跨会话持久化**：MCP Server信息随消息保存，会话恢复后仍然有效

### 2. 信息透明
- **工具来源明确**：每个工具调用都显示其所属的MCP Server
- **执行日志清晰**：日志中明确显示使用的是记录的还是当前选中的MCP Server
- **状态一致性**：确保工具执行使用正确的MCP Server

### 3. 错误处理
- **服务器不可用**：如果记录的MCP Server不再可用，会有相应的错误提示
- **向后兼容**：对于没有MCP Server信息的旧消息，仍然可以正常显示和执行

## 测试验证

### 测试覆盖
- ✅ 所有现有测试继续通过 (6/6)
- ✅ 新增的数据结构序列化/反序列化正常
- ✅ MCP Server信息正确记录和传递
- ✅ 工具执行逻辑正确处理MCP Server信息

### 质量保证
- ✅ 编译成功，无错误
- ✅ 向后兼容性保持
- ✅ 数据持久化正常工作

## 使用流程

### 典型使用场景
1. **用户选择MCP Server**：在下拉框中选择一个MCP Server
2. **发送消息触发工具调用**：LLM响应包含工具调用
3. **系统记录MCP Server信息**：在工具调用消息中自动记录选中的MCP Server
4. **显示工具信息**：工具名称后显示"(MCP Server名称)"
5. **执行工具**：点击执行按钮，自动使用记录的MCP Server
6. **再次执行**：无需重新选择MCP Server，直接使用记录的信息

### 日志示例
```
📡 使用消息中记录的MCP服务器: 文件搜索服务器
🚀 开始执行单个工具调用: search_files
📡 通过MCP服务器执行工具: 文件搜索服务器 -> search_files
```

## 总结

本次功能实现显著改善了AI助手中MCP Server的使用体验：

1. **持久化记录**：MCP Server选择信息随消息保存，确保一致性
2. **信息透明**：清晰显示每个工具所属的MCP Server
3. **操作简化**：再次执行时无需重新选择MCP Server
4. **向后兼容**：不影响现有功能和数据

这些改进使用户能够更高效地使用MCP工具，减少重复操作，提高整体的使用体验。
