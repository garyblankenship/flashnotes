/**
 * Keyboard shortcuts hook for the app
 * Centralizes all keyboard shortcut handling
 */

export interface KeyboardShortcutHandlers {
  onOpenPalette: () => void;
  onCreateBuffer: () => void;
  onDeleteBuffer: () => void;
  onOpenSettings: () => void;
  onTogglePreview: () => void;
  onMoveBufferUp: () => void;
  onMoveBufferDown: () => void;
  onClearSearch: () => void;
  onToggleAlwaysOnTop: () => void;
  onCopyToClipboard: () => void;
  onToggleSidebar: () => void;
}

export interface KeyboardShortcutState {
  isPaletteOpen: boolean;
  isSettingsOpen: boolean;
}

/**
 * Creates a keyboard event handler for app shortcuts
 */
export function createKeyboardHandler(
  handlers: KeyboardShortcutHandlers,
  getState: () => KeyboardShortcutState
) {
  return function handleKeydown(e: KeyboardEvent) {
    const state = getState();

    // Cmd+P: Open command palette
    if (e.key === 'p' && e.metaKey && !e.shiftKey) {
      e.preventDefault();
      handlers.onOpenPalette();
      return;
    }

    // Cmd+N: New buffer
    if (e.key === 'n' && e.metaKey && !e.shiftKey) {
      e.preventDefault();
      handlers.onCreateBuffer();
      return;
    }

    // Cmd+W: Delete current buffer
    if (e.key === 'w' && e.metaKey && !e.shiftKey) {
      e.preventDefault();
      handlers.onDeleteBuffer();
      return;
    }

    // Cmd+,: Open settings
    if (e.key === ',' && e.metaKey) {
      e.preventDefault();
      handlers.onOpenSettings();
      return;
    }

    // Cmd+E: Toggle preview mode
    if (e.key === 'e' && e.metaKey && !e.shiftKey) {
      e.preventDefault();
      handlers.onTogglePreview();
      return;
    }

    // Cmd+Shift+Up: Move buffer up
    if (e.key === 'ArrowUp' && e.metaKey && e.shiftKey) {
      e.preventDefault();
      handlers.onMoveBufferUp();
      return;
    }

    // Cmd+Shift+Down: Move buffer down
    if (e.key === 'ArrowDown' && e.metaKey && e.shiftKey) {
      e.preventDefault();
      handlers.onMoveBufferDown();
      return;
    }

    // Escape: Clear search (when no modal open)
    if (e.key === 'Escape' && !state.isPaletteOpen && !state.isSettingsOpen) {
      handlers.onClearSearch();
      return;
    }

    // Cmd+Shift+T: Toggle always on top
    if (e.key === 't' && e.metaKey && e.shiftKey) {
      e.preventDefault();
      handlers.onToggleAlwaysOnTop();
      return;
    }

    // Cmd+Shift+C: Copy to clipboard
    if (e.key === 'c' && e.metaKey && e.shiftKey) {
      e.preventDefault();
      handlers.onCopyToClipboard();
      return;
    }

    // Cmd+B: Toggle sidebar
    if (e.key === 'b' && e.metaKey && !e.shiftKey) {
      e.preventDefault();
      handlers.onToggleSidebar();
      return;
    }
  };
}
