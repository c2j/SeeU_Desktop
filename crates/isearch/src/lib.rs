pub mod indexer;
pub mod schema;
pub mod ui;
pub mod watcher;
pub mod utils;

use eframe::egui;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{Receiver, Sender};
use std::path::Path;
use std::fs;
use std::thread;
use std::time::Duration;
use std::io::Write;
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

    // Search result sorting
    pub sort_by: SortBy,
    pub sort_order: SortOrder,

    // Directory input (替代文件对话框)
    pub directory_input: String,
    pub show_directory_input_dialog: bool,
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

impl Default for ISearchState {
    fn default() -> Self {
        // Create indexer
        let indexer = Arc::new(Mutex::new(Indexer::new()));

        // Initialize indexer
        if let Ok(indexer_lock) = indexer.lock() {
            let _ = indexer_lock.initialize_index();
        }

        // Create file watcher
        let file_watcher = Arc::new(Mutex::new(FileWatcher::new(indexer.clone())));

        // Create communication channels for background operations
        let (stats_sender, stats_receiver) = std::sync::mpsc::channel();
        let (deletion_sender, deletion_receiver) = std::sync::mpsc::channel();

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
            sort_by: SortBy::default(),
            sort_order: SortOrder::default(),
            directory_input: String::new(),
            show_directory_input_dialog: false,
        }
    }
}

impl ISearchState {
    /// Initialize the state
    pub fn initialize(&mut self) {
        // Load indexed directories from disk
        self.load_indexed_directories();

        // Load search options from disk
        self.load_search_options();

        // Start watching all directories
        for directory in &self.indexed_directories {
            if let Ok(mut watcher) = self.file_watcher.lock() {
                let _ = watcher.watch_directory(directory);
            }
        }
    }

    /// Load indexed directories from disk
    fn load_indexed_directories(&mut self) {
        let base_path = dirs::data_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
        let config_path = base_path.join("seeu_desktop").join("isearch").join("directories.json");

        if let Ok(json) = fs::read_to_string(config_path) {
            if let Ok(directories) = serde_json::from_str::<Vec<IndexedDirectory>>(&json) {
                self.indexed_directories = directories;
            }
        }
    }

    /// Save indexed directories to disk
    pub fn save_indexed_directories(&self) {
        let base_path = dirs::data_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
        let config_dir = base_path.join("seeu_desktop").join("isearch");
        let config_path = config_dir.join("directories.json");

        fs::create_dir_all(&config_dir).ok();

        if let Ok(json) = serde_json::to_string_pretty(&self.indexed_directories) {
            let _ = fs::write(config_path, json);
        }
    }

    /// Save search options to disk
    pub fn save_search_options(&self) {
        let base_path = dirs::data_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
        let config_dir = base_path.join("seeu_desktop").join("isearch");
        let config_path = config_dir.join("search_options.json");

        fs::create_dir_all(&config_dir).ok();

        let options = serde_json::json!({
            "enable_content_preview": self.enable_content_preview,
            "enable_file_type_filter": self.enable_file_type_filter,
            "search_hidden_files": self.search_hidden_files,
            "enable_file_monitoring": self.enable_file_monitoring,
            "sort_by": self.sort_by,
            "sort_order": self.sort_order,
        });

        if let Ok(json) = serde_json::to_string_pretty(&options) {
            let _ = fs::write(config_path, json);
        }
    }

