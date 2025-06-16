# MCP测试功能修复总结

## 问题描述

在MCP设置界面中，能正常连接到MCP Server，但在对Server进行测试时，报错：
```
Simulated tool '...' execution real call failed
```

**最新发现的根本问题**：
从日志分析发现真正的错误是：
```
[2025-06-15 19:46:03 ERROR] RMCP service call_tool failed for 'get_value': Transport error: disconnected
[2025-06-15 19:46:03 ERROR] RMCP tool call failed for 'get_value': Failed to call tool 'get_value': Transport error: disconnected
```

这表明rmcp服务连接已经断开，导致工具调用失败并回退到模拟模式。

## 问题分析

通过分析代码和参考`examples/rmcp_examples`下的示例，发现了以下问题：

### 1. 参数格式转换问题
- **问题**：UI传递的参数是`HashMap<String, String>`，但rmcp期望的是`Option<serde_json::Map<String, serde_json::Value>>`
- **位置**：`crates/itools/src/ui/mcp_settings.rs`中的`execute_real_tool_test`方法
- **原因**：直接使用`serde_json::json!(parameters)`会创建错误的JSON结构

### 2. rmcp客户端参数处理问题
- **问题**：`rmcp_client.rs`中的`call_tool`方法参数转换逻辑不完善
- **位置**：`crates/itools/src/mcp/rmcp_client.rs`中的`McpClient::call_tool`方法
- **原因**：参数类型检查和转换逻辑不够健壮

### 3. 连接生命周期管理问题
- **问题**：rmcp服务连接断开后没有被检测到和处理
- **位置**：`crates/itools/src/mcp/rmcp_client.rs`中的连接管理
- **原因**：缺少连接状态监控和重连机制

### 4. 错误处理和日志不足
- **问题**：当rmcp调用失败时，错误信息不够详细，难以调试
- **影响**：无法准确定位问题原因

## 修复方案

### 1. 修复UI层参数转换 (`mcp_settings.rs`)

**修复前**：
```rust
let arguments = serde_json::json!(parameters);
```

**修复后**：
```rust
let arguments = if parameters.is_empty() {
    serde_json::Value::Object(serde_json::Map::new())
} else {
    // Convert HashMap<String, String> to JSON object with proper value types
    let mut json_map = serde_json::Map::new();
    for (key, value) in parameters {
        // Try to parse the value as JSON first, fallback to string
        let json_value = serde_json::from_str::<serde_json::Value>(value)
            .unwrap_or_else(|_| serde_json::Value::String(value.clone()));
        json_map.insert(key.clone(), json_value);
    }
    serde_json::Value::Object(json_map)
};
```

**改进点**：
- 正确处理空参数情况
- 智能类型转换：尝试将字符串解析为JSON，失败则保持为字符串
- 确保最终格式为JSON对象

### 2. 修复rmcp客户端参数处理 (`rmcp_client.rs`)

**修复前**：
```rust
let arguments_map = arguments.and_then(|v| v.as_object().cloned());
```

**修复后**：
```rust
let arguments_map = if let Some(args) = arguments {
    if let Some(obj) = args.as_object() {
        Some(obj.clone())
    } else {
        log::warn!("Arguments for tool '{}' are not an object, attempting conversion: {:?}", name, args);
        None
    }
} else {
    None
};
```

**改进点**：
- 增加详细的参数类型检查
- 添加警告日志用于调试
- 更健壮的错误处理

### 3. 修复连接生命周期管理 (`rmcp_client.rs`)

**修复前**：
```rust
pub fn disconnect_server(&mut self, server_id: Uuid) -> Result<()> {
    if let Some(connection) = self.servers.get_mut(&server_id) {
        connection.status = ConnectionStatus::Disconnected;
        connection.capabilities = None;
        connection.last_ping = None;
    }
    // ... 没有正确关闭rmcp服务
}
```

