use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use crate::platform;

/// Application configuration
#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub theme: String,
    pub ai_settings: AiSettings,
    pub window_size: WindowSize,
}

/// AI settings
#[derive(Debug, Serialize, Deserialize)]
pub struct AiSettings {
    pub api_url: String,
    pub api_key: String,
    pub model: String,
}

/// Window size
#[derive(Debug, Serialize, Deserialize)]
pub struct WindowSize {
    pub width: f32,
    pub height: f32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            theme: "dark".to_string(),
            ai_settings: AiSettings {
                api_url: "http://localhost:11434/v1".to_string(),
                api_key: "EMPTY".to_string(),
                model: "qwen3:4b".to_string(),
            },
            window_size: WindowSize {
                width: 1280.0,
                height: 720.0,
            },
        }
    }
}

impl Config {
    /// Load configuration from file
    pub fn load() -> Self {
        let config_path = Self::config_path();

        if let Ok(config_str) = fs::read_to_string(config_path) {
            if let Ok(config) = serde_json::from_str(&config_str) {
                return config;
            }
        }

        // Return default config if loading fails
        Self::default()
    }

    /// Save configuration to file
    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let config_path = Self::config_path();

        // Create parent directories if they don't exist
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Serialize and save config
        let config_str = serde_json::to_string_pretty(self)?;
        fs::write(config_path, config_str)?;

        Ok(())
    }

    /// Get the path to the configuration file
    fn config_path() -> PathBuf {
        let mut path = platform::app_config_dir();
        path.push("config.json");
        path
    }
}