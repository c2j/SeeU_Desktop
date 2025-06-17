use std::collections::HashMap;
use serde::{Deserialize, Serialize};

use super::server_manager::{McpServerConfig, TransportConfig};

/// Server template for easy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerTemplate {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: String,
    pub icon: String,
    pub config: McpServerConfig,
    pub requirements: Vec<String>,
    pub installation_notes: Option<String>,
    pub documentation_url: Option<String>,
    pub tags: Vec<String>,
}

/// Template category
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateCategory {
    pub id: String,
    pub name: String,
    pub description: String,
    pub icon: String,
}

/// Server template manager
#[derive(Debug)]
pub struct ServerTemplateManager {
    templates: HashMap<String, ServerTemplate>,
    categories: HashMap<String, TemplateCategory>,
}

impl ServerTemplateManager {
    /// Create a new template manager
    pub fn new() -> Self {
        let mut manager = Self {
            templates: HashMap::new(),
            categories: HashMap::new(),
        };
        
        manager.initialize_default_categories();
        manager.initialize_default_templates();
        
        manager
    }

    /// Initialize default categories
    fn initialize_default_categories(&mut self) {
        let categories = vec![
            TemplateCategory {
                id: "development".to_string(),
                name: "开发工具".to_string(),
                description: "代码开发和版本控制相关的MCP服务器".to_string(),
                icon: "💻".to_string(),
            },
            TemplateCategory {
                id: "filesystem".to_string(),
                name: "文件系统".to_string(),
                description: "文件和目录操作相关的MCP服务器".to_string(),
                icon: "📁".to_string(),
            },
            TemplateCategory {
                id: "database".to_string(),
                name: "数据库".to_string(),
                description: "数据库连接和查询相关的MCP服务器".to_string(),
                icon: "🗄️".to_string(),
            },
            TemplateCategory {
                id: "web".to_string(),
                name: "Web服务".to_string(),
                description: "Web API和网络服务相关的MCP服务器".to_string(),
                icon: "🌐".to_string(),
            },
            TemplateCategory {
                id: "ai".to_string(),
                name: "AI工具".to_string(),
                description: "人工智能和机器学习相关的MCP服务器".to_string(),
                icon: "🤖".to_string(),
            },
            TemplateCategory {
                id: "productivity".to_string(),
                name: "生产力工具".to_string(),
                description: "提高工作效率的MCP服务器".to_string(),
                icon: "⚡".to_string(),
            },
            TemplateCategory {
                id: "system".to_string(),
                name: "系统工具".to_string(),
                description: "系统管理和监控相关的MCP服务器".to_string(),
                icon: "⚙️".to_string(),
            },
        ];

        for category in categories {
            self.categories.insert(category.id.clone(), category);
        }
    }

