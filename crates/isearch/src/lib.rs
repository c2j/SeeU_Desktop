pub mod indexer;
pub mod schema;
pub mod ui;
pub mod watcher;
pub mod file_types;
pub mod utils;

// egui is now only used in ui.rs
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{Receiver, Sender};
use std::path::Path;
use std::fs;
use std::io::Write;
use std::collections::HashMap;
// use rfd::FileDialog; // 移除 rfd 依赖以避免 async-recursion
use indexer::Indexer;
use watcher::FileWatcher;

/// iSearch state
pub struct ISearchState {
    pub search_query: String,
    pub search_results: Vec<SearchResult>,
    pub indexed_directories: Vec<IndexedDirectory>,
    pub is_searching: bool,
    pub is_indexing: bool,
    pub index_stats: IndexStats,
    pub selected_directory: Option<usize>,
    pub show_directory_dialog: bool,
    pub navigate_to_settings: bool,
    pub show_directories_panel: bool,  // Control directory panel visibility

    // Search statistics
    pub search_stats: SearchStats,
    pub has_more_results: bool,

    // Shared components
    indexer: Arc<Mutex<Indexer>>,
    file_watcher: Arc<Mutex<FileWatcher>>,

    // Background indexing communication
    stats_sender: Option<Sender<DirectoryIndexResult>>,
    stats_receiver: Option<Receiver<DirectoryIndexResult>>,

    // Background deletion communication
    deletion_sender: Option<Sender<String>>, // Send directory path when deletion is complete
    deletion_receiver: Option<Receiver<String>>,

    // File properties dialog
    pub show_properties_dialog: bool,
    pub properties_file: Option<SearchResult>,

    // Search options
    pub enable_content_preview: bool,
    pub enable_file_type_filter: bool,
    pub search_hidden_files: bool,
    pub enable_file_monitoring: bool,
    pub search_on_typing: bool,  // 是否在输入时触发搜索

    // Search result sorting
    pub sort_by: SortBy,
    pub sort_order: SortOrder,

    // Search result view mode
    pub view_mode: ViewMode,

    // Directory input (替代文件对话框)
    pub directory_input: String,
    pub show_directory_input_dialog: bool,

    // Search optimization
    search_cache: HashMap<String, (Vec<SearchResult>, SearchStats)>,
    last_search_query: String,
    search_result_receiver: Option<Receiver<SearchResponse>>,

    // File type filter UI
    pub show_file_type_filter: bool,
    pub selected_file_types: Vec<String>,
}

/// Search result
#[derive(Debug, Clone)]
pub struct SearchResult {
    pub id: String,
    pub filename: String,
    pub path: String,
    pub file_type: String,
    pub size_bytes: u64,
    pub modified: DateTime<Utc>,
    pub content_preview: String,
    pub score: f32,  // Search relevance score
}

/// Indexed directory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexedDirectory {
    pub path: String,
    pub last_indexed: Option<DateTime<Utc>>,
    #[serde(default)]
    pub file_count: usize,
    #[serde(default)]
    pub total_size_bytes: u64,
}

/// Index statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexStats {
    pub total_files: usize,
    pub total_size_bytes: u64,
    pub last_updated: Option<DateTime<Utc>>,
}

/// Directory indexing result
#[derive(Debug, Clone)]
pub struct DirectoryIndexResult {
    pub directory_path: String,
    pub stats: IndexStats,
}

/// Search statistics
#[derive(Debug, Clone, Default)]
pub struct SearchStats {
    pub total_results: usize,
    pub total_matches: usize,  // Total matches before deduplication
    pub search_time_ms: u64,
    pub query_time: DateTime<Utc>,
}

/// Search request for background processing
#[derive(Debug, Clone)]
pub struct SearchRequest {
    pub query: String,
    pub file_type_filter: Option<String>,
    pub filename_filter: Option<String>,
}

/// Search response from background processing
#[derive(Debug, Clone)]
pub struct SearchResponse {
    pub query: String,
    pub results: Vec<SearchResult>,
    pub stats: SearchStats,
    pub has_more_results: bool,
}

/// Sort criteria for search results
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SortBy {
    Relevance,    // Default: by search score
    FileName,     // By file name
    DirectoryName, // By directory path
    FileSize,     // By file size
    ModifiedTime, // By modification time
}

impl Default for SortBy {
    fn default() -> Self {
        SortBy::Relevance
    }
}

impl SortBy {
    pub fn display_name(&self) -> &'static str {
        match self {
            SortBy::Relevance => "相关性",
            SortBy::FileName => "文件名",
            SortBy::DirectoryName => "目录名",
            SortBy::FileSize => "文件大小",
            SortBy::ModifiedTime => "修改时间",
        }
    }
}

/// Sort order for search results
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SortOrder {
    Ascending,  // 升序
    Descending, // 降序
}

/// View mode for search results
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ViewMode {
    Detailed,  // 详细视图 - 当前的卡片式显示
    List,      // 列表视图 - 表格式显示
}

impl Default for SortOrder {
    fn default() -> Self {
        SortOrder::Descending // Default to descending for relevance
    }
}

