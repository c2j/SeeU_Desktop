# zhushoude_duckdb NLP功能文档

## 概述

zhushoude_duckdb现在集成了强大的中文自然语言处理功能，包括实体识别、关系抽取和知识图谱构建。这些功能专门为笔记系统的语义搜索和实体关系分析而设计。

## 主要功能

### 1. 中文命名实体识别（NER）

**位置**: `src/nlp/ner.rs`

**功能特点**:
- 支持多种实体类型：人名、地名、机构名、时间、概念、技术术语、产品名称
- 基于规则和模式匹配的识别方法
- 内置中文词典支持
- 置信度评估
- 重叠实体处理

**支持的实体类型**:
- `PERSON`: 人名（如：张三、李四）
- `LOCATION`: 地名（如：北京、清华大学）
- `ORGANIZATION`: 机构名（如：阿里巴巴、腾讯）
- `TIME`: 时间表达（如：2024年1月1日、今天）
- `CONCEPT`: 概念术语（如：机器学习理论）
- `TECHNOLOGY`: 技术名词（如：Python、人工智能）
- `PRODUCT`: 产品名称（如：软件、应用程序）
- `OTHER`: 其他类型

**使用示例**:
```rust
use zhushoude_duckdb::nlp::ChineseNER;

let ner = ChineseNER::new()?;
let text = "张三在清华大学研究人工智能技术";
let entities = ner.extract_entities(text)?;

for entity in entities {
    println!("{} ({:?}) - 置信度: {:.3}", 
             entity.text, entity.entity_type, entity.confidence);
}
```

### 2. 实体关系抽取

**位置**: `src/nlp/relation_extraction.rs`

**功能特点**:
- 基于语法模式的关系识别
- 支持多种关系类型
- 实体共现分析
- 关系置信度评估
- 重复关系去除

**支持的关系类型**:
- `CONTAINS`: 包含关系（A包含B）
- `BELONGS_TO`: 属于关系（A属于B）
- `RELATED_TO`: 相关关系（A与B相关）
- `DEPENDS_ON`: 依赖关系（A依赖B）
- `LOCATED_IN`: 位于关系（A位于B）
- `WORKS_AT`: 工作关系（A工作于B）
- `CREATES`: 创建关系（A创建B）
- `USES`: 使用关系（A使用B）
- `STUDIES`: 学习关系（A学习B）
- `RESEARCHES`: 研究关系（A研究B）
- `OCCURS_AT`: 时间关系（A发生在B时间）

**使用示例**:
```rust
use zhushoude_duckdb::nlp::{ChineseNER, ChineseRelationExtractor};

let ner = ChineseNER::new()?;
let extractor = ChineseRelationExtractor::new()?;

let text = "张三工作于阿里巴巴公司";
let entities = ner.extract_entities(text)?;
let relations = extractor.extract_relations(text, &entities)?;

for relation in relations {
    println!("{} --[{}]--> {}", 
             relation.subject.text,
             relation.relation_type.as_str(),
             relation.object.text);
}
```

### 3. 知识图谱构建

**位置**: `src/nlp/knowledge_graph.rs`

**功能特点**:
- 自动构建知识图谱
- 实体节点和关系边管理
- 图谱统计信息
- 相关实体查找
- 持久化存储支持

**核心组件**:
- `KnowledgeNode`: 知识图谱节点
- `KnowledgeEdge`: 知识图谱边
- `KnowledgeGraph`: 完整知识图谱
- `KnowledgeGraphBuilder`: 图谱构建器

**使用示例**:
```rust
use zhushoude_duckdb::nlp::KnowledgeGraphBuilder;

let mut kg_builder = KnowledgeGraphBuilder::new(db_manager);

// 从实体和关系构建知识图谱
kg_builder.build_from_entities_and_relations(&entities, &relations, "doc_id").await?;

// 查找相关实体
let related = kg_builder.find_related_entities("清华大学", 2);

// 获取实体关系
let relations = kg_builder.get_entity_relations("张三");
```

