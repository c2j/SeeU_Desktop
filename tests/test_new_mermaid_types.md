# 新增Mermaid图表类型测试

本文档用于测试新增的Mermaid图表类型支持。

## 1. 类图 (Class Diagram)

```mermaid
classDiagram
    class Animal {
        +String name
        +int age
        +makeSound()
        +move()
    }
    
    class Dog {
        +String breed
        +bark()
        +wagTail()
    }
    
    class Cat {
        +String color
        +meow()
        +purr()
    }
    
    Animal <|-- Dog
    Animal <|-- Cat
    
    class Owner {
        +String name
        +feedPet()
        +walkDog()
    }
    
    Owner --> Animal : owns
```

## 2. 状态图 (State Diagram)

```mermaid
stateDiagram-v2
    [*] --> Idle
    
    Idle --> Processing : start
    Processing --> Success : complete
    Processing --> Error : fail
    Success --> [*]
    Error --> Retry : retry
    Retry --> Processing
    Error --> [*] : abort
    
    state Processing {
        [*] --> Validating
        Validating --> Executing
        Executing --> [*]
    }
```

## 3. Git图 (Git Graph)

```mermaid
gitGraph
    commit id: "Initial"
    commit id: "Setup"
    branch feature
    checkout feature
    commit id: "Feature A"
    commit id: "Feature B"
    checkout main
    commit id: "Hotfix"
    merge feature
    commit id: "Release"
```

## 4. 用户旅程图 (User Journey)

```mermaid
journey
    title My working day
    section Go to work
      Make tea: 5: Me
      Go upstairs: 3: Me
      Do work: 1: Me, Cat
    section Go home
      Go downstairs: 5: Me
      Sit down: 5: Me
```

## 5. 实体关系图 (Entity Relationship Diagram)

```mermaid
erDiagram
    CUSTOMER {
        string name
        string custNumber
        string sector
    }
    ORDER {
        int orderNumber
        string deliveryAddress
        datetime orderDate
    }
    LINE-ITEM {
        string productCode
        int quantity
        float pricePerUnit
    }
    DELIVERY-ADDRESS {
        string street
        string city
        string country
    }
    
    CUSTOMER ||--o{ ORDER : places
    ORDER ||--|{ LINE-ITEM : contains
    CUSTOMER }|..|{ DELIVERY-ADDRESS : uses
```

## 测试说明

以上图表应该能够正确渲染，显示：
- 类图：显示类的结构和继承关系
- 状态图：显示状态转换和嵌套状态
- Git图：显示分支、合并和提交历史
- 用户旅程图：显示用户体验流程和满意度
- 实体关系图：显示数据库表结构和关系

如果图表无法正确渲染，应该显示相应的占位符和错误信息。
