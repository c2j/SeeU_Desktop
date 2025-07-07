use serde::{Deserialize, Serialize};

/// Startup configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartupConfig {
    /// Whether to enable lazy loading of notes
    pub lazy_load_notes: bool,
    
    /// Whether to enable async initialization
    pub async_initialization: bool,
    
    /// Whether to show startup progress
    pub show_startup_progress: bool,
    
    /// Maximum number of notes to load at startup (0 = load all)
    pub max_notes_at_startup: usize,
    
    /// Whether to enable background data migration
    pub background_migration: bool,
    
    /// Whether to enable background indexing
    pub background_indexing: bool,
    
    /// Startup timeout in seconds
    pub startup_timeout_seconds: u64,
}

impl Default for StartupConfig {
    fn default() -> Self {
        Self {
            lazy_load_notes: true,
            async_initialization: true,
            show_startup_progress: true,
            max_notes_at_startup: 0, // Load all by default for now
            background_migration: true,
            background_indexing: true,
            startup_timeout_seconds: 30,
        }
    }
}

impl StartupConfig {
    /// Load configuration from file
    pub fn load() -> Self {
        let base_path = dirs::config_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
        let config_path = base_path.join("seeu_desktop").join("startup.toml");
        
        if config_path.exists() {
            match std::fs::read_to_string(&config_path) {
                Ok(content) => {
                    match toml::from_str(&content) {
                        Ok(config) => {
                            return config;
                        }
                        Err(_e) => {
                            // Failed to parse config, use default
                        }
                    }
                }
                Err(_e) => {
                    // Failed to read config file, use default
                }
            }
        }
        
        // Return default config and save it
        let default_config = Self::default();
        let _ = default_config.save(); // Ignore save errors during startup
        default_config
    }
    
    /// Save configuration to file
    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let base_path = dirs::config_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
        let config_path = base_path.join("seeu_desktop").join("startup.toml");
        
        // Create parent directory if it doesn't exist
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        let content = toml::to_string_pretty(self)?;
        std::fs::write(&config_path, content)?;

        Ok(())
    }
    

}

/// Startup performance metrics
#[derive(Debug, Clone)]
pub struct StartupMetrics {
    pub start_time: std::time::Instant,
    pub font_load_time: Option<std::time::Duration>,
    pub database_init_time: Option<std::time::Duration>,
    pub search_init_time: Option<std::time::Duration>,
    pub plugin_init_time: Option<std::time::Duration>,
    pub total_time: Option<std::time::Duration>,
}

impl Default for StartupMetrics {
    fn default() -> Self {
        Self::new()
    }
}

impl StartupMetrics {
    pub fn new() -> Self {
        Self {
            start_time: std::time::Instant::now(),
            font_load_time: None,
            database_init_time: None,
            search_init_time: None,
            plugin_init_time: None,
            total_time: None,
        }
    }
    
    pub fn record_font_load(&mut self) {
        self.font_load_time = Some(self.start_time.elapsed());
    }
    
    pub fn record_database_init(&mut self) {
        self.database_init_time = Some(self.start_time.elapsed());
    }
    
    pub fn record_search_init(&mut self) {
        self.search_init_time = Some(self.start_time.elapsed());
    }
    
    pub fn record_plugin_init(&mut self) {
        self.plugin_init_time = Some(self.start_time.elapsed());
    }
    
    pub fn record_total(&mut self) {
        self.total_time = Some(self.start_time.elapsed());
    }
    
    pub fn log_metrics(&self) {
        // Metrics logging disabled during startup to avoid potential issues
        // Can be re-enabled later if needed
    }
}
