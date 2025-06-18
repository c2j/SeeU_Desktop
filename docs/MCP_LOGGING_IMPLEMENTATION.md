# MCP工具调用日志记录实现

## 概述

本文档描述了在AI助手中实现的MCP（Model Context Protocol）工具调用日志记录功能。当用户在MCP下拉框中选择某项并发送消息时，系统会详细记录相关的MCP服务器信息、工具定义以及调用过程。

## 实现的日志记录功能

### 1. MCP服务器选择日志记录

**位置**: `crates/aiAssist/src/state.rs`

#### 功能描述
- 当用户在AI助手界面选择或切换MCP服务器时，系统会记录详细的选择信息
- 包括服务器名称、服务器能力统计（工具、资源、提示数量）

#### 日志示例
```
🎯 AI助手 - 用户选择MCP服务器: 文件系统工具
📋 服务器能力: 工具:5 资源:2 提示:1
```

#### 实现方法
- `set_selected_mcp_server()`: 设置选中的MCP服务器并记录变化
- `check_mcp_server_selection_change()`: 检查并记录UI中的服务器选择变化

### 2. 消息发送时的工具定义日志记录

**位置**: `crates/aiAssist/src/state.rs` (第452-487行)

#### 功能描述
- 在发送消息给LLM之前，记录选中的MCP服务器信息
- 详细记录可用工具列表及其描述
- 记录转换为OpenAI Function Calling格式的工具数量

#### 日志示例
```
🔧 AI助手 - 选中MCP服务器: 文件系统工具
📋 MCP服务器能力统计:
  - 工具数量: 5
  - 资源数量: 2
  - 提示数量: 1
🛠️ 可用工具列表:
  1. list_directory - 列出目录内容
  2. read_file - 读取文件内容
  3. write_file - 写入文件内容
  4. create_directory - 创建目录
  5. delete_file - 删除文件
✅ 已将 5 个MCP工具转换为OpenAI Function Calling格式
```

### 3. API请求日志记录

**位置**: `crates/aiAssist/src/api.rs`

#### 功能描述
- 记录发送给LLM的请求详情
- 详细记录tools定义，包括工具名称、描述和参数

#### 日志示例
```
📤 发送请求到: http://localhost:11434/v1/chat/completions
🤖 使用模型: llama3.1:8b
💬 消息数量: 3
🛠️ 发送工具定义给LLM:
  - 工具数量: 5
  1. 工具名称: list_directory
     工具描述: 列出目录内容
     参数数量: 2
       - path
       - recursive
  2. 工具名称: read_file
     工具描述: 读取文件内容
     参数数量: 1
       - file_path
```

### 4. LLM响应中的工具调用检测日志

**位置**: `crates/aiAssist/src/state.rs` (第530-549行)

#### 功能描述
- 当LLM响应包含工具调用时，详细记录每个工具调用的信息
- 包括工具调用ID、函数名称和参数

#### 日志示例
```
🎯 LLM响应包含工具调用:
  - 工具调用数量: 2
  1. 工具调用ID: call_abc123
     函数名称: list_directory
     函数参数: {"path": "/home/user", "recursive": false}
  2. 工具调用ID: call_def456
     函数名称: read_file
     函数参数: {"file_path": "/home/user/document.txt"}
```

### 5. MCP集成管理器日志记录

**位置**: `crates/aiAssist/src/mcp_integration.rs`

#### 功能描述
- 记录工具调用响应的处理过程
- 记录工具调用批次的创建和执行
- 详细记录每个工具调用的执行结果