**修复后**：
```rust
pub fn disconnect_server(&mut self, server_id: Uuid) -> Result<()> {
    if let Some(connection) = self.servers.get_mut(&server_id) {
        // If we have an rmcp service, cancel it properly
        if let Some(rmcp_service) = &connection.rmcp_service {
            log::info!("Cancelling rmcp service for server: {}", server_id);
        }

        connection.status = ConnectionStatus::Disconnected;
        connection.capabilities = None;
        connection.last_ping = None;
        connection.rmcp_service = None; // This will drop the service
    }
    // ...
}
```

**新增方法**：
- `ensure_server_connected()`: 检查连接状态并在需要时重新连接
- 增强的连接状态检查和错误处理

### 4. 增强错误处理和日志

**改进点**：
- 在所有关键步骤添加详细的debug日志
- 改进错误消息，包含更多上下文信息
- 在rmcp调用前后记录参数和响应
- 检测连接断开错误并提供重连建议

## 修复文件列表

1. `crates/itools/src/mcp/rmcp_client.rs`
   - 修复`McpClient::call_tool`方法的参数处理
   - 修复`McpClient::read_resource`方法的错误处理
   - 修复`McpClient::get_prompt`方法的参数处理
   - 增强所有方法的日志记录

2. `crates/itools/src/ui/mcp_settings.rs`
   - 修复`execute_real_tool_test`方法的参数转换
   - 修复`execute_real_prompt_test`方法的参数转换
   - 增强错误日志记录
   - 添加连接状态检查和重连逻辑

3. `crates/itools/src/mcp/server_manager.rs`
   - 添加`ensure_server_connected`方法

## 验证方法

1. **启动应用程序**：`cargo run`
2. **进入iTools模块**：点击iTools标签
3. **打开MCP设置**：点击MCP设置按钮
4. **添加测试服务器**：可以使用examples中的服务器配置
5. **连接服务器**：点击连接按钮
6. **测试工具**：点击测试工具按钮，应该不再显示"Simulated tool ... execution real call failed"错误

## 预期结果

修复后，MCP服务器测试应该能够：
- 正确传递参数给rmcp服务
- 检测并处理连接断开问题
- 在需要时自动重新连接到服务器
- 成功调用真实的MCP服务器工具
- 返回实际的工具执行结果
- 提供详细的调试信息和错误诊断

## 🔧 关键修复点

### 连接断开问题的解决方案

1. **问题识别**：通过日志分析发现"Transport error: disconnected"是根本原因
2. **并发冲突检测**：发现rmcp服务在处理能力查询和工具调用时发生冲突
3. **序列化操作**：避免同时进行能力查询和工具调用
4. **连接稳定化**：在连接建立后添加延迟以稳定rmcp服务
5. **重连机制**：添加`ensure_server_connected`方法来处理断开的连接
6. **资源清理**：正确释放断开的rmcp服务资源

### 🚨 根本原因分析

通过详细的日志分析，发现问题的真正原因是：

**并发访问冲突**：
- 重连成功后，系统立即开始查询服务器能力（`list_tools`, `list_resources`, `list_prompts`）
- 同时，UI尝试调用工具进行测试
- rmcp服务无法处理并发请求，导致连接断开

**时间线分析**：
```
22:09:47 - 重连成功，创建rmcp服务
22:09:47 - 开始能力查询（list_tools, list_resources, list_prompts）
22:09:47 - 能力查询成功完成
22:09:47 - 立即尝试工具调用（get_value）
22:09:47 - 工具调用失败："Transport error: disconnected"
```

**解决方案**：
1. **避免连接测试冲突**：在`ensure_server_connected`中跳过连接测试，避免与能力查询冲突
2. **添加稳定化延迟**：在能力查询完成后添加200ms延迟
3. **UI层延迟**：在工具调用前添加100ms延迟
4. **序列化操作**：确保操作按顺序进行，避免并发访问

## 🔄 最新修复 (2025-06-15 22:47)

