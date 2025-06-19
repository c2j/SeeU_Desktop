# AI助手工具调用结果显示优化

## 概述

本次优化改进了AI助手界面中工具调用结果的显示方式，将工具执行结果按时间顺序放置在工具调用消息框内部的下方，并显示时间戳，提供更好的用户体验。

## 主要改进

### 1. 工具调用和结果的统一显示

**之前的实现：**
- 工具调用和工具调用结果分别在不同的消息中显示
- 用户需要在聊天记录中寻找对应的结果
- 缺乏直观的关联性

**优化后的实现：**
- 工具调用和相关的执行结果在同一个消息框中显示
- 结果按时间戳排序，显示在工具调用下方
- 提供清晰的视觉层次和关联性

### 2. 时间戳显示

新增了用户友好的时间戳格式：
- **刚刚**：60秒内的结果
- **X分钟前**：1小时内的结果
- **X小时前**：24小时内的结果
- **MM-DD HH:MM**：超过24小时的结果显示具体日期时间

### 3. 主题适配的视觉设计

- 支持深色和浅色主题
- 成功/失败状态使用不同的颜色和图标
- 保持与整体UI风格的一致性

## 技术实现

### 核心函数修改

#### 1. `render_tool_calls_in_message` 函数签名更新

```rust
// 之前
fn render_tool_calls_in_message(
    ui: &mut egui::Ui, 
    tool_calls: &[crate::api::ToolCall], 
    _max_width: f32
) -> Option<crate::api::ToolCall>

// 优化后
fn render_tool_calls_in_message(
    ui: &mut egui::Ui, 
    tool_calls: &[crate::api::ToolCall], 
    _max_width: f32, 
    tool_results: Option<&[crate::state::ToolCallResult]>
) -> Option<crate::api::ToolCall>
```

#### 2. 新增辅助函数

- `render_single_tool_result()`: 渲染单个工具调用结果
- `format_tool_result_timestamp()`: 格式化时间戳显示

### 数据结构

工具调用结果通过 `tool_call_id` 与工具调用关联：

```rust
pub struct ToolCallResult {
    pub tool_call_id: String,      // 关联的工具调用ID
    pub result: String,             // 执行结果
    pub success: bool,              // 是否成功
    pub error: Option<String>,      // 错误信息
    pub timestamp: DateTime<Utc>,   // 执行时间戳
}
```

### UI渲染逻辑

1. **工具调用渲染**：显示工具名称、参数和执行按钮
2. **结果关联**：根据 `tool_call_id` 筛选相关结果
3. **时间排序**：按时间戳对结果进行排序
4. **结果显示**：在工具调用下方显示所有相关结果

## 向后兼容性

- 保持对现有消息格式的兼容
- 如果消息只有工具调用结果而没有工具调用，仍会单独显示结果
- 不影响现有的工具调用执行流程

## 测试覆盖

新增了以下测试用例：

1. **时间戳格式化测试**：验证不同时间间隔的格式化输出
2. **工具调用和结果关联测试**：验证数据结构的正确性和关联逻辑

## 用户体验改进

### 视觉改进
- 更清晰的信息层次结构
- 一致的主题适配
- 直观的成功/失败状态指示

### 交互改进
- 减少用户在聊天记录中查找结果的时间
- 提供实时的执行状态反馈
- 支持多个结果的时间顺序显示

### 信息密度优化
- 相关信息集中显示
- 减少界面滚动需求
- 提高信息查找效率

## 未来扩展

1. **结果筛选**：可以添加按状态筛选结果的功能
2. **结果导出**：支持将工具调用结果导出为文件
3. **结果搜索**：在大量结果中快速搜索特定内容
4. **结果统计**：显示工具调用的成功率和执行时间统计

## 问题修复 (2024-06-19)

### 修复的问题

