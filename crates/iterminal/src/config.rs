use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Terminal configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalConfig {
    /// Font family
    pub font_family: String,
    /// Font size
    pub font_size: f32,
    /// Background color (RGBA)
    pub background_color: [f32; 4],
    /// Text color (RGBA)
    pub text_color: [f32; 4],
    /// Cursor color (RGBA)
    pub cursor_color: [f32; 4],
    /// Selection color (RGBA)
    pub selection_color: [f32; 4],
    /// Maximum number of lines in scrollback buffer
    pub scrollback_lines: usize,
    /// Default shell command
    pub shell_command: String,
    /// Working directory
    pub working_directory: Option<PathBuf>,
    /// Enable bell sound
    pub enable_bell: bool,
    /// Cursor blink interval in milliseconds
    pub cursor_blink_interval: u64,
    /// Tab size
    pub tab_size: usize,
    /// Enable line numbers
    pub show_line_numbers: bool,
}

impl Default for TerminalConfig {
    fn default() -> Self {
        Self {
            font_family: "Source Code Pro".to_string(),
            font_size: 14.0,
            background_color: [0.1, 0.1, 0.1, 1.0], // Dark background
            text_color: [0.9, 0.9, 0.9, 1.0],       // Light text
            cursor_color: [0.0, 1.0, 0.0, 1.0],     // Green cursor
            selection_color: [0.3, 0.3, 0.7, 0.5],  // Blue selection
            scrollback_lines: 10000,
            shell_command: Self::default_shell(),
            working_directory: None,
            enable_bell: false,
            cursor_blink_interval: 500,
            tab_size: 4,
            show_line_numbers: false,
        }
    }
}

impl TerminalConfig {
    /// Get default shell command for the current platform
    fn default_shell() -> String {
        #[cfg(windows)]
        {
            // On Windows, try PowerShell first, then fall back to cmd.exe
            if std::process::Command::new("powershell").arg("-Command").arg("echo test").output().is_ok() {
                "powershell".to_string()
            } else {
                "cmd.exe".to_string()
            }
        }
        #[cfg(unix)]
        {
            std::env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string())
        }
    }

    /// Load configuration from file
    pub fn load() -> Self {
        if let Some(config_dir) = dirs::config_dir() {
            let config_path = config_dir.join("seeu_desktop").join("terminal_config.json");
            if let Ok(content) = std::fs::read_to_string(&config_path) {
                if let Ok(config) = serde_json::from_str(&content) {
                    return config;
                }
            }
        }
        Self::default()
    }

    /// Save configuration to file
    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(config_dir) = dirs::config_dir() {
            let seeu_config_dir = config_dir.join("seeu_desktop");
            std::fs::create_dir_all(&seeu_config_dir)?;
            
            let config_path = seeu_config_dir.join("terminal_config.json");
            let content = serde_json::to_string_pretty(self)?;
            std::fs::write(&config_path, content)?;
        }
        Ok(())
    }

    /// Get working directory or default to home
    pub fn get_working_directory(&self) -> PathBuf {
        self.working_directory
            .clone()
            .or_else(|| dirs::home_dir())
            .unwrap_or_else(|| PathBuf::from("."))
    }
}
