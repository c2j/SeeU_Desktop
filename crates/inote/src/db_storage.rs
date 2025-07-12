use std::path::PathBuf;

use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, Result as SqlResult, Error as SqlError};
use r2d2::{Pool, PooledConnection};
use r2d2_sqlite::SqliteConnectionManager;
use log;

use crate::notebook::Notebook;
use crate::note::Note;
use crate::tag::Tag;
use crate::mcp_server::McpServerRecord;
use crate::db_state::RecentNoteAccess;

/// Database schema version
const DB_VERSION: i32 = 2;

/// SQLite storage manager for iNote
#[derive(Clone)]
pub struct DbStorageManager {
    pool: Pool<SqliteConnectionManager>,
    db_path: PathBuf,
}

/// Database connection type
type DbConnection = PooledConnection<SqliteConnectionManager>;

impl DbStorageManager {
    /// Create a placeholder storage manager (will be initialized later)
    pub fn new_placeholder() -> Self {
        // Create a minimal in-memory database for placeholder
        let manager = SqliteConnectionManager::memory();
        let pool = Pool::new(manager).expect("Failed to create placeholder pool");
        Self {
            pool,
            db_path: PathBuf::from(":placeholder:")
        }
    }

    /// Create a new storage manager with in-memory database (for testing)
    pub fn new_memory() -> Result<Self, Box<dyn std::error::Error>> {
        let manager = SqliteConnectionManager::memory();
        let pool = Pool::new(manager)?;
        let storage = Self {
            pool,
            db_path: PathBuf::from(":memory:")
        };
        storage.init_database()?;
        Ok(storage)
    }

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

    /// Initialize storage asynchronously (replaces placeholder)
    pub fn initialize_async(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Only initialize if this is a placeholder
        if self.db_path.to_string_lossy() == ":placeholder:" {
            // Get database path
            let mut db_path = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."));
            db_path.push("seeu_desktop");
            db_path.push("inote");

            // Create directories if they don't exist
            std::fs::create_dir_all(&db_path)?;

            db_path.push("inote.db");

            log::info!("Initializing database asynchronously: {}", db_path.display());

            // Create connection manager
            let manager = SqliteConnectionManager::file(&db_path);
            let pool = Pool::new(manager)?;

            // Replace placeholder with real database
            self.pool = pool;
            self.db_path = db_path;

            // Initialize database
            self.init_database()?;

            log::info!("Database initialized successfully");
        }

        Ok(())
    }

    /// Check if this is a placeholder storage
    pub fn is_placeholder(&self) -> bool {
        self.db_path.to_string_lossy() == ":placeholder:"
    }

    /// Get a database connection from the pool
    fn get_connection(&self) -> Result<DbConnection, Box<dyn std::error::Error>> {
        Ok(self.pool.get()?)
    }

    /// Save a notebook
    pub fn save_notebook(&self, notebook: &Notebook) -> Result<(), Box<dyn std::error::Error>> {
        let conn = self.get_connection()?;

        // 使用更安全的INSERT方式，避免"Execute returned results"错误
        let update_result = conn.execute(
            "UPDATE notebooks SET name = ?, description = ?, updated_at = ?, sort_order = ? WHERE id = ?",
            params![
                notebook.name,
                notebook.description,
                notebook.updated_at.to_rfc3339(),
                notebook.sort_order,
                notebook.id
            ],
        );

        match update_result {
            Ok(0) => {
                // 没有更新任何行，说明笔记本不存在，需要插入
                conn.execute(
                    "INSERT INTO notebooks (id, name, description, created_at, updated_at, sort_order) VALUES (?, ?, ?, ?, ?, ?)",
                    params![
                        notebook.id,
                        notebook.name,
                        notebook.description,
                        notebook.created_at.to_rfc3339(),
                        notebook.updated_at.to_rfc3339(),
                        notebook.sort_order
                    ],
                )?;
            }
            Ok(_) => {
                // 更新成功
            }
            Err(e) => {
                return Err(Box::new(e));
            }
        }

        Ok(())
    }