1. **重复提示框问题**
   - **问题**：点击一次执行按钮，出现两个"开始执行 1 个工具调用..."提示
   - **原因**：UI中点击执行按钮时记录一次日志，`execute_single_tool_call`中又记录一次日志，主应用程序中还会创建独立的系统消息
   - **解决方案**：
     - 将UI中的日志级别从`info`改为`debug`
     - 将`execute_single_tool_call`中的日志级别从`info`改为`debug`
     - 移除主应用程序中创建独立系统消息的逻辑

2. **工具执行结果独立显示问题**
   - **问题**：工具执行的提示和执行结果仍然是独立的消息框，未按要求与"AI助手请求调用 xx 个工具"放在一个框内
   - **原因**：主应用程序中的`process_pending_tool_execution`方法创建独立的系统消息来显示执行状态和结果
   - **解决方案**：
     - 修改主应用程序中的工具执行逻辑，不再创建独立的系统消息
     - 新增`add_tool_results_to_message`方法，将工具调用结果直接添加到原始的工具调用消息中
     - 修改UI渲染逻辑，在工具调用框内部显示相关的执行结果

### 技术实现细节

#### 1. 修改的文件

- `crates/aiAssist/src/ui.rs`：更新工具调用渲染函数，支持在同一框内显示结果
- `crates/aiAssist/src/state.rs`：减少重复日志记录
- `src/app.rs`：修改工具执行逻辑，将结果添加到原始消息而非创建新消息

#### 2. 核心修改

**UI渲染逻辑**：
```rust
// 修改函数签名，支持传递工具结果
fn render_tool_calls_in_message(
    ui: &mut egui::Ui,
    tool_calls: &[crate::api::ToolCall],
    _max_width: f32,
    tool_results: Option<&[crate::state::ToolCallResult]>  // 新增参数
) -> Option<crate::api::ToolCall>

// 在工具调用下方显示相关的执行结果
if let Some(results) = tool_results {
    let related_results: Vec<_> = results.iter()
        .filter(|result| result.tool_call_id == tool_call.id)
        .collect();

    if !related_results.is_empty() {
        // 按时间戳排序并显示结果
        let mut sorted_results = related_results;
        sorted_results.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

        for result in sorted_results {
            render_single_tool_result(ui, result, is_dark_mode);
        }
    }
}
```

**主应用程序逻辑**：
```rust
// 新增方法：将工具调用结果添加到对应的工具调用消息中
fn add_tool_results_to_message(&mut self, tool_call_results: &[aiAssist::state::ToolCallResult]) {
    // 找到最近的包含工具调用的助手消息
    for message in self.ai_assist_state.chat_messages.iter_mut().rev() {
        if message.role == aiAssist::state::MessageRole::Assistant && message.tool_calls.is_some() {
            // 检查工具调用ID匹配并添加结果
            if let Some(tool_calls) = &message.tool_calls {
                let tool_call_ids: std::collections::HashSet<String> =
                    tool_calls.iter().map(|tc| tc.id.clone()).collect();
                let result_ids: std::collections::HashSet<String> =
                    tool_call_results.iter().map(|tr| tr.tool_call_id.clone()).collect();

                if !tool_call_ids.is_disjoint(&result_ids) {
                    let mut all_results = message.tool_call_results.clone().unwrap_or_default();
                    all_results.extend_from_slice(tool_call_results);
                    message.tool_call_results = Some(all_results);
                    break;
                }
            }
        }
    }
}
```

#### 3. 测试验证

- 所有现有测试继续通过 (4/4)
- 新增的时间戳格式化测试通过
- 新增的工具调用和结果关联测试通过
- 编译成功，无错误

### 用户体验改进

1. **消除重复提示**：用户不再看到重复的"开始执行工具调用"消息
2. **统一信息显示**：工具调用和执行结果现在在同一个消息框中显示
3. **时间顺序清晰**：执行结果按时间戳排序，显示在工具调用下方
4. **视觉层次优化**：相关信息集中显示，减少界面混乱

## 总结

本次优化显著改善了AI助手中工具调用结果的显示体验，通过将相关信息集中显示、添加时间戳和优化视觉设计，为用户提供了更直观、更高效的工具调用结果查看方式。修复了重复提示和独立消息框的问题，实现了用户要求的统一显示效果。
