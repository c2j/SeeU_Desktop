use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::time::{Duration, Instant, SystemTime};
use std::fs;
use notify::{Watcher, RecursiveMode, Result, Event, EventKind};
// use chrono::{DateTime, Utc}; // Commented out unused imports
use crate::IndexedDirectory;
use crate::indexer::Indexer;

/// Enhanced file system watcher with incremental updates
pub struct EnhancedFileWatcher {
    watcher: Option<Arc<Mutex<Box<dyn Watcher + Send>>>>,
    event_receiver: Option<Receiver<Result<Event>>>,
    watched_directories: HashMap<String, WatchedDirectory>,
    indexer: Arc<Mutex<Indexer>>,
    
    // Change tracking
    pending_changes: Arc<Mutex<HashMap<String, FileChange>>>,
    change_batch_timeout: Duration,
    last_batch_process: Instant,
    
    // Incremental update communication
    update_sender: Option<Sender<IncrementalUpdate>>,
    update_receiver: Option<Receiver<IncrementalUpdate>>,
    
    // Configuration
    debounce_duration: Duration,
    max_batch_size: usize,
    ignore_patterns: Vec<String>,
}

/// Information about a watched directory
#[derive(Debug, Clone)]
struct WatchedDirectory {
    path: PathBuf,
    last_scan: Instant,
    file_count: usize,
    total_size: u64,
    last_modified: Option<SystemTime>,
}

/// File change information
#[derive(Debug, Clone)]
pub struct FileChange {
    pub path: PathBuf,
    pub change_type: ChangeType,
    pub timestamp: Instant,
    pub size: Option<u64>,
    pub modified: Option<SystemTime>,
}

/// Type of file change
#[derive(Debug, Clone, PartialEq)]
pub enum ChangeType {
    Created,
    Modified,
    Deleted,
    Renamed { from: PathBuf, to: PathBuf },
}

/// Incremental update information
#[derive(Debug, Clone)]
pub struct IncrementalUpdate {
    pub directory_path: String,
    pub changes: Vec<FileChange>,
    pub batch_id: u64,
}

/// Update result
#[derive(Debug, Clone)]
pub struct UpdateResult {
    pub directory_path: String,
    pub batch_id: u64,
    pub success: bool,
    pub processed_files: usize,
    pub added_files: usize,
    pub updated_files: usize,
    pub removed_files: usize,
    pub error_message: Option<String>,
}

impl EnhancedFileWatcher {
    /// Create a new enhanced file watcher
    pub fn new(indexer: Arc<Mutex<Indexer>>) -> Self {
        let (update_sender, update_receiver) = channel();
        
        Self {
            watcher: None,
            event_receiver: None,
            watched_directories: HashMap::new(),
            indexer,
            pending_changes: Arc::new(Mutex::new(HashMap::new())),
            change_batch_timeout: Duration::from_secs(5), // Process changes every 5 seconds
            last_batch_process: Instant::now(),
            update_sender: Some(update_sender),
            update_receiver: Some(update_receiver),
            debounce_duration: Duration::from_millis(500), // 500ms debounce
            max_batch_size: 100, // Process up to 100 changes at once
            ignore_patterns: vec![
                ".git".to_string(),
                ".svn".to_string(),
                ".hg".to_string(),
                "node_modules".to_string(),
                ".DS_Store".to_string(),
                "Thumbs.db".to_string(),
                "*.tmp".to_string(),
                "*.temp".to_string(),
                "*.swp".to_string(),
                "*.lock".to_string(),
            ],
        }
    }

    /// Set debounce duration for file change detection
    pub fn set_debounce_duration(&mut self, duration: Duration) {
        self.debounce_duration = duration;
    }

    /// Set maximum batch size for processing changes
    pub fn set_max_batch_size(&mut self, size: usize) {
        self.max_batch_size = size;
    }

    /// Add ignore pattern for files/directories to ignore
    pub fn add_ignore_pattern(&mut self, pattern: String) {
        self.ignore_patterns.push(pattern);
    }

