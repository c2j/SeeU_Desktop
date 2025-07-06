use aiAssist::state::AIAssistState;
use aiAssist::mcp_tools::{McpServerCapabilities, McpToolInfo, McpResourceInfo, McpPromptInfo, McpPromptArgument};
use aiAssist::mcp_integration::McpIntegrationManager;
use uuid::Uuid;
use serde_json::json;

#[test]
fn test_mcp_integration_manager_instantiation() {
    let _manager = McpIntegrationManager::new();
    // 测试MCP集成管理器能够正常创建
    assert!(true); // 管理器创建成功
}

#[test]
fn test_ai_assist_state_mcp_server_management() {
    let mut state = AIAssistState::default();
    let server_id = Uuid::new_v4();
    let server_name = "Test MCP Server".to_string();
    
    // 测试设置选中的MCP服务器
    state.set_selected_mcp_server(Some(server_id));
    assert_eq!(state.selected_mcp_server, Some(server_id));
    
    // 测试更新服务器能力
    let capabilities = McpServerCapabilities {
        tools: vec![
            McpToolInfo {
                name: "test_tool".to_string(),
                description: Some("Test tool".to_string()),
                input_schema: Some(json!({"type": "object"})),
            }
        ],
        resources: vec![],
        prompts: vec![],
    };
    
    state.update_mcp_server_capabilities(server_id, capabilities.clone());
    assert!(state.mcp_server_capabilities.contains_key(&server_id));
    assert_eq!(state.mcp_server_capabilities[&server_id].tools.len(), 1);
    
    // 测试服务器名称管理
    state.server_names.insert(server_id, server_name.clone());
    assert_eq!(state.server_names[&server_id], server_name);
}

#[test]
fn test_mcp_server_capabilities_structure() {
    let capabilities = McpServerCapabilities {
        tools: vec![
            McpToolInfo {
                name: "calculator".to_string(),
                description: Some("A calculator tool".to_string()),
                input_schema: Some(json!({
                    "type": "object",
                    "properties": {
                        "operation": {"type": "string"},
                        "a": {"type": "number"},
                        "b": {"type": "number"}
                    }
                })),
            }
        ],
        resources: vec![
            McpResourceInfo {
                uri: "file://test.txt".to_string(),
                name: "test file".to_string(),
                description: Some("A test file".to_string()),
                mime_type: Some("text/plain".to_string()),
            }
        ],
        prompts: vec![
            McpPromptInfo {
                name: "test_prompt".to_string(),
                description: Some("A test prompt".to_string()),
                arguments: vec![],
            }
        ],
    };
    
    assert_eq!(capabilities.tools.len(), 1);
    assert_eq!(capabilities.resources.len(), 1);
    assert_eq!(capabilities.prompts.len(), 1);
    assert_eq!(capabilities.tools[0].name, "calculator");
    assert_eq!(capabilities.resources[0].uri, "file://test.txt");
    assert_eq!(capabilities.prompts[0].name, "test_prompt");
}

#[test]
fn test_mcp_tool_info_structure() {
    let tool_info = McpToolInfo {
        name: "file_reader".to_string(),
        description: Some("Read file contents".to_string()),
        input_schema: Some(json!({
            "type": "object",
            "properties": {
                "path": {"type": "string", "description": "File path to read"}
            },
            "required": ["path"]
        })),
    };
    
    assert_eq!(tool_info.name, "file_reader");
    assert!(tool_info.description.is_some());
    assert!(tool_info.input_schema.is_some());
    
    if let Some(schema) = &tool_info.input_schema {
        assert_eq!(schema["type"], "object");
        assert!(schema["properties"].is_object());
    }
}

#[test]
fn test_mcp_resource_info_structure() {
    let resource_info = McpResourceInfo {
        uri: "file:///path/to/document.txt".to_string(),
        name: "Important Document".to_string(),
        description: Some("A very important document".to_string()),
        mime_type: Some("text/plain".to_string()),
    };
    
    assert_eq!(resource_info.uri, "file:///path/to/document.txt");
    assert_eq!(resource_info.name, "Important Document");
    assert!(resource_info.description.is_some());
    assert!(resource_info.mime_type.is_some());
}

#[test]
fn test_mcp_prompt_info_structure() {
    let prompt_info = McpPromptInfo {
        name: "code_review".to_string(),
        description: Some("Review code for best practices".to_string()),
        arguments: vec![
            McpPromptArgument {
                name: "language".to_string(),
                description: Some("Programming language".to_string()),
                required: true,
            },
            McpPromptArgument {
                name: "code".to_string(),
                description: Some("Code to review".to_string()),
                required: true,
            },
        ],
    };

    assert_eq!(prompt_info.name, "code_review");
    assert!(prompt_info.description.is_some());
    assert_eq!(prompt_info.arguments.len(), 2);

    let language_arg = McpPromptArgument {
        name: "language".to_string(),
        description: Some("Programming language".to_string()),
        required: true,
    };
    let code_arg = McpPromptArgument {
        name: "code".to_string(),
        description: Some("Code to review".to_string()),
        required: true,
    };

    assert!(prompt_info.arguments.contains(&language_arg));
    assert!(prompt_info.arguments.contains(&code_arg));
}
