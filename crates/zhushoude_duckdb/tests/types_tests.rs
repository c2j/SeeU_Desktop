//! 类型定义模块的单元测试

use zhushoude_duckdb::types::*;
use serde_json::json;

#[test]
fn test_document_creation() {
    let document = Document {
        id: "doc1".to_string(),
        title: "测试文档".to_string(),
        content: "这是一个测试文档的内容".to_string(),
        doc_type: DocumentType::Note,
        metadata: json!({"author": "测试用户", "tags": ["测试", "文档"]}),
    };
    
    assert_eq!(document.id, "doc1");
    assert_eq!(document.title, "测试文档");
    assert!(document.content.contains("测试文档"));
    assert!(matches!(document.doc_type, DocumentType::Note));
}

#[test]
fn test_note_creation() {
    let note = Note {
        id: "note1".to_string(),
        title: "学习笔记".to_string(),
        content: "今天学习了Rust编程".to_string(),
        metadata: json!({"subject": "编程", "difficulty": "中等"}),
    };
    
    assert_eq!(note.id, "note1");
    assert_eq!(note.title, "学习笔记");
    assert!(note.content.contains("Rust"));
}

#[test]
fn test_code_document_creation() {
    let code_doc = CodeDocument {
        id: "code1".to_string(),
        filename: "main.rs".to_string(),
        content: "fn main() { println!(\"Hello, world!\"); }".to_string(),
        language: CodeLanguage::Rust,
        metadata: json!({"project": "hello_world", "lines": 1}),
    };
    
    assert_eq!(code_doc.filename, "main.rs");
    assert!(matches!(code_doc.language, CodeLanguage::Rust));
    assert!(code_doc.content.contains("main"));
}

#[test]
fn test_document_type_display() {
    assert_eq!(DocumentType::Note.to_string(), "note");
    assert_eq!(DocumentType::Code(CodeLanguage::Java).to_string(), "code_java");
    assert_eq!(DocumentType::Markdown.to_string(), "markdown");
    assert_eq!(DocumentType::Text.to_string(), "text");
}

#[test]
fn test_code_language_display() {
    assert_eq!(CodeLanguage::Java.to_string(), "java");
    assert_eq!(CodeLanguage::SQL.to_string(), "sql");
    assert_eq!(CodeLanguage::Rust.to_string(), "rust");
    assert_eq!(CodeLanguage::Python.to_string(), "python");
    assert_eq!(CodeLanguage::JavaScript.to_string(), "javascript");
    assert_eq!(CodeLanguage::TypeScript.to_string(), "typescript");
    assert_eq!(CodeLanguage::Cpp.to_string(), "cpp");
    assert_eq!(CodeLanguage::C.to_string(), "c");
    assert_eq!(CodeLanguage::Go.to_string(), "go");
    assert_eq!(CodeLanguage::Other("kotlin".to_string()).to_string(), "kotlin");
}

#[test]
fn test_search_result() {
    let result = SearchResult {
        document_id: "doc1".to_string(),
        title: "相关文档".to_string(),
        content: "这是搜索到的内容".to_string(),
        doc_type: "note".to_string(),
        similarity_score: 0.85,
        metadata: Some(json!({"relevance": "high"})),
    };
    
    assert_eq!(result.similarity_score, 0.85);
    assert!(result.similarity_score > 0.8);
    assert!(result.metadata.is_some());
}

#[test]
fn test_hybrid_result() {
    let result = HybridResult {
        document_id: "doc1".to_string(),
        title: "混合搜索结果".to_string(),
        content: "内容".to_string(),
        score: 0.7,
        final_score: 0.8,
        result_type: ResultType::Semantic,
        metadata: json!({}),
    };
    
    assert_eq!(result.score, 0.7);
    assert_eq!(result.final_score, 0.8);
    assert!(matches!(result.result_type, ResultType::Semantic));
}

#[test]
fn test_hybrid_query() {
    let query = HybridQuery {
        text: "机器学习算法".to_string(),
        query_type: QueryType::Note,
        limit: 10,
        enable_semantic: true,
        enable_graph: false,
        weights: SearchWeights {
            semantic: 0.8,
            graph: 0.2,
        },
    };
    
    assert_eq!(query.text, "机器学习算法");
    assert_eq!(query.query_type, QueryType::Note);
    assert_eq!(query.limit, 10);
    assert!(query.enable_semantic);
    assert!(!query.enable_graph);
    assert_eq!(query.weights.semantic, 0.8);
    assert_eq!(query.weights.graph, 0.2);
}

#[test]
fn test_graph_node() {
    use std::collections::HashMap;
    
    let mut properties = HashMap::new();
    properties.insert("visibility".to_string(), json!("public"));
    properties.insert("abstract".to_string(), json!(false));
    
    let node = GraphNode {
        id: "class1".to_string(),
        label: "UserService".to_string(),
        node_type: NodeType::Class,
        properties,
    };
    
    assert_eq!(node.label, "UserService");
    assert!(matches!(node.node_type, NodeType::Class));
    assert_eq!(node.properties.len(), 2);
}

