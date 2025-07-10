use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{channel, Sender, Receiver};
use std::thread;
use std::time::{Duration, Instant};
use crate::{SearchResult, SearchStats};

/// Search optimization manager
pub struct SearchOptimizer {
    // Result caching
    cache: Arc<Mutex<HashMap<String, CachedSearchResult>>>,
    cache_size_limit: usize,
    cache_ttl: Duration,

    // Instant search
    instant_search_enabled: bool,
    search_delay: Duration,
    last_search_time: Arc<Mutex<Instant>>,
    
    // Async search
    search_sender: Option<Sender<SearchRequest>>,
    result_receiver: Option<Receiver<SearchResponse>>,
    
    // Performance metrics
    search_metrics: Arc<Mutex<SearchMetrics>>,
}

/// Cached search result with timestamp
#[derive(Debug, Clone)]
struct CachedSearchResult {
    results: Vec<SearchResult>,
    stats: SearchStats,
    timestamp: Instant,
    query_hash: u64,
}

/// Search request for async processing
#[derive(Debug, Clone)]
pub struct SearchRequest {
    pub query: String,
    pub file_type_filter: Option<String>,
    pub filename_filter: Option<String>,
    pub max_results: usize,
    pub request_id: u64,
}

/// Search response from async processing
#[derive(Debug, Clone)]
pub struct SearchResponse {
    pub results: Vec<SearchResult>,
    pub stats: SearchStats,
    pub request_id: u64,
    pub query: String,
    pub success: bool,
    pub error_message: Option<String>,
}

/// Performance metrics for search operations
#[derive(Debug, Clone)]
pub struct SearchMetrics {
    pub total_searches: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub average_search_time: Duration,
    pub last_search_time: Duration,
    pub total_search_time: Duration,
}

impl SearchOptimizer {
    /// Create a new search optimizer
    pub fn new() -> Self {
        let (search_sender, search_receiver) = channel();
        let (result_sender, result_receiver) = channel();
        
        let optimizer = Self {
            cache: Arc::new(Mutex::new(HashMap::new())),
            cache_size_limit: 1000, // Cache up to 1000 search results
            cache_ttl: Duration::from_secs(300), // 5 minutes TTL
            instant_search_enabled: true,
            search_delay: Duration::from_millis(300), // 300ms delay for instant search
            last_search_time: Arc::new(Mutex::new(Instant::now())),
            search_sender: Some(search_sender),
            result_receiver: Some(result_receiver),
            search_metrics: Arc::new(Mutex::new(SearchMetrics::new())),
        };

        // Start async search worker thread
        optimizer.start_search_worker(search_receiver, result_sender);
        
        optimizer
    }

    /// Enable or disable instant search
    pub fn set_instant_search_enabled(&mut self, enabled: bool) {
        self.instant_search_enabled = enabled;
    }

    /// Set search delay for instant search
    pub fn set_search_delay(&mut self, delay: Duration) {
        self.search_delay = delay;
    }

    /// Set cache size limit
    pub fn set_cache_size_limit(&mut self, limit: usize) {
        self.cache_size_limit = limit;
    }

    /// Set cache TTL
    pub fn set_cache_ttl(&mut self, ttl: Duration) {
        self.cache_ttl = ttl;
    }

    /// Check if enough time has passed for instant search
    pub fn should_trigger_instant_search(&self) -> bool {
        if !self.instant_search_enabled {
            return false;
        }

        if let Ok(last_search) = self.last_search_time.lock() {
            last_search.elapsed() >= self.search_delay
        } else {
            false
        }
    }

    /// Update last search time
    pub fn update_search_time(&self) {
        if let Ok(mut last_search) = self.last_search_time.lock() {
            *last_search = Instant::now();
        }
    }

    /// Get cached search result if available and valid
    pub fn get_cached_result(&self, query: &str) -> Option<(Vec<SearchResult>, SearchStats)> {
        if let Ok(cache) = self.cache.lock() {
            let query_hash = self.hash_query(query);
            if let Some(cached) = cache.get(query) {
                // Check if cache is still valid
                if cached.timestamp.elapsed() <= self.cache_ttl && cached.query_hash == query_hash {
                    // Update metrics
                    if let Ok(mut metrics) = self.search_metrics.lock() {
                        metrics.cache_hits += 1;
                    }
                    return Some((cached.results.clone(), cached.stats.clone()));
                }
            }
        }

        // Cache miss
        if let Ok(mut metrics) = self.search_metrics.lock() {
            metrics.cache_misses += 1;
        }
        None
    }

