#!/usr/bin/env python3
"""
语义搜索向量化API测试工具
用于验证向量化服务是否正常工作
"""

import requests
import json
import os
import sys
from typing import List, Dict, Any

def test_openai_embedding(api_key: str, model: str = "text-embedding-3-small") -> bool:
    """测试OpenAI向量化API"""
    print(f"🔍 测试OpenAI向量化API (模型: {model})")
    
    url = "https://api.openai.com/v1/embeddings"
    headers = {
        "Authorization": f"Bearer {api_key}",
        "Content-Type": "application/json"
    }
    
    data = {
        "input": "这是一个测试文本，用于验证向量化功能。",
        "model": model
    }
    
    try:
        response = requests.post(url, headers=headers, json=data, timeout=30)
        
        if response.status_code == 200:
            result = response.json()
            embedding = result['data'][0]['embedding']
            print(f"✅ OpenAI API测试成功")
            print(f"   向量维度: {len(embedding)}")
            print(f"   使用量: {result.get('usage', {})}")
            return True
        else:
            print(f"❌ OpenAI API测试失败: {response.status_code}")
            print(f"   错误信息: {response.text}")
            return False
            
    except Exception as e:
        print(f"❌ OpenAI API测试异常: {e}")
        return False

def test_local_embedding(api_base: str = "http://localhost:11434/v1", 
                        model: str = "nomic-embed-text") -> bool:
    """测试本地向量化API（如Ollama）"""
    print(f"🔍 测试本地向量化API (地址: {api_base}, 模型: {model})")
    
    # 首先检查服务是否可用
    try:
        health_url = api_base.replace("/v1", "/api/tags")
        health_response = requests.get(health_url, timeout=5)
        if health_response.status_code != 200:
            print(f"❌ 本地服务不可用: {health_response.status_code}")
            return False
    except Exception as e:
        print(f"❌ 无法连接到本地服务: {e}")
        return False
    
    # 测试向量化
    url = f"{api_base}/embeddings"
    headers = {"Content-Type": "application/json"}
    
    data = {
        "input": "这是一个测试文本，用于验证向量化功能。",
        "model": model
    }
    
    try:
        response = requests.post(url, headers=headers, json=data, timeout=30)
        
        if response.status_code == 200:
            result = response.json()
            embedding = result['data'][0]['embedding']
            print(f"✅ 本地API测试成功")
            print(f"   向量维度: {len(embedding)}")
            return True
        else:
            print(f"❌ 本地API测试失败: {response.status_code}")
            print(f"   错误信息: {response.text}")
            return False
            
    except Exception as e:
        print(f"❌ 本地API测试异常: {e}")
        return False

def test_helix_connection(port: int = 6969) -> bool:
    """测试HelixDB连接"""
    print(f"🔍 测试HelixDB连接 (端口: {port})")
    
    try:
        # 尝试连接HelixDB健康检查端点
        url = f"http://localhost:{port}/health"
        response = requests.get(url, timeout=5)
        
        if response.status_code == 200:
            print("✅ HelixDB连接成功")
            return True
        else:
            print(f"❌ HelixDB连接失败: {response.status_code}")
            return False
            
    except requests.exceptions.ConnectionError:
        print("❌ HelixDB服务未运行或端口不可达")
        return False
    except Exception as e:
        print(f"❌ HelixDB连接异常: {e}")
        return False

def check_environment() -> Dict[str, Any]:
    """检查环境配置"""
    print("🔍 检查环境配置")
    
    env_info = {
        "python_version": sys.version,
        "openai_api_key": bool(os.getenv("OPENAI_API_KEY")),
        "requests_available": True
    }
    
    try:
        import requests
        print(f"✅ Python版本: {sys.version.split()[0]}")
        print(f"✅ requests库: {requests.__version__}")
    except ImportError:
        print("❌ requests库未安装")
        env_info["requests_available"] = False
    
    if os.getenv("OPENAI_API_KEY"):
        print("✅ 发现OPENAI_API_KEY环境变量")
    else:
        print("⚠️ 未设置OPENAI_API_KEY环境变量")
    
    return env_info

def main():
    """主测试函数"""
    print("🧪 语义搜索向量化API测试工具")
    print("=" * 40)
    
    # 检查环境
    env_info = check_environment()
    if not env_info["requests_available"]:
        print("❌ 缺少必要依赖，请运行: pip install requests")
        return False
    
    print()
    
    # 测试结果统计
    tests_passed = 0
    tests_total = 0
    
    # 测试HelixDB连接
    tests_total += 1
    if test_helix_connection():
        tests_passed += 1
    
    print()
    
    # 测试OpenAI API（如果有API密钥）
    openai_api_key = os.getenv("OPENAI_API_KEY")
    if openai_api_key:
        tests_total += 1
        if test_openai_embedding(openai_api_key):
            tests_passed += 1
    else:
        print("⚠️ 跳过OpenAI API测试（未设置API密钥）")
    
    print()
    
    # 测试本地API
    tests_total += 1
    if test_local_embedding():
        tests_passed += 1
    
    print()
    
    # 输出测试结果
    print("📊 测试结果汇总")
    print("-" * 20)
    print(f"总测试数: {tests_total}")
    print(f"通过: {tests_passed}")
    print(f"失败: {tests_total - tests_passed}")
    
    if tests_passed == tests_total:
        print("🎉 所有测试通过！")
        return True
    else:
        print("⚠️ 部分测试失败，请检查配置")
        return False

def print_usage():
    """打印使用说明"""
    print("""
使用方法:
    python test_embedding_api.py

环境变量:
    OPENAI_API_KEY    - OpenAI API密钥（可选）

依赖安装:
    pip install requests

测试内容:
    1. HelixDB连接测试
    2. OpenAI向量化API测试（如果有API密钥）
    3. 本地向量化API测试（Ollama等）

示例:
    # 设置API密钥并运行测试
    export OPENAI_API_KEY="sk-..."
    python test_embedding_api.py
    
    # 仅测试本地服务
    python test_embedding_api.py
""")

if __name__ == "__main__":
    if len(sys.argv) > 1 and sys.argv[1] in ["-h", "--help"]:
        print_usage()
    else:
        success = main()
        sys.exit(0 if success else 1)
