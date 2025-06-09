# iTerminal - SeeU Desktop 终端模块

## 概述

iTerminal 是 SeeU Desktop 应用程序的终端模块，提供了一个功能完整的终端仿真器，支持多标签页、命令历史、可配置设置等功能。

## 功能特性

### 🚀 核心功能
- **多标签页支持** - 同时运行多个终端会话
- **命令执行** - 支持系统命令和内置命令
- **实时输出** - 异步处理命令输出，不阻塞UI
- **命令历史** - 保存和搜索命令历史记录
- **滚动缓冲** - 可配置的输出缓冲区大小

### 🎨 用户界面
- **现代化设计** - 基于 egui 的现代化终端界面
- **可配置外观** - 字体、颜色、大小等可自定义
- **响应式布局** - 自适应窗口大小
- **键盘快捷键** - 支持常用的终端快捷键

### ⚙️ 配置选项
- **字体设置** - 字体族、大小、缩放比例
- **颜色主题** - 背景色、文本色、光标色等
- **行为设置** - Shell 命令、工作目录、响铃等
- **滚动设置** - 缓冲区大小、滚动行为

### 🔧 安全内置命令

#### 📁 文件和目录操作
- `ls [选项] [目录]` - 列出目录内容 (-l 详细, -a 显示隐藏文件, -h 人性化大小)
- `dir [选项] [目录]` - 列出目录内容 (Windows风格)
- `cd [目录]` - 切换工作目录
- `pwd` - 显示当前工作目录
- `cat <文件>` - 显示文件内容
- `head [-n 行数] <文件>` - 显示文件开头几行 (默认10行)
- `tail [-n 行数] <文件>` - 显示文件结尾几行 (默认10行)
- `wc [-l|-w|-c] <文件>` - 统计文件行数/单词数/字符数
- `find <目录> -name <模式>` - 查找文件
- `tree [目录]` - 显示目录树结构

#### 🔍 搜索和过滤
- `grep <模式> <文件>` - 在文件中搜索文本模式
- `which <命令>` - 查找命令位置

#### 💻 系统信息
- `whoami` - 显示当前用户名
- `hostname` - 显示主机名
- `date [-u|-I]` - 显示日期时间 (-u UTC时间, -I ISO格式)
- `uptime` - 显示系统运行时间
- `ps [选项]` - 显示进程信息
- `env [变量名]` - 显示环境变量
- `du [-h] [目录]` - 显示磁盘使用情况 (-h 人性化显示)
- `df [-h]` - 显示文件系统使用情况 (-h 人性化显示)

#### 🛠️ 实用工具
- `echo <文本>` - 输出文本
- `clear` - 清空终端输出
- `history` - 查看命令历史 (使用📜按钮)
- `help` - 显示帮助信息
- `exit` - 退出当前终端会话

## 技术架构

### 模块结构
```
crates/iterminal/
├── src/
│   ├── lib.rs          # 模块入口
│   ├── state.rs        # 状态管理
│   ├── ui.rs           # 用户界面
│   ├── terminal.rs     # 终端管理器
│   ├── session.rs      # 会话管理
│   ├── command.rs      # 命令执行
│   ├── history.rs      # 历史记录
│   └── config.rs       # 配置管理
└── Cargo.toml          # 依赖配置
```

### 核心组件

#### TerminalManager
- 管理多个终端会话
- 处理命令执行和输出
- 维护活动会话状态

#### TerminalSession
- 单个终端会话的状态
- 输出缓冲区管理
- 输入处理和光标控制

#### CommandExecutor
- 异步命令执行
- 实时输出流处理
- 内置命令支持

#### CommandHistory
- 命令历史记录
- 历史搜索和导航
- 持久化存储

## 使用方法

### 基本操作
1. **创建新标签页** - 点击 "+" 按钮
2. **切换标签页** - 点击标签页标题
3. **关闭标签页** - 点击 "×" 按钮（至少保留一个）
4. **输入命令** - 在输入框中输入命令并按 Enter
5. **滚动输出** - 使用鼠标滚轮或 Page Up/Down 键

### 快捷键
- `Enter` - 执行命令
- `↑/↓` - 浏览命令历史
- `Ctrl+Home` - 滚动到顶部
- `Ctrl+End` - 滚动到底部
- `Home` - 光标移到行首
- `End` - 光标移到行尾

### 设置配置
1. 点击 "⚙" 按钮打开设置
2. 调整字体、颜色、行为等选项
3. 点击 "保存" 应用更改

### 查看历史
1. 点击 "📜" 按钮打开历史记录
2. 使用搜索框过滤命令
3. 点击 "📋" 复制命令
4. 点击 "▶" 执行命令

## 配置文件

配置文件保存在：
- **macOS**: `~/Library/Application Support/seeu_desktop/terminal_config.json`
- **Windows**: `%APPDATA%\seeu_desktop\terminal_config.json`
- **Linux**: `~/.config/seeu_desktop/terminal_config.json`

### 配置示例
```json
{
  "font_family": "Source Code Pro",
  "font_size": 14.0,
  "background_color": [0.1, 0.1, 0.1, 1.0],
  "text_color": [0.9, 0.9, 0.9, 1.0],
  "cursor_color": [0.0, 1.0, 0.0, 1.0],
  "scrollback_lines": 10000,
  "shell_command": "/bin/bash",
  "enable_bell": false
}
```

## 开发说明

### 依赖项
- `egui` - GUI 框架
- `tokio` - 异步运行时
- `portable-pty` - 跨平台 PTY 支持
- `vte` - ANSI 转义序列解析
- `crossterm` - 跨平台终端操作
- `uuid` - 唯一标识符
- `chrono` - 时间处理
- `serde` - 序列化支持

### 集成到主应用
```rust
// 在 Cargo.toml 中添加依赖
iterminal = { path = "crates/iterminal" }

// 在应用状态中添加终端状态
pub struct App {
    // ...
    pub iterminal_state: ITerminalState,
}

// 在渲染函数中调用终端渲染
iterminal::render_iterminal(ui, &mut app.iterminal_state);

// 在更新函数中调用终端更新
iterminal::update_iterminal(&mut app.iterminal_state);
```

## 故障排除

### 常见问题

1. **字符重复输入**
   - 确保没有同时处理键盘和文本事件
   - 使用 egui 的 TextEdit 组件处理输入

2. **命令执行失败**
   - 检查 Shell 命令配置
   - 确认工作目录权限
   - 查看错误输出

3. **输出显示异常**
   - 检查 ANSI 转义序列解析
   - 调整字体和颜色设置
   - 确认缓冲区大小

### 调试模式
启用日志输出：
```bash
RUST_LOG=debug cargo run
```

## 贡献指南

1. Fork 项目
2. 创建功能分支
3. 提交更改
4. 创建 Pull Request

## 许可证

本项目采用 MIT 许可证。详见 LICENSE 文件。
