# CLI Execution Flows

## Overview

Flashnotes is a macOS-native scratchpad application built with Tauri (Rust backend) and SvelteKit (TypeScript frontend). It provides an infinite-buffer, database-backed note-taking experience with full-text search, vim mode support, and real-time markdown preview. The application follows a desktop app pattern rather than traditional CLI commands, with all interactions happening through the GUI and keyboard shortcuts.

| Command | Subcommand(s) | Description | Main Implementation File(s) |
|---------|--------------|-------------|-----------------------------|
| npm run dev | | Development mode with Vite dev server | `package.json` |
| npm run build | | Production build of SvelteKit frontend | `package.json` |
| npm run tauri dev | | Development mode with hot-reload for both frontend and backend | `src-tauri/` |
| npm run tauri build | | Production build creating macOS app bundle | `src-tauri/` |
| npm test:e2e | | End-to-end test suite for buffer operations | `tests/test-buffer-ops.sh` |
| create_buffer | | Tauri IPC: Create new note buffer | `src-tauri/src/commands/buffer.rs` |
| save_buffer | | Tauri IPC: Save buffer content | `src-tauri/src/commands/buffer.rs` |
| get_buffer_content | | Tauri IPC: Retrieve buffer content | `src-tauri/src/commands/buffer.rs` |
| get_sidebar_data | | Tauri IPC: Get sidebar buffer list | `src-tauri/src/commands/buffer.rs` |
| search_buffers | | Tauri IPC: Full-text search via FTS5 | `src-tauri/src/commands/buffer.rs` |
| delete_buffer | | Tauri IPC: Delete current buffer | `src-tauri/src/commands/buffer.rs` |
| toggle_pin | | Tauri IPC: Pin/unpin buffer | `src-tauri/src/commands/buffer.rs` |
| get_settings | | Tauri IPC: Retrieve app settings | `src-tauri/src/commands/settings.rs` |
| set_setting | | Tauri IPC: Update app setting | `src-tauri/src/commands/settings.rs` |
| reorder_buffers | | Tauri IPC: Reorder buffers in sidebar | `src-tauri/src/commands/buffer.rs` |
| cleanup_empty_buffers | | Tauri IPC: Delete all empty buffers | `src-tauri/src/commands/buffer.rs` |
| toggle_always_on_top | | Tauri IPC: Toggle window stay-on-top | `src-tauri/src/commands/settings.rs` |

## Global Execution Model

### Application Startup Flow

1. **Process Entry**
   - Binary execution starts at `src-tauri/src/main.rs`
   - Calls `lib.rs::run()` which initializes Tauri application

2. **Tauri Initialization** (`src-tauri/src/lib.rs`)
   - Loads configuration from `tauri.conf.json`
   - Initializes plugins:
     - `tauri-plugin-opener`: For opening external URLs
     - `tauri-plugin-window-state`: Window persistence
     - `tauri-plugin-single-instance`: Prevents multiple app windows
   - Sets up `AppState` with SQLite database connection
   - Registers all IPC command handlers
   - Creates main window with overlay title bar

3. **Frontend Loading**
   - Development: Serves from Vite dev server at `http://127.0.0.1:1420`
   - Production: Loads from `../build` directory (built SvelteKit app)
   - Establishes IPC connection between frontend and Rust backend

4. **Database Initialization** (`src-tauri/src/db/schema.rs`)
   - Opens/creates SQLite database at:
     - Development: `./flashnotes_dev.db`
     - Production: `~/Library/Application Support/com.flashnotes.app/flashnotes.db`
   - Enables WAL mode, foreign keys, and optimizing PRAGMAs
   - Creates tables: `buffers`, `settings`, `buffers_fts`
   - Sets up triggers for FTS5 auto-sync

### Shared Components

- **Database Connection**: Managed via `AppState` with `Mutex<Connection>` for thread safety
- **Error Handling**: Unified `thiserror`-based error types mapped to Tauri responses
- **Window Management**: Single window instance with state persistence
- **Menu System**: Native macOS menu with command integration

## Command Flows

### npm run dev

**Synopsis**
```bash
npm run dev
```

**Description**
Starts Vite development server for the SvelteKit frontend at port 1420 with hot module replacement.

