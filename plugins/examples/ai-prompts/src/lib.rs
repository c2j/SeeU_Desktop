use seeu_plugin_sdk::*;
use std::collections::HashMap;

#[derive(Default)]
pub struct AIPromptsPlugin {
    prompts: HashMap<String, PromptTemplate>,
}

#[derive(Clone)]
struct PromptTemplate {
    name: String,
    description: String,
    template: String,
    arguments: Vec<PromptArgument>,
}

impl Plugin for AIPromptsPlugin {
    fn init(&mut self) -> Result<(), PluginError> {
        // 初始化提示模板
        self.prompts.insert("code_review".to_string(), PromptTemplate {
            name: "code_review".to_string(),
            description: "代码审查提示模板".to_string(),
            template: "请审查以下{{language}}代码，重点关注：\n\n1. 代码质量和可读性\n2. 潜在的bug和安全问题\n3. 性能优化建议\n4. 最佳实践建议\n\n代码：\n```{{language}}\n{{code}}\n```\n\n{{#if context}}上下文信息：{{context}}{{/if}}".to_string(),
            arguments: vec![
                PromptArgument {
                    name: "language".to_string(),
                    description: "编程语言".to_string(),
                    required: true,
                    argument_type: "string".to_string(),
                },
                PromptArgument {
                    name: "code".to_string(),
                    description: "要审查的代码".to_string(),
                    required: true,
                    argument_type: "string".to_string(),
                },
                PromptArgument {
                    name: "context".to_string(),
                    description: "额外的上下文信息".to_string(),
                    required: false,
                    argument_type: "string".to_string(),
                },
            ],
        });

        self.prompts.insert("explain_code".to_string(), PromptTemplate {
            name: "explain_code".to_string(),
            description: "代码解释提示模板".to_string(),
            template: "请详细解释以下{{language}}代码的功能和工作原理：\n\n```{{language}}\n{{code}}\n```\n\n请包括：\n1. 代码的主要功能\n2. 关键算法或逻辑\n3. 输入输出说明\n4. 使用示例{{#if audience}}\n\n目标受众：{{audience}}{{/if}}".to_string(),
            arguments: vec![
                PromptArgument {
                    name: "language".to_string(),
                    description: "编程语言".to_string(),
                    required: true,
                    argument_type: "string".to_string(),
                },
                PromptArgument {
                    name: "code".to_string(),
                    description: "要解释的代码".to_string(),
                    required: true,
                    argument_type: "string".to_string(),
                },
                PromptArgument {
                    name: "audience".to_string(),
                    description: "目标受众（如：初学者、专家等）".to_string(),
                    required: false,
                    argument_type: "string".to_string(),
                },
            ],
        });

        self.prompts.insert("write_documentation".to_string(), PromptTemplate {
            name: "write_documentation".to_string(),
            description: "文档编写提示模板".to_string(),
            template: "为以下{{type}}编写{{doc_type}}文档：\n\n{{#if title}}标题：{{title}}\n{{/if}}内容：\n{{content}}\n\n请确保文档：\n1. 结构清晰，易于理解\n2. 包含必要的示例\n3. 符合{{format}}格式\n4. 适合{{audience}}阅读".to_string(),
            arguments: vec![
                PromptArgument {
                    name: "type".to_string(),
                    description: "文档类型（如：API、函数、类等）".to_string(),
                    required: true,
                    argument_type: "string".to_string(),
                },
                PromptArgument {
                    name: "doc_type".to_string(),
                    description: "文档种类（如：用户手册、技术文档等）".to_string(),
                    required: true,
                    argument_type: "string".to_string(),
                },
                PromptArgument {
                    name: "content".to_string(),
                    description: "要编写文档的内容".to_string(),
                    required: true,
                    argument_type: "string".to_string(),
                },
                PromptArgument {
                    name: "format".to_string(),
                    description: "文档格式（如：Markdown、HTML等）".to_string(),
                    required: false,
                    argument_type: "string".to_string(),
                },
                PromptArgument {
                    name: "audience".to_string(),
                    description: "目标读者".to_string(),
                    required: false,
                    argument_type: "string".to_string(),
                },
                PromptArgument {
                    name: "title".to_string(),
                    description: "文档标题".to_string(),
                    required: false,
                    argument_type: "string".to_string(),
                },
            ],
        });

        self.prompts.insert("debug_help".to_string(), PromptTemplate {
            name: "debug_help".to_string(),
            description: "调试帮助提示模板".to_string(),
            template: "我在调试{{language}}代码时遇到了问题：\n\n错误信息：\n{{error}}\n\n相关代码：\n```{{language}}\n{{code}}\n```\n\n{{#if steps}}已尝试的解决步骤：\n{{steps}}\n{{/if}}请帮助我：\n1. 分析错误原因\n2. 提供解决方案\n3. 给出预防类似问题的建议".to_string(),
            arguments: vec![
                PromptArgument {
                    name: "language".to_string(),
                    description: "编程语言".to_string(),
                    required: true,
                    argument_type: "string".to_string(),
                },
                PromptArgument {
                    name: "error".to_string(),
                    description: "错误信息或问题描述".to_string(),
                    required: true,
                    argument_type: "string".to_string(),
                },
                PromptArgument {
                    name: "code".to_string(),
                    description: "相关代码".to_string(),
                    required: true,
                    argument_type: "string".to_string(),
                },
                PromptArgument {
                    name: "steps".to_string(),
                    description: "已尝试的解决步骤".to_string(),
                    required: false,
                    argument_type: "string".to_string(),
                },
            ],
        });

        Ok(())
    }

