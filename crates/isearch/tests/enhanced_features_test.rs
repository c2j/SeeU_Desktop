use isearch::*;
use isearch::advanced_search::AdvancedSearchParser;
use isearch::export::{ExportFormat, ExportConfig, ExportMetadata, SearchResultExporter};
use chrono::Utc;
use std::time::Duration;

#[test]
fn test_advanced_search_parser() {
    let mut parser = AdvancedSearchParser::new();
    
    // Test basic query parsing
    let result = parser.parse("test file");
    assert!(result.is_ok());
    let query = result.unwrap();
    assert_eq!(query.terms.len(), 2);
    assert_eq!(query.terms[0].text, "test");
    assert_eq!(query.terms[1].text, "file");
    
    // Test filter parsing
    let result = parser.parse("document filetype:pdf size:>1MB");
    assert!(result.is_ok());
    let query = result.unwrap();
    assert_eq!(query.terms.len(), 1);
    assert_eq!(query.terms[0].text, "document");
    assert!(query.filters.contains_key("filetype"));
    assert!(query.filters.contains_key("size"));
}

#[test]
fn test_search_optimizer() {
    let optimizer = isearch::search_optimizer::SearchOptimizer::new();
    
    // Test cache functionality
    let query = "test query";
    let cached_result = optimizer.get_cached_result(query);
    assert!(cached_result.is_none()); // Should be empty initially
    
    // Test metrics
    let metrics = optimizer.get_metrics();
    assert_eq!(metrics.total_searches, 0);
    assert_eq!(metrics.cache_hits, 0);
    assert_eq!(metrics.cache_misses, 1); // One miss from the get_cached_result call
}

#[test]
fn test_export_functionality() {
    // Create sample search results
    let results = vec![
        SearchResult {
            id: "1".to_string(),
            filename: "test1.txt".to_string(),
            path: "/path/to/test1.txt".to_string(),
            file_type: "text".to_string(),
            size_bytes: 1024,
            modified: Utc::now(),
            score: 0.95,
            content_preview: "This is a test file content".to_string(),
        },
        SearchResult {
            id: "2".to_string(),
            filename: "test2.pdf".to_string(),
            path: "/path/to/test2.pdf".to_string(),
            file_type: "pdf".to_string(),
            size_bytes: 2048,
            modified: Utc::now(),
            score: 0.85,
            content_preview: "PDF document content".to_string(),
        },
    ];

    let config = ExportConfig {
        format: ExportFormat::Json,
        include_stats: true,
        include_content_preview: true,
        max_preview_length: 100,
        sort_by_relevance: true,
        include_metadata: true,
    };

    let metadata = ExportMetadata {
        query: "test query".to_string(),
        export_time: Utc::now(),
        total_results: results.len(),
        format: ExportFormat::Json,
        stats: Some(SearchStats {
            search_time_ms: 50,
            total_results: 2,
            total_matches: 2,
            query_time: Utc::now(),
        }),
    };

    // Test export to string
    let export_result = SearchResultExporter::export_to_string(&results, &config, &metadata);
    assert!(export_result.is_ok());
    let exported_content = export_result.unwrap();
    assert!(exported_content.contains("test1.txt"));
    assert!(exported_content.contains("test2.pdf"));
    assert!(exported_content.contains("metadata"));
}

#[test]
fn test_export_formats() {
    let formats = ISearchState::get_export_formats();
    assert!(formats.len() >= 5);
    assert!(formats.contains(&ExportFormat::Csv));
    assert!(formats.contains(&ExportFormat::Json));
    assert!(formats.contains(&ExportFormat::Text));
    assert!(formats.contains(&ExportFormat::Html));
    assert!(formats.contains(&ExportFormat::Markdown));
}

#[test]
fn test_background_indexer_creation() {
    use std::sync::{Arc, Mutex};
    use isearch::indexer::Indexer;
    use isearch::background_indexer::BackgroundIndexer;
    
    let indexer = Arc::new(Mutex::new(Indexer::new()));
    let background_indexer = BackgroundIndexer::new(indexer);
    
    // Test idle detection
    assert!(!background_indexer.is_idle()); // Should not be idle immediately
}

#[test]
fn test_enhanced_watcher_creation() {
    use std::sync::{Arc, Mutex};
    use isearch::indexer::Indexer;
    use isearch::enhanced_watcher::EnhancedFileWatcher;
    
    let indexer = Arc::new(Mutex::new(Indexer::new()));
    let mut enhanced_watcher = EnhancedFileWatcher::new(indexer);
    
    // Test configuration
    enhanced_watcher.set_debounce_duration(Duration::from_millis(1000));
    enhanced_watcher.set_max_batch_size(50);
    enhanced_watcher.add_ignore_pattern("*.tmp".to_string());
    
    // Test statistics (should be empty initially)
    let stats = enhanced_watcher.get_watch_statistics();
    assert!(stats.is_empty());
}

#[test]
fn test_isearch_state_enhanced_features() {
    let mut state = ISearchState::default();
    
    // Test enhanced monitoring settings (just call the methods, don't check private fields)
    state.set_enhanced_monitoring_enabled(true);
    state.set_file_change_debounce(1000);
    state.set_incremental_updates_enabled(true);

    // Test search optimization settings
    state.set_instant_search_enabled(true);
    state.set_search_delay(500);

    // Test background indexing settings
    state.set_auto_update_enabled(true);
    state.set_idle_threshold(10);
    
    // Test export functionality
    state.open_export_dialog();
    assert!(state.show_export_dialog);
    
    state.set_export_format(ExportFormat::Json);
    assert_eq!(state.export_format, ExportFormat::Json);
    
    state.close_export_dialog();
    assert!(!state.show_export_dialog);
    
    // Test search metrics
    let metrics = state.get_search_metrics();
    assert_eq!(metrics.total_searches, 0);
    
    // Test cache clearing
    state.clear_search_cache(); // Should not panic
}

#[test]
fn test_activity_monitor() {
    use isearch::background_indexer::ActivityMonitor;
    
    let monitor = ActivityMonitor::new();
    
    // Test activity recording
    monitor.record_activity();
    
    // Test idle detection
    let is_idle = monitor.is_idle(Duration::from_secs(1));
    assert!(!is_idle); // Should not be idle immediately after recording activity
    
    // Test time since last activity
    let time_since = monitor.time_since_last_activity();
    assert!(time_since.is_some());
    assert!(time_since.unwrap() < Duration::from_secs(1));
}
