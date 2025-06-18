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
}