### 问题确认
通过健康检查功能，我们确认了问题的根本原因：
```
[2025-06-15 22:47:38 INFO] ✅ Server 1f4dde59-ea16-4ae9-9004-1d97adb74726 appears to be connected, skipping connection test to avoid conflicts
[2025-06-15 22:47:38 ERROR] ❌ RMCP service health check failed: Failed to list tools: Transport error: disconnected
```

**关键发现**：
- 服务器状态显示为`Connected`
- `rmcp_service`存在
- 但底层transport连接已断开

这是一个**状态不一致**问题：我们的状态管理没有反映rmcp服务的实际连接状态。

### 🛠️ 最终修复方案

1. **状态一致性检查**：
   - 在工具调用前进行rmcp服务健康检查
   - 检测到连接断开时自动更新服务器状态
   - 避免状态显示为Connected但实际连接已断开的问题

2. **自动重连机制**：
   - 检测到连接丢失时自动尝试重连
   - 重连成功后自动重试工具调用
   - 提供详细的重连日志和状态反馈

3. **改进错误处理**：
   - 区分连接丢失和其他类型的错误
   - 针对连接丢失错误提供特殊处理逻辑
   - 保持向后兼容的模拟数据回退机制

4. **借用检查优化**：
   - 重构`ensure_server_connected`方法避免借用冲突
   - 使用临时变量管理状态更新时机
   - 确保线程安全的状态管理

### 修复的文件
- `crates/itools/src/mcp/rmcp_client.rs` - 健康检查和重连逻辑
- `crates/itools/src/ui/mcp_settings.rs` - 自动重连和重试机制

### 预期效果
- ✅ 检测到连接断开时自动重连
- ✅ 重连成功后自动重试工具调用
- ✅ 状态管理与实际连接状态保持一致
- ✅ 不再显示"Simulated tool ... execution real call failed"错误

## 🔧 最终修复 (2025-06-15 23:00)

### 🎯 根本问题确认

通过深入分析rmcp示例代码，发现了问题的真正根源：**rmcp服务生命周期管理不当**。

在rmcp示例中，客户端在完成操作后会调用`service.cancel().await?`来正确关闭服务。但在我们的代码中，我们没有正确管理rmcp服务的生命周期，导致：

1. **资源泄漏**：旧的rmcp服务没有被正确关闭
2. **连接冲突**：新旧服务可能同时存在，导致连接问题
3. **进程管理问题**：底层进程可能在某个时候自动退出

### 🛠️ 最终解决方案

1. **正确的服务关闭**：
   - 实现异步的`disconnect_server_async`方法
   - 在重连前调用`rmcp_service.service.cancel().await`
   - 确保旧服务被正确清理

2. **改进重连逻辑**：
   - 在创建新连接前先清理旧连接
   - 添加详细的清理日志
   - 确保资源不会泄漏

3. **生命周期管理**：
   - 正确管理rmcp服务的创建和销毁
   - 避免同时存在多个服务实例
   - 确保进程资源得到正确释放

### 修复的关键代码

```rust
// 正确关闭rmcp服务
if let Some(rmcp_service) = connection.rmcp_service.take() {
    log::info!("🔄 Properly cancelling old rmcp service for server: {}", server_id);
    if let Err(e) = rmcp_service.service.cancel().await {
        log::warn!("Failed to cancel old rmcp service: {}", e);
    } else {
        log::info!("✅ Successfully cancelled old rmcp service");
    }
}
```

### 预期效果
- ✅ 正确管理rmcp服务生命周期
- ✅ 避免资源泄漏和连接冲突
- ✅ 稳定的工具调用功能
- ✅ 不再出现"Transport error: disconnected"错误

## 参考资料

- `examples/rmcp_examples/clients/src/everything_stdio.rs` - rmcp客户端使用示例
- `examples/rmcp_examples/clients/src/git_stdio.rs` - 参数传递示例
- rmcp官方文档：https://github.com/modelcontextprotocol/rust-sdk
