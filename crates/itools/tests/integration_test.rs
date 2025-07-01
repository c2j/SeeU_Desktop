use itools::{initialize, IToolsState};
use itools::roles::UserRole;
use itools::state::{IToolsView, AuditResult};
use itools::plugins::marketplace::{MarketplaceFilters, SortBy};
use itools::plugins::{PluginStatus, PluginMetadata};
use itools::mcp::protocol::{McpRequest, McpResponse, methods};
use itools::security::permissions::{PermissionCheck, PermissionLevel, PermissionContext};
use uuid::Uuid;
use chrono::Utc;
use serde_json::json;

#[test]
fn test_itools_initialization() {
    let mut state = initialize();
    state.initialize();

    // Test basic state
    assert_eq!(state.current_role.display_name(), "业务用户");
    assert_eq!(state.plugin_manager.get_installed_plugins().len(), 0);
    assert_eq!(state.mcp_client.get_connected_plugins().len(), 0);

    // Test marketplace has preset plugins
    let marketplace = state.plugin_manager.get_marketplace();
    let categories = marketplace.get_categories();
    assert!(!categories.is_empty());

    // Test featured plugins
    let featured = marketplace.get_featured_plugins();
    assert!(!featured.is_empty());
}

#[test]
fn test_user_role_management() {
    let mut state = initialize();

    // Test default role
    assert_eq!(state.current_role, UserRole::BusinessUser);

    // Test role switching
    state.set_role(UserRole::Developer);
    assert_eq!(state.current_role, UserRole::Developer);

    state.set_role(UserRole::Operations);
    assert_eq!(state.current_role, UserRole::Operations);

    state.set_role(UserRole::Administrator);
    assert_eq!(state.current_role, UserRole::Administrator);

    // Test role display names
    assert_eq!(UserRole::BusinessUser.display_name(), "业务用户");
    assert_eq!(UserRole::Developer.display_name(), "开发者");
    assert_eq!(UserRole::Operations.display_name(), "运维人员");
    assert_eq!(UserRole::Administrator.display_name(), "管理员");
}

#[test]
fn test_ui_state_management() {
    let mut state = initialize();

    // Test default UI state
    assert_eq!(state.ui_state.current_view, IToolsView::Dashboard);
    assert!(state.ui_state.search_query.is_empty());
    assert!(state.ui_state.selected_plugin.is_none());
    assert!(!state.ui_state.show_install_dialog);
    assert!(!state.ui_state.show_role_dialog);

    // Test view switching
    state.ui_state.current_view = IToolsView::PluginMarket;
    assert_eq!(state.ui_state.current_view, IToolsView::PluginMarket);

    state.ui_state.current_view = IToolsView::InstalledPlugins;
    assert_eq!(state.ui_state.current_view, IToolsView::InstalledPlugins);

    state.ui_state.current_view = IToolsView::McpSettings;
    assert_eq!(state.ui_state.current_view, IToolsView::McpSettings);
}

#[test]
fn test_security_context() {
    let state = initialize();

    // Test security context initialization
    assert!(!state.security_context.session_id.to_string().is_empty());
    assert!(state.security_context.permissions.is_empty());
    assert!(state.security_context.audit_log.is_empty());
}

#[test]
fn test_audit_logging() {
    let mut state = initialize();

    // Test audit entry creation
    let plugin_id = Uuid::new_v4();
    state.security_context.audit_log.push(itools::state::AuditEntry {
        timestamp: Utc::now(),
        action: "plugin_install".to_string(),
        plugin_id: Some(plugin_id),
        user_role: UserRole::Developer,
        result: AuditResult::Success,
    });

    assert_eq!(state.security_context.audit_log.len(), 1);
    let entry = &state.security_context.audit_log[0];
    assert_eq!(entry.action, "plugin_install");
    assert_eq!(entry.plugin_id, Some(plugin_id));
    assert_eq!(entry.user_role, UserRole::Developer);

    // Test different audit results
    state.security_context.audit_log.push(itools::state::AuditEntry {
        timestamp: Utc::now(),
        action: "plugin_access".to_string(),
        plugin_id: None,
        user_role: UserRole::BusinessUser,
        result: AuditResult::Denied("Insufficient permissions".to_string()),
    });

    assert_eq!(state.security_context.audit_log.len(), 2);
}

