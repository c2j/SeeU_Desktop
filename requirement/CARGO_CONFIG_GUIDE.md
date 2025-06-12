# Cargo 配置指南

本文档详细说明了 SeeU Desktop 项目的 `.cargo/config.toml` 配置文件，以及如何为跨平台构建和 vcpkg 集成进行优化。

## 配置文件结构

### 1. 构建配置

```toml
[build]
target-applies-to-host = ["proc-macro"]
```

这个设置确保在交叉编译时，过程宏（proc-macro）依赖项使用主机架构而不是目标架构。

### 2. Windows 目标配置

#### Windows MSVC x64
```toml
[target.x86_64-pc-windows-msvc]
rustflags = ["-C", "target-feature=+crt-static"]

[target.x86_64-pc-windows-msvc.env]
VCPKG_DEFAULT_TRIPLET = "x64-windows-static"
```

#### Windows MSVC x86 (32位)
```toml
[target.i686-pc-windows-msvc]
rustflags = ["-C", "target-feature=+crt-static"]

[target.i686-pc-windows-msvc.env]
VCPKG_DEFAULT_TRIPLET = "x86-windows-static"
```

#### Windows GNU (备用)
```toml
[target.x86_64-pc-windows-gnu]
rustflags = ["-C", "target-feature=+crt-static"]
# linker = "x86_64-w64-mingw32-gcc"  # 交叉编译时取消注释
```

### 3. Linux 目标配置

#### Linux x64 GNU
```toml
[target.x86_64-unknown-linux-gnu]
rustflags = ["-C", "target-feature=+crt-static"]

[target.x86_64-unknown-linux-gnu.env]
VCPKG_DEFAULT_TRIPLET = "x64-linux"
```

#### Linux x64 musl (静态链接)
```toml
[target.x86_64-unknown-linux-musl]
rustflags = ["-C", "target-feature=+crt-static"]

[target.x86_64-unknown-linux-musl.env]
VCPKG_DEFAULT_TRIPLET = "x64-linux"
```

#### Linux ARM64
```toml
[target.aarch64-unknown-linux-gnu]
rustflags = ["-C", "target-feature=+crt-static"]
# linker = "aarch64-linux-gnu-gcc"  # 交叉编译时取消注释

[target.aarch64-unknown-linux-gnu.env]
VCPKG_DEFAULT_TRIPLET = "arm64-linux"
```

### 4. macOS 目标配置

#### macOS Intel x64
```toml
[target.x86_64-apple-darwin]
rustflags = []

[target.x86_64-apple-darwin.env]
VCPKG_DEFAULT_TRIPLET = "x64-osx"
```

#### macOS Apple Silicon ARM64
```toml
[target.aarch64-apple-darwin]
rustflags = []

[target.aarch64-apple-darwin.env]
VCPKG_DEFAULT_TRIPLET = "arm64-osx"
```

### 5. 全局环境变量

```toml
[env]
VCPKG_FEATURE_FLAGS = "manifests,versions"
```

启用 vcpkg 的清单模式和版本控制功能。

### 6. 构建配置文件

#### Release 配置
```toml
[profile.release]
opt-level = 3          # 最高优化级别
debug = false          # 不包含调试信息
strip = true           # 剥离符号表
lto = true             # 启用链接时优化
codegen-units = 1      # 单个代码生成单元（更好的优化）
panic = "abort"        # panic 时直接终止
```

#### Debug 配置
```toml
[profile.dev]
opt-level = 0          # 无优化
debug = true           # 包含完整调试信息
strip = false          # 保留符号表
lto = false            # 禁用链接时优化
```

#### 自定义配置文件
```toml
[profile.dev-fast]
inherits = "dev"
opt-level = 1          # 轻度优化
debug = "line-tables-only"  # 仅行号调试信息

[profile.release-with-debug]
inherits = "release"
debug = true           # 保留调试信息
strip = false          # 保留符号表
```

## vcpkg 集成说明

### Triplet 映射

| Rust Target | vcpkg Triplet | 说明 |
|-------------|---------------|------|
| `x86_64-pc-windows-msvc` | `x64-windows-static` | Windows 64位静态链接 |
| `i686-pc-windows-msvc` | `x86-windows-static` | Windows 32位静态链接 |
| `x86_64-unknown-linux-gnu` | `x64-linux` | Linux 64位 |
| `aarch64-unknown-linux-gnu` | `arm64-linux` | Linux ARM64 |
| `x86_64-apple-darwin` | `x64-osx` | macOS Intel |
| `aarch64-apple-darwin` | `arm64-osx` | macOS Apple Silicon |

