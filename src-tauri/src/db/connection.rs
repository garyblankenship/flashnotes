use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::{Connection, Result};
use std::path::PathBuf;
use std::time::Duration;
use tauri::{AppHandle, Manager};
use tracing::error;

/// Database initialization error
#[derive(Debug)]
pub struct DbInitError {
    pub message: String,
    pub details: String,
}

impl std::fmt::Display for DbInitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.message, self.details)
    }
}

impl std::error::Error for DbInitError {}

/// Get the database path in the app's data directory
pub fn get_db_path(app: &AppHandle) -> std::result::Result<PathBuf, DbInitError> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| DbInitError {
            message: "Failed to get app data directory".to_string(),
            details: e.to_string(),
        })?;

    // Ensure directory exists
    std::fs::create_dir_all(&app_data_dir).map_err(|e| DbInitError {
        message: "Failed to create app data directory".to_string(),
        details: e.to_string(),
    })?;

    Ok(app_data_dir.join("flashnotes.db"))
}

/// Get app data directory
pub fn get_app_data_dir(app: &AppHandle) -> std::result::Result<PathBuf, DbInitError> {
    app.path()
        .app_data_dir()
        .map_err(|e| DbInitError {
            message: "Failed to get app data directory".to_string(),
            details: e.to_string(),
        })
}

/// Create a new database connection with optimized settings
pub fn create_connection(path: &PathBuf) -> Result<Connection> {
    let conn = Connection::open(path)?;

    // Critical PRAGMA settings for performance and FTS triggers
    conn.execute_batch(
        "
        -- Allow FTS triggers to write to virtual tables
        PRAGMA trusted_schema = ON;
        PRAGMA journal_mode = WAL;
        PRAGMA synchronous = NORMAL;
        PRAGMA foreign_keys = ON;
        PRAGMA cache_size = -64000;
        PRAGMA busy_timeout = 5000;
        PRAGMA temp_store = MEMORY;
        ",
    )?;

    Ok(conn)
}

/// Create a connection for development/testing with in-memory database
#[allow(dead_code)]
pub fn create_memory_connection() -> Result<Connection> {
    let conn = Connection::open_in_memory()?;

    conn.execute_batch(
        "
        PRAGMA trusted_schema = ON;
        PRAGMA foreign_keys = ON;
        PRAGMA temp_store = MEMORY;
        ",
    )?;

    Ok(conn)
}

/// Create a connection pool for read operations
pub fn create_reader_pool(path: &PathBuf) -> std::result::Result<Pool<SqliteConnectionManager>, DbInitError> {
    let manager = SqliteConnectionManager::file(path)
        .with_init(|conn| {
            conn.execute_batch(
                "
                PRAGMA trusted_schema = ON;
                PRAGMA journal_mode = WAL;
                PRAGMA synchronous = NORMAL;
                PRAGMA foreign_keys = ON;
                PRAGMA cache_size = -32000;
                PRAGMA busy_timeout = 5000;
                PRAGMA temp_store = MEMORY;
                PRAGMA query_only = ON;
                "
            )?;
            Ok(())
        });

    Pool::builder()
        .max_size(4) // 4 reader connections
        .min_idle(Some(1))
        .connection_timeout(Duration::from_secs(5))
        .build(manager)
        .map_err(|e| {
            error!("Failed to create reader pool: {}", e);
            DbInitError {
                message: "Failed to create database connection pool".to_string(),
                details: e.to_string(),
            }
        })
}
