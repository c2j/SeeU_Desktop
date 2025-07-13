use iterminal::remote_server::{AuthMethod, RemoteServer};
use iterminal::remote_server_ui::{RemoteServerUI, RemoteServerAction};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    env_logger::init();

    println!("🔧 调试连接测试功能");
    println!("====================");

    // 创建远程服务器UI
    let mut remote_ui = RemoteServerUI::new()?;
    println!("✅ 远程服务器UI初始化成功");

    // 创建测试服务器
    let test_server = RemoteServer::new(
        "测试服务器".to_string(),
        "github.com".to_string(),
        "git".to_string(),
        AuthMethod::Agent,
    );

    let server_id = test_server.id;
    let server_name = test_server.name.clone();

    // 添加服务器
    match remote_ui.manager.add_server(test_server) {
        Ok(_) => println!("✅ 添加测试服务器成功: {}", server_name),
        Err(e) => {
            println!("❌ 添加测试服务器失败: {}", e);
            return Ok(());
        }
    }

    // 模拟用户点击测试连接按钮
    println!("\n🔍 模拟连接测试...");
    println!("服务器ID: {}", server_id);
    println!("服务器名称: {}", server_name);

    // 直接调用连接测试方法
    println!("\n1. 直接调用 start_connection_test...");
    remote_ui.start_connection_test(server_id);

    // 检查测试状态
    println!("\n2. 检查测试状态...");
    if let Some(status) = remote_ui.get_connection_test_status(server_id) {
        println!("   测试状态: {:?}", status);
    } else {
        println!("   没有找到测试状态");
    }

    // 模拟处理RemoteServerAction::TestConnection
    println!("\n3. 模拟处理 TestConnection 动作...");
    let action = RemoteServerAction::TestConnection(server_id);
    println!("   创建动作: {:?}", action);

    // 再次调用连接测试
    match action {
        RemoteServerAction::TestConnection(id) => {
            println!("   处理 TestConnection 动作，ID: {}", id);
            remote_ui.start_connection_test(id);
        }
        _ => {}
    }

    // 最终状态检查
    println!("\n4. 最终状态检查...");
    if let Some(status) = remote_ui.get_connection_test_status(server_id) {
        println!("   最终测试状态: {:?}", status);
    } else {
        println!("   没有找到最终测试状态");
    }

    // 显示所有服务器
    println!("\n5. 服务器列表:");
    for server in remote_ui.manager.list_servers() {
        println!("   - {} ({})", server.name, server.get_connection_string());
    }

    println!("\n🎉 调试测试完成！");

    Ok(())
}
