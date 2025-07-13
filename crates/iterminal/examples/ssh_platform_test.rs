use iterminal::ssh_connection::SshConnectionBuilder;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    env_logger::init();

    println!("🔧 跨平台SSH支持测试");
    println!("===================");

    // 获取SSH支持状态信息
    println!("\n📋 SSH支持状态:");
    println!("{}", SshConnectionBuilder::get_ssh_support_info());

    // 测试SSH客户端可用性
    println!("\n🔍 详细检查:");
    
    if SshConnectionBuilder::check_ssh_availability() {
        println!("✅ SSH客户端检查通过");
        
        if let Some(ssh_cmd) = SshConnectionBuilder::get_ssh_command() {
            println!("   使用SSH命令: {}", ssh_cmd);
        }
        
        if let Some(version) = SshConnectionBuilder::get_ssh_version() {
            println!("   SSH版本: {}", version.trim());
        }
    } else {
        println!("❌ SSH客户端不可用");
    }

    // 测试sshpass可用性
    if SshConnectionBuilder::check_sshpass_availability() {
        println!("✅ sshpass工具可用");
    } else {
        println!("⚠️  sshpass工具不可用");
    }

    println!("\n🌍 平台信息:");
    println!("   操作系统: {}", std::env::consts::OS);
    println!("   架构: {}", std::env::consts::ARCH);

    Ok(())
}
