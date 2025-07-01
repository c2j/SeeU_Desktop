use aiAssist::api::{ApiService, Tool, FunctionDefinition, ToolCall, FunctionCall};
use serde_json::json;

#[test]
fn test_api_service_instantiation() {
    let _api_service = ApiService::new();
    // 测试API服务能够正常创建
    assert!(true); // API服务创建成功
}

#[test]
fn test_tool_definition_json_serialization() {
    let tool = Tool {
        tool_type: "function".to_string(),
        function: FunctionDefinition {
            name: "test_function".to_string(),
            description: Some("A test function".to_string()),
            parameters: Some(json!({
                "type": "object",
                "properties": {
                    "param1": {"type": "string"}
                }
            })),
        },
    };
    
    let serialized = serde_json::to_string(&tool).unwrap();
    let deserialized: Tool = serde_json::from_str(&serialized).unwrap();
    
    assert_eq!(tool.tool_type, deserialized.tool_type);
    assert_eq!(tool.function.name, deserialized.function.name);
}

#[test]
fn test_function_call_structure() {
    let function_call = FunctionCall {
        name: "test_function".to_string(),
        arguments: r#"{"param1": "value1", "param2": 42}"#.to_string(),
    };
    
    assert_eq!(function_call.name, "test_function");
    
    // 测试参数是否为有效的JSON
    let parsed_args: serde_json::Value = serde_json::from_str(&function_call.arguments).unwrap();
    assert_eq!(parsed_args["param1"], "value1");
    assert_eq!(parsed_args["param2"], 42);
}

#[test]
fn test_tool_call_structure() {
    let tool_call = ToolCall {
        id: "call_123".to_string(),
        call_type: "function".to_string(),
        function: FunctionCall {
            name: "test_function".to_string(),
            arguments: r#"{"test": true}"#.to_string(),
        },
    };
    
    assert_eq!(tool_call.id, "call_123");
    assert_eq!(tool_call.call_type, "function");
    assert_eq!(tool_call.function.name, "test_function");
}
