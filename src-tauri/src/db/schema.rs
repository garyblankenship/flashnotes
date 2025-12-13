use rusqlite::{Connection, Result};

/// Initialize the database schema including FTS5 tables and triggers
pub fn initialize_schema(conn: &Connection) -> Result<()> {
    // Create main buffers table
    conn.execute(
        "
        CREATE TABLE IF NOT EXISTS buffers (
            id TEXT PRIMARY KEY,
            content TEXT NOT NULL DEFAULT '',
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL,
            accessed_at INTEGER NOT NULL,
            is_archived INTEGER DEFAULT 0,
            is_pinned INTEGER DEFAULT 0,
            sort_order INTEGER DEFAULT 0
        );
        ",
        [],
    )?;

    // Migration: Add sort_order column if it doesn't exist
    conn.execute(
        "ALTER TABLE buffers ADD COLUMN sort_order INTEGER DEFAULT 0",
        [],
    ).ok(); // Ignore error if column already exists

    // Create settings table (key-value store)
    conn.execute(
        "
        CREATE TABLE IF NOT EXISTS settings (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );
        ",
        [],
    )?;

    // Insert default settings if not present
    conn.execute(
        "
        INSERT OR IGNORE INTO settings (key, value) VALUES
            ('font_family', 'JetBrains Mono'),
            ('font_size', '13'),
            ('line_height', '1.5');
        ",
        [],
    )?;

    // Create index for sidebar query performance
    conn.execute(
        "
        CREATE INDEX IF NOT EXISTS idx_buffers_sidebar
        ON buffers (is_archived, is_pinned DESC, accessed_at DESC);
        ",
        [],
    )?;

    // Create FTS5 virtual table for full-text search
    // Using external content table pattern to save disk space
    conn.execute(
        "
        CREATE VIRTUAL TABLE IF NOT EXISTS buffers_fts USING fts5(
            content,
            content='buffers',
            content_rowid='rowid'
        );
        ",
        [],
    )?;

    // Create triggers to keep FTS index in sync
    // INSERT trigger
    conn.execute(
        "
        CREATE TRIGGER IF NOT EXISTS buffers_ai AFTER INSERT ON buffers BEGIN
            INSERT INTO buffers_fts(rowid, content) VALUES (new.rowid, new.content);
        END;
        ",
        [],
    )?;

    // DELETE trigger
    conn.execute(
        "
        CREATE TRIGGER IF NOT EXISTS buffers_ad AFTER DELETE ON buffers BEGIN
            INSERT INTO buffers_fts(buffers_fts, rowid, content) VALUES('delete', old.rowid, old.content);
        END;
        ",
        [],
    )?;

    // UPDATE trigger
    conn.execute(
        "
        CREATE TRIGGER IF NOT EXISTS buffers_au AFTER UPDATE ON buffers BEGIN
            INSERT INTO buffers_fts(buffers_fts, rowid, content) VALUES('delete', old.rowid, old.content);
            INSERT INTO buffers_fts(rowid, content) VALUES (new.rowid, new.content);
        END;
        ",
        [],
    )?;

    Ok(())
}

/// Run database integrity check
#[allow(dead_code)]
pub fn check_integrity(conn: &Connection) -> Result<bool> {
    let result: String = conn.query_row("PRAGMA integrity_check;", [], |row| row.get(0))?;
    Ok(result == "ok")
}

/// Vacuum the database to reclaim space
#[allow(dead_code)]
pub fn vacuum(conn: &Connection) -> Result<()> {
    conn.execute("VACUUM;", [])?;
    Ok(())
}
