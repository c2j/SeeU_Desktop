# 思源笔记导入功能指南

## 概述

SeeU Desktop 现在支持从思源笔记的存储目录中直接导入笔记本和笔记，包括图片等附件。这个功能经过了全面优化，能够正确解析思源笔记的原生存储格式。

## 主要特性

### ✅ 完整的思源笔记支持
- **自动识别笔记本**: 按照思源笔记ID格式 (`YYYYMMDDHHMMSS-xxxxxxx`) 自动识别笔记本目录
- **解析笔记本配置**: 读取 `.siyuan/conf.json` 获取笔记本名称、图标等元信息
- **解析.sy文档文件**: 正确解析JSON格式的思源笔记文档
- **资源文件处理**: 自动导入 `assets` 目录中的图片、附件等资源文件

### ✅ 丰富的节点类型支持
- **文本段落** (`NodeParagraph`)
- **标题** (`NodeHeading`) - 支持1-6级标题
- **列表** (`NodeList`/`NodeListItem`) - 支持有序和无序列表
- **代码块** (`NodeCodeBlock`) - 保留语言类型信息
- **引用块** (`NodeBlockquote`)
- **文本标记** (`NodeTextMark`) - 加粗、斜体、链接等
- **块引用** (`block-ref`) - 思源笔记特有的块引用系统

### ✅ 智能内容转换
- **Markdown转换**: 将思源笔记内容转换为标准Markdown格式
- **标签提取**: 从文档属性和文本内容中提取标签
- **附件处理**: 自动识别和导入图片、文档等附件
- **路径处理**: 正确处理相对路径和资源引用

## 使用方法

### 1. 通过UI界面导入

1. 打开 SeeU Desktop
2. 进入 iNote 模块
3. 点击 "📥 从思源笔记导入" 按钮
4. 输入思源笔记的工作空间根目录路径
5. 点击 "开始导入"

### 2. 工作空间目录结构

思源笔记的工作空间目录通常具有以下结构：

```
工作空间目录/
├── assets/                 # 全局资源文件
├── emojis/                 # 自定义表情图片
├── snippets/               # 代码片段
├── storage/                # 查询条件、布局、闪卡数据等
├── templates/              # 模板片段
├── widgets/                # 挂件
├── plugins/                # 插件
├── public/                 # 公开数据
├── 20210808180117-czj9bvb/ # 笔记本1
│   ├── .siyuan/
│   │   └── conf.json       # 笔记本配置
│   ├── assets/             # 笔记本专属资源
│   ├── 20200812220555-lj3enxa.sy  # 文档文件
│   └── 子目录/             # 子文档目录
├── 20210808180117-6v0mkxr/ # 笔记本2
└── .siyuan/                # 全局配置目录
```

### 3. 支持的路径示例

- **macOS**: `~/SiYuan/` 或 `/Users/username/Documents/SiYuan/`
- **Windows**: `C:\Users\username\Documents\SiYuan\`
- **Linux**: `~/SiYuan/` 或 `/home/username/Documents/SiYuan/`

## 导入过程详解

### 第一步：验证目录结构
- 检查工作空间根目录是否存在
- 验证是否包含符合格式的笔记本目录

### 第二步：扫描笔记本
- 使用正则表达式 `^[0-9]{14}-[0-9a-z]{7}$` 识别笔记本目录
- 读取每个笔记本的 `.siyuan/conf.json` 配置文件
- 提取笔记本名称、图标、排序等信息

### 第三步：处理文档
- 遍历笔记本中的所有 `.sy` 文件
- 解析JSON格式的文档结构
- 递归处理文档节点树
- 转换为Markdown格式

### 第四步：处理资源文件
- 扫描 `assets` 目录中的所有文件
- 建立资源文件路径映射
- 识别文档中的资源引用
- 创建附件记录

### 第五步：提取元数据
- 从文档属性中提取标签
- 从文本内容中识别井号标签 (`#标签名`)
- 处理自定义属性和备注
- 保留创建和更新时间

## 技术实现细节

### 数据结构

```rust
// 思源笔记文档结构
pub struct SiyuanDocument {
    pub id: String,
    pub spec: Option<String>,
    pub doc_type: String,
    pub properties: Option<HashMap<String, Value>>,
    pub children: Option<Vec<SiyuanNode>>,
}

// 思源笔记节点结构
pub struct SiyuanNode {
    pub id: Option<String>,
    pub node_type: String,
    pub data: Option<String>,
    pub properties: Option<HashMap<String, Value>>,
    pub children: Option<Vec<SiyuanNode>>,
    pub heading_level: Option<i32>,
    // ... 其他字段
}
```

### 节点转换示例

```rust
// 标题节点转换
"NodeHeading" => {
    let level = node.heading_level.unwrap_or(1);
    result.push_str(&"#".repeat(level as usize));
    result.push(' ');
    // 处理子节点...
    result.push_str("\n\n");
}

// 列表项转换
"NodeListItem" => {
    result.push_str(&"  ".repeat(depth));
    result.push_str("- ");
    // 处理子节点...
    result.push('\n');
}
```

## 常见问题解决

### Q: 导入失败，提示"无效的思源笔记目录"
**A**: 请确保：
- 选择的是思源笔记的工作空间根目录
- 目录中包含以时间戳格式命名的笔记本文件夹
- 有足够的文件读取权限

### Q: 部分笔记内容丢失
**A**: 可能原因：
- 思源笔记使用了不支持的节点类型
- 文档结构损坏或格式不正确
- 检查日志文件获取详细错误信息

### Q: 图片和附件没有导入
**A**: 请检查：
- `assets` 目录是否存在
- 文件路径引用是否正确
- 文件是否有读取权限

### Q: 中文内容显示乱码
**A**: 确保：
- 思源笔记文件使用UTF-8编码
- 系统支持中文字符显示

## 性能优化建议

### 大型笔记本处理
- 导入过程在后台线程中执行，不会阻塞UI
- 支持进度显示和错误报告
- 建议分批导入大量笔记本

### 内存使用优化
- 使用流式处理避免一次性加载所有数据
- 及时释放不需要的资源
- 对大文档进行分块处理

## 开发者信息

### 依赖项
- `regex`: 用于笔记本ID格式验证
- `serde_json`: JSON文档解析
- `walkdir`: 递归目录遍历
- `chrono`: 时间处理
- `uuid`: 生成唯一标识符

### 测试
运行测试套件：
```bash
cargo test siyuan_import
```

### 示例代码
查看 `examples/siyuan_import_example.rs` 了解如何使用导入API。

## 更新日志

### v1.0.0 (2024-12-XX)
- ✅ 完全重写思源笔记导入功能
- ✅ 支持原生.sy文件格式解析
- ✅ 添加资源文件处理
- ✅ 改进节点类型转换
- ✅ 优化错误处理和日志记录
- ✅ 添加完整的测试套件

---

如有问题或建议，请提交Issue或联系开发团队。
