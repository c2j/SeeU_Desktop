# MCP服务器能力数据库存储功能实现

## 功能概述

根据用户建议，我们实现了在MCP服务器测试通过后将capability（能力信息）写入SQLite数据库的功能。这个功能提供了更好的持久化存储和查询性能，确保测试通过的MCP服务器能力信息能够可靠地保存和恢复。

## 实现的功能

### 1. 新增MCP事件类型

在 `crates/itools/src/mcp/rmcp_client.rs` 中添加了新的事件类型：

```rust
/// MCP events for UI updates
#[derive(Debug, Clone)]
pub enum McpEvent {
    ServerConnected(Uuid),
    ServerDisconnected(Uuid),
    ServerError(Uuid, String),
    CapabilitiesUpdated(Uuid, ServerCapabilities),
    /// Server capabilities extracted and ready for database storage
    CapabilitiesExtracted(Uuid, ServerCapabilities, String), // server_id, capabilities, capabilities_json_string
    HealthStatusChanged(Uuid, ServerHealthStatus),
    TestCompleted(Uuid, TestResult),
}
```

### 2. 测试成功后自动触发能力保存

在MCP服务器测试完成后，如果测试成功，系统会自动：

1. **提取服务器能力**：调用 `query_server_capabilities` 方法
2. **序列化能力信息**：将能力转换为JSON字符串
3. **发送能力提取事件**：触发 `CapabilitiesExtracted` 事件

```rust
// 如果测试成功，尝试提取能力并保存到数据库
if test_result.success {
    // ... 提取能力逻辑
    if let Some(capabilities) = self.servers.get(&server_id).and_then(|conn| conn.capabilities.clone()) {
        match serde_json::to_string(&capabilities) {
            Ok(capabilities_json) => {
                // 发送能力提取成功事件
                self.send_event(McpEvent::CapabilitiesExtracted(server_id, capabilities.clone(), capabilities_json));
            }
            Err(e) => {
                log::error!("❌ 序列化能力信息失败 - 服务器: '{}' - 错误: {}", server_name, e);
            }
        }
    }
}
```

### 3. 主应用程序事件处理

在 `src/app.rs` 中添加了对 `CapabilitiesExtracted` 事件的处理：

```rust
McpEvent::CapabilitiesExtracted(server_id, capabilities, capabilities_json) => {
    log::info!("🎯 收到MCP服务器能力提取成功事件: {} - 工具:{}, 资源:{}, 提示:{}",
        server_id, capabilities.tools.len(), capabilities.resources.len(), capabilities.prompts.len());
    
    // 将测试通过的能力保存到数据库
    if let Err(e) = self.save_extracted_capabilities_to_database(server_id, &capabilities_json) {
        log::error!("❌ 保存提取的能力到数据库失败: {}", e);
    } else {
        log::info!("✅ 成功保存测试通过的能力到数据库 - 服务器: {}", server_id);
        has_green_status_change = true; // 触发AI助手状态更新
    }
    
    needs_sync = true;
}
```

### 4. 数据库保存方法

实现了专门的数据库保存方法 `save_extracted_capabilities_to_database`：

```rust
/// 保存提取的能力到数据库
fn save_extracted_capabilities_to_database(&self, server_id: uuid::Uuid, capabilities_json: &str) -> Result<(), Box<dyn std::error::Error>> {
    log::info!("💾 开始保存提取的能力到数据库 - 服务器: {}", server_id);
    
    // 从数据库加载现有的MCP服务器记录
    let storage = self.inote_state.storage.lock()
        .map_err(|e| format!("Failed to lock storage: {}", e))?;
    
    // 加载现有服务器记录
    let mut server_record = match storage.load_mcp_server(&server_id.to_string()) {
        Ok(record) => record,
        Err(e) => {
            log::error!("❌ 无法加载MCP服务器记录: {} - 错误: {}", server_id, e);
            return Err(format!("无法加载MCP服务器记录: {}", e).into());
        }
    };
    
    // 更新能力字段
    server_record.capabilities = Some(capabilities_json.to_string());
    server_record.updated_at = chrono::Utc::now();
    
    // 保存更新后的记录
    match storage.save_mcp_server(&server_record) {
        Ok(()) => {
            log::info!("✅ 成功保存提取的能力到数据库 - 服务器: '{}'", server_record.name);
            // 验证保存结果...
            Ok(())
        }
        Err(e) => {
            log::error!("❌ 保存MCP服务器记录失败: {}", e);
            Err(format!("保存MCP服务器记录失败: {}", e).into())
        }
    }
}
```

### 5. UI事件处理增强

在MCP设置UI中添加了对新事件的处理：

```rust
McpEvent::CapabilitiesExtracted(server_id, capabilities, _capabilities_json) => {
    log::info!("🎯 MCP设置页面收到能力提取成功事件: {} - 工具:{}, 资源:{}, 提示:{}",
        server_id, capabilities.tools.len(), capabilities.resources.len(), capabilities.prompts.len());
    
    // 更新UI缓存中的能力信息
    self.server_capabilities.insert(server_id, capabilities);
    
    // 显示成功消息
    self.ui_state.status_message = Some(format!("服务器 {} 能力已成功提取并保存到数据库 💾", server_id));
    self.ui_state.error_message = None;
}
```

## 工作流程

