use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use notify::{Watcher, RecursiveMode, Result, Event, EventKind};
use crate::IndexedDirectory;
use crate::indexer::Indexer;

/// File system watcher
pub struct FileWatcher {
    watcher: Option<Arc<Mutex<Box<dyn Watcher + Send>>>>,
    receiver: Option<Receiver<Result<Event>>>,
    watched_directories: HashMap<String, PathBuf>,
    last_reindex_time: HashMap<String, Instant>,
    _indexer: Arc<Mutex<Indexer>>, // Keep for future use
    reindex_cooldown: Duration,
}

impl FileWatcher {
    /// Create a new file watcher
    pub fn new(indexer: Arc<Mutex<Indexer>>) -> Self {
        Self {
            watcher: None,
            receiver: None,
            watched_directories: HashMap::new(),
            last_reindex_time: HashMap::new(),
            _indexer: indexer,
            reindex_cooldown: Duration::from_secs(30), // Reindex at most once every 30 seconds
        }
    }

    /// Start watching a directory
    pub fn watch_directory(&mut self, directory: &IndexedDirectory) -> Result<()> {
        let path = Path::new(&directory.path);

        if !path.exists() || !path.is_dir() {
            return Err(notify::Error::generic("Directory does not exist"));
        }

        // Create a channel to receive events if not already created
        if self.receiver.is_none() {
            let (tx, rx) = channel();

            // Create a watcher
            let watcher = notify::recommended_watcher(tx)?;

            // Store the watcher and receiver
            self.watcher = Some(Arc::new(Mutex::new(Box::new(watcher))));
            self.receiver = Some(rx);
        }

        // Start watching the directory
        if let Some(watcher_arc) = &self.watcher {
            if let Ok(mut watcher) = watcher_arc.lock() {
                watcher.watch(path, RecursiveMode::Recursive)?;

                // Store the watched directory
                self.watched_directories.insert(directory.path.clone(), path.to_path_buf());
                self.last_reindex_time.insert(directory.path.clone(), Instant::now());

                log::info!("Started watching directory: {}", directory.path);
            }
        }

        Ok(())
    }

    /// Stop watching a directory
    pub fn unwatch_directory(&mut self, directory_path: &str) -> Result<()> {
        if let Some(path) = self.watched_directories.get(directory_path) {
            if let Some(watcher_arc) = &self.watcher {
                if let Ok(mut watcher) = watcher_arc.lock() {
                    watcher.unwatch(path)?;

                    // Remove from watched directories
                    self.watched_directories.remove(directory_path);
                    self.last_reindex_time.remove(directory_path);

                    log::info!("Stopped watching directory: {}", directory_path);
                }
            }
        }

        Ok(())
    }

    /// Check for file system events and return directories that need reindexing
    pub fn check_events(&mut self) -> Vec<String> {
        let mut directories_to_reindex = Vec::new();
        let mut event_dirs = HashMap::new();

        // Check for events
        if let Some(rx) = &self.receiver {
            // Non-blocking check for events
            while let Ok(Ok(event)) = rx.try_recv() {
                // Only care about create, modify, remove events
                match event.kind {
                    EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_) => {
                        // Find which directory this event belongs to
                        for (dir_path, _) in &self.watched_directories {
                            for path in &event.paths {
                                if path.starts_with(dir_path) {
                                    // Mark this directory for reindexing
                                    event_dirs.insert(dir_path.clone(), true);
                                    break;
                                }
                            }
                        }
                    },
                    _ => {}
                }
            }
        }

        // Check cooldown for each directory
        let now = Instant::now();
        for (dir_path, _) in event_dirs {
            if let Some(last_time) = self.last_reindex_time.get(&dir_path) {
                if now.duration_since(*last_time) < self.reindex_cooldown {
                    // Skip reindexing if cooldown hasn't elapsed
                    continue;
                }
            }

            // Update last reindex time
            self.last_reindex_time.insert(dir_path.clone(), now);
            directories_to_reindex.push(dir_path);
        }

        directories_to_reindex
    }

    /// Check if a directory is being watched
    pub fn is_watching(&self, directory_path: &str) -> bool {
        self.watched_directories.contains_key(directory_path)
    }

    /// Get the list of watched directories
    pub fn get_watched_directories(&self) -> Vec<String> {
        self.watched_directories.keys().cloned().collect()
    }
}
