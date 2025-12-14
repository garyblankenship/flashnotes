use crate::db::queries::{self, AppSettings};
use crate::state::AppState;
use tauri::{Manager, State, WebviewWindow};

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

/// Toggle always on top window state
#[tauri::command]
pub fn toggle_always_on_top(window: WebviewWindow, state: State<'_, AppState>) -> Result<bool, String> {
    let is_on_top = window.is_always_on_top().map_err(|e| e.to_string())?;
    let new_state = !is_on_top;
    window.set_always_on_top(new_state).map_err(|e| e.to_string())?;

    // Persist the setting
    let conn = state.db.lock();
    let _ = queries::set_setting(&conn, "always_on_top", if new_state { "true" } else { "false" });

    // Update menu checkmark
    if let Some(menu) = window.app_handle().menu() {
        if let Some(item) = menu.get("stay_on_top") {
            if let Some(check_item) = item.as_check_menuitem() {
                let _ = check_item.set_checked(new_state);
            }
        }
    }

    Ok(new_state)
}
