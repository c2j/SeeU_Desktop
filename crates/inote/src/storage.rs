use std::fs;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use crate::{Notebook, Note, Tag};

/// Storage manager for iNote
pub struct StorageManager {
    base_path: PathBuf,
}

impl StorageManager {
    /// Create a new storage manager
    pub fn new() -> Self {
        let mut base_path = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."));
        base_path.push("seeu_desktop");
        base_path.push("inote");

        // Create directories if they don't exist
        match fs::create_dir_all(&base_path) {
            Ok(_) => {
                // Directory created successfully
            },
            Err(err) => {
                log::error!("Failed to create storage directory: {}", err);
                // Fallback to current directory
                base_path = PathBuf::from("./inote_data");
                if let Err(err) = fs::create_dir_all(&base_path) {
                    log::error!("Failed to create fallback storage directory: {}", err);
                }
            }
        }

        // Test if directory is writable
        let test_file = base_path.join("test_write.tmp");
        match fs::write(&test_file, b"test") {
            Ok(_) => {
                fs::remove_file(test_file).ok();
            },
            Err(err) => {
                log::error!("Storage directory is not writable: {}", err);
            }
        }

        Self { base_path }
    }

    /// Get the path for notebooks
    pub fn notebooks_path(&self) -> PathBuf {
        let mut path = self.base_path.clone();
        path.push("notebooks");
        fs::create_dir_all(&path).ok();
        path
    }

    /// Get the path for notes
    pub fn notes_path(&self) -> PathBuf {
        let mut path = self.base_path.clone();
        path.push("notes");
        fs::create_dir_all(&path).ok();
        path
    }

    /// Get the path for tags
    pub fn tags_path(&self) -> PathBuf {
        let mut path = self.base_path.clone();
        path.push("tags");
        fs::create_dir_all(&path).ok();
        path
    }

    /// Get the path for attachments
    pub fn attachments_path(&self) -> PathBuf {
        let mut path = self.base_path.clone();
        path.push("attachments");
        fs::create_dir_all(&path).ok();
        path
    }

    /// Save a notebook
    pub fn save_notebook(&self, notebook: &Notebook) -> Result<(), Box<dyn std::error::Error>> {
        let notebooks_dir = self.notebooks_path();

        // Ensure directory exists
        if let Err(err) = fs::create_dir_all(&notebooks_dir) {
            log::error!("Failed to create notebooks directory: {}", err);
            return Err(Box::new(err));
        }

        let path = notebooks_dir.join(format!("{}.json", notebook.id));

        // Serialize notebook to JSON
        let json = match serde_json::to_string_pretty(notebook) {
            Ok(json) => json,
            Err(err) => {
                log::error!("Failed to serialize notebook: {}", err);
                return Err(Box::new(err));
            }
        };

        // Write JSON to file
        match fs::write(&path, &json) {
            Ok(_) => Ok(()),
            Err(err) => {
                log::error!("Failed to write notebook file: {}", err);
                Err(Box::new(err))
            }
        }
    }

    /// Load a notebook
    pub fn load_notebook(&self, id: &str) -> Result<Notebook, Box<dyn std::error::Error>> {
        let path = self.notebooks_path().join(format!("{}.json", id));
        let json = fs::read_to_string(path)?;
        let notebook = serde_json::from_str(&json)?;
        Ok(notebook)
    }

    /// List all notebooks
    pub fn list_notebooks(&self) -> Result<Vec<Notebook>, Box<dyn std::error::Error>> {
        let mut notebooks = Vec::new();

        for entry in fs::read_dir(self.notebooks_path())? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() && path.extension().map_or(false, |ext| ext == "json") {
                if let Ok(json) = fs::read_to_string(&path) {
                    if let Ok(notebook) = serde_json::from_str(&json) {
                        notebooks.push(notebook);
                    }
                }
            }
        }

        Ok(notebooks)
    }

    /// Save a note
    pub fn save_note(&self, note: &Note) -> Result<(), Box<dyn std::error::Error>> {
        let path = self.notes_path().join(format!("{}.json", note.id));
        let json = serde_json::to_string_pretty(note)?;
        fs::write(path, json)?;
        Ok(())
    }

    /// Load a note
    pub fn load_note(&self, id: &str) -> Result<Note, Box<dyn std::error::Error>> {
        let path = self.notes_path().join(format!("{}.json", id));
        let json = fs::read_to_string(path)?;
        let note = serde_json::from_str(&json)?;
        Ok(note)
    }

    /// Delete a note
    pub fn delete_note(&self, id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let path = self.notes_path().join(format!("{}.json", id));
        if path.exists() {
            fs::remove_file(&path)?;
        }
        Ok(())
    }

    /// Delete a notebook
    pub fn delete_notebook(&self, id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let path = self.notebooks_path().join(format!("{}.json", id));
        if path.exists() {
            fs::remove_file(&path)?;
        }
        Ok(())
    }

    /// Save a tag
    pub fn save_tag(&self, tag: &Tag) -> Result<(), Box<dyn std::error::Error>> {
        let path = self.tags_path().join(format!("{}.json", tag.id));
        let json = serde_json::to_string_pretty(tag)?;
        fs::write(path, json)?;
        Ok(())
    }

    /// Load a tag
    pub fn load_tag(&self, id: &str) -> Result<Tag, Box<dyn std::error::Error>> {
        let path = self.tags_path().join(format!("{}.json", id));
        let json = fs::read_to_string(path)?;
        let tag = serde_json::from_str(&json)?;
        Ok(tag)
    }

    /// List all tags
    pub fn list_tags(&self) -> Result<Vec<Tag>, Box<dyn std::error::Error>> {
        let mut tags = Vec::new();

        for entry in fs::read_dir(self.tags_path())? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() && path.extension().map_or(false, |ext| ext == "json") {
                if let Ok(json) = fs::read_to_string(&path) {
                    if let Ok(tag) = serde_json::from_str(&json) {
                        tags.push(tag);
                    }
                }
            }
        }

        Ok(tags)
    }

    /// Delete a tag
    pub fn delete_tag(&self, id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let path = self.tags_path().join(format!("{}.json", id));
        if path.exists() {
            fs::remove_file(&path)?;
        }
        Ok(())
    }
}