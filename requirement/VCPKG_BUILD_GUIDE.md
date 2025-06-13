# vcpkg 跨平台构建指南

本文档介绍如何使用 vcpkg 包管理器为 SeeU Desktop 进行跨平台构建，支持静态链接。

## 概述

vcpkg 是 Microsoft 开发的 C++ 包管理器，支持跨平台构建和静态链接。我们已经将项目配置为使用 vcpkg 来管理系统依赖，替代传统的 pkg-config 方式。

## 支持的平台

- **Linux**: x86_64, ARM64 (aarch64)
- **Windows**: x86_64, x86 (32位，可选)
- **静态链接**: 所有平台都支持静态链接

## 快速开始

### 1. 安装 vcpkg

运行自动安装脚本：

```bash
./scripts/setup-vcpkg.sh
```

这个脚本会：
- 自动检测操作系统
- 安装必要的依赖
- 下载并编译 vcpkg
- 安装项目所需的包
- 配置环境变量

### 2. 验证安装

```bash
./scripts/test-vcpkg.sh
```

### 3. 构建项目

```bash
# 构建 Linux x64 版本
./scripts/build-vcpkg.sh --target linux-x64

# 构建 Windows x64 版本
./scripts/build-vcpkg.sh --target windows-x64

# 构建所有支持的平台
./scripts/build-vcpkg.sh --target all
```

## 详细配置

### vcpkg.json 配置

项目的 `vcpkg.json` 文件定义了所需的依赖包：

```json
{
  "name": "seeu-desktop",
  "dependencies": [
    "openssl",
    "sqlite3",
    "zlib",
    "libpng",
    "libjpeg-turbo",
    "giflib"
  ]
}
```

### Cargo 配置

`.cargo/config.toml` 文件包含了各个目标平台的配置：

- 静态链接标志
- vcpkg triplet 配置
- 环境变量设置

### 构建脚本 (build.rs)

`build.rs` 文件负责：
- 检测目标平台
- 设置 vcpkg triplet
- 配置链接参数
- 处理静态链接

## 构建选项

### 目标平台

- `linux-x64`: Linux x86_64
- `linux-arm64`: Linux ARM64
- `windows-x64`: Windows x86_64
- `windows-x86`: Windows x86 (32位)
- `all`: 所有支持的平台

### 构建模式

- `release`: 发布版本（默认）
- `debug`: 调试版本

### 静态链接

- `true`: 启用静态链接（默认）
- `false`: 使用动态链接

### 示例命令

```bash
# 构建 Linux x64 发布版本，启用静态链接
./scripts/build-vcpkg.sh --target linux-x64 --mode release --static true

# 构建 Windows x64 调试版本
./scripts/build-vcpkg.sh --target windows-x64 --mode debug

# 构建所有平台
./scripts/build-vcpkg.sh --target all
```

## 环境变量

### VCPKG_ROOT

指向 vcpkg 安装目录的路径。

```bash
export VCPKG_ROOT="/opt/vcpkg"
```

### VCPKG_DEFAULT_TRIPLET

指定默认的 vcpkg triplet。

```bash
export VCPKG_DEFAULT_TRIPLET="x64-linux"
```

## 故障排除

### vcpkg 未找到

如果遇到 "vcpkg not found" 错误：

1. 确保 `VCPKG_ROOT` 环境变量已设置
2. 运行 `./scripts/setup-vcpkg.sh` 重新安装
3. 重启终端或运行 `source ~/.bashrc`

### 包安装失败

如果包安装失败：

1. 检查网络连接
2. 更新 vcpkg：`cd $VCPKG_ROOT && git pull && ./bootstrap-vcpkg.sh`
3. 清理并重新安装：`./vcpkg remove --outdated && ./vcpkg install`

### 构建失败

如果构建失败：

1. 检查 Rust 目标是否已安装：`rustup target list --installed`
2. 安装缺失的目标：`rustup target add <target>`
3. 清理构建缓存：`cargo clean`

## 与传统构建的比较

| 特性 | pkg-config | vcpkg |
|------|------------|-------|
| 跨平台支持 | 有限 | 优秀 |
| 静态链接 | 复杂 | 简单 |
| 依赖管理 | 手动 | 自动 |
| Windows 支持 | 困难 | 原生 |
| 构建一致性 | 依赖系统 | 独立 |

## 高级用法

### 自定义 triplet

可以创建自定义的 vcpkg triplet 来满足特殊需求：

```bash
# 创建自定义 triplet
cp $VCPKG_ROOT/triplets/x64-linux.cmake $VCPKG_ROOT/triplets/community/x64-linux-custom.cmake

# 编辑 triplet 文件
# 设置 VCPKG_DEFAULT_TRIPLET="x64-linux-custom"
```

### 离线构建

对于离线环境，可以预先下载所有依赖：

```bash
# 导出已安装的包
./vcpkg export --zip

# 在离线环境中导入
./vcpkg import <exported-zip>
```

## 贡献

如果您发现构建问题或有改进建议，请：

1. 检查现有的 issue
2. 创建新的 issue 并提供详细信息
3. 提交 Pull Request

## 参考资源

- [vcpkg 官方文档](https://vcpkg.io/)
- [Rust 交叉编译指南](https://rust-lang.github.io/rustup/cross-compilation.html)
- [静态链接最佳实践](https://doc.rust-lang.org/cargo/reference/config.html)