### 静态链接配置

所有目标都配置了 `target-feature=+crt-static`，这确保：
- Windows: 静态链接 MSVC 运行时
- Linux: 静态链接 glibc/musl
- 减少运行时依赖

## 交叉编译配置

### 启用交叉编译链接器

如果需要在非目标平台上进行交叉编译，取消注释相应的链接器配置：

```toml
# Linux ARM64 交叉编译
[target.aarch64-unknown-linux-gnu]
linker = "aarch64-linux-gnu-gcc"

# Windows 交叉编译
[target.x86_64-pc-windows-gnu]
linker = "x86_64-w64-mingw32-gcc"
```

### 安装交叉编译工具链

```bash
# Ubuntu/Debian
sudo apt-get install gcc-aarch64-linux-gnu gcc-mingw-w64

# macOS
brew install FiloSottile/musl-cross/musl-cross
brew install mingw-w64
```

## 网络和注册表配置

### 网络设置
```toml
[net]
retry = 3                    # 重试次数
git-fetch-with-cli = true    # 使用 git CLI

[http]
timeout = 30                 # 超时时间（秒）
multiplexing = true          # HTTP/2 多路复用
check-revoke = true          # 检查证书撤销
```

### 镜像注册表（可选）

如果网络访问 crates.io 较慢，可以启用镜像：

```toml
[source.crates-io]
replace-with = "ustc"

[source.ustc]
registry = "https://mirrors.ustc.edu.cn/crates.io-index"
```

## 使用示例

### 1. 构建特定目标

```bash
# 构建 Windows x64
cargo build --target x86_64-pc-windows-msvc --release

# 构建 Linux ARM64
cargo build --target aarch64-unknown-linux-gnu --release

# 构建 macOS 通用二进制
cargo build --target x86_64-apple-darwin --release
cargo build --target aarch64-apple-darwin --release
```

### 2. 使用自定义配置文件

```bash
# 快速开发构建
cargo build --profile dev-fast

# 带调试信息的发布构建
cargo build --profile release-with-debug
```

### 3. 设置环境变量

```bash
# 临时设置 vcpkg 路径
export VCPKG_ROOT="/path/to/vcpkg"
cargo build --release

# 设置特定 triplet
export VCPKG_DEFAULT_TRIPLET="x64-windows-static"
cargo build --target x86_64-pc-windows-msvc --release
```

## 故障排除

### 1. 链接器错误

如果遇到链接器错误，检查：
- 是否安装了正确的交叉编译工具链
- 链接器路径是否正确
- 是否设置了正确的环境变量

### 2. vcpkg 集成问题

```bash
# 检查 vcpkg 环境
echo $VCPKG_ROOT
echo $VCPKG_DEFAULT_TRIPLET

# 验证 vcpkg 包
$VCPKG_ROOT/vcpkg list
```

### 3. 静态链接问题

如果静态链接失败：
- 确保目标平台支持静态链接
- 检查依赖库是否提供静态版本
- 考虑使用 musl 目标（Linux）

### 4. 性能问题

如果编译速度慢：
- 增加并行作业数：`export CARGO_BUILD_JOBS=8`
- 使用 sccache：`export RUSTC_WRAPPER=sccache`
- 考虑使用 `dev-fast` 配置文件

## 最佳实践

### 1. 开发环境

```bash
# 设置开发环境变量
export CARGO_BUILD_JOBS=$(nproc)
export RUSTC_WRAPPER=sccache
```

### 2. CI/CD 环境

```yaml
# GitHub Actions 示例
env:
  VCPKG_ROOT: ${{ github.workspace }}/vcpkg
  CARGO_BUILD_JOBS: 4
```

### 3. 本地构建

```bash
# 创建本地配置覆盖
cat > .cargo/config.local.toml << EOF
[env]
VCPKG_ROOT = "/usr/local/share/vcpkg"
EOF
```

### 4. 清理构建

```bash
# 清理所有目标
cargo clean

# 清理特定目标
cargo clean --target x86_64-pc-windows-msvc
```

这个配置文件为 SeeU Desktop 提供了完整的跨平台构建支持，与 vcpkg 包管理器无缝集成，并优化了构建性能和二进制大小。
