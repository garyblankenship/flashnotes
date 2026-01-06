use parking_lot::Mutex;
use r2d2::{Pool, PooledConnection};
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::Connection;
use std::path::PathBuf;
use tracing::{debug, warn};

/// Connection pool error type
pub type PoolError = r2d2::Error;

/// Application state holding database connections
/// - Single writer connection (Mutex-protected) for all writes
/// - Connection pool for concurrent reads
pub struct AppState {
    /// Dedicated writer connection - all writes go through here
    pub writer: Mutex<Connection>,
    /// Connection pool for read operations
    pub reader_pool: Pool<SqliteConnectionManager>,
    /// App data directory for backups (available for future use)
    #[allow(dead_code)]
    pub app_data_dir: PathBuf,
}

impl AppState {
    /// Create new app state with writer connection and reader pool
    pub fn new(writer: Connection, reader_pool: Pool<SqliteConnectionManager>, app_data_dir: PathBuf) -> Self {
        Self {
            writer: Mutex::new(writer),
            reader_pool,
            app_data_dir,
        }
    }

    /// Get a read-only connection from the pool
    /// Falls back to writer if pool is exhausted
    pub fn get_reader(&self) -> Result<PooledConnection<SqliteConnectionManager>, PoolError> {
        match self.reader_pool.get() {
            Ok(conn) => {
                debug!("Acquired reader connection from pool");
                Ok(conn)
            }
            Err(e) => {
                warn!("Failed to get reader from pool: {}, operations will use writer", e);
                Err(e)
            }
        }
    }
}

