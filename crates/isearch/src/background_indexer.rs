use std::sync::{Arc, Mutex};
use std::sync::mpsc::{channel, Sender, Receiver};
use std::thread;
use std::time::{Duration, Instant};
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use crate::{IndexedDirectory, IndexStats};
use crate::indexer::Indexer;

/// Background indexing manager
pub struct BackgroundIndexer {
    indexer: Arc<Mutex<Indexer>>,
    idle_threshold: Duration,
    last_activity: Arc<Mutex<Instant>>,
    is_running: Arc<Mutex<bool>>,
    update_sender: Option<Sender<BackgroundIndexCommand>>,
    result_receiver: Option<Receiver<BackgroundIndexResult>>,
    pending_updates: Arc<Mutex<HashMap<String, DateTime<Utc>>>>,
}

/// Commands for background indexing
#[derive(Debug, Clone)]
pub enum BackgroundIndexCommand {
    UpdateDirectory(IndexedDirectory),
    UpdateAllDirectories(Vec<IndexedDirectory>),
    CheckForUpdates,
    Stop,
}

/// Results from background indexing
#[derive(Debug, Clone)]
pub struct BackgroundIndexResult {
    pub directory_path: String,
    pub stats: IndexStats,
    pub success: bool,
    pub error_message: Option<String>,
}

/// System activity monitor
pub struct ActivityMonitor {
    last_activity: Arc<Mutex<Instant>>,
    monitoring: Arc<Mutex<bool>>,
}

impl BackgroundIndexer {
    /// Create a new background indexer
    pub fn new(indexer: Arc<Mutex<Indexer>>) -> Self {
        let (update_sender, update_receiver) = channel();
        let (result_sender, result_receiver) = channel();
        
        let background_indexer = Self {
            indexer: indexer.clone(),
            idle_threshold: Duration::from_secs(300), // 5 minutes default
            last_activity: Arc::new(Mutex::new(Instant::now())),
            is_running: Arc::new(Mutex::new(false)),
            update_sender: Some(update_sender),
            result_receiver: Some(result_receiver),
            pending_updates: Arc::new(Mutex::new(HashMap::new())),
        };

        // Start background thread
        background_indexer.start_background_thread(update_receiver, result_sender);
        
        background_indexer
    }

    /// Set the idle threshold for automatic updates
    pub fn set_idle_threshold(&mut self, threshold: Duration) {
        self.idle_threshold = threshold;
    }

    /// Record user activity to reset idle timer
    pub fn record_activity(&self) {
        if let Ok(mut last_activity) = self.last_activity.lock() {
            *last_activity = Instant::now();
        }
    }

    /// Check if system is idle
    pub fn is_idle(&self) -> bool {
        if let Ok(last_activity) = self.last_activity.lock() {
            last_activity.elapsed() >= self.idle_threshold
        } else {
            false
        }
    }

    /// Schedule a directory for background update
    pub fn schedule_directory_update(&self, directory: IndexedDirectory) {
        if let Some(sender) = &self.update_sender {
            let _ = sender.send(BackgroundIndexCommand::UpdateDirectory(directory));
        }
    }

    /// Schedule all directories for background update
    pub fn schedule_all_directories_update(&self, directories: Vec<IndexedDirectory>) {
        if let Some(sender) = &self.update_sender {
            let _ = sender.send(BackgroundIndexCommand::UpdateAllDirectories(directories));
        }
    }

    /// Check for pending updates and process them if system is idle
    pub fn check_for_updates(&self) {
        if let Some(sender) = &self.update_sender {
            let _ = sender.send(BackgroundIndexCommand::CheckForUpdates);
        }
    }

    /// Get results from background indexing
    pub fn get_results(&self) -> Vec<BackgroundIndexResult> {
        let mut results = Vec::new();
        if let Some(receiver) = &self.result_receiver {
            while let Ok(result) = receiver.try_recv() {
                results.push(result);
            }
        }
        results
    }

    /// Stop background indexing
    pub fn stop(&self) {
        if let Some(sender) = &self.update_sender {
            let _ = sender.send(BackgroundIndexCommand::Stop);
        }
        if let Ok(mut is_running) = self.is_running.lock() {
            *is_running = false;
        }
    }

