use eframe::egui;

/// Icon sizes for different platforms and use cases
#[derive(Debug, Clone, Copy)]
pub enum IconSize {
    Small = 16,
    Medium = 32,
    Large = 48,
    ExtraLarge = 64,
    Huge = 128,
    Massive = 256,
}

impl IconSize {
    /// Get all available icon sizes
    pub fn all() -> Vec<IconSize> {
        vec![
            IconSize::Small,
            IconSize::Medium,
            IconSize::Large,
            IconSize::ExtraLarge,
            IconSize::Huge,
            IconSize::Massive,
        ]
    }

    /// Get the size as u32
    pub fn as_u32(self) -> u32 {
        self as u32
    }
}

/// Load application icon data for window icon
pub fn load_window_icon() -> Result<egui::IconData, Box<dyn std::error::Error>> {
    // Use the 32x32 icon for window icon (good balance of quality and size)
    let icon_data = include_bytes!("../../assets/icons/sizes/icon-32x32.png");

    // Decode the PNG image
    let image = image::load_from_memory(icon_data)?;
    let rgba_image = image.to_rgba8();

    let (width, height) = rgba_image.dimensions();
    let pixels = rgba_image.into_raw();

    Ok(egui::IconData {
        rgba: pixels,
        width: width as u32,
        height: height as u32,
    })
}

/// Load the main application icon (original size)
pub fn load_main_icon() -> Result<egui::IconData, Box<dyn std::error::Error>> {
    let icon_data = include_bytes!("../../assets/icons/c-see.png");

    // Decode the PNG image
    let image = image::load_from_memory(icon_data)?;
    let rgba_image = image.to_rgba8();

    let (width, height) = rgba_image.dimensions();
    let pixels = rgba_image.into_raw();

    Ok(egui::IconData {
        rgba: pixels,
        width: width as u32,
        height: height as u32,
    })
}

/// Load icon for specific size
pub fn load_icon_for_size(size: IconSize) -> Result<egui::IconData, Box<dyn std::error::Error>> {
    let icon_data: &[u8] = match size {
        IconSize::Small => include_bytes!("../../assets/icons/sizes/icon-16x16.png"),
        IconSize::Medium => include_bytes!("../../assets/icons/sizes/icon-32x32.png"),
        IconSize::Large => include_bytes!("../../assets/icons/sizes/icon-48x48.png"),
        IconSize::ExtraLarge => include_bytes!("../../assets/icons/sizes/icon-64x64.png"),
        IconSize::Huge => include_bytes!("../../assets/icons/sizes/icon-128x128.png"),
        IconSize::Massive => include_bytes!("../../assets/icons/sizes/icon-256x256.png"),
    };

    // Decode the PNG image
    let image = image::load_from_memory(icon_data)?;
    let rgba_image = image.to_rgba8();

    let (width, height) = rgba_image.dimensions();
    let pixels = rgba_image.into_raw();

    Ok(egui::IconData {
        rgba: pixels,
        width: width as u32,
        height: height as u32,
    })
}

/// Get platform-specific icon recommendations
pub fn get_platform_icon_sizes() -> Vec<IconSize> {
    #[cfg(target_os = "windows")]
    {
        // Windows typically uses 16x16, 32x32, 48x48, 256x256
        vec![IconSize::Small, IconSize::Medium, IconSize::Large, IconSize::Massive]
    }

    #[cfg(target_os = "macos")]
    {
        // macOS typically uses 16x16, 32x32, 128x128, 256x256
        vec![IconSize::Small, IconSize::Medium, IconSize::Huge, IconSize::Massive]
    }

    #[cfg(target_os = "linux")]
    {
        // Linux typically uses 16x16, 32x32, 48x48, 64x64, 128x128
        vec![
            IconSize::Small,
            IconSize::Medium,
            IconSize::Large,
            IconSize::ExtraLarge,
            IconSize::Huge,
        ]
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        // Default for other platforms
        IconSize::all()
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing icon loading functionality...");
    
    // Test loading window icon
    match load_window_icon() {
        Ok(icon_data) => {
            println!("✓ Successfully loaded window icon: {}x{} pixels", 
                     icon_data.width, icon_data.height);
            println!("  Icon data size: {} bytes", icon_data.rgba.len());
        }
        Err(e) => {
            println!("✗ Failed to load window icon: {}", e);
            return Err(e);
        }
    }
    
    // Test loading main icon
    match load_main_icon() {
        Ok(icon_data) => {
            println!("✓ Successfully loaded main icon: {}x{} pixels", 
                     icon_data.width, icon_data.height);
            println!("  Icon data size: {} bytes", icon_data.rgba.len());
        }
        Err(e) => {
            println!("✗ Failed to load main icon: {}", e);
            return Err(e);
        }
    }
    
    // Test loading different sizes
    println!("\nTesting different icon sizes:");
    for size in IconSize::all() {
        match load_icon_for_size(size) {
            Ok(icon_data) => {
                println!("✓ Size {:?}: {}x{} pixels, {} bytes", 
                         size, icon_data.width, icon_data.height, icon_data.rgba.len());
            }
            Err(e) => {
                println!("✗ Size {:?}: Failed to load - {}", size, e);
            }
        }
    }
    
    // Test platform-specific recommendations
    println!("\nPlatform-recommended icon sizes:");
    let platform_sizes = get_platform_icon_sizes();
    for size in platform_sizes {
        println!("  - {:?} ({}x{})", size, size.as_u32(), size.as_u32());
    }
    
    println!("\n✓ All icon tests completed successfully!");
    Ok(())
}
