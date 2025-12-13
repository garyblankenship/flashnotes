use crate::db::queries::{self, BufferSummary, SearchResult};
use crate::state::AppState;
use chrono::Utc;
use tauri::State;
use uuid::Uuid;

/// Convert rusqlite errors to user-friendly strings
fn map_db_error<T>(result: Result<T, rusqlite::Error>, context: &str) -> Result<T, String> {
    result.map_err(|e| format!("{}: {}", context, e))
}

/// Get current Unix timestamp
fn now() -> i64 {
    Utc::now().timestamp()
}

/// Create a new buffer and return its ID
#[tauri::command]
pub fn create_buffer(state: State<'_, AppState>) -> Result<String, String> {
    let id = Uuid::new_v4().to_string();
    let conn = state.db.lock();
    map_db_error(
        queries::create_buffer(&conn, &id, "", now()),
        "Failed to create buffer",
    )?;
    Ok(id)
}

/// Save buffer content
#[tauri::command]
pub fn save_buffer(state: State<'_, AppState>, id: String, content: String) -> Result<(), String> {
    let conn = state.db.lock();
    map_db_error(
        queries::update_buffer_content(&conn, &id, &content, now()),
        "Failed to save buffer",
    )?;
    Ok(())
}

/// Get buffer content by ID
#[tauri::command]
pub fn get_buffer_content(state: State<'_, AppState>, id: String) -> Result<String, String> {
    let conn = state.db.lock();

    // Touch the buffer to update accessed_at for "recently used" sorting
    map_db_error(
        queries::touch_buffer(&conn, &id, now()),
        "Failed to touch buffer",
    )?;

    let buffer = map_db_error(
        queries::get_buffer_content(&conn, &id),
        "Failed to get buffer",
    )?;

    buffer
        .map(|b| b.content)
        .ok_or_else(|| format!("Buffer not found: {}", id))
}

/// Get sidebar data (list of buffer summaries)
#[tauri::command]
pub fn get_sidebar_data(state: State<'_, AppState>) -> Result<Vec<BufferSummary>, String> {
    let conn = state.db.lock();
    map_db_error(
        queries::get_sidebar_buffers(&conn, 100),
        "Failed to get sidebar data",
    )
}

/// Search buffers using FTS5
#[tauri::command]
pub fn search_buffers(state: State<'_, AppState>, query: String) -> Result<Vec<SearchResult>, String> {
    let conn = state.db.lock();
    map_db_error(
        queries::search_buffers(&conn, &query, 20),
        "Failed to search buffers",
    )
}

/// Archive a buffer (soft delete)
#[tauri::command]
pub fn archive_buffer(state: State<'_, AppState>, id: String) -> Result<(), String> {
    let conn = state.db.lock();
    map_db_error(queries::archive_buffer(&conn, &id), "Failed to archive buffer")?;
    Ok(())
}

/// Permanently delete a buffer
#[tauri::command]
pub fn delete_buffer_permanently(state: State<'_, AppState>, id: String) -> Result<(), String> {
    let conn = state.db.lock();
    map_db_error(queries::delete_buffer(&conn, &id), "Failed to delete buffer")?;
    Ok(())
}

/// Toggle pin status
#[tauri::command]
pub fn toggle_pin(state: State<'_, AppState>, id: String) -> Result<(), String> {
    let conn = state.db.lock();
    map_db_error(queries::toggle_pin(&conn, &id), "Failed to toggle pin")?;
    Ok(())
}

/// Get total buffer count
#[tauri::command]
pub fn get_buffer_count(state: State<'_, AppState>) -> Result<i64, String> {
    let conn = state.db.lock();
    map_db_error(
        queries::get_buffer_count(&conn, false),
        "Failed to get buffer count",
    )
}
