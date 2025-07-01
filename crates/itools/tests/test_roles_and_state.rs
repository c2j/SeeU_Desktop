use itools::{initialize, IToolsState};
use itools::roles::UserRole;
use itools::state::{IToolsView, AuditResult};

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
    let plugin_id = uuid::Uuid::new_v4();
    state.security_context.audit_log.push(itools::state::AuditEntry {
        timestamp: chrono::Utc::now(),
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
        timestamp: chrono::Utc::now(),
        action: "plugin_access".to_string(),
        plugin_id: None,
        user_role: UserRole::BusinessUser,
        result: AuditResult::Denied("Insufficient permissions".to_string()),
    });
    
    assert_eq!(state.security_context.audit_log.len(), 2);
}
