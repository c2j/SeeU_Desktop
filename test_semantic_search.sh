#!/bin/bash

# 语义搜索功能测试脚本
# 用于验证语义搜索的基础功能

set -e

echo "🧪 语义搜索功能测试脚本"
echo "=========================="

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 测试结果统计
TESTS_PASSED=0
TESTS_FAILED=0
TESTS_TOTAL=0

# 测试函数
run_test() {
    local test_name="$1"
    local test_command="$2"
    
    echo -e "\n${BLUE}🔍 测试: $test_name${NC}"
    TESTS_TOTAL=$((TESTS_TOTAL + 1))
    
    if eval "$test_command"; then
        echo -e "${GREEN}✅ 通过: $test_name${NC}"
        TESTS_PASSED=$((TESTS_PASSED + 1))
        return 0
    else
        echo -e "${RED}❌ 失败: $test_name${NC}"
        TESTS_FAILED=$((TESTS_FAILED + 1))
        return 1
    fi
}

# 检查HelixDB安装
check_helix_installation() {
    if command -v helix &> /dev/null; then
        echo "HelixDB版本: $(helix --version 2>/dev/null || echo '未知')"
        return 0
    else
        echo "HelixDB未安装或不在PATH中"
        return 1
    fi
}

# 检查端口占用
check_port_usage() {
    if lsof -i :6969 &> /dev/null; then
        echo "端口6969已被占用（可能是HelixDB进程）"
        return 0
    else
        echo "端口6969未被占用"
        return 1
    fi
}

# 检查配置目录
check_config_directory() {
    local config_dir=""
    
    # 根据操作系统确定配置目录
    if [[ "$OSTYPE" == "darwin"* ]]; then
        config_dir="$HOME/Library/Application Support/SeeU_Desktop/semantic_search"
    elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
        config_dir="$HOME/.local/share/SeeU_Desktop/semantic_search"
    else
        echo "不支持的操作系统: $OSTYPE"
        return 1
    fi
    
    if [[ -d "$config_dir" ]]; then
        echo "配置目录存在: $config_dir"
        ls -la "$config_dir" 2>/dev/null || true
        return 0
    else
        echo "配置目录不存在: $config_dir"
        return 1
    fi
}

# 检查配置文件
check_config_file() {
    local config_dir=""
    
    # 根据操作系统确定配置目录
    if [[ "$OSTYPE" == "darwin"* ]]; then
        config_dir="$HOME/Library/Application Support/SeeU_Desktop/semantic_search"
    elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
        config_dir="$HOME/.local/share/SeeU_Desktop/semantic_search"
    else
        return 1
    fi
    
    local config_file="$config_dir/config.toml"
    
    if [[ -f "$config_file" ]]; then
        echo "配置文件存在: $config_file"
        echo "配置内容:"
        cat "$config_file" 2>/dev/null || echo "无法读取配置文件"
        return 0
    else
        echo "配置文件不存在: $config_file"
        return 1
    fi
}

# 创建测试配置
create_test_config() {
    local config_dir=""
    
    # 根据操作系统确定配置目录
    if [[ "$OSTYPE" == "darwin"* ]]; then
        config_dir="$HOME/Library/Application Support/SeeU_Desktop/semantic_search"
    elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
        config_dir="$HOME/.local/share/SeeU_Desktop/semantic_search"
    else
        echo "不支持的操作系统: $OSTYPE"
        return 1
    fi
    
    # 创建配置目录
    mkdir -p "$config_dir"
    
    # 创建测试配置文件
    local config_file="$config_dir/config.toml"
    
    cat > "$config_file" << 'EOF'
# 语义搜索测试配置
enabled = true

[helix_config]
database_path = ""
port = 6969
connection_timeout = 30
query_timeout = 30

[embedding_config]
provider = "local"  # 使用本地模型避免API密钥问题
api_key = ""
model = "text-embedding"
batch_size = 5
cache_size = 100

[search_weights]
semantic_weight = 0.7
keyword_weight = 0.3
EOF
    
    echo "已创建测试配置文件: $config_file"
    return 0
}

