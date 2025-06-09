# SeeU-Desktop (Rust + egui + eframe)

欢迎来到 SeeU-Desktop 项目！这是一个完全使用 Rust 原生技术栈构建的现代化桌面应用程序，基于 egui 和 eframe 框架实现跨平台图形界面，旨在提供极致性能和原生体验。

## ✨ 项目概述

SeeU-Desktop 采用纯 Rust 实现，摒弃了混合架构的复杂性，通过 egui 和 eframe 框架提供高性能、低延迟的用户界面。项目特点包括：

- **纯 Rust 实现**：从核心逻辑到界面渲染，全部使用 Rust 语言
- **高性能渲染**：基于 egui 的即时模式 GUI，渲染效率高，资源占用低
- **模块化设计**：核心功能以独立模块形式实现，便于维护和扩展
- **跨平台原生体验**：同时支持 Windows、macOS 和 Linux，提供一致的原生体验
- **低资源占用**：相比混合架构，内存占用更少，启动更快

## 🚀 主要功能

### 核心功能

- **跨平台支持**：Windows、macOS 和 Linux 平台一致体验
- **多工作区**：终端、文件管理、数据分析等多功能工作区
- **AI 助手**：内置智能助手，支持自然语言交互
- **系统资源监控**：实时监控 CPU、内存使用情况

### 专业模块

- **iNote**：高性能笔记应用
  - 支持 Markdown 和富文本编辑
  - 树形笔记本结构
  - 标签云管理
  - 附件支持

- **iSearch**：文件搜索功能
  - 高效文件索引
  - 复合查询支持
  - 实时搜索结果
  - 多格式文件内容搜索

## 🏁 开始使用

### 先决条件

确保您的系统已安装以下软件：

- [Rust](https://www.rust-lang.org/tools/install) (建议使用最新稳定版)
- 根据您的操作系统，可能需要安装额外的构建工具：
  - **Windows**：安装 Visual Studio 构建工具
  - **macOS**：安装 Xcode 命令行工具 (`xcode-select --install`)
  - **Linux**：安装 `build-essential`、`libclang-dev` 和 `pkg-config`
- **vcpkg**（推荐）：用于跨平台依赖管理和静态链接

### 安装

1. 克隆或下载此仓库到您的本地计算机
2. 打开终端，导航到项目根目录：
   ```bash
   cd SeeU-Desktop
   ```
3. （推荐）设置 vcpkg 依赖管理：
   ```bash
   ./scripts/setup-vcpkg.sh
   ```
4. 构建项目：
   ```bash
   cargo build --release
   ```

### 运行应用程序

要启动应用程序，请运行：

```bash
cargo run --release
```

### 构建可执行文件

要为当前平台构建可执行文件，请运行：

```bash
cargo build --release
```

编译后的可执行文件将位于 `target/release/` 目录中。

### 跨平台构建

SeeU-Desktop 支持跨平台编译，可以在一个平台上构建适用于其他平台的可执行文件。

#### 跨平台构建方法

项目支持四种跨平台构建方法：

1. **vcpkg 构建**：使用 vcpkg 包管理器进行跨平台构建（推荐）
2. **原生构建**：在目标平台上直接构建
3. **交叉编译**：使用交叉编译工具链
4. **Docker 构建**：使用 Docker 容器进行隔离构建

#### 安装跨平台编译工具

首先，安装目标平台的 Rust 工具链：

```bash
# 安装 Windows 目标
rustup target add x86_64-pc-windows-msvc

# 安装 Linux 目标
rustup target add x86_64-unknown-linux-gnu

# 安装 macOS 目标
rustup target add x86_64-apple-darwin

# 如果需要支持 Apple Silicon (M1/M2)
rustup target add aarch64-apple-darwin
```

对于交叉编译，您可能还需要安装额外的工具：

```bash
# 在 macOS 上安装 Windows 交叉编译工具
brew install mingw-w64

# 在 macOS 上安装 Linux 交叉编译工具
brew install FiloSottile/musl-cross/musl-cross
```

#### 使用 Docker 进行构建（推荐）

如果您安装了 Docker，构建脚本会自动使用 Docker 进行跨平台构建，无需安装额外的交叉编译工具。这是最简单可靠的方法。

#### 使用 vcpkg 构建（推荐）

使用 vcpkg 进行跨平台构建，支持静态链接：

```bash
# 设置 vcpkg（首次使用）
./scripts/setup-vcpkg.sh

# 使用 vcpkg 构建特定平台
./scripts/build-vcpkg.sh --target linux-x64
./scripts/build-vcpkg.sh --target linux-arm64
./scripts/build-vcpkg.sh --target windows-x64
./scripts/build-vcpkg.sh --target windows-x86

# 构建所有支持的平台
./scripts/build-vcpkg.sh --target all

# 启用静态链接（默认）
./scripts/build-vcpkg.sh --target linux-x64 --static true
```

#### 使用传统构建脚本

项目还提供了传统的跨平台构建脚本：

```bash
# 构建 Windows 版本
./scripts/build-windows.sh

# 构建 Linux 版本
./scripts/build-linux.sh

# 构建 macOS 版本
./scripts/build-macos.sh

# 构建所有平台版本
./scripts/build-all.sh
```

构建脚本会自动检测可用的构建方法（vcpkg、Docker 或原生），并选择最合适的方式进行构建。

构建完成后，可执行文件将位于 `dist/` 目录中。

## 📂 项目结构

```
SeeU-Desktop/
├── README.md            # 本项目说明文件
├── Cargo.toml           # Rust 项目配置
├── Cargo.lock           # 依赖锁定文件
├── assets/              # 静态资源
│   ├── fonts/           # 字体文件
│   ├── icons/           # 图标资源
│   └── themes/          # 主题配置
├── src/                 # 主程序源代码
│   ├── main.rs          # 程序入口点
│   ├── app.rs           # 应用程序主体
│   ├── ui/              # UI 组件
│   ├── modules/         # 功能模块
│   ├── services/        # 服务层
│   └── utils/           # 工具函数
├── crates/              # 子模块 crates
│   ├── inote/           # iNote 模块
│   └── isearch/         # iSearch 模块
└── tests/               # 测试代码
```

## 🔧 技术实现

SeeU-Desktop 使用以下技术：

- **egui**：即时模式 GUI 库
- **eframe**：跨平台窗口和渲染框架
- **serde**：序列化和反序列化
- **tokio**：异步运行时
- **tantivy**：全文搜索引擎
- **notify**：文件系统监控
- **pulldown-cmark**：Markdown 渲染

## 📝 贡献

欢迎贡献代码、报告问题或提出改进建议！请参阅 [CONTRIBUTING.md](CONTRIBUTING.md) 了解更多信息。

## 📄 许可证

本项目采用 MIT 许可证 - 详情请参阅 [LICENSE](LICENSE) 文件。