impl SortOrder {
    pub fn display_name(&self) -> &'static str {
        match self {
            SortOrder::Ascending => "升序",
            SortOrder::Descending => "降序",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            SortOrder::Ascending => "⬆",
            SortOrder::Descending => "⬇",
        }
    }
}

impl Default for ViewMode {
    fn default() -> Self {
        ViewMode::Detailed // Default to detailed view
    }
}

impl ViewMode {
    pub fn display_name(&self) -> &'static str {
        match self {
            ViewMode::Detailed => "详细视图",
            ViewMode::List => "列表视图",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            ViewMode::Detailed => "📋",
            ViewMode::List => "📊",
        }
    }
}

impl Default for ISearchState {
    fn default() -> Self {
        // Create indexer
        let indexer = Arc::new(Mutex::new(Indexer::new()));

        // Initialize indexer asynchronously to avoid blocking startup
        let indexer_clone = indexer.clone();
        std::thread::spawn(move || {
            if let Ok(indexer_lock) = indexer_clone.lock() {
                let _ = indexer_lock.initialize_index(); // Ignore errors during startup
            }
        });

        // Create file watcher
        let file_watcher = Arc::new(Mutex::new(FileWatcher::new(indexer.clone())));

        // Create communication channels for background operations
        let (stats_sender, stats_receiver) = std::sync::mpsc::channel();
        let (deletion_sender, deletion_receiver) = std::sync::mpsc::channel();
        let (_search_sender, search_result_receiver) = std::sync::mpsc::channel();

        Self {
            search_query: String::new(),
            search_results: Vec::new(),
            indexed_directories: Vec::new(),
            is_searching: false,
            is_indexing: false,
            index_stats: IndexStats {
                total_files: 0,
                total_size_bytes: 0,
                last_updated: None,
            },
            selected_directory: None,
            show_directory_dialog: false,
            navigate_to_settings: false,
            show_directories_panel: false,  // Default to collapsed
            search_stats: SearchStats::default(),
            has_more_results: false,
            indexer,
            file_watcher,
            stats_sender: Some(stats_sender),
            stats_receiver: Some(stats_receiver),
            deletion_sender: Some(deletion_sender),
            deletion_receiver: Some(deletion_receiver),
            show_properties_dialog: false,
            properties_file: None,
            enable_content_preview: true,
            enable_file_type_filter: true,
            search_hidden_files: false,
            enable_file_monitoring: true,
            search_on_typing: false,  // 默认按回车触发搜索
            sort_by: SortBy::default(),
            sort_order: SortOrder::default(),
            view_mode: ViewMode::default(),
            directory_input: String::new(),
            show_directory_input_dialog: false,
            search_cache: HashMap::new(),
            last_search_query: String::new(),
            search_result_receiver: Some(search_result_receiver),
            show_file_type_filter: false,
            selected_file_types: Vec::new(),
        }
    }
}

impl ISearchState {
    /// Get indexer reference for external use
    pub fn get_indexer(&self) -> Arc<Mutex<Indexer>> {
        self.indexer.clone()
    }

    /// Initialize the state
    pub fn initialize(&mut self) {
        // Load indexed directories from disk
        self.load_indexed_directories();

        // Load search options from disk
        self.load_search_options();

        // Start watching directories asynchronously to avoid blocking startup
        let directories = self.indexed_directories.clone();
        let file_watcher = self.file_watcher.clone();

        std::thread::spawn(move || {
            for directory in &directories {
                if let Ok(mut watcher) = file_watcher.lock() {
                    let _ = watcher.watch_directory(directory); // Ignore errors during startup
                }
            }
        });
    }

    /// Load indexed directories from disk (public method for external use)
    pub fn load_indexed_directories(&mut self) {
        let base_path = dirs::data_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
        let config_path = base_path.join("seeu_desktop").join("isearch").join("directories.json");

        if let Ok(json) = fs::read_to_string(config_path) {
            if let Ok(directories) = serde_json::from_str::<Vec<IndexedDirectory>>(&json) {
                self.indexed_directories = directories;
            }
        }
    }

    /// Save indexed directories to disk (async to avoid UI blocking)
    pub fn save_indexed_directories(&self) {
        let directories = self.indexed_directories.clone();

        // Perform file I/O in background thread to avoid blocking UI
        std::thread::spawn(move || {
            let base_path = dirs::data_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
            let config_dir = base_path.join("seeu_desktop").join("isearch");
            let config_path = config_dir.join("directories.json");

            if let Err(e) = std::fs::create_dir_all(&config_dir) {
                log::warn!("Failed to create config directory: {}", e);
                return;
            }

            if let Ok(json) = serde_json::to_string_pretty(&directories) {
                if let Err(e) = std::fs::write(config_path, json) {
                    log::warn!("Failed to save indexed directories: {}", e);
                }
            }
        });
    }

