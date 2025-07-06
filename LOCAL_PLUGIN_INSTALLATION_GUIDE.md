# iTools本地插件安装指南

## 概述

iTools现在支持从本地文件系统安装插件包，解决了网络连接问题和提供了更灵活的插件分发方式。

## 功能特性

### ✅ 已实现的功能

1. **本地文件安装**
   - 支持从本地文件系统选择插件包
   - 文件对话框选择器
   - 支持多种插件包格式

2. **插件包格式支持**
   - `.itpkg` - iTools标准插件包（tar.gz格式）
   - `.zip` - ZIP格式插件包
   - `.tar.gz` - 标准tar.gz格式

3. **完整的安装流程**
   - 文件验证和解压
   - 插件元数据验证
   - 运行时加载和服务启动

4. **UI集成**
   - 插件市场界面添加"📁 从本地安装"按钮
   - 文件选择对话框
   - 安装进度显示

## 插件包格式规范

### 标准目录结构
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
└── resources/           # 资源文件目录
    ├── templates/       # 模板文件
    └── data/           # 数据文件
```

### 核心文件说明

#### manifest.json（插件清单）
定义插件的能力、权限和依赖关系：
```json
{
  "schema_version": "1.0",
  "mcp_version": "2024-11-05",
  "capabilities": {
    "provides_tools": true,
    "provides_resources": false,
    "provides_prompts": false
  },
  "permissions": [...],
  "tools": [...],
  "configuration": {...}
}
```

#### metadata.json（插件元数据）
包含插件的基本信息：
```json
{
  "id": "com.example.plugin-name",
  "name": "plugin-name",
  "display_name": "插件显示名称",
  "version": "1.0.0",
  "description": "插件描述",
  "author": "开发者",
  "plugin_type": "javascript",
  "entry_point": "src/plugin.js",
  "runtime_requirements": {
    "memory_mb": 64,
    "cpu_cores": 1,
    "disk_mb": 10
  }
}
```

## 使用方法

### 1. 在插件市场安装本地插件

1. 打开iTools应用
2. 进入插件市场页面
3. 点击"📁 从本地安装"按钮
4. 在文件对话框中选择插件包文件（.itpkg、.zip、.tar.gz）
5. 等待安装完成
6. 在已安装插件列表中启用插件

### 2. 支持的插件类型

- **JavaScript插件**: 入口点为`.js`文件
- **Python插件**: 入口点为`.py`文件
- **WASM插件**: 入口点为`.wasm`文件
- **MCP服务器插件**: 入口点为`.json`配置文件

## 示例插件包

项目中提供了两个示例插件包：

### 1. 简单计算器（JavaScript）
- **文件**: `examples/plugins/packages/simple-calculator.itpkg`
- **功能**: 基本数学运算和单位转换
- **类型**: JavaScript插件
- **大小**: ~8KB

### 2. 文本处理器（Python）
- **文件**: `examples/plugins/packages/text-processor.itpkg`
- **功能**: 文本分析、格式转换、内容提取
- **类型**: Python插件
- **大小**: ~8KB

## 创建自定义插件包

### 1. 使用构建脚本

```bash
# 构建所有插件包
./examples/plugins/build_packages.sh

# 构建特定插件包
./examples/plugins/build_packages.sh simple-calculator
```

### 2. 手动创建

```bash
# 使用tar创建插件包
tar -czf my-plugin.itpkg -C my-plugin/ .

# 使用zip创建插件包
cd my-plugin && zip -r ../my-plugin.zip .
```

## 技术实现

### 核心组件

1. **PluginManager**
   - `install_plugin_from_file()` - 本地文件安装入口
   - `process_install_from_file_task()` - 安装任务处理
   - `validate_and_load_plugin()` - 插件验证和加载

2. **UI组件**
   - 文件选择对话框（rfd crate）
   - 安装进度显示
   - 错误处理和用户反馈

3. **插件验证**
   - 文件格式验证
   - JSON结构验证
   - 入口点存在性检查
   - 插件类型匹配验证

### 安装流程

1. **文件选择**: 用户通过文件对话框选择插件包
2. **格式验证**: 检查文件扩展名和格式
3. **解压提取**: 解压插件包到临时目录
4. **元数据验证**: 验证manifest.json和metadata.json
5. **入口点检查**: 确认插件入口点文件存在
6. **插件注册**: 将插件添加到系统中
7. **运行时加载**: 根据插件类型加载到相应运行时

## 错误处理

### 常见错误和解决方案

1. **不支持的文件格式**
   - 错误: "Unsupported plugin package format"
   - 解决: 确保文件扩展名为.itpkg、.zip或.tar.gz

2. **缺少必需文件**
   - 错误: "Missing required file: manifest.json"
   - 解决: 确保插件包包含manifest.json和metadata.json

3. **JSON格式错误**
   - 错误: "Invalid manifest.json"
   - 解决: 检查JSON文件语法和结构

4. **入口点不存在**
   - 错误: "Entry point not found"
   - 解决: 确保metadata.json中指定的入口点文件存在

## 最佳实践

1. **版本管理**: 使用语义化版本号
2. **文档完整**: 提供详细的README和使用说明
3. **测试验证**: 在打包前测试插件功能
4. **安全考虑**: 明确声明所需权限
5. **兼容性**: 指定支持的平台和架构

## 后续计划

1. **插件签名验证**: 添加数字签名支持
2. **依赖管理**: 自动解析和安装依赖
3. **版本更新**: 支持插件版本升级
4. **批量安装**: 支持同时安装多个插件
5. **插件商店**: 集成官方插件商店

## 故障排除

### 日志查看
安装过程中的详细日志会显示在控制台中，包括：
- 文件读取状态
- 解压进度
- 验证结果
- 错误详情

### 常见问题
1. **权限问题**: 确保有足够权限读取插件文件和写入安装目录
2. **磁盘空间**: 确保有足够空间解压和安装插件
3. **文件损坏**: 重新下载或重新创建插件包

通过本地插件安装功能，用户现在可以：
- 离线安装插件
- 安装自定义开发的插件
- 分享插件包文件
- 避免网络连接问题

这大大提高了iTools插件系统的灵活性和可用性。