**Entry Point and Files**
- Primary file: `package.json` (script definition)
- Implementation: Vite dev server via SvelteKit
- Port: 1420 (configured in `tauri.conf.json`)

**Argument and Option Parsing**
No arguments. Uses SvelteKit's default Vite configuration.

**Execution Flow (Step-by-Step)**

1. **Startup**
   - npm executes script from `package.json`
   - Vite loads SvelteKit configuration
   - Starts development server on port 1420

2. **Development Features**
   - Hot Module Replacement for Svelte components
   - TypeScript compilation and type checking
   - Tailwind CSS processing with PostCSS
   - Asset serving and optimization

3. **Integration with Tauri**
   - `npm run tauri dev` command depends on this
   - Tauri loads frontend from `http://127.0.0.1:1420`

**External Interactions**
- Filesystem: Serves static assets from project directory
- Network: Local HTTP server on port 1420

**Error and Edge Case Flows**
- Port conflict: Vite automatically selects next available port
- TypeScript errors: Displayed in terminal and browser overlay
- Failed builds: Clear error messages with file locations

**Notes and Assumptions**
- Must be running before `tauri dev` for development mode
- Uses SvelteKit's default development configuration

### npm run tauri dev

**Synopsis**
```bash
npm run tauri dev
```

**Description**
Launches the application in development mode with both frontend (Vite) and backend (Tauri) hot-reload enabled.

**Entry Point and Files**
- Primary files: `src-tauri/src/main.rs`, `src-tauri/src/lib.rs`
- Configuration: `src-tauri/tauri.conf.json`
- Frontend: Served from Vite dev server (npm run dev)

**Argument and Option Parsing**
No CLI arguments. Configuration comes from `tauri.conf.json`.

**Execution Flow (Step-by-Step)**

1. **Pre-start**
   - Executes `beforeDevCommand`: `npm run dev`
   - Waits for frontend server at `http://127.0.0.1:1420`

2. **Rust Compilation**
   - Compiles Rust backend with debug symbols
   - Links with system libraries (SQLite on macOS)

3. **Tauri Runtime**
   - Initializes WebView with frontend URL
   - Creates application window with overlay title bar
   - Establishes IPC context for command communication

4. **Development Features**
   - Auto-reloads on Rust code changes
   - Hot-reloads on frontend changes via Vite
   - Debug console available
   - Database uses `flashnotes_dev.db`

**External Interactions**
- Database: SQLite at `./flashnotes_dev.db`
- Frontend: HTTP connection to Vite dev server
- macOS: Creates application bundle in memory

**Error and Edge Case Flows**
- Frontend server down: Waits and retries connection
- Rust compilation errors: Displayed with full stack trace
- Database errors: Logged but doesn't crash app

**Notes and Assumptions**
- Requires `npm run dev` to be executed first
- Uses development database (separate from production)

### npm run tauri build

**Synopsis**
```bash
npm run tauri build
```

**Description**
Creates production build of the entire application, generating a macOS app bundle and DMG installer.

**Entry Point and Files**
- Primary files: `src-tauri/src/main.rs`, `src-tauri/Cargo.toml`
- Frontend: Built to `../build` directory
- Output: `src-tauri/target/release/bundle/macos/`

**Argument and Option Parsing**
No arguments. Build configuration from `tauri.conf.json` and `Cargo.toml`.

**Execution Flow (Step-by-Step)**

1. **Frontend Build**
   - Executes `beforeBuildCommand`: `npm run build`
   - SvelteKit builds optimized static site to `../build`
   - Assets minified and fingerprinted

2. **Rust Compilation**
   - Compiles in release mode with optimizations
   - Bundles SQLite and all dependencies
   - Links frontend assets into binary

3. **App Bundle Creation**
   - Creates macOS app structure
   - Copies icons and metadata
   - Sets up Info.plist with app ID `com.flashnotes.notes`

4. **DMG Generation**
   - Creates disk image installer
   - Configures with custom background and layout
   - Outputs to `bundle/macos/` directory

**External Interactions**
- Filesystem: Writes build artifacts
- macOS tools: Uses `codesign` for app signing (if configured)

**Error and Edge Case Flows**
- Frontend build failures: Stops before Rust compilation
- Missing dependencies: Clear cargo error messages
- Code signing failures: Warning but build continues