    /// Save search options to disk (async to avoid UI blocking)
    pub fn save_search_options(&self) {
        // Clone the options to avoid borrowing issues in the background thread
        let options = serde_json::json!({
            "enable_content_preview": self.enable_content_preview,
            "enable_file_type_filter": self.enable_file_type_filter,
            "search_hidden_files": self.search_hidden_files,
            "enable_file_monitoring": self.enable_file_monitoring,
            "search_on_typing": self.search_on_typing,
            "sort_by": self.sort_by,
            "sort_order": self.sort_order,
            "view_mode": self.view_mode,
        });

        // Perform file I/O in background thread to avoid blocking UI
        std::thread::spawn(move || {
            let base_path = dirs::data_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
            let config_dir = base_path.join("seeu_desktop").join("isearch");
            let config_path = config_dir.join("search_options.json");

            if let Err(e) = std::fs::create_dir_all(&config_dir) {
                log::warn!("Failed to create config directory: {}", e);
                return;
            }

            if let Ok(json) = serde_json::to_string_pretty(&options) {
                if let Err(e) = std::fs::write(config_path, json) {
                    log::warn!("Failed to save search options: {}", e);
                }
            }
        });
    }

    /// Load search options from disk (public method for external use)
    pub fn load_search_options(&mut self) {
        let base_path = dirs::data_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
        let config_path = base_path.join("seeu_desktop").join("isearch").join("search_options.json");

        if let Ok(json) = fs::read_to_string(config_path) {
            if let Ok(options) = serde_json::from_str::<serde_json::Value>(&json) {
                if let Some(enable_content_preview) = options.get("enable_content_preview").and_then(|v| v.as_bool()) {
                    self.enable_content_preview = enable_content_preview;
                }
                if let Some(enable_file_type_filter) = options.get("enable_file_type_filter").and_then(|v| v.as_bool()) {
                    self.enable_file_type_filter = enable_file_type_filter;
                }
                if let Some(search_hidden_files) = options.get("search_hidden_files").and_then(|v| v.as_bool()) {
                    self.search_hidden_files = search_hidden_files;
                }
                if let Some(enable_file_monitoring) = options.get("enable_file_monitoring").and_then(|v| v.as_bool()) {
                    self.enable_file_monitoring = enable_file_monitoring;
                }
                if let Some(search_on_typing) = options.get("search_on_typing").and_then(|v| v.as_bool()) {
                    self.search_on_typing = search_on_typing;
                }
                if let Some(sort_by) = options.get("sort_by") {
                    if let Ok(sort_by_enum) = serde_json::from_value::<SortBy>(sort_by.clone()) {
                        self.sort_by = sort_by_enum;
                    }
                }
                if let Some(sort_order) = options.get("sort_order") {
                    if let Ok(sort_order_enum) = serde_json::from_value::<SortOrder>(sort_order.clone()) {
                        self.sort_order = sort_order_enum;
                    }
                }
                if let Some(view_mode) = options.get("view_mode") {
                    if let Ok(view_mode_enum) = serde_json::from_value::<ViewMode>(view_mode.clone()) {
                        self.view_mode = view_mode_enum;
                    }
                }
            }
        }
    }

    /// Add a directory to the index
    pub fn add_directory(&mut self, path: String) {
        // Check if directory exists
        if !Path::new(&path).exists() {
            log::error!("Directory does not exist: {}", path);
            return;
        }

        // Check if directory is already indexed
        if self.indexed_directories.iter().any(|dir| dir.path == path) {
            // Directory already indexed, skip silently
            return;
        }

        // Create new indexed directory
        let directory = IndexedDirectory {
            path: path.clone(),
            last_indexed: None,
            file_count: 0,
            total_size_bytes: 0,
        };

        // Add to list
        self.indexed_directories.push(directory.clone());

        // Save to disk
        self.save_indexed_directories();

        // Start watching the directory
        if let Ok(mut watcher) = self.file_watcher.lock() {
            let _ = watcher.watch_directory(&directory);
        }

        // Start indexing in background thread to avoid blocking UI
        self.is_indexing = true;

        // Clone necessary data for background thread
        let indexer = self.indexer.clone();
        let directory_clone = directory.clone();
        let stats_sender = self.stats_sender.clone();

        // Get search hidden files setting
        let include_hidden = self.search_hidden_files;

        // Spawn background thread for indexing
        std::thread::spawn(move || {
            log::info!("Starting background indexing for directory: {}", directory_clone.path);

            if let Ok(indexer_lock) = indexer.lock() {
                match indexer_lock.index_directory_with_options(&directory_clone, include_hidden) {
                    Ok(stats) => {
                        // Directory indexed successfully, send results silently

                        // Send results back to main thread
                        if let Some(sender) = &stats_sender {
                            let result = DirectoryIndexResult {
                                directory_path: directory_clone.path.clone(),
                                stats,
                            };
                            let _ = sender.send(result);
                        }
                    }
                    Err(err) => {
                        log::error!("Failed to index directory '{}': {}", directory_clone.path, err);

                        // Send empty stats to indicate completion (even if failed)
                        if let Some(sender) = &stats_sender {
                            let result = DirectoryIndexResult {
                                directory_path: directory_clone.path.clone(),
                                stats: IndexStats {
                                    total_files: 0,
                                    total_size_bytes: 0,
                                    last_updated: Some(Utc::now()),
                                },
                            };
                            let _ = sender.send(result);
                        }
                    }
                }
            }
        });
    }

