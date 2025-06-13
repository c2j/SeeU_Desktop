# 构建脚本使用指南

本文档详细介绍了 SeeU Desktop 项目的新构建脚本系统，包括 vcpkg 集成和跨平台构建功能。

## 脚本概览

### 主要构建脚本

1. **`scripts/build-linux.sh`** - Linux 平台构建
2. **`scripts/build-windows.sh`** - Windows 平台构建  
3. **`scripts/build-macos.sh`** - macOS 平台构建
4. **`scripts/build-all.sh`** - 多平台统一构建
5. **`scripts/build-vcpkg.sh`** - 纯 vcpkg 构建工具

### 辅助脚本

1. **`scripts/setup-vcpkg.sh`** - vcpkg 安装和配置
2. **`scripts/test-vcpkg.sh`** - vcpkg 配置验证
3. **`scripts/test-build-scripts.sh`** - 构建脚本测试

## 详细使用说明

### 1. Linux 构建脚本 (`build-linux.sh`)

#### 基本用法
```bash
# 默认构建（x64，静态链接，release 模式）
./scripts/build-linux.sh

# 查看帮助
./scripts/build-linux.sh --help
```

#### 高级选项
```bash
# 构建 ARM64 版本
./scripts/build-linux.sh --arch arm64

# 构建所有架构（包含 ARM64）
./scripts/build-linux.sh --arch all --include-arm64

# 使用动态链接
./scripts/build-linux.sh --static false

# 调试模式构建
./scripts/build-linux.sh --mode debug
```

#### 构建策略
1. **vcpkg 优先**: 如果检测到 vcpkg，优先使用
2. **原生构建**: 在 Linux 系统上回退到原生构建
3. **Docker 构建**: 在非 Linux 系统上使用 Docker

### 2. Windows 构建脚本 (`build-windows.sh`)

#### 基本用法
```bash
# 默认构建（x64，静态链接，release 模式）
./scripts/build-windows.sh

# 查看帮助
./scripts/build-windows.sh --help
```

#### 高级选项
```bash
# 构建 32位 版本
./scripts/build-windows.sh --arch x86

# 构建所有架构（包含 32位）
./scripts/build-windows.sh --arch all --include-32bit

# 使用动态链接
./scripts/build-windows.sh --static false

# 调试模式构建
./scripts/build-windows.sh --mode debug
```

#### 构建策略
1. **vcpkg 优先**: 如果检测到 vcpkg，优先使用
2. **原生构建**: 在 Windows 系统上回退到原生构建
3. **Docker 构建**: 在非 Windows 系统上使用 Docker

### 3. macOS 构建脚本 (`build-macos.sh`)

#### 基本用法
```bash
# 默认构建（当前架构，release 模式）
./scripts/build-macos.sh

# 查看帮助
./scripts/build-macos.sh --help
```

#### 高级选项
```bash
# 构建特定架构
./scripts/build-macos.sh --arch x64
./scripts/build-macos.sh --arch arm64

# 构建所有架构并创建通用二进制
./scripts/build-macos.sh --arch all --universal true

# 调试模式构建
./scripts/build-macos.sh --mode debug
```

#### 特殊功能
- 自动创建 macOS 应用程序包 (.app)
- 支持通用二进制文件 (Universal Binary)
- 只能在 macOS 系统上运行

### 4. 统一构建脚本 (`build-all.sh`)

#### 基本用法
```bash
# 构建所有支持的平台
./scripts/build-all.sh

# 查看帮助
./scripts/build-all.sh --help
```

#### 高级选项
```bash
# 包含所有架构
./scripts/build-all.sh --include-32bit --include-arm64

# 选择特定平台
./scripts/build-all.sh --platforms linux,windows

# 使用动态链接
./scripts/build-all.sh --static false

# 调试模式构建
./scripts/build-all.sh --mode debug
```

#### 智能平台检测
- **Linux**: 默认构建 linux,windows
- **macOS**: 默认构建 macos,linux,windows  
- **Windows**: 默认构建 windows,linux

### 5. vcpkg 专用构建脚本 (`build-vcpkg.sh`)

#### 基本用法
```bash
# 构建特定目标
./scripts/build-vcpkg.sh --target linux-x64
./scripts/build-vcpkg.sh --target windows-x64
./scripts/build-vcpkg.sh --target linux-arm64

# 构建所有目标
./scripts/build-vcpkg.sh --target all
```

