# vcpkg 迁移总结

本文档总结了将 SeeU Desktop 项目从基于 pkg-config 的构建系统迁移到基于 vcpkg 的跨平台构建系统的所有更改。

## 迁移概述

### 目标
- 将基于 pkg-config 的编译功能切换到基于 vcpkg
- 支持 Linux（x86_64 和 ARM64）、Windows（64位必须支持，32位可选支持）的交叉编译
- 支持静态链接

### 实现的功能
✅ vcpkg 包管理器集成  
✅ 跨平台构建支持（Linux x64/ARM64, Windows x64/x86）  
✅ 静态链接支持  
✅ 自动化安装和配置脚本  
✅ 向后兼容的构建脚本  
✅ 完整的文档和示例  

## 文件更改清单

### 新增文件

1. **vcpkg.json** - vcpkg 清单文件
   - 定义项目依赖包
   - 配置包版本和特性

2. **build.rs** - Rust 构建脚本
   - vcpkg 集成逻辑
   - 目标平台检测
   - 静态链接配置

3. **scripts/setup-vcpkg.sh** - vcpkg 安装脚本
   - 自动检测操作系统
   - 安装 vcpkg 和依赖
   - 配置环境变量

4. **scripts/build-vcpkg.sh** - 基于 vcpkg 的构建脚本
   - 支持多目标平台构建
   - 静态/动态链接选项
   - 详细的构建日志

5. **scripts/test-vcpkg.sh** - vcpkg 配置测试脚本
   - 验证 vcpkg 安装
   - 检查依赖包
   - 测试构建配置

6. **docs/VCPKG_BUILD_GUIDE.md** - 详细构建指南
   - 完整的使用说明
   - 故障排除指南
   - 高级配置选项

9. **docs/BUILD_SCRIPTS_GUIDE.md** - 构建脚本详细指南
   - 所有构建脚本的使用说明
   - 命令行参数详解
   - 最佳实践和故障排除

10. **docs/CARGO_CONFIG_GUIDE.md** - Cargo 配置详细说明
    - .cargo/config.toml 完整解释
    - vcpkg 集成配置
    - 交叉编译设置指南

7. **examples/build-example.sh** - 构建示例脚本
   - 演示完整构建流程
   - 适合新用户快速上手

8. **scripts/test-build-scripts.sh** - 构建脚本测试工具
   - 验证所有脚本语法正确性
   - 检查依赖和环境配置
   - 提供构建系统健康检查

### 修改文件

1. **Cargo.toml**
   - 添加 vcpkg 构建依赖
   - 保持现有依赖不变

2. **.cargo/config.toml** (完全重写)
   - 完整的跨平台目标配置
   - 每个目标的 vcpkg triplet 映射
   - 静态链接和优化配置
   - 交叉编译链接器设置
   - 自定义构建配置文件
   - 网络和注册表优化

3. **scripts/build-linux.sh** (完全重写)
   - 优先使用 vcpkg 构建
   - 支持 x64 和 ARM64 架构
   - 支持静态/动态链接选择
   - 智能回退到原生构建或 Docker
   - 详细的命令行参数支持

4. **scripts/build-windows.sh** (完全重写)
   - 优先使用 vcpkg 构建
   - 支持 x64 和 x86 (32位) 架构
   - 支持静态/动态链接选择
   - 智能回退到原生构建或 Docker
   - 详细的命令行参数支持

5. **scripts/build-macos.sh** (完全重写)
   - 支持 Intel 和 Apple Silicon 架构
   - 自动创建通用二进制文件
   - 创建 macOS 应用程序包
   - 详细的命令行参数支持

6. **scripts/build-all.sh** (完全重写)
   - 统一的多平台构建脚本
   - 支持选择性平台构建
   - 详细的构建报告和摘要
   - 智能平台检测和推荐

7. **README.md**
   - 更新安装说明
   - 添加 vcpkg 构建方法
   - 更新跨平台构建部分

8. **requirement/REQ-main.md**
   - 同步 README.md 的更改
   - 添加 vcpkg 先决条件

## 支持的构建目标

