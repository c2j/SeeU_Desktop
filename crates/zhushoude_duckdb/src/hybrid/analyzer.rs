//! 代码图分析器模块

use crate::{Result, DatabaseManager, GraphNode, CodeSearchResult};
use std::sync::Arc;

/// 代码图分析器
pub struct CodeGraphAnalyzer {
    db_manager: Arc<DatabaseManager>,
}

impl CodeGraphAnalyzer {
    /// 创建新的代码图分析器
    pub fn new(db_manager: Arc<DatabaseManager>) -> Self {
        Self { db_manager }
    }
    
    /// 分析代码并构建图
    pub async fn analyze_code(&self, code: &str, language: &str) -> Result<Vec<GraphNode>> {
        let mut nodes = Vec::new();
        
        match language.to_lowercase().as_str() {
            "rust" => {
                nodes = self.analyze_rust_code(code).await?;
            }
            "python" => {
                nodes = self.analyze_python_code(code).await?;
            }
            "javascript" | "typescript" => {
                nodes = self.analyze_js_code(code).await?;
            }
            _ => {
                // 通用代码分析
                nodes = self.analyze_generic_code(code).await?;
            }
        }
        
        Ok(nodes)
    }
    
    /// 分析Rust代码
    async fn analyze_rust_code(&self, code: &str) -> Result<Vec<GraphNode>> {
        let mut nodes = Vec::new();
        
        // 简单的正则表达式解析（实际应用中应使用AST解析器）
        use regex::Regex;
        
        // 查找结构体定义
        let struct_re = Regex::new(r"struct\s+(\w+)").unwrap();
        for cap in struct_re.captures_iter(code) {
            if let Some(name) = cap.get(1) {
                nodes.push(GraphNode {
                    id: format!("struct_{}", name.as_str()),
                    label: name.as_str().to_string(),
                    node_type: crate::NodeType::Class,
                    properties: std::collections::HashMap::new(),
                });
            }
        }
        
        // 查找函数定义
        let fn_re = Regex::new(r"fn\s+(\w+)").unwrap();
        for cap in fn_re.captures_iter(code) {
            if let Some(name) = cap.get(1) {
                nodes.push(GraphNode {
                    id: format!("fn_{}", name.as_str()),
                    label: name.as_str().to_string(),
                    node_type: crate::NodeType::Method,
                    properties: std::collections::HashMap::new(),
                });
            }
        }
        
        Ok(nodes)
    }
    
    /// 分析Python代码
    async fn analyze_python_code(&self, code: &str) -> Result<Vec<GraphNode>> {
        let mut nodes = Vec::new();
        
        use regex::Regex;
        
        // 查找类定义
        let class_re = Regex::new(r"class\s+(\w+)").unwrap();
        for cap in class_re.captures_iter(code) {
            if let Some(name) = cap.get(1) {
                nodes.push(GraphNode {
                    id: format!("class_{}", name.as_str()),
                    label: name.as_str().to_string(),
                    node_type: crate::NodeType::Class,
                    properties: std::collections::HashMap::new(),
                });
            }
        }
        
        // 查找函数定义
        let def_re = Regex::new(r"def\s+(\w+)").unwrap();
        for cap in def_re.captures_iter(code) {
            if let Some(name) = cap.get(1) {
                nodes.push(GraphNode {
                    id: format!("def_{}", name.as_str()),
                    label: name.as_str().to_string(),
                    node_type: crate::NodeType::Method,
                    properties: std::collections::HashMap::new(),
                });
            }
        }
        
        Ok(nodes)
    }
    
    /// 分析JavaScript/TypeScript代码
    async fn analyze_js_code(&self, code: &str) -> Result<Vec<GraphNode>> {
        let mut nodes = Vec::new();
        
        use regex::Regex;
        
        // 查找类定义
        let class_re = Regex::new(r"class\s+(\w+)").unwrap();
        for cap in class_re.captures_iter(code) {
            if let Some(name) = cap.get(1) {
                nodes.push(GraphNode {
                    id: format!("class_{}", name.as_str()),
                    label: name.as_str().to_string(),
                    node_type: crate::NodeType::Class,
                    properties: std::collections::HashMap::new(),
                });
            }
        }
        
        // 查找函数定义
        let fn_re = Regex::new(r"function\s+(\w+)").unwrap();
        for cap in fn_re.captures_iter(code) {
            if let Some(name) = cap.get(1) {
                nodes.push(GraphNode {
                    id: format!("function_{}", name.as_str()),
                    label: name.as_str().to_string(),
                    node_type: crate::NodeType::Method,
                    properties: std::collections::HashMap::new(),
                });
            }
        }
        
        Ok(nodes)
    }
    
