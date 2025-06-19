#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::AIAssistState;
    use crate::mcp_tools::{McpServerCapabilities, McpToolInfo};
    use uuid::Uuid;
    use std::collections::HashMap;

    #[test]
    fn test_mcp_server_selection_logging() {
        // 初始化日志
        let _ = env_logger::builder().is_test(true).try_init();
        
        // 创建AI助手状态
        let mut state = AIAssistState::default();
        
        // 创建测试MCP服务器
        let server_id = Uuid::new_v4();
        let server_name = "测试MCP服务器".to_string();
        
        // 添加服务器名称
        state.server_names.insert(server_id, server_name.clone());
        
        // 创建测试能力
        let capabilities = McpServerCapabilities {
            tools: vec![
                McpToolInfo {
                    name: "test_tool".to_string(),
                    description: Some("测试工具".to_string()),
                    input_schema: Some(serde_json::json!({
                        "type": "object",
                        "properties": {
                            "param1": {"type": "string"}
                        }
                    })),
                }
            ],
            resources: vec![],
            prompts: vec![],
        };
        
        // 添加服务器能力
        state.update_mcp_server_capabilities(server_id, capabilities);
        
        // 测试选择服务器（应该记录日志）
        println!("=== 测试选择MCP服务器 ===");
        state.set_selected_mcp_server(Some(server_id));
        
        // 验证选择
        assert_eq!(state.get_selected_mcp_server(), Some(server_id));
        
        // 测试取消选择（应该记录日志）
        println!("=== 测试取消选择MCP服务器 ===");
        state.set_selected_mcp_server(None);
        
        // 验证取消选择
        assert_eq!(state.get_selected_mcp_server(), None);
        
        println!("=== 测试完成 ===");
    }
    
    #[test]
    fn test_mcp_server_selection_change_detection() {
        // 初始化日志
        let _ = env_logger::builder().is_test(true).try_init();
        
        // 创建AI助手状态
        let mut state = AIAssistState::default();
        
        // 创建两个测试MCP服务器
        let server_id1 = Uuid::new_v4();
        let server_id2 = Uuid::new_v4();
        
        state.server_names.insert(server_id1, "服务器1".to_string());
        state.server_names.insert(server_id2, "服务器2".to_string());
        
        // 创建测试能力
        let capabilities = McpServerCapabilities {
            tools: vec![],
            resources: vec![],
            prompts: vec![],
        };
        
        state.update_mcp_server_capabilities(server_id1, capabilities.clone());
        state.update_mcp_server_capabilities(server_id2, capabilities);
        
        // 测试从无到有的选择
        println!("=== 测试从无到有的选择 ===");
        let previous = state.selected_mcp_server;
        state.selected_mcp_server = Some(server_id1);
        state.check_mcp_server_selection_change(previous);
        
        // 测试服务器切换
        println!("=== 测试服务器切换 ===");
        let previous = state.selected_mcp_server;
        state.selected_mcp_server = Some(server_id2);
        state.check_mcp_server_selection_change(previous);
        
        // 测试从有到无的选择
        println!("=== 测试从有到无的选择 ===");
        let previous = state.selected_mcp_server;
        state.selected_mcp_server = None;
        state.check_mcp_server_selection_change(previous);
        
        println!("=== 测试完成 ===");
    }

    #[test]
    fn test_tool_call_result_timestamp_formatting() {
        use chrono::{DateTime, Utc, Duration};
        use crate::ui::format_tool_result_timestamp;

        // 测试刚刚执行的结果
        let now = Utc::now();
        let just_now = now - Duration::seconds(30);
        assert_eq!(format_tool_result_timestamp(&just_now), "刚刚");

        // 测试几分钟前的结果
        let minutes_ago = now - Duration::minutes(5);
        assert_eq!(format_tool_result_timestamp(&minutes_ago), "5分钟前");

        // 测试几小时前的结果
        let hours_ago = now - Duration::hours(2);
        assert_eq!(format_tool_result_timestamp(&hours_ago), "2小时前");

        // 测试几天前的结果（应该显示具体时间）
        let days_ago = now - Duration::days(2);
        let formatted = format_tool_result_timestamp(&days_ago);
        assert!(formatted.contains("-") && formatted.contains(":"));

        println!("时间戳格式化测试完成");
    }

    #[test]
    fn test_chat_message_with_tool_calls_and_results() {
        use crate::state::{ChatMessage, MessageRole, ToolCallResult};
        use crate::api::ToolCall;
        use chrono::Utc;
        use uuid::Uuid;

        // 创建一个包含工具调用的消息
        let tool_call_id = "test_tool_call_123".to_string();
        let tool_call = ToolCall {
            id: tool_call_id.clone(),
            call_type: "function".to_string(),
            function: crate::api::FunctionCall {
                name: "test_function".to_string(),
                arguments: r#"{"param1": "value1"}"#.to_string(),
            },
        };

        let mut message = ChatMessage {
            id: Uuid::new_v4(),
            role: MessageRole::Assistant,
            content: "我将为您执行工具调用".to_string(),
            timestamp: Utc::now(),
            attachments: vec![],
            tool_calls: Some(vec![tool_call]),
            tool_call_results: None,
            mcp_server_info: None,
        };

        // 添加工具调用结果
        let result = ToolCallResult {
            tool_call_id: tool_call_id.clone(),
            result: "工具执行成功，返回结果：测试数据".to_string(),
            success: true,
            error: None,
            timestamp: Utc::now(),
        };

        message.tool_call_results = Some(vec![result]);

        // 验证消息结构
        assert!(message.tool_calls.is_some());
        assert!(message.tool_call_results.is_some());
        assert_eq!(message.tool_calls.as_ref().unwrap().len(), 1);
        assert_eq!(message.tool_call_results.as_ref().unwrap().len(), 1);

        // 验证工具调用ID匹配
        let tool_call_id_from_call = &message.tool_calls.as_ref().unwrap()[0].id;
        let tool_call_id_from_result = &message.tool_call_results.as_ref().unwrap()[0].tool_call_id;
        assert_eq!(tool_call_id_from_call, tool_call_id_from_result);

        println!("工具调用和结果关联测试完成");
    }

    #[test]
    fn test_multiple_tool_call_results_unique_ids() {
        use crate::state::ToolCallResult;
        use chrono::{Utc, Duration};

        // 创建多个具有相同tool_call_id但不同时间戳的结果
        let tool_call_id = "test_tool_123".to_string();
        let base_time = Utc::now();

        let results = vec![
            ToolCallResult {
                tool_call_id: tool_call_id.clone(),
                result: "第一次执行结果".to_string(),
                success: true,
                error: None,
                timestamp: base_time,
            },
            ToolCallResult {
                tool_call_id: tool_call_id.clone(),
                result: "第二次执行结果".to_string(),
                success: true,
                error: None,
                timestamp: base_time + Duration::seconds(1),
            },
            ToolCallResult {
                tool_call_id: tool_call_id.clone(),
                result: "第三次执行结果".to_string(),
                success: false,
                error: Some("模拟错误".to_string()),
                timestamp: base_time + Duration::seconds(2),
            },
        ];

        // 验证每个结果都有唯一的时间戳
        for (i, result) in results.iter().enumerate() {
            let unique_id = format!("{}_{}_{}_{}",
                result.tool_call_id,
                result.timestamp.timestamp_millis(),
                i,
                result.success
            );

            // 验证ID是唯一的
            assert!(!unique_id.is_empty());
            assert!(unique_id.contains(&result.tool_call_id));
            assert!(unique_id.contains(&result.timestamp.timestamp_millis().to_string()));
            assert!(unique_id.contains(&i.to_string()));
            assert!(unique_id.contains(&result.success.to_string()));
        }

        // 验证所有ID都是不同的
        let mut unique_ids = std::collections::HashSet::new();
        for (i, result) in results.iter().enumerate() {
            let unique_id = format!("{}_{}_{}_{}",
                result.tool_call_id,
                result.timestamp.timestamp_millis(),
                i,
                result.success
            );
            assert!(unique_ids.insert(unique_id), "发现重复的ID");
        }

        println!("多次工具调用结果唯一ID测试完成");
    }

    #[test]
    fn test_precise_timestamp_formatting() {
        use crate::ui::format_precise_timestamp;
        use chrono::{Utc, TimeZone};

        // 创建一个固定的时间戳进行测试
        let test_time = Utc.with_ymd_and_hms(2024, 6, 19, 14, 30, 45).unwrap();

        // 格式化时间戳
        let formatted = format_precise_timestamp(&test_time);

        // 验证格式是否正确（注意：这会根据本地时区进行转换）
        // 格式应该是 yyyy-mm-dd HH:MM:SS
        assert!(formatted.len() == 19); // "2024-06-19 14:30:45" 的长度
        assert!(formatted.contains("-")); // 包含日期分隔符
        assert!(formatted.contains(":")); // 包含时间分隔符
        assert!(formatted.contains(" ")); // 包含日期和时间之间的空格

        // 验证格式模式
        let parts: Vec<&str> = formatted.split(' ').collect();
        assert_eq!(parts.len(), 2); // 应该有日期和时间两部分

        let date_part = parts[0];
        let time_part = parts[1];

        // 验证日期部分格式 (yyyy-mm-dd)
        let date_components: Vec<&str> = date_part.split('-').collect();
        assert_eq!(date_components.len(), 3);
        assert_eq!(date_components[0].len(), 4); // 年份4位
        assert_eq!(date_components[1].len(), 2); // 月份2位
        assert_eq!(date_components[2].len(), 2); // 日期2位

        // 验证时间部分格式 (HH:MM:SS)
        let time_components: Vec<&str> = time_part.split(':').collect();
        assert_eq!(time_components.len(), 3);
        assert_eq!(time_components[0].len(), 2); // 小时2位
        assert_eq!(time_components[1].len(), 2); // 分钟2位
        assert_eq!(time_components[2].len(), 2); // 秒2位

        println!("精准时间戳格式化测试完成: {}", formatted);
    }
}
