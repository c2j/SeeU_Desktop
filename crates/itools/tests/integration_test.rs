use itools::initialize;

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
