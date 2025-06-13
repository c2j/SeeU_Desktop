# MCP (Model Context Protocol) 集成指南

## 概述

iTools 模块现在支持 MCP (Model Context Protocol) 协议，允许 AI 助手与外部工具和资源进行无缝集成。

## 功能特性

### 1. MCP 服务器管理
- 支持多种传输方式：命令行、TCP、Unix Socket、WebSocket
- 按目录层次组织服务器配置
- 服务器状态监控和连接管理
- 配置导入/导出功能

### 2. 服务器发现和验证
- 自动发现服务器能力（工具、资源、提示）
- 连接状态实时监控
- 服务器功能验证和测试

### 3. 用户界面
- 直观的树形结构展示服务器
- 实时状态指示器
- 简化的配置管理界面

## 使用方法

### 1. 访问 MCP 设置

在 iTools 界面中，点击 "⚙️ MCP设置" 选项卡即可进入 MCP 服务器管理界面。

### 2. 添加服务器

1. 点击 "➕ 添加服务器" 按钮
2. 填写服务器信息：
   - 名称：服务器的显示名称
   - 描述：可选的服务器描述
   - 目录：服务器所属的分类目录
3. 配置传输方式：
   - **命令行**：适用于本地可执行程序
   - **TCP**：适用于远程服务器
   - **Unix Socket**：适用于本地 IPC 通信
   - **WebSocket**：适用于 Web 服务

### 3. 预设服务器示例

系统提供了一些预设的示例服务器配置：

#### Everything Server
```json
{
  "name": "Everything Server",
  "description": "MCP server with everything capabilities",
  "transport": {
    "type": "command",
    "command": "npx",
    "args": ["-y", "@modelcontextprotocol/server-everything"],
    "env": {}
  },
  "enabled": false,
  "auto_start": false,
  "directory": "Examples"
}
```

#### File System Server
```json
{
  "name": "File System Server",
  "description": "MCP server for file system operations",
  "transport": {
    "type": "command",
    "command": "npx",
    "args": ["-y", "@modelcontextprotocol/server-filesystem"],
    "env": {}
  },
  "enabled": false,
  "auto_start": false,
  "directory": "Examples"
}
```

### 4. 服务器管理

- **连接/断开**：点击服务器旁边的连接按钮
- **测试连接**：使用 🧪 按钮测试服务器响应
- **查看设置**：使用 🔧 按钮查看服务器详细信息

### 5. 配置导入/导出

- **导入配置**：从 JSON 文件导入服务器配置
- **导出配置**：将当前配置导出为 JSON 文件
- 支持按目录选择性导出

## 技术实现

### 架构组件

1. **McpServerManager**：服务器配置和生命周期管理
2. **RmcpClient**：MCP 协议客户端实现
3. **McpSettingsUi**：用户界面组件
4. **TransportConfig**：多种传输方式支持

### 协议支持

- JSON-RPC 2.0 基础协议
- MCP 标准方法：
  - `initialize`：服务器初始化
  - `tools/list`：列出可用工具
  - `resources/list`：列出可用资源
  - `prompts/list`：列出可用提示

### 安全特性

- 服务器权限管理
- 连接状态验证
- 错误处理和恢复

## 配置文件

MCP 服务器配置存储在：
```
~/.config/seeu_desktop/mcp_servers.json
```

配置文件格式：
```json
[
  {
    "name": "服务器名称",
    "description": "服务器描述",
    "transport": {
      "type": "command|tcp|unix|websocket",
      // 传输特定配置
    },
    "enabled": true,
    "auto_start": false,
    "directory": "目录名称",
    "metadata": {}
  }
]
```

## 开发指南

### 添加新的传输方式

1. 在 `TransportConfig` 枚举中添加新变体
2. 在 `RmcpClient` 中实现连接逻辑
3. 在 UI 中添加配置界面

### 扩展服务器能力

1. 修改 `ServerCapabilities` 结构
2. 更新能力查询逻辑
3. 在 UI 中显示新能力

## 故障排除

### 常见问题

1. **服务器连接失败**
   - 检查命令路径是否正确
   - 验证网络连接（TCP/WebSocket）
   - 查看错误日志

2. **配置文件损坏**
   - 删除配置文件重新开始
   - 使用备份配置恢复

3. **权限问题**
   - 确保有执行命令的权限
   - 检查文件系统访问权限

### 日志调试

启用详细日志：
```bash
RUST_LOG=debug cargo run
```

## 未来计划

- [ ] 完整的 rmcp crate 集成
- [ ] 更多预设服务器模板
- [ ] 服务器性能监控
- [ ] 批量操作支持
- [ ] 服务器依赖管理

## 参考资源

- [MCP 规范](https://spec.modelcontextprotocol.io/)
- [Rust SDK](https://github.com/modelcontextprotocol/rust-sdk)
- [示例服务器](https://github.com/modelcontextprotocol/servers)