    /// Remove a directory from the index
    pub fn remove_directory(&mut self, index: usize) {
        if index >= self.indexed_directories.len() {
            return;
        }

        // Get directory path
        let directory_path = self.indexed_directories[index].path.clone();

        // Stop watching the directory
        if let Ok(mut watcher) = self.file_watcher.lock() {
            let _ = watcher.unwatch_directory(&directory_path);
        }

        // Remove from list immediately (UI responsiveness)
        self.indexed_directories.remove(index);

        // Save to disk
        self.save_indexed_directories();

        // Reset selected directory if needed
        if self.selected_directory == Some(index) {
            self.selected_directory = None;
        } else if let Some(selected) = self.selected_directory {
            if selected > index {
                self.selected_directory = Some(selected - 1);
            }
        }

        // Start background deletion from index to avoid blocking UI
        let indexer = self.indexer.clone();
        let directory_path_clone = directory_path.clone();
        let deletion_sender = self.deletion_sender.clone();

        std::thread::spawn(move || {
            log::info!("Starting background deletion for directory: {}", directory_path_clone);

            if let Ok(indexer_lock) = indexer.lock() {
                match indexer_lock.remove_directory_from_index(&directory_path_clone) {
                    Ok(()) => {
                        log::info!("Successfully removed directory '{}' from index", directory_path_clone);

                        // Notify main thread that deletion is complete
                        if let Some(sender) = &deletion_sender {
                            let _ = sender.send(directory_path_clone);
                        }
                    }
                    Err(err) => {
                        log::error!("Failed to remove directory '{}' from index: {}", directory_path_clone, err);

                        // Still notify completion even if failed
                        if let Some(sender) = &deletion_sender {
                            let _ = sender.send(directory_path_clone);
                        }
                    }
                }
            }
        });

        // Clear search cache since index content has changed
        self.search_cache.clear();
        log::info!("Cleared search cache after directory removal");

        // Update index stats immediately (will be more accurate after background deletion completes)
        self.update_index_stats();
    }

    /// Update index statistics after directory removal
    fn update_index_stats(&mut self) {
        // For now, we'll reset the stats and let them be recalculated during the next indexing
        // In a more sophisticated implementation, we could query the index to get accurate counts
        self.index_stats = IndexStats {
            total_files: 0,
            total_size_bytes: 0,
            last_updated: Some(Utc::now()),
        };

        // Optionally, we could trigger a background task to recalculate stats from the index
        // This would involve querying all documents in the index and summing up the counts
        log::info!("Index statistics reset after directory removal");
    }

    /// Reindex all directories (useful after code changes)
    pub fn reindex_all_directories(&mut self) {
        log::info!("Starting reindex of all directories");

        // Clone the directories list to avoid borrowing issues
        let directories = self.indexed_directories.clone();

        if directories.is_empty() {
            log::info!("No directories to reindex");
            return;
        }

        // Clear search cache since we're rebuilding the index
        self.search_cache.clear();
        log::info!("Cleared search cache before reindexing");

        // Set indexing state
        self.is_indexing = true;

        // Clone necessary data for background thread
        let indexer = self.indexer.clone();
        let stats_sender = self.stats_sender.clone();
        let include_hidden = self.search_hidden_files;

        // Start indexing in background thread
        std::thread::spawn(move || {
            // First, completely rebuild the index (delete and recreate with new schema)
            if let Ok(indexer_lock) = indexer.lock() {
                if let Err(e) = indexer_lock.rebuild_index() {
                    log::error!("Failed to rebuild index before reindexing: {}", e);
                    return;
                }
            }

            for directory in directories {
                log::info!("Reindexing directory: {}", directory.path);

                if let Ok(indexer_lock) = indexer.lock() {
                    match indexer_lock.index_directory_with_options(&directory, include_hidden) {
                        Ok(stats) => {
                            log::info!("Successfully reindexed directory '{}': {} files, {} bytes",
                                     directory.path, stats.total_files, stats.total_size_bytes);

                            // Send result through channel
                            if let Some(sender) = &stats_sender {
                                let result = DirectoryIndexResult {
                                    directory_path: directory.path.clone(),
                                    stats,
                                };
                                let _ = sender.send(result);
                            }
                        },
                        Err(e) => {
                            log::error!("Failed to reindex directory '{}': {}", directory.path, e);
                        }
                    }
                }
            }

            log::info!("Completed reindexing all directories");
        });
    }

