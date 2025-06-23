use seeu_plugin_sdk::*;

simple_tool_plugin! {
    name: "file-tools",
    display_name: "File Tools Plugin",
    version: "0.1.0",
    description: "A collection of file operation tools",
    author: "SeeU Team",
    tools: [
        ToolDefinition {
            name: "list_files".to_string(),
            description: "List files in a directory".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Directory path to list"
                    },
                    "include_hidden": {
                        "type": "boolean",
                        "description": "Include hidden files (default: false)"
                    }
                },
                "required": ["path"]
            }),
            output_schema: Some(json!({
                "type": "object",
                "properties": {
                    "files": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "properties": {
                                "name": {"type": "string"},
                                "type": {"type": "string"},
                                "size": {"type": "integer"},
                                "modified": {"type": "string"}
                            }
                        }
                    }
                }
            }))
        },
        ToolDefinition {
            name: "file_info".to_string(),
            description: "Get detailed information about a file".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "File path"
                    }
                },
                "required": ["path"]
            }),
            output_schema: Some(json!({
                "type": "object",
                "properties": {
                    "name": {"type": "string"},
                    "path": {"type": "string"},
                    "size": {"type": "integer"},
                    "type": {"type": "string"},
                    "extension": {"type": "string"},
                    "created": {"type": "string"},
                    "modified": {"type": "string"},
                    "permissions": {"type": "string"}
                }
            }))
        },
        ToolDefinition {
            name: "search_files".to_string(),
            description: "Search for files by name pattern".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "directory": {
                        "type": "string",
                        "description": "Directory to search in"
                    },
                    "pattern": {
                        "type": "string",
                        "description": "File name pattern (supports wildcards)"
                    },
                    "recursive": {
                        "type": "boolean",
                        "description": "Search recursively (default: false)"
                    }
                },
                "required": ["directory", "pattern"]
            }),
            output_schema: Some(json!({
                "type": "object",
                "properties": {
                    "matches": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "properties": {
                                "path": {"type": "string"},
                                "name": {"type": "string"},
                                "size": {"type": "integer"}
                            }
                        }
                    },
                    "total_found": {"type": "integer"}
                }
            }))
        },
        ToolDefinition {
            name: "calculate_hash".to_string(),
            description: "Calculate file hash (MD5, SHA1, SHA256)".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "File path"
                    },
                    "algorithm": {
                        "type": "string",
                        "enum": ["md5", "sha1", "sha256"],
                        "description": "Hash algorithm (default: sha256)"
                    }
                },
                "required": ["path"]
            }),
            output_schema: Some(json!({
                "type": "object",
                "properties": {
                    "hash": {"type": "string"},
                    "algorithm": {"type": "string"},
                    "file_size": {"type": "integer"}
                }
            }))
        }
    ],
    handler: handle_file_tool
}

fn handle_file_tool(tool_name: &str, arguments: serde_json::Value) -> Result<serde_json::Value, PluginError> {
    match tool_name {
        "list_files" => {
            let path = arguments.get("path")
                .and_then(|p| p.as_str())
                .ok_or_else(|| PluginError {
                    code: -32602,
                    message: "Missing required parameter: path".to_string(),
                    data: None,
                })?;
            
            let include_hidden = arguments.get("include_hidden")
                .and_then(|h| h.as_bool())
                .unwrap_or(false);
            
            // 模拟文件列表（在实际实现中，这里会调用主机API）
            let files = vec![
                json!({
                    "name": "document.txt",
                    "type": "file",
                    "size": 1024,
                    "modified": "2024-01-15T10:30:00Z"
                }),
                json!({
                    "name": "images",
                    "type": "directory",
                    "size": 0,
                    "modified": "2024-01-14T15:20:00Z"
                }),
                json!({
                    "name": "script.py",
                    "type": "file",
                    "size": 2048,
                    "modified": "2024-01-16T09:15:00Z"
                })
            ];
            
            Ok(json!({
                "files": files,
                "path": path,
                "include_hidden": include_hidden
            }))
        }
        
        "file_info" => {
            let path = arguments.get("path")
                .and_then(|p| p.as_str())
                .ok_or_else(|| PluginError {
                    code: -32602,
                    message: "Missing required parameter: path".to_string(),
                    data: None,
                })?;
            
            // 模拟文件信息
            Ok(json!({
                "name": "document.txt",
                "path": path,
                "size": 1024,
                "type": "text/plain",
                "extension": "txt",
                "created": "2024-01-10T08:00:00Z",
                "modified": "2024-01-15T10:30:00Z",
                "permissions": "rw-r--r--"
            }))
        }
        
        "search_files" => {
            let directory = arguments.get("directory")
                .and_then(|d| d.as_str())
                .ok_or_else(|| PluginError {
                    code: -32602,
                    message: "Missing required parameter: directory".to_string(),
                    data: None,
                })?;
            
            let pattern = arguments.get("pattern")
                .and_then(|p| p.as_str())
                .ok_or_else(|| PluginError {
                    code: -32602,
                    message: "Missing required parameter: pattern".to_string(),
                    data: None,
                })?;
            
            let recursive = arguments.get("recursive")
                .and_then(|r| r.as_bool())
                .unwrap_or(false);
            
            // 模拟搜索结果
            let matches = vec![
                json!({
                    "path": format!("{}/found_file1.txt", directory),
                    "name": "found_file1.txt",
                    "size": 512
                }),
                json!({
                    "path": format!("{}/subdir/found_file2.txt", directory),
                    "name": "found_file2.txt",
                    "size": 1024
                })
            ];
            
            Ok(json!({
                "matches": matches,
                "total_found": 2,
                "pattern": pattern,
                "recursive": recursive
            }))
        }
        
        "calculate_hash" => {
            let path = arguments.get("path")
                .and_then(|p| p.as_str())
                .ok_or_else(|| PluginError {
                    code: -32602,
                    message: "Missing required parameter: path".to_string(),
                    data: None,
                })?;
            
            let algorithm = arguments.get("algorithm")
                .and_then(|a| a.as_str())
                .unwrap_or("sha256");
            
            // 模拟哈希计算
            let hash = match algorithm {
                "md5" => "d41d8cd98f00b204e9800998ecf8427e",
                "sha1" => "da39a3ee5e6b4b0d3255bfef95601890afd80709",
                "sha256" => "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
                _ => return Err(PluginError {
                    code: -32602,
                    message: format!("Unsupported hash algorithm: {}", algorithm),
                    data: None,
                })
            };
            
            Ok(json!({
                "hash": hash,
                "algorithm": algorithm,
                "file_size": 1024,
                "path": path
            }))
        }
        
        _ => Err(PluginError {
            code: -32601,
            message: format!("Unknown tool: {}", tool_name),
            data: None,
        })
    }
}