#### 日志示例
```
🔄 MCP集成管理器 - 处理工具调用响应:
  - 工具调用数量: 2
  1. 解析工具调用: list_directory
     ✅ 成功解析为MCP工具调用
     - 原始名称: list_directory
     - 调用类型: CallTool
     - 目标服务器: 文件系统工具 (server-uuid-123)

📦 创建工具调用批次:
  - 批次ID: batch-uuid-456
  - 待处理工具调用数量: 2
✅ 工具调用批次已创建，等待用户确认

🚀 开始执行已确认的工具调用:
  - 批次ID: batch-uuid-456
  - 工具调用数量: 2
  1. 执行工具: list_directory
     - 服务器: 文件系统工具
     - 调用ID: call_abc123
     - 参数: {"path": "/home/user", "recursive": false}
     ✅ 工具调用成功
     - 结果: {"files": ["document.txt", "image.png"]}

🏁 工具调用批次执行完成 - 成功: 2, 失败: 0
```

## 日志级别说明

- **INFO**: 正常的操作流程记录，如服务器选择、工具调用等
- **DEBUG**: 详细的调试信息（在某些情况下使用）
- **WARN**: 警告信息，如缺少服务器能力信息
- **ERROR**: 错误信息，如工具调用失败

## 使用场景

这些日志记录功能在以下场景中特别有用：

1. **调试MCP集成问题**: 可以清楚地看到工具定义是否正确传递给LLM
2. **监控工具调用性能**: 可以跟踪工具调用的成功率和执行时间
3. **用户行为分析**: 了解用户如何使用不同的MCP服务器和工具
4. **问题排查**: 当工具调用失败时，可以通过日志快速定位问题

## 诊断功能增强

为了解决MCP服务器工具识别问题，我们添加了以下诊断功能：

### 1. 能力提取过程日志记录

**位置**: `crates/itools/src/mcp/rmcp_client.rs`

- 详细记录rmcp服务的工具查询过程
- 记录原始响应数据结构
- 逐个记录解析的工具信息

#### 日志示例
```
✅ 成功从rmcp服务查询工具 - 服务器: server-uuid-123
🔍 原始工具响应: {"tools": [{"name": "fetch", "description": "Fetch data", ...}]}
📊 成功解析 1 个工具从rmcp服务响应
🛠️ 解析的工具列表:
  1. fetch - Fetch data from URL
```

### 2. 数据库同步过程日志记录

**位置**: `src/app.rs`

- 记录从itools到数据库的同步过程
- 详细记录运行时能力和静态能力的使用情况
- 记录从数据库加载到AI助手的过程

#### 日志示例
```
🔧 找到服务器 'simple-tool' 的运行时能力: 工具=1, 资源=0, 提示=0
🛠️ 服务器 'simple-tool' 的工具详情:
  1. fetch - Fetch data from URL
📊 转换后的AI助手能力 - 服务器 'simple-tool': 工具=1, 资源=0, 提示=0
```

### 3. 问题诊断指南

当看到工具数量为0时，按以下顺序检查：

1. **检查rmcp服务连接**
   - 查找 "❌ 从rmcp服务查询工具失败" 日志
   - 确认MCP服务器是否正常启动

2. **检查工具响应格式**
   - 查找 "🔍 原始工具响应" 日志
   - 确认响应中是否包含 "tools" 数组

3. **检查能力同步**
   - 查找 "⚠️ 服务器既没有运行时能力也没有静态能力配置" 日志
   - 确认能力提取是否成功

4. **检查数据库存储**
   - 查找 "⚠️ 无法解析服务器的能力JSON" 日志
   - 确认能力信息是否正确存储到数据库

## 测试

### 单元测试

实现了单元测试来验证日志记录功能：

- `test_mcp_server_selection_logging`: 测试MCP服务器选择的日志记录
- `test_mcp_server_selection_change_detection`: 测试服务器选择变化的检测和日志记录

运行测试：
```bash
cargo test --package aiAssist test_mcp_server_selection_logging -- --nocapture
cargo test --package aiAssist test_mcp_server_selection_change_detection -- --nocapture
```

### 集成测试

使用提供的测试脚本进行完整的工具识别测试：

```bash
chmod +x test_mcp_logging.sh
./test_mcp_logging.sh
```

测试步骤：
1. 启动应用程序
2. 进入AI助手模块
3. 选择MCP服务器
4. 观察日志输出
5. 发送消息测试工具调用
