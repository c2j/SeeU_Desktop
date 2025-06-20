# MCP服务器连接测试功能实现

## 功能概述

根据用户建议，我们在添加MCP服务器时增加了"连接"按钮，用于在保存前测试服务器连接并获取能力信息。只有连接测试成功的服务器才能被保存，确保保存的服务器都是可用的，并且能力信息会一并保存到数据库中。

## 实现的功能

### 1. 新增UI状态字段

在 `crates/itools/src/ui/mcp_settings.rs` 中的 `McpUiState` 结构体中添加了：

```rust
/// Connection test result for add server dialog
connection_test_result: Option<TestResult>,

/// Tested capabilities from connection test
tested_capabilities: Option<ServerCapabilities>,
```

### 2. 连接测试按钮

在添加服务器对话框中增加了"🔗 连接"按钮：

```rust
// 连接测试按钮
if ui.button("🔗 连接").clicked() {
    self.test_connection_before_add();
}

ui.separator();

// 只有连接测试成功后才能添加服务器
let can_add = self.ui_state.connection_test_result.is_some() && 
             self.ui_state.connection_test_result.as_ref().unwrap().success;

ui.add_enabled_ui(can_add, |ui| {
    if ui.button("添加服务器").clicked() {
        // 添加服务器逻辑
    }
});
```

### 3. 连接测试结果显示

在按钮区域之前显示连接测试结果：

```rust
// 显示连接测试结果
if let Some(test_result) = &self.ui_state.connection_test_result {
    ui.group(|ui| {
        ui.vertical(|ui| {
            if test_result.success {
                ui.colored_label(Color32::GREEN, "✅ 连接测试成功");
                if let Some(capabilities) = &self.ui_state.tested_capabilities {
                    ui.label(format!("🔧 工具: {}", capabilities.tools.len()));
                    ui.label(format!("📁 资源: {}", capabilities.resources.len()));
                    ui.label(format!("💬 提示: {}", capabilities.prompts.len()));
                }
            } else {
                ui.colored_label(Color32::RED, "❌ 连接测试失败");
                if let Some(error) = &test_result.error_message {
                    ui.label(RichText::new(error).color(Color32::RED).small());
                }
            }
        });
    });
    ui.separator();
}
```

### 4. 连接测试实现

实现了 `test_connection_before_add` 方法：

```rust
/// Test connection before adding server
fn test_connection_before_add(&mut self) {
    log::info!("🔗 开始测试连接 - 准备添加服务器");
    
    // 获取当前配置
    let config = if self.ui_state.add_server_json_mode {
        match self.parse_json_config() {
            Ok(config) => config,
            Err(e) => {
                self.ui_state.error_message = Some(format!("JSON 解析失败: {}", e));
                return;
            }
        }
    } else {
        self.new_server_config.clone()
    };

    // 验证配置
    if let Err(e) = self.server_manager.validate_server_config(&config) {
        self.ui_state.error_message = Some(format!("配置验证失败: {}", e));
        return;
    }

    // 清理之前的测试结果
    self.ui_state.connection_test_result = None;
    self.ui_state.tested_capabilities = None;
    self.ui_state.error_message = None;
    self.ui_state.status_message = Some("正在测试连接...".to_string());

    // 创建临时服务器ID用于测试
    let temp_server_id = uuid::Uuid::new_v4();
    
    // 异步执行连接测试
    // ... 测试逻辑
}
```

### 5. 能力信息保存

在连接测试成功后，将能力信息添加到服务器配置中：

```rust
// 如果连接测试成功，将能力信息添加到配置中
if let Some(test_result) = &self.ui_state.connection_test_result {
    if test_result.success {
        if let Some(capabilities) = &self.ui_state.tested_capabilities {
            // 将能力信息序列化并保存到配置中
            match serde_json::to_value(capabilities) {
                Ok(capabilities_json) => {
                    config.capabilities = Some(capabilities_json);
                    log::info!("✅ 将测试获取的能力信息添加到服务器配置中 - 工具:{}, 资源:{}, 提示:{}",
                        capabilities.tools.len(), capabilities.resources.len(), capabilities.prompts.len());
                }
                Err(e) => {
                    log::error!("❌ 序列化能力信息失败: {}", e);
                }
            }
        }
    }
}
```

### 6. 状态清理

在取消或成功添加服务器后，清理连接测试状态：

```rust
// 清理连接测试状态
self.ui_state.connection_test_result = None;
self.ui_state.tested_capabilities = None;
```

## 工作流程

1. **用户填写服务器配置**：在添加服务器对话框中填写服务器信息
2. **点击连接按钮**：用户点击"🔗 连接"按钮测试服务器连接
3. **配置验证**：系统验证服务器配置的有效性
4. **连接测试**：系统尝试连接到MCP服务器并获取能力信息
5. **结果显示**：显示连接测试结果和获取的能力信息
6. **条件保存**：只有连接测试成功后，"添加服务器"按钮才会启用
7. **能力保存**：保存服务器时，将测试获取的能力信息一并保存到数据库

## 优势

### 1. 确保服务器可用性
- 只有能够成功连接的服务器才能被保存
- 避免保存无效或不可用的服务器配置
- 提高MCP服务器管理的可靠性

### 2. 预先获取能力信息
- 在保存时就获取并保存服务器的工具、资源和提示能力
- 避免后续使用时的能力信息缺失问题
- 确保AI助手能够立即使用新添加的服务器

### 3. 用户体验优化
- 实时反馈连接测试结果
- 清晰显示服务器能力信息
- 防止用户保存无效配置

### 4. 数据完整性
- 保存的服务器配置包含完整的能力信息
- 减少数据不一致的问题
- 提高系统的整体稳定性

## 日志示例

连接测试成功时的日志输出：

```
🔗 开始测试连接 - 准备添加服务器
✅ 连接测试成功 - 工具:1, 资源:0, 提示:0
✅ 将测试获取的能力信息添加到服务器配置中 - 工具:1, 资源:0, 提示:0
✅ 已添加新服务器: 'simple-tool' (uuid-here)
```

连接测试失败时的日志输出：

```
🔗 开始测试连接 - 准备添加服务器
❌ 连接测试失败: 无法连接到服务器
```

## 总结

这个功能实现了用户建议的"添加服务器时增加连接按钮，只有正常获取了capability保存才有意义"的需求，提供了：

- **连接验证**：确保服务器可用性
- **能力获取**：预先获取并保存能力信息
- **条件保存**：只有测试成功才能保存
- **用户反馈**：清晰的测试结果显示

这个功能将显著提升MCP服务器管理的质量和可靠性，确保保存的每个服务器都是可用的，并且具有完整的能力信息。
