use crate::db::queries::{self, AppSettings};
use crate::state::AppState;
use tauri::State;

/// Convert rusqlite errors to user-friendly strings
fn map_db_error<T>(result: Result<T, rusqlite::Error>, context: &str) -> Result<T, String> {
    result.map_err(|e| format!("{}: {}", context, e))
}

/// Get all app settings
#[tauri::command]
pub fn get_settings(state: State<'_, AppState>) -> Result<AppSettings, String> {
    let conn = state.db.lock();
    map_db_error(queries::get_settings(&conn), "Failed to get settings")
}

/// Update a single setting
#[tauri::command]
pub fn set_setting(state: State<'_, AppState>, key: String, value: String) -> Result<(), String> {
    let conn = state.db.lock();
    map_db_error(
        queries::set_setting(&conn, &key, &value),
        "Failed to save setting",
    )
}
