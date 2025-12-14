use rusqlite::{Connection, Result, params};
use serde::{Deserialize, Serialize};

/// Summary of a buffer for sidebar display
#[derive(Debug, Serialize, Deserialize)]
pub struct BufferSummary {
    pub id: String,
    pub title: String,
    pub preview: String,
    pub updated_at: i64,
    pub is_pinned: bool,
}

/// Search result with highlighted snippet
#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResult {
    pub id: String,
    pub snippet: String,
    pub updated_at: i64,
}

/// Full buffer content
#[derive(Debug, Serialize, Deserialize)]
pub struct Buffer {
    pub id: String,
    pub content: String,
    pub created_at: i64,
    pub updated_at: i64,
    pub accessed_at: i64,
    pub is_archived: bool,
    pub is_pinned: bool,
}

/// Extract title and preview from content
pub fn extract_title_preview(content: &str) -> (String, String) {
    let lines: Vec<&str> = content.lines().collect();

    let title = lines
        .iter()
        .find(|line| !line.trim().is_empty())
        .map(|s| s.trim().chars().take(100).collect())
        .unwrap_or_else(|| "Untitled".to_string());

    let preview = lines
        .iter()
        .skip_while(|line| line.trim().is_empty())
        .skip(1) // Skip title line
        .find(|line| !line.trim().is_empty())
        .map(|s| s.trim().chars().take(100).collect())
        .unwrap_or_default();

    (title, preview)
}

/// Get sidebar buffers (non-archived, sorted by pinned then sort_order then accessed_at)
pub fn get_sidebar_buffers(conn: &Connection, limit: usize) -> Result<Vec<BufferSummary>> {
    let mut stmt = conn.prepare(
        "
        SELECT id, content, updated_at, is_pinned
        FROM buffers
        WHERE is_archived = 0
        ORDER BY is_pinned DESC, sort_order ASC, accessed_at DESC
        LIMIT ?
        "
    )?;

    let rows = stmt.query_map([limit as i64], |row| {
        let id: String = row.get(0)?;
        let content: String = row.get(1)?;
        let updated_at: i64 = row.get(2)?;
        let is_pinned: i64 = row.get(3)?;

        let (title, preview) = extract_title_preview(&content);

        Ok(BufferSummary {
            id,
            title,
            preview,
            updated_at,
            is_pinned: is_pinned != 0,
        })
    })?;

    rows.collect()
}

/// Search buffers using FTS5
pub fn search_buffers(conn: &Connection, query: &str, limit: usize) -> Result<Vec<SearchResult>> {
    if query.trim().is_empty() {
        return Ok(Vec::new());
    }

    // Escape special FTS5 characters and add prefix matching
    let safe_query = query
        .replace('"', "\"\"")
        .split_whitespace()
        .map(|term| format!("\"{}\"*", term))
        .collect::<Vec<_>>()
        .join(" ");

    let mut stmt = conn.prepare(
        "
        SELECT b.id, highlight(buffers_fts, 0, '<mark>', '</mark>') as snippet, b.updated_at
        FROM buffers_fts
        JOIN buffers b ON buffers_fts.rowid = b.rowid
        WHERE buffers_fts MATCH ?
        AND b.is_archived = 0
        ORDER BY rank
        LIMIT ?
        "
    )?;

    let rows = stmt.query_map(params![safe_query, limit as i64], |row| {
        Ok(SearchResult {
            id: row.get(0)?,
            snippet: row.get(1)?,
            updated_at: row.get(2)?,
        })
    })?;

    rows.collect()
}

/// Get full buffer content by ID
pub fn get_buffer_content(conn: &Connection, id: &str) -> Result<Option<Buffer>> {
    let mut stmt = conn.prepare(
        "
        SELECT id, content, created_at, updated_at, accessed_at, is_archived, is_pinned
        FROM buffers
        WHERE id = ?
        "
    )?;

    let result = stmt.query_row([id], |row| {
        Ok(Buffer {
            id: row.get(0)?,
            content: row.get(1)?,
            created_at: row.get(2)?,
            updated_at: row.get(3)?,
            accessed_at: row.get(4)?,
            is_archived: row.get::<_, i64>(5)? != 0,
            is_pinned: row.get::<_, i64>(6)? != 0,
        })
    });

    match result {
        Ok(buffer) => Ok(Some(buffer)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e),
    }
}

/// Create a new buffer with sort_order = min(existing) - 1 to place at top
pub fn create_buffer(conn: &Connection, id: &str, content: &str, timestamp: i64) -> Result<()> {
    // Get the minimum sort_order to place new buffer at top
    let min_order: i64 = conn
        .query_row(
            "SELECT COALESCE(MIN(sort_order), 0) - 1 FROM buffers WHERE is_archived = 0",
            [],
            |row| row.get(0),
        )
        .unwrap_or(-1);

    conn.execute(
        "
        INSERT INTO buffers (id, content, created_at, updated_at, accessed_at, sort_order)
        VALUES (?, ?, ?, ?, ?, ?)
        ",
        params![id, content, timestamp, timestamp, timestamp, min_order],
    )?;
    Ok(())
}

