use uuid::Uuid;
use chrono::Utc;
use serde_json::json;

use super::plugin::{
    Plugin, PluginMetadata, PluginManifest, PluginCapabilities, PluginPermission,
    PermissionType, ResourceDefinition, ToolDefinition, PromptDefinition, PromptArgument
};
use super::marketplace::MarketplacePlugin;
use crate::roles::UserRole;
use crate::state::PermissionLevel;

/// Plugin presets for marketplace
pub struct PluginPresets;

impl PluginPresets {
    /// Create Filesystem MCP Server plugin
    pub fn create_filesystem_plugin() -> MarketplacePlugin {
        let metadata = PluginMetadata {
            name: "filesystem-mcp".to_string(),
            display_name: "文件系统 MCP 服务器".to_string(),
            description: "提供受限访问桌面和下载目录的文件读写功能，支持安全的文件操作".to_string(),
            version: "1.0.0".to_string(),
            author: "SeeU Team".to_string(),
            homepage: Some("https://github.com/seeu/filesystem-mcp".to_string()),
            repository: Some("https://github.com/seeu/filesystem-mcp".to_string()),
            license: "MIT".to_string(),
            keywords: vec![
                "filesystem".to_string(),
                "files".to_string(),
                "mcp".to_string(),
                "desktop".to_string(),
            ],
            categories: vec!["file-system".to_string()],
            target_roles: vec![
                UserRole::BusinessUser,
                UserRole::Developer,
                UserRole::Operations,
                UserRole::Administrator,
            ],
            icon: Some("📁".to_string()),
            screenshots: vec![],
        };

        let manifest = PluginManifest {
            schema_version: "1.0".to_string(),
            mcp_version: "2025-03-26".to_string(),
            capabilities: PluginCapabilities {
                provides_resources: true,
                provides_tools: true,
                provides_prompts: false,
                supports_sampling: false,
                supports_notifications: true,
                supports_progress: true,
            },
            permissions: vec![
                PluginPermission {
                    permission_type: PermissionType::FileSystem,
                    resource: "~/Desktop/*".to_string(),
                    description: "读写桌面目录文件".to_string(),
                    required: true,
                    level: PermissionLevel::Medium,
                },
                PluginPermission {
                    permission_type: PermissionType::FileSystem,
                    resource: "~/Downloads/*".to_string(),
                    description: "读写下载目录文件".to_string(),
                    required: true,
                    level: PermissionLevel::Medium,
                },
            ],
            dependencies: vec![],
            resources: vec![
                ResourceDefinition {
                    uri: "file://desktop".to_string(),
                    name: "桌面文件".to_string(),
                    description: "桌面目录中的文件和文件夹".to_string(),
                    mime_type: Some("inode/directory".to_string()),
                },
                ResourceDefinition {
                    uri: "file://downloads".to_string(),
                    name: "下载文件".to_string(),
                    description: "下载目录中的文件".to_string(),
                    mime_type: Some("inode/directory".to_string()),
                },
            ],
            tools: vec![
                ToolDefinition {
                    name: "read_file".to_string(),
                    description: "读取文件内容".to_string(),
                    input_schema: json!({
                        "type": "object",
                        "properties": {
                            "path": {
                                "type": "string",
                                "description": "文件路径"
                            }
                        },
                        "required": ["path"]
                    }),
                    output_schema: Some(json!({
                        "type": "object",
                        "properties": {
                            "content": {
                                "type": "string",
                                "description": "文件内容"
                            },
                            "encoding": {
                                "type": "string",
                                "description": "文件编码"
                            }
                        }
                    })),
                },
                ToolDefinition {
                    name: "write_file".to_string(),
                    description: "写入文件内容".to_string(),
                    input_schema: json!({
                        "type": "object",
                        "properties": {
                            "path": {
                                "type": "string",
                                "description": "文件路径"
                            },
                            "content": {
                                "type": "string",
                                "description": "要写入的内容"
                            }
                        },
                        "required": ["path", "content"]
                    }),
                    output_schema: Some(json!({
                        "type": "object",
                        "properties": {
                            "success": {
                                "type": "boolean",
                                "description": "是否成功写入"
                            }
                        }
                    })),
                },
                ToolDefinition {
                    name: "list_directory".to_string(),
                    description: "列出目录内容".to_string(),
                    input_schema: json!({
                        "type": "object",
                        "properties": {
                            "path": {
                                "type": "string",
                                "description": "目录路径"
                            }
                        },
                        "required": ["path"]
                    }),
                    output_schema: Some(json!({
                        "type": "object",
                        "properties": {
                            "entries": {
                                "type": "array",
                                "items": {
                                    "type": "object",
                                    "properties": {
                                        "name": {"type": "string"},
                                        "type": {"type": "string"},
                                        "size": {"type": "number"},
                                        "modified": {"type": "string"}
                                    }
                                }
                            }
                        }
                    })),
                },
            ],
            prompts: vec![],
            configuration: None,
        };

        let plugin = Plugin::new(metadata, manifest);

        MarketplacePlugin {
            plugin,
            download_url: "https://marketplace.seeu.app/plugins/filesystem-mcp/download".to_string(),
            download_count: 15420,
            rating: 4.8,
            review_count: 234,
            last_updated: Utc::now(),
            verified: true,
            featured: true,
            size_mb: 2.1,
            screenshots: vec![],
            changelog: "Initial release with secure file operations".to_string(),
        }
    }

