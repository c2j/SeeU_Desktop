# Mermaid 图表测试

这是一个测试文档，用于验证我们的笔记系统是否支持Mermaid图表渲染。

## 流程图示例

```mermaid
graph TD
    A[开始] --> B{是否登录?}
    B -->|是| C[显示主页]
    B -->|否| D[显示登录页]
    D --> E[用户输入凭据]
    E --> F{验证成功?}
    F -->|是| C
    F -->|否| G[显示错误信息]
    G --> D
    C --> H[结束]
```

## 时序图示例

```mermaid
sequenceDiagram
    participant 用户
    participant 前端
    participant 后端
    participant 数据库
    
    用户->>前端: 登录请求
    前端->>后端: 发送凭据
    后端->>数据库: 验证用户
    数据库-->>后端: 返回结果
    后端-->>前端: 登录响应
    前端-->>用户: 显示结果
```

## 类图示例

```mermaid
classDiagram
    class Note {
        +String id
        +String title
        +String content
        +DateTime created_at
        +DateTime updated_at
        +create()
        +update()
        +delete()
    }
    
    class Notebook {
        +String id
        +String name
        +String description
        +List~Note~ notes
        +addNote()
        +removeNote()
    }
    
    class Tag {
        +String id
        +String name
        +String color
    }
    
    Notebook ||--o{ Note : contains
    Note }o--o{ Tag : tagged_with
```

## 状态图示例

```mermaid
stateDiagram-v2
    [*] --> 草稿
    草稿 --> 编辑中 : 开始编辑
    编辑中 --> 草稿 : 保存草稿
    编辑中 --> 已发布 : 发布
    已发布 --> 编辑中 : 编辑
    已发布 --> 已归档 : 归档
    已归档 --> [*]
```

## 甘特图示例

```mermaid
gantt
    title 项目开发计划
    dateFormat  YYYY-MM-DD
    section 设计阶段
    需求分析           :done,    des1, 2024-01-01,2024-01-05
    UI设计            :done,    des2, 2024-01-06, 2024-01-15
    section 开发阶段
    后端开发          :active,  dev1, 2024-01-16, 2024-02-15
    前端开发          :         dev2, 2024-01-20, 2024-02-20
    section 测试阶段
    单元测试          :         test1, 2024-02-16, 2024-02-25
    集成测试          :         test2, 2024-02-21, 2024-03-01
```

## 饼图示例

```mermaid
pie title 编程语言使用比例
    "Rust" : 45
    "Python" : 25
    "JavaScript" : 20
    "其他" : 10
```

## Git图示例

```mermaid
gitgraph
    commit
    commit
    branch develop
    checkout develop
    commit
    commit
    checkout main
    merge develop
    commit
    commit
```

## 用户旅程图示例

```mermaid
journey
    title 用户使用笔记应用的旅程
    section 发现
      打开应用: 5: 用户
      浏览界面: 4: 用户
    section 使用
      创建笔记: 5: 用户
      编辑内容: 4: 用户
      添加标签: 3: 用户
    section 分享
      导出笔记: 4: 用户
      分享链接: 3: 用户
```

## 实体关系图示例

```mermaid
erDiagram
    USER ||--o{ NOTE : creates
    USER {
        string id
        string username
        string email
        datetime created_at
    }
    NOTE ||--o{ TAG_RELATION : has
    NOTE {
        string id
        string title
        text content
        datetime created_at
        datetime updated_at
        string user_id
    }
    TAG ||--o{ TAG_RELATION : belongs_to
    TAG {
        string id
        string name
        string color
    }
    TAG_RELATION {
        string note_id
        string tag_id
    }
```

---

这些图表应该在笔记预览和幻灯片播放模式中都能正确显示。如果看到的是占位符图像而不是实际的Mermaid图表，说明我们的实现还需要进一步完善。
