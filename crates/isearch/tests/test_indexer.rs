use isearch::indexer::Indexer;
use isearch::{SearchResult, IndexStats, IndexedDirectory};
use chrono::Utc;

#[test]
fn test_indexer_creation() {
    let indexer = Indexer::new();
    // 测试索引器能够正常创建
    // 初始化索引
    assert!(indexer.initialize_index().is_ok());
}

#[test]
fn test_search_result_creation() {
    let result = SearchResult {
        id: "test-id".to_string(),
        filename: "test.rs".to_string(),
        path: "/test/file.rs".to_string(),
        file_type: "Rust".to_string(),
        size_bytes: 1024,
        modified: Utc::now(),
        content_preview: "fn main() { println!(\"Hello\"); }".to_string(),
        score: 0.95,
    };

    assert_eq!(result.filename, "test.rs");
    assert_eq!(result.path, "/test/file.rs");
    assert_eq!(result.file_type, "Rust");
    assert_eq!(result.size_bytes, 1024);
    assert_eq!(result.score, 0.95);
    assert!(result.content_preview.contains("main"));
}

#[test]
fn test_index_stats_creation() {
    let stats = IndexStats {
        total_files: 100,
        total_size_bytes: 1024000,
        last_updated: Some(Utc::now()),
    };

    assert_eq!(stats.total_files, 100);
    assert_eq!(stats.total_size_bytes, 1024000);
    assert!(stats.last_updated.is_some());
}

#[test]
fn test_indexed_directory_creation() {
    let indexed_dir = IndexedDirectory {
        path: "/test/directory".to_string(),
        last_indexed: Some(Utc::now()),
        file_count: 50,
        total_size_bytes: 2048000,
    };

    assert_eq!(indexed_dir.path, "/test/directory");
    assert!(indexed_dir.last_indexed.is_some());
    assert_eq!(indexed_dir.file_count, 50);
    assert_eq!(indexed_dir.total_size_bytes, 2048000);
}

#[test]
fn test_search_result_scoring() {
    // 测试搜索结果评分
    let high_score_result = SearchResult {
        id: "high-score".to_string(),
        filename: "important.rs".to_string(),
        path: "/important/file.rs".to_string(),
        file_type: "Rust".to_string(),
        size_bytes: 2048,
        modified: Utc::now(),
        content_preview: "fn main() { /* important code */ }".to_string(),
        score: 0.95,
    };

    let low_score_result = SearchResult {
        id: "low-score".to_string(),
        filename: "other.rs".to_string(),
        path: "/other/file.rs".to_string(),
        file_type: "Rust".to_string(),
        size_bytes: 1024,
        modified: Utc::now(),
        content_preview: "fn helper() { /* helper code */ }".to_string(),
        score: 0.25,
    };

    assert!(high_score_result.score > low_score_result.score);
    assert!(high_score_result.score >= 0.0 && high_score_result.score <= 1.0);
    assert!(low_score_result.score >= 0.0 && low_score_result.score <= 1.0);
}

#[test]
fn test_index_stats_calculations() {
    // 测试索引统计计算
    let stats = IndexStats {
        total_files: 1000,
        total_size_bytes: 50_000_000, // 50MB
        last_updated: Some(Utc::now()),
    };

    // 计算平均文件大小
    let avg_file_size = stats.total_size_bytes / stats.total_files as u64;
    assert_eq!(avg_file_size, 50_000); // 50KB per file on average

    // 验证统计数据的合理性
    assert!(stats.total_files > 0);
    assert!(stats.total_size_bytes > 0);
    assert!(stats.last_updated.is_some());
}

#[test]
fn test_indexed_directory_validation() {
    // 测试索引目录验证
    let valid_dir = IndexedDirectory {
        path: "/valid/path".to_string(),
        last_indexed: Some(Utc::now()),
        file_count: 25,
        total_size_bytes: 1_000_000,
    };

    let empty_dir = IndexedDirectory {
        path: "/empty/path".to_string(),
        last_indexed: Some(Utc::now()),
        file_count: 0,
        total_size_bytes: 0,
    };

    // 验证有效目录
    assert!(!valid_dir.path.is_empty());
    assert!(valid_dir.file_count > 0);
    assert!(valid_dir.total_size_bytes > 0);

    // 验证空目录
    assert!(!empty_dir.path.is_empty());
    assert_eq!(empty_dir.file_count, 0);
    assert_eq!(empty_dir.total_size_bytes, 0);
}

#[test]
fn test_search_result_content_preview() {
    // 测试搜索结果内容预览
    let long_content = "This is a very long content that should be truncated in the preview. ".repeat(10);
    let result = SearchResult {
        id: "preview-test".to_string(),
        filename: "long_file.txt".to_string(),
        path: "/test/long_file.txt".to_string(),
        file_type: "Text".to_string(),
        size_bytes: long_content.len() as u64,
        modified: Utc::now(),
        content_preview: if long_content.len() > 200 {
            format!("{}...", &long_content[..200])
        } else {
            long_content.clone()
        },
        score: 0.8,
    };

    // 验证预览内容被适当截断
    assert!(result.content_preview.len() <= 203); // 200 + "..."
    assert!(result.content_preview.contains("This is a very long content"));
    
    if result.content_preview.len() > 200 {
        assert!(result.content_preview.ends_with("..."));
    }
}
