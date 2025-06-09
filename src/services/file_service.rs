use std::path::Path;
use std::fs;

/// File service for handling file operations
pub struct FileService {
    // TODO: Add fields for file service
}

impl FileService {
    /// Create a new file service
    pub fn new() -> Self {
        Self {}
    }

    /// List files in a directory
    pub fn list_files(&self, path: &Path) -> Result<Vec<fs::DirEntry>, std::io::Error> {
        let entries = fs::read_dir(path)?
            .filter_map(|entry| entry.ok())
            .collect();

        Ok(entries)
    }

    /// Read a file
    pub fn read_file(&self, path: &Path) -> Result<String, std::io::Error> {
        fs::read_to_string(path)
    }

    /// Write to a file
    pub fn write_file(&self, path: &Path, content: &str) -> Result<(), std::io::Error> {
        fs::write(path, content)
    }
}