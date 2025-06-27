# Mermaid 渲染性能测试

这个文档用于测试Mermaid图表的渲染性能优化。

## 流程图测试

```mermaid
graph TD
    A[开始] --> B{是否有缓存?}
    B -->|是| C[使用缓存字体]
    B -->|否| D[初始化字体数据库]
    D --> E[扫描系统字体]
    E --> F[加载嵌入字体]
    F --> G[选择最佳字体]
    G --> H[缓存字体信息]
    H --> C
    C --> I[渲染SVG]
    I --> J[转换为纹理]
    J --> K[显示图表]
    K --> L[结束]
```

## 序列图测试

```mermaid
sequenceDiagram
    participant U as 用户
    participant A as 应用程序
    participant M as Mermaid渲染器
    participant F as 字体缓存
    
    U->>A: 打开包含Mermaid的笔记
    A->>M: 请求渲染图表
    M->>F: 检查字体缓存
    alt 缓存存在
        F-->>M: 返回缓存字体
    else 缓存不存在
        F->>F: 初始化字体数据库
        F->>F: 扫描系统字体
        F->>F: 选择最佳字体
        F-->>M: 返回字体信息
    end
    M->>M: 生成SVG
    M->>M: 渲染为纹理
    M-->>A: 返回渲染结果
    A-->>U: 显示图表
```

## 类图测试

```mermaid
classDiagram
    class FontCache {
        -fontdb: Option~Arc~Database~~
        -last_working_font: Option~String~
        -available_fonts: Vec~String~
        -font_mapping: HashMap~String, String~
        -initialized: bool
        +get_or_create_fontdb() (Arc~Database~, Option~String~)
        +update_last_working_font(font: String)
        +preload_font_cache()
    }
    
    class MermaidRenderer {
        -diagram_cache: HashMap~String, Arc~TextureHandle~~
        -svg_options: usvg::Options
        +render_diagram(ui: &mut Ui, code: &str, font: Option~&str~)
        +get_svg_font_family(font: Option~&str~) String
        +svg_to_texture(ctx: &Context, svg: &str) Result~TextureHandle~
    }
    
    FontCache --> MermaidRenderer : 提供字体信息
```

## 状态图测试

```mermaid
stateDiagram-v2
    [*] --> 未初始化
    未初始化 --> 初始化中 : 首次渲染
    初始化中 --> 扫描字体 : 开始字体发现
    扫描字体 --> 选择字体 : 完成扫描
    选择字体 --> 缓存就绪 : 选择最佳字体
    缓存就绪 --> 渲染中 : 渲染请求
    渲染中 --> 缓存就绪 : 渲染完成
    缓存就绪 --> [*] : 应用关闭
```

## 甘特图测试

```mermaid
gantt
    title Mermaid渲染优化项目时间线
    dateFormat  YYYY-MM-DD
    section 分析阶段
    问题分析           :done,    analysis, 2024-01-01, 2024-01-03
    性能测试           :done,    testing,  2024-01-02, 2024-01-04
    section 开发阶段
    字体缓存设计       :done,    design,   2024-01-04, 2024-01-06
    缓存实现           :done,    impl,     2024-01-06, 2024-01-08
    性能优化           :active,  optimize, 2024-01-08, 2024-01-10
    section 测试阶段
    功能测试           :         func_test, 2024-01-10, 2024-01-12
    性能验证           :         perf_test, 2024-01-12, 2024-01-14
```

## 饼图测试

```mermaid
pie title 字体渲染时间分布
    "字体扫描" : 45
    "字体选择" : 20
    "SVG生成" : 25
    "纹理转换" : 10
```

## 测试说明

1. **字体缓存优化**：通过全局缓存避免重复的字体数据库初始化
2. **字体映射**：建立应用设置到实际字体名称的映射关系
3. **预加载机制**：在应用启动时预加载字体缓存
4. **字体验证**：在初始化时验证字体的实际可用性
5. **智能回退**：使用经过验证的字体回退列表
6. **性能监控**：记录字体初始化和渲染时间

## 字体优化特性

### 字体验证机制
- 在字体选择时验证字体是否真正可用
- 避免选择无法正常渲染的字体
- 减少usvg字体回退警告

### 智能字体选择
- 优先选择Arial Unicode MS等广泛支持的字体
- 对每个候选字体进行实际渲染测试
- 建立基于实际可用性的字体回退列表

### 减少日志警告
- 通过字体验证减少"Fallback from X to Y"警告
- 选择经过验证的字体避免渲染问题
- 提供更清晰的字体选择日志

## 预期改进

- 首次渲染后，后续Mermaid图表渲染速度显著提升
- 减少字体扫描导致的UI卡顿
- 更好地利用应用程序的字体设置
- 提供更一致的字体渲染效果
- 显著减少字体回退警告日志
- 提高字体渲染的可靠性
