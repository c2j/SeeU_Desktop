# SeeU Desktop 插件开发指南

本指南将帮助您开发基于 WebAssembly (WASM) 的 SeeU Desktop 插件。

## 快速开始

### 1. 环境准备

确保您已安装以下工具：

```bash
# 安装 Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 安装 WASM 目标
rustup target add wasm32-wasi

# 验证安装
rustc --version
rustup target list --installed | grep wasm32-wasi
```

### 2. 创建新插件

使用 Hello World 插件作为模板：

```bash
# 复制模板
cp -r examples/hello-world my-plugin
cd my-plugin

# 修改 Cargo.toml
sed -i 's/hello-world-plugin/my-plugin/g' Cargo.toml

# 修改 plugin.json
# 更新插件名称、描述等信息
```

### 3. 开发插件

编辑 `src/lib.rs` 文件，实现您的插件逻辑：

```rust
use seeu_plugin_sdk::*;

simple_tool_plugin! {
    name: "my-plugin",
    display_name: "My Awesome Plugin",
    version: "0.1.0",
    description: "My plugin description",
    author: "Your Name",
    tools: [
        ToolDefinition {
            name: "my_tool".to_string(),
            description: "My tool description".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "input": {
                        "type": "string",
                        "description": "Input parameter"
                    }
                },
                "required": ["input"]
            }),
            output_schema: Some(json!({
                "type": "object",
                "properties": {
                    "result": {
                        "type": "string",
                        "description": "Tool result"
                    }
                }
            }))
        }
    ],
    handler: handle_my_tool
}

fn handle_my_tool(tool_name: &str, arguments: serde_json::Value) -> Result<serde_json::Value, PluginError> {
    match tool_name {
        "my_tool" => {
            let input = arguments.get("input")
                .and_then(|i| i.as_str())
                .unwrap_or("");
            
            Ok(json!({
                "result": format!("Processed: {}", input)
            }))
        }
        _ => Err(PluginError {
            code: -32601,
            message: format!("Unknown tool: {}", tool_name),
            data: None,
        })
    }
}
```

### 4. 构建插件

```bash
# 构建单个插件
cargo build --target wasm32-wasi --release

# 或使用构建脚本构建所有插件
cd ..
./build.sh
```

## 插件类型

### 工具插件 (Tool Plugin)

提供可执行的工具，如文件操作、数据处理等。

```rust
simple_tool_plugin! {
    // 配置...
    tools: [/* 工具定义 */],
    handler: my_handler
}
```

### 资源插件 (Resource Plugin)

提供数据资源访问，如文件内容、API 数据等。

```rust
simple_resource_plugin! {
    // 配置...
    resources: [/* 资源定义 */],
    handler: my_resource_handler
}
```

### 自定义插件

实现完整的 `Plugin` trait：

```rust
#[derive(Default)]
pub struct MyPlugin;

impl Plugin for MyPlugin {
    fn init(&mut self) -> Result<(), PluginError> {
        // 初始化逻辑
        Ok(())
    }
    
    fn get_capabilities(&self) -> PluginCapabilities {
        PluginCapabilities {
            provides_tools: true,
            provides_resources: false,
            provides_prompts: false,
            ..Default::default()
        }
    }
    
    // 实现其他必需方法...
}

export_plugin!(MyPlugin);
```

## 权限系统

插件可以请求不同级别的权限：

```json
{
  "permissions": [
    {
      "type": "FileSystem",
      "resource": "read",
      "description": "读取文件",
      "required": true,
      "level": "Medium"
    },
    {
      "type": "Network",
      "resource": "http",
      "description": "HTTP 请求",
      "required": false,
      "level": "High"
    }
  ]
}
```

### 权限级别

- **Low**: 基本操作，无安全风险
- **Medium**: 文件读取、网络访问等
- **High**: 文件写入、系统信息访问
- **Critical**: 进程执行、系统配置修改

## 最佳实践

### 1. 错误处理

始终提供清晰的错误信息：

```rust
fn my_function() -> Result<Value, PluginError> {
    // 验证输入
    if input.is_empty() {
        return Err(PluginError {
            code: -32602,
            message: "Input cannot be empty".to_string(),
            data: None,
        });
    }
    
    // 处理逻辑...
    Ok(result)
}
```

### 2. 输入验证

验证所有用户输入：

```rust
let name = arguments.get("name")
    .and_then(|n| n.as_str())
    .ok_or_else(|| PluginError {
        code: -32602,
        message: "Missing required parameter: name".to_string(),
        data: None,
    })?;

if name.trim().is_empty() {
    return Err(PluginError {
        code: -32602,
        message: "Name cannot be empty".to_string(),
        data: None,
    });
}
```

### 3. 性能优化

- 避免在 WASM 中进行大量计算
- 使用主机函数处理 I/O 操作
- 合理使用内存，避免内存泄漏

### 4. 文档编写

为每个工具提供清晰的文档：

```json
{
  "name": "process_text",
  "description": "处理文本内容，支持多种格式转换",
  "input_schema": {
    "type": "object",
    "properties": {
      "text": {
        "type": "string",
        "description": "要处理的文本内容"
      },
      "format": {
        "type": "string",
        "enum": ["uppercase", "lowercase", "title"],
        "description": "转换格式"
      }
    },
    "required": ["text"]
  }
}
```

## 调试技巧

### 1. 日志记录

使用主机日志函数：

```rust
// 在插件中记录日志
log::info!("Processing request: {:?}", request);
log::error!("Failed to process: {}", error);
```

### 2. 单元测试

为插件逻辑编写测试：

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_my_tool() {
        let args = json!({"input": "test"});
        let result = handle_my_tool("my_tool", args).unwrap();
        assert_eq!(result["result"], "Processed: test");
    }
}
```

### 3. 本地测试

在构建为 WASM 之前，先在本地测试逻辑：

```bash
# 运行测试
cargo test

# 本地运行（如果有 main 函数）
cargo run
```

## 发布插件

### 1. 版本管理

遵循语义化版本控制：

- `1.0.0` - 主要版本
- `1.1.0` - 次要版本（新功能）
- `1.1.1` - 补丁版本（bug 修复）

### 2. 打包

使用构建脚本创建发布包：

```bash
./build.sh
```

### 3. 测试

在发布前充分测试：

- 功能测试
- 性能测试
- 兼容性测试
- 安全测试

## 常见问题

### Q: 如何在插件中访问文件系统？

A: 通过主机函数请求权限，然后使用提供的 API。

### Q: 插件可以相互通信吗？

A: 目前不支持直接通信，但可以通过主应用作为中介。

### Q: 如何处理大量数据？

A: 考虑使用流式处理或将数据处理委托给主机。

### Q: 插件更新如何处理？

A: 主应用会检查版本并提示用户更新。

## 更多资源

- [示例插件](examples/)
- [SDK 文档](sdk/rust/README.md)
- [API 参考](API_REFERENCE.md)
- [社区论坛](https://github.com/c2j/SeeU_Desktop/discussions)