    /// Start watching a directory with enhanced monitoring
    pub fn watch_directory(&mut self, directory: &IndexedDirectory) -> Result<()> {
        let path = Path::new(&directory.path);

        if !path.exists() || !path.is_dir() {
            return Err(notify::Error::generic("Directory does not exist"));
        }

        // Create watcher if not exists
        if self.event_receiver.is_none() {
            let (tx, rx) = channel();
            let watcher = notify::recommended_watcher(tx)?;
            
            self.watcher = Some(Arc::new(Mutex::new(Box::new(watcher))));
            self.event_receiver = Some(rx);
        }

        // Start watching the directory
        if let Some(watcher_arc) = &self.watcher {
            if let Ok(mut watcher) = watcher_arc.lock() {
                watcher.watch(path, RecursiveMode::Recursive)?;
                
                // Initialize directory info
                let watched_dir = WatchedDirectory {
                    path: path.to_path_buf(),
                    last_scan: Instant::now(),
                    file_count: 0,
                    total_size: 0,
                    last_modified: None,
                };
                
                self.watched_directories.insert(directory.path.clone(), watched_dir);
                log::info!("Started enhanced watching for directory: {}", directory.path);
            }
        }

        Ok(())
    }

    /// Stop watching a directory
    pub fn unwatch_directory(&mut self, directory_path: &str) -> Result<()> {
        if let Some(watched_dir) = self.watched_directories.remove(directory_path) {
            if let Some(watcher_arc) = &self.watcher {
                if let Ok(mut watcher) = watcher_arc.lock() {
                    watcher.unwatch(&watched_dir.path)?;
                    log::info!("Stopped watching directory: {}", directory_path);
                }
            }
        }
        Ok(())
    }

    /// Process file system events and detect changes
    pub fn process_events(&mut self) -> Vec<UpdateResult> {
        let mut results = Vec::new();
        
        // Process file system events
        self.collect_file_changes();
        
        // Process batched changes if timeout reached or batch is full
        if self.should_process_batch() {
            results.extend(self.process_pending_changes());
        }
        
        // Get completed update results
        results.extend(self.get_update_results());
        
        results
    }

    /// Check if we should process the current batch of changes
    fn should_process_batch(&self) -> bool {
        let batch_timeout_reached = self.last_batch_process.elapsed() >= self.change_batch_timeout;
        
        let batch_size_reached = if let Ok(pending) = self.pending_changes.lock() {
            pending.len() >= self.max_batch_size
        } else {
            false
        };
        
        batch_timeout_reached || batch_size_reached
    }

    /// Collect file system events and convert to file changes
    fn collect_file_changes(&mut self) {
        if let Some(receiver) = &self.event_receiver {
            while let Ok(Ok(event)) = receiver.try_recv() {
                self.process_file_event(event);
            }
        }
    }

    /// Process a single file system event
    fn process_file_event(&self, event: Event) {
        for path in &event.paths {
            // Skip ignored files/directories
            if self.should_ignore_path(path) {
                continue;
            }

            // Find which directory this change belongs to
            let directory_path = self.find_containing_directory(path);
            if directory_path.is_none() {
                continue;
            }

            let change_type = match event.kind {
                EventKind::Create(_) => ChangeType::Created,
                EventKind::Modify(_) => ChangeType::Modified,
                EventKind::Remove(_) => ChangeType::Deleted,
                _ => continue, // Ignore other event types
            };

            let file_change = FileChange {
                path: path.clone(),
                change_type,
                timestamp: Instant::now(),
                size: self.get_file_size(path),
                modified: self.get_file_modified_time(path),
            };

            // Add to pending changes with debouncing
            if let Ok(mut pending) = self.pending_changes.lock() {
                let key = path.to_string_lossy().to_string();
                
                // Check if we should debounce this change
                if let Some(existing) = pending.get(&key) {
                    if existing.timestamp.elapsed() < self.debounce_duration {
                        continue; // Skip this change due to debouncing
                    }
                }
                
                pending.insert(key, file_change);
            }
        }
    }

    /// Check if a path should be ignored
    fn should_ignore_path(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy();
        
        for pattern in &self.ignore_patterns {
            if pattern.contains('*') {
                // Simple wildcard matching
                if self.matches_wildcard(&path_str, pattern) {
                    return true;
                }
            } else if path_str.contains(pattern) {
                return true;
            }
        }
        
        false
    }

