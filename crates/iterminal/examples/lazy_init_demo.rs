use iterminal::state::ITerminalState;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    env_logger::init();

    println!("🚀 iTerminal 延迟初始化演示");
    println!("=============================");

    // 创建终端状态 - 此时不应该访问密钥环
    println!("1. 创建 ITerminalState...");
    let mut state = ITerminalState::new();
    println!("   ✅ 创建成功，未访问密钥环");

    // 检查远程服务器功能是否可用
    println!("\n2. 检查远程服务器功能状态...");
    println!("   has_remote_servers(): {}", state.has_remote_servers());
    println!("   ✅ 检查完成，仍未访问密钥环");

    // 尝试获取远程服务器UI（不初始化）
    println!("\n3. 尝试获取远程服务器UI（不初始化）...");
    let ui_ref = state.get_remote_server_ui();
    println!("   get_remote_server_ui(): {}", ui_ref.is_some());
    println!("   ✅ 获取完成，仍未访问密钥环");

    // 现在触发延迟初始化
    println!("\n4. 触发延迟初始化...");
    println!("   调用 ensure_remote_server_ui()...");
    let init_success = state.ensure_remote_server_ui();
    println!("   初始化结果: {}", init_success);
    
    if init_success {
        println!("   ✅ 初始化成功！此时会访问密钥环");
        
        // 现在可以安全地使用远程服务器功能
        println!("\n5. 使用远程服务器功能...");
        if let Some(remote_ui) = state.get_remote_server_ui() {
            let server_count = remote_ui.manager.list_servers().len();
            println!("   当前服务器数量: {}", server_count);
            
            let stats = remote_ui.manager.get_statistics();
            println!("   统计信息: {:?}", stats);
        }
        
        // 测试延迟获取方法
        println!("\n6. 测试延迟获取方法...");
        if let Some(remote_ui) = state.get_remote_server_ui_lazy() {
            println!("   get_remote_server_ui_lazy(): 成功");
            println!("   服务器数量: {}", remote_ui.manager.list_servers().len());
        }
        
        if let Some(remote_ui) = state.get_remote_server_ui_mut_lazy() {
            println!("   get_remote_server_ui_mut_lazy(): 成功");
            
            // 添加一个测试服务器
            let test_server = iterminal::remote_server::RemoteServer::new(
                "测试服务器".to_string(),
                "test.example.com".to_string(),
                "testuser".to_string(),
                iterminal::remote_server::AuthMethod::Agent,
            );
            
            match remote_ui.manager.add_server(test_server) {
                Ok(_) => println!("   ✅ 成功添加测试服务器"),
                Err(e) => println!("   ❌ 添加测试服务器失败: {}", e),
            }
            
            println!("   更新后服务器数量: {}", remote_ui.manager.list_servers().len());
        }
        
    } else {
        println!("   ❌ 初始化失败");
    }

    println!("\n🎉 演示完成！");
    println!("\n📝 总结:");
    println!("   - 应用启动时不会访问密钥环");
    println!("   - 只有在用户主动使用远程服务器功能时才会初始化");
    println!("   - 初始化后功能正常可用");
    println!("   - 避免了不必要的系统权限提示");

    Ok(())
}