    /// Create Git Integration plugin
    pub fn create_git_plugin() -> MarketplacePlugin {
        let metadata = PluginMetadata {
            name: "git-integration".to_string(),
            display_name: "Git 集成".to_string(),
            description: "代码仓库管理、分支操作、提交历史可视化工具".to_string(),
            version: "1.2.0".to_string(),
            author: "SeeU Team".to_string(),
            homepage: Some("https://github.com/seeu/git-integration".to_string()),
            repository: Some("https://github.com/seeu/git-integration".to_string()),
            license: "MIT".to_string(),
            keywords: vec![
                "git".to_string(),
                "version-control".to_string(),
                "development".to_string(),
                "repository".to_string(),
            ],
            categories: vec!["development".to_string()],
            target_roles: vec![UserRole::Developer, UserRole::Administrator],
            icon: Some("🔀".to_string()),
            screenshots: vec![],
        };

        let manifest = PluginManifest {
            schema_version: "1.0".to_string(),
            mcp_version: "2025-03-26".to_string(),
            capabilities: PluginCapabilities {
                provides_resources: true,
                provides_tools: true,
                provides_prompts: true,
                supports_sampling: false,
                supports_notifications: true,
                supports_progress: true,
            },
            permissions: vec![
                PluginPermission {
                    permission_type: PermissionType::FileSystem,
                    resource: "/code/*".to_string(),
                    description: "访问代码仓库目录".to_string(),
                    required: true,
                    level: PermissionLevel::High,
                },
                PluginPermission {
                    permission_type: PermissionType::ProcessExecution,
                    resource: "git".to_string(),
                    description: "执行 Git 命令".to_string(),
                    required: true,
                    level: PermissionLevel::High,
                },
            ],
            dependencies: vec![],
            resources: vec![
                ResourceDefinition {
                    uri: "git://repositories".to_string(),
                    name: "Git 仓库".to_string(),
                    description: "本地 Git 仓库信息".to_string(),
                    mime_type: Some("application/git".to_string()),
                },
            ],
            tools: vec![
                ToolDefinition {
                    name: "git_status".to_string(),
                    description: "获取仓库状态".to_string(),
                    input_schema: json!({
                        "type": "object",
                        "properties": {
                            "repository_path": {
                                "type": "string",
                                "description": "仓库路径"
                            }
                        },
                        "required": ["repository_path"]
                    }),
                    output_schema: None,
                },
                ToolDefinition {
                    name: "git_commit".to_string(),
                    description: "提交更改".to_string(),
                    input_schema: json!({
                        "type": "object",
                        "properties": {
                            "repository_path": {"type": "string"},
                            "message": {"type": "string"},
                            "files": {
                                "type": "array",
                                "items": {"type": "string"}
                            }
                        },
                        "required": ["repository_path", "message"]
                    }),
                    output_schema: None,
                },
            ],
            prompts: vec![
                PromptDefinition {
                    name: "commit_message".to_string(),
                    description: "生成提交信息".to_string(),
                    arguments: vec![
                        PromptArgument {
                            name: "changes".to_string(),
                            description: "代码更改描述".to_string(),
                            required: true,
                            argument_type: "string".to_string(),
                        },
                    ],
                },
            ],
            configuration: None,
        };

        let plugin = Plugin::new(metadata, manifest);

        MarketplacePlugin {
            plugin,
            download_url: "https://marketplace.seeu.app/plugins/git-integration/download".to_string(),
            download_count: 8932,
            rating: 4.6,
            review_count: 156,
            last_updated: Utc::now(),
            verified: true,
            featured: true,
            size_mb: 3.2,
            screenshots: vec![],
            changelog: "Added branch management and merge conflict resolution".to_string(),
        }
    }

