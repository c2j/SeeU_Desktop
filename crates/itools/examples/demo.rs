use itools::{initialize, IToolsState};
use itools::roles::UserRole;
use itools::state::AuditResult;
use itools::plugins::marketplace::{MarketplaceFilters, SortBy};

fn main() {
    // 初始化日志
    env_logger::init();

    println!("🔧 iTools 演示程序");
    println!("==================");

    // 初始化 iTools 状态
    let mut state = initialize();
    state.initialize();

    println!("\n✅ iTools 模块初始化完成");

    // 演示角色系统
    demo_role_system(&mut state);

    // 演示插件市场
    demo_plugin_marketplace(&state);

    // 演示安全审计
    demo_security_audit(&mut state);

    println!("\n🎉 演示完成！");
}

fn demo_role_system(state: &mut IToolsState) {
    println!("\n📋 角色系统演示");
    println!("================");

    let roles = [
        UserRole::BusinessUser,
        UserRole::Developer,
        UserRole::Operations,
        UserRole::Administrator,
    ];

    for role in &roles {
        println!("\n角色: {}", role.display_name());

        let role_def = role.get_role_definition();
        println!("  描述: {}", role_def.description);
        println!("  权限数量: {}", role_def.permissions.len());
        println!("  UI 组件: {} 个", role_def.ui_components.len());

        // 测试一些权限
        let test_permissions = [
            ("read", "/data/report.xlsx"),
            ("write", "/code/main.rs"),
            ("execute", "/system/restart"),
        ];

        for (action, resource) in &test_permissions {
            let has_permission = role.has_permission(action, resource);
            let status = if has_permission { "✅" } else { "❌" };
            println!("  {} {} on {}", status, action, resource);
        }
    }

    // 切换到开发者角色
    state.current_role = UserRole::Developer;
    println!("\n🔄 切换到开发者角色");
}

fn demo_plugin_marketplace(state: &IToolsState) {
    println!("\n🛒 插件市场演示");
    println!("================");

    let marketplace = state.plugin_manager.get_marketplace();

    // 显示分类
    println!("\n📂 可用分类:");
    for category in marketplace.get_categories() {
        println!("  {} {} - {} ({} 个插件)",
                category.icon.as_deref().unwrap_or("📦"),
                category.name,
                category.description,
                category.plugin_count);
    }

    // 显示推荐插件
    println!("\n⭐ 推荐插件:");
    let featured = marketplace.get_featured_plugins();
    for plugin in featured.iter().take(3) {
        println!("  {} {} v{}",
                plugin.plugin.metadata.icon.as_deref().unwrap_or("📦"),
                plugin.plugin.metadata.display_name,
                plugin.plugin.metadata.version);
        println!("    {}", plugin.plugin.metadata.description);
        println!("    评分: {:.1}/5.0, 下载: {}", plugin.rating, plugin.download_count);
    }

    // 搜索插件
    println!("\n🔍 搜索 'filesystem' 插件:");
    let filters = MarketplaceFilters {
        query: "filesystem".to_string(),
        category: None,
        role: Some(UserRole::Developer),
        permission_level: None,
        verified_only: false,
        featured_only: false,
        sort_by: SortBy::Rating,
    };

    let results = marketplace.search_plugins(&filters);
    for plugin in results.iter().take(2) {
        println!("  📦 {} - {}",
                plugin.plugin.metadata.display_name,
                plugin.plugin.metadata.description);
    }
}



fn demo_security_audit(state: &mut IToolsState) {
    println!("\n🔒 安全审计演示");
    println!("================");

    // 记录一些审计事件
    let audit_events = [
        ("角色切换到开发者", AuditResult::Success),
        ("尝试访问系统配置", AuditResult::Denied("权限不足".to_string())),
        ("查看插件市场", AuditResult::Success),
        ("安装新插件", AuditResult::Success),
    ];

    for (action, result) in &audit_events {
        state.log_audit(action.to_string(), None, result.clone());
    }

    println!("📝 已记录 {} 条审计事件", state.security_context.audit_log.len());

    // 显示审计日志
    println!("\n📋 最近的审计事件:");
    for (i, entry) in state.security_context.audit_log.iter().rev().take(3).enumerate() {
        let status = match &entry.result {
            AuditResult::Success => "✅",
            AuditResult::Denied(_) => "❌",
            AuditResult::Error(_) => "⚠️",
        };

        println!("  {}. {} {} - {}",
                i + 1,
                status,
                entry.action,
                entry.timestamp.format("%H:%M:%S"));
    }

    println!("\n🔐 会话信息:");
    println!("  会话 ID: {}", state.security_context.session_id);
    println!("  当前角色: {}", state.current_role.display_name());
    println!("  权限缓存: {} 条", state.security_context.permissions.len());
}