#### 支持的目标
- `linux-x64` - Linux x86_64
- `linux-arm64` - Linux ARM64
- `windows-x64` - Windows x86_64
- `windows-x86` - Windows x86 (32位)
- `all` - 所有支持的目标

## 构建输出

### 目录结构
```
dist/
├── linux/
│   ├── seeu_desktop           # 主二进制文件
│   ├── seeu-desktop.sh        # 启动脚本
│   ├── assets/                # 资源文件
│   ├── x64/                   # x64 特定版本（如果构建了多架构）
│   └── arm64/                 # ARM64 特定版本（如果构建了多架构）
├── windows/
│   ├── seeu_desktop.exe       # 主二进制文件
│   ├── seeu-desktop.bat       # 启动脚本
│   ├── assets/                # 资源文件
│   ├── x64/                   # x64 特定版本（如果构建了多架构）
│   └── x86/                   # x86 特定版本（如果构建了多架构）
└── macos/
    ├── seeu_desktop           # 主二进制文件
    ├── seeu-desktop.sh        # 启动脚本
    ├── SeeU Desktop.app/      # macOS 应用程序包
    ├── assets/                # 资源文件
    ├── x64/                   # Intel 特定版本（如果构建了多架构）
    └── arm64/                 # Apple Silicon 特定版本（如果构建了多架构）
```

## 环境要求

### 必需工具
- **Rust** (rustup, cargo)
- **Git**

### 推荐工具
- **vcpkg** (通过 `./scripts/setup-vcpkg.sh` 安装)
- **Docker** (用于跨平台构建)

### 平台特定工具
- **Linux**: build-essential, pkg-config
- **Windows**: Visual Studio Build Tools
- **macOS**: Xcode Command Line Tools, lipo

## 故障排除

### 常见问题

#### 1. vcpkg 未找到
```bash
# 解决方案：安装 vcpkg
./scripts/setup-vcpkg.sh
source ~/.bashrc  # 或重启终端
```

#### 2. Rust 目标未安装
```bash
# 解决方案：安装缺失的目标
rustup target add x86_64-unknown-linux-musl
rustup target add x86_64-pc-windows-msvc
```

#### 3. 构建失败
```bash
# 解决方案：清理并重试
cargo clean
./scripts/build-<platform>.sh
```

#### 4. 权限问题
```bash
# 解决方案：确保脚本可执行
chmod +x scripts/*.sh
```

### 调试工具

#### 测试构建脚本
```bash
# 验证所有脚本语法和依赖
./scripts/test-build-scripts.sh
```

#### 测试 vcpkg 配置
```bash
# 验证 vcpkg 安装和配置
./scripts/test-vcpkg.sh
```

## 最佳实践

### 1. 首次使用
```bash
# 1. 测试构建系统
./scripts/test-build-scripts.sh

# 2. 设置 vcpkg（推荐）
./scripts/setup-vcpkg.sh

# 3. 验证 vcpkg 配置
./scripts/test-vcpkg.sh

# 4. 构建当前平台
./scripts/build-<current-platform>.sh
```

### 2. 日常开发
```bash
# 快速构建当前平台
./scripts/build-linux.sh    # 或 build-windows.sh, build-macos.sh

# 完整测试构建
./scripts/build-all.sh
```

### 3. 发布构建
```bash
# 构建所有平台和架构
./scripts/build-all.sh --include-32bit --include-arm64

# 或使用 vcpkg 构建所有目标
./scripts/build-vcpkg.sh --target all
```

## 性能优化

### 1. 并行构建
```bash
# 设置 Cargo 并行作业数
export CARGO_BUILD_JOBS=4
```

### 2. 缓存优化
```bash
# 使用 sccache 加速编译
export RUSTC_WRAPPER=sccache
```

### 3. 目标缓存
```bash
# 避免重复安装 Rust 目标
rustup target list --installed
```

## 扩展和自定义

### 添加新平台支持
1. 创建新的构建脚本 `scripts/build-<platform>.sh`
2. 在 `scripts/build-all.sh` 中添加平台支持
3. 更新 `.cargo/config.toml` 添加目标配置

### 自定义构建选项
1. 修改相应的构建脚本
2. 添加新的命令行参数
3. 更新帮助文档

### 集成 CI/CD
```yaml
# GitHub Actions 示例
- name: Setup vcpkg
  run: ./scripts/setup-vcpkg.sh

- name: Build all platforms
  run: ./scripts/build-all.sh
```

这个新的构建系统提供了强大的跨平台构建能力，同时保持了简单易用的接口。通过 vcpkg 集成，我们实现了一致的依赖管理和静态链接支持。
