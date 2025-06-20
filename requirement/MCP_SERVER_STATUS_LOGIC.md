# MCP Server 状态变化逻辑与持久化机制

## 概述

本文档详细说明了SeeU Desktop中MCP Server的状态变化逻辑（红灯→黄灯→绿灯）以及状态持久化机制。

## 状态定义

### 🔴 红灯 (Red)
- **含义**: 服务器配置已添加或修改，但尚未测试
- **触发条件**:
  - 新添加服务器配置
  - 修改现有服务器配置
  - 应用程序启动时，如果配置中没有保存的健康状态

### 🟡 黄灯 (Yellow)  
- **含义**: 服务器已成功连接，但尚未通过功能测试
- **触发条件**:
  - 服务器连接成功，rmcp客户端创建完成
  - 功能测试失败（连接成功但工具调用失败）

### 🟢 绿灯 (Green)
- **含义**: 服务器已连接并通过所有功能测试，可用于AI助手
- **触发条件**:
  - 功能测试成功完成

## 状态变化流程

### 1. 初始状态设置

```rust
// 在 rmcp_client.rs 的 add_server_config 方法中
let health_status = config.last_health_status.clone().unwrap_or(ServerHealthStatus::Red);
```

- 新服务器默认为红灯状态
- 如果配置文件中有保存的状态，则恢复该状态

### 2. 红灯 → 黄灯

**触发位置**: `rmcp_client.rs` 的 `connect_server` 方法

```rust
// 连接成功后
connection.health_status = ServerHealthStatus::Yellow; // Connected but not tested
self.send_event(McpEvent::HealthStatusChanged(server_id, ServerHealthStatus::Yellow));
```

**条件**:
- rmcp客户端成功创建
- 服务器进程启动成功
- 基本连接建立

### 3. 黄灯 → 绿灯 或 保持黄灯

**触发位置**: `rmcp_client.rs` 的 `test_server_functionality` 方法

```rust
// 根据测试结果更新状态
let new_health_status = if test_result.success {
    ServerHealthStatus::Green
} else {
    ServerHealthStatus::Yellow // Connected but tests failed
};
```

**绿灯条件**:
- 功能测试成功 (`test_result.success == true`)
- 工具调用、资源读取或提示获取至少一项成功

**保持黄灯条件**:
- 连接正常但功能测试失败
- 工具调用返回错误

### 4. 配置修改时重置

**触发位置**: `rmcp_client.rs` 的 `update_server_config` 方法

```rust
// 配置修改时重置为红灯
connection.health_status = ServerHealthStatus::Red;
self.send_event(McpEvent::HealthStatusChanged(server_id, ServerHealthStatus::Red));
```

## 持久化机制

### 1. 配置文件位置

```rust
// 在 state.rs 中定义
let mcp_config_path = config_dir.join("seeu_desktop").join("mcp_servers.json");
```

**实际路径**:
- **macOS**: `~/Library/Application Support/seeu_desktop/mcp_servers.json`
- **Linux**: `~/.config/seeu_desktop/mcp_servers.json`
- **Windows**: `%APPDATA%\seeu_desktop\mcp_servers.json`

### 2. 持久化字段

在 `McpServerConfig` 结构中定义了三个持久化字段：

```rust
#[serde(default)]
pub last_health_status: Option<ServerHealthStatus>,
#[serde(default)]
pub last_test_time: Option<chrono::DateTime<chrono::Utc>>,
#[serde(default)]
pub last_test_success: Option<bool>,
```

### 3. 状态保存时机

#### 自动保存
- **测试完成后**: 每次运行测试后自动保存状态
- **配置修改后**: 添加、删除、更新服务器配置后保存
- **状态变化时**: 健康状态发生变化时更新配置并保存

#### 保存流程

```rust
// 在 rmcp_client.rs 的 test_server_functionality 方法中
// 更新配置中的状态信息以便持久化
if let Some(config) = self.server_configs.get_mut(&server_id) {
    config.last_health_status = Some(new_health_status);
    config.last_test_time = Some(test_time);
    config.last_test_success = Some(test_result.success);
}
```

```rust
// 在 server_manager.rs 中
async fn save_configuration_after_test(&mut self) -> Result<()> {
    // 从客户端获取更新后的配置
    let updated_configs = self.client.get_all_server_configs();
    
    // 更新本地配置并保存到文件
    self.save_configuration().await
}
```

### 4. 状态恢复

应用程序启动时，从配置文件恢复服务器状态：

```rust
// 在 rmcp_client.rs 的 add_server_config 方法中
let health_status = config.last_health_status.clone().unwrap_or(ServerHealthStatus::Red);
```

## 配置文件格式示例

```json
[
  {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "name": "Everything Server",
    "description": "MCP server with everything capabilities",
    "transport": {
      "Command": {
        "command": "npx",
        "args": ["-y", "@modelcontextprotocol/server-everything"],
        "env": {}
      }
    },
    "enabled": true,
    "auto_start": false,
    "directory": "Examples",
    "metadata": {},
    "last_health_status": "Green",
    "last_test_time": "2025-06-16T14:30:00Z",
    "last_test_success": true
  }
]
```

## 状态同步到AI助手

### 同步逻辑

在 `app.rs` 的 `sync_mcp_servers_to_ai_assistant` 方法中：

```rust
// 只有绿灯状态的服务器才同步到AI助手
if matches!(health_status, itools::mcp::rmcp_client::ServerHealthStatus::Green) {
    // 同步到AI助手
    synced_count += 1;
} else {
    // 从AI助手中移除非绿灯服务器
    self.ai_assist_state.mcp_server_capabilities.remove(server_id);
}
```

### 同步频率

- **定期同步**: 每30秒检查一次状态变化
- **事件驱动**: 状态变化时立即同步

## 测试流程

### 1. 连接测试
- 验证服务器进程能否启动
- 检查基本通信是否正常
- 成功后状态变为黄灯

### 2. 功能测试
- 调用服务器的工具、资源或提示
- 验证返回结果是否正确
- 成功后状态变为绿灯

### 3. 测试失败处理
- 连接失败：保持红灯状态
- 功能测试失败：保持黄灯状态
- 记录详细错误信息供调试

## 状态持久化的优势

1. **会话恢复**: 应用重启后保持服务器状态
2. **避免重复测试**: 已测试通过的服务器无需重新测试
3. **状态历史**: 记录测试时间和结果
4. **用户体验**: 减少等待时间，提高效率

## 故障排除

### 状态不正确
1. 检查配置文件是否存在且格式正确
2. 查看日志中的状态变化事件
3. 手动重新测试服务器

### 状态不持久化
1. 检查配置目录权限
2. 确认磁盘空间充足
3. 查看保存操作的错误日志

这个机制确保了MCP Server状态的准确性和持久性，为AI助手提供了可靠的工具调用基础。
