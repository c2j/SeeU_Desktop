#!/bin/bash

# 测试MCP工具识别日志记录功能
# 这个脚本会启动应用程序并监控日志输出

echo "🚀 启动SeeU Desktop应用程序以测试MCP工具识别日志记录..."
echo "📋 请按照以下步骤测试:"
echo "1. 打开应用程序"
echo "2. 进入AI助手模块"
echo "3. 在MCP下拉框中选择 'simple-tool' 服务器"
echo "4. 观察日志输出，查看工具识别情况"
echo "5. 发送一条消息测试工具调用"
echo ""
echo "🔍 关键日志标识符:"
echo "- '🔧 AI助手 - 选中MCP服务器'"
echo "- '📋 MCP服务器能力统计'"
echo "- '🛠️ 可用工具列表'"
echo "- '✅ 已将 X 个MCP工具转换为OpenAI Function Calling格式'"
echo "- '📤 发送请求到'"
echo "- '🛠️ 发送工具定义给LLM'"
echo ""
echo "⚠️ 如果看到工具数量为0，说明存在问题需要进一步调试"
echo ""

# 设置日志级别为INFO以确保看到详细日志
export RUST_LOG=info

# 启动应用程序
cargo run

echo ""
echo "✅ 测试完成！请检查上述日志输出中的MCP工具识别信息。"
