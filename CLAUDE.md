# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build & Development Commands

```bash
# Development (runs frontend + Tauri backend with hot reload)
npm run tauri dev

# Production build (creates .app and .dmg in src-tauri/target/release/bundle/)
npm run tauri build

# Frontend only
npm run dev          # Vite dev server on port 1420
npm run build        # Build to build/

# Type checking
npm run check        # Svelte + TypeScript validation

# Rust only
cargo build --manifest-path src-tauri/Cargo.toml
```

## Architecture

**Tauri v2 desktop app:** Rust backend + SvelteKit 5 frontend, macOS only.

### Backend (src-tauri/src/)

```
lib.rs          → App initialization, plugin registration, command handlers
state.rs        → AppState with Mutex<Connection> for SQLite
hotkey.rs       → Global shortcut (Cmd+Shift+Space) registration
commands/
  buffer.rs     → 9 Tauri commands: create, save, get, search, archive, delete, pin
db/
  connection.rs → SQLite connection with WAL mode, PRAGMA settings
  schema.rs     → Table creation + FTS5 virtual table + sync triggers
  queries.rs    → SQL query functions with prepared statements
```

**Database:** SQLite with FTS5 full-text search. Schema:
- `buffers` table: id (UUID), content, created_at, updated_at, accessed_at, is_archived, is_pinned
- `buffers_fts` virtual table (external content mode) for sub-millisecond search
- Auto-sync triggers maintain FTS index on INSERT/UPDATE/DELETE

### Frontend (src/)

```
routes/+page.svelte    → Main app layout, keyboard shortcuts, orchestration
lib/
  stores/buffers.svelte.ts → Svelte 5 runes state ($state, $derived)
  components/
    Editor.svelte      → CodeMirror 6 with Zed theme, debounced save
    Sidebar.svelte     → Buffer list + search input
    BufferItem.svelte  → Individual buffer row
    SearchResults.svelte → FTS results with <mark> highlighting
    CommandPalette.svelte → Cmd+P buffer switcher modal
    EditorHeader.svelte → Title + dirty indicator
```

### IPC Pattern

Frontend calls Rust via `invoke()`:
```typescript
import { invoke } from '@tauri-apps/api/core';
const id = await invoke<string>('create_buffer');
await invoke('save_buffer', { id, content });
const buffers = await invoke<BufferSummary[]>('get_sidebar_data');
```

Commands defined in `src-tauri/src/commands/buffer.rs` with `#[tauri::command]` macro.

## Key Patterns

### State Management (buffers.svelte.ts)
- Svelte 5 runes: `$state` for reactive values, `$derived` for computed
- Debounced save (500ms) on content changes
- Save-before-switch when selecting different buffer
- Auto-save on window blur

### Search
- FTS5 with prefix matching: `"term"*`
- Results limited to 20, sidebar limited to 100 buffers
- `highlight()` function generates `<mark>` tags for snippets

### Keyboard Shortcuts
- `Cmd+Shift+Space` → Toggle window (global, registered via tauri-plugin-global-shortcut)
- `Cmd+P` → Command palette
- `Cmd+N` → New buffer
- `Escape` → Clear search / close palette

## Styling

CSS variables in `src/app.css` (Zed dark theme):
```css
--bg-app: #151515;  --bg-editor: #191919;  --bg-active: #252525;
--border-subtle: #333333;  --text-main: #EBEBEC;  --text-muted: #868686;
--accent: #5898F8;
```

Font: JetBrains Mono (loaded via Google Fonts).

## Type Definitions

```typescript
interface BufferSummary {
  id: string;
  title: string;      // First non-empty line of content
  preview: string;    // Second non-empty line
  updated_at: number; // Unix timestamp
  is_pinned: boolean;
}

interface SearchResult {
  id: string;
  snippet: string;    // HTML with <mark> tags from FTS5 highlight()
  updated_at: number;
}
```

## macOS-Specific Features

- Tray icon (lives in menu bar)
- Hidden from Dock (Accessory activation policy)
- Chromeless window with overlay title bar
- Traffic light position: x=12, y=18
- Window state persistence via tauri-plugin-window-state
