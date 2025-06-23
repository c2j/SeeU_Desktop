# SeeU Desktop 插件系统

基于 WebAssembly (WASM) 的安全插件系统，支持多语言开发和安全沙箱执行。

## 架构概述

### 核心组件
- **WASM 运行时**: 使用 `wasmtime` 提供安全的执行环境
- **插件接口**: 标准化的 MCP 兼容接口
- **权限系统**: 细粒度的权限控制
- **生命周期管理**: 插件的安装、启用、禁用、卸载

### 插件类型
1. **工具插件**: 提供新的 MCP 工具
2. **资源插件**: 提供数据资源访问
3. **提示插件**: 提供 AI 提示模板
4. **UI 插件**: 扩展用户界面

## 目录结构

```
plugins/
├── README.md                 # 本文档
├── examples/                 # 示例插件
│   ├── hello-world/         # 简单的 Hello World 插件
│   ├── file-tools/          # 文件操作工具插件
│   ├── web-scraper/         # 网页抓取插件
│   └── ai-prompts/          # AI 提示模板插件
├── sdk/                     # 插件开发 SDK
│   ├── rust/               # Rust SDK
│   ├── javascript/         # JavaScript SDK
│   └── python/             # Python SDK (通过 WASM)
└── templates/              # 插件模板
    ├── tool-plugin/        # 工具插件模板
    ├── resource-plugin/    # 资源插件模板
    └── ui-plugin/          # UI 插件模板
```

## 插件开发指南

### 1. 插件清单 (plugin.json)
每个插件必须包含一个 `plugin.json` 文件：

```json
{
  "schema_version": "1.0",
  "name": "example-plugin",
  "display_name": "示例插件",
  "version": "1.0.0",
  "description": "这是一个示例插件",
  "author": "SeeU Team",
  "license": "MIT",
  "mcp_version": "1.0",
  "capabilities": {
    "provides_tools": true,
    "provides_resources": false,
    "provides_prompts": false,
    "supports_sampling": false
  },
  "permissions": [
    {
      "type": "FileSystem",
      "resource": "read",
      "description": "读取文件",
      "required": true,
      "level": "Medium"
    }
  ],
  "entry_point": "plugin.wasm",
  "target_roles": ["Developer", "DataAnalyst"]
}
```

### 2. WASM 接口

插件通过标准化的 WASM 接口与主应用通信：

```rust
// 插件必须导出的函数
#[no_mangle]
pub extern "C" fn plugin_init() -> i32;

#[no_mangle]
pub extern "C" fn plugin_get_capabilities() -> *const u8;

#[no_mangle]
pub extern "C" fn plugin_handle_request(request_ptr: *const u8, request_len: usize) -> *const u8;

#[no_mangle]
pub extern "C" fn plugin_cleanup();
```

### 3. 权限系统

插件权限分为以下级别：
- **Low**: 基本操作，无风险
- **Medium**: 文件读取、网络访问等
- **High**: 文件写入、系统信息访问
- **Critical**: 进程执行、系统配置修改

## 安全特性

1. **沙箱隔离**: 每个插件在独立的 WASM 实例中运行
2. **权限控制**: 细粒度的权限管理
3. **资源限制**: CPU、内存使用限制
4. **API 白名单**: 只能访问授权的 API
5. **代码签名**: 插件完整性验证

## 开发工具

### Rust 插件开发
```bash
# 安装 WASM 目标
rustup target add wasm32-wasi

# 编译插件
cargo build --target wasm32-wasi --release
```

### JavaScript 插件开发
```bash
# 使用 AssemblyScript
npm install -g assemblyscript
asc plugin.ts -o plugin.wasm
```

## 插件市场

插件可以通过以下方式分发：
1. **官方市场**: 经过审核的安全插件
2. **社区市场**: 社区贡献的插件
3. **本地安装**: 从文件安装插件
4. **开发模式**: 直接加载源码

## 示例插件

本目录包含以下示例插件：

1. **hello-world**: 最简单的插件示例
2. **file-tools**: 文件操作工具集
3. **web-scraper**: 网页内容抓取工具
4. **ai-prompts**: AI 提示模板集合

每个示例都包含完整的源码和构建说明。

## 开始开发

1. 选择一个模板或示例作为起点
2. 修改 `plugin.json` 配置
3. 实现插件逻辑
4. 编译为 WASM
5. 测试和调试
6. 打包发布

更多详细信息请参考各个示例插件的 README 文件。