    /// Initialize default templates
    fn initialize_default_templates(&mut self) {
        let templates = vec![
            // Development Tools
            ServerTemplate {
                id: "mcp-server-everything".to_string(),
                name: "Everything Server".to_string(),
                description: "包含所有MCP功能的综合服务器，适合测试和开发".to_string(),
                category: "development".to_string(),
                icon: "🔧".to_string(),
                config: McpServerConfig {
                    id: uuid::Uuid::new_v4(),
                    name: "Everything Server".to_string(),
                    description: Some("MCP server with everything capabilities".to_string()),
                    transport: TransportConfig::Command {
                        command: "npx".to_string(),
                        args: vec!["-y".to_string(), "@modelcontextprotocol/server-everything".to_string()],
                        env: HashMap::new(),
                    },
                    enabled: false,
                    auto_start: false,
                    directory: "Development".to_string(),
                    metadata: HashMap::new(),
                    last_health_status: None,
                    last_test_time: None,
                    last_test_success: None,
                },
                requirements: vec!["Node.js".to_string(), "npm".to_string()],
                installation_notes: Some("需要安装Node.js和npm。首次运行时会自动下载依赖。".to_string()),
                documentation_url: Some("https://github.com/modelcontextprotocol/servers".to_string()),
                tags: vec!["testing".to_string(), "development".to_string(), "comprehensive".to_string()],
            },

            ServerTemplate {
                id: "mcp-server-git".to_string(),
                name: "Git Server".to_string(),
                description: "Git版本控制操作服务器，支持仓库管理和版本控制".to_string(),
                category: "development".to_string(),
                icon: "🔀".to_string(),
                config: McpServerConfig {
                    id: uuid::Uuid::new_v4(),
                    name: "Git Server".to_string(),
                    description: Some("Git version control operations".to_string()),
                    transport: TransportConfig::Command {
                        command: "npx".to_string(),
                        args: vec!["-y".to_string(), "@modelcontextprotocol/server-git".to_string()],
                        env: HashMap::new(),
                    },
                    enabled: false,
                    auto_start: false,
                    directory: "Development".to_string(),
                    metadata: HashMap::new(),
                    last_health_status: None,
                    last_test_time: None,
                    last_test_success: None,
                },
                requirements: vec!["Node.js".to_string(), "Git".to_string()],
                installation_notes: Some("需要安装Git命令行工具".to_string()),
                documentation_url: Some("https://github.com/modelcontextprotocol/servers/tree/main/src/git".to_string()),
                tags: vec!["git".to_string(), "version-control".to_string(), "repository".to_string()],
            },

            // File System
            ServerTemplate {
                id: "mcp-server-filesystem".to_string(),
                name: "File System Server".to_string(),
                description: "文件系统操作服务器，支持文件读写和目录管理".to_string(),
                category: "filesystem".to_string(),
                icon: "📂".to_string(),
                config: McpServerConfig {
                    id: uuid::Uuid::new_v4(),
                    name: "File System Server".to_string(),
                    description: Some("File system operations".to_string()),
                    transport: TransportConfig::Command {
                        command: "npx".to_string(),
                        args: vec!["-y".to_string(), "@modelcontextprotocol/server-filesystem".to_string()],
                        env: HashMap::new(),
                    },
                    enabled: false,
                    auto_start: false,
                    directory: "FileSystem".to_string(),
                    metadata: HashMap::new(),
                    last_health_status: None,
                    last_test_time: None,
                    last_test_success: None,
                },
                requirements: vec!["Node.js".to_string()],
                installation_notes: Some("提供安全的文件系统访问，支持读写权限控制".to_string()),
                documentation_url: Some("https://github.com/modelcontextprotocol/servers/tree/main/src/filesystem".to_string()),
                tags: vec!["files".to_string(), "directories".to_string(), "io".to_string()],
            },

            // Database
            ServerTemplate {
                id: "mcp-server-sqlite".to_string(),
                name: "SQLite Server".to_string(),
                description: "SQLite数据库操作服务器，支持SQL查询和数据管理".to_string(),
                category: "database".to_string(),
                icon: "🗃️".to_string(),
                config: McpServerConfig {
                    id: uuid::Uuid::new_v4(),
                    name: "SQLite Server".to_string(),
                    description: Some("SQLite database operations".to_string()),
                    transport: TransportConfig::Command {
                        command: "npx".to_string(),
                        args: vec!["-y".to_string(), "@modelcontextprotocol/server-sqlite".to_string()],
                        env: HashMap::new(),
                    },
                    enabled: false,
                    auto_start: false,
                    directory: "Database".to_string(),
                    metadata: HashMap::new(),
                    last_health_status: None,
                    last_test_time: None,
                    last_test_success: None,
                },
                requirements: vec!["Node.js".to_string(), "SQLite".to_string()],
                installation_notes: Some("支持SQLite数据库的查询、插入、更新和删除操作".to_string()),
                documentation_url: Some("https://github.com/modelcontextprotocol/servers/tree/main/src/sqlite".to_string()),
                tags: vec!["sqlite".to_string(), "database".to_string(), "sql".to_string()],
            },

            // Web Services
            ServerTemplate {
                id: "mcp-server-fetch".to_string(),
                name: "Fetch Server".to_string(),
                description: "HTTP请求服务器，支持GET、POST等Web API调用".to_string(),
                category: "web".to_string(),
                icon: "🌍".to_string(),
                config: McpServerConfig {
                    id: uuid::Uuid::new_v4(),
                    name: "Fetch Server".to_string(),
                    description: Some("HTTP requests and web API calls".to_string()),
                    transport: TransportConfig::Command {
                        command: "npx".to_string(),
                        args: vec!["-y".to_string(), "@modelcontextprotocol/server-fetch".to_string()],
                        env: HashMap::new(),
                    },
                    enabled: false,
                    auto_start: false,
                    directory: "Web".to_string(),
                    metadata: HashMap::new(),
                    last_health_status: None,
                    last_test_time: None,
                    last_test_success: None,
                },
                requirements: vec!["Node.js".to_string()],
                installation_notes: Some("支持HTTP/HTTPS请求，可以调用REST API".to_string()),
                documentation_url: Some("https://github.com/modelcontextprotocol/servers/tree/main/src/fetch".to_string()),
                tags: vec!["http".to_string(), "api".to_string(), "web".to_string()],
            },

            // Productivity Tools
            ServerTemplate {
                id: "mcp-server-brave-search".to_string(),
                name: "Brave Search Server".to_string(),
                description: "Brave搜索引擎服务器，提供网络搜索功能".to_string(),
                category: "productivity".to_string(),
                icon: "🔍".to_string(),
                config: McpServerConfig {
                    id: uuid::Uuid::new_v4(),
                    name: "Brave Search Server".to_string(),
                    description: Some("Brave search engine integration".to_string()),
                    transport: TransportConfig::Command {
                        command: "npx".to_string(),
                        args: vec!["-y".to_string(), "@modelcontextprotocol/server-brave-search".to_string()],
                        env: HashMap::new(),
                    },
                    enabled: false,
                    auto_start: false,
                    directory: "Productivity".to_string(),
                    metadata: HashMap::new(),
                    last_health_status: None,
                    last_test_time: None,
                    last_test_success: None,
                },
                requirements: vec!["Node.js".to_string(), "Brave Search API Key".to_string()],
                installation_notes: Some("需要Brave Search API密钥，请访问https://brave.com/search/api/".to_string()),
                documentation_url: Some("https://github.com/modelcontextprotocol/servers/tree/main/src/brave-search".to_string()),
                tags: vec!["search".to_string(), "web".to_string(), "information".to_string()],
            },

            // System Tools
            ServerTemplate {
                id: "mcp-server-memory".to_string(),
                name: "Memory Server".to_string(),
                description: "内存存储服务器，提供临时数据存储和检索".to_string(),
                category: "system".to_string(),
                icon: "🧠".to_string(),
                config: McpServerConfig {
                    id: uuid::Uuid::new_v4(),
                    name: "Memory Server".to_string(),
                    description: Some("In-memory data storage and retrieval".to_string()),
                    transport: TransportConfig::Command {
                        command: "npx".to_string(),
                        args: vec!["-y".to_string(), "@modelcontextprotocol/server-memory".to_string()],
                        env: HashMap::new(),
                    },
                    enabled: false,
                    auto_start: false,
                    directory: "System".to_string(),
                    metadata: HashMap::new(),
                    last_health_status: None,
                    last_test_time: None,
                    last_test_success: None,
                },
                requirements: vec!["Node.js".to_string()],
                installation_notes: Some("提供会话期间的临时数据存储".to_string()),
                documentation_url: Some("https://github.com/modelcontextprotocol/servers/tree/main/src/memory".to_string()),
                tags: vec!["memory".to_string(), "storage".to_string(), "temporary".to_string()],
            },
        ];

        for template in templates {
            self.templates.insert(template.id.clone(), template);
        }
    }

