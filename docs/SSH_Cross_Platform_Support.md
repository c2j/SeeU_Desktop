# SSH跨平台支持文档

## 概述

SeeU Desktop的iTerminal模块现在提供了完整的跨平台SSH支持，能够在Windows、macOS和Linux系统上自动检测和使用可用的SSH客户端。

## 支持的平台

### macOS
- ✅ 内置OpenSSH客户端
- ✅ 通过Homebrew安装的SSH工具
- ✅ sshpass支持（需要通过Homebrew安装）

### Linux
- ✅ 系统包管理器安装的OpenSSH
- ✅ sshpass支持（通过包管理器安装）
- ✅ 各种发行版支持（Ubuntu、CentOS、Arch等）

### Windows
- ✅ Windows 10/11内置OpenSSH客户端
- ✅ Git for Windows附带的SSH客户端
- ✅ WSL (Windows Subsystem for Linux)
- ⚠️ sshpass在Windows上需要WSL支持

## SSH客户端检测逻辑

系统会按以下顺序检测可用的SSH客户端：

1. **标准SSH命令** (`ssh`)
2. **Windows OpenSSH** (`C:\Windows\System32\OpenSSH\ssh.exe`)
3. **Git for Windows SSH** (多个可能路径)
4. **PowerShell SSH模块**

## 认证方式支持

### 1. 密码认证
- **macOS/Linux**: 使用`sshpass`工具自动输入密码
- **Windows**: 建议使用WSL或手动输入密码
- **回退方案**: 如果sshpass不可用，回退到交互式密码输入

### 2. 私钥认证
- ✅ 所有平台完全支持
- ✅ 支持密码保护的私钥
- ✅ 自动检测密钥文件

### 3. SSH Agent认证
- ✅ 所有平台完全支持
- ✅ 自动使用系统SSH Agent

## 安装指南

### Windows用户

#### 方法1: 启用Windows OpenSSH
```powershell
# 以管理员身份运行PowerShell
Add-WindowsCapability -Online -Name OpenSSH.Client~~~~0.0.1.0
```

#### 方法2: 安装Git for Windows
1. 下载并安装 [Git for Windows](https://git-scm.com/download/win)
2. 安装时选择包含SSH客户端

#### 方法3: 使用WSL
```bash
# 安装WSL Ubuntu
wsl --install -d Ubuntu

# 在WSL中安装SSH工具
sudo apt update
sudo apt install openssh-client sshpass
```

### macOS用户

SSH通常已预装，如需sshpass：
```bash
# 使用Homebrew安装sshpass
brew install sshpass
```

### Linux用户

#### Ubuntu/Debian
```bash
sudo apt update
sudo apt install openssh-client sshpass
```

#### CentOS/RHEL
```bash
sudo yum install openssh-clients sshpass
# 或在较新版本中
sudo dnf install openssh-clients sshpass
```

#### Arch Linux
```bash
sudo pacman -S openssh sshpass
```

## 使用方法

### 1. 检查SSH支持状态
在iTerminal的远程服务器管理界面中，点击"🔧 SSH状态"按钮查看当前系统的SSH支持状态。

### 2. 配置远程服务器
1. 点击"➕ 添加服务器"
2. 填写服务器信息
3. 选择认证方式：
   - **密码认证**: 输入用户名和密码
   - **私钥认证**: 选择私钥文件路径
   - **SSH Agent**: 使用系统SSH Agent

### 3. 连接到远程服务器
1. 在服务器列表中选择目标服务器
2. 点击"连接"按钮
3. 系统会自动使用最佳的SSH客户端和认证方式

## 故障排除

### SSH客户端不可用
**症状**: 显示"SSH客户端不可用"错误

**解决方案**:
- Windows: 按照上述安装指南安装OpenSSH或Git for Windows
- macOS: SSH应该已预装，检查系统完整性
- Linux: 安装openssh-client包

### sshpass不可用
**症状**: 密码认证需要手动输入

**解决方案**:
- 安装sshpass工具（见上述安装指南）
- 或改用私钥认证方式

### 连接超时
**症状**: 连接时出现超时错误

**解决方案**:
1. 检查网络连接
2. 验证服务器地址和端口
3. 检查防火墙设置
4. 尝试使用网络连接测试功能

## 技术实现

### 核心组件
- `SshConnectionBuilder`: SSH连接构建器
- `RemoteServerManager`: 远程服务器管理
- `SshConnectionConfig`: SSH连接配置

### 关键方法
- `check_ssh_availability()`: 检查SSH客户端可用性
- `get_ssh_command()`: 获取可用的SSH命令路径
- `get_ssh_support_info()`: 获取详细的支持状态信息
- `build_ssh_command()`: 构建平台特定的SSH命令

## 安全考虑

1. **密码存储**: 使用系统密钥环安全存储密码
2. **私钥保护**: 支持密码保护的私钥文件
3. **连接验证**: 支持主机密钥验证
4. **超时控制**: 防止连接挂起

## 未来改进

- [ ] 支持更多SSH客户端（PuTTY等）
- [ ] 图形化密钥生成工具
- [ ] SSH隧道和端口转发支持
- [ ] 连接会话恢复功能
