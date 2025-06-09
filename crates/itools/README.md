# iTools - AI 工具集成模块

iTools 是 SeeU Desktop 应用程序的 AI 工具集成模块，基于 Model Context Protocol (MCP) 实现，为不同角色的用户提供个性化的 AI 工具和插件管理功能。

## 功能特性

### 🔧 核心功能
- **多角色权限系统**: 支持业务用户、开发者、运维人员、管理员四种角色
- **插件市场**: 浏览、搜索、安装和管理 AI 工具插件
- **MCP 协议支持**: 基于标准 Model Context Protocol 实现插件通信
- **安全沙箱**: 为插件提供安全的执行环境
- **AI 助手集成**: 与现有 aiAssist 模块集成，支持插件调用和任务自动化

> **注意**: iTools 模块专注于插件管理和 MCP 协议支持，AI 对话功能由现有的 `aiAssist` 模块提供，避免功能重复。

### 🎯 角色定制
每个角色都有专门的界面组件和权限设置：

#### 业务用户
- 数据看板和可视化图表
- 文档模板和一键分析
- 受限的插件访问权限

#### 开发者
- 代码编辑器和 API 调试器
- Git 集成和版本控制
- 插件开发工具

#### 运维人员
- 系统监控和性能分析
- 日志查看器和安全审计
- 插件状态监控

#### 管理员
- 角色权限管理
- 插件市场审核
- 安全策略配置

### 🔌 预置插件
- **文件系统 MCP 服务器**: 安全的文件读写操作
- **Git 集成**: 代码仓库管理和版本控制
- **BI 连接器**: Tableau/Power BI 集成
- **系统监控**: 实时性能监控和告警

## 技术架构

### 模块结构
```
itools/
├── src/
│   ├── lib.rs              # 主模块入口
│   ├── state.rs            # 状态管理
│   ├── roles/              # 角色权限系统
│   ├── plugins/            # 插件管理
│   │   ├── manager.rs      # 插件管理器
│   │   ├── marketplace.rs  # 插件市场
│   │   ├── sandbox.rs      # 安全沙箱
│   │   └── presets.rs      # 预置插件
│   ├── mcp/                # MCP 协议实现
│   │   ├── client.rs       # MCP 客户端
│   │   ├── protocol.rs     # 协议定义
│   │   └── transport.rs    # 传输层
│   ├── security/           # 安全模块
│   │   ├── permissions.rs  # 权限管理
│   │   ├── audit.rs        # 审计日志
│   │   └── policy.rs       # 安全策略
│   └── ui/                 # 用户界面
│       ├── main_ui.rs      # 主界面
│       ├── dashboard.rs    # 仪表板
│       ├── marketplace.rs  # 市场界面
│       ├── plugins.rs      # 插件管理界面
│       └── settings.rs     # 设置界面
└── tests/
    └── integration_test.rs # 集成测试
```

### 核心技术
- **Rust**: 高性能、内存安全的系统编程语言
- **egui**: 即时模式 GUI 框架
- **MCP**: Model Context Protocol 标准协议
- **tokio**: 异步运行时
- **serde**: 序列化/反序列化
- **rusqlite**: 本地数据库存储
- **wasmtime**: WASM 运行时（插件沙箱）

## 使用方法

### 基本使用
```rust
use itools::{initialize, render_itools, update_itools};

// 初始化 iTools 模块
let mut itools_state = initialize();
itools_state.initialize();

// 在 UI 循环中渲染
render_itools(ui, &mut itools_state);

// 在更新循环中处理后台任务
update_itools(&mut itools_state);
```

### 角色切换
```rust
use itools::roles::UserRole;

// 切换到开发者角色
itools_state.current_role = UserRole::Developer;

// 检查权限
if itools_state.current_role.has_permission("read", "/code/project.rs") {
    // 执行操作
}
```

### 插件管理
```rust
// 安装插件
itools_state.plugin_manager.install_plugin(
    plugin_id,
    download_url
)?;

// 启用插件
itools_state.plugin_manager.enable_plugin(
    plugin_id,
    &itools_state.current_role
)?;
```

## 安全特性

### 权限控制
- 基于角色的访问控制 (RBAC)
- 细粒度权限管理
- 时间窗口和 IP 限制

### 审计日志
- 完整的操作审计
- 安全事件监控
- 风险等级评估

### 插件沙箱
- 资源使用限制
- 网络访问控制
- 文件系统隔离

## 开发指南

### 添加新插件
1. 实现 MCP 协议接口
2. 定义插件清单 (manifest)
3. 配置权限和依赖
4. 添加到插件市场

### 自定义角色
1. 扩展 `UserRole` 枚举
2. 定义角色权限
3. 配置 UI 组件
4. 更新权限检查逻辑

### 扩展 UI 组件
1. 在 `UiComponent` 中添加新组件
2. 实现渲染逻辑
3. 配置角色访问权限
4. 添加到仪表板

## 测试

运行测试套件：
```bash
cargo test --package itools
```

测试覆盖：
- 模块初始化
- 角色权限系统
- 插件市场功能
- AI 助手会话
- 审计日志记录

## 许可证

MIT License - 详见 LICENSE 文件

## 贡献

欢迎提交 Issue 和 Pull Request！

## 更新日志

### v0.1.0 (2024-01-01)
- 初始版本发布
- 基础 MCP 协议支持
- 多角色权限系统
- 插件市场和管理
- AI 助手集成
- 安全审计功能
