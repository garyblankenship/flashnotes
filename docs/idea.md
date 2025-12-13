Here is the **Revised Build Specification (v2)**.

This spec incorporates the "Hot Exit" philosophy but hardens the architecture for performance (FTS5, WAL mode) and usability (CodeMirror 6, Global Hotkeys).

---

# Project Codename: **Flashbuffer**
**Target:** macOS (Tauri v2)
**Core Identity:** An infinite-buffer, database-backed scratchpad. No files. No saving.

---

## 1. Technology Stack Selection

*   **Frontend Framework:** **Svelte** or **SolidJS**.
    *   *Reason:* High performance, compiled (no Virtual DOM overhead), crucial for feeling "native" and instant.
*   **Editor Component:** **CodeMirror 6**.
    *   *Reason:* Modular, accessible, handles large documents better than generic `textarea`, lighter/faster startup than Monaco.
*   **Backend:** **Rust** (Tauri).
*   **Database:** **SQLite** (bundled via `rusqlite`).
*   **IPC Strategy:** Async commands with decoupled persistence.

---

## 2. Database Architecture (Hardened)

The database must be optimized for write-heavy, read-frequent workloads.

### 2.1 Configuration
On connection, run:
```sql
PRAGMA journal_mode = WAL;  -- Write-Ahead Logging (Non-blocking writes)
PRAGMA synchronous = NORMAL; -- Faster writes, safe enough for app crashes
PRAGMA foreign_keys = ON;
```

### 2.2 Schema

#### `buffers` (The Truth)
```sql
CREATE TABLE buffers (
    id TEXT PRIMARY KEY,       -- UUID v4
    content TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    accessed_at INTEGER NOT NULL, -- For "Recently Used" sorting
    is_archived INTEGER DEFAULT 0,
    is_pinned INTEGER DEFAULT 0
);
```

#### `buffers_fts` ( The Speed)
Uses **FTS5** virtual table for sub-millisecond full-text search.
```sql
CREATE VIRTUAL TABLE buffers_fts USING fts5(
    content,
    content='buffers', -- External Content Table pattern (saves disk space)
    content_rowid='rowid'
);
```

#### `triggers` (The Automation)
Automatically keep FTS index in sync. Rust code never writes to `buffers_fts` directly.
```sql
CREATE TRIGGER buffers_ai AFTER INSERT ON buffers BEGIN
  INSERT INTO buffers_fts(rowid, content) VALUES (new.rowid, new.content);
END;
-- (Add corresponding DELETE and UPDATE triggers)
```

---

## 3. Data Flow & Persistence Strategy

The biggest risk is UI lag during typing. We decouple **Typing** from **Persisting**.

### 3.1 The "Hot Loop"
1.  **User Types:** `CodeMirror` updates local JS state immediately.
2.  **Debounce (Frontend):** Wait 500ms of idleness.
3.  **IPC Call:** Send `save_buffer_content(id, content)` to Rust.
    *   *Optimization:* If content > 50KB, only send if changed.
4.  **Rust Handler:** Spawns an async task to write to SQLite. **Does not block the UI thread.**

### 3.2 The "Safety Net"
1.  **Window Blur / App Hide:** Immediately trigger `save_buffer_content`.
2.  **App Quit:** Tauri `OnDestroy` hook forces a final flush.

---

## 4. Feature Specification

### 4.1 The Global Hotkey (The "Drafts" Juice)
*   **Behavior:** An OS-level shortcut (e.g., `Cmd+Shift+Space`) toggles the window visibility.
*   **Focus:** When toggled ON, focus *must* land immediately in the editor.
*   **Tray Icon:** App lives in the Menu Bar (Tray). Dock icon is optional (can be hidden in `tauri.conf.json`).

### 4.2 The Buffer List (Sidebar)
*   **Sorting Logic (SQL):**
    ```sql
    SELECT * FROM buffers 
    WHERE is_archived = 0 
    ORDER BY is_pinned DESC, accessed_at DESC 
    LIMIT 50;
    ```
*   **Visuals:**
    *   Show first 2 lines of text as "Title" and "Preview".
    *   Show relative time ("2m ago").
*   **Virtualization:** If user has 5,000 buffers, only render the visible 20 DOM nodes.

### 4.3 Search (The Killer Feature)
*   **UI:** Omnibar at top of sidebar.
*   **Query:**
    ```sql
    -- Rust FTS Query
    SELECT rowid, highlight(buffers_fts, 0, '<b>', '</b>') 
    FROM buffers_fts 
    WHERE buffers_fts MATCH :query 
    ORDER BY rank 
    LIMIT 20;
    ```
*   **Experience:** Results update per-keystroke. Arrow keys move selection down the list. Enter opens buffer.

---

## 5. UI Layout & UX

**Style:** macOS Native-ish (San Francisco font).

```
[ Sidebar (250px) ] [ Editor (Flex Grow)          ]
| Search...       | |                             |
| [List Item 1]   | |  Type here...               |
| [List Item 2]   | |                             |
|                 | |                             |
| (Cmd+N to add)  | |                             |
```

*   **Window:** Use Tauri `NSWindowStyleMaskFullSizeContentView` for a transparent title bar effect.
*   **Dark Mode:** Auto-detect system preference.

---

## 6. Rust Command Interface (API)

These are the functions exposed to the Frontend.

```rust
// Queries
#[tauri::command]
fn get_sidebar_data() -> Vec<BufferSummary>;

#[tauri::command]
fn search_buffers(query: String) -> Vec<SearchResult>;

#[tauri::command]
fn get_buffer_content(id: String) -> String;

// Actions
#[tauri::command]
fn create_buffer() -> String; // Returns new UUID

#[tauri::command]
fn save_buffer(id: String, content: String);

#[tauri::command]
fn archive_buffer(id: String);

#[tauri::command]
fn delete_buffer_permanently(id: String);
```

---

## 7. Migration & Import (Future Proofing)

*   **Import Folder:** A command to "Suck in a folder of .txt files".
    *   Iterate folder, create UUID for each, insert into DB, ignore original file.
*   **Export:**
    *   "Export All to JSON" (for backup).
    *   "Export Current to File" (Save As dialog).

---

## 8. Implementation Steps (Order of Operations)

1.  **Skeleton:** `npm create tauri-app`. Setup Svelte + Tailwind.
2.  **DB Core:** Setup `rusqlite` with the Schema (including FTS5) in `main.rs`.
3.  **The Editor:** Install CodeMirror 6. Get text typing.
4.  **The Wire:** Implement `save_buffer` command. Verify data survives a restart.
5.  **The List:** Implement sidebar fetching (sorted by `accessed_at`).
6.  **Search:** Implement the FTS5 query command.
7.  **Polish:** Add the Global Shortcut and "Hide Dock Icon" logic.

## 9. Failure Mode Handling

*   **Corrupt DB:** On startup, if SQLite fails to open, copy `db.sqlite` to `db.sqlite.bak` and start fresh. Notify user.
*   **Performance:** If DB size > 500MB, run `VACUUM` on shutdown.

---

This spec moves the complexity to the **Database (SQLite)** and allows the frontend to be dumb and fast. FTS5 is the secret weapon here—it makes a 10,000-note database feel like a 10-note list.