    /// Create BI Connector plugin
    pub fn create_bi_plugin() -> MarketplacePlugin {
        let metadata = PluginMetadata {
            name: "bi-connector".to_string(),
            display_name: "BI 连接器".to_string(),
            description: "连接 Tableau/Power BI，自动生成数据看板和可视化报表".to_string(),
            version: "2.1.0".to_string(),
            author: "SeeU Team".to_string(),
            homepage: Some("https://github.com/seeu/bi-connector".to_string()),
            repository: Some("https://github.com/seeu/bi-connector".to_string()),
            license: "MIT".to_string(),
            keywords: vec![
                "bi".to_string(),
                "tableau".to_string(),
                "powerbi".to_string(),
                "dashboard".to_string(),
                "visualization".to_string(),
            ],
            categories: vec!["data-analysis".to_string()],
            target_roles: vec![UserRole::BusinessUser, UserRole::Administrator],
            icon: Some("📊".to_string()),
            screenshots: vec![],
        };

        let manifest = PluginManifest {
            schema_version: "1.0".to_string(),
            mcp_version: "2025-03-26".to_string(),
            capabilities: PluginCapabilities {
                provides_resources: true,
                provides_tools: true,
                provides_prompts: true,
                supports_sampling: true,
                supports_notifications: true,
                supports_progress: true,
            },
            permissions: vec![
                PluginPermission {
                    permission_type: PermissionType::Network,
                    resource: "tableau.com".to_string(),
                    description: "连接 Tableau 服务".to_string(),
                    required: false,
                    level: PermissionLevel::Medium,
                },
                PluginPermission {
                    permission_type: PermissionType::Network,
                    resource: "powerbi.microsoft.com".to_string(),
                    description: "连接 Power BI 服务".to_string(),
                    required: false,
                    level: PermissionLevel::Medium,
                },
                PluginPermission {
                    permission_type: PermissionType::UserData,
                    resource: "/data/business/*".to_string(),
                    description: "访问业务数据".to_string(),
                    required: true,
                    level: PermissionLevel::High,
                },
            ],
            dependencies: vec![],
            resources: vec![
                ResourceDefinition {
                    uri: "bi://dashboards".to_string(),
                    name: "BI 看板".to_string(),
                    description: "已创建的 BI 看板和报表".to_string(),
                    mime_type: Some("application/bi-dashboard".to_string()),
                },
            ],
            tools: vec![
                ToolDefinition {
                    name: "create_dashboard".to_string(),
                    description: "创建数据看板".to_string(),
                    input_schema: json!({
                        "type": "object",
                        "properties": {
                            "data_source": {"type": "string"},
                            "chart_type": {"type": "string"},
                            "title": {"type": "string"},
                            "filters": {
                                "type": "array",
                                "items": {"type": "string"}
                            }
                        },
                        "required": ["data_source", "chart_type", "title"]
                    }),
                    output_schema: None,
                },
                ToolDefinition {
                    name: "export_report".to_string(),
                    description: "导出报表".to_string(),
                    input_schema: json!({
                        "type": "object",
                        "properties": {
                            "dashboard_id": {"type": "string"},
                            "format": {"type": "string", "enum": ["pdf", "excel", "png"]},
                            "filters": {"type": "object"}
                        },
                        "required": ["dashboard_id", "format"]
                    }),
                    output_schema: None,
                },
            ],
            prompts: vec![
                PromptDefinition {
                    name: "analyze_data".to_string(),
                    description: "分析数据趋势".to_string(),
                    arguments: vec![
                        PromptArgument {
                            name: "dataset".to_string(),
                            description: "数据集名称".to_string(),
                            required: true,
                            argument_type: "string".to_string(),
                        },
                        PromptArgument {
                            name: "metrics".to_string(),
                            description: "关注的指标".to_string(),
                            required: false,
                            argument_type: "array".to_string(),
                        },
                    ],
                },
            ],
            configuration: None,
        };

        let plugin = Plugin::new(metadata, manifest);

        MarketplacePlugin {
            plugin,
            download_url: "https://marketplace.seeu.app/plugins/bi-connector/download".to_string(),
            download_count: 5678,
            rating: 4.4,
            review_count: 89,
            last_updated: Utc::now(),
            verified: true,
            featured: true,
            size_mb: 5.7,
            screenshots: vec![],
            changelog: "Enhanced chart customization and added new data sources".to_string(),
        }
    }

