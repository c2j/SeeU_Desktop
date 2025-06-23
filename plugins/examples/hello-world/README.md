# Hello World Plugin

这是一个简单的 Hello World 插件，演示了 SeeU Desktop 插件系统的基本功能。

## 功能

这个插件提供了三个简单的工具：

1. **hello** - 向指定的人问好
2. **echo** - 回显输入的文本
3. **random_number** - 在指定范围内生成随机数

## 构建

确保已安装 Rust 和 WASM 目标：

```bash
# 安装 WASM 目标
rustup target add wasm32-wasi

# 构建插件
cargo build --target wasm32-wasi --release
```

构建完成后，WASM 文件将位于 `target/wasm32-wasi/release/hello_world_plugin.wasm`。

## 安装

1. 将 `plugin.json` 和生成的 `.wasm` 文件复制到插件目录
2. 在 SeeU Desktop 的插件管理器中安装插件
3. 启用插件

## 使用示例

### hello 工具
```json
{
  "name": "hello",
  "arguments": {
    "name": "Alice"
  }
}
```

响应：
```json
{
  "message": "Hello, Alice! 👋"
}
```

### echo 工具
```json
{
  "name": "echo",
  "arguments": {
    "text": "Hello, World!"
  }
}
```

响应：
```json
{
  "echoed": "Hello, World!"
}
```

### random_number 工具
```json
{
  "name": "random_number",
  "arguments": {
    "min": 1,
    "max": 10
  }
}
```

响应：
```json
{
  "number": 7
}
```

## 开发说明

这个插件使用了 `seeu-plugin-sdk` 提供的 `simple_tool_plugin!` 宏，大大简化了插件开发过程。

### 关键组件

1. **工具定义**: 在宏中定义了三个工具及其输入/输出模式
2. **处理函数**: `handle_tool_call` 函数处理所有工具调用
3. **错误处理**: 适当的错误处理和验证

### 扩展

要添加新工具，只需：

1. 在 `tools` 数组中添加新的 `ToolDefinition`
2. 在 `handle_tool_call` 函数中添加相应的处理逻辑
3. 更新 `plugin.json` 中的工具列表

## 许可证

MIT License
