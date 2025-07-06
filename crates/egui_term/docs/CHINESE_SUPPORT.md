# egui_term 中文字符支持

## 概述

egui_term 基于 alacritty_terminal 构建，具备完整的中文字符支持能力。本文档说明了中文字符的输入、显示和处理机制。

## 支持的功能

### ✅ UTF-8 编码支持

- **终端编码**：在 Unix 系统上自动设置 `IUTF8` 标志
- **字符传输**：通过 `text.as_bytes().to_vec()` 正确处理 UTF-8 编码
- **兼容性**：与标准终端应用程序完全兼容

### ✅ 全角字符处理

- **宽度计算**：使用 `c.width()` 正确计算中文字符宽度
- **双倍宽度**：中文字符占用两个终端列
- **对齐处理**：正确处理中英文混合文本的对齐

### ✅ 字符渲染

- **字体回退**：egui 自动使用系统可用的中文字体
- **Unicode 支持**：完整支持 Unicode 字符集
- **显示质量**：清晰的中文字符渲染

## 技术实现

### 字符输入流程

1. **事件捕获**：egui 捕获 `Text` 事件
2. **编码转换**：`process_text_event` 将文本转换为 UTF-8 字节
3. **终端写入**：通过 `BackendCommand::Write` 发送到终端
4. **字符处理**：alacritty_terminal 的 `input` 方法处理字符

### 字符显示流程

1. **字符解析**：终端解析 UTF-8 字节流
2. **宽度计算**：计算字符显示宽度（1或2列）
3. **网格布局**：在终端网格中正确放置字符
4. **渲染输出**：egui 渲染字符到屏幕

## 使用示例

### 基本中文输入

```rust
use egui_term::{TerminalView, TerminalBackend, BackendSettings};

// 创建终端后端
let backend = TerminalBackend::new(
    0,
    ctx.clone(),
    sender,
    BackendSettings::default(),
)?;

// 在终端中输入中文
// 用户可以直接输入：你好世界
```

### 测试中文支持

在运行的终端中执行：

```bash
# 测试中文输出
echo "你好世界！这是中文测试。"

# 测试中文文件操作
touch 中文文件名.txt
ls -la 中文*

# 测试中文编辑
nano 测试.txt
```

## 字体配置

### 默认配置（推荐）

```rust
// 使用默认字体设置，依赖系统字体回退
let terminal = TerminalView::new(ui, &mut backend)
    .set_font(TerminalFont::new(FontSettings {
        font_type: egui::FontId::monospace(14.0),
    }));
```

### 自定义字体（可选）

```rust
fn setup_chinese_font(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    
    // 添加中文字体到等宽字体族
    fonts.families
        .entry(egui::FontFamily::Monospace)
        .or_default()
        .push("PingFang SC".to_owned());
    
    ctx.set_fonts(fonts);
}
```

## 常见问题

### Q: 中文字符显示为方块？

**A**: 检查系统是否安装了中文字体。在 macOS 上通常自带中文字体支持。

### Q: 中文字符对齐不正确？

**A**: 确保使用等宽字体，并且终端正确计算了字符宽度。

### Q: 输入法无法输入中文？

**A**: 确保应用程序窗口获得了焦点，并且系统输入法正常工作。

## 兼容性

- **操作系统**：macOS, Linux, Windows
- **字符集**：完整 Unicode 支持
- **输入法**：支持系统输入法
- **终端应用**：与标准终端应用程序兼容

## 性能考虑

- **渲染性能**：中文字符渲染性能与英文字符相当
- **内存使用**：UTF-8 编码高效存储中文字符
- **响应速度**：输入响应速度不受中文字符影响

## 开发建议

1. **测试覆盖**：确保测试用例包含中文字符场景
2. **字体回退**：依赖系统字体回退而非硬编码字体
3. **编码一致**：始终使用 UTF-8 编码处理文本
4. **宽度计算**：正确处理全角字符的宽度计算

## 更新日志

- **v0.1.0**: 初始中文字符支持
- **当前版本**: 完整的 UTF-8 和全角字符支持
