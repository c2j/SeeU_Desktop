use std::collections::HashMap;
use uuid::Uuid;
use serde::{Deserialize, Serialize};
use anyhow::Result;

use super::plugin::{Plugin, PluginMetadata, PluginManifest};
use crate::roles::UserRole;
use crate::state::PermissionLevel;

/// Plugin marketplace client
#[derive(Debug)]
pub struct PluginMarketplace {
    /// Available plugins from marketplace
    available_plugins: HashMap<Uuid, MarketplacePlugin>,

    /// Featured plugins
    featured_plugins: Vec<Uuid>,

    /// Categories
    categories: Vec<PluginCategory>,

    /// Last refresh time
    last_refresh: Option<chrono::DateTime<chrono::Utc>>,

    /// Marketplace URL
    marketplace_url: String,
}

/// Plugin information from marketplace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketplacePlugin {
    pub plugin: Plugin,
    pub download_url: String,
    pub download_count: u64,
    pub rating: f32,
    pub review_count: u32,
    pub last_updated: chrono::DateTime<chrono::Utc>,
    pub verified: bool,
    pub featured: bool,
}

/// Plugin category
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginCategory {
    pub id: String,
    pub name: String,
    pub description: String,
    pub icon: Option<String>,
    pub plugin_count: u32,
}

/// Search filters for marketplace
#[derive(Debug, Default, Clone)]
pub struct MarketplaceFilters {
    pub query: String,
    pub category: Option<String>,
    pub role: Option<UserRole>,
    pub permission_level: Option<PermissionLevel>,
    pub verified_only: bool,
    pub featured_only: bool,
    pub sort_by: SortBy,
}

/// Sort options for marketplace
#[derive(Debug, Clone, PartialEq)]
pub enum SortBy {
    Relevance,
    Name,
    Rating,
    Downloads,
    LastUpdated,
    Newest,
}

impl Default for SortBy {
    fn default() -> Self {
        SortBy::Relevance
    }
}

impl PluginMarketplace {
    /// Create a new marketplace client
    pub fn new() -> Self {
        Self {
            available_plugins: HashMap::new(),
            featured_plugins: Vec::new(),
            categories: Vec::new(),
            last_refresh: None,
            marketplace_url: "https://marketplace.seeu.app".to_string(), // Placeholder URL
        }
    }

    /// Initialize the marketplace
    pub fn initialize(&mut self) {
        log::info!("Initializing plugin marketplace");

        // Load preset plugins
        self.load_preset_plugins();

        // Load categories
        self.load_categories();

        // Refresh marketplace data
        self.refresh_marketplace();
    }

    /// Update marketplace (called from main loop)
    pub fn update(&mut self) {
        // Check if we need to refresh marketplace data
        if self.should_refresh() {
            self.refresh_marketplace();
        }
    }

    /// Search plugins in marketplace
    pub fn search_plugins(&self, filters: &MarketplaceFilters) -> Vec<&MarketplacePlugin> {
        let mut results: Vec<&MarketplacePlugin> = self.available_plugins
            .values()
            .filter(|plugin| self.matches_filters(plugin, filters))
            .collect();

        // Sort results
        self.sort_results(&mut results, &filters.sort_by);

        results
    }

    /// Get featured plugins
    pub fn get_featured_plugins(&self) -> Vec<&MarketplacePlugin> {
        self.featured_plugins
            .iter()
            .filter_map(|id| self.available_plugins.get(id))
            .collect()
    }

    /// Get plugin by ID
    pub fn get_plugin(&self, id: &Uuid) -> Option<&MarketplacePlugin> {
        self.available_plugins.get(id)
    }

    /// Get all categories
    pub fn get_categories(&self) -> &Vec<PluginCategory> {
        &self.categories
    }

    /// Get plugins by category
    pub fn get_plugins_by_category(&self, category_id: &str) -> Vec<&MarketplacePlugin> {
        self.available_plugins
            .values()
            .filter(|plugin| plugin.plugin.metadata.categories.contains(&category_id.to_string()))
            .collect()
    }

    /// Check if marketplace should be refreshed
    fn should_refresh(&self) -> bool {
        match self.last_refresh {
            Some(last) => {
                let now = chrono::Utc::now();
                let duration = now.signed_duration_since(last);
                duration.num_hours() >= 1 // Refresh every hour
            }
            None => true,
        }
    }

    /// Refresh marketplace data
    fn refresh_marketplace(&mut self) {
        log::info!("Refreshing marketplace data");

        // TODO: Implement actual marketplace API calls
        // For now, we'll use preset data

        self.last_refresh = Some(chrono::Utc::now());
    }