#[test]
fn test_graph_edge() {
    use std::collections::HashMap;
    
    let mut properties = HashMap::new();
    properties.insert("method_name".to_string(), json!("getUserById"));
    
    let edge = GraphEdge {
        id: "edge1".to_string(),
        source_id: "class1".to_string(),
        target_id: "class2".to_string(),
        edge_type: EdgeType::DependsOn,
        weight: 0.9,
        properties,
    };
    
    assert_eq!(edge.source_id, "class1");
    assert_eq!(edge.target_id, "class2");
    assert!(matches!(edge.edge_type, EdgeType::DependsOn));
    assert_eq!(edge.weight, 0.9);
}

#[test]
fn test_node_type_display() {
    assert_eq!(NodeType::Class.to_string(), "class");
    assert_eq!(NodeType::Method.to_string(), "method");
    assert_eq!(NodeType::Variable.to_string(), "variable");
    assert_eq!(NodeType::Package.to_string(), "package");
    assert_eq!(NodeType::Interface.to_string(), "interface");
    assert_eq!(NodeType::Other.to_string(), "other");
}

#[test]
fn test_edge_type_display() {
    assert_eq!(EdgeType::DependsOn.to_string(), "depends_on");
    assert_eq!(EdgeType::Inherits.to_string(), "inherits");
    assert_eq!(EdgeType::Implements.to_string(), "implements");
    assert_eq!(EdgeType::Calls.to_string(), "calls");
    assert_eq!(EdgeType::Contains.to_string(), "contains");
    assert_eq!(EdgeType::Other.to_string(), "other");
}

#[test]
fn test_performance_stats() {
    let stats = PerformanceStats {
        memory_usage: MemoryUsage {
            total_mb: 150.5,
            model_mb: 95.0,
            cache_mb: 30.0,
            database_mb: 25.5,
        },
        cache_stats: CacheStats {
            hits: 1000,
            misses: 200,
            hit_rate: 0.833,
            size: 5000,
        },
        query_stats: QueryStats {
            total_queries: 500,
            avg_query_time_ms: 45.2,
            semantic_queries: 300,
            graph_queries: 100,
            hybrid_queries: 100,
        },
    };
    
    assert_eq!(stats.memory_usage.total_mb, 150.5);
    assert_eq!(stats.cache_stats.hits, 1000);
    assert_eq!(stats.query_stats.total_queries, 500);
    
    // 验证计算
    let expected_total = stats.cache_stats.hits + stats.cache_stats.misses;
    assert_eq!(expected_total, 1200);
    
    let expected_query_total = stats.query_stats.semantic_queries + 
                              stats.query_stats.graph_queries + 
                              stats.query_stats.hybrid_queries;
    assert_eq!(expected_query_total, 500);
}

#[test]
fn test_serialization() {
    let document = Document {
        id: "test".to_string(),
        title: "测试".to_string(),
        content: "内容".to_string(),
        doc_type: DocumentType::Code(CodeLanguage::Java),
        metadata: json!({"test": true}),
    };
    
    // 测试JSON序列化
    let json_str = serde_json::to_string(&document).expect("序列化失败");
    let deserialized: Document = serde_json::from_str(&json_str).expect("反序列化失败");
    
    assert_eq!(document.id, deserialized.id);
    assert_eq!(document.title, deserialized.title);
    assert_eq!(document.content, deserialized.content);
}

#[test]
fn test_result_type_serialization() {
    let types = vec![
        ResultType::Semantic,
        ResultType::Graph,
        ResultType::Hybrid,
    ];
    
    for result_type in types {
        let json_str = serde_json::to_string(&result_type).expect("序列化失败");
        let deserialized: ResultType = serde_json::from_str(&json_str).expect("反序列化失败");
        
        match (&result_type, &deserialized) {
            (ResultType::Semantic, ResultType::Semantic) => {},
            (ResultType::Graph, ResultType::Graph) => {},
            (ResultType::Hybrid, ResultType::Hybrid) => {},
            _ => panic!("序列化/反序列化不匹配"),
        }
    }
}

#[test]
fn test_default_implementations() {
    let memory_usage = MemoryUsage::default();
    assert_eq!(memory_usage.total_mb, 0.0);
    assert_eq!(memory_usage.model_mb, 0.0);
    
    let cache_stats = CacheStats::default();
    assert_eq!(cache_stats.hits, 0);
    assert_eq!(cache_stats.misses, 0);
    assert_eq!(cache_stats.hit_rate, 0.0);
    
    let query_stats = QueryStats::default();
    assert_eq!(query_stats.total_queries, 0);
    assert_eq!(query_stats.avg_query_time_ms, 0.0);
    
    let performance_stats = PerformanceStats::default();
    assert_eq!(performance_stats.memory_usage.total_mb, 0.0);
    assert_eq!(performance_stats.cache_stats.hits, 0);
    assert_eq!(performance_stats.query_stats.total_queries, 0);
}
