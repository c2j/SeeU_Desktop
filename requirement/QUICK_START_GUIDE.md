# SeeU Desktop 快速开始指南

本指南帮助您快速设置和使用新的 vcpkg 跨平台构建系统。

## 🚀 快速开始（5分钟）

### 1. 验证构建系统
```bash
# 检查所有构建脚本和依赖
./scripts/test-build-scripts.sh
```

### 2. 设置 vcpkg（推荐）
```bash
# 自动安装和配置 vcpkg
./scripts/setup-vcpkg.sh

# 重新加载环境变量
source ~/.bashrc  # 或重启终端
```

### 3. 验证 vcpkg 配置
```bash
# 检查 vcpkg 安装和包
./scripts/test-vcpkg.sh
```

### 4. 构建项目
```bash
# 构建当前平台（自动检测）
./scripts/build-linux.sh     # Linux
./scripts/build-windows.sh   # Windows  
./scripts/build-macos.sh     # macOS

# 或构建所有平台
./scripts/build-all.sh
```

## 📋 系统要求

### 必需工具
- ✅ **Rust** (rustup, cargo)
- ✅ **Git**

### 推荐工具
- 🔧 **vcpkg** (通过脚本自动安装)
- 🐳 **Docker** (用于跨平台构建)

### 平台特定工具
- **Linux**: `build-essential`, `pkg-config`
- **Windows**: Visual Studio Build Tools
- **macOS**: Xcode Command Line Tools

## 🎯 支持的构建目标

| 平台 | 架构 | 静态链接 | 状态 |
|------|------|----------|------|
| Linux | x86_64 | ✅ | 完全支持 |
| Linux | ARM64 | ✅ | 完全支持 |
| Windows | x86_64 | ✅ | 完全支持 |
| Windows | x86 (32位) | ✅ | 可选支持 |
| macOS | Intel x64 | ❌ | 完全支持 |
| macOS | Apple Silicon | ❌ | 完全支持 |

## 🛠️ 常用命令

### 基本构建
```bash
# 默认构建（当前平台，静态链接，release 模式）
./scripts/build-linux.sh

# 查看帮助
./scripts/build-linux.sh --help
```

### 高级构建选项
```bash
# 构建特定架构
./scripts/build-linux.sh --arch x64
./scripts/build-linux.sh --arch arm64
./scripts/build-linux.sh --arch all --include-arm64

# 构建模式
./scripts/build-linux.sh --mode debug
./scripts/build-linux.sh --mode release

# 链接方式
./scripts/build-linux.sh --static true   # 静态链接（默认）
./scripts/build-linux.sh --static false  # 动态链接
```

### Windows 特定选项
```bash
# 包含 32位构建
./scripts/build-windows.sh --arch all --include-32bit

# 仅 32位构建
./scripts/build-windows.sh --arch x86
```

### macOS 特定选项
```bash
# 创建通用二进制文件
./scripts/build-macos.sh --arch all --universal true

# 特定架构
./scripts/build-macos.sh --arch x64      # Intel
./scripts/build-macos.sh --arch arm64    # Apple Silicon
```

### 多平台构建
```bash
# 构建所有平台
./scripts/build-all.sh

# 包含所有架构
./scripts/build-all.sh --include-32bit --include-arm64

# 选择特定平台
./scripts/build-all.sh --platforms linux,windows

# 使用 vcpkg 构建所有目标
./scripts/build-vcpkg.sh --target all
```

## 📁 构建输出

构建完成后，二进制文件位于 `dist/` 目录：

```
dist/
├── linux/
│   ├── seeu_desktop           # 主程序
│   ├── seeu-desktop.sh        # 启动脚本
│   └── assets/                # 资源文件
├── windows/
│   ├── seeu_desktop.exe       # 主程序
│   ├── seeu-desktop.bat       # 启动脚本
│   └── assets/                # 资源文件
└── macos/
    ├── seeu_desktop           # 主程序
    ├── seeu-desktop.sh        # 启动脚本
    ├── SeeU Desktop.app/      # macOS 应用包
    └── assets/                # 资源文件
```

## 🔧 故障排除

### 问题：vcpkg 未找到
```bash
# 解决方案
./scripts/setup-vcpkg.sh
source ~/.bashrc
```

### 问题：Rust 目标未安装
```bash
# 解决方案
rustup target add x86_64-unknown-linux-musl
rustup target add x86_64-pc-windows-msvc
```

### 问题：构建失败
```bash
# 解决方案
cargo clean
./scripts/build-<platform>.sh
```

### 问题：权限错误
```bash
# 解决方案
chmod +x scripts/*.sh
```

## 📚 详细文档

- **[vcpkg 构建指南](docs/VCPKG_BUILD_GUIDE.md)** - 详细的 vcpkg 使用说明
- **[构建脚本指南](docs/BUILD_SCRIPTS_GUIDE.md)** - 所有构建脚本的详细说明
- **[Cargo 配置指南](docs/CARGO_CONFIG_GUIDE.md)** - .cargo/config.toml 配置说明
- **[迁移总结](VCPKG_MIGRATION_SUMMARY.md)** - 完整的迁移变更记录

## 🎉 成功示例

### 示例 1：Linux 开发者
```bash
# 1. 验证系统
./scripts/test-build-scripts.sh

# 2. 设置 vcpkg
./scripts/setup-vcpkg.sh

# 3. 构建 Linux 版本
./scripts/build-linux.sh

# 4. 运行程序
./dist/linux/seeu-desktop.sh
```

### 示例 2：跨平台发布
```bash
# 1. 设置 vcpkg
./scripts/setup-vcpkg.sh

# 2. 构建所有平台和架构
./scripts/build-all.sh --include-32bit --include-arm64

# 3. 检查输出
find dist -name "seeu_desktop*" -type f
```

### 示例 3：macOS 通用应用
```bash
# 1. 构建通用二进制
./scripts/build-macos.sh --arch all --universal true

# 2. 运行应用
open "dist/macos/SeeU Desktop.app"
```

## 🔄 从旧系统迁移

如果您之前使用过旧的构建系统：

1. **备份现有配置**（如果有自定义修改）
2. **运行新的测试脚本**：`./scripts/test-build-scripts.sh`
3. **设置 vcpkg**：`./scripts/setup-vcpkg.sh`
4. **使用新的构建脚本**：`./scripts/build-<platform>.sh`

新系统完全向后兼容，如果 vcpkg 不可用，会自动回退到传统构建方法。

## 🆘 获取帮助

如果遇到问题：

1. **查看帮助**：`./scripts/build-<platform>.sh --help`
2. **运行诊断**：`./scripts/test-build-scripts.sh`
3. **检查 vcpkg**：`./scripts/test-vcpkg.sh`
4. **查看详细文档**：`docs/` 目录下的指南
5. **提交 Issue**：包含诊断信息和错误日志

---

**恭喜！** 您现在拥有了一个强大的跨平台构建系统，支持 Linux（x64/ARM64）、Windows（x64/x86）和 macOS（Intel/Apple Silicon），并且可以进行静态链接构建。开始构建您的应用程序吧！ 🎉