    /// Search for files with caching and async processing
    pub fn search(&mut self) {
        if self.search_query.trim().is_empty() {
            self.search_results.clear();
            return;
        }

        let query = self.search_query.trim().to_string();

        // Check cache first
        if let Some((cached_results, cached_stats)) = self.search_cache.get(&query) {
            log::debug!("Using cached search results for query: {}", query);
            self.search_results = cached_results.clone();
            self.search_stats = cached_stats.clone();
            self.has_more_results = cached_results.len() >= 100;
            self.sort_results();
            return;
        }

        // If same query as last time and still searching, don't start new search
        if self.is_searching && self.last_search_query == query {
            return;
        }

        self.is_searching = true;
        self.last_search_query = query.clone();

        // Parse advanced search query
        let (parsed_query, file_type_filter, filename_filter) = self.parse_advanced_query(&query);

        // For now, let's do synchronous search but with optimizations
        let start_time = std::time::Instant::now();

        // Perform search
        let mut raw_results = {
            let indexer_lock = self.indexer.lock().unwrap();
            indexer_lock.search_advanced(&parsed_query, file_type_filter.as_deref(), filename_filter.as_deref()).unwrap_or_default()
        };

        // Apply file type filter from UI if enabled and selected
        if self.enable_file_type_filter && !self.selected_file_types.is_empty() {
            raw_results.retain(|result| {
                self.selected_file_types.contains(&result.file_type.to_lowercase())
            });
        }

        // Store the total number of matches before deduplication
        let total_matches = raw_results.len();

        // Deduplicate results by file path, keeping only the highest-ranked match
        let mut path_to_best_result: HashMap<String, SearchResult> = HashMap::new();

        for result in raw_results {
            // Check if we already have a result for this path
            if let Some(existing_result) = path_to_best_result.get(&result.path) {
                // If the new result has a higher score, replace the existing one
                if result.score > existing_result.score {
                    path_to_best_result.insert(result.path.clone(), result);
                }
            } else {
                // First time seeing this path, add it to the map
                path_to_best_result.insert(result.path.clone(), result);
            }
        }

        // Convert the map values to a vector
        let deduplicated_results: Vec<SearchResult> = path_to_best_result.into_values().collect();

        // Calculate search time
        let search_time = start_time.elapsed();

        // Check if there are more results than the limit (100)
        self.has_more_results = deduplicated_results.len() >= 100;

        // Limit to 100 results if needed
        let final_results = if deduplicated_results.len() > 100 {
            deduplicated_results[0..100].to_vec()
        } else {
            deduplicated_results
        };

        // Update search statistics
        self.search_stats = SearchStats {
            total_results: final_results.len(),
            total_matches,
            search_time_ms: search_time.as_millis() as u64,
            query_time: Utc::now(),
        };

        // Cache the results (limit cache size to prevent memory issues)
        if self.search_cache.len() >= 50 {
            // Remove oldest entries (simple FIFO, could be improved with LRU)
            let keys_to_remove: Vec<String> = self.search_cache.keys().take(10).cloned().collect();
            for key in keys_to_remove {
                self.search_cache.remove(&key);
            }
        }

        self.search_cache.insert(query, (final_results.clone(), self.search_stats.clone()));

        // Update state with results
        self.search_results = final_results;

        // Apply current sort settings
        self.sort_results();

        self.is_searching = false;
    }

    /// Process search results from background thread (placeholder for future async implementation)
    pub fn process_search_results(&mut self) {
        // Currently using synchronous search, this method is reserved for future async implementation
    }

    /// Parse advanced search query
    fn parse_advanced_query(&self, query: &str) -> (String, Option<String>, Option<String>) {
        let mut parsed_query = String::new();
        let mut file_type = None;
        let mut filename = None;

        // Split query into parts
        let parts: Vec<&str> = query.split_whitespace().collect();

        for part in parts {
            // Check for filetype: filter
            if part.starts_with("filetype:") {
                file_type = part.strip_prefix("filetype:").map(|s| s.to_string());
            }
            // Check for filename: filter
            else if part.starts_with("filename:") {
                filename = part.strip_prefix("filename:").map(|s| s.to_string());
            }
            // Add to parsed query
            else {
                if !parsed_query.is_empty() {
                    parsed_query.push(' ');
                }

                // Handle exact phrase matching with quotes
                if (part.starts_with("\"") && part.ends_with("\"")) ||
                   (part.starts_with("'") && part.ends_with("'")) {
                    let phrase = &part[1..part.len()-1];
                    parsed_query.push_str(&format!("\"{}\"", phrase));
                }
                // Handle AND operator with +
                else if part.starts_with("+") {
                    let term = &part[1..];
                    parsed_query.push_str(&format!("+{}", term));
                }
                else {
                    parsed_query.push_str(part);
                }
            }
        }

        (parsed_query, file_type, filename)
    }