    /// Create System Monitor plugin
    pub fn create_system_monitor_plugin() -> MarketplacePlugin {
        let metadata = PluginMetadata {
            name: "system-monitor".to_string(),
            display_name: "系统监控".to_string(),
            description: "实时 CPU/内存监控、异常进程告警和系统性能分析".to_string(),
            version: "1.5.2".to_string(),
            author: "SeeU Team".to_string(),
            homepage: Some("https://github.com/seeu/system-monitor".to_string()),
            repository: Some("https://github.com/seeu/system-monitor".to_string()),
            license: "MIT".to_string(),
            keywords: vec![
                "monitoring".to_string(),
                "system".to_string(),
                "performance".to_string(),
                "cpu".to_string(),
                "memory".to_string(),
            ],
            categories: vec!["monitoring".to_string()],
            target_roles: vec![UserRole::Operations, UserRole::Administrator],
            icon: Some("📈".to_string()),
            screenshots: vec![],
        };

        let manifest = PluginManifest {
            schema_version: "1.0".to_string(),
            mcp_version: "2025-03-26".to_string(),
            capabilities: PluginCapabilities {
                provides_resources: true,
                provides_tools: true,
                provides_prompts: false,
                supports_sampling: false,
                supports_notifications: true,
                supports_progress: false,
            },
            permissions: vec![
                PluginPermission {
                    permission_type: PermissionType::SystemInfo,
                    resource: "/proc/*".to_string(),
                    description: "读取系统进程信息".to_string(),
                    required: true,
                    level: PermissionLevel::Medium,
                },
                PluginPermission {
                    permission_type: PermissionType::SystemInfo,
                    resource: "/sys/*".to_string(),
                    description: "读取系统硬件信息".to_string(),
                    required: true,
                    level: PermissionLevel::Medium,
                },
            ],
            dependencies: vec![],
            resources: vec![
                ResourceDefinition {
                    uri: "system://metrics".to_string(),
                    name: "系统指标".to_string(),
                    description: "实时系统性能指标".to_string(),
                    mime_type: Some("application/metrics".to_string()),
                },
            ],
            tools: vec![
                ToolDefinition {
                    name: "get_system_info".to_string(),
                    description: "获取系统信息".to_string(),
                    input_schema: json!({
                        "type": "object",
                        "properties": {
                            "include_processes": {"type": "boolean", "default": false}
                        }
                    }),
                    output_schema: None,
                },
                ToolDefinition {
                    name: "monitor_process".to_string(),
                    description: "监控特定进程".to_string(),
                    input_schema: json!({
                        "type": "object",
                        "properties": {
                            "process_name": {"type": "string"},
                            "alert_threshold": {"type": "number"}
                        },
                        "required": ["process_name"]
                    }),
                    output_schema: None,
                },
            ],
            prompts: vec![],
            configuration: None,
        };

        let plugin = Plugin::new(metadata, manifest);

        MarketplacePlugin {
            plugin,
            download_url: "https://marketplace.seeu.app/plugins/system-monitor/download".to_string(),
            download_count: 12340,
            rating: 4.7,
            review_count: 203,
            last_updated: Utc::now(),
            verified: true,
            featured: true,
            size_mb: 1.8,
            screenshots: vec![],
            changelog: "Improved performance monitoring and added disk I/O metrics".to_string(),
        }
    }
}
