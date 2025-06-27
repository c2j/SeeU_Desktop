# 语义搜索功能测试指南

## 📋 概述

本指南提供了完整的语义搜索功能测试方案，包括自动化测试脚本和手动测试步骤。

## 🎯 测试目标

验证以下功能：
- ✅ HelixDB进程管理和连接
- ✅ 向量化服务（OpenAI/本地模型）
- ✅ 笔记自动向量化
- ✅ 语义搜索查询
- ✅ 数据持久化和恢复

## 🚀 快速开始

### 1. 环境检查
```bash
# 检查HelixDB安装
helix --version

# 检查Rust环境
cargo --version

# 检查Python环境（用于API测试）
python3 --version
```

### 2. 运行自动化测试
```bash
# 基础功能测试
./test_semantic_search.sh

# API连接测试
python3 test_embedding_api.py

# 详细日志模式
RUST_LOG=debug ./test_semantic_search.sh
```

### 3. 配置语义搜索
```bash
# 创建配置目录
mkdir -p ~/Library/Application\ Support/SeeU_Desktop/semantic_search

# 复制配置模板
cp semantic_search_config_example.toml ~/Library/Application\ Support/SeeU_Desktop/semantic_search/config.toml

# 编辑配置（添加API密钥等）
nano ~/Library/Application\ Support/SeeU_Desktop/semantic_search/config.toml
```

## 📁 测试文件说明

| 文件 | 用途 | 类型 |
|------|------|------|
| `semantic_search_test_guide.md` | 详细测试指南 | 文档 |
| `test_semantic_search.sh` | 自动化测试脚本 | Shell脚本 |
| `test_embedding_api.py` | API连接测试工具 | Python脚本 |
| `semantic_search_config_example.toml` | 配置文件模板 | 配置文件 |

## 🔧 配置选项

### OpenAI配置（推荐）
```toml
enabled = true

[embedding_config]
provider = "openai"
api_key = "sk-your-api-key-here"
model = "text-embedding-3-small"
```

### 本地模型配置（免费）
```toml
enabled = true

[embedding_config]
provider = "local"
api_base = "http://localhost:11434/v1"
model = "nomic-embed-text"
```

## 🧪 测试步骤

### 阶段1：基础环境验证
1. **运行环境检查**：
   ```bash
   ./test_semantic_search.sh
   ```

2. **检查输出**：
   - ✅ HelixDB安装检查通过
   - ✅ 应用编译检查通过
   - ✅ 配置目录创建成功

### 阶段2：API服务验证
1. **测试向量化API**：
   ```bash
   # 设置API密钥（如果使用OpenAI）
   export OPENAI_API_KEY="sk-..."
   
   # 运行API测试
   python3 test_embedding_api.py
   ```

2. **检查输出**：
   - ✅ OpenAI API连接成功（如果配置）
   - ✅ 本地API连接成功（如果运行）
   - ✅ HelixDB连接成功

### 阶段3：应用集成测试
1. **启动应用**：
   ```bash
   RUST_LOG=info cargo run
   ```

2. **观察日志**：
   ```
   🔧 开始初始化语义搜索模块...
   ✅ 语义搜索模块初始化成功
   ```

3. **创建测试笔记**：
   - 创建包含不同主题的笔记
   - 观察向量化日志

4. **测试搜索功能**：
   - 执行语义搜索查询
   - 验证搜索结果相关性

### 阶段4：持久化测试
1. **重启应用**：
   - 关闭应用
   - 重新启动
   - 验证数据保持

2. **检查进程**：
   ```bash
   # 检查HelixDB进程
   ps aux | grep helix
   lsof -i :6969
   ```

## 🔍 故障排除

### 常见问题及解决方案

#### 1. HelixDB未安装
```bash
# macOS
brew install helix-db

# 或从GitHub下载
wget https://github.com/helix-db/helix/releases/latest/download/helix-macos
chmod +x helix-macos
sudo mv helix-macos /usr/local/bin/helix
```

#### 2. 端口被占用
```bash
# 查找占用进程
lsof -i :6969

# 终止进程
kill -9 <PID>
```

#### 3. API密钥问题
```bash
# 验证OpenAI API密钥
curl -H "Authorization: Bearer $OPENAI_API_KEY" \
     https://api.openai.com/v1/models
```

#### 4. 本地模型服务
```bash
# 启动Ollama服务
ollama serve

# 拉取嵌入模型
ollama pull nomic-embed-text
```

## 📊 测试验收标准

### 必须通过的测试
- [ ] 应用启动无错误
- [ ] 语义搜索模块初始化成功
- [ ] HelixDB进程正常运行
- [ ] 至少一种向量化服务可用
- [ ] 笔记可以正常向量化
- [ ] 搜索返回相关结果

### 可选测试
- [ ] OpenAI API连接成功
- [ ] 本地模型API连接成功
- [ ] 混合搜索权重调整
- [ ] 大量笔记处理性能

## 🎉 测试完成后

如果所有测试通过，您可以：

1. **开始使用语义搜索**：
   - 创建更多笔记
   - 尝试不同的搜索查询
   - 调整搜索权重

2. **性能优化**：
   - 调整批处理大小
   - 优化缓存设置
   - 监控内存使用

3. **功能扩展**：
   - 集成到其他模块
   - 添加更多向量化提供商
   - 实现增量索引更新

## 📞 获取帮助

如果遇到问题：

1. **查看详细日志**：
   ```bash
   RUST_LOG=debug cargo run
   ```

2. **检查配置文件**：
   ```bash
   cat ~/Library/Application\ Support/SeeU_Desktop/semantic_search/config.toml
   ```

3. **参考详细指南**：
   - `semantic_search_test_guide.md` - 完整测试指南
   - `semantic_search_config_example.toml` - 配置示例

4. **常用调试命令**：
   ```bash
   # 检查进程
   ps aux | grep -E "(helix|seeu)"
   
   # 检查端口
   netstat -an | grep 6969
   
   # 检查日志
   tail -f ~/.local/share/SeeU_Desktop/logs/app.log
   ```

---

**祝您测试顺利！** 🚀