### Linux
- `x86_64-unknown-linux-gnu` - 标准 Linux x64
- `x86_64-unknown-linux-musl` - 静态链接 Linux x64
- `aarch64-unknown-linux-gnu` - 标准 Linux ARM64
- `aarch64-unknown-linux-musl` - 静态链接 Linux ARM64

### Windows
- `x86_64-pc-windows-msvc` - Windows x64
- `i686-pc-windows-msvc` - Windows x86 (32位，可选)

## 使用方法

### 快速开始
```bash
# 1. 设置 vcpkg
./scripts/setup-vcpkg.sh

# 2. 测试配置
./scripts/test-vcpkg.sh

# 3. 构建项目
./scripts/build-vcpkg.sh --target linux-x64
```

### 高级用法
```bash
# 构建所有平台
./scripts/build-vcpkg.sh --target all

# 启用静态链接
./scripts/build-vcpkg.sh --target windows-x64 --static true

# 调试模式构建
./scripts/build-vcpkg.sh --target linux-x64 --mode debug
```

## 向后兼容性

- 现有的构建脚本（`build-linux.sh`, `build-windows.sh`）仍然可用
- 如果未安装 vcpkg，会自动回退到传统构建方法
- Docker 构建方法保持不变
- 所有现有的 Cargo 命令仍然有效

## 依赖包管理

### vcpkg 管理的包
- openssl - SSL/TLS 支持
- sqlite3 - 数据库支持
- zlib - 压缩支持
- libpng - PNG 图像支持
- libjpeg-turbo - JPEG 图像支持
- giflib - GIF 图像支持

### Rust crates（不变）
- 所有现有的 Rust 依赖保持不变
- 通过 Cargo.toml 管理

## 构建输出

构建完成后，二进制文件位于：
```
dist/
├── linux-x64/
│   ├── seeu_desktop
│   ├── seeu-desktop.sh
│   └── assets/
├── linux-arm64/
│   ├── seeu_desktop
│   ├── seeu-desktop.sh
│   └── assets/
├── windows-x64/
│   ├── seeu_desktop.exe
│   └── assets/
└── windows-x86/
    ├── seeu_desktop.exe
    └── assets/
```

## 环境变量

### 必需的环境变量
- `VCPKG_ROOT` - vcpkg 安装路径

### 可选的环境变量
- `VCPKG_DEFAULT_TRIPLET` - 默认目标 triplet
- `VCPKG_FEATURE_FLAGS` - vcpkg 特性标志

## 故障排除

### 常见问题
1. **vcpkg 未找到** - 运行 `./scripts/setup-vcpkg.sh`
2. **包安装失败** - 检查网络连接，更新 vcpkg
3. **构建失败** - 检查 Rust 目标，清理缓存

### 调试命令
```bash
# 检查 vcpkg 状态
./scripts/test-vcpkg.sh

# 查看已安装的包
$VCPKG_ROOT/vcpkg list

# 查看可用的 triplet
$VCPKG_ROOT/vcpkg help triplet
```

## 性能对比

| 特性 | pkg-config | vcpkg |
|------|------------|-------|
| 跨平台支持 | 有限 | 优秀 |
| 静态链接 | 复杂 | 简单 |
| 依赖管理 | 手动 | 自动 |
| Windows 支持 | 困难 | 原生 |
| 构建一致性 | 依赖系统 | 独立 |
| 设置复杂度 | 低 | 中等 |

## 下一步计划

1. **测试和验证** - 在不同平台上测试构建
2. **CI/CD 集成** - 更新持续集成配置
3. **文档完善** - 根据用户反馈改进文档
4. **性能优化** - 优化构建时间和二进制大小

## 贡献指南

如果您遇到问题或有改进建议：

1. 查看现有的 issue
2. 运行 `./scripts/test-vcpkg.sh` 收集诊断信息
3. 创建详细的 issue 报告
4. 提交 Pull Request（如果有解决方案）

---

**注意**: 这个迁移保持了完全的向后兼容性。如果您不想使用 vcpkg，现有的构建方法仍然可用。
