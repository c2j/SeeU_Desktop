# SeeU Desktop 开发指南

## 📖 目录

1. [开发环境搭建](#开发环境搭建)
2. [项目结构](#项目结构)
3. [技术架构](#技术架构)
4. [开发流程](#开发流程)
5. [模块开发](#模块开发)
6. [构建部署](#构建部署)
7. [贡献指南](#贡献指南)

## 🛠️ 开发环境搭建

### 系统要求

#### 必需软件
- **Rust**: 1.70+ (推荐使用最新稳定版)
- **Git**: 版本控制
- **IDE**: 推荐 VS Code + rust-analyzer 或 CLion

#### 平台特定要求

**Windows**
```bash
# 安装 Visual Studio Build Tools
# 或完整的 Visual Studio (包含 C++ 工具)
```

**macOS**
```bash
# 安装 Xcode 命令行工具
xcode-select --install
```

**Linux (Ubuntu/Debian)**
```bash
sudo apt update
sudo apt install build-essential libclang-dev pkg-config
sudo apt install libgtk-3-dev libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev
```

### 项目克隆与构建

```bash
# 克隆项目
git clone https://github.com/your-org/SeeU_Desktop.git
cd SeeU_Desktop

# 构建项目
cargo build

# 运行开发版本
cargo run

# 运行测试
cargo test

# 构建发布版本
cargo build --release
```

### 开发工具配置

#### VS Code 配置
推荐安装以下扩展：
- `rust-analyzer`: Rust 语言支持
- `CodeLLDB`: 调试支持
- `Better TOML`: TOML 文件支持
- `Error Lens`: 错误提示增强

#### 调试配置
在 `.vscode/launch.json` 中配置调试：
```json
{
    "version": "0.2.0",
    "configurations": [
        {
            "type": "lldb",
            "request": "launch",
            "name": "Debug SeeU Desktop",
            "cargo": {
                "args": ["build", "--bin=seeu_desktop"],
                "filter": {
                    "name": "seeu_desktop",
                    "kind": "bin"
                }
            },
            "args": [],
            "cwd": "${workspaceFolder}"
        }
    ]
}
```

## 📁 项目结构

### 目录结构
```
SeeU-Desktop/
├── src/                    # 主应用源代码
│   ├── main.rs            # 程序入口
│   ├── app.rs             # 应用主体
│   ├── ui/                # UI 组件
│   │   ├── navigation.rs  # 导航栏
│   │   ├── workspace.rs   # 工作区
│   │   ├── status_bar.rs  # 状态栏
│   │   ├── right_sidebar.rs # 右侧边栏
│   │   ├── settings.rs    # 设置界面
│   │   └── theme.rs       # 主题管理
│   ├── modules/           # 功能模块
│   │   ├── home.rs        # 主页模块
│   │   └── mod.rs         # 模块定义
│   ├── services/          # 服务层
│   │   ├── system_service.rs # 系统服务
│   │   └── mod.rs         # 服务定义
│   ├── config/            # 配置管理
│   │   └── mod.rs
│   ├── utils/             # 工具函数
│   │   └── mod.rs
│   └── platform.rs       # 平台适配
├── crates/                # 子模块 crates
│   ├── inote/             # 笔记模块
│   ├── isearch/           # 搜索模块
│   ├── aiAssist/          # AI助手模块
│   ├── itools/            # 工具集成模块 (MCP协议支持)
│   └── iterminal/         # 安全终端模块
├── assets/                # 静态资源
│   ├── fonts/             # 字体文件
│   ├── icons/             # 图标资源
│   └── themes/            # 主题配置
├── scripts/               # 构建脚本
├── tests/                 # 测试代码
├── docs/                  # 文档
└── requirement/           # 需求文档
```

### 核心文件说明

#### `src/main.rs`
应用程序入口点，负责：
- 初始化日志系统
- 配置 eframe 应用
- 启动主应用循环

#### `src/app.rs`
应用主体，包含：
- 全局状态管理
- 模块状态协调
- 主要的 `eframe::App` 实现

#### `src/ui/`
UI 组件模块，采用组件化设计：
- 每个组件负责特定的UI区域
- 使用 egui 的布局系统
- 支持主题和响应式设计

## 🏗️ 技术架构

### 架构概览

```
┌─────────────────────────────────────┐
│           egui Frontend             │
├─────────────────────────────────────┤
│         Application Layer           │
│  ┌─────────┬─────────┬─────────┐   │
│  │ iNote   │iSearch  │ iTools  │   │
│  │ Module  │ Module  │ Module  │   │
│  └─────────┴─────────┴─────────┘   │
├─────────────────────────────────────┤
│           Service Layer             │
│  ┌─────────┬─────────┬─────────┐   │
│  │ System  │   AI    │ Storage │   │
│  │Service  │Service  │Service  │   │
│  └─────────┴─────────┴─────────┘   │
├─────────────────────────────────────┤
│          Platform Layer             │
│     (Windows/macOS/Linux)           │
└─────────────────────────────────────┘
```

### 核心技术栈

#### GUI 框架
- **egui**: 即时模式 GUI 框架
- **eframe**: 跨平台窗口管理
- **egui_extras**: 扩展组件

#### 数据存储
- **SQLite**: 本地数据库（通过 rusqlite）
- **Tantivy**: 全文搜索引擎
- **Serde**: 序列化/反序列化

#### 异步处理
- **Tokio**: 异步运行时
- **Channels**: 线程间通信

#### 系统集成
- **sysinfo**: 系统信息获取
- **notify**: 文件系统监控
- **arboard**: 剪贴板操作

#### MCP协议支持
- **rmcp**: MCP协议实现
- **JSON-RPC**: JSON-RPC 2.0协议支持
- **Protocol Handler**: 自定义协议处理器

#### 终端支持
- **portable-pty**: 跨平台PTY支持
- **vte**: ANSI转义序列解析
- **crossterm**: 跨平台终端操作

### 状态管理

#### 全局状态
```rust
pub struct SeeUApp {
    // 全局状态
    pub active_module: Module,
    pub show_right_sidebar: bool,
    pub search_query: String,
    
    // 模块状态
    pub inote_state: DbINoteState,
    pub isearch_state: ISearchState,
    pub ai_assist_state: AIAssistState,
    pub itools_state: IToolsState,
    pub iterminal_state: ITerminalState,
    
    // 服务
    pub system_service: SystemService,
    pub mcp_integration_manager: McpIntegrationManager,

    // 配置
    pub theme: Theme,
    pub app_settings: AppSettings,
}
```

#### 模块状态隔离
- 每个模块维护独立的状态
- 通过消息传递进行模块间通信
- 避免状态耦合和循环依赖

## 🔄 开发流程

### Git 工作流

#### 分支策略
- `main`: 主分支，稳定版本
- `develop`: 开发分支，集成新功能
- `feature/*`: 功能分支
- `bugfix/*`: 修复分支
- `release/*`: 发布分支

#### 提交规范
```
type(scope): description

[optional body]

[optional footer]
```

类型说明：
- `feat`: 新功能
- `fix`: 修复
- `docs`: 文档
- `style`: 格式
- `refactor`: 重构
- `test`: 测试
- `chore`: 构建/工具

示例：
```
feat(inote): add markdown preview support

- Implement real-time markdown rendering
- Add preview/edit mode toggle
- Support for common markdown syntax

Closes #123
```

### 代码规范

#### Rust 代码风格
```bash
# 格式化代码
cargo fmt

# 检查代码
cargo clippy

# 运行测试
cargo test
```

#### 命名约定
- **模块**: snake_case (如 `ai_assist`)
- **结构体**: PascalCase (如 `SeeUApp`)
- **函数**: snake_case (如 `render_ui`)
- **常量**: SCREAMING_SNAKE_CASE (如 `MAX_RESULTS`)

#### 文档注释
```rust
/// Renders the main application UI
/// 
/// # Arguments
/// 
/// * `ctx` - The egui context
/// * `frame` - The eframe frame
/// 
/// # Examples
/// 
/// ```
/// app.update(&ctx, &mut frame);
/// ```
pub fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
    // 实现...
}
```

### 测试策略

#### 单元测试
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_note() {
        let mut state = INoteState::new();
        let note_id = state.create_note("Test".to_string(), "Content".to_string());
        assert!(note_id.is_some());
    }
}
```

#### 集成测试
```rust
// tests/integration_tests.rs
use seeu_desktop::*;

#[tokio::test]
async fn test_app_initialization() {
    let app = SeeUApp::new(&Default::default()).await;
    assert_eq!(app.active_module, Module::Home);
}
```

## 🔗 MCP集成开发

### MCP协议实现

#### 协议处理器
```rust
// crates/itools/src/mcp/protocol_handler.rs
pub struct ProtocolHandler {
    pub state: ProtocolState,
    pub capabilities: Option<ServerCapabilities>,
    pub request_id_counter: u64,
}

impl ProtocolHandler {
    pub fn new() -> Self {
        Self {
            state: ProtocolState::Disconnected,
            capabilities: None,
            request_id_counter: 0,
        }
    }

    pub async fn initialize(&mut self, server_info: ServerInfo) -> Result<(), ProtocolError> {
        // 实现MCP初始化握手
    }
}
```

#### JSON-RPC 2.0支持
```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub id: Option<serde_json::Value>,
    pub method: String,
    pub params: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub id: Option<serde_json::Value>,
    pub result: Option<serde_json::Value>,
    pub error: Option<JsonRpcError>,
}
```

### MCP服务器管理

#### 服务器配置
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerConfig {
    pub id: String,
    pub name: String,
    pub description: String,
    pub transport_type: TransportType,
    pub command: String,
    pub args: Vec<String>,
    pub working_directory: Option<String>,
    pub environment_variables: HashMap<String, String>,
    pub category: String,
    pub directory: String,
}
```

#### 状态管理
```rust
#[derive(Debug, Clone, PartialEq)]
pub enum ServerStatus {
    New,           // 🔴 新添加或配置修改
    Connected,     // 🟡 连接成功但未测试
    Tested,        // 🟢 功能测试通过
    Error,         // ❌ 连接或测试失败
}
```

### AI助手工具调用集成

#### 工具转换器
```rust
// crates/aiAssist/src/mcp_tools.rs
pub struct McpToolConverter;

impl McpToolConverter {
    pub fn convert_capabilities_to_tools(capabilities: &ServerCapabilities) -> Vec<Tool> {
        let mut tools = Vec::new();

        // 转换MCP工具为OpenAI Function Calling格式
        if let Some(mcp_tools) = &capabilities.tools {
            for tool in mcp_tools {
                tools.push(Tool {
                    r#type: "function".to_string(),
                    function: FunctionDefinition {
                        name: format!("mcp_call_tool_{}", tool.name),
                        description: tool.description.clone(),
                        parameters: tool.input_schema.clone(),
                    },
                });
            }
        }

        tools
    }
}
```

#### 工具执行器
```rust
pub struct McpToolExecutor;

impl McpToolExecutor {
    pub async fn execute_tool_call(
        server_id: String,
        tool_call: &ToolCall,
        mcp_manager: &mut McpManager,
    ) -> Result<String, String> {
        // 解析工具调用信息
        let mcp_info = McpToolConverter::parse_mcp_tool_call(tool_call)?;

        // 执行MCP工具调用
        match mcp_info.call_type {
            McpCallType::CallTool => {
                mcp_manager.call_tool(&server_id, &mcp_info.tool_name, &mcp_info.arguments).await
            }
            McpCallType::ReadResource => {
                mcp_manager.read_resource(&server_id, &mcp_info.resource_uri).await
            }
            McpCallType::GetPrompt => {
                mcp_manager.get_prompt(&server_id, &mcp_info.prompt_name, &mcp_info.arguments).await
            }
        }
    }
}
```

## 🧩 模块开发

### 创建新模块

#### 1. 创建 Crate
```bash
# 在 crates/ 目录下创建新模块
cargo new --lib crates/my_module
```

#### 2. 配置 Cargo.toml
```toml
# 在根目录 Cargo.toml 中添加
[workspace]
members = [
    "crates/my_module",
    # ... 其他模块
]

# 在 dependencies 中添加
my_module = { path = "crates/my_module" }
```

#### 3. 实现模块接口
```rust
// crates/my_module/src/lib.rs
use eframe::egui;

pub struct MyModuleState {
    // 模块状态
}

impl MyModuleState {
    pub fn new() -> Self {
        Self {
            // 初始化
        }
    }
}

pub fn render_my_module(ui: &mut egui::Ui, state: &mut MyModuleState) {
    ui.heading("My Module");
    // UI 实现
}
```

#### 4. 集成到主应用
```rust
// src/app.rs
use my_module::{MyModuleState, render_my_module};

pub struct SeeUApp {
    pub my_module_state: MyModuleState,
    // ...
}

// 在 render_workspace 中添加
Module::MyModule => {
    render_my_module(ui, &mut self.my_module_state);
}
```

### UI 组件开发

#### 基本组件结构
```rust
pub fn render_component(ui: &mut egui::Ui, state: &mut ComponentState) {
    ui.group(|ui| {
        ui.vertical(|ui| {
            // 标题
            ui.heading("Component Title");
            
            // 内容
            ui.horizontal(|ui| {
                // 水平布局内容
            });
            
            // 操作按钮
            ui.horizontal(|ui| {
                if ui.button("Action").clicked() {
                    // 处理点击
                }
            });
        });
    });
}
```

#### 响应式设计
```rust
pub fn render_responsive_layout(ui: &mut egui::Ui) {
    let available_width = ui.available_width();
    
    if available_width > 800.0 {
        // 宽屏布局
        ui.columns(2, |columns| {
            render_left_panel(&mut columns[0]);
            render_right_panel(&mut columns[1]);
        });
    } else {
        // 窄屏布局
        ui.vertical(|ui| {
            render_left_panel(ui);
            render_right_panel(ui);
        });
    }
}
```

## 🖥 终端模块开发

### 安全终端架构

#### 核心组件
```rust
// crates/iterminal/src/lib.rs
pub struct ITerminalState {
    pub terminal_manager: TerminalManager,
    pub show_settings: bool,
    pub show_history: bool,
    pub config: TerminalConfig,
}

// crates/iterminal/src/terminal.rs
pub struct TerminalManager {
    pub sessions: Vec<TerminalSession>,
    pub active_session_index: usize,
    pub next_session_id: u32,
}

// crates/iterminal/src/session.rs
pub struct TerminalSession {
    pub id: u32,
    pub title: String,
    pub output_buffer: Vec<String>,
    pub input_buffer: String,
    pub command_history: Vec<String>,
    pub current_directory: PathBuf,
    pub is_running_command: bool,
}
```

#### 安全命令系统
```rust
// crates/iterminal/src/command.rs
pub struct CommandExecutor {
    safe_commands: HashMap<String, CommandHandler>,
}

impl CommandExecutor {
    pub fn new() -> Self {
        let mut executor = Self {
            safe_commands: HashMap::new(),
        };

        // 注册安全命令
        executor.register_file_commands();
        executor.register_system_commands();
        executor.register_utility_commands();

        executor
    }

    fn register_file_commands(&mut self) {
        self.safe_commands.insert("ls".to_string(), Box::new(LsCommand));
        self.safe_commands.insert("cd".to_string(), Box::new(CdCommand));
        self.safe_commands.insert("pwd".to_string(), Box::new(PwdCommand));
        // ... 更多文件命令
    }
}

pub trait CommandHandler {
    fn execute(&self, args: &[String], session: &mut TerminalSession) -> CommandResult;
    fn description(&self) -> &'static str;
    fn usage(&self) -> &'static str;
}
```

#### 异步命令执行
```rust
pub async fn execute_command_async(
    command: String,
    args: Vec<String>,
    session: &mut TerminalSession,
) -> Result<String, String> {
    // 检查命令是否在安全列表中
    if !is_safe_command(&command) {
        return Err(format!("命令 '{}' 不在安全命令列表中", command));
    }

    // 异步执行命令
    let output = tokio::task::spawn_blocking(move || {
        execute_safe_command(&command, &args)
    }).await.map_err(|e| e.to_string())?;

    // 更新会话状态
    session.add_output(&output);
    session.add_to_history(&format!("{} {}", command, args.join(" ")));

    Ok(output)
}
```

### 配置管理
```rust
// crates/iterminal/src/config.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalConfig {
    // 外观设置
    pub font_size: f32,
    pub font_scale: f32,
    pub scroll_buffer_lines: usize,

    // 行为设置
    pub default_shell: String,
    pub enable_bell: bool,
    pub cursor_blink_interval: u64,
    pub tab_size: usize,

    // 颜色设置
    pub background_color: [f32; 4],
    pub text_color: [f32; 4],
    pub cursor_color: [f32; 4],
}

impl Default for TerminalConfig {
    fn default() -> Self {
        Self {
            font_size: 14.0,
            font_scale: 1.0,
            scroll_buffer_lines: 1000,
            default_shell: default_shell(),
            enable_bell: true,
            cursor_blink_interval: 500,
            tab_size: 4,
            background_color: [0.0, 0.0, 0.0, 1.0],
            text_color: [1.0, 1.0, 1.0, 1.0],
            cursor_color: [1.0, 1.0, 1.0, 1.0],
        }
    }
}
```

### 状态管理模式

#### 状态结构设计
```rust
#[derive(Debug, Clone)]
pub struct ModuleState {
    // 核心数据
    pub data: Vec<DataItem>,
    
    // UI 状态
    pub selected_item: Option<usize>,
    pub show_dialog: bool,
    
    // 异步状态
    pub is_loading: bool,
    pub error_message: Option<String>,
}
```

#### 状态更新模式
```rust
impl ModuleState {
    pub fn handle_action(&mut self, action: Action) {
        match action {
            Action::SelectItem(index) => {
                self.selected_item = Some(index);
            }
            Action::ShowDialog => {
                self.show_dialog = true;
            }
            Action::LoadData => {
                self.is_loading = true;
                // 触发异步加载
            }
        }
    }
}
```

## 🚀 构建部署

### 本地构建

#### 开发构建
```bash
# 快速构建（调试版本）
cargo build

# 运行
cargo run

# 带日志运行
RUST_LOG=debug cargo run
```

#### 发布构建
```bash
# 优化构建
cargo build --release

# 生成的可执行文件位于
# target/release/seeu_desktop
```

### 跨平台构建

#### 使用构建脚本
```bash
# 构建所有平台
./scripts/build-all.sh

# 构建特定平台
./scripts/build-linux.sh
./scripts/build-macos-native.sh
./scripts/build-windows.sh
```

#### 手动跨平台构建
```bash
# 添加目标平台
rustup target add x86_64-pc-windows-gnu
rustup target add x86_64-apple-darwin
rustup target add x86_64-unknown-linux-gnu

# 构建特定平台
cargo build --release --target x86_64-pc-windows-gnu
```

### Docker 构建

#### Linux 构建
```dockerfile
# Dockerfile.linux
FROM rust:1.70 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bullseye-slim
RUN apt-get update && apt-get install -y libgtk-3-0
COPY --from=builder /app/target/release/seeu_desktop /usr/local/bin/
CMD ["seeu_desktop"]
```

#### 构建命令
```bash
# 构建 Docker 镜像
docker build -f Dockerfile.linux -t seeu-desktop:latest .

# 运行
docker run -it --rm \
  -e DISPLAY=$DISPLAY \
  -v /tmp/.X11-unix:/tmp/.X11-unix \
  seeu-desktop:latest
```

## 📦 依赖项管理

### 核心依赖
- `egui` - GUI框架
- `eframe` - 跨平台窗口管理
- `tokio` - 异步运行时
- `uuid` - 唯一标识符
- `chrono` - 时间处理
- `serde` - 序列化

### 数据库和搜索
- `rusqlite` - SQLite数据库
- `tantivy` - 全文搜索引擎
- `sqlx` - 异步SQL工具包

### MCP协议支持
- `rmcp` - MCP协议实现
- `serde_json` - JSON序列化
- `reqwest` - HTTP客户端
- `tungstenite` - WebSocket支持

### 终端相关
- `portable-pty` - 跨平台PTY支持
- `vte` - ANSI转义序列解析
- `crossterm` - 跨平台终端操作

### AI集成
- `base64` - Base64编码
- `mime_guess` - MIME类型检测
- `image` - 图像处理

### 工具库
- `dirs` - 系统目录
- `regex` - 正则表达式
- `hostname` - 主机名获取
- `sysinfo` - 系统信息
- `notify` - 文件系统监控
- `arboard` - 剪贴板操作

### 开发依赖
- `cargo-watch` - 自动重新编译
- `cargo-expand` - 宏展开工具
- `cargo-audit` - 安全审计
- `cargo-deny` - 依赖检查

### 发布流程

#### 版本发布步骤
1. **更新版本号**: 修改 `Cargo.toml` 中的版本
2. **更新变更日志**: 记录新功能和修复
3. **运行测试**: 确保所有测试通过
4. **构建发布版本**: 生成各平台可执行文件
5. **创建发布标签**: `git tag v0.1.0`
6. **推送标签**: `git push origin v0.1.0`

#### 自动化发布
```yaml
# .github/workflows/release.yml
name: Release
on:
  push:
    tags: ['v*']
jobs:
  build:
    strategy:
      matrix:
        os: [ubuntu-latest, windows-latest, macos-latest]
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - run: cargo build --release
      - uses: actions/upload-artifact@v3
        with:
          name: seeu-desktop-${{ matrix.os }}
          path: target/release/seeu_desktop*
```

## 🤝 贡献指南

### 贡献流程

1. **Fork 项目**: 在 GitHub 上 fork 项目
2. **创建分支**: `git checkout -b feature/my-feature`
3. **开发功能**: 实现新功能或修复
4. **编写测试**: 确保代码质量
5. **提交代码**: 遵循提交规范
6. **创建 PR**: 提交 Pull Request
7. **代码审查**: 响应审查意见
8. **合并代码**: 审查通过后合并

### 代码审查标准

#### 功能性
- [ ] 功能按预期工作
- [ ] 边界情况处理正确
- [ ] 错误处理完善

#### 代码质量
- [ ] 代码清晰易读
- [ ] 遵循项目规范
- [ ] 适当的注释和文档

#### 性能
- [ ] 无明显性能问题
- [ ] 内存使用合理
- [ ] 异步操作正确

#### 测试
- [ ] 包含适当的测试
- [ ] 测试覆盖率足够
- [ ] 测试可靠稳定

### 问题报告

#### Bug 报告模板
```markdown
## Bug 描述
简要描述遇到的问题

## 复现步骤
1. 打开应用
2. 点击...
3. 输入...
4. 观察到错误

## 期望行为
描述期望的正确行为

## 实际行为
描述实际发生的错误行为

## 环境信息
- 操作系统: 
- 应用版本: 
- Rust 版本: 

## 附加信息
- 错误日志
- 截图
- 其他相关信息
```

#### 功能请求模板
```markdown
## 功能描述
简要描述建议的新功能

## 使用场景
描述什么情况下需要这个功能

## 详细设计
详细描述功能的工作方式

## 替代方案
是否有其他解决方案

## 优先级
功能的重要性和紧急程度
```

---

感谢您对 SeeU Desktop 项目的贡献！