    fn get_capabilities(&self) -> PluginCapabilities {
        PluginCapabilities {
            provides_prompts: true,
            ..Default::default()
        }
    }

    fn get_metadata(&self) -> PluginMetadata {
        PluginMetadata {
            name: "ai-prompts".to_string(),
            display_name: "AI Prompts Plugin".to_string(),
            version: "0.1.0".to_string(),
            description: "AI提示模板集合，提供代码审查、解释、文档编写等模板".to_string(),
            author: "SeeU Team".to_string(),
            license: "MIT".to_string(),
            homepage: Some("https://github.com/c2j/SeeU_Desktop".to_string()),
            repository: Some("https://github.com/c2j/SeeU_Desktop".to_string()),
            keywords: vec!["ai".to_string(), "prompts".to_string(), "templates".to_string()],
            categories: vec!["prompts".to_string(), "ai".to_string()],
        }
    }

    fn get_permissions(&self) -> Vec<PluginPermission> {
        vec![]
    }

    fn handle_request(&mut self, request: PluginRequest) -> PluginResponse {
        match request.method.as_str() {
            "prompts/list" => {
                let prompts: Vec<PromptDefinition> = self.prompts.values().map(|template| {
                    PromptDefinition {
                        name: template.name.clone(),
                        description: template.description.clone(),
                        arguments: template.arguments.clone(),
                    }
                }).collect();
                
                utils::success_response(request.id, json!({"prompts": prompts}))
            }
            
            "prompts/get" => {
                if let Ok(params) = utils::parse_json::<serde_json::Value>(&request.params) {
                    if let Some(name) = params.get("name").and_then(|n| n.as_str()) {
                        if let Some(template) = self.prompts.get(name) {
                            let arguments = params.get("arguments")
                                .and_then(|a| a.as_object())
                                .cloned()
                                .unwrap_or_default();
                            
                            let rendered = render_template(&template.template, &arguments);
                            
                            utils::success_response(request.id, json!({
                                "messages": [
                                    {
                                        "role": "user",
                                        "content": {
                                            "type": "text",
                                            "text": rendered
                                        }
                                    }
                                ]
                            }))
                        } else {
                            utils::error_response(request.id, -32602, format!("Prompt not found: {}", name))
                        }
                    } else {
                        utils::error_response(request.id, -32602, "Missing prompt name".to_string())
                    }
                } else {
                    utils::error_response(request.id, -32602, "Invalid parameters".to_string())
                }
            }
            
            _ => utils::error_response(request.id, -32601, "Method not found".to_string()),
        }
    }

    fn cleanup(&mut self) {
        self.prompts.clear();
    }
}

/// 简单的模板渲染函数
fn render_template(template: &str, arguments: &serde_json::Map<String, serde_json::Value>) -> String {
    let mut result = template.to_string();
    
    // 替换简单的变量 {{variable}}
    for (key, value) in arguments {
        let placeholder = format!("{{{{{}}}}}", key);
        let replacement = match value {
            serde_json::Value::String(s) => s.clone(),
            _ => value.to_string(),
        };
        result = result.replace(&placeholder, &replacement);
    }
    
    // 处理条件语句 {{#if variable}}...{{/if}}
    // 这是一个简化的实现，实际应用中可能需要更复杂的模板引擎
    for (key, value) in arguments {
        let if_start = format!("{{{{#if {}}}}}", key);
        let if_end = "{{/if}}";
        
        if let Some(start_pos) = result.find(&if_start) {
            if let Some(end_pos) = result[start_pos..].find(if_end) {
                let end_pos = start_pos + end_pos;
                let content = &result[start_pos + if_start.len()..end_pos];
                
                let replacement = if !value.is_null() && value != &serde_json::Value::Bool(false) {
                    content.to_string()
                } else {
                    String::new()
                };
                
                result.replace_range(start_pos..end_pos + if_end.len(), &replacement);
            }
        }
    }
    
    result
}

export_plugin!(AIPromptsPlugin);