### 4. 增强的语义搜索

**集成位置**: `src/lib.rs` 中的 `ZhushoudeDB`

**功能特点**:
- 基于实体的搜索增强
- 相关实体推荐
- 语义相似度计算
- 搜索结果重排序

**新增API**:
```rust
// 添加笔记并提取实体关系
let (entities, relations) = db.add_note_with_entities(&document).await?;

// 增强的语义搜索
let results = db.search_notes_with_entities("查询文本", 10).await?;

// 获取相关实体
let related = db.get_related_entities("实体名", 2).await?;

// 获取实体关系
let relations = db.get_entity_relations("实体名").await?;

// 获取知识图谱统计
let stats = db.get_knowledge_graph_stats().await?;
```

## inote集成

### 知识图谱管理器

**位置**: `crates/inote/src/knowledge_graph_integration.rs`

**功能**:
- 笔记的实体提取和关系识别
- 语义搜索接口
- 知识图谱统计
- 异步处理支持

### 可视化界面

**位置**: `crates/inote/src/knowledge_graph_ui.rs`

**功能**:
- 实体关系图可视化
- 交互式图谱浏览
- 实体详情展示
- 缩放和平移支持

## 测试

### 单元测试

**位置**: `tests/nlp_tests.rs`

**覆盖范围**:
- NER功能测试
- 关系抽取测试
- 知识图谱构建测试
- 边界情况测试
- 性能测试

### 集成测试

**位置**: `tests/integration_tests.rs`, `crates/inote/tests/`

**覆盖范围**:
- 端到端流程测试
- 多文档处理测试
- 并发操作测试
- 错误处理测试

### 性能基准测试

**位置**: `benches/nlp_benchmark.rs`

**测试项目**:
- NER性能基准
- 关系抽取性能
- 知识图谱构建性能
- 内存使用测试
- 并发处理性能

**运行基准测试**:
```bash
cargo bench --bench nlp_benchmark
```

## 配置和使用

### 启用NLP功能

在创建`ZhushoudeDB`实例时，NLP功能会自动初始化：

```rust
let config = ZhushoudeConfig::default();
let db = ZhushoudeDB::new(config).await?;
```

### 在inote中启用知识图谱

```rust
// 在db_state.rs中
state.initialize_knowledge_graph(true);
```

### 性能优化建议

1. **批量处理**: 对于大量文档，建议批量处理以提高效率
2. **置信度过滤**: 设置合适的置信度阈值过滤低质量结果
3. **缓存策略**: 利用内置缓存机制减少重复计算
4. **异步处理**: 使用异步API避免阻塞UI线程

## 扩展和定制

### 添加新的实体类型

1. 在`EntityType`枚举中添加新类型
2. 更新`ChineseNER`的模式和词典
3. 添加相应的测试用例

### 添加新的关系类型

1. 在`RelationType`枚举中添加新类型
2. 在`ChineseRelationExtractor`中添加识别模式
3. 更新图数据模型支持

### 自定义识别规则

可以通过修改`ChineseNER`和`ChineseRelationExtractor`的初始化方法来添加自定义规则和词典。

## 注意事项

1. **中文支持**: 所有功能都针对中文文本进行了优化
2. **资源消耗**: NLP处理会消耗一定的CPU和内存资源
3. **准确性**: 基于规则的方法在某些复杂情况下可能不够准确
4. **扩展性**: 设计支持未来集成更先进的机器学习模型

## 未来改进方向

1. **机器学习模型**: 集成预训练的中文NLP模型
2. **实时更新**: 支持知识图谱的实时更新和增量构建
3. **多语言支持**: 扩展到其他语言的支持
4. **高级查询**: 支持更复杂的图查询语言
5. **可视化增强**: 提供更丰富的图谱可视化功能