    /// Load a notebook
    pub fn load_notebook(&self, id: &str) -> Result<Notebook, Box<dyn std::error::Error>> {
        let conn = self.get_connection()?;

        let notebook = conn.query_row(
            "SELECT id, name, description, created_at, updated_at, sort_order FROM notebooks WHERE id = ?",
            params![id],
            |row| {
                let id: String = row.get(0)?;
                let name: String = row.get(1)?;
                let description: String = row.get(2)?;
                let created_at_str: String = row.get(3)?;
                let updated_at_str: String = row.get(4)?;
                let sort_order: i32 = row.get(5).unwrap_or(0);

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
                    sort_order,
                };

                // Load note IDs for this notebook (ordered by created_at DESC for newest first)
                let mut stmt = conn.prepare("SELECT id FROM notes WHERE notebook_id = ? ORDER BY created_at DESC")?;
                let note_ids_iter = stmt.query_map(params![notebook.id], |row| {
                    let id: String = row.get(0)?;
                    Ok(id)
                })?;

                for note_id in note_ids_iter {
                    notebook.note_ids.push(note_id?); // 直接 push，因为查询已经按 created_at DESC 排序
                }

                Ok(notebook)
            },
        )?;

        Ok(notebook)
    }

    /// List all notebooks
    pub fn list_notebooks(&self) -> Result<Vec<Notebook>, Box<dyn std::error::Error>> {
        // Return empty list if this is a placeholder
        if self.is_placeholder() {
            return Ok(Vec::new());
        }

        let conn = self.get_connection()?;
        let mut notebooks = Vec::new();

        let mut stmt = conn.prepare(
            "SELECT id, name, description, created_at, updated_at, sort_order FROM notebooks ORDER BY sort_order, created_at"
        )?;

        let notebook_iter = stmt.query_map([], |row| {
            let id: String = row.get(0)?;
            let name: String = row.get(1)?;
            let description: String = row.get(2)?;
            let created_at_str: String = row.get(3)?;
            let updated_at_str: String = row.get(4)?;
            let sort_order: i32 = row.get(5).unwrap_or(0);

            let created_at = DateTime::parse_from_rfc3339(&created_at_str)
                .map_err(|e| SqlError::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e)))?
                .with_timezone(&Utc);

            let updated_at = DateTime::parse_from_rfc3339(&updated_at_str)
                .map_err(|e| SqlError::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e)))?
                .with_timezone(&Utc);

            Ok((id, name, description, created_at, updated_at, sort_order))
        })?;

        for notebook_result in notebook_iter {
            let (id, name, description, created_at, updated_at, sort_order) = notebook_result?;

            let mut notebook = Notebook {
                id,
                name,
                description,
                created_at,
                updated_at,
                note_ids: Vec::new(),
                expanded: true,
                sort_order,
            };

            // Load note IDs for this notebook
            let mut stmt = conn.prepare("SELECT id FROM notes WHERE notebook_id = ? ORDER BY created_at DESC")?;
            let note_ids_iter = stmt.query_map(params![notebook.id], |row| {
                let id: String = row.get(0)?;
                Ok(id)
            })?;

            let mut note_count = 0;
            for note_id in note_ids_iter {
                notebook.note_ids.push(note_id?); // 直接 push，因为查询已经按 created_at DESC 排序
                note_count += 1;
            }

            log::debug!("Loaded notebook '{}' with {} notes", notebook.name, note_count);

            notebooks.push(notebook);
        }

        Ok(notebooks)
    }

    /// Save a note
    pub fn save_note(&self, note: &Note, notebook_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        log::info!("开始保存笔记: {} (标题: '{}') 到笔记本: {}", note.id, note.title, notebook_id);

        let mut conn = self.get_connection()?;
        log::debug!("获取数据库连接成功");

        // 验证笔记本是否存在于数据库中
        log::debug!("验证笔记本是否存在于数据库中...");
        let notebook_exists = conn.query_row(
            "SELECT COUNT(*) FROM notebooks WHERE id = ?",
            params![notebook_id],
            |row| row.get::<_, i64>(0),
        ).unwrap_or(0) > 0;

        if !notebook_exists {
            log::error!("笔记本 '{}' 不存在于数据库中，无法保存笔记", notebook_id);
            return Err(format!("笔记本 '{}' 不存在于数据库中", notebook_id).into());
        }
        log::debug!("笔记本 '{}' 存在于数据库中，继续保存笔记", notebook_id);

        // Begin transaction
        let tx = conn.transaction()?;
        log::debug!("开始数据库事务");

        // Save note
        log::debug!("保存笔记主体数据...");
        log::debug!("笔记内容长度: {} 字符", note.content.len());
        log::debug!("笔记标题: '{}'", note.title);

        // 使用更安全的INSERT方式，避免"Execute returned results"错误
        log::debug!("尝试插入或更新笔记...");

        // 首先尝试更新现有笔记
        let update_result = tx.execute(
            "UPDATE notes SET notebook_id = ?, title = ?, content = ?, updated_at = ? WHERE id = ?",
            params![
                notebook_id,
                note.title,
                note.content,
                note.updated_at.to_rfc3339(),
                note.id
            ],
        );

        let rows_affected = match update_result {
            Ok(0) => {
                // 没有更新任何行，说明笔记不存在，需要插入
                log::debug!("笔记不存在，执行插入操作...");
                tx.execute(
                    "INSERT INTO notes (id, notebook_id, title, content, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?)",
                    params![
                        note.id,
                        notebook_id,
                        note.title,
                        note.content,
                        note.created_at.to_rfc3339(),
                        note.updated_at.to_rfc3339()
                    ],
                ).map_err(|e| {
                    let error_msg = e.to_string();
                    log::error!("执行INSERT笔记语句失败: {}", error_msg);
                    log::error!("笔记ID: {}", note.id);
                    log::error!("笔记本ID: {}", notebook_id);
                    log::error!("笔记标题: '{}'", note.title);
                    log::error!("笔记内容前100字符: '{}'", &note.content.chars().take(100).collect::<String>());

                    if error_msg.contains("FOREIGN KEY constraint failed") {
                        log::error!("外键约束失败 - 笔记本ID '{}' 可能不存在于数据库中", notebook_id);
                    }

                    e
                })?
            }
            Ok(rows) => {
                // 更新成功
                log::debug!("笔记更新成功，影响行数: {}", rows);
                rows
            }
            Err(e) => {
                let error_msg = e.to_string();
                log::error!("执行UPDATE笔记语句失败: {}", error_msg);
                log::error!("笔记ID: {}", note.id);
                log::error!("笔记本ID: {}", notebook_id);

                if error_msg.contains("FOREIGN KEY constraint failed") {
                    log::error!("外键约束失败 - 笔记本ID '{}' 可能不存在于数据库中", notebook_id);
                }

                return Err(Box::new(e));
            }
        };
        log::debug!("笔记主体数据保存完成，影响行数: {}", rows_affected);

        // Delete existing tag associations
        log::debug!("删除现有标签关联...");
        tx.execute(
            "DELETE FROM note_tags WHERE note_id = ?",
            params![note.id],
        ).map_err(|e| {
            log::error!("执行DELETE标签关联语句失败: {}", e);
            e
        })?;
        log::debug!("标签关联删除完成");

        // Add tag associations
        log::debug!("添加 {} 个标签关联", note.tag_ids.len());
        for tag_id in &note.tag_ids {
            tx.execute(
                "INSERT INTO note_tags (note_id, tag_id) VALUES (?, ?)",
                params![note.id, tag_id],
            ).map_err(|e| {
                log::error!("执行INSERT标签关联语句失败: {}", e);
                log::error!("笔记ID: {}, 标签ID: {}", note.id, tag_id);
                e
            })?;
        }
        log::debug!("标签关联添加完成");

        // Delete existing attachments
        log::debug!("删除现有附件...");
        tx.execute(
            "DELETE FROM attachments WHERE note_id = ?",
            params![note.id],
        ).map_err(|e| {
            log::error!("执行DELETE附件语句失败: {}", e);
            e
        })?;
        log::debug!("附件删除完成");

        // Add attachments
        log::debug!("添加 {} 个附件", note.attachments.len());
        for attachment in &note.attachments {
            tx.execute(
                "INSERT INTO attachments (id, note_id, name, file_path, file_type, created_at) VALUES (?, ?, ?, ?, ?, ?)",
                params![
                    attachment.id,
                    note.id,
                    attachment.name,
                    attachment.file_path,
                    attachment.file_type,
                    attachment.created_at.to_rfc3339()
                ],
            ).map_err(|e| {
                log::error!("执行INSERT附件语句失败: {}", e);
                log::error!("附件ID: {}, 笔记ID: {}", attachment.id, note.id);
                e
            })?;
        }
        log::debug!("附件添加完成");

        // Commit transaction
        log::debug!("提交数据库事务...");
        tx.commit()?;

        // 强制同步到磁盘，确保数据真正写入
        log::debug!("强制同步数据到磁盘...");
        // PRAGMA 语句可能返回结果，使用 query 而不是 execute
        let _checkpoint_result = conn.query_row("PRAGMA wal_checkpoint(FULL)", [], |row| {
            let busy: i32 = row.get(0).unwrap_or(0);
            let log_pages: i32 = row.get(1).unwrap_or(0);
            let checkpointed: i32 = row.get(2).unwrap_or(0);
            log::debug!("WAL checkpoint结果: busy={}, log_pages={}, checkpointed={}", busy, log_pages, checkpointed);
            Ok(())
        }).unwrap_or_else(|e| {
            log::warn!("WAL checkpoint失败，但不影响数据保存: {}", e);
        });

        log::info!("✅ 笔记 '{}' 成功保存到数据库并同步到磁盘", note.id);

        Ok(())
    }

    /// Load a note
    pub fn load_note(&self, id: &str) -> Result<Note, Box<dyn std::error::Error>> {
        log::debug!("开始加载笔记: {}", id);
        let conn = self.get_connection()?;
        log::debug!("获取数据库连接成功");

        log::debug!("执行查询笔记SQL: SELECT id, title, content, created_at, updated_at FROM notes WHERE id = '{}'", id);
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
        ).map_err(|e| {
            log::error!("查询笔记 '{}' 失败: {}", id, e);
            e
        })?;

        log::debug!("✅ 成功加载笔记: {} (标题: '{}')", note.id, note.title);
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
        // Return empty list if this is a placeholder
        if self.is_placeholder() {
            return Ok(Vec::new());
        }

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

    /// Get all notes
    pub fn get_all_notes(&self) -> Result<Vec<Note>, Box<dyn std::error::Error>> {
        let conn = self.get_connection()?;
        let mut notes = Vec::new();

        let mut stmt = conn.prepare(
            "SELECT id, title, content, created_at, updated_at FROM notes ORDER BY updated_at DESC"
        )?;

        let note_iter = stmt.query_map([], |row| {
            let created_at_str: String = row.get(3)?;
            let updated_at_str: String = row.get(4)?;

            let created_at = DateTime::parse_from_rfc3339(&created_at_str)
                .map_err(|_e| rusqlite::Error::InvalidColumnType(3, "created_at".to_string(), rusqlite::types::Type::Text))?
                .with_timezone(&Utc);

            let updated_at = DateTime::parse_from_rfc3339(&updated_at_str)
                .map_err(|_e| rusqlite::Error::InvalidColumnType(4, "updated_at".to_string(), rusqlite::types::Type::Text))?
                .with_timezone(&Utc);

            Ok(Note {
                id: row.get(0)?,
                title: row.get(1)?,
                content: row.get(2)?,
                created_at,
                updated_at,
                tag_ids: Vec::new(), // Will be loaded separately if needed
                attachments: Vec::new(), // Will be loaded separately if needed
            })
        })?;

        for note in note_iter {
            notes.push(note?);
        }

        Ok(notes)
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
             ORDER BY created_at DESC"
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
                updated_at TEXT NOT NULL,
                sort_order INTEGER DEFAULT 0
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

        // Create mcp_servers table for storing green-status MCP servers
        conn.execute(
            "CREATE TABLE IF NOT EXISTS mcp_servers (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                description TEXT,
                transport_type TEXT NOT NULL,
                transport_config TEXT NOT NULL,
                directory TEXT NOT NULL,
                capabilities TEXT,
                health_status TEXT NOT NULL DEFAULT 'Red',
                last_test_time TEXT,
                last_test_success INTEGER DEFAULT 0,
                enabled INTEGER DEFAULT 1,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
            [],
        )?;

        // Create settings table for storing application settings
        conn.execute(
            "CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL,
                updated_at INTEGER NOT NULL
            )",
            [],
        )?;

        // Create recent_notes table for storing recently accessed notes
        conn.execute(
            "CREATE TABLE IF NOT EXISTS recent_notes (
                note_id TEXT NOT NULL,
                note_title TEXT NOT NULL,
                accessed_at TEXT NOT NULL,
                PRIMARY KEY (note_id),
                FOREIGN KEY (note_id) REFERENCES notes(id) ON DELETE CASCADE
            )",
            [],
        )?;

        // Enable foreign keys
        conn.execute("PRAGMA foreign_keys = ON", [])?;

        // Create indexes for better performance
        self.create_indexes(conn)?;

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

        // Migrate from version 1 to 2: Add sort_order to notebooks
        if current_version < 2 {
            log::info!("Adding sort_order column to notebooks table");

            // Add sort_order column to notebooks table
            conn.execute(
                "ALTER TABLE notebooks ADD COLUMN sort_order INTEGER DEFAULT 0",
                [],
            )?;

            // Set sort_order based on created_at for existing notebooks
            conn.execute(
                "UPDATE notebooks SET sort_order = (
                    SELECT COUNT(*) FROM notebooks n2
                    WHERE n2.created_at <= notebooks.created_at
                )",
                [],
            )?;

            log::info!("Successfully added sort_order column and initialized values");
        }

        Ok(())
    }

    /// Create database indexes for better performance
    fn create_indexes(&self, conn: &Connection) -> SqlResult<()> {
        // Index on notes.notebook_id for faster notebook queries
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_notes_notebook_id ON notes(notebook_id)",
            [],
        )?;

        // Index on notes.updated_at for faster sorting
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_notes_updated_at ON notes(updated_at)",
            [],
        )?;

        // Index on note_tags.note_id for faster tag queries
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_note_tags_note_id ON note_tags(note_id)",
            [],
        )?;

        // Index on note_tags.tag_id for faster tag-based searches
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_note_tags_tag_id ON note_tags(tag_id)",
            [],
        )?;

        // Index on attachments.note_id for faster attachment queries
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_attachments_note_id ON attachments(note_id)",
            [],
        )?;

        // Index on mcp_servers.health_status for faster green server queries
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_mcp_servers_health_status ON mcp_servers(health_status)",
            [],
        )?;

        // Index on mcp_servers.enabled for faster enabled server queries
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_mcp_servers_enabled ON mcp_servers(enabled)",
            [],
        )?;

        // Full-text search index on notes content and title
        conn.execute(
            "CREATE VIRTUAL TABLE IF NOT EXISTS notes_fts USING fts5(
                id UNINDEXED,
                title,
                content,
                content='notes',
                content_rowid='rowid'
            )",
            [],
        )?;

        // Trigger to keep FTS table in sync
        conn.execute(
            "CREATE TRIGGER IF NOT EXISTS notes_fts_insert AFTER INSERT ON notes BEGIN
                INSERT INTO notes_fts(rowid, id, title, content) VALUES (new.rowid, new.id, new.title, new.content);
            END",
            [],
        )?;

        conn.execute(
            "CREATE TRIGGER IF NOT EXISTS notes_fts_delete AFTER DELETE ON notes BEGIN
                DELETE FROM notes_fts WHERE rowid = old.rowid;
            END",
            [],
        )?;

        conn.execute(
            "CREATE TRIGGER IF NOT EXISTS notes_fts_update AFTER UPDATE ON notes BEGIN
                DELETE FROM notes_fts WHERE rowid = old.rowid;
                INSERT INTO notes_fts(rowid, id, title, content) VALUES (new.rowid, new.id, new.title, new.content);
            END",
            [],
        )?;

        log::info!("Database indexes created successfully");
        Ok(())
    }

    /// Save an MCP server record
    pub fn save_mcp_server(&self, server: &McpServerRecord) -> Result<(), Box<dyn std::error::Error>> {
        let conn = self.get_connection()?;

        conn.execute(
            "INSERT OR REPLACE INTO mcp_servers (id, name, description, transport_type, transport_config, directory, capabilities, health_status, last_test_time, last_test_success, enabled, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            params![
                server.id,
                server.name,
                server.description,
                server.transport_type,
                server.transport_config,
                server.directory,
                server.capabilities,
                server.health_status,
                server.last_test_time.map(|t| t.to_rfc3339()),
                if server.last_test_success { 1 } else { 0 },
                if server.enabled { 1 } else { 0 },
                server.created_at.to_rfc3339(),
                server.updated_at.to_rfc3339()
            ],
        )?;

        Ok(())
    }

    /// Load an MCP server record
    pub fn load_mcp_server(&self, id: &str) -> Result<McpServerRecord, Box<dyn std::error::Error>> {
        let conn = self.get_connection()?;

        let server = conn.query_row(
            "SELECT id, name, description, transport_type, transport_config, directory, capabilities, health_status, last_test_time, last_test_success, enabled, created_at, updated_at
             FROM mcp_servers WHERE id = ?",
            params![id],
            |row| {
                let last_test_time_str: Option<String> = row.get(8)?;
                let last_test_time = last_test_time_str
                    .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                    .map(|dt| dt.with_timezone(&Utc));

                let created_at_str: String = row.get(11)?;
                let created_at = DateTime::parse_from_rfc3339(&created_at_str)
                    .map_err(|_e| SqlError::InvalidColumnType(11, "created_at".to_string(), rusqlite::types::Type::Text))?
                    .with_timezone(&Utc);

                let updated_at_str: String = row.get(12)?;
                let updated_at = DateTime::parse_from_rfc3339(&updated_at_str)
                    .map_err(|_e| SqlError::InvalidColumnType(12, "updated_at".to_string(), rusqlite::types::Type::Text))?
                    .with_timezone(&Utc);

                Ok(McpServerRecord {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    description: row.get(2)?,
                    transport_type: row.get(3)?,
                    transport_config: row.get(4)?,
                    directory: row.get(5)?,
                    capabilities: row.get(6)?,
                    health_status: row.get(7)?,
                    last_test_time,
                    last_test_success: row.get::<_, i32>(9)? != 0,
                    enabled: row.get::<_, i32>(10)? != 0,
                    created_at,
                    updated_at,
                })
            },
        )?;

        Ok(server)
    }

    /// List all MCP servers
    pub fn list_mcp_servers(&self) -> Result<Vec<McpServerRecord>, Box<dyn std::error::Error>> {
        let conn = self.get_connection()?;

        let mut stmt = conn.prepare(
            "SELECT id, name, description, transport_type, transport_config, directory, capabilities, health_status, last_test_time, last_test_success, enabled, created_at, updated_at
             FROM mcp_servers ORDER BY name"
        )?;

        let server_iter = stmt.query_map([], |row| {
            let last_test_time_str: Option<String> = row.get(8)?;
            let last_test_time = last_test_time_str
                .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                .map(|dt| dt.with_timezone(&Utc));

            let created_at_str: String = row.get(11)?;
            let created_at = DateTime::parse_from_rfc3339(&created_at_str)
                .map_err(|_e| SqlError::InvalidColumnType(11, "created_at".to_string(), rusqlite::types::Type::Text))?
                .with_timezone(&Utc);

            let updated_at_str: String = row.get(12)?;
            let updated_at = DateTime::parse_from_rfc3339(&updated_at_str)
                .map_err(|_e| SqlError::InvalidColumnType(12, "updated_at".to_string(), rusqlite::types::Type::Text))?
                .with_timezone(&Utc);

            Ok(McpServerRecord {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                transport_type: row.get(3)?,
                transport_config: row.get(4)?,
                directory: row.get(5)?,
                capabilities: row.get(6)?,
                health_status: row.get(7)?,
                last_test_time,
                last_test_success: row.get::<_, i32>(9)? != 0,
                enabled: row.get::<_, i32>(10)? != 0,
                created_at,
                updated_at,
            })
        })?;

        let mut servers = Vec::new();
        for server in server_iter {
            servers.push(server?);
        }

        Ok(servers)
    }

    /// List green (ready) MCP servers
    pub fn list_green_mcp_servers(&self) -> Result<Vec<McpServerRecord>, Box<dyn std::error::Error>> {
        let conn = self.get_connection()?;

        let mut stmt = conn.prepare(
            "SELECT id, name, description, transport_type, transport_config, directory, capabilities, health_status, last_test_time, last_test_success, enabled, created_at, updated_at
             FROM mcp_servers WHERE health_status = 'Green' AND enabled = 1 ORDER BY name"
        )?;

        let server_iter = stmt.query_map([], |row| {
            let last_test_time_str: Option<String> = row.get(8)?;
            let last_test_time = last_test_time_str
                .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                .map(|dt| dt.with_timezone(&Utc));

            let created_at_str: String = row.get(11)?;
            let created_at = DateTime::parse_from_rfc3339(&created_at_str)
                .map_err(|_e| SqlError::InvalidColumnType(11, "created_at".to_string(), rusqlite::types::Type::Text))?
                .with_timezone(&Utc);

            let updated_at_str: String = row.get(12)?;
            let updated_at = DateTime::parse_from_rfc3339(&updated_at_str)
                .map_err(|_e| SqlError::InvalidColumnType(12, "updated_at".to_string(), rusqlite::types::Type::Text))?
                .with_timezone(&Utc);

            Ok(McpServerRecord {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                transport_type: row.get(3)?,
                transport_config: row.get(4)?,
                directory: row.get(5)?,
                capabilities: row.get(6)?,
                health_status: row.get(7)?,
                last_test_time,
                last_test_success: row.get::<_, i32>(9)? != 0,
                enabled: row.get::<_, i32>(10)? != 0,
                created_at,
                updated_at,
            })
        })?;

        let mut servers = Vec::new();
        for server in server_iter {
            servers.push(server?);
        }

        Ok(servers)
    }

    /// Delete an MCP server record
    pub fn delete_mcp_server(&self, id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let conn = self.get_connection()?;

        conn.execute(
            "DELETE FROM mcp_servers WHERE id = ?",
            params![id],
        )?;

        Ok(())
    }

    /// Save a setting
    pub fn save_setting(&self, key: &str, value: &str) -> Result<(), Box<dyn std::error::Error>> {
        let conn = self.get_connection()?;
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs() as i64;

        conn.execute(
            "INSERT OR REPLACE INTO settings (key, value, updated_at) VALUES (?, ?, ?)",
            params![key, value, now],
        )?;

        Ok(())
    }

    /// Load a setting
    pub fn load_setting(&self, key: &str) -> Result<Option<String>, Box<dyn std::error::Error>> {
        let conn = self.get_connection()?;

        match conn.query_row(
            "SELECT value FROM settings WHERE key = ?",
            params![key],
            |row| row.get::<_, String>(0),
        ) {
            Ok(value) => Ok(Some(value)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(Box::new(e)),
        }
    }

    /// Save a recent note access record
    pub fn save_recent_note(&self, note_id: &str, note_title: &str, accessed_at: &DateTime<Utc>) -> Result<(), Box<dyn std::error::Error>> {
        let conn = self.get_connection()?;

        // Insert or replace the recent note record
        conn.execute(
            "INSERT OR REPLACE INTO recent_notes (note_id, note_title, accessed_at) VALUES (?, ?, ?)",
            params![note_id, note_title, accessed_at.to_rfc3339()],
        )?;

        // Keep only the most recent 20 records
        conn.execute(
            "DELETE FROM recent_notes WHERE note_id NOT IN (
                SELECT note_id FROM recent_notes ORDER BY accessed_at DESC LIMIT 20
            )",
            [],
        )?;

        Ok(())
    }

    /// Load recent notes from database
    pub fn load_recent_notes(&self, limit: usize) -> Result<Vec<RecentNoteAccess>, Box<dyn std::error::Error>> {
        let conn = self.get_connection()?;
        let mut recent_notes = Vec::new();

        let mut stmt = conn.prepare(
            "SELECT note_id, note_title, accessed_at FROM recent_notes ORDER BY accessed_at DESC LIMIT ?"
        )?;

        let recent_iter = stmt.query_map(params![limit], |row| {
            let note_id: String = row.get(0)?;
            let note_title: String = row.get(1)?;
            let accessed_at_str: String = row.get(2)?;

            let accessed_at = DateTime::parse_from_rfc3339(&accessed_at_str)
                .map_err(|e| SqlError::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e)))?
                .with_timezone(&Utc);

            Ok(RecentNoteAccess {
                note_id,
                note_title,
                accessed_at,
            })
        })?;

        for recent_result in recent_iter {
            recent_notes.push(recent_result?);
        }

        Ok(recent_notes)
    }
}