    /// Get all templates
    pub fn get_templates(&self) -> Vec<&ServerTemplate> {
        self.templates.values().collect()
    }

    /// Get templates by category
    pub fn get_templates_by_category(&self, category_id: &str) -> Vec<&ServerTemplate> {
        self.templates.values()
            .filter(|template| template.category == category_id)
            .collect()
    }

    /// Get template by ID
    pub fn get_template(&self, template_id: &str) -> Option<&ServerTemplate> {
        self.templates.get(template_id)
    }

    /// Get all categories
    pub fn get_categories(&self) -> Vec<&TemplateCategory> {
        self.categories.values().collect()
    }

    /// Get category by ID
    pub fn get_category(&self, category_id: &str) -> Option<&TemplateCategory> {
        self.categories.get(category_id)
    }

    /// Search templates by name or description
    pub fn search_templates(&self, query: &str) -> Vec<&ServerTemplate> {
        let query_lower = query.to_lowercase();
        self.templates.values()
            .filter(|template| {
                template.name.to_lowercase().contains(&query_lower) ||
                template.description.to_lowercase().contains(&query_lower) ||
                template.tags.iter().any(|tag| tag.to_lowercase().contains(&query_lower))
            })
            .collect()
    }

    /// Add custom template
    pub fn add_template(&mut self, template: ServerTemplate) {
        self.templates.insert(template.id.clone(), template);
    }

    /// Remove template
    pub fn remove_template(&mut self, template_id: &str) -> Option<ServerTemplate> {
        self.templates.remove(template_id)
    }
}

impl Default for ServerTemplateManager {
    fn default() -> Self {
        Self::new()
    }
}
