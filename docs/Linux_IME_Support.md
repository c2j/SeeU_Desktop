# Linux IME 支持指南

## 问题描述

在Linux桌面环境下，使用IME输入法（如ibus、fcitx等）输入中文时，可能会遇到以下问题：
- 只能在文本框的开头部分输入一次中文
- 输入法候选窗口位置不正确
- 中文输入被键盘快捷键干扰

## 解决方案

### 1. 环境变量设置

在启动应用程序之前，需要设置正确的IME环境变量：

#### 对于 ibus 输入法：
```bash
export GTK_IM_MODULE=ibus
export QT_IM_MODULE=ibus
export XMODIFIERS=@im=ibus
```

#### 对于 fcitx 输入法：
```bash
export GTK_IM_MODULE=fcitx
export QT_IM_MODULE=fcitx
export XMODIFIERS=@im=fcitx
```

#### 对于 scim 输入法：
```bash
export GTK_IM_MODULE=scim
export QT_IM_MODULE=scim
export XMODIFIERS=@im=scim
```

### 2. 使用提供的启动脚本

我们提供了一个专门的启动脚本来设置正确的环境变量：

```bash
./scripts/run-linux-ime.sh
```

### 3. 代码层面的改进

#### 3.1 升级egui版本
- 从 0.31.1 升级到 0.32.0 以获得更好的IME支持

#### 3.2 键盘事件处理改进
- 在处理键盘快捷键之前检查IME组合状态
- 避免在IME输入过程中干扰文本输入

#### 3.3 文本输入组件改进
- 为floem和egui文本输入组件添加IME状态检查
- 优化事件处理顺序

### 4. 测试方法

1. 启动应用程序：
   ```bash
   ./scripts/run-linux-ime.sh
   ```

2. 在任意文本输入框中：
   - 切换到中文输入法
   - 尝试输入中文字符
   - 验证可以在文本框的任意位置输入中文
   - 验证输入法候选窗口显示正常

### 5. 故障排除

#### 5.1 检查输入法状态
```bash
# 检查当前输入法
echo $GTK_IM_MODULE
echo $QT_IM_MODULE
echo $XMODIFIERS

# 检查输入法进程
ps aux | grep ibus
ps aux | grep fcitx
```

#### 5.2 常见问题

**问题1：输入法无法激活**
- 确保输入法服务正在运行
- 检查环境变量设置是否正确

**问题2：候选窗口位置错误**
- 这是已知的egui/winit限制
- 新版本有所改善

**问题3：快捷键干扰输入**
- 代码已添加IME状态检查
- 确保使用最新版本

### 6. 技术细节

#### 6.1 IME事件处理
应用程序现在会检查以下IME事件：
- `CompositionStart` - IME开始组合
- `CompositionUpdate` - IME组合更新
- `CompositionEnd` - IME组合结束

#### 6.2 键盘事件过滤
在IME组合状态下，以下键盘事件会被忽略：
- Enter键（避免意外发送消息）
- 方向键（避免干扰候选选择）
- Tab键（避免焦点转移）

### 7. 已知限制

1. **候选窗口位置**：由于winit的限制，候选窗口可能不会精确跟随光标位置
2. **某些输入法兼容性**：部分输入法可能需要额外配置
3. **Wayland支持**：在Wayland下可能需要额外设置

### 8. 未来改进

- 等待winit完全实现IME支持
- 考虑使用其他GUI框架（如已迁移的floem）
- 添加更多输入法的特定支持

## 参考资料

- [winit IME tracking issue](https://github.com/rust-windowing/winit/issues/1497)
- [egui IME support discussion](https://github.com/emilk/egui/issues/93)
- Linux输入法配置文档
