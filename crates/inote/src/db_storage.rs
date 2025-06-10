use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, Result as SqlResult, Error as SqlError};
use r2d2::{Pool, PooledConnection};
use r2d2_sqlite::SqliteConnectionManager;
use log;

use crate::notebook::Notebook;
use crate::note::Note;
use crate::tag::Tag;

/// Database schema version
const DB_VERSION: i32 = 1;

/// SQLite storage manager for iNote
#[derive(Clone)]
pub struct DbStorageManager {
    pool: Pool<SqliteConnectionManager>,
    db_path: PathBuf,
}

/// Database connection type
type DbConnection = PooledConnection<SqliteConnectionManager>;

impl DbStorageManager {
    /// Create a new storage manager
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        // Get database path
        let mut db_path = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."));
        db_path.push("seeu_desktop");
        db_path.push("inote");

        // Create directories if they don't exist
        std::fs::create_dir_all(&db_path)?;

        db_path.push("inote.db");

        log::info!("Database path: {}", db_path.display());

        // Create connection manager
        let manager = SqliteConnectionManager::file(&db_path);
        let pool = Pool::new(manager)?;

        // Initialize database
        let storage = Self { pool, db_path };
        storage.init_database()?;

        Ok(storage)
    }

    /// Get a database connection from the pool
    fn get_connection(&self) -> Result<DbConnection, Box<dyn std::error::Error>> {
        Ok(self.pool.get()?)
    }

    /// Save a notebook
    pub fn save_notebook(&self, notebook: &Notebook) -> Result<(), Box<dyn std::error::Error>> {
        let conn = self.get_connection()?;

        conn.execute(
            "INSERT OR REPLACE INTO notebooks (id, name, description, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?)",
            params![
                notebook.id,
                notebook.name,
                notebook.description,
                notebook.created_at.to_rfc3339(),
                notebook.updated_at.to_rfc3339()
            ],
        )?;

        Ok(())
    }

    /// Load a notebook
    pub fn load_notebook(&self, id: &str) -> Result<Notebook, Box<dyn std::error::Error>> {
        let conn = self.get_connection()?;

        let notebook = conn.query_row(
            "SELECT id, name, description, created_at, updated_at FROM notebooks WHERE id = ?",
            params![id],
            |row| {
                let id: String = row.get(0)?;
                let name: String = row.get(1)?;
                let description: String = row.get(2)?;
                let created_at_str: String = row.get(3)?;
                let updated_at_str: String = row.get(4)?;

                let created_at = DateTime::parse_from_rfc3339(&created_at_str)
                    .map_err(|e| SqlError::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e)))?
                    .with_timezone(&Utc);

                let updated_at = DateTime::parse_from_rfc3339(&updated_at_str)
                    .map_err(|e| SqlError::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e)))?
                    .with_timezone(&Utc);

                let mut notebook = Notebook {
                    id,
                    name,
                    description,
                    created_at,
                    updated_at,
                    note_ids: Vec::new(),
                    expanded: true,
                };

                // Load note IDs for this notebook
                let mut stmt = conn.prepare("SELECT id FROM notes WHERE notebook_id = ?")?;
                let note_ids_iter = stmt.query_map(params![notebook.id], |row| {
                    let id: String = row.get(0)?;
                    Ok(id)
                })?;

                for note_id in note_ids_iter {
                    notebook.note_ids.push(note_id?);
                }

                Ok(notebook)
            },
        )?;

        Ok(notebook)
    }

    /// List all notebooks
    pub fn list_notebooks(&self) -> Result<Vec<Notebook>, Box<dyn std::error::Error>> {
        let conn = self.get_connection()?;
        let mut notebooks = Vec::new();

        let mut stmt = conn.prepare(
            "SELECT id, name, description, created_at, updated_at FROM notebooks ORDER BY name"
        )?;

        let notebook_iter = stmt.query_map([], |row| {
            let id: String = row.get(0)?;
            let name: String = row.get(1)?;
            let description: String = row.get(2)?;
            let created_at_str: String = row.get(3)?;
            let updated_at_str: String = row.get(4)?;

            let created_at = DateTime::parse_from_rfc3339(&created_at_str)
                .map_err(|e| SqlError::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e)))?
                .with_timezone(&Utc);

            let updated_at = DateTime::parse_from_rfc3339(&updated_at_str)
                .map_err(|e| SqlError::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e)))?
                .with_timezone(&Utc);

            Ok((id, name, description, created_at, updated_at))
        })?;

        for notebook_result in notebook_iter {
            let (id, name, description, created_at, updated_at) = notebook_result?;

            let mut notebook = Notebook {
                id,
                name,
                description,
                created_at,
                updated_at,
                note_ids: Vec::new(),
                expanded: true,
            };

            // Load note IDs for this notebook
            let mut stmt = conn.prepare("SELECT id FROM notes WHERE notebook_id = ?")?;
            let note_ids_iter = stmt.query_map(params![notebook.id], |row| {
                let id: String = row.get(0)?;
                Ok(id)
            })?;

            for note_id in note_ids_iter {
                notebook.note_ids.push(note_id?);
            }

            notebooks.push(notebook);
        }

        Ok(notebooks)
    }

    /// Save a note
    pub fn save_note(&self, note: &Note, notebook_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut conn = self.get_connection()?;

        // Begin transaction
        let tx = conn.transaction()?;

        // Save note
        tx.execute(
            "INSERT OR REPLACE INTO notes (id, notebook_id, title, content, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?)",
            params![
                note.id,
                notebook_id,
                note.title,
                note.content,
                note.created_at.to_rfc3339(),
                note.updated_at.to_rfc3339()
            ],
        )?;

        // Delete existing tag associations
        tx.execute(
            "DELETE FROM note_tags WHERE note_id = ?",
            params![note.id],
        )?;

        // Add tag associations
        for tag_id in &note.tag_ids {
            tx.execute(
                "INSERT INTO note_tags (note_id, tag_id) VALUES (?, ?)",
                params![note.id, tag_id],
            )?;
        }

        // Delete existing attachments
        tx.execute(
            "DELETE FROM attachments WHERE note_id = ?",
            params![note.id],
        )?;

        // Add attachments
        for attachment in &note.attachments {
            tx.execute(
                "INSERT INTO attachments (id, note_id, name, file_path, file_type, created_at)
                 VALUES (?, ?, ?, ?, ?, ?)",
                params![
                    attachment.id,
                    note.id,
                    attachment.name,
                    attachment.file_path,
                    attachment.file_type,
                    attachment.created_at.to_rfc3339()
                ],
            )?;
        }

        // Commit transaction
        tx.commit()?;

        Ok(())
    }

    /// Load a note
    pub fn load_note(&self, id: &str) -> Result<Note, Box<dyn std::error::Error>> {
        let conn = self.get_connection()?;

        let note = conn.query_row(
            "SELECT id, title, content, created_at, updated_at FROM notes WHERE id = ?",
            params![id],
            |row| {
                let id: String = row.get(0)?;
                let title: String = row.get(1)?;
                let content: String = row.get(2)?;
                let created_at_str: String = row.get(3)?;
                let updated_at_str: String = row.get(4)?;

                let created_at = DateTime::parse_from_rfc3339(&created_at_str)
                    .map_err(|e| SqlError::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e)))?
                    .with_timezone(&Utc);

                let updated_at = DateTime::parse_from_rfc3339(&updated_at_str)
                    .map_err(|e| SqlError::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e)))?
                    .with_timezone(&Utc);

                let mut note = Note {
                    id,
                    title,
                    content,
                    created_at,
                    updated_at,
                    tag_ids: Vec::new(),
                    attachments: Vec::new(),
                };

                // Load tag IDs for this note
                let mut stmt = conn.prepare("SELECT tag_id FROM note_tags WHERE note_id = ?")?;
                let tag_ids_iter = stmt.query_map(params![note.id], |row| {
                    let id: String = row.get(0)?;
                    Ok(id)
                })?;

                for tag_id in tag_ids_iter {
                    note.tag_ids.push(tag_id?);
                }

                // Load attachments for this note
                if let Err(err) = Self::load_attachments_for_note(&mut note, &conn) {
                    log::error!("Error loading attachments for note {}: {}", note.id, err);
                }

                Ok(note)
            },
        )?;

        Ok(note)
    }

    /// Delete a note
    pub fn delete_note(&self, id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut conn = self.get_connection()?;

        // Begin transaction
        let tx = conn.transaction()?;

        // Delete note tags
        tx.execute(
            "DELETE FROM note_tags WHERE note_id = ?",
            params![id],
        )?;

        // Delete note
        tx.execute(
            "DELETE FROM notes WHERE id = ?",
            params![id],
        )?;

        // Commit transaction
        tx.commit()?;

        Ok(())
    }

    /// Save a tag
    pub fn save_tag(&self, tag: &Tag) -> Result<(), Box<dyn std::error::Error>> {
        let conn = self.get_connection()?;

        conn.execute(
            "INSERT OR REPLACE INTO tags (id, name, color, created_at)
             VALUES (?, ?, ?, ?)",
            params![
                tag.id,
                tag.name,
                tag.color,
                tag.created_at.to_rfc3339()
            ],
        )?;

        Ok(())
    }

    /// Load a tag
    pub fn load_tag(&self, id: &str) -> Result<Tag, Box<dyn std::error::Error>> {
        let conn = self.get_connection()?;

        let tag = conn.query_row(
            "SELECT id, name, color, created_at FROM tags WHERE id = ?",
            params![id],
            |row| {
                let id: String = row.get(0)?;
                let name: String = row.get(1)?;
                let color: String = row.get(2)?;
                let created_at_str: String = row.get(3)?;

                let created_at = DateTime::parse_from_rfc3339(&created_at_str)
                    .map_err(|e| SqlError::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e)))?
                    .with_timezone(&Utc);

                Ok(Tag {
                    id,
                    name,
                    color,
                    created_at,
                })
            },
        )?;

        Ok(tag)
    }

    /// List all tags
    pub fn list_tags(&self) -> Result<Vec<Tag>, Box<dyn std::error::Error>> {
        let conn = self.get_connection()?;
        let mut tags = Vec::new();

        let mut stmt = conn.prepare(
            "SELECT id, name, color, created_at FROM tags ORDER BY name"
        )?;

        let tag_iter = stmt.query_map([], |row| {
            let id: String = row.get(0)?;
            let name: String = row.get(1)?;
            let color: String = row.get(2)?;
            let created_at_str: String = row.get(3)?;

            let created_at = DateTime::parse_from_rfc3339(&created_at_str)
                .map_err(|e| SqlError::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e)))?
                .with_timezone(&Utc);

            Ok(Tag {
                id,
                name,
                color,
                created_at,
            })
        })?;

        for tag_result in tag_iter {
            tags.push(tag_result?);
        }

        Ok(tags)
    }

    /// Delete a notebook
    pub fn delete_notebook(&self, id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut conn = self.get_connection()?;

        // Begin transaction
        let tx = conn.transaction()?;

        // Get all note IDs in this notebook
        let note_ids: Vec<String> = {
            let mut stmt = tx.prepare("SELECT id FROM notes WHERE notebook_id = ?")?;
            let note_ids: Result<Vec<String>, _> = stmt.query_map(params![id], |row| {
                Ok(row.get::<_, String>(0)?)
            })?.collect();
            note_ids?
        };

        // Delete note tags for all notes in this notebook
        for note_id in &note_ids {
            tx.execute(
                "DELETE FROM note_tags WHERE note_id = ?",
                params![note_id],
            )?;
        }

        // Delete all notes in this notebook
        tx.execute(
            "DELETE FROM notes WHERE notebook_id = ?",
            params![id],
        )?;

        // Delete the notebook
        tx.execute(
            "DELETE FROM notebooks WHERE id = ?",
            params![id],
        )?;

        // Commit transaction
        tx.commit()?;

        Ok(())
    }

    /// Delete a tag
    pub fn delete_tag(&self, id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut conn = self.get_connection()?;

        // Begin transaction
        let tx = conn.transaction()?;

        // Delete tag associations
        tx.execute(
            "DELETE FROM note_tags WHERE tag_id = ?",
            params![id],
        )?;

        // Delete tag
        tx.execute(
            "DELETE FROM tags WHERE id = ?",
            params![id],
        )?;

        // Commit transaction
        tx.commit()?;

        Ok(())
    }

    /// Search notes
    pub fn search_notes(&self, query: &str) -> Result<Vec<Note>, Box<dyn std::error::Error>> {
        if query.is_empty() {
            return Ok(Vec::new());
        }

        let conn = self.get_connection()?;
        let mut notes = Vec::new();

        // Search in title and content
        let search_query = format!("%{}%", query.to_lowercase());

        let mut stmt = conn.prepare(
            "SELECT id, title, content, created_at, updated_at
             FROM notes
             WHERE LOWER(title) LIKE ? OR LOWER(content) LIKE ?
             ORDER BY updated_at DESC"
        )?;

        let note_iter = stmt.query_map(params![search_query, search_query], |row| {
            let id: String = row.get(0)?;
            let title: String = row.get(1)?;
            let content: String = row.get(2)?;
            let created_at_str: String = row.get(3)?;
            let updated_at_str: String = row.get(4)?;

            let created_at = DateTime::parse_from_rfc3339(&created_at_str)
                .map_err(|e| SqlError::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e)))?
                .with_timezone(&Utc);

            let updated_at = DateTime::parse_from_rfc3339(&updated_at_str)
                .map_err(|e| SqlError::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e)))?
                .with_timezone(&Utc);

            let mut note = Note {
                id,
                title,
                content,
                created_at,
                updated_at,
                tag_ids: Vec::new(),
                attachments: Vec::new(),
            };

            // Load tag IDs for this note
            let mut stmt = conn.prepare("SELECT tag_id FROM note_tags WHERE note_id = ?")?;
            let tag_ids_iter = stmt.query_map(params![note.id], |row| {
                let id: String = row.get(0)?;
                Ok(id)
            })?;

            for tag_id in tag_ids_iter {
                note.tag_ids.push(tag_id?);
            }

            Ok(note)
        })?;

        for note_result in note_iter {
            let mut note = note_result?;
            // Load attachments for this note
            if let Err(err) = Self::load_attachments_for_note(&mut note, &conn) {
                log::error!("Error loading attachments for note {}: {}", note.id, err);
            }
            notes.push(note);
        }

        Ok(notes)
    }

    /// Get notebook ID for a note
    pub fn get_notebook_id_for_note(&self, note_id: &str) -> Result<String, Box<dyn std::error::Error>> {
        let conn = self.get_connection()?;

        let notebook_id: String = conn.query_row(
            "SELECT notebook_id FROM notes WHERE id = ?",
            params![note_id],
            |row| row.get(0),
        )?;

        Ok(notebook_id)
    }

    /// Get notes for a notebook
    pub fn get_notes_for_notebook(&self, notebook_id: &str) -> Result<Vec<Note>, Box<dyn std::error::Error>> {
        let conn = self.get_connection()?;
        let mut notes = Vec::new();

        let mut stmt = conn.prepare(
            "SELECT id, title, content, created_at, updated_at
             FROM notes
             WHERE notebook_id = ?
             ORDER BY updated_at DESC"
        )?;

        let note_iter = stmt.query_map(params![notebook_id], |row| {
            let id: String = row.get(0)?;
            let title: String = row.get(1)?;
            let content: String = row.get(2)?;
            let created_at_str: String = row.get(3)?;
            let updated_at_str: String = row.get(4)?;

            let created_at = DateTime::parse_from_rfc3339(&created_at_str)
                .map_err(|e| SqlError::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e)))?
                .with_timezone(&Utc);

            let updated_at = DateTime::parse_from_rfc3339(&updated_at_str)
                .map_err(|e| SqlError::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e)))?
                .with_timezone(&Utc);

            let mut note = Note {
                id,
                title,
                content,
                created_at,
                updated_at,
                tag_ids: Vec::new(),
                attachments: Vec::new(),
            };

            // Load tag IDs for this note
            let mut stmt = conn.prepare("SELECT tag_id FROM note_tags WHERE note_id = ?")?;
            let tag_ids_iter = stmt.query_map(params![note.id], |row| {
                let id: String = row.get(0)?;
                Ok(id)
            })?;

            for tag_id in tag_ids_iter {
                note.tag_ids.push(tag_id?);
            }

            Ok(note)
        })?;

        for note_result in note_iter {
            let mut note = note_result?;
            // Load attachments for this note
            if let Err(err) = Self::load_attachments_for_note(&mut note, &conn) {
                log::error!("Error loading attachments for note {}: {}", note.id, err);
            }
            notes.push(note);
        }

        Ok(notes)
    }

    /// Get notes for a tag
    pub fn get_notes_for_tag(&self, tag_id: &str) -> Result<Vec<Note>, Box<dyn std::error::Error>> {
        let conn = self.get_connection()?;
        let mut notes = Vec::new();

        // First, check if the tag exists
        let tag_exists: bool = match conn.query_row(
            "SELECT COUNT(*) FROM tags WHERE id = ?",
            params![tag_id],
            |row| row.get::<_, i64>(0)
        ) {
            Ok(count) => count > 0,
            Err(_) => false
        };

        if !tag_exists {
            return Ok(Vec::new());
        }

        // Check if there are any note-tag associations
        let association_count: i64 = match conn.query_row(
            "SELECT COUNT(*) FROM note_tags WHERE tag_id = ?",
            params![tag_id],
            |row| row.get(0)
        ) {
            Ok(count) => count,
            Err(_) => 0
        };

        if association_count == 0 {
            return Ok(Vec::new());
        }

        let mut stmt = conn.prepare(
            "SELECT n.id, n.title, n.content, n.created_at, n.updated_at
             FROM notes n
             JOIN note_tags nt ON n.id = nt.note_id
             WHERE nt.tag_id = ?
             ORDER BY n.updated_at DESC"
        )?;

        let note_iter = stmt.query_map(params![tag_id], |row| {
            let id: String = row.get(0)?;
            let title: String = row.get(1)?;
            let content: String = row.get(2)?;
            let created_at_str: String = row.get(3)?;
            let updated_at_str: String = row.get(4)?;

            log::info!("DB: Found note {} ({}) with tag ID {}", id, title, tag_id);

            let created_at = DateTime::parse_from_rfc3339(&created_at_str)
                .map_err(|e| SqlError::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e)))?
                .with_timezone(&Utc);

            let updated_at = DateTime::parse_from_rfc3339(&updated_at_str)
                .map_err(|e| SqlError::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e)))?
                .with_timezone(&Utc);

            let mut note = Note {
                id,
                title,
                content,
                created_at,
                updated_at,
                tag_ids: Vec::new(),
                attachments: Vec::new(),
            };

            // Load tag IDs for this note
            let mut stmt = conn.prepare("SELECT tag_id FROM note_tags WHERE note_id = ?")?;
            let tag_ids_iter = stmt.query_map(params![note.id], |row| {
                let id: String = row.get(0)?;
                Ok(id)
            })?;

            for tag_id_result in tag_ids_iter {
                let tag_id = tag_id_result?;
                log::info!("DB: Note {} has tag ID {}", note.id, tag_id);
                note.tag_ids.push(tag_id);
            }

            Ok(note)
        })?;

        for note_result in note_iter {
            match note_result {
                Ok(mut note) => {
                    // Load attachments for this note
                    if let Err(err) = Self::load_attachments_for_note(&mut note, &conn) {
                        log::error!("DB: Error loading attachments for note {}: {}", note.id, err);
                    }
                    notes.push(note);
                },
                Err(err) => {
                    log::error!("DB: Error processing note result: {}", err);
                }
            }
        }

        log::info!("DB: Returning {} notes for tag ID {}", notes.len(), tag_id);
        Ok(notes)
    }

    /// Initialize database
    fn init_database(&self) -> Result<(), Box<dyn std::error::Error>> {
        let conn = self.pool.get()?;

        // Create tables if they don't exist
        self.create_tables(&conn)?;

        // Check database version
        self.check_version(&conn)?;

        Ok(())
    }

    /// Load attachments for a note
    fn load_attachments_for_note(note: &mut Note, conn: &Connection) -> Result<(), Box<dyn std::error::Error>> {
        let mut stmt = conn.prepare("SELECT id, name, file_path, file_type, created_at FROM attachments WHERE note_id = ?")?;
        let attachment_iter = stmt.query_map(params![note.id], |row| {
            let id: String = row.get(0)?;
            let name: String = row.get(1)?;
            let file_path: String = row.get(2)?;
            let file_type: String = row.get(3)?;
            let created_at_str: String = row.get(4)?;

            let created_at = DateTime::parse_from_rfc3339(&created_at_str)
                .map_err(|e| SqlError::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e)))?
                .with_timezone(&Utc);

            Ok(crate::note::Attachment {
                id,
                name,
                file_path,
                file_type,
                created_at,
            })
        })?;

        for attachment in attachment_iter {
            note.attachments.push(attachment?);
        }

        Ok(())
    }

    /// Create database tables
    fn create_tables(&self, conn: &Connection) -> SqlResult<()> {
        // Create version table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS version (
                id INTEGER PRIMARY KEY,
                version INTEGER NOT NULL
            )",
            [],
        )?;

        // Create notebooks table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS notebooks (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                description TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
            [],
        )?;

        // Create notes table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS notes (
                id TEXT PRIMARY KEY,
                notebook_id TEXT NOT NULL,
                title TEXT NOT NULL,
                content TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                FOREIGN KEY (notebook_id) REFERENCES notebooks(id) ON DELETE CASCADE
            )",
            [],
        )?;

        // Create tags table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS tags (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                color TEXT NOT NULL,
                created_at TEXT NOT NULL
            )",
            [],
        )?;

        // Create note_tags table (many-to-many relationship)
        conn.execute(
            "CREATE TABLE IF NOT EXISTS note_tags (
                note_id TEXT NOT NULL,
                tag_id TEXT NOT NULL,
                PRIMARY KEY (note_id, tag_id),
                FOREIGN KEY (note_id) REFERENCES notes(id) ON DELETE CASCADE,
                FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
            )",
            [],
        )?;

        // Create attachments table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS attachments (
                id TEXT PRIMARY KEY,
                note_id TEXT NOT NULL,
                name TEXT NOT NULL,
                file_path TEXT NOT NULL,
                file_type TEXT NOT NULL,
                created_at TEXT NOT NULL,
                FOREIGN KEY (note_id) REFERENCES notes(id) ON DELETE CASCADE
            )",
            [],
        )?;

        // Enable foreign keys
        conn.execute("PRAGMA foreign_keys = ON", [])?;

        Ok(())
    }

    /// Check database version and upgrade if needed
    fn check_version(&self, conn: &Connection) -> Result<(), Box<dyn std::error::Error>> {
        // Check if version table is empty
        let count: i32 = conn.query_row(
            "SELECT COUNT(*) FROM version",
            [],
            |row| row.get(0),
        )?;

        if count == 0 {
            // Insert initial version
            conn.execute(
                "INSERT INTO version (id, version) VALUES (1, ?)",
                params![DB_VERSION],
            )?;
        } else {
            // Get current version
            let current_version: i32 = conn.query_row(
                "SELECT version FROM version WHERE id = 1",
                [],
                |row| row.get(0),
            )?;

            // Upgrade database if needed
            if current_version < DB_VERSION {
                self.upgrade_database(conn, current_version)?;

                // Update version
                conn.execute(
                    "UPDATE version SET version = ? WHERE id = 1",
                    params![DB_VERSION],
                )?;
            }
        }

        Ok(())
    }

    /// Upgrade database to the latest version
    fn upgrade_database(&self, conn: &Connection, current_version: i32) -> Result<(), Box<dyn std::error::Error>> {
        log::info!("Upgrading database from version {} to {}", current_version, DB_VERSION);

        // Implement database migrations here when needed
        // For now, we don't need any migrations

        Ok(())
    }
}