# 检查应用编译
check_app_compilation() {
    echo "检查应用编译状态..."
    if cargo check --quiet; then
        echo "应用编译成功"
        return 0
    else
        echo "应用编译失败"
        return 1
    fi
}

# 测试应用启动（短时间）
test_app_startup() {
    echo "测试应用启动（10秒超时）..."
    
    # 启动应用并在后台运行
    timeout 10s cargo run &> /tmp/seeu_test.log &
    local app_pid=$!
    
    # 等待几秒让应用启动
    sleep 5
    
    # 检查进程是否还在运行
    if kill -0 $app_pid 2>/dev/null; then
        echo "应用成功启动"
        # 终止应用
        kill $app_pid 2>/dev/null || true
        wait $app_pid 2>/dev/null || true
        
        # 检查日志中的语义搜索相关信息
        if grep -q "语义搜索" /tmp/seeu_test.log; then
            echo "发现语义搜索相关日志"
            grep "语义搜索" /tmp/seeu_test.log | head -5
        fi
        
        return 0
    else
        echo "应用启动失败"
        echo "错误日志:"
        cat /tmp/seeu_test.log | tail -10
        return 1
    fi
}

# 主测试流程
main() {
    echo -e "${YELLOW}开始语义搜索功能测试...${NC}"
    
    # 基础环境检查
    echo -e "\n${BLUE}=== 基础环境检查 ===${NC}"
    run_test "HelixDB安装检查" "check_helix_installation"
    run_test "应用编译检查" "check_app_compilation"
    
    # 配置检查
    echo -e "\n${BLUE}=== 配置检查 ===${NC}"
    run_test "配置目录检查" "check_config_directory"
    run_test "配置文件检查" "check_config_file"
    
    # 如果配置文件不存在，创建测试配置
    if [[ $TESTS_FAILED -gt 0 ]]; then
        echo -e "\n${YELLOW}创建测试配置...${NC}"
        if create_test_config; then
            echo -e "${GREEN}✅ 测试配置创建成功${NC}"
        else
            echo -e "${RED}❌ 测试配置创建失败${NC}"
        fi
    fi
    
    # 应用测试
    echo -e "\n${BLUE}=== 应用功能测试 ===${NC}"
    run_test "应用启动测试" "test_app_startup"
    
    # 进程检查（在应用启动测试后）
    echo -e "\n${BLUE}=== 进程检查 ===${NC}"
    run_test "HelixDB进程检查" "check_port_usage"
    
    # 测试结果汇总
    echo -e "\n${BLUE}=== 测试结果汇总 ===${NC}"
    echo "总测试数: $TESTS_TOTAL"
    echo -e "通过: ${GREEN}$TESTS_PASSED${NC}"
    echo -e "失败: ${RED}$TESTS_FAILED${NC}"
    
    if [[ $TESTS_FAILED -eq 0 ]]; then
        echo -e "\n${GREEN}🎉 所有测试通过！语义搜索基础功能正常。${NC}"
        return 0
    else
        echo -e "\n${YELLOW}⚠️ 部分测试失败，请检查上述错误信息。${NC}"
        echo -e "\n${BLUE}💡 故障排除建议：${NC}"
        echo "1. 确保HelixDB已正确安装"
        echo "2. 检查网络连接（如使用在线API）"
        echo "3. 查看应用日志了解详细错误信息"
        echo "4. 参考 semantic_search_test_guide.md 获取详细测试指南"
        return 1
    fi
}

# 清理函数
cleanup() {
    echo -e "\n${YELLOW}清理测试环境...${NC}"
    # 清理临时文件
    rm -f /tmp/seeu_test.log
    
    # 终止可能残留的进程
    pkill -f "cargo run" 2>/dev/null || true
    pkill -f "seeu_desktop" 2>/dev/null || true
}

# 设置清理陷阱
trap cleanup EXIT

# 运行主测试
main "$@"