#[test]
fn test_mcp_protocol_messages() {
    // Test MCP request creation
    let request = McpRequest::new(
        json!(1),
        methods::INITIALIZE.to_string(),
        Some(json!({"test": "data"}))
    );

    assert_eq!(request.jsonrpc, "2.0");
    assert_eq!(request.id, json!(1));
    assert_eq!(request.method, methods::INITIALIZE);
    assert!(request.params.is_some());

    // Test initialize request
    let client_info = itools::mcp::protocol::ClientInfo {
        name: "iTools Test".to_string(),
        version: "1.0.0".to_string(),
    };

    let init_request = McpRequest::initialize(json!(2), client_info);
    assert_eq!(init_request.method, methods::INITIALIZE);
    assert!(init_request.params.is_some());
}

#[test]
fn test_plugin_metadata() {
    let metadata = PluginMetadata {
        id: Uuid::new_v4(),
        name: "Test Plugin".to_string(),
        version: "1.0.0".to_string(),
        description: "A test plugin".to_string(),
        author: "Test Author".to_string(),
        homepage: Some("https://example.com".to_string()),
        repository: Some("https://github.com/example/plugin".to_string()),
        license: "MIT".to_string(),
        keywords: vec!["test".to_string(), "plugin".to_string()],
        categories: vec!["development".to_string()],
        required_permissions: vec![PermissionLevel::Read],
        supported_roles: vec![UserRole::Developer, UserRole::Administrator],
        min_app_version: "1.0.0".to_string(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    assert_eq!(metadata.name, "Test Plugin");
    assert_eq!(metadata.version, "1.0.0");
    assert_eq!(metadata.author, "Test Author");
    assert_eq!(metadata.license, "MIT");
    assert_eq!(metadata.keywords.len(), 2);
    assert_eq!(metadata.categories.len(), 1);
    assert_eq!(metadata.required_permissions.len(), 1);
    assert_eq!(metadata.supported_roles.len(), 2);
}

#[test]
fn test_plugin_status_variants() {
    // Test all plugin status variants
    assert_eq!(PluginStatus::Installed, PluginStatus::Installed);
    assert_eq!(PluginStatus::Enabled, PluginStatus::Enabled);
    assert_eq!(PluginStatus::Disabled, PluginStatus::Disabled);
    assert_eq!(PluginStatus::Installing, PluginStatus::Installing);
    assert_eq!(PluginStatus::Uninstalling, PluginStatus::Uninstalling);
    assert_eq!(PluginStatus::Error, PluginStatus::Error);

    // Test status inequality
    assert_ne!(PluginStatus::Installed, PluginStatus::Enabled);
    assert_ne!(PluginStatus::Installing, PluginStatus::Error);
}

#[test]
fn test_marketplace_filters() {
    let filters = MarketplaceFilters {
        query: "filesystem".to_string(),
        category: Some("file-system".to_string()),
        role: Some(UserRole::Developer),
        permission_level: Some(PermissionLevel::Write),
        verified_only: true,
        featured_only: false,
        sort_by: SortBy::Rating,
    };

    assert_eq!(filters.query, "filesystem");
    assert_eq!(filters.category, Some("file-system".to_string()));
    assert_eq!(filters.role, Some(UserRole::Developer));
    assert_eq!(filters.permission_level, Some(PermissionLevel::Write));
    assert!(filters.verified_only);
    assert!(!filters.featured_only);
    assert_eq!(filters.sort_by, SortBy::Rating);
}

#[test]
fn test_permission_levels() {
    // Test permission level hierarchy
    assert_eq!(PermissionLevel::None, PermissionLevel::None);
    assert_eq!(PermissionLevel::Read, PermissionLevel::Read);
    assert_eq!(PermissionLevel::Write, PermissionLevel::Write);
    assert_eq!(PermissionLevel::Execute, PermissionLevel::Execute);
    assert_eq!(PermissionLevel::Admin, PermissionLevel::Admin);

    // Test permission level inequality
    assert_ne!(PermissionLevel::None, PermissionLevel::Read);
    assert_ne!(PermissionLevel::Write, PermissionLevel::Execute);
}

#[test]
fn test_permission_context() {
    let context = PermissionContext {
        user_role: UserRole::Developer,
        plugin_id: Some(Uuid::new_v4()),
        resource_path: Some("/test/path".to_string()),
        operation: "read_file".to_string(),
        timestamp: Utc::now(),
        session_id: Uuid::new_v4(),
        ip_address: Some("127.0.0.1".to_string()),
        user_agent: Some("iTools/1.0".to_string()),
    };

    assert_eq!(context.user_role, UserRole::Developer);
    assert!(context.plugin_id.is_some());
    assert_eq!(context.resource_path, Some("/test/path".to_string()));
    assert_eq!(context.operation, "read_file");
    assert!(context.ip_address.is_some());
    assert!(context.user_agent.is_some());
}

#[test]
fn test_mcp_methods_constants() {
    // Test MCP method constants
    assert_eq!(methods::INITIALIZE, "initialize");
    assert_eq!(methods::INITIALIZED, "initialized");
    assert_eq!(methods::PING, "ping");
    assert_eq!(methods::LIST_RESOURCES, "resources/list");
    assert_eq!(methods::READ_RESOURCE, "resources/read");
    assert_eq!(methods::LIST_TOOLS, "tools/list");
    assert_eq!(methods::CALL_TOOL, "tools/call");
    assert_eq!(methods::LIST_PROMPTS, "prompts/list");
    assert_eq!(methods::GET_PROMPT, "prompts/get");
    assert_eq!(methods::COMPLETE, "completion/complete");
    assert_eq!(methods::CANCEL, "$/cancelRequest");
    assert_eq!(methods::PROGRESS, "$/progress");
    assert_eq!(methods::LOG, "$/log");
}

#[test]
fn test_sort_by_variants() {
    // Test sort by options
    assert_eq!(SortBy::Name, SortBy::Name);
    assert_eq!(SortBy::Rating, SortBy::Rating);
    assert_eq!(SortBy::Downloads, SortBy::Downloads);
    assert_eq!(SortBy::Updated, SortBy::Updated);
    assert_eq!(SortBy::Created, SortBy::Created);

    // Test sort by inequality
    assert_ne!(SortBy::Name, SortBy::Rating);
    assert_ne!(SortBy::Downloads, SortBy::Updated);
}

#[test]
fn test_role_permissions() {
    use itools::roles::UserRole;

    let business_user = UserRole::BusinessUser;
    let developer = UserRole::Developer;
    let admin = UserRole::Administrator;

    // Test role display names
    assert_eq!(business_user.display_name(), "业务用户");
    assert_eq!(developer.display_name(), "开发者");
    assert_eq!(admin.display_name(), "管理员");

    // Test permissions
    assert!(admin.has_permission("*", "*"));
    assert!(developer.has_permission("read", "/code/test.rs"));
    assert!(!business_user.has_permission("write", "/system/config"));
}

#[test]
fn test_plugin_marketplace() {
    use itools::plugins::marketplace::{MarketplaceFilters, SortBy};
    use itools::roles::UserRole;

    let mut state = initialize();
    state.initialize();

    let marketplace = state.plugin_manager.get_marketplace();

    // Test search with filters
    let filters = MarketplaceFilters {
        query: "filesystem".to_string(),
        category: Some("file-system".to_string()),
        role: Some(UserRole::BusinessUser),
        permission_level: None,
        verified_only: true,
        featured_only: false,
        sort_by: SortBy::Rating,
    };

    let results = marketplace.search_plugins(&filters);
    assert!(!results.is_empty());

    // Test categories
    let categories = marketplace.get_categories();
    let file_system_category = categories.iter()
        .find(|c| c.id == "file-system")
        .expect("Should have file-system category");

    assert_eq!(file_system_category.name, "文件系统");
}



#[test]
fn test_audit_logging() {
    let mut state = initialize();
    state.initialize();

    // Test audit logging
    use itools::state::AuditResult;

    state.log_audit(
        "Test action".to_string(),
        None,
        AuditResult::Success,
    );

    assert_eq!(state.security_context.audit_log.len(), 1);
    assert_eq!(state.security_context.audit_log[0].action, "Test action");
}