**Notes and Assumptions**
- Creates standalone app bundle
- Production database location: `~/Library/Application Support/com.flashnotes.app/`

### create_buffer (IPC Command)

**Synopsis**
```javascript
await invoke('create_buffer', { content?: string })
```

**Description**
Creates a new note buffer with optional initial content. Assigns unique ID, title, and sort order.

**Entry Point and Files**
- Primary file: `src-tauri/src/commands/buffer.rs`
- Function: `create_buffer(state: State<AppState>, content: Option<String>)`

**Argument and Option Parsing**
| Argument/Option | Type | Required? | Default | Description | Affects Flow? (Y/N) |
|-----------------|------|-----------|---------|-------------|----------------------|
| content | string | no | undefined | Initial buffer content | N |

**Execution Flow (Step-by-Step)**

1. **Parameter Processing**
   - Extract optional content from command parameters
   - Generate new UUID for buffer ID

2. **Database Transaction**
   - Lock database connection via `state.db.lock()`
   - Determine new sort_order (current max + 1)
   - INSERT into buffers table with:
     - Generated UUID
     - Content (if provided)
     - Current timestamps
     - archived=false, pinned=false

3. **Response Generation**
   - Generate title from first line or "Untitled"
   - Create preview from first 100 characters
   - Return BufferSummary with all metadata

4. **Frontend Integration**
   - BufferStore receives response
   - Updates active buffer immediately
   - Refreshes sidebar list

**External Interactions**
- Database: INSERT into buffers table
- FTS5: Trigger auto-updates search index

**Error and Edge Case Flows**
- Database lock timeout: Returns database error
- Invalid UUID: Extremely unlikely, but returns generic error
- Empty content: Creates empty buffer with "Untitled" title

**Notes and Assumptions**
- Always creates new buffer at top of sort order
- Title extraction handles markdown headers
- Preview strips markdown formatting

### save_buffer (IPC Command)

**Synopsis**
```javascript
await invoke('save_buffer', { id: string, content: string })
```

**Description**
Saves buffer content and updates metadata. Returns new title and preview for UI updates.

**Entry Point and Files**
- Primary file: `src-tauri/src/commands/buffer.rs`
- Function: `save_buffer(state: State<AppState>, id: String, content: String)`

**Argument and Option Parsing**
| Argument/Option | Type | Required? | Default | Description | Affects Flow? (Y/N) |
|-----------------|------|-----------|---------|-------------|----------------------|
| id | string | yes | - | Buffer UUID | Y |
| content | string | yes | - | Buffer content | N |

**Execution Flow (Step-by-Step)**

1. **Validation**
   - Parse buffer ID from string
   - Validate content is not null

2. **Database Update**
   - Lock database connection
   - UPDATE buffers SET:
     - content = provided content
     - updated_at = current timestamp
   - WHERE id matches buffer ID

3. **Response Generation**
   - Generate new title from first line
   - Create preview from content
   - Return SaveResponse with title and preview

4. **Frontend Integration**
   - Debounced save (500ms) in BufferStore
   - Updates buffer title in sidebar
   - Marks buffer as clean (not dirty)

**External Interactions**
- Database: UPDATE on buffers table
- FTS5: Trigger auto-updates search index

**Error and Edge Case Flows**
- Buffer not found: Returns "Buffer not found" error
- Database lock: Retries or returns timeout error
- Invalid UUID: Returns parsing error

**Notes and Assumptions**
- Called from frontend debounced save mechanism
- Always updates updated_at timestamp
- Title extraction is consistent with create_buffer

### get_sidebar_data (IPC Command)

**Synopsis**
```javascript
await invoke('get_sidebar_data')
```

**Description**
Retrieves all non-archived buffers for sidebar display, ordered by pinned status and sort order.

**Entry Point and Files**
- Primary file: `src-tauri/src/commands/buffer.rs`
- Function: `get_sidebar_data(state: State<AppState>)`

**Argument and Option Parsing**
No arguments.

**Execution Flow (Step-by-Step)**

1. **Database Query**
   - Lock database connection
   - SELECT all non-archived buffers
   - ORDER BY pinned DESC, sort_order ASC

2. **Response Mapping**
   - Convert each row to BufferSummary
   - Include title, preview, timestamps
   - Preserve sort_order for reordering

