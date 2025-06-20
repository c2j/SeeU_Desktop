const { execSync } = require('child_process');

// 获取用户输入的所有参数（跳过前两个默认参数）
const userArgs = process.argv.slice(2).join(' ');

// 将参数传递给 npx 命令
try {
  execSync(`npx -y @modelcontextprotocol/server-filesystem ${userArgs}`, { 
    stdio: 'inherit' 
  });
} catch (error) {
  process.exit(1); // 如果命令执行失败，退出码为 1
}
