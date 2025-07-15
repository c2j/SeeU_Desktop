use eframe::egui;

/// Settings category information
#[derive(Debug, Clone, PartialEq)]
pub struct SettingsCategory {
    pub id: String,
    pub display_name: String,
    pub icon: String,
    pub description: String,
}

impl SettingsCategory {
    pub fn new(id: &str, display_name: &str, icon: &str, description: &str) -> Self {
        Self {
            id: id.to_string(),
            display_name: display_name.to_string(),
            icon: icon.to_string(),
            description: description.to_string(),
        }
    }

    // /// Get the full display name with icon
    // pub fn full_display_name(&self) -> String {
    //     format!("{} {}", self.icon, self.display_name)
    // }
}

/// Trait for settings modules
/// Each crate should implement this trait to provide its own settings UI
pub trait SettingsModule {
    /// Get the settings category information
    fn get_category(&self) -> SettingsCategory;

    /// Render the settings UI for this module
    /// Returns true if any settings were changed
    fn render_settings(&mut self, ui: &mut egui::Ui) -> bool;

    /// Save settings to persistent storage
    fn save_settings(&self) -> Result<(), Box<dyn std::error::Error>>;

    /// Load settings from persistent storage
    fn load_settings(&mut self) -> Result<(), Box<dyn std::error::Error>>;

    /// Reset settings to default values
    fn reset_to_default(&mut self) -> Result<(), Box<dyn std::error::Error>>;

    /// Get a brief status or summary of current settings
    fn get_settings_summary(&self) -> String {
        "设置已配置".to_string()
    }

    /// Check if settings are valid
    fn validate_settings(&self) -> Result<(), String> {
        Ok(())
    }

    /// Get help text for this settings module
    fn get_help_text(&self) -> Option<String> {
        None
    }
}

/// Settings module registry
/// Manages all available settings modules
pub struct SettingsRegistry {
    modules: Vec<Box<dyn SettingsModule>>,
    current_category_id: Option<String>,
}

impl SettingsRegistry {
    pub fn new() -> Self {
        Self {
            modules: Vec::new(),
            current_category_id: None,
        }
    }

    /// Register a settings module
    pub fn register_module(&mut self, module: Box<dyn SettingsModule>) {
        self.modules.push(module);
    }

    // /// Get all available categories
    // pub fn get_categories(&self) -> Vec<SettingsCategory> {
    //     self.modules.iter().map(|m| m.get_category()).collect()
    // }

    // /// Get current selected category ID
    // pub fn get_current_category_id(&self) -> Option<&String> {
    //     self.current_category_id.as_ref()
    // }

    /// Set current category
    pub fn set_current_category(&mut self, category_id: String) {
        self.current_category_id = Some(category_id);
    }

    /// Render the current category's settings
    pub fn render_current_settings(&mut self, ui: &mut egui::Ui) -> bool {
        if let Some(category_id) = &self.current_category_id.clone() {
            if let Some(module) = self.modules.iter_mut().find(|m| m.get_category().id == *category_id) {
                return module.render_settings(ui);
            }
        }
        false
    }

    /// Save all settings
    pub fn save_all_settings(&self) -> Result<(), Vec<Box<dyn std::error::Error>>> {
        let mut errors = Vec::new();
        
        for module in &self.modules {
            if let Err(e) = module.save_settings() {
                errors.push(e);
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    // /// Load all settings
    // pub fn load_all_settings(&mut self) -> Result<(), Vec<Box<dyn std::error::Error>>> {
    //     let mut errors = Vec::new();
        
    //     for module in &mut self.modules {
    //         if let Err(e) = module.load_settings() {
    //             errors.push(e);
    //         }
    //     }

    //     if errors.is_empty() {
    //         Ok(())
    //     } else {
    //         Err(errors)
    //     }
    // }

    /// Reset all settings to default
    pub fn reset_all_to_default(&mut self) -> Result<(), Vec<Box<dyn std::error::Error>>> {
        let mut errors = Vec::new();
        
        for module in &mut self.modules {
            if let Err(e) = module.reset_to_default() {
                errors.push(e);
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Validate all settings
    pub fn validate_all_settings(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        
        for module in &self.modules {
            if let Err(e) = module.validate_settings() {
                errors.push(format!("{}: {}", module.get_category().display_name, e));
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    // /// Get settings summary for all modules
    // pub fn get_all_summaries(&self) -> Vec<(SettingsCategory, String)> {
    //     self.modules.iter()
    //         .map(|m| (m.get_category(), m.get_settings_summary()))
    //         .collect()
    // }
}

impl Default for SettingsRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for SettingsRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SettingsRegistry")
            .field("modules_count", &self.modules.len())
            .field("current_category_id", &self.current_category_id)
            .finish()
    }
}

/// Modular settings state
#[derive(Debug)]
pub struct ModularSettingsState {
    pub settings_registry: Option<SettingsRegistry>,
    pub selected_category: String,
    pub has_unsaved_changes: bool,
    pub last_save_status: String,
    pub validation_status: String,
    pub validation_errors: Vec<String>,
    pub show_reset_confirmation: bool,
}

impl Default for ModularSettingsState {
    fn default() -> Self {
        Self {
            settings_registry: None,
            selected_category: "app".to_string(),
            has_unsaved_changes: false,
            last_save_status: "未保存".to_string(),
            validation_status: "未验证".to_string(),
            validation_errors: Vec::new(),
            show_reset_confirmation: false,
        }
    }
}