    /// Load search options from disk
    fn load_search_options(&mut self) {
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
            log::info!("Directory already indexed: {}", path);
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

        // Spawn background thread for indexing
        std::thread::spawn(move || {
            log::info!("Starting background indexing for directory: {}", directory_clone.path);

            if let Ok(indexer_lock) = indexer.lock() {
                match indexer_lock.index_directory(&directory_clone) {
                    Ok(stats) => {
                        log::info!("Successfully indexed directory '{}': {} files, {:.1} MB",
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

    /// Search for files
    pub fn search(&mut self) {
        if self.search_query.trim().is_empty() {
            self.search_results.clear();
            return;
        }

        self.is_searching = true;

        // Record start time for search statistics
        let start_time = std::time::Instant::now();

        // Parse advanced search query
        let (parsed_query, file_type_filter, filename_filter) = self.parse_advanced_query(&self.search_query);

        // Perform search
        let raw_results = {
            let indexer_lock = self.indexer.lock().unwrap();
            indexer_lock.search_advanced(&parsed_query, file_type_filter.as_deref(), filename_filter.as_deref()).unwrap_or_default()
        };

        // Store the total number of matches before deduplication
        let total_matches = raw_results.len();

        // Deduplicate results by file path, keeping only the highest-ranked match
        let mut path_to_best_result: std::collections::HashMap<String, SearchResult> = std::collections::HashMap::new();

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
        let mut deduplicated_results: Vec<SearchResult> = path_to_best_result.into_values().collect();

        // Calculate search time
        let search_time = start_time.elapsed();

        // Small delay to show the spinner (only if search was very fast)
        if search_time.as_millis() < 300 {
            thread::sleep(Duration::from_millis(300) - search_time);
        }

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
            total_matches: total_matches,  // Total matches before deduplication
            search_time_ms: search_time.as_millis() as u64,
            query_time: Utc::now(),
        };

        // Update state with results
        self.search_results = final_results;

        // Apply current sort settings
        self.sort_results();

        self.is_searching = false;
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

        // Start indexing in background thread
        std::thread::spawn(move || {
            if let Ok(indexer_lock) = indexer.lock() {
                match indexer_lock.index_directory(&directory_clone) {
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

        // Start indexing in background thread
        std::thread::spawn(move || {
            for directory in directories {
                if let Ok(indexer_lock) = indexer.lock() {
                    match indexer_lock.index_directory(&directory) {
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
        if let Ok(mut watcher) = self.file_watcher.lock() {
            // 只检测事件，不立即执行索引
            let directories_to_reindex = watcher.check_events();

            // 如果有需要重新索引的目录，启动后台线程处理
            if !directories_to_reindex.is_empty() {
                // 克隆必要的数据
                let indexer = self.indexer.clone();
                let directories = directories_to_reindex.clone();
                let stats_sender = self.stats_sender.clone();

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
                            if let Ok(stats) = indexer_lock.index_directory(&directory) {
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

/// Render the iSearch module
pub fn render_isearch(ui: &mut egui::Ui, state: &mut ISearchState) {
    // Process directory dialog
    state.process_directory_dialog();

    // Process file watcher events
    state.process_watcher_events();

    // Check for completed indexing operations (important for updating UI)
    state.check_reindex_results();

    // Use available_rect to get the actual available space
    let available_rect = ui.available_rect_before_wrap();
    let content_height = available_rect.height() - 20.0; // Reserve space for status bar and padding

    ui.allocate_ui_with_layout(
        egui::Vec2::new(available_rect.width(), content_height),
        egui::Layout::top_down(egui::Align::LEFT),
        |ui| {
        // Search bar
        ui.vertical(|ui| {
            let search_id = ui.make_persistent_id("search_input");

            ui.horizontal(|ui| {
                // Directory panel toggle button (only show if there are indexed directories)
                if !state.indexed_directories.is_empty() {
                    let toggle_text = if state.show_directories_panel { "📁 ▼" } else { "📁 ▶" };
                    if ui.button(toggle_text).on_hover_text("显示/隐藏索引目录").clicked() {
                        state.show_directories_panel = !state.show_directories_panel;
                    }
                }

                ui.label("🔍");
                let response = ui.add(
                    egui::TextEdit::singleline(&mut state.search_query)
                        .hint_text("搜索文件... (支持 filetype:pdf, filename:name, +必须, \"精确短语\")")
                        .desired_width(ui.available_width() - 100.0)
                        .id(search_id)
                );

                if ui.button("搜索").clicked() ||
                   (response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter))) {
                    state.search();
                }

                let help_button = ui.button("?").on_hover_text("点击查看高级搜索语法帮助");
                if help_button.clicked() {
                    ui.memory_mut(|mem| mem.toggle_popup(ui.make_persistent_id("search_syntax_help")));
                }
            });

            // Show popup with search syntax help
            let popup_id = ui.make_persistent_id("search_syntax_help");
            if ui.memory(|mem| mem.is_popup_open(popup_id)) {
                egui::Window::new("高级搜索语法")
                    .id(popup_id)
                    .fixed_size([400.0, 200.0])
                    .show(ui.ctx(), |ui| {
                        ui.heading("高级搜索语法");
                        ui.separator();
                        ui.label("支持以下高级搜索语法：");
                        ui.label("• filetype:pdf - 按文件类型筛选");
                        ui.label("• filename:report - 按文件名筛选");
                        ui.label("• +关键词 - 必须包含该关键词");
                        ui.label("• \"精确短语\" - 精确匹配短语");
                        ui.label("示例：project +important filetype:pdf \"quarterly report\"");
                    });
            }
        });

        ui.separator();

        // Main content - show directory info panel only if there are indexed directories and panel is enabled
        if !state.indexed_directories.is_empty() && state.show_directories_panel {
            egui::SidePanel::left("directories_panel")
                .resizable(true)
                .default_width(200.0)
                .show_inside(ui, |ui| {
                    ui.heading("索引目录");

                    ui.separator();

                    // Directory list with detailed info - full width
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        // Clone the directories to avoid borrowing issues
                        let directories = state.indexed_directories.clone();
                        let selected_directory = state.selected_directory;

                        for (i, directory) in directories.iter().enumerate() {
                            let is_selected = selected_directory == Some(i);

                            // Full width group for each directory
                            ui.allocate_ui_with_layout(
                                egui::Vec2::new(ui.available_width(), 0.0),
                                egui::Layout::top_down(egui::Align::LEFT),
                                |ui| {
                                    ui.group(|ui| {
                                        ui.set_min_width(ui.available_width());
                                        ui.vertical(|ui| {
                                            // Directory path with wrapping
                                            let path_text = format!("📁 {}", directory.path);
                                            ui.allocate_ui_with_layout(
                                                egui::Vec2::new(ui.available_width(), 0.0),
                                                egui::Layout::top_down(egui::Align::LEFT),
                                                |ui| {
                                                    if ui.selectable_label(is_selected, &path_text).clicked() {
                                                        state.selected_directory = Some(i);
                                                    }
                                                }
                                            );

                                            // Directory stats
                                            ui.horizontal(|ui| {
                                                ui.label(egui::RichText::new(format!("📄 {} 个文件", directory.file_count)).small().weak());
                                                ui.label(egui::RichText::new(format!("💾 {:.1} MB", directory.total_size_bytes as f64 / (1024.0 * 1024.0))).small().weak());
                                            });

                                            // Last indexed time
                                            if let Some(last_indexed) = directory.last_indexed {
                                                ui.label(egui::RichText::new(format!("🕒 {}", last_indexed.format("%m-%d %H:%M"))).small().weak());
                                            } else {
                                                ui.label(egui::RichText::new("🕒 未索引").small().weak());
                                            }

                                            // Update button for this directory
                                            ui.horizontal(|ui| {
                                                if ui.small_button("🔄 更新此目录").on_hover_text("重新索引此目录").clicked() {
                                                    state.update_directory_index(i);
                                                }
                                            });
                                        });
                                    });
                                }
                            );

                            ui.add_space(4.0);
                        }
                    });

                    // Index stats
                    ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                        ui.label(format!(
                            "已索引 {} 个文件 ({:.1} MB)",
                            state.index_stats.total_files,
                            state.index_stats.total_size_bytes as f64 / (1024.0 * 1024.0)
                        ));

                        if let Some(last_updated) = state.index_stats.last_updated {
                            ui.label(format!(
                                "最后更新: {}",
                                last_updated.format("%Y-%m-%d %H:%M")
                            ));
                        }

                        if state.is_indexing {
                            ui.horizontal(|ui| {
                                ui.spinner();
                                ui.label("正在索引...");
                            });
                        }

                        ui.separator();

                        // Update all indexes button
                        if ui.button("🔄 更新全部索引").on_hover_text("重新索引所有目录").clicked() {
                            state.update_all_indexes();
                        }

                        ui.add_space(5.0);
                        ui.label(egui::RichText::new("💡 在设置中管理索引目录").weak());
                    });
                });
        }

        egui::CentralPanel::default().show_inside(ui, |ui| {
            // Add a scroll area for the entire central panel content to prevent overflow
            let central_height = ui.available_height();
            egui::ScrollArea::vertical()
                .max_height(central_height)
                .auto_shrink([false, true])
                .show(ui, |ui| {
                    ui.heading("搜索结果");

                    // Check if there are no indexed directories
                    if state.indexed_directories.is_empty() {
                ui.centered_and_justified(|ui| {
                    ui.vertical_centered(|ui| {
                        ui.add_space(50.0);
                        ui.label(egui::RichText::new("📂").size(48.0));
                        ui.add_space(10.0);
                        ui.label(egui::RichText::new("暂无索引目录").heading());
                        ui.add_space(10.0);
                        ui.label("请先在设置中添加要搜索的目录");
                        ui.add_space(15.0);

                        if ui.button("🔧 前往设置").clicked() {
                            state.navigate_to_settings = true;
                            log::info!("Navigate to settings for directory indexing");
                        }

                        ui.add_space(20.0);
                        ui.separator();
                        ui.add_space(10.0);

                        ui.label(egui::RichText::new("💡 提示").strong());
                        ui.label("• 添加目录后系统会自动建立索引");
                        ui.label("• 支持多个目录同时索引");
                        ui.label("• 索引完成后即可进行快速搜索");
                    });
                });
            } else if state.search_query.is_empty() {
                ui.centered_and_justified(|ui| {
                    ui.vertical_centered(|ui| {
                        ui.add_space(20.0);
                        ui.label(egui::RichText::new("🔍").size(32.0));
                        ui.add_space(10.0);
                        ui.label(egui::RichText::new("开始搜索").heading());
                        ui.add_space(5.0);
                        ui.label("在上方搜索框中输入关键词开始搜索");
                    });
                });

                ui.add_space(20.0);
                ui.separator();
                ui.add_space(15.0);

                // Always show search syntax help when search is empty (not collapsible)
                ui.heading("🎯 高级搜索语法");
                ui.add_space(10.0);

                ui.group(|ui| {
                    ui.vertical(|ui| {
                        ui.label(egui::RichText::new("支持以下高级搜索语法：").strong());
                        ui.add_space(8.0);

                        // File type filtering
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("📄").size(16.0));
                            ui.label(egui::RichText::new("filetype:pdf").code());
                            ui.label("- 按文件类型筛选");
                        });

                        // Filename filtering
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("📝").size(16.0));
                            ui.label(egui::RichText::new("filename:report").code());
                            ui.label("- 按文件名筛选");
                        });

                        // Required keywords
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("✅").size(16.0));
                            ui.label(egui::RichText::new("+关键词").code());
                            ui.label("- 必须包含该关键词");
                        });

                        // Exact phrases
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("💬").size(16.0));
                            ui.label(egui::RichText::new("\"精确短语\"").code());
                            ui.label("- 精确匹配短语");
                        });

                        ui.add_space(10.0);
                        ui.separator();
                        ui.add_space(8.0);

                        // Example section
                        ui.label(egui::RichText::new("💡 示例：").strong());
                        ui.add_space(5.0);
                        ui.horizontal(|ui| {
                            ui.label("🔍");
                            ui.label(egui::RichText::new("project +important filetype:pdf \"quarterly report\"").code());
                        });
                        ui.label(egui::RichText::new("查找包含 'project' 和 'important' 的 PDF 文件，且包含精确短语 'quarterly report'").weak());
                    });
                });

            } else if state.search_results.is_empty() && !state.is_searching {
                ui.centered_and_justified(|ui| {
                    ui.label("未找到匹配结果");
                });
            } else if state.is_searching {
                ui.centered_and_justified(|ui| {
                    ui.spinner();
                    ui.label("正在搜索...");
                });
            } else {
                // Search statistics at the top
                ui.horizontal(|ui| {
                    if state.search_stats.total_matches > state.search_stats.total_results {
                        // Show both total matches and deduplicated results
                        ui.label(format!(
                            "找到 {} 个匹配（{} 个文件），耗时 {:.2} 秒",
                            state.search_stats.total_matches,
                            state.search_stats.total_results,
                            state.search_stats.search_time_ms as f64 / 1000.0
                        ));
                    } else {
                        // Just show the total results
                        ui.label(format!(
                            "找到 {} 个结果，耗时 {:.2} 秒",
                            state.search_stats.total_results,
                            state.search_stats.search_time_ms as f64 / 1000.0
                        ));
                    }

                    if state.has_more_results {
                        ui.label("(显示前 100 条结果)");
                    }
                });

                ui.separator();

                // Sort controls
                ui.horizontal(|ui| {
                    ui.label("排序：");

                    // Sort by buttons
                    let sort_options = [
                        SortBy::Relevance,
                        SortBy::FileName,
                        SortBy::DirectoryName,
                        SortBy::FileSize,
                        SortBy::ModifiedTime,
                    ];

                    for sort_option in &sort_options {
                        let is_current = state.sort_by == *sort_option;
                        let button_text = if is_current {
                            format!("{} {}", sort_option.display_name(), state.sort_order.icon())
                        } else {
                            sort_option.display_name().to_string()
                        };

                        let button = if is_current {
                            ui.add(egui::Button::new(button_text).fill(ui.visuals().selection.bg_fill))
                        } else {
                            ui.button(button_text)
                        };

                        if button.clicked() {
                            state.set_sort_by(sort_option.clone());
                        }
                    }
                });

                ui.add_space(5.0);
                ui.separator();

                // Search results with proper height constraint
                // Since we now have an outer scroll area, we can be more generous with the inner scroll area
                // but still need to reserve space for bottom statistics
                let remaining_height = ui.available_height() - 80.0; // Reserve space for statistics
                egui::ScrollArea::vertical()
                    .max_height(remaining_height.max(200.0)) // Ensure minimum height of 200px
                    .auto_shrink([false, true])
                    .show(ui, |ui| {
                    // Clone the results to avoid borrowing issues
                    let results = state.search_results.clone();
                    for result in &results {
                        ui.push_id(&result.id, |ui| {
                            // Create a frame for the search result item with hover effect
                            let frame_response = egui::Frame::NONE
                                .inner_margin(egui::Margin::same(8))
                                .corner_radius(egui::Rounding::same(4))
                                .show(ui, |ui| {
                                    // File name and icon
                                    ui.horizontal(|ui| {
                                        // File type icon
                                        let icon = match result.file_type.as_str() {
                                            "pdf" => "📄",
                                            "doc" | "docx" => "📝",
                                            "xls" | "xlsx" => "📊",
                                            "ppt" | "pptx" => "📽",
                                            "txt" | "md" => "📃",
                                            "rs" | "js" | "py" | "cpp" => "💻",
                                            "jpg" | "png" | "gif" => "🖼",
                                            _ => "📁",
                                        };
                                        ui.label(icon);

                                        // File name with truncation and copy button
                                        let truncated_filename = utils::truncate_with_ellipsis(&result.filename, 40);
                                        ui.heading(truncated_filename);
                                        if ui.small_button("📋").on_hover_text("复制文件名").clicked() {
                                            state.copy_filename(result);
                                        }

                                        // File size, date, and score
                                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                            ui.label(format!("{}", result.modified.format("%Y-%m-%d %H:%M")));
                                            ui.label(format!("{:.1} KB", result.size_bytes as f64 / 1024.0));
                                            // Uncomment the following line to display the score (for debugging)
                                            // ui.label(format!("得分: {:.2}", result.score));
                                        });
                                    });

                                    // File path with truncation and copy button
                                    ui.horizontal(|ui| {
                                        ui.label("📂");
                                        let truncated_path = utils::truncate_with_ellipsis(&result.path, 60);
                                        ui.label(truncated_path);
                                        if ui.small_button("📋").on_hover_text("复制完整路径").clicked() {
                                            state.copy_path_and_name(result);
                                        }
                                    });

                                    // Content preview with truncation (only if enabled)
                                    if state.enable_content_preview && !result.content_preview.is_empty() {
                                        ui.add_space(4.0);
                                        // We'll still use wrap(true) but truncate to a reasonable length first
                                        // to prevent extremely long lines from causing layout issues
                                        let truncated_preview = utils::truncate_with_ellipsis(&result.content_preview, 300);
                                        ui.add(egui::Label::new(truncated_preview).wrap());
                                    }

                                    // Action buttons
                                    ui.horizontal(|ui| {
                                        // Open file button
                                        if ui.button("打开文件").clicked() {
                                            let path = result.path.clone();
                                            state.open_file(&path);
                                        }

                                        // Open folder button
                                        if ui.button("打开文件夹").clicked() {
                                            let path = result.path.clone();
                                            state.open_folder(&path);
                                        }

                                        // Add space to push the menu button to the right
                                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                            // Context menu button - use a more reliable approach
                                            ui.menu_button("...", |ui| {
                                                ui.set_min_width(150.0);

                                                // Properties button (first)
                                                if ui.button("📋 属性").clicked() {
                                                    state.show_file_properties(result);
                                                    ui.close_menu();
                                                }

                                                ui.separator();

                                                // Copy submenu (second)
                                                ui.menu_button("📄 复制", |ui| {
                                                    if ui.button("名称").clicked() {
                                                        state.copy_filename(result);
                                                        ui.close_menu();
                                                    }
                                                    if ui.button("路径").clicked() {
                                                        state.copy_filepath(result);
                                                        ui.close_menu();
                                                    }
                                                    if ui.button("路径+名称").clicked() {
                                                        state.copy_path_and_name(result);
                                                        ui.close_menu();
                                                    }
                                                });
                                            });
                                        });
                                    });
                                });

                            // Add hover effect by painting a background when hovered
                            if frame_response.response.hovered() {
                                let rect = frame_response.response.rect;
                                let hover_color = if ui.visuals().dark_mode {
                                    egui::Color32::from_rgba_unmultiplied(100, 150, 255, 25) // Blue overlay for dark mode
                                } else {
                                    egui::Color32::from_rgba_unmultiplied(50, 100, 200, 20) // Blue overlay for light mode
                                };
                                ui.painter().rect_filled(rect, egui::Rounding::same(4), hover_color);

                                // Add a subtle border when hovered
                                let border_color = if ui.visuals().dark_mode {
                                    egui::Color32::from_rgba_unmultiplied(150, 180, 255, 60)
                                } else {
                                    egui::Color32::from_rgba_unmultiplied(80, 120, 220, 80)
                                };
                                ui.painter().rect_stroke(rect, egui::Rounding::same(4), egui::Stroke::new(1.0, border_color), egui::StrokeKind::Outside);
                            }

                            ui.add_space(4.0);
                            ui.separator();
                        });
                    }

                    // Add some spacing before bottom statistics
                    ui.add_space(10.0);

                    // Search statistics at the bottom - compact layout
                    ui.horizontal(|ui| {
                        if state.search_stats.total_matches > state.search_stats.total_results {
                            // Show both total matches and deduplicated results
                            ui.label(egui::RichText::new(format!(
                                "找到 {} 个匹配（{} 个文件），耗时 {:.2} 秒",
                                state.search_stats.total_matches,
                                state.search_stats.total_results,
                                state.search_stats.search_time_ms as f64 / 1000.0
                            )).small());
                        } else {
                            // Just show the total results
                            ui.label(egui::RichText::new(format!(
                                "找到 {} 个结果，耗时 {:.2} 秒",
                                state.search_stats.total_results,
                                state.search_stats.search_time_ms as f64 / 1000.0
                            )).small());
                        }

                        if state.has_more_results {
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                ui.label(egui::RichText::new("请使用更精确的搜索条件缩小结果范围").small().weak());
                            });
                        }
                    });

                    // Show query time - compact
                    ui.label(egui::RichText::new(format!(
                        "查询时间: {}",
                        state.search_stats.query_time.format("%Y-%m-%d %H:%M:%S")
                    )).small().weak());
                });
            }
                });
        });
        }
    );

    // Show file properties dialog if requested
    if state.show_properties_dialog {
        if let Some(file) = &state.properties_file.clone() {
            let file_path = file.path.clone();
            egui::Window::new("📋 文件属性")
                .collapsible(false)
                .resizable(false)
                .fixed_size([450.0, 500.0])
                .show(ui.ctx(), |ui| {
                    ui.vertical(|ui| {
                        ui.add_space(10.0);

                        // File icon and name with copy button
                        ui.horizontal(|ui| {
                            let icon = match file.file_type.as_str() {
                                "pdf" => "📄",
                                "doc" | "docx" => "📝",
                                "xls" | "xlsx" => "📊",
                                "ppt" | "pptx" => "📽",
                                "txt" | "md" => "📃",
                                "rs" | "js" | "py" | "cpp" => "💻",
                                "jpg" | "png" | "gif" => "🖼",
                                _ => "📁",
                            };
                            ui.label(egui::RichText::new(icon).size(24.0));
                            ui.add_space(8.0);

                            // File name with wrapping for long names
                            ui.vertical(|ui| {
                                ui.horizontal(|ui| {
                                    ui.label(egui::RichText::new(&file.filename).heading());
                                    if ui.small_button("📋").on_hover_text("复制文件名").clicked() {
                                        state.copy_filename(file);
                                    }
                                });
                            });
                        });

                        ui.add_space(15.0);

                        // Properties grid with copy buttons
                        egui::Grid::new("file_properties")
                            .num_columns(3)
                            .spacing([10.0, 8.0])
                            .show(ui, |ui| {
                                ui.label(egui::RichText::new("名称:").strong());
                                ui.add(egui::Label::new(&file.filename).wrap());
                                if ui.small_button("📋").on_hover_text("复制文件名").clicked() {
                                    state.copy_filename(file);
                                }
                                ui.end_row();

                                ui.label(egui::RichText::new("路径:").strong());
                                ui.add(egui::Label::new(&file.path).wrap());
                                if ui.small_button("📋").on_hover_text("复制完整路径").clicked() {
                                    state.copy_path_and_name(file);
                                }
                                ui.end_row();

                                ui.label(egui::RichText::new("类型:").strong());
                                ui.label(&file.file_type.to_uppercase());
                                ui.label(""); // Empty cell for alignment
                                ui.end_row();

                                ui.label(egui::RichText::new("大小:").strong());
                                ui.label(format!("{:.1} KB ({} 字节)",
                                    file.size_bytes as f64 / 1024.0,
                                    file.size_bytes));
                                ui.label(""); // Empty cell for alignment
                                ui.end_row();

                                ui.label(egui::RichText::new("修改时间:").strong());
                                ui.label(file.modified.format("%Y-%m-%d %H:%M:%S").to_string());
                                ui.label(""); // Empty cell for alignment
                                ui.end_row();

                                ui.label(egui::RichText::new("搜索评分:").strong());
                                ui.label(format!("{:.2}", file.score));
                                ui.label(""); // Empty cell for alignment
                                ui.end_row();
                            });

                        ui.add_space(15.0);

                        // Content preview if available and enabled
                        if state.enable_content_preview && !file.content_preview.is_empty() {
                            ui.label(egui::RichText::new("内容预览:").strong());
                            ui.add_space(5.0);

                            egui::ScrollArea::vertical()
                                .max_height(100.0)
                                .show(ui, |ui| {
                                    ui.add(egui::Label::new(&file.content_preview).wrap());
                                });

                            ui.add_space(15.0);
                        }

                        // Action buttons
                        ui.horizontal(|ui| {
                            if ui.button("📂 打开文件夹").clicked() {
                                state.open_folder(&file_path);
                            }

                            if ui.button("📄 打开文件").clicked() {
                                state.open_file(&file_path);
                            }

                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if ui.button("关闭").clicked() {
                                    state.show_properties_dialog = false;
                                    state.properties_file = None;
                                }
                            });
                        });

                        ui.add_space(5.0);
                    });
                });
        }

    // Directory input dialog (替代文件对话框)
    if state.show_directory_input_dialog {
        egui::Window::new("添加索引目录")
            .collapsible(false)
            .resizable(false)
            .default_width(400.0)
            .show(ui.ctx(), |ui| {
                ui.vertical(|ui| {
                    ui.label("请输入要索引的目录路径：");
                    ui.add_space(5.0);

                    let response = ui.add(
                        egui::TextEdit::singleline(&mut state.directory_input)
                            .hint_text("例如：/home/user/Documents")
                            .desired_width(ui.available_width())
                    );

                    // 自动聚焦输入框
                    if state.show_directory_input_dialog {
                        response.request_focus();
                    }

                    ui.add_space(10.0);

                    // 常用目录快捷按钮
                    ui.label("常用目录：");
                    ui.horizontal_wrapped(|ui| {
                        if let Some(home_dir) = dirs::home_dir() {
                            if ui.small_button("🏠 主目录").clicked() {
                                state.directory_input = home_dir.to_string_lossy().to_string();
                            }
                        }

                        if let Some(documents_dir) = dirs::document_dir() {
                            if ui.small_button("📄 文档").clicked() {
                                state.directory_input = documents_dir.to_string_lossy().to_string();
                            }
                        }

                        if let Some(downloads_dir) = dirs::download_dir() {
                            if ui.small_button("📥 下载").clicked() {
                                state.directory_input = downloads_dir.to_string_lossy().to_string();
                            }
                        }

                        if let Some(desktop_dir) = dirs::desktop_dir() {
                            if ui.small_button("🖥 桌面").clicked() {
                                state.directory_input = desktop_dir.to_string_lossy().to_string();
                            }
                        }
                    });

                    ui.add_space(10.0);
                    ui.separator();
                    ui.add_space(5.0);

                    // 按钮
                    ui.horizontal(|ui| {
                        if ui.button("添加").clicked() ||
                           (response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter))) {
                            state.add_directory_from_input();
                        }

                        if ui.button("取消").clicked() {
                            state.show_directory_input_dialog = false;
                            state.directory_input.clear();
                        }
                    });
                });
            });
    }
    }
}