    /// Start the background indexing thread
    fn start_background_thread(
        &self,
        update_receiver: Receiver<BackgroundIndexCommand>,
        result_sender: Sender<BackgroundIndexResult>,
    ) {
        let indexer = self.indexer.clone();
        let is_running = self.is_running.clone();
        let last_activity = self.last_activity.clone();
        let idle_threshold = self.idle_threshold;
        let pending_updates = self.pending_updates.clone();

        // Set running flag
        if let Ok(mut running) = is_running.lock() {
            *running = true;
        }

        thread::spawn(move || {
            log::info!("Background indexer thread started");
            
            while let Ok(running) = is_running.lock() {
                if !*running {
                    break;
                }
                drop(running);

                // Check for commands with timeout
                match update_receiver.recv_timeout(Duration::from_secs(30)) {
                    Ok(command) => {
                        match command {
                            BackgroundIndexCommand::UpdateDirectory(directory) => {
                                Self::handle_directory_update(
                                    &indexer,
                                    &result_sender,
                                    directory,
                                    &last_activity,
                                    idle_threshold,
                                );
                            }
                            BackgroundIndexCommand::UpdateAllDirectories(directories) => {
                                for directory in directories {
                                    Self::handle_directory_update(
                                        &indexer,
                                        &result_sender,
                                        directory,
                                        &last_activity,
                                        idle_threshold,
                                    );
                                }
                            }
                            BackgroundIndexCommand::CheckForUpdates => {
                                Self::handle_check_for_updates(
                                    &indexer,
                                    &result_sender,
                                    &pending_updates,
                                    &last_activity,
                                    idle_threshold,
                                );
                            }
                            BackgroundIndexCommand::Stop => {
                                log::info!("Background indexer received stop command");
                                break;
                            }
                        }
                    }
                    Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                        // Periodic check for pending updates
                        Self::handle_check_for_updates(
                            &indexer,
                            &result_sender,
                            &pending_updates,
                            &last_activity,
                            idle_threshold,
                        );
                    }
                    Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                        log::info!("Background indexer channel disconnected");
                        break;
                    }
                }
            }

            log::info!("Background indexer thread stopped");
        });
    }

    /// Handle directory update in background thread
    fn handle_directory_update(
        indexer: &Arc<Mutex<Indexer>>,
        result_sender: &Sender<BackgroundIndexResult>,
        directory: IndexedDirectory,
        last_activity: &Arc<Mutex<Instant>>,
        idle_threshold: Duration,
    ) {
        // Check if system is idle before starting update
        let is_idle = if let Ok(last_activity) = last_activity.lock() {
            last_activity.elapsed() >= idle_threshold
        } else {
            false
        };

        if !is_idle {
            log::debug!("System not idle, skipping background update for: {}", directory.path);
            return;
        }

        log::info!("Starting background update for directory: {}", directory.path);
        
        let result = match indexer.lock() {
            Ok(indexer_lock) => {
                match indexer_lock.index_directory(&directory) {
                    Ok(stats) => BackgroundIndexResult {
                        directory_path: directory.path.clone(),
                        stats,
                        success: true,
                        error_message: None,
                    },
                    Err(e) => BackgroundIndexResult {
                        directory_path: directory.path.clone(),
                        stats: IndexStats {
                            total_files: 0,
                            total_size_bytes: 0,
                            last_updated: Some(Utc::now()),
                        },
                        success: false,
                        error_message: Some(e.to_string()),
                    },
                }
            }
            Err(e) => BackgroundIndexResult {
                directory_path: directory.path.clone(),
                stats: IndexStats {
                    total_files: 0,
                    total_size_bytes: 0,
                    last_updated: Some(Utc::now()),
                },
                success: false,
                error_message: Some(format!("Failed to lock indexer: {}", e)),
            },
        };

        if result.success {
            log::info!("Background update completed for: {}", directory.path);
        } else {
            log::error!("Background update failed for {}: {:?}", directory.path, result.error_message);
        }

        let _ = result_sender.send(result);
    }

    /// Handle periodic check for updates
    fn handle_check_for_updates(
        _indexer: &Arc<Mutex<Indexer>>,
        _result_sender: &Sender<BackgroundIndexResult>,
        _pending_updates: &Arc<Mutex<HashMap<String, DateTime<Utc>>>>,
        last_activity: &Arc<Mutex<Instant>>,
        idle_threshold: Duration,
    ) {
        // Check if system is idle
        let is_idle = if let Ok(last_activity) = last_activity.lock() {
            last_activity.elapsed() >= idle_threshold
        } else {
            false
        };

        if is_idle {
            log::debug!("System is idle, checking for pending updates");
            // TODO: Implement logic to check for file system changes and schedule updates
        }
    }
}

impl ActivityMonitor {
    /// Create a new activity monitor
    pub fn new() -> Self {
        Self {
            last_activity: Arc::new(Mutex::new(Instant::now())),
            monitoring: Arc::new(Mutex::new(false)),
        }
    }

    /// Start monitoring system activity
    pub fn start_monitoring(&self) {
        if let Ok(mut monitoring) = self.monitoring.lock() {
            *monitoring = true;
        }

        // TODO: Implement platform-specific activity monitoring
        // For now, we'll rely on manual activity recording
    }

    /// Stop monitoring system activity
    pub fn stop_monitoring(&self) {
        if let Ok(mut monitoring) = self.monitoring.lock() {
            *monitoring = false;
        }
    }

    /// Record activity
    pub fn record_activity(&self) {
        if let Ok(mut last_activity) = self.last_activity.lock() {
            *last_activity = Instant::now();
        }
    }

    /// Check if system is idle
    pub fn is_idle(&self, threshold: Duration) -> bool {
        if let Ok(last_activity) = self.last_activity.lock() {
            last_activity.elapsed() >= threshold
        } else {
            false
        }
    }

    /// Get time since last activity
    pub fn time_since_last_activity(&self) -> Option<Duration> {
        if let Ok(last_activity) = self.last_activity.lock() {
            Some(last_activity.elapsed())
        } else {
            None
        }
    }
}

impl Default for ActivityMonitor {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for BackgroundIndexer {
    fn drop(&mut self) {
        self.stop();
    }
}
