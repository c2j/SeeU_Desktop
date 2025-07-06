# iTools插件包格式规范

## 概述

iTools插件包是一个标准化的压缩文件（.tar.gz或.zip），包含插件的所有必要文件和元数据。

## 包格式

### 文件扩展名
- `.itpkg` - iTools插件包（实际上是tar.gz格式）
- `.zip` - 兼容的ZIP格式插件包

### 目录结构

```
plugin-name.itpkg (tar.gz)
├── manifest.json          # 插件清单文件（必需）
├── metadata.json          # 插件元数据（必需）
├── icon.png              # 插件图标（可选）
├── README.md             # 说明文档（可选）
├── LICENSE               # 许可证文件（可选）
├── src/                  # 源代码目录
│   ├── main.wasm        # WASM插件主文件
│   ├── plugin.js        # JavaScript插件文件
│   └── plugin.py        # Python插件文件
├── config/              # 配置文件目录
│   ├── default.json     # 默认配置
│   └── schema.json      # 配置模式
├── resources/           # 资源文件目录
│   ├── templates/       # 模板文件
│   ├── data/           # 数据文件
│   └── assets/         # 静态资源
└── tests/              # 测试文件目录
    ├── unit/           # 单元测试
    └── integration/    # 集成测试
```

## 核心文件规范

### 1. manifest.json（插件清单）

```json
{
  "schema_version": "1.0",
  "mcp_version": "2024-11-05",
  "capabilities": {
    "provides_resources": true,
    "provides_tools": true,
    "provides_prompts": false,
    "supports_sampling": false,
    "supports_notifications": true,
    "supports_progress": true
  },
  "permissions": [
    {
      "type": "file_system",
      "paths": ["/tmp", "~/Documents"],
      "operations": ["read", "write"]
    },
    {
      "type": "network",
      "hosts": ["api.example.com"],
      "ports": [80, 443]
    }
  ],
  "dependencies": [
    {
      "name": "python",
      "version": ">=3.8",
      "optional": false
    }
  ],
  "resources": [
    {
      "uri": "file://data/sample.json",
      "name": "Sample Data",
      "description": "Sample data file",
      "mime_type": "application/json"
    }
  ],
  "tools": [
    {
      "name": "process_data",
      "description": "Process data files",
      "input_schema": {
        "type": "object",
        "properties": {
          "file_path": {"type": "string"},
          "format": {"type": "string", "enum": ["json", "csv"]}
        },
        "required": ["file_path"]
      }
    }
  ],
  "prompts": [],
  "configuration": {
    "schema": "config/schema.json",
    "default": "config/default.json"
  }
}
```

### 2. metadata.json（插件元数据）

```json
{
  "id": "com.example.data-processor",
  "name": "data-processor",
  "display_name": "Data Processor",
  "version": "1.0.0",
  "description": "A powerful data processing plugin",
  "author": "Example Developer",
  "email": "dev@example.com",
  "website": "https://example.com",
  "repository": "https://github.com/example/data-processor",
  "license": "MIT",
  "keywords": ["data", "processing", "analytics"],
  "categories": ["数据处理", "分析工具"],
  "icon": "icon.png",
  "screenshots": ["screenshots/main.png", "screenshots/config.png"],
  "min_itools_version": "0.1.0",
  "supported_platforms": ["windows", "macos", "linux"],
  "supported_architectures": ["x86_64", "arm64"],
  "created_at": "2024-01-01T00:00:00Z",
  "updated_at": "2024-07-06T00:00:00Z",
  "plugin_type": "wasm",
  "entry_point": "src/main.wasm",
  "runtime_requirements": {
    "memory_mb": 64,
    "cpu_cores": 1,
    "disk_mb": 10
  }
}
```

## 插件类型

### 1. WASM插件
- **入口文件**: `src/main.wasm`
- **特点**: 高性能、安全沙箱、跨平台
- **适用场景**: 计算密集型任务、数据处理

### 2. JavaScript插件
- **入口文件**: `src/plugin.js`
- **特点**: 快速开发、丰富生态
- **适用场景**: UI交互、API调用

### 3. Python插件
- **入口文件**: `src/plugin.py`
- **特点**: 丰富的库支持、AI/ML友好
- **适用场景**: 数据科学、机器学习

### 4. MCP服务器插件
- **入口文件**: `src/mcp_server.json`
- **特点**: 标准MCP协议、工具集成
- **适用场景**: 外部工具集成、API封装

## 配置文件

### config/schema.json（配置模式）
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "type": "object",
  "properties": {
    "api_key": {
      "type": "string",
      "description": "API密钥",
      "minLength": 1
    },
    "timeout": {
      "type": "integer",
      "description": "超时时间（秒）",
      "minimum": 1,
      "maximum": 300,
      "default": 30
    },
    "debug": {
      "type": "boolean",
      "description": "启用调试模式",
      "default": false
    }
  },
  "required": ["api_key"]
}
```

### config/default.json（默认配置）
```json
{
  "timeout": 30,
  "debug": false,
  "batch_size": 100,
  "output_format": "json"
}
```

## 权限系统

### 权限类型
- `file_system`: 文件系统访问
- `network`: 网络访问
- `system`: 系统调用
- `ui`: 用户界面操作
- `data`: 数据访问

### 权限级别
- `read`: 只读访问
- `write`: 写入访问
- `execute`: 执行权限
- `admin`: 管理员权限

## 打包工具

### 命令行工具（建议）
```bash
# 创建插件包
itools-pack create my-plugin/

# 验证插件包
itools-pack validate my-plugin.itpkg

# 安装插件包
itools-pack install my-plugin.itpkg
```

### 手动打包
```bash
# 使用tar创建插件包
tar -czf my-plugin.itpkg -C my-plugin/ .

# 使用zip创建插件包
cd my-plugin && zip -r ../my-plugin.zip .
```

## 安装流程

1. **验证包格式**: 检查文件结构和必需文件
2. **解析元数据**: 读取manifest.json和metadata.json
3. **权限检查**: 验证插件请求的权限
4. **依赖检查**: 确认系统依赖是否满足
5. **解压安装**: 将文件解压到插件目录
6. **注册插件**: 在系统中注册插件信息
7. **初始化**: 运行插件初始化代码

## 最佳实践

1. **版本管理**: 使用语义化版本号
2. **文档完整**: 提供详细的README和API文档
3. **测试覆盖**: 包含完整的单元测试和集成测试
4. **安全考虑**: 最小权限原则，明确声明所需权限
5. **性能优化**: 合理设置资源需求，避免过度消耗
6. **兼容性**: 明确支持的平台和架构
7. **错误处理**: 提供友好的错误信息和恢复机制
