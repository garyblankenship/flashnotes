<p align="center">
  <img src="src-tauri/icons/128x128@2x.png" alt="Flashnotes" width="128" height="128">
</p>

<h1 align="center">Flashnotes</h1>

<p align="center">
  An infinite-buffer, database-backed scratchpad for macOS with Zed-like aesthetics.
</p>

<p align="center">
  <img src="screenshot.png" alt="Flashnotes Screenshot" width="800">
</p>

## Features

- **Instant capture** - Global hotkey (Cmd+Shift+Space) summons the app from anywhere
- **No files, no saving** - Everything auto-saves to SQLite with WAL mode
- **Lightning search** - Full-text search powered by FTS5 (sub-millisecond)
- **Command palette** - Cmd+P for quick buffer switching
- **Zed-inspired UI** - Dark theme, chromeless window, JetBrains Mono font

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Cmd+Shift+Space` | Toggle window visibility (global) |
| `Cmd+P` | Open command palette |
| `Cmd+N` | Create new buffer |
| `Cmd+E` | Toggle markdown preview |
| `Cmd+,` | Open settings |
| `Escape` | Clear search / close palette |

## Development

```bash
# Install dependencies
npm install

# Run in development mode
npm run tauri dev

# Build for production
npm run tauri build
```

## Tech Stack

- **Frontend:** Svelte 5 + Tailwind CSS + CodeMirror 6
- **Backend:** Rust (Tauri v2)
- **Database:** SQLite with FTS5 full-text search
- **Font:** JetBrains Mono

## Troubleshooting

### Cargo permission denied

If you see `Permission denied (os error 13)` when running `npm run tauri dev`:

```bash
# Check cargo permissions
ls -la $(which cargo)

# Fix permissions
chmod +x ~/.cargo/bin/*

# Or reinstall Rust (recommended)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

This usually happens when Rust was installed with `sudo` or the toolchain got corrupted.
