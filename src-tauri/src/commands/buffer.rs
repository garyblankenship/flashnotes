use crate::db::queries::{self, BufferSummary, SearchResult};
use crate::state::AppState;
use chrono::Utc;
use tauri::State;
use uuid::Uuid;
use tracing::{info, warn};

/// Maximum buffer size in bytes (10MB)
const MAX_BUFFER_SIZE: usize = 10 * 1024 * 1024;

/// Convert rusqlite errors to user-friendly strings
fn map_db_error<T>(result: Result<T, rusqlite::Error>, context: &str) -> Result<T, String> {
    result.map_err(|e| format!("{}: {}", context, e))
}

/// Get current Unix timestamp
fn now() -> i64 {
    Utc::now().timestamp()
}

/// Validate buffer content size
fn validate_buffer_size(content: &str) -> Result<(), String> {
    let size = content.len();
    if size > MAX_BUFFER_SIZE {
        let size_mb = size as f64 / (1024.0 * 1024.0);
        return Err(format!(
            "Buffer content too large ({:.2}MB). Maximum size is 10MB. \
            Consider splitting your note into smaller parts.",
            size_mb
        ));
    }
    Ok(())
}

/// Create a new buffer with optional content, return summary for immediate UI update
#[tauri::command]
pub fn create_buffer(state: State<'_, AppState>, content: Option<String>) -> Result<BufferSummary, String> {
    let content = content.unwrap_or_default();

    // Validate size before creating
    validate_buffer_size(&content)?;

    let id = Uuid::new_v4().to_string();
    let timestamp = now();
    let conn = state.writer.lock();

    map_db_error(
        queries::create_buffer(&conn, &id, &content, timestamp),
        "Failed to create buffer",
    )?;

    info!("Created buffer: {}", id);

    // Return summary for immediate UI update (no refetch needed)
    let (title, preview) = queries::extract_title_preview(&content);
    Ok(BufferSummary {
        id,
        title,
        preview,
        updated_at: timestamp,
        is_pinned: false,
    })
}

/// Save buffer content and return updated title/preview for sidebar
#[tauri::command]
pub fn save_buffer(state: State<'_, AppState>, id: String, content: String) -> Result<(String, String), String> {
    // Validate size before saving
    validate_buffer_size(&content)?;

    let conn = state.writer.lock();
    map_db_error(
        queries::update_buffer_content(&conn, &id, &content, now()),
        "Failed to save buffer",
    )?;
    // Return new title/preview so frontend can update sidebar without refetch
    Ok(queries::extract_title_preview(&content))
}

/// Get buffer content by ID
#[tauri::command]
pub fn get_buffer_content(state: State<'_, AppState>, id: String) -> Result<String, String> {
    // Try reader pool first, fall back to writer
    let content = match state.get_reader() {
        Ok(conn) => {
            // Touch using writer (it's a write operation)
            {
                let writer = state.writer.lock();
                map_db_error(
                    queries::touch_buffer(&writer, &id, now()),
                    "Failed to touch buffer",
                )?;
            }

            map_db_error(
                queries::get_buffer_content(&conn, &id),
                "Failed to get buffer",
            )?
        }
        Err(_) => {
            // Fall back to writer
            let conn = state.writer.lock();
            map_db_error(
                queries::touch_buffer(&conn, &id, now()),
                "Failed to touch buffer",
            )?;

            map_db_error(
                queries::get_buffer_content(&conn, &id),
                "Failed to get buffer",
            )?
        }
    };

    content
        .map(|b| b.content)
        .ok_or_else(|| format!("Buffer not found: {}", id))
}

/// Get sidebar data (list of buffer summaries)
#[tauri::command]
pub fn get_sidebar_data(state: State<'_, AppState>, offset: Option<usize>) -> Result<Vec<BufferSummary>, String> {
    let offset = offset.unwrap_or(0);
    let limit = 100; // Initial load and each subsequent batch

    // Try reader pool first
    match state.get_reader() {
        Ok(conn) => map_db_error(
            queries::get_sidebar_buffers(&conn, limit, offset),
            "Failed to get sidebar data",
        ),
        Err(_) => {
            let conn = state.writer.lock();
            map_db_error(
                queries::get_sidebar_buffers(&conn, limit, offset),
                "Failed to get sidebar data",
            )
        }
    }
}

/// Search buffers using FTS5
#[tauri::command]
pub fn search_buffers(state: State<'_, AppState>, query: String) -> Result<Vec<SearchResult>, String> {
    // Validate and sanitize query
    let sanitized = match queries::sanitize_search_query(&query) {
        Some(q) => q,
        None => {
            warn!("Invalid search query rejected: {:?}", query);
            return Ok(Vec::new()); // Return empty results for invalid queries
        }
    };

    // Try reader pool first
    match state.get_reader() {
        Ok(conn) => map_db_error(
            queries::search_buffers(&conn, &sanitized, 20),
            "Failed to search buffers",
        ),
        Err(_) => {
            let conn = state.writer.lock();
            map_db_error(
                queries::search_buffers(&conn, &sanitized, 20),
                "Failed to search buffers",
            )
        }
    }
}

/// Delete a buffer and return the next buffer ID to select (if any)
#[tauri::command]
pub fn delete_buffer(state: State<'_, AppState>, id: String) -> Result<Option<String>, String> {
    let conn = state.writer.lock();

    // Get next buffer before deleting
    let next_id = map_db_error(
        queries::get_next_buffer_id(&conn, &id),
        "Failed to get next buffer",
    )?;

    // Delete the buffer
    map_db_error(queries::delete_buffer(&conn, &id), "Failed to delete buffer")?;

    info!("Deleted buffer: {}", id);

    Ok(next_id)
}

/// Toggle pin status and return new state
#[tauri::command]
pub fn toggle_pin(state: State<'_, AppState>, id: String) -> Result<bool, String> {
    let conn = state.writer.lock();
    map_db_error(queries::toggle_pin(&conn, &id), "Failed to toggle pin")
}

/// Reorder buffers by setting sort_order
#[tauri::command]
pub fn reorder_buffers(state: State<'_, AppState>, ids: Vec<String>) -> Result<(), String> {
    let mut conn = state.writer.lock();
    map_db_error(
        queries::reorder_buffers(&mut conn, &ids),
        "Failed to reorder buffers",
    )?;
    Ok(())
}

/// Delete all empty buffers
#[tauri::command]
pub fn cleanup_empty_buffers(state: State<'_, AppState>) -> Result<usize, String> {
    let conn = state.writer.lock();
    let count = map_db_error(
        queries::delete_empty_buffers(&conn),
        "Failed to cleanup empty buffers",
    )?;
    if count > 0 {
        info!("Cleaned up {} empty buffers", count);
    }
    Ok(count)
}
