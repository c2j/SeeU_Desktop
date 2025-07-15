// Platform-specific code and utilities

use std::path::PathBuf;

/// Get the platform-specific application data directory
pub fn app_data_dir() -> PathBuf {
    let mut path = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push("seeu_desktop");
    path
}

// /// Get the platform-specific configuration directory
// pub fn app_config_dir() -> PathBuf {
//     let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
//     path.push("seeu_desktop");
//     path
// }

// /// Get the platform-specific log directory
pub fn app_log_dir() -> PathBuf {
    let mut path = app_data_dir();
    path.push("logs");
    path
}

// /// Get the platform name
// pub fn platform_name() -> &'static str {
//     #[cfg(target_os = "windows")]
//     {
//         "windows"
//     }
//     #[cfg(target_os = "macos")]
//     {
//         "macos"
//     }
//     #[cfg(target_os = "linux")]
//     {
//         "linux"
//     }
//     #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
//     {
//         "unknown"
//     }
// }

// /// Check if running on Windows
// pub fn is_windows() -> bool {
//     #[cfg(target_os = "windows")]
//     {
//         true
//     }
//     #[cfg(not(target_os = "windows"))]
//     {
//         false
//     }
// }

// /// Check if running on macOS
// pub fn is_macos() -> bool {
//     #[cfg(target_os = "macos")]
//     {
//         true
//     }
//     #[cfg(not(target_os = "macos"))]
//     {
//         false
//     }
// }

// / Check if running on Linux
// pub fn is_linux() -> bool {
//     #[cfg(target_os = "linux")]
//     {
//         true
//     }
//     #[cfg(not(target_os = "linux"))]
//     {
//         false
//     }
// }

// /// Get platform-specific font paths
// pub fn get_system_fonts() -> Vec<PathBuf> {
//     let mut fonts = Vec::new();
    
//     #[cfg(target_os = "windows")]
//     {
//         // Windows font paths
//         if let Some(windir) = std::env::var_os("WINDIR") {
//             let mut font_dir = PathBuf::from(windir);
//             font_dir.push("Fonts");
//             fonts.push(font_dir);
//         }
//     }
    
//     #[cfg(target_os = "macos")]
//     {
//         // macOS font paths
//         fonts.push(PathBuf::from("/System/Library/Fonts"));
//         fonts.push(PathBuf::from("/Library/Fonts"));
        
//         if let Some(home) = dirs::home_dir() {
//             let mut user_fonts = home;
//             user_fonts.push("Library");
//             user_fonts.push("Fonts");
//             fonts.push(user_fonts);
//         }
//     }
    
//     #[cfg(target_os = "linux")]
//     {
//         // Linux font paths
//         fonts.push(PathBuf::from("/usr/share/fonts"));
//         fonts.push(PathBuf::from("/usr/local/share/fonts"));
        
//         if let Some(home) = dirs::home_dir() {
//             let mut user_fonts = home;
//             user_fonts.push(".local");
//             user_fonts.push("share");
//             user_fonts.push("fonts");
//             fonts.push(user_fonts);
            
//             // Also check .fonts directory (older location)
//             let mut old_user_fonts = dirs::home_dir().unwrap();
//             old_user_fonts.push(".fonts");
//             fonts.push(old_user_fonts);
//         }
//     }
    
//     fonts
// }
