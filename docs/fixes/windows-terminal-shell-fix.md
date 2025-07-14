# Windows Terminal Shell Fix

## 问题描述

在Windows系统下，iTerminal创建会话时出现"系统找不到指定的文件"错误。错误信息显示：

```
Failed to initialize terminal backend for new session 'Terminal 1': 系统找不到指定的文件。 (os error 2)
```

## 问题原因

问题的根本原因是在`crates/egui_term/src/backend/settings.rs`文件中，`DEFAULT_SHELL`常量被硬编码为`"/bin/bash"`：

```rust
const DEFAULT_SHELL: &str = "/bin/bash";
```

这个路径在Windows系统上不存在，导致终端后端初始化失败。

## 解决方案

### 1. 修改默认Shell选择逻辑

将硬编码的常量替换为动态函数，根据操作系统选择合适的默认shell：

```rust
/// Get the default shell for the current platform
fn get_default_shell() -> String {
    #[cfg(windows)]
    {
        // On Windows, try PowerShell first, then fall back to cmd.exe
        if std::process::Command::new("powershell").arg("-Command").arg("echo test").output().is_ok() {
            "powershell".to_string()
        } else {
            "cmd.exe".to_string()
        }
    }
    #[cfg(unix)]
    {
        // On Unix-like systems, use SHELL environment variable or fall back to /bin/bash
        std::env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string())
    }
}
```

### 2. 更新BackendSettings::default()

修改`BackendSettings::default()`方法使用新的动态函数：

```rust
impl Default for BackendSettings {
    fn default() -> Self {
        Self {
            shell: get_default_shell(),  // 使用动态函数而不是硬编码常量
            args: vec![],
            working_directory: None,
            ssh_config: None,
            env_vars: HashMap::new(),
        }
    }
}
```

### 3. 同步更新iTerminal配置

为了保持一致性，也更新了`crates/iterminal/src/config.rs`中的默认shell选择逻辑：

```rust
/// Get default shell command for the current platform
fn default_shell() -> String {
    #[cfg(windows)]
    {
        // On Windows, try PowerShell first, then fall back to cmd.exe
        if std::process::Command::new("powershell").arg("-Command").arg("echo test").output().is_ok() {
            "powershell".to_string()
        } else {
            "cmd.exe".to_string()
        }
    }
    #[cfg(unix)]
    {
        std::env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string())
    }
}
```

## 测试验证

添加了单元测试来验证修复的正确性：

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_shell_selection() {
        let shell = get_default_shell();
        
        #[cfg(windows)]
        {
            // On Windows, should be either powershell or cmd.exe
            assert!(shell == "powershell" || shell == "cmd.exe", 
                   "Windows shell should be powershell or cmd.exe, got: {}", shell);
        }
        
        #[cfg(unix)]
        {
            // On Unix, should be either from SHELL env var or /bin/bash
            assert!(shell.contains("sh") || shell.contains("bash") || shell.contains("zsh"), 
                   "Unix shell should contain sh, bash, or zsh, got: {}", shell);
        }
    }

    #[test]
    fn test_backend_settings_default() {
        let settings = BackendSettings::default();
        
        // Should be a valid shell for the current platform
        #[cfg(windows)]
        {
            assert!(settings.shell == "powershell" || settings.shell == "cmd.exe", 
                   "Windows default shell should be powershell or cmd.exe, got: {}", settings.shell);
        }
        
        #[cfg(unix)]
        {
            // On Unix, should be a valid shell path
            assert!(settings.shell.contains("sh") || settings.shell.contains("bash") || settings.shell.contains("zsh"), 
                   "Unix default shell should contain sh, bash, or zsh, got: {}", settings.shell);
            
            // Should be an absolute path on Unix systems
            assert!(settings.shell.starts_with('/'), 
                   "Unix shell should be an absolute path, got: {}", settings.shell);
        }
    }
}
```

## 修复效果

- **Windows系统**：现在会优先尝试使用PowerShell，如果不可用则回退到cmd.exe
- **Unix系统**：继续使用SHELL环境变量，如果未设置则回退到/bin/bash
- **跨平台兼容性**：确保在所有支持的操作系统上都能正确选择可用的shell
- **向后兼容性**：Unix系统的行为保持不变，只修复了Windows系统的问题

## 相关文件

- `crates/egui_term/src/backend/settings.rs` - 主要修复文件
- `crates/iterminal/src/config.rs` - 同步更新的配置文件
- `docs/fixes/windows-terminal-shell-fix.md` - 本文档

## 测试命令

```bash
# 运行相关测试
cargo test -p egui_term
cargo test -p iterminal

# 编译验证
cargo build --release
```