    /// Sort search results according to current sort settings
    pub fn sort_results(&mut self) {
        match self.sort_by {
            SortBy::Relevance => {
                // Sort by search score
                match self.sort_order {
                    SortOrder::Descending => {
                        self.search_results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
                    }
                    SortOrder::Ascending => {
                        self.search_results.sort_by(|a, b| a.score.partial_cmp(&b.score).unwrap_or(std::cmp::Ordering::Equal));
                    }
                }
            }
            SortBy::FileName => {
                // Sort by file name (case-insensitive)
                match self.sort_order {
                    SortOrder::Ascending => {
                        self.search_results.sort_by(|a, b| a.filename.to_lowercase().cmp(&b.filename.to_lowercase()));
                    }
                    SortOrder::Descending => {
                        self.search_results.sort_by(|a, b| b.filename.to_lowercase().cmp(&a.filename.to_lowercase()));
                    }
                }
            }
            SortBy::DirectoryName => {
                // Sort by directory path (case-insensitive)
                match self.sort_order {
                    SortOrder::Ascending => {
                        self.search_results.sort_by(|a, b| {
                            let dir_a = std::path::Path::new(&a.path).parent().unwrap_or(std::path::Path::new("")).to_string_lossy().to_lowercase();
                            let dir_b = std::path::Path::new(&b.path).parent().unwrap_or(std::path::Path::new("")).to_string_lossy().to_lowercase();
                            dir_a.cmp(&dir_b)
                        });
                    }
                    SortOrder::Descending => {
                        self.search_results.sort_by(|a, b| {
                            let dir_a = std::path::Path::new(&a.path).parent().unwrap_or(std::path::Path::new("")).to_string_lossy().to_lowercase();
                            let dir_b = std::path::Path::new(&b.path).parent().unwrap_or(std::path::Path::new("")).to_string_lossy().to_lowercase();
                            dir_b.cmp(&dir_a)
                        });
                    }
                }
            }
            SortBy::FileSize => {
                // Sort by file size
                match self.sort_order {
                    SortOrder::Ascending => {
                        self.search_results.sort_by(|a, b| a.size_bytes.cmp(&b.size_bytes));
                    }
                    SortOrder::Descending => {
                        self.search_results.sort_by(|a, b| b.size_bytes.cmp(&a.size_bytes));
                    }
                }
            }
            SortBy::ModifiedTime => {
                // Sort by modification time
                match self.sort_order {
                    SortOrder::Ascending => {
                        self.search_results.sort_by(|a, b| a.modified.cmp(&b.modified));
                    }
                    SortOrder::Descending => {
                        self.search_results.sort_by(|a, b| b.modified.cmp(&a.modified));
                    }
                }
            }
        }
    }

    /// Change sort criteria and re-sort results
    pub fn set_sort_by(&mut self, sort_by: SortBy) {
        // If clicking the same sort criteria, toggle the order
        if self.sort_by == sort_by {
            self.sort_order = match self.sort_order {
                SortOrder::Ascending => SortOrder::Descending,
                SortOrder::Descending => SortOrder::Ascending,
            };
        } else {
            // Different sort criteria, use default order for that criteria
            self.sort_by = sort_by.clone();
            self.sort_order = match sort_by {
                SortBy::Relevance => SortOrder::Descending, // Higher score first
                SortBy::FileName => SortOrder::Ascending,   // A-Z
                SortBy::DirectoryName => SortOrder::Ascending, // A-Z
                SortBy::FileSize => SortOrder::Descending,  // Larger files first
                SortBy::ModifiedTime => SortOrder::Descending, // Newer files first
            };
        }

        // Re-sort the current results
        self.sort_results();

        // Save sort preferences
        self.save_search_options();
    }

    /// Open directory input dialog to select a directory
    pub fn open_directory_dialog(&mut self) {
        self.show_directory_input_dialog = true;
        // 设置默认路径
        if self.directory_input.is_empty() {
            self.directory_input = dirs::home_dir()
                .unwrap_or_else(|| std::path::PathBuf::from("/"))
                .to_string_lossy()
                .to_string();
        }
    }

    /// Process directory input dialog result
    pub fn process_directory_dialog(&mut self) {
        // 这个方法现在由 UI 直接调用，不需要处理文件对话框
    }

    /// Add directory from input (called by UI)
    pub fn add_directory_from_input(&mut self) {
        if !self.directory_input.trim().is_empty() {
            let path = self.directory_input.trim().to_string();
            self.add_directory(path);
            self.directory_input.clear();
            self.show_directory_input_dialog = false;
        }
    }

    /// Open a file
    pub fn open_file(&self, path: &str) {
        if let Err(err) = open::that(path) {
            log::error!("Failed to open file {}: {}", path, err);
        }
    }

    /// Open the folder containing a file
    pub fn open_folder(&self, file_path: &str) {
        let path = Path::new(file_path);

        // Get the parent directory
        if let Some(parent_dir) = path.parent() {
            let folder_path = parent_dir.to_string_lossy().to_string();

            // On different platforms, we might want to select the file in the folder
            #[cfg(target_os = "windows")]
            {
                // On Windows, use explorer with /select to highlight the file
                let output = std::process::Command::new("explorer")
                    .args(&["/select,", file_path])
                    .output();

                if output.is_err() {
                    // Fallback to opening just the folder
                    if let Err(err) = open::that(&folder_path) {
                        log::error!("Failed to open folder {}: {}", folder_path, err);
                    }
                }
            }

            #[cfg(target_os = "macos")]
            {
                // On macOS, use Finder with -R to reveal the file
                let output = std::process::Command::new("open")
                    .args(&["-R", file_path])
                    .output();

                if output.is_err() {
                    // Fallback to opening just the folder
                    if let Err(err) = open::that(&folder_path) {
                        log::error!("Failed to open folder {}: {}", folder_path, err);
                    }
                }
            }

            #[cfg(not(any(target_os = "windows", target_os = "macos")))]
            {
                // On Linux and other platforms, just open the folder
                if let Err(err) = open::that(&folder_path) {
                    log::error!("Failed to open folder {}: {}", folder_path, err);
                }
            }
        } else {
            log::error!("Could not determine parent directory for file: {}", file_path);
        }
    }

