use std::fs;
use std::path::{Path, PathBuf};
use std::io::Write;
use arboard::ImageData;
use uuid::Uuid;

/// Image storage manager for handling image files
pub struct ImageStorageManager {
    base_path: PathBuf,
}

/// Error types for image storage operations
#[derive(Debug)]
pub enum ImageStorageError {
    IoError(std::io::Error),
    InvalidFormat(String),
    NotFound(String),
}

impl std::fmt::Display for ImageStorageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ImageStorageError::IoError(e) => write!(f, "IO error: {}", e),
            ImageStorageError::InvalidFormat(msg) => write!(f, "Invalid format: {}", msg),
            ImageStorageError::NotFound(msg) => write!(f, "Not found: {}", msg),
        }
    }
}

impl std::error::Error for ImageStorageError {}

impl From<std::io::Error> for ImageStorageError {
    fn from(error: std::io::Error) -> Self {
        ImageStorageError::IoError(error)
    }
}

impl ImageStorageManager {
    /// Create a new image storage manager
    pub fn new() -> Result<Self, ImageStorageError> {
        let mut base_path = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."));
        base_path.push("seeu_desktop");
        base_path.push("inote");
        base_path.push("images");

        // Create images directory if it doesn't exist
        fs::create_dir_all(&base_path)?;

        Ok(Self { base_path })
    }

    /// Save image data to storage and return the relative path
    pub fn save_image(&self, image_data: &ImageData) -> Result<String, ImageStorageError> {
        // Generate unique filename
        let image_id = Uuid::new_v4().to_string();
        let filename = format!("{}.png", image_id);
        let file_path = self.base_path.join(&filename);

        // Convert ImageData to PNG bytes
        let png_bytes = self.image_data_to_png(image_data)?;

        // Write to file
        let mut file = fs::File::create(&file_path)?;
        file.write_all(&png_bytes)?;

        log::info!("Saved image to: {:?}", file_path);

        // Return relative path for markdown
        Ok(format!("images/{}", filename))
    }

    /// Load image from storage
    pub fn load_image(&self, relative_path: &str) -> Result<Vec<u8>, ImageStorageError> {
        let file_path = if relative_path.starts_with("images/") {
            self.base_path.join(relative_path.strip_prefix("images/").unwrap())
        } else {
            self.base_path.join(relative_path)
        };

        if !file_path.exists() {
            return Err(ImageStorageError::NotFound(format!("Image not found: {:?}", file_path)));
        }

        let bytes = fs::read(&file_path)?;
        Ok(bytes)
    }

    /// Delete image from storage
    pub fn delete_image(&self, relative_path: &str) -> Result<(), ImageStorageError> {
        let file_path = if relative_path.starts_with("images/") {
            self.base_path.join(relative_path.strip_prefix("images/").unwrap())
        } else {
            self.base_path.join(relative_path)
        };

        if file_path.exists() {
            fs::remove_file(&file_path)?;
            log::info!("Deleted image: {:?}", file_path);
        }

        Ok(())
    }

    /// Get absolute path for a relative image path
    pub fn get_absolute_path(&self, relative_path: &str) -> PathBuf {
        if relative_path.starts_with("images/") {
            self.base_path.join(relative_path.strip_prefix("images/").unwrap())
        } else {
            self.base_path.join(relative_path)
        }
    }

    /// Check if image exists
    pub fn image_exists(&self, relative_path: &str) -> bool {
        self.get_absolute_path(relative_path).exists()
    }

    /// Convert ImageData to PNG bytes
    fn image_data_to_png(&self, image_data: &ImageData) -> Result<Vec<u8>, ImageStorageError> {
        let width = image_data.width;
        let height = image_data.height;
        let bytes = &image_data.bytes;

        // Create PNG encoder
        let mut png_data = Vec::new();
        {
            let mut encoder = png::Encoder::new(&mut png_data, width as u32, height as u32);
            encoder.set_color(png::ColorType::Rgba);
            encoder.set_depth(png::BitDepth::Eight);
            
            let mut writer = encoder.write_header()
                .map_err(|e| ImageStorageError::InvalidFormat(format!("PNG header error: {}", e)))?;
            
            writer.write_image_data(bytes)
                .map_err(|e| ImageStorageError::InvalidFormat(format!("PNG write error: {}", e)))?;
        }

        Ok(png_data)
    }

    /// Get all image files in storage
    pub fn list_images(&self) -> Result<Vec<String>, ImageStorageError> {
        let mut images = Vec::new();
        
        if self.base_path.exists() {
            for entry in fs::read_dir(&self.base_path)? {
                let entry = entry?;
                let path = entry.path();
                
                if path.is_file() {
                    if let Some(extension) = path.extension() {
                        if extension == "png" || extension == "jpg" || extension == "jpeg" || extension == "gif" {
                            if let Some(filename) = path.file_name() {
                                if let Some(filename_str) = filename.to_str() {
                                    images.push(format!("images/{}", filename_str));
                                }
                            }
                        }
                    }
                }
            }
        }
        
        Ok(images)
    }

    /// Clean up orphaned images (images not referenced in any note)
    pub fn cleanup_orphaned_images(&self, referenced_images: &[String]) -> Result<usize, ImageStorageError> {
        let all_images = self.list_images()?;
        let mut deleted_count = 0;

        for image_path in all_images {
            if !referenced_images.contains(&image_path) {
                self.delete_image(&image_path)?;
                deleted_count += 1;
            }
        }

        log::info!("Cleaned up {} orphaned images", deleted_count);
        Ok(deleted_count)
    }
}

impl Default for ImageStorageManager {
    fn default() -> Self {
        Self::new().unwrap_or_else(|e| {
            log::error!("Failed to create image storage manager: {}", e);
            // Fallback to current directory
            Self {
                base_path: PathBuf::from("./inote_images"),
            }
        })
    }
}
