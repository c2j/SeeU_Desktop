use seeu_plugin_sdk::*;

// 使用宏创建一个简单的工具插件
simple_tool_plugin! {
    name: "hello-world",
    display_name: "Hello World Plugin",
    version: "0.1.0",
    description: "A simple plugin that demonstrates basic functionality",
    author: "SeeU Team",
    tools: [
        ToolDefinition {
            name: "hello".to_string(),
            description: "Say hello to someone".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "name": {
                        "type": "string",
                        "description": "Name of the person to greet"
                    }
                },
                "required": ["name"]
            }),
            output_schema: Some(json!({
                "type": "object",
                "properties": {
                    "message": {
                        "type": "string",
                        "description": "The greeting message"
                    }
                }
            }))
        },
        ToolDefinition {
            name: "echo".to_string(),
            description: "Echo back the input text".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "text": {
                        "type": "string",
                        "description": "Text to echo back"
                    }
                },
                "required": ["text"]
            }),
            output_schema: Some(json!({
                "type": "object",
                "properties": {
                    "echoed": {
                        "type": "string",
                        "description": "The echoed text"
                    }
                }
            }))
        },
        ToolDefinition {
            name: "random_number".to_string(),
            description: "Generate a random number within a range".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "min": {
                        "type": "integer",
                        "description": "Minimum value (default: 1)"
                    },
                    "max": {
                        "type": "integer",
                        "description": "Maximum value (default: 100)"
                    }
                }
            }),
            output_schema: Some(json!({
                "type": "object",
                "properties": {
                    "number": {
                        "type": "integer",
                        "description": "The generated random number"
                    }
                }
            }))
        }
    ],
    handler: handle_tool_call
}

/// Handle tool calls
fn handle_tool_call(tool_name: &str, arguments: serde_json::Value) -> Result<serde_json::Value, PluginError> {
    match tool_name {
        "hello" => {
            let name = arguments.get("name")
                .and_then(|n| n.as_str())
                .unwrap_or("World");
            
            Ok(json!({
                "message": format!("Hello, {}! 👋", name)
            }))
        }
        
        "echo" => {
            let text = arguments.get("text")
                .and_then(|t| t.as_str())
                .unwrap_or("");
            
            Ok(json!({
                "echoed": text
            }))
        }
        
        "random_number" => {
            let min = arguments.get("min")
                .and_then(|m| m.as_i64())
                .unwrap_or(1) as u32;
            
            let max = arguments.get("max")
                .and_then(|m| m.as_i64())
                .unwrap_or(100) as u32;
            
            if min > max {
                return Err(PluginError {
                    code: -32602,
                    message: "Minimum value cannot be greater than maximum value".to_string(),
                    data: None,
                });
            }
            
            // Simple pseudo-random number generation (for demo purposes)
            let range = max - min + 1;
            let number = min + (get_pseudo_random() % range);
            
            Ok(json!({
                "number": number
            }))
        }
        
        _ => Err(PluginError {
            code: -32601,
            message: format!("Unknown tool: {}", tool_name),
            data: None,
        })
    }
}

/// Simple pseudo-random number generator for demo purposes
fn get_pseudo_random() -> u32 {
    use std::sync::atomic::{AtomicU32, Ordering};
    static SEED: AtomicU32 = AtomicU32::new(1);
    
    let current = SEED.load(Ordering::Relaxed);
    let next = current.wrapping_mul(1103515245).wrapping_add(12345);
    SEED.store(next, Ordering::Relaxed);
    
    next
}
