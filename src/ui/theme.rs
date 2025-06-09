use eframe::egui;

/// Application theme
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Theme {
    DarkModern,
    LightModern,
    // Legacy themes (for backward compatibility)
    Dark,
    Light,
}

impl Theme {
    /// Get display name for the theme
    pub fn display_name(&self) -> &'static str {
        match self {
            Theme::DarkModern => "Dark Modern",
            Theme::LightModern => "Light Modern",
            Theme::Dark => "Dark (Legacy)",
            Theme::Light => "Light (Legacy)",
        }
    }

    /// Get all available themes
    pub fn all() -> Vec<Theme> {
        vec![Theme::DarkModern, Theme::LightModern]
    }

    /// Convert from string
    pub fn from_string(s: &str) -> Theme {
        match s {
            "dark_modern" => Theme::DarkModern,
            "light_modern" => Theme::LightModern,
            "dark" => Theme::Dark,
            "light" => Theme::Light,
            _ => Theme::DarkModern, // Default
        }
    }

    /// Convert to string
    pub fn to_string(&self) -> &'static str {
        match self {
            Theme::DarkModern => "dark_modern",
            Theme::LightModern => "light_modern",
            Theme::Dark => "dark",
            Theme::Light => "light",
        }
    }
}

/// Configure the visual appearance of the application
pub fn configure_visuals(ctx: &egui::Context, theme: Theme) {
    match theme {
        Theme::DarkModern => {
            ctx.set_visuals(egui::Visuals {
                dark_mode: true,
                panel_fill: egui::Color32::from_rgb(24, 24, 27),        // Modern dark gray
                window_fill: egui::Color32::from_rgb(39, 39, 42),       // Slightly lighter
                faint_bg_color: egui::Color32::from_rgb(30, 30, 33),    // Subtle background
                extreme_bg_color: egui::Color32::from_rgb(16, 16, 18),  // Very dark
                code_bg_color: egui::Color32::from_rgb(45, 45, 48),     // Code background
                warn_fg_color: egui::Color32::from_rgb(255, 193, 7),    // Modern amber
                error_fg_color: egui::Color32::from_rgb(244, 67, 54),   // Modern red
                hyperlink_color: egui::Color32::from_rgb(100, 181, 246), // Modern blue
                selection: egui::style::Selection {
                    bg_fill: egui::Color32::from_rgb(63, 81, 181),      // Modern indigo
                    stroke: egui::Stroke::new(1.0, egui::Color32::from_rgb(100, 181, 246)),
                },
                ..Default::default()
            });
        },
        Theme::LightModern => {
            ctx.set_visuals(egui::Visuals {
                dark_mode: false,
                panel_fill: egui::Color32::from_rgb(248, 249, 250),     // Modern light gray
                window_fill: egui::Color32::from_rgb(255, 255, 255),    // Pure white
                faint_bg_color: egui::Color32::from_rgb(243, 244, 246), // Very light gray
                extreme_bg_color: egui::Color32::from_rgb(249, 250, 251), // Off-white
                code_bg_color: egui::Color32::from_rgb(241, 245, 249),  // Light blue-gray
                warn_fg_color: egui::Color32::from_rgb(245, 124, 0),    // Modern orange
                error_fg_color: egui::Color32::from_rgb(211, 47, 47),   // Modern red
                hyperlink_color: egui::Color32::from_rgb(25, 118, 210), // Modern blue
                selection: egui::style::Selection {
                    bg_fill: egui::Color32::from_rgb(227, 242, 253),    // Light blue
                    stroke: egui::Stroke::new(1.0, egui::Color32::from_rgb(25, 118, 210)),
                },
                ..Default::default()
            });
        },
        // Legacy themes for backward compatibility
        Theme::Dark => {
            ctx.set_visuals(egui::Visuals {
                dark_mode: true,
                panel_fill: egui::Color32::from_rgb(30, 30, 30),
                window_fill: egui::Color32::from_rgb(40, 40, 40),
                faint_bg_color: egui::Color32::from_rgb(35, 35, 35),
                extreme_bg_color: egui::Color32::from_rgb(20, 20, 20),
                code_bg_color: egui::Color32::from_rgb(45, 45, 45),
                warn_fg_color: egui::Color32::from_rgb(255, 143, 0),
                error_fg_color: egui::Color32::from_rgb(255, 0, 0),
                hyperlink_color: egui::Color32::from_rgb(90, 170, 255),
                ..Default::default()
            });
        },
        Theme::Light => {
            ctx.set_visuals(egui::Visuals {
                dark_mode: false,
                panel_fill: egui::Color32::from_rgb(240, 240, 240),
                window_fill: egui::Color32::from_rgb(250, 250, 250),
                faint_bg_color: egui::Color32::from_rgb(235, 235, 235),
                extreme_bg_color: egui::Color32::from_rgb(255, 255, 255),
                code_bg_color: egui::Color32::from_rgb(245, 245, 245),
                warn_fg_color: egui::Color32::from_rgb(230, 130, 0),
                error_fg_color: egui::Color32::from_rgb(230, 0, 0),
                hyperlink_color: egui::Color32::from_rgb(0, 120, 255),
                ..Default::default()
            });
        }
    }
}