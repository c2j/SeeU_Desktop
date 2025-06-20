# AI助手Function Call日志优化文档

## 概述

本文档描述了对SeeU Desktop AI助手Function Call功能的日志记录优化，以便更好地调试Function Call相关问题。

## 优化内容

### 1. 完整的LLM请求JSON日志记录

在所有API调用方法中添加了完整的请求JSON打印：

- `send_chat_with_tools_full_response()` - 返回完整响应的方法
- `send_chat_with_tools()` - 返回文本内容的方法  
- `send_chat_stream_with_tools()` - 流式请求方法

**日志格式：**
```
📋 完整请求JSON:
  {
    "model": "qwen2.5:7b",
    "messages": [
      {
        "role": "user",
        "content": "用户消息内容"
      }
    ],
    "stream": false,
    "temperature": 0.7,
    "max_tokens": 2000,
    "tools": [
      {
        "type": "function",
        "function": {
          "name": "tool_name",
          "description": "工具描述",
          "parameters": {
            // JSON Schema
          }
        }
      }
    ],
    "tool_choice": null
  }
```

### 2. 完整的LLM响应JSON日志记录

在非流式API调用中添加了完整的响应JSON打印：

**日志格式：**
```
📥 完整响应JSON:
  {
    "id": "chatcmpl-xxx",
    "object": "chat.completion",
    "created": 1234567890,
    "model": "qwen2.5:7b",
    "choices": [
      {
        "index": 0,
        "message": {
          "role": "assistant",
          "content": "响应内容",
          "tool_calls": [
            {
              "id": "call_xxx",
              "type": "function",
              "function": {
                "name": "tool_name",
                "arguments": "{\"param\": \"value\"}"
              }
            }
          ]
        },
        "finish_reason": "tool_calls"
      }
    ],
    "usage": {
      "prompt_tokens": 100,
      "completion_tokens": 50,
      "total_tokens": 150
    }
  }
```

### 3. MCP工具调用解析日志

增强了MCP工具调用解析过程的日志记录：

**日志格式：**
```
🔍 解析MCP工具调用:
  - 调用ID: call_xxx
  - 函数名称: tool_name
  - 函数参数: {"param": "value"}
  - 识别为直接工具调用: tool_name
✅ MCP工具调用解析成功
```

### 4. MCP工具执行日志

详细记录MCP工具的执行过程：

**执行开始日志：**
```
🔧 开始执行MCP工具调用:
  - 服务器ID: xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx
  - 调用类型: CallTool
  - 工具名称: tool_name
  - 调用ID: call_xxx
  - 参数JSON: {"param": "value"}
```

**执行结果日志：**
```
✅ MCP工具调用执行完成:
  - 调用ID: call_xxx
  - 执行状态: 成功
  - 结果JSON:
    {
      "success": true,
      "message": "工具执行成功",
      "data": "结果数据"
    }
```

### 5. 工具调用批次创建日志

增强了工具调用批次创建过程的日志记录：

**日志格式：**
```
📦 开始创建工具调用批次
  - 批次ID: xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx
  - 工具调用总数: 2
  - 处理第 1 个工具调用: tool_name_1
    ✅ 成功解析为MCP工具调用，目标服务器: 服务器名称
  - 处理第 2 个工具调用: tool_name_2
    ✅ 成功解析为MCP工具调用，目标服务器: 服务器名称
✅ 成功创建工具调用批次:
  - 有效工具调用数: 2
  - 等待用户确认执行
```

## 技术实现

### 1. 数据结构扩展

为了支持响应JSON序列化，为以下结构体添加了`Serialize` trait：

- `ChatResponse`
- `ChatResponseMessage`
- `Choice`
- `Usage`

### 2. 日志级别

所有Function Call相关的日志都使用`log::info!`级别，确保在正常运行时可以看到详细信息。错误信息使用`log::error!`级别。

### 3. JSON格式化

使用`serde_json::to_string_pretty()`来格式化JSON输出，确保日志的可读性。每行都有适当的缩进。

## 使用方法

1. 确保日志级别设置为`info`或更详细
2. 在AI助手中选择MCP服务器
3. 发送需要调用工具的消息
4. 查看控制台输出，可以看到完整的请求、响应和执行过程

## 调试价值

这些详细的日志记录可以帮助：

1. **验证请求格式** - 确保发送给LLM的请求格式正确
2. **检查响应解析** - 验证LLM返回的tool_calls是否被正确解析
3. **追踪执行流程** - 了解工具调用的完整执行过程
4. **定位问题** - 快速找到Function Call失败的原因
5. **性能分析** - 通过时间戳分析各个步骤的耗时

## 注意事项

- 日志中可能包含敏感信息（如API密钥），在生产环境中需要注意日志安全
- 大量的JSON日志可能会影响性能，可以考虑在生产环境中降低日志级别
- 建议定期清理日志文件，避免占用过多磁盘空间