    /// Simple wildcard pattern matching
    fn matches_wildcard(&self, text: &str, pattern: &str) -> bool {
        // Very basic wildcard matching - could be improved
        if pattern == "*" {
            return true;
        }
        
        if let Some(star_pos) = pattern.find('*') {
            let prefix = &pattern[..star_pos];
            let suffix = &pattern[star_pos + 1..];
            
            text.starts_with(prefix) && text.ends_with(suffix)
        } else {
            text == pattern
        }
    }

    /// Find which watched directory contains the given path
    fn find_containing_directory(&self, path: &Path) -> Option<String> {
        for (dir_path, _) in &self.watched_directories {
            if path.starts_with(dir_path) {
                return Some(dir_path.clone());
            }
        }
        None
    }

    /// Get file size safely
    fn get_file_size(&self, path: &Path) -> Option<u64> {
        fs::metadata(path).ok().map(|m| m.len())
    }

    /// Get file modified time safely
    fn get_file_modified_time(&self, path: &Path) -> Option<SystemTime> {
        fs::metadata(path).ok().and_then(|m| m.modified().ok())
    }

    /// Process all pending changes
    fn process_pending_changes(&mut self) -> Vec<UpdateResult> {
        let results = Vec::new();
        
        if let Ok(mut pending) = self.pending_changes.lock() {
            if pending.is_empty() {
                return results;
            }

            // Group changes by directory
            let mut changes_by_dir: HashMap<String, Vec<FileChange>> = HashMap::new();
            
            for (_, change) in pending.drain() {
                if let Some(dir_path) = self.find_containing_directory(&change.path) {
                    changes_by_dir.entry(dir_path).or_insert_with(Vec::new).push(change);
                }
            }

            // Submit incremental updates for each directory
            for (dir_path, changes) in changes_by_dir {
                if let Some(sender) = &self.update_sender {
                    let update = IncrementalUpdate {
                        directory_path: dir_path,
                        changes,
                        batch_id: self.generate_batch_id(),
                    };
                    
                    if sender.send(update).is_err() {
                        log::error!("Failed to send incremental update");
                    }
                }
            }
            
            self.last_batch_process = Instant::now();
        }
        
        results
    }

    /// Generate a unique batch ID
    fn generate_batch_id(&self) -> u64 {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos() as u64
    }

    /// Get completed update results
    fn get_update_results(&self) -> Vec<UpdateResult> {
        let mut results = Vec::new();
        
        if let Some(receiver) = &self.update_receiver {
            while let Ok(result) = receiver.try_recv() {
                // Convert IncrementalUpdate to UpdateResult
                // This is a placeholder - actual implementation would process the update
                let update_result = UpdateResult {
                    directory_path: result.directory_path,
                    batch_id: result.batch_id,
                    success: true,
                    processed_files: result.changes.len(),
                    added_files: result.changes.iter().filter(|c| c.change_type == ChangeType::Created).count(),
                    updated_files: result.changes.iter().filter(|c| c.change_type == ChangeType::Modified).count(),
                    removed_files: result.changes.iter().filter(|c| c.change_type == ChangeType::Deleted).count(),
                    error_message: None,
                };
                
                results.push(update_result);
            }
        }
        
        results
    }

    /// Get statistics about watched directories
    pub fn get_watch_statistics(&self) -> HashMap<String, WatchStatistics> {
        let mut stats = HashMap::new();
        
        for (path, watched_dir) in &self.watched_directories {
            let stat = WatchStatistics {
                directory_path: path.clone(),
                file_count: watched_dir.file_count,
                total_size: watched_dir.total_size,
                last_scan: watched_dir.last_scan,
                is_active: true,
            };
            stats.insert(path.clone(), stat);
        }
        
        stats
    }
}

/// Statistics for a watched directory
#[derive(Debug, Clone)]
pub struct WatchStatistics {
    pub directory_path: String,
    pub file_count: usize,
    pub total_size: u64,
    pub last_scan: Instant,
    pub is_active: bool,
}
