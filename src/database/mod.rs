use anyhow::{Context, Result};
use dirs::data_dir;
use rusqlite::{params, Connection};
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;
use tracing;

pub struct Database {
    conn: Rc<RefCell<Connection>>,
}

impl Database {
    pub fn new() -> Result<Self> {
        let db_path = Self::db_path();
        Self::new_with_path(db_path)
    }

    pub fn new_with_path(path: std::path::PathBuf) -> Result<Self> {
        tracing::info!(path = ?path, "Opening database");
        let conn = Connection::open(&path)
            .with_context(|| format!("Failed to open database at {:?}", path))?;
        conn.execute_batch(
            "
            PRAGMA journal_mode = WAL;
            PRAGMA foreign_keys = ON;
        ",
        )
        .context("Failed to configure database pragmas")?;
        let db = Database {
            conn: Rc::new(RefCell::new(conn)),
        };
        db.create_tables()
            .context("Failed to create database tables")?;
        tracing::info!(path = ?path, "Database opened successfully");
        Ok(db)
    }

    fn db_path() -> PathBuf {
        let mut path = data_dir().unwrap_or_else(|| PathBuf::from("."));
        path.push("zongflow");
        std::fs::create_dir_all(&path).ok();
        path.push("zongflow.db");
        path
    }

    fn create_tables(&self) -> Result<()> {
        self.conn
            .borrow()
            .execute_batch(
                "
            CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS documents (
                id INTEGER PRIMARY KEY,
                title TEXT NOT NULL,
                author TEXT,
                format TEXT NOT NULL,
                path TEXT UNIQUE NOT NULL,
                date_added TEXT NOT NULL,
                last_opened TEXT,
                cover_path TEXT
            );
        ",
            )
            .context("Failed to create database tables")?;
        Ok(())
    }

    // Settings methods
    pub fn set_setting(&self, key: &str, value: &str) -> Result<()> {
        self.conn.borrow().execute(
            "INSERT OR REPLACE INTO settings (key, value) VALUES (?1, ?2)",
            params![key, value],
        )?;
        Ok(())
    }

    pub fn get_setting(&self, key: &str) -> Result<Option<String>> {
        let conn = self.conn.borrow();
        let mut stmt = conn.prepare("SELECT value FROM settings WHERE key = ?1")?;
        let mut rows = stmt.query(params![key])?;
        if let Some(row) = rows.next()? {
            Ok(Some(row.get(0)?))
        } else {
            Ok(None)
        }
    }

    pub fn delete_setting(&self, key: &str) -> Result<()> {
        self.conn
            .borrow()
            .execute("DELETE FROM settings WHERE key = ?1", params![key])?;
        Ok(())
    }

    pub fn clear_settings(&self) -> Result<()> {
        self.conn.borrow().execute_batch("DELETE FROM settings;")?;
        Ok(())
    }

    // Document methods
    pub fn insert_document(
        &self,
        title: &str,
        author: Option<&str>,
        format: &str,
        path: &str,
        cover_path: Option<&str>,
    ) -> Result<i64> {
        let date_added = chrono::Utc::now().to_rfc3339();
        tracing::debug!(title = %title, format = %format, path = %path, "Inserting document");
        self.conn.borrow().execute(
            "INSERT INTO documents (title, author, format, path, date_added, cover_path) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![title, author, format, path, date_added, cover_path],
        ).with_context(|| format!("Failed to insert document '{}' at '{}'", title, path))?;
        let id = self.conn.borrow().last_insert_rowid();
        tracing::info!(id = id, title = %title, "Document inserted successfully");
        Ok(id)
    }

    pub fn update_document_last_opened(&self, path: &str) -> Result<()> {
        let last_opened = chrono::Utc::now().to_rfc3339();
        self.conn.borrow().execute(
            "UPDATE documents SET last_opened = ?1 WHERE path = ?2",
            params![last_opened, path],
        )?;
        Ok(())
    }

    fn document_from_row(row: &rusqlite::Row) -> rusqlite::Result<Document> {
        Ok(Document {
            id: row.get(0)?,
            title: row.get(1)?,
            author: row.get(2)?,
            format: row.get(3)?,
            path: row.get(4)?,
            date_added: row.get(5)?,
            last_opened: row.get(6)?,
            cover_path: row.get(7)?,
        })
    }

    pub fn get_document_by_path(&self, path: &str) -> Result<Option<Document>> {
        let conn = self.conn.borrow();
        let mut stmt = conn.prepare(
            "SELECT id, title, author, format, path, date_added, last_opened, cover_path FROM documents WHERE path = ?1"
        ).with_context(|| format!("Failed to prepare query for document path: {}", path))?;
        let mut rows = stmt
            .query(params![path])
            .with_context(|| format!("Failed to query document by path: {}", path))?;
        if let Some(row) = rows.next()? {
            Ok(Some(Self::document_from_row(row)?))
        } else {
            Ok(None)
        }
    }

    pub fn list_documents(&self) -> Result<Vec<Document>> {
        let conn = self.conn.borrow();
        let mut stmt = conn.prepare(
            "SELECT id, title, author, format, path, date_added, last_opened, cover_path FROM documents ORDER BY last_opened DESC, date_added DESC"
        )?;
        let docs = stmt.query_map([], Self::document_from_row)?;
        let mut result = Vec::new();
        for doc in docs {
            result.push(doc?);
        }
        Ok(result)
    }

    pub fn delete_document(&self, path: &str) -> Result<()> {
        tracing::debug!(path = %path, "Deleting document");
        let rows = self
            .conn
            .borrow()
            .execute("DELETE FROM documents WHERE path = ?1", params![path])
            .with_context(|| format!("Failed to delete document at path: {}", path))?;
        if rows == 0 {
            tracing::warn!(path = %path, "No document found to delete");
        } else {
            tracing::info!(path = %path, "Document deleted successfully");
        }
        Ok(())
    }

    pub fn clear_documents(&self) -> Result<()> {
        tracing::warn!("Clearing all documents from database");
        self.conn.borrow().execute_batch("DELETE FROM documents;")?;
        tracing::info!("All documents cleared from database");
        Ok(())
    }

    /// Test helper: Set last_opened for a document by id
    pub fn set_last_opened_for_test(&self, id: i64, last_opened: &str) -> Result<()> {
        self.conn.borrow().execute(
            "UPDATE documents SET last_opened = ? WHERE id = ?",
            params![last_opened, id],
        )?;
        Ok(())
    }
}

impl Clone for Database {
    fn clone(&self) -> Self {
        Database {
            conn: Rc::clone(&self.conn),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Document {
    pub id: i64,
    pub title: String,
    pub author: Option<String>,
    pub format: String,
    pub path: String,
    pub date_added: String,
    pub last_opened: Option<String>,
    pub cover_path: Option<String>,
}