    /// Copy text to clipboard
    fn copy_to_clipboard(&self, text: &str) {
        // For now, we'll use a simple approach with the clipboard crate
        // In a production app, you might want to use a more robust clipboard implementation
        if let Err(err) = self.set_clipboard_text(text) {
            log::error!("Failed to copy to clipboard: {}", err);
        } else {
            log::info!("Copied to clipboard: {}", text);
        }
    }

    /// Set clipboard text (platform-specific implementation)
    fn set_clipboard_text(&self, text: &str) -> Result<(), Box<dyn std::error::Error>> {
        // For now, we'll use a simple cross-platform approach
        // In a real implementation, you might want to use the clipboard crate

        #[cfg(target_os = "windows")]
        {
            std::process::Command::new("cmd")
                .args(&["/C", &format!("echo {} | clip", text.replace("\"", "\\\""))])
                .output()?;
        }

        #[cfg(target_os = "macos")]
        {
            std::process::Command::new("pbcopy")
                .arg(text)
                .stdin(std::process::Stdio::piped())
                .spawn()?
                .stdin
                .as_mut()
                .ok_or("Failed to open stdin")?
                .write_all(text.as_bytes())?;
        }

        #[cfg(target_os = "linux")]
        {
            // Try xclip first, then xsel as fallback
            let xclip_result = std::process::Command::new("xclip")
                .args(&["-selection", "clipboard"])
                .stdin(std::process::Stdio::piped())
                .spawn()
                .and_then(|mut child| {
                    child.stdin.as_mut()
                        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::Other, "Failed to open stdin"))?
                        .write_all(text.as_bytes())?;
                    child.wait()
                });

