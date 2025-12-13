use rusqlite::{Connection, Result};
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

/// Get the database path in the app's data directory
pub fn get_db_path(app: &AppHandle) -> PathBuf {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .expect("Failed to get app data directory");

    // Ensure directory exists
    std::fs::create_dir_all(&app_data_dir).expect("Failed to create app data directory");

    app_data_dir.join("flashnotes.db")
}

/// Create a new database connection with optimized settings
pub fn create_connection(path: &PathBuf) -> Result<Connection> {
    let conn = Connection::open(path)?;

    // Critical PRAGMA settings for performance
    conn.execute_batch(
        "
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
        PRAGMA foreign_keys = ON;
        PRAGMA temp_store = MEMORY;
        ",
    )?;

    Ok(conn)
}
