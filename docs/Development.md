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
│   ├── itools/            # 工具集成模块
│   └── iterminal/         # 终端模块
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
    
    // 服务
    pub system_service: SystemService,
    
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