            if xclip_result.is_err() {
                // Fallback to xsel
                std::process::Command::new("xsel")
                    .args(&["-ib"])
                    .stdin(std::process::Stdio::piped())
                    .spawn()?
                    .stdin
                    .as_mut()
                    .ok_or("Failed to open stdin")?
                    .write_all(text.as_bytes())?;
            }
        }

        Ok(())
    }

    /// Copy file name to clipboard
    pub fn copy_filename(&self, result: &SearchResult) {
        self.copy_to_clipboard(&result.filename);
    }

    /// Copy file path to clipboard (directory only)
    pub fn copy_filepath(&self, result: &SearchResult) {
        // Extract directory path without filename
        let path = std::path::Path::new(&result.path);
        if let Some(parent) = path.parent() {
            self.copy_to_clipboard(&parent.to_string_lossy());
        } else {
            self.copy_to_clipboard(&result.path);
        }
    }

    /// Copy file path and name to clipboard (full path)
    pub fn copy_path_and_name(&self, result: &SearchResult) {
        self.copy_to_clipboard(&result.path);
    }

    /// Show file properties dialog
    pub fn show_file_properties(&mut self, result: &SearchResult) {
        self.properties_file = Some(result.clone());
        self.show_properties_dialog = true;
        log::info!("Showing properties for file: {}", result.filename);
    }

    /// Update index for a specific directory
    pub fn update_directory_index(&mut self, directory_index: usize) {
        if directory_index >= self.indexed_directories.len() {
            log::warn!("Invalid directory index: {}", directory_index);
            return;
        }

        let directory = &self.indexed_directories[directory_index];
        log::info!("Starting manual index update for directory: {}", directory.path);

        // Set indexing state
        self.is_indexing = true;

        // Clone necessary data for the background thread
        let directory_clone = directory.clone();
        let indexer = self.indexer.clone();
        let stats_sender = self.stats_sender.clone();

        // Get search hidden files setting
        let include_hidden = self.search_hidden_files;

        // Start indexing in background thread
        std::thread::spawn(move || {
            if let Ok(indexer_lock) = indexer.lock() {
                match indexer_lock.index_directory_with_options(&directory_clone, include_hidden) {
                    Ok(stats) => {
                        log::info!("Manual index update completed for directory '{}': {} files, {:.1} MB",
                                  directory_clone.path, stats.total_files,
                                  stats.total_size_bytes as f64 / (1024.0 * 1024.0));

                        // Send results back to main thread
                        if let Some(sender) = &stats_sender {
                            let result = DirectoryIndexResult {
                                directory_path: directory_clone.path.clone(),
                                stats,
                            };
                            let _ = sender.send(result);
                        }
                    }
                    Err(e) => {
                        log::error!("Failed to update index for directory '{}': {}", directory_clone.path, e);

                        // Send empty stats to indicate completion (even if failed)
                        if let Some(sender) = &stats_sender {
                            let result = DirectoryIndexResult {
                                directory_path: directory_clone.path.clone(),
                                stats: IndexStats {
                                    total_files: 0,
                                    total_size_bytes: 0,
                                    last_updated: Some(Utc::now()),
                                },
                            };
                            let _ = sender.send(result);
                        }
                    }
                }
            }
        });
    }

    /// Update index for all directories
    pub fn update_all_indexes(&mut self) {
        if self.indexed_directories.is_empty() {
            log::info!("No directories to update");
            return;
        }

        log::info!("Starting manual index update for all {} directories", self.indexed_directories.len());

        // Set indexing state
        self.is_indexing = true;

        // Clone necessary data for the background thread
        let directories = self.indexed_directories.clone();
        let indexer = self.indexer.clone();
        let stats_sender = self.stats_sender.clone();

        // Get search hidden files setting
        let include_hidden = self.search_hidden_files;

        // Start indexing in background thread
        std::thread::spawn(move || {
            for directory in directories {
                if let Ok(indexer_lock) = indexer.lock() {
                    match indexer_lock.index_directory_with_options(&directory, include_hidden) {
                        Ok(stats) => {
                            log::info!("Manual index update completed for directory '{}': {} files, {:.1} MB",
                                      directory.path, stats.total_files,
                                      stats.total_size_bytes as f64 / (1024.0 * 1024.0));

                            // Send results back to main thread
                            if let Some(sender) = &stats_sender {
                                let result = DirectoryIndexResult {
                                    directory_path: directory.path.clone(),
                                    stats,
                                };
                                let _ = sender.send(result);
                            }
                        }
                        Err(e) => {
                            log::error!("Failed to update index for directory '{}': {}", directory.path, e);

                            // Send empty stats to indicate completion (even if failed)
                            if let Some(sender) = &stats_sender {
                                let result = DirectoryIndexResult {
                                    directory_path: directory.path.clone(),
                                    stats: IndexStats {
                                        total_files: 0,
                                        total_size_bytes: 0,
                                        last_updated: Some(Utc::now()),
                                    },
                                };
                                let _ = sender.send(result);
                            }
                        }
                    }
                }
            }
        });
    }

    /// Process file watcher events
    pub fn process_watcher_events(&mut self) {
        // Only process file watcher events if file monitoring is enabled
        if !self.enable_file_monitoring {
            return;
        }

        if let Ok(mut watcher) = self.file_watcher.lock() {
            // 只检测事件，不立即执行索引
            let directories_to_reindex = watcher.check_events();

            // 如果有需要重新索引的目录，启动后台线程处理
            if !directories_to_reindex.is_empty() {
                // 克隆必要的数据
                let indexer = self.indexer.clone();
                let directories = directories_to_reindex.clone();
                let stats_sender = self.stats_sender.clone();
                let include_hidden = self.search_hidden_files;

                // 在后台线程中执行索引
                std::thread::spawn(move || {
                    for dir_path in directories {
                        let directory = IndexedDirectory {
                            path: dir_path.clone(),
                            last_indexed: Some(Utc::now()),
                            file_count: 0,
                            total_size_bytes: 0,
                        };

                        if let Ok(indexer_lock) = indexer.lock() {
                            if let Ok(stats) = indexer_lock.index_directory_with_options(&directory, include_hidden) {
                                // 通过通道发送结果
                                if let Some(sender) = &stats_sender {
                                    let result = DirectoryIndexResult {
                                        directory_path: dir_path.clone(),
                                        stats,
                                    };
                                    let _ = sender.send(result);
                                }
                            }
                        }
                    }
                });
            }
        }

        // 检查是否有索引完成的结果
        self.check_reindex_results();
    }

    /// Check for completed reindex results from background threads
    pub fn check_reindex_results(&mut self) {
        // Check for completed indexing operations
        if let Some(receiver) = &self.stats_receiver {
            // Try to receive any completed indexing results without blocking
            while let Ok(result) = receiver.try_recv() {
                // Update global index stats with the new results
                self.index_stats.total_files += result.stats.total_files;
                self.index_stats.total_size_bytes += result.stats.total_size_bytes;
                self.index_stats.last_updated = Some(Utc::now());

                // Update the specific directory's stats
                for directory in &mut self.indexed_directories {
                    if directory.path == result.directory_path {
                        directory.last_indexed = Some(Utc::now());
                        directory.file_count = result.stats.total_files;
                        directory.total_size_bytes = result.stats.total_size_bytes;
                        break;
                    }
                }

                // Save updated directories to disk
                self.save_indexed_directories();

                // Mark indexing as complete
                self.is_indexing = false;

                log::info!("Background indexing completed for directory '{}': {} files, {:.1} MB",
                          result.directory_path, result.stats.total_files,
                          result.stats.total_size_bytes as f64 / (1024.0 * 1024.0));
            }
        }

        // Check for completed deletion operations
        if let Some(receiver) = &self.deletion_receiver {
            while let Ok(deleted_directory_path) = receiver.try_recv() {
                log::info!("Background deletion completed for directory: {}", deleted_directory_path);
                // Deletion completion doesn't require UI updates since we already removed from list
                // But we could trigger a stats recalculation here if needed
            }
        }
    }
}

// Re-export UI functions for backward compatibility
pub use ui::{render_isearch, render_isearch_with_sidebar_info};