    /// 通用代码分析
    async fn analyze_generic_code(&self, code: &str) -> Result<Vec<GraphNode>> {
        let mut nodes = Vec::new();
        
        // 基于行数和复杂度的简单分析
        let lines: Vec<&str> = code.lines().collect();
        let complexity = self.calculate_complexity(code);
        
        nodes.push(GraphNode {
            id: "generic_code".to_string(),
            label: format!("Code Block ({} lines)", lines.len()),
            node_type: crate::NodeType::Method,
            properties: {
                let mut props = std::collections::HashMap::new();
                props.insert("lines".to_string(), serde_json::Value::Number(serde_json::Number::from(lines.len())));
                props.insert("complexity".to_string(), serde_json::Value::Number(serde_json::Number::from(complexity)));
                props
            },
        });
        
        Ok(nodes)
    }
    
    /// 计算代码复杂度
    fn calculate_complexity(&self, code: &str) -> usize {
        // 简单的复杂度计算：基于控制流语句的数量
        let keywords = ["if", "else", "for", "while", "switch", "case", "try", "catch"];
        let mut complexity = 1; // 基础复杂度
        
        for keyword in &keywords {
            complexity += code.matches(keyword).count();
        }
        
        complexity
    }
    
    /// 查找相关代码
    pub async fn find_related_code(&self, query: &str) -> Result<Vec<CodeSearchResult>> {
        let mut results = Vec::new();

        // 这里应该实现基于图结构的代码搜索
        // 暂时返回模拟结果
        results.push(CodeSearchResult {
            node_id: "example_node".to_string(),
            label: "Example Function".to_string(),
            code_snippet: "fn example() { /* code */ }".to_string(),
            relevance_score: 0.8,
            properties: serde_json::json!({
                "language": "rust",
                "type": "function"
            }),
        });

        Ok(results)
    }
    
    /// 构建依赖关系图
    pub async fn build_dependency_graph(&self, project_path: &str) -> Result<()> {
        // 扫描项目文件
        let files = self.scan_project_files(project_path).await?;
        
        // 分析每个文件
        for file_path in files {
            if let Ok(content) = tokio::fs::read_to_string(&file_path).await {
                let language = self.detect_language(&file_path);
                let _nodes = self.analyze_code(&content, &language).await?;
                
                // 存储节点到图数据库
                // 这里需要调用GraphStorage来存储节点
            }
        }
        
        Ok(())
    }
    
    /// 扫描项目文件
    async fn scan_project_files(&self, project_path: &str) -> Result<Vec<String>> {
        let mut files = Vec::new();
        
        // 支持的文件扩展名
        let extensions = [".rs", ".py", ".js", ".ts", ".java", ".cpp", ".c", ".go"];
        
        // 递归扫描目录
        if let Ok(mut entries) = tokio::fs::read_dir(project_path).await {
            while let Ok(Some(entry)) = entries.next_entry().await {
                let path = entry.path();
                
                if path.is_file() {
                    if let Some(ext) = path.extension() {
                        if let Some(ext_str) = ext.to_str() {
                            if extensions.iter().any(|&e| e.ends_with(ext_str)) {
                                if let Some(path_str) = path.to_str() {
                                    files.push(path_str.to_string());
                                }
                            }
                        }
                    }
                }
            }
        }
        
        Ok(files)
    }
    
    /// 检测编程语言
    fn detect_language(&self, file_path: &str) -> String {
        if let Some(ext) = std::path::Path::new(file_path).extension() {
            match ext.to_str().unwrap_or("") {
                "rs" => "rust".to_string(),
                "py" => "python".to_string(),
                "js" => "javascript".to_string(),
                "ts" => "typescript".to_string(),
                "java" => "java".to_string(),
                "cpp" | "cc" | "cxx" => "cpp".to_string(),
                "c" => "c".to_string(),
                "go" => "go".to_string(),
                _ => "unknown".to_string(),
            }
        } else {
            "unknown".to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ZhushoudeConfig;
    
    #[tokio::test]
    async fn test_code_analysis() {
        let config = ZhushoudeConfig {
            database_path: ":memory:".to_string(),
            ..Default::default()
        };
        
        let db_manager = Arc::new(DatabaseManager::new(config).await.unwrap());
        let analyzer = CodeGraphAnalyzer::new(db_manager);
        
        let rust_code = r#"
            struct User {
                name: String,
                age: u32,
            }
            
            fn create_user(name: String, age: u32) -> User {
                User { name, age }
            }
        "#;
        
        let nodes = analyzer.analyze_code(rust_code, "rust").await.unwrap();
        assert!(!nodes.is_empty());
        
        // 应该找到一个结构体和一个函数
        let struct_nodes: Vec<_> = nodes.iter().filter(|n| matches!(n.node_type, crate::NodeType::Class)).collect();
        let fn_nodes: Vec<_> = nodes.iter().filter(|n| matches!(n.node_type, crate::NodeType::Method)).collect();
        
        assert_eq!(struct_nodes.len(), 1);
        assert_eq!(fn_nodes.len(), 1);
    }
}