3. **Frontend Integration**
   - Populates sidebar buffer list
   - Separates pinned and unpinned sections
   - Updates buffer selection state

**External Interactions**
- Database: SELECT query on buffers table

**Error and Edge Case Flows**
- Database empty: Returns empty vector
- Connection error: Returns database error
- No non-archived buffers: Returns empty vector

**Notes and Assumptions**
- Always returns results in correct order
- Archives buffers are filtered out
- Used for initial load and refreshes

### search_buffers (IPC Command)

**Synopsis**
```javascript
await invoke('search_buffers', { query: string })
```

**Description**
Performs full-text search using SQLite FTS5, returning highlighted snippets and matching buffers.

**Entry Point and Files**
- Primary file: `src-tauri/src/commands/buffer.rs`
- Function: `search_buffers(state: State<AppState>, query: String)`

**Argument and Option Parsing**
| Argument/Option | Type | Required? | Default | Description | Affects Flow? (Y/N) |
|-----------------|------|-----------|---------|-------------|----------------------|
| query | string | yes | - | Search query text | Y |

**Execution Flow (Step-by-Step)**

1. **Query Preparation**
   - Trim whitespace from query
   - Prepare FTS5 MATCH query with highlighting

2. **Database Search**
   - Join buffers table with buffers_fts
   - Search using FTS5 MATCH syntax
   - Return highlighted snippets (bm25() relevance)
   - ORDER BY relevance score

3. **Response Generation**
   - Map results to SearchResult objects
   - Include buffer metadata and highlighted content
   - Return vector of search results

4. **Frontend Integration**
   - Updates search results in BufferStore
   - Highlights matching terms in preview
   - Allows direct buffer selection from search

**External Interactions**
- Database: FTS5 virtual table query
- Text processing: SQLite snippet highlighting

**Error and Edge Case Flows**
- Empty query: Returns empty results
- No matches: Returns empty vector
- FTS5 errors: Falls back to LIKE query

**Notes and Assumptions**
- Uses FTS5 for fast full-text search
- Highlighting preserves context around matches
- Results ordered by relevance (bm25 score)

### delete_buffer (IPC Command)

**Synopsis**
```javascript
await invoke('delete_buffer', { id: string })
```

**Description**
Deletes a buffer and determines which buffer should become active next.

**Entry Point and Files**
- Primary file: `src-tauri/src/commands/buffer.rs`
- Function: `delete_buffer(state: State<AppState>, id: String)`

**Argument and Option Parsing**
| Argument/Option | Type | Required? | Default | Description | Affects Flow? (Y/N) |
|-----------------|------|-----------|---------|-------------|----------------------|
| id | string | yes | - | Buffer UUID to delete | Y |

**Execution Flow (Step-by-Step)**

1. **Database Transaction**
   - Begin transaction
   - Find buffer with next highest sort_order (previous in list)
   - If no previous, find buffer with next lowest sort_order (next in list)
   - DELETE buffer with specified ID
   - Commit transaction

2. **Response Generation**
   - Return ID of buffer that should become active
   - Return null if no buffers remain

3. **Frontend Integration**
   - BufferStore handles buffer deletion
   - Switches to returned buffer ID or creates new one
   - Updates sidebar list immediately

**External Interactions**
- Database: DELETE from buffers table
- FTS5: Trigger auto-updates search index

**Error and Edge Case Flows**
- Buffer not found: Returns null for next buffer
- Last buffer deleted: Returns null to trigger new buffer creation
- Database error: Transaction rolls back

**Notes and Assumptions**
- Smart buffer selection preserves context
- Single buffer deletion handled gracefully
- Triggers archive cleanup if needed

### npm test:e2e

**Synopsis**
```bash
npm test:e2e
```

**Description**
End-to-end test suite that validates buffer creation, reordering, deletion, and application stability.

**Entry Point and Files**
- Primary file: `tests/test-buffer-ops.sh`
- Test app: `/Users/vampire/www/flashnotes/src-tauri/target/release/bundle/macos/Flashnotes.app`

**Argument and Option Parsing**
No arguments. Uses hardcoded paths and values.

**Execution Flow (Step-by-Step)**

1. **Setup**
   - Kill any existing Flashnotes processes
   - Launch production app bundle
   - Wait for process to initialize

