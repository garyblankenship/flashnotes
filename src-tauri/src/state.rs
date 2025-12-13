use parking_lot::Mutex;
use rusqlite::Connection;

/// Application state holding the database connection
pub struct AppState {
    pub db: Mutex<Connection>,
}

impl AppState {
    pub fn new(conn: Connection) -> Self {
        Self {
            db: Mutex::new(conn),
        }
    }
}