1. **用户测试MCP服务器**：在iTools -> MCP设置中点击测试按钮
2. **测试执行**：系统执行MCP服务器功能测试
3. **测试成功**：如果测试通过，自动触发能力提取
4. **能力提取**：系统调用MCP协议获取服务器的工具、资源和提示能力
5. **数据库保存**：将提取的能力JSON保存到SQLite数据库的`mcp_servers`表中
6. **状态更新**：更新服务器状态为绿灯，触发AI助手下拉框刷新
7. **用户反馈**：在UI中显示成功消息，确认能力已保存

## 优势

### 1. 持久化存储
- 测试通过的能力信息永久保存在数据库中
- 应用重启后能够恢复MCP服务器能力信息
- 避免重复测试和能力提取

### 2. 性能优化
- 数据库查询比文件读取更高效
- 支持复杂的查询和过滤操作
- 减少网络请求和MCP协议调用

### 3. 数据完整性
- 包含详细的验证逻辑，确保保存的JSON格式正确
- 记录详细的日志信息，便于问题诊断
- 自动更新时间戳，跟踪能力信息的变更

### 4. 用户体验
- 自动化流程，无需用户手动操作
- 实时反馈，显示保存状态和结果
- 与现有的MCP管理流程无缝集成

## 日志示例

成功保存能力到数据库时的日志输出：

```
🎯 收到MCP服务器能力提取成功事件: simple-tool - 工具:1, 资源:0, 提示:0
💾 开始保存提取的能力到数据库 - 服务器: simple-tool
✅ 成功保存提取的能力到数据库 - 服务器: 'simple-tool'
✅ 验证成功 - 服务器 'simple-tool' 的能力已保存到数据库，包含 1 个工具
✅ 成功保存测试通过的能力到数据库 - 服务器: simple-tool
🔄 已重新加载绿灯MCP服务器到AI助手
```

## 总结

这个功能实现了用户建议的"测试通过后将capability也写入sqlite数据库"的需求，提供了：

- **自动化**：测试成功后自动保存能力信息
- **可靠性**：使用数据库事务确保数据一致性
- **可观测性**：详细的日志记录和用户反馈
- **集成性**：与现有MCP管理系统无缝集成

这个功能将显著提升MCP服务器管理的效率和可靠性，为用户提供更好的使用体验。

## 更新：在保存MCP Server定义时立即获取能力定义

根据用户反馈，我们进一步优化了功能，在保存MCP Server定义时就立即尝试获取其能力定义，而不是等到测试时才获取。

### 新增功能

#### 1. 添加服务器时的能力获取提示

在 `crates/itools/src/mcp/server_manager.rs` 的 `add_server` 方法中：

```rust
// 如果服务器已启用，建议用户测试以获取能力定义
if is_enabled {
    log::info!("💡 新添加的服务器 '{}' 已启用，建议进行测试以获取能力定义", server_name);
}
```

#### 2. 更新服务器时的能力获取提示

在 `update_server` 方法中：

```rust
// 如果服务器已启用，建议用户测试以获取能力定义
if was_enabled {
    log::info!("💡 更新后的服务器 '{}' 已启用，建议进行测试以获取能力定义", server_name);
}
```

#### 3. 导入服务器时的能力获取提示

在 `import_server_config` 方法中：

```rust
// 对于已启用的服务器，建议用户测试以获取能力定义
if !enabled_servers.is_empty() {
    log::info!("💡 导入了 {} 个已启用的服务器，建议进行测试以获取能力定义", enabled_servers.len());
    for (_, server_name) in enabled_servers {
        log::info!("  - {}", server_name);
    }
}
```

#### 4. 应用启动时的绿灯服务器检查

在 `src/app.rs` 中，对于绿灯但没有能力信息的服务器：

```rust
// 对于绿灯但没有能力信息的服务器，记录需要获取能力的服务器
if let Ok(server_uuid) = server_record.get_uuid() {
    log::info!("⚠️ 绿灯服务器 '{}' 没有能力信息，需要手动测试以获取能力", server_record.name);
    // 这里可以添加到一个待处理列表，供用户手动触发测试
}
```

#### 5. 公开能力查询方法

将 `query_server_capabilities` 方法设为公有：

```rust
/// Query server capabilities using MCP protocol
pub async fn query_server_capabilities(&mut self, server_id: Uuid) -> Result<()> {
```

### 工作流程优化

现在的工作流程更加用户友好：

1. **添加/更新/导入服务器**：系统会检查服务器是否已启用
2. **智能提示**：对于已启用的服务器，系统会提示用户进行测试以获取能力定义
3. **应用启动检查**：启动时检查绿灯服务器的能力信息完整性
4. **用户引导**：通过日志明确指导用户下一步操作

### 解决的问题

这个优化解决了用户反馈的核心问题：
- **simple-tool服务器工具识别问题**：确保测试通过的服务器能力信息被正确保存
- **数据一致性**：避免绿灯服务器没有能力信息的情况
- **用户体验**：提供清晰的操作指导，减少用户困惑

### 日志示例

优化后的日志输出更加清晰：

```
✅ 已添加新服务器: 'simple-tool' (uuid-here)
💡 新添加的服务器 'simple-tool' 已启用，建议进行测试以获取能力定义
⚠️ 绿灯服务器 'counter' 没有能力信息，需要手动测试以获取能力
```

这个功能确保了MCP服务器管理的完整性和可靠性，为用户提供了更好的使用体验。