2. **Buffer Creation Tests**
   - Create 3 test buffers (AAA-First, BBB-Second, CCC-Third)
   - Each buffer created with Cmd+N keyboard shortcut
   - Content typed to identify buffers

3. **Reorder Tests**
   - Attempt buffer reordering with Cmd+Shift+↓
   - Attempt buffer reordering with Cmd+Shift+↑
   - Verify sort_order values in database
   - Note: AppleScript limitations in testing webview keyboard events

4. **Deletion Tests**
   - Delete all 3 test buffers with Cmd+W
   - Verify application remains stable
   - Confirm window still exists

5. **Cleanup**
   - Quit application
   - Report test results

**External Interactions**
- macOS: AppleScript for keyboard simulation and process control
- Database: Direct SQLite queries to verify state
- Filesystem: App bundle execution

**Error and Edge Case Flows**
- App not found: Fails with clear error
- Process crash: Detected and reported
- Database inaccessible: Logged but doesn't fail test
- Window closed unexpectedly: Test failure

**Exit Codes**
- 0: All tests passed
- 1: App failed to launch or crashed
- 2: Window closed unexpectedly
- 3: Sort order validation failed

**Notes and Assumptions**
- Requires production build to exist
- Hardcoded paths for test environment
- Some tests may require manual verification due to AppleScript limitations

## Shared Components and Utilities

### Database Layer (`src-tauri/src/db/`)

**Components:**
- `schema.rs` - Database initialization and table creation
- `queries.rs` - High-level query functions
- `connection.rs` - Connection management

**Used by:** All IPC commands
**Responsibilities:**
- SQLite database with WAL mode for concurrency
- FTS5 virtual table for full-text search
- Automatic triggers for search index sync
- Connection pooling via AppState

### AppState (`src-tauri/src/state.rs`)

**Used by:** All command handlers
**Responsibilities:**
- Thread-safe database access via Mutex<Connection>
- Dependency injection for Tauri commands
- Shared state across command invocations

### BufferStore (`src/lib/stores/buffers.svelte.ts`)

**Used by:** Frontend Svelte components
**Responsibilities:**
- Reactive state management with Svelte 5 runes
- Debounced auto-save (500ms)
- Optimistic UI updates
- Command invocation wrapper

### SettingsStore (`src/lib/stores/settings.svelte.ts`)

**Used by:** SettingsModal, main app
**Responsibilities:**
- Persistent app settings
- Font configuration
- Window state (always-on-top)
- Editor mode toggles

### Menu System (`src-tauri/src/lib.rs`)

**Used by:** Application initialization
**Responsibilities:**
- Native macOS menu bar
- Command shortcuts
- "Stay on Top" toggle integration
- GitHub link integration

## Assumptions and Limitations

### Assumptions
1. **macOS-only Application**: Uses macOS-specific APIs and private APIs
2. **Single Instance**: Enforced by Tauri plugin, prevents multiple windows
3. **SQLite Bundle**: Assumes SQLite is available on target system
4. **Development Environment**: Assumes npm and Rust toolchain are installed
5. **Database Location**: Assumes standard macOS app support directory exists

### Limitations
1. **No CLI Arguments**: Application doesn't support command-line arguments for direct file opening
2. **AppleScript Testing**: E2E tests limited by AppleScript's ability to send keyboard events to webviews
3. **Single Window**: Architecture doesn't support multiple editor windows
4. **No Export Functionality**: No built-in export to other formats
5. **No Plugin System**: Extensibility limited to built-in features

### Missing Information
1. **Crash Reporting**: No automated crash reporting or analytics
2. **Auto-Update**: No auto-update mechanism configured
3. **Backup Strategy**: No automatic backup or sync functionality
4. **Performance Metrics**: No built-in performance monitoring
5. **Accessibility**: No explicit accessibility features documented

### Architecture Decisions Not Explicitly Stated
1. **Why SQLite**: Chosen for portability and FTS5 support
2. **Why Tauri v2**: Explicit upgrade from v1 for improved capabilities
3. **Why Svelte 5**: Early adoption for runes-based reactivity
4. **Why CodeMirror 6**: For extensibility and vim support
5. **No Cloud Sync**: Intentionally local-only for privacy