/// Update buffer content
pub fn update_buffer_content(conn: &Connection, id: &str, content: &str, timestamp: i64) -> Result<bool> {
    let rows_affected = conn.execute(
        "
        UPDATE buffers
        SET content = ?, updated_at = ?
        WHERE id = ?
        ",
        params![content, timestamp, id],
    )?;
    Ok(rows_affected > 0)
}

/// Update buffer accessed timestamp (for "recently used" sorting)
pub fn touch_buffer(conn: &Connection, id: &str, timestamp: i64) -> Result<bool> {
    let rows_affected = conn.execute(
        "
        UPDATE buffers
        SET accessed_at = ?
        WHERE id = ?
        ",
        params![timestamp, id],
    )?;
    Ok(rows_affected > 0)
}

/// Get the next buffer ID (for selection after delete)
pub fn get_next_buffer_id(conn: &Connection, current_id: &str) -> Result<Option<String>> {
    // Get first non-archived buffer that isn't the one being deleted
    let mut stmt = conn.prepare(
        "SELECT id FROM buffers
         WHERE is_archived = 0 AND id != ?
         ORDER BY is_pinned DESC, sort_order ASC, accessed_at DESC
         LIMIT 1"
    )?;

    match stmt.query_row([current_id], |row| row.get(0)) {
        Ok(id) => Ok(Some(id)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e),
    }
}

/// Permanently delete a buffer
pub fn delete_buffer(conn: &Connection, id: &str) -> Result<bool> {
    let rows_affected = conn.execute(
        "DELETE FROM buffers WHERE id = ?",
        params![id],
    )?;
    Ok(rows_affected > 0)
}

/// Toggle pin status and return new state
pub fn toggle_pin(conn: &Connection, id: &str) -> Result<bool> {
    conn.execute(
        "UPDATE buffers SET is_pinned = NOT is_pinned WHERE id = ?",
        params![id],
    )?;
    // Return the new pin state
    conn.query_row(
        "SELECT is_pinned FROM buffers WHERE id = ?",
        params![id],
        |row| Ok(row.get::<_, i64>(0)? != 0),
    )
}

/// Reorder buffers by setting sort_order based on provided ID list
/// Wrapped in transaction for 10-50× performance improvement
pub fn reorder_buffers(conn: &mut Connection, ids: &[String]) -> Result<()> {
    let tx = conn.transaction()?;
    for (index, id) in ids.iter().enumerate() {
        tx.execute(
            "UPDATE buffers SET sort_order = ? WHERE id = ?",
            params![index as i64, id],
        )?;
    }
    tx.commit()
}

/// Delete all empty buffers (content is empty or whitespace only)
pub fn delete_empty_buffers(conn: &Connection) -> Result<usize> {
    let rows_affected = conn.execute(
        "DELETE FROM buffers WHERE TRIM(content) = '' AND is_archived = 0",
        [],
    )?;
    Ok(rows_affected)
}

/// App settings
#[derive(Debug, Serialize, Deserialize)]
pub struct AppSettings {
    pub font_family: String,
    pub font_size: i32,
    pub line_height: f64,
    pub sidebar_width: i32,
    pub sidebar_collapsed: bool,
    pub vim_mode: bool,
    pub always_on_top: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            font_family: "JetBrains Mono".to_string(),
            font_size: 13,
            line_height: 1.5,
            sidebar_width: 256,
            sidebar_collapsed: false,
            vim_mode: false,
            always_on_top: false,
        }
    }
}

/// Get all settings
pub fn get_settings(conn: &Connection) -> Result<AppSettings> {
    let mut stmt = conn.prepare("SELECT key, value FROM settings")?;
    let rows = stmt.query_map([], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
    })?;

    let mut settings = AppSettings::default();
    for row in rows {
        let (key, value) = row?;
        match key.as_str() {
            "font_family" => settings.font_family = value,
            "font_size" => settings.font_size = value.parse().unwrap_or(13),
            "line_height" => settings.line_height = value.parse().unwrap_or(1.5),
            "sidebar_width" => settings.sidebar_width = value.parse().unwrap_or(256),
            "sidebar_collapsed" => settings.sidebar_collapsed = value == "true",
            "vim_mode" => settings.vim_mode = value == "true",
            "always_on_top" => settings.always_on_top = value == "true",
            _ => {}
        }
    }

    Ok(settings)
}

/// Set a single setting
pub fn set_setting(conn: &Connection, key: &str, value: &str) -> Result<()> {
    conn.execute(
        "INSERT OR REPLACE INTO settings (key, value) VALUES (?, ?)",
        params![key, value],
    )?;
    Ok(())
}
