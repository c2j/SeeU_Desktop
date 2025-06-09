use std::path::PathBuf;
use std::fs;
use log;
use serde_json;

use crate::notebook::Notebook;
use crate::note::Note;
use crate::tag::Tag;
use crate::db_storage::DbStorageManager;
use crate::storage::StorageManager;

/// Migrate data from file storage to SQLite database
pub struct DataMigration {
    file_storage: StorageManager,
    db_storage: DbStorageManager,
}

impl DataMigration {
    /// Create a new data migration instance
    pub fn new(db_storage: DbStorageManager) -> Self {
        let file_storage = StorageManager::new();
        Self { file_storage, db_storage }
    }

    /// Create a new data migration instance with a reference
    pub fn new_with_ref(db_storage: &DbStorageManager) -> Self {
        let file_storage = StorageManager::new();
        Self { file_storage, db_storage: db_storage.clone() }
    }

    /// Migrate all data from file storage to SQLite database
    pub fn migrate(&self) -> Result<(), Box<dyn std::error::Error>> {
        log::info!("Starting data migration from files to SQLite database");

        // Migrate notebooks
        self.migrate_notebooks()?;

        // Migrate tags
        self.migrate_tags()?;

        // Migrate notes
        self.migrate_notes()?;

        log::info!("Data migration completed successfully");
        Ok(())
    }

    /// Migrate notebooks from file storage to SQLite database
    fn migrate_notebooks(&self) -> Result<(), Box<dyn std::error::Error>> {
        log::info!("Migrating notebooks");

        let notebooks_dir = self.file_storage.notebooks_path();
        if !notebooks_dir.exists() {
            log::info!("No notebooks directory found, skipping notebook migration");
            return Ok(());
        }

        let entries = fs::read_dir(notebooks_dir)?;

        for entry in entries {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() && path.extension().map_or(false, |ext| ext == "json") {
                match self.migrate_notebook(&path) {
                    Ok(_) => log::info!("Migrated notebook: {}", path.display()),
                    Err(err) => log::error!("Failed to migrate notebook {}: {}", path.display(), err),
                }
            }
        }

        Ok(())
    }

    /// Migrate a single notebook
    fn migrate_notebook(&self, path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        let json = fs::read_to_string(path)?;
        let notebook: Notebook = serde_json::from_str(&json)?;

        self.db_storage.save_notebook(&notebook)?;

        Ok(())
    }

    /// Migrate tags from file storage to SQLite database
    fn migrate_tags(&self) -> Result<(), Box<dyn std::error::Error>> {
        log::info!("Migrating tags");

        let tags_dir = self.file_storage.tags_path();
        if !tags_dir.exists() {
            log::info!("No tags directory found, skipping tag migration");
            return Ok(());
        }

        let entries = fs::read_dir(tags_dir)?;

        for entry in entries {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() && path.extension().map_or(false, |ext| ext == "json") {
                match self.migrate_tag(&path) {
                    Ok(_) => log::info!("Migrated tag: {}", path.display()),
                    Err(err) => log::error!("Failed to migrate tag {}: {}", path.display(), err),
                }
            }
        }

        Ok(())
    }

    /// Migrate a single tag
    fn migrate_tag(&self, path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        let json = fs::read_to_string(path)?;
        let tag: Tag = serde_json::from_str(&json)?;

        self.db_storage.save_tag(&tag)?;

        Ok(())
    }

    /// Migrate notes from file storage to SQLite database
    fn migrate_notes(&self) -> Result<(), Box<dyn std::error::Error>> {
        log::info!("Migrating notes");

        let notes_dir = self.file_storage.notes_path();
        if !notes_dir.exists() {
            log::info!("No notes directory found, skipping note migration");
            return Ok(());
        }

        // First, load all notebooks to get the notebook-note relationships
        let notebooks_dir = self.file_storage.notebooks_path();
        let mut note_to_notebook = std::collections::HashMap::new();

        if notebooks_dir.exists() {
            let entries = fs::read_dir(notebooks_dir)?;

            for entry in entries {
                let entry = entry?;
                let path = entry.path();

                if path.is_file() && path.extension().map_or(false, |ext| ext == "json") {
                    let json = fs::read_to_string(&path)?;
                    let notebook: Notebook = serde_json::from_str(&json)?;

                    for note_id in &notebook.note_ids {
                        note_to_notebook.insert(note_id.clone(), notebook.id.clone());
                    }
                }
            }
        }

        // Now migrate the notes
        let entries = fs::read_dir(notes_dir)?;

        for entry in entries {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() && path.extension().map_or(false, |ext| ext == "json") {
                match self.migrate_note(&path, &note_to_notebook) {
                    Ok(_) => log::info!("Migrated note: {}", path.display()),
                    Err(err) => log::error!("Failed to migrate note {}: {}", path.display(), err),
                }
            }
        }

        Ok(())
    }

    /// Migrate a single note
    fn migrate_note(&self, path: &PathBuf, note_to_notebook: &std::collections::HashMap<String, String>) -> Result<(), Box<dyn std::error::Error>> {
        let json = fs::read_to_string(path)?;
        let note: Note = serde_json::from_str(&json)?;

        // Find the notebook for this note
        let notebook_id = match note_to_notebook.get(&note.id) {
            Some(id) => id.clone(),
            None => {
                // If no notebook is found, use the first notebook or create a default one
                let notebooks = self.db_storage.list_notebooks()?;
                if let Some(notebook) = notebooks.first() {
                    notebook.id.clone()
                } else {
                    // Create a default notebook
                    let notebook = Notebook::new("Default".to_string(), "Default notebook".to_string());
                    self.db_storage.save_notebook(&notebook)?;
                    notebook.id
                }
            }
        };

        self.db_storage.save_note(&note, &notebook_id)?;

        Ok(())
    }
}