    /// Cache search result
    pub fn cache_result(&self, query: &str, results: Vec<SearchResult>, stats: SearchStats) {
        if let Ok(mut cache) = self.cache.lock() {
            // Clean up expired entries if cache is getting full
            if cache.len() >= self.cache_size_limit {
                self.cleanup_cache(&mut cache);
            }

            let cached_result = CachedSearchResult {
                results,
                stats,
                timestamp: Instant::now(),
                query_hash: self.hash_query(query),
            };

            cache.insert(query.to_string(), cached_result);
        }
    }

    /// Submit async search request
    pub fn submit_async_search(&self, request: SearchRequest) -> bool {
        if let Some(sender) = &self.search_sender {
            sender.send(request).is_ok()
        } else {
            false
        }
    }

    /// Get async search results
    pub fn get_async_results(&self) -> Vec<SearchResponse> {
        let mut results = Vec::new();
        if let Some(receiver) = &self.result_receiver {
            while let Ok(response) = receiver.try_recv() {
                results.push(response);
            }
        }
        results
    }

    /// Get search metrics
    pub fn get_metrics(&self) -> SearchMetrics {
        if let Ok(metrics) = self.search_metrics.lock() {
            metrics.clone()
        } else {
            SearchMetrics::new()
        }
    }

    /// Clear search cache
    pub fn clear_cache(&self) {
        if let Ok(mut cache) = self.cache.lock() {
            cache.clear();
        }
    }

    /// Hash query for cache key comparison
    fn hash_query(&self, query: &str) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        query.hash(&mut hasher);
        hasher.finish()
    }

    /// Clean up expired cache entries
    fn cleanup_cache(&self, cache: &mut HashMap<String, CachedSearchResult>) {
        let now = Instant::now();
        cache.retain(|_, cached| now.duration_since(cached.timestamp) <= self.cache_ttl);
        
        // If still too many entries, remove oldest ones
        if cache.len() >= self.cache_size_limit {
            let mut entries: Vec<_> = cache.iter().map(|(k, v)| (k.clone(), v.timestamp)).collect();
            entries.sort_by_key(|(_, timestamp)| *timestamp);

            let to_remove = cache.len() - (self.cache_size_limit * 3 / 4); // Remove 25% of entries
            for (key, _) in entries.iter().take(to_remove) {
                cache.remove(key);
            }
        }
    }

    /// Start async search worker thread
    fn start_search_worker(&self, receiver: Receiver<SearchRequest>, sender: Sender<SearchResponse>) {
        let _cache = self.cache.clone();
        let metrics = self.search_metrics.clone();
        
        thread::spawn(move || {
            log::info!("Search optimizer worker thread started");
            
            while let Ok(request) = receiver.recv() {
                let start_time = Instant::now();
                
                // TODO: Implement actual search logic here
                // For now, create a placeholder response
                let response = SearchResponse {
                    results: Vec::new(),
                    stats: SearchStats::default(),
                    request_id: request.request_id,
                    query: request.query.clone(),
                    success: true,
                    error_message: None,
                };

                let search_duration = start_time.elapsed();
                
                // Update metrics
                if let Ok(mut metrics) = metrics.lock() {
                    metrics.total_searches += 1;
                    metrics.last_search_time = search_duration;
                    metrics.total_search_time += search_duration;
                    metrics.average_search_time = Duration::from_nanos(
                        metrics.total_search_time.as_nanos() as u64 / metrics.total_searches
                    );
                }

                if sender.send(response).is_err() {
                    log::warn!("Failed to send search response");
                    break;
                }
            }
            
            log::info!("Search optimizer worker thread stopped");
        });
    }
}

impl SearchMetrics {
    pub fn new() -> Self {
        Self {
            total_searches: 0,
            cache_hits: 0,
            cache_misses: 0,
            average_search_time: Duration::from_millis(0),
            last_search_time: Duration::from_millis(0),
            total_search_time: Duration::from_millis(0),
        }
    }

    /// Get cache hit rate as percentage
    pub fn cache_hit_rate(&self) -> f64 {
        if self.cache_hits + self.cache_misses == 0 {
            0.0
        } else {
            (self.cache_hits as f64 / (self.cache_hits + self.cache_misses) as f64) * 100.0
        }
    }
}

impl Default for SearchOptimizer {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for SearchMetrics {
    fn default() -> Self {
        Self::new()
    }
}