    /// Load preset plugins (built-in plugins)
    fn load_preset_plugins(&mut self) {
        log::info!("Loading preset plugins");

        // Filesystem MCP Server
        let filesystem_plugin = self.create_filesystem_plugin();
        self.available_plugins.insert(filesystem_plugin.plugin.id, filesystem_plugin);

        // Git Integration
        let git_plugin = self.create_git_plugin();
        self.available_plugins.insert(git_plugin.plugin.id, git_plugin);

        // BI Connector
        let bi_plugin = self.create_bi_plugin();
        self.available_plugins.insert(bi_plugin.plugin.id, bi_plugin);

        // System Monitor
        let monitor_plugin = self.create_system_monitor_plugin();
        self.available_plugins.insert(monitor_plugin.plugin.id, monitor_plugin);

        // Set featured plugins
        self.featured_plugins = self.available_plugins.keys().cloned().collect();
    }

    /// Load plugin categories
    fn load_categories(&mut self) {
        self.categories = vec![
            PluginCategory {
                id: "file-system".to_string(),
                name: "文件系统".to_string(),
                description: "文件和目录操作工具".to_string(),
                icon: Some("📁".to_string()),
                plugin_count: 1,
            },
            PluginCategory {
                id: "development".to_string(),
                name: "开发工具".to_string(),
                description: "代码开发和版本控制工具".to_string(),
                icon: Some("💻".to_string()),
                plugin_count: 1,
            },
            PluginCategory {
                id: "data-analysis".to_string(),
                name: "数据分析".to_string(),
                description: "商业智能和数据可视化工具".to_string(),
                icon: Some("📊".to_string()),
                plugin_count: 1,
            },
            PluginCategory {
                id: "monitoring".to_string(),
                name: "系统监控".to_string(),
                description: "系统性能和资源监控工具".to_string(),
                icon: Some("📈".to_string()),
                plugin_count: 1,
            },
        ];
    }

    /// Check if plugin matches search filters
    fn matches_filters(&self, plugin: &MarketplacePlugin, filters: &MarketplaceFilters) -> bool {
        // Query filter
        if !filters.query.is_empty() {
            let query_lower = filters.query.to_lowercase();
            let matches_name = plugin.plugin.metadata.name.to_lowercase().contains(&query_lower);
            let matches_description = plugin.plugin.metadata.description.to_lowercase().contains(&query_lower);
            let matches_keywords = plugin.plugin.metadata.keywords
                .iter()
                .any(|keyword| keyword.to_lowercase().contains(&query_lower));

            if !matches_name && !matches_description && !matches_keywords {
                return false;
            }
        }

        // Category filter
        if let Some(category) = &filters.category {
            if !plugin.plugin.metadata.categories.contains(category) {
                return false;
            }
        }

        // Role filter
        if let Some(role) = &filters.role {
            if !plugin.plugin.is_compatible_with_role(role) {
                return false;
            }
        }

        // Permission level filter
        if let Some(max_level) = &filters.permission_level {
            if plugin.plugin.get_max_permission_level() > *max_level {
                return false;
            }
        }

        // Verified only filter
        if filters.verified_only && !plugin.verified {
            return false;
        }

        // Featured only filter
        if filters.featured_only && !plugin.featured {
            return false;
        }

        true
    }

    /// Sort search results
    fn sort_results(&self, results: &mut Vec<&MarketplacePlugin>, sort_by: &SortBy) {
        match sort_by {
            SortBy::Name => {
                results.sort_by(|a, b| a.plugin.metadata.name.cmp(&b.plugin.metadata.name));
            }
            SortBy::Rating => {
                results.sort_by(|a, b| b.rating.partial_cmp(&a.rating).unwrap_or(std::cmp::Ordering::Equal));
            }
            SortBy::Downloads => {
                results.sort_by(|a, b| b.download_count.cmp(&a.download_count));
            }
            SortBy::LastUpdated => {
                results.sort_by(|a, b| b.last_updated.cmp(&a.last_updated));
            }
            SortBy::Newest => {
                results.sort_by(|a, b| b.plugin.installed_at.cmp(&a.plugin.installed_at));
            }
            SortBy::Relevance => {
                // For relevance, we could implement a scoring algorithm
                // For now, just sort by rating
                results.sort_by(|a, b| b.rating.partial_cmp(&a.rating).unwrap_or(std::cmp::Ordering::Equal));
            }
        }
    }
}

impl Default for PluginMarketplace {
    fn default() -> Self {
        Self::new()
    }
}
