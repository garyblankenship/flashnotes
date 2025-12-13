import { invoke } from '@tauri-apps/api/core';
import type { AppSettings } from '$lib/types';

// Default settings
const defaults: AppSettings = {
  font_family: 'JetBrains Mono',
  font_size: 13,
  line_height: 1.5,
  preview_mode: false,
};

// Available font options
export const fontFamilies = [
  'JetBrains Mono',
  'SF Mono',
  'Menlo',
  'Monaco',
  'Fira Code',
  'Source Code Pro',
  'Cascadia Code',
  'IBM Plex Mono',
] as const;

export const fontSizes = [10, 11, 12, 13, 14, 15, 16, 18, 20] as const;
export const lineHeights = [1.2, 1.4, 1.5, 1.6, 1.8, 2.0] as const;

// Reactive state
let settings = $state<AppSettings>({ ...defaults });
let isLoading = $state(false);
let lastError = $state<string | null>(null);

// Error handling
function handleError(message: string, error: unknown): void {
  const errorMsg = error instanceof Error ? error.message : String(error);
  lastError = `${message}: ${errorMsg}`;
  console.error(lastError);
}

// Actions
async function loadSettings(): Promise<void> {
  try {
    isLoading = true;
    lastError = null;
    settings = await invoke<AppSettings>('get_settings');
    applyToDocument();
  } catch (error) {
    handleError('Failed to load settings', error);
    settings = { ...defaults };
  } finally {
    isLoading = false;
  }
}

async function updateSetting<K extends keyof AppSettings>(
  key: K,
  value: AppSettings[K]
): Promise<void> {
  try {
    lastError = null;
    await invoke('set_setting', { key, value: String(value) });
    settings = { ...settings, [key]: value };
    applyToDocument();
  } catch (error) {
    handleError(`Failed to update ${key}`, error);
  }
}

// Apply settings to CSS custom properties
function applyToDocument(): void {
  const root = document.documentElement;
  root.style.setProperty('--editor-font-family', `'${settings.font_family}', monospace`);
  root.style.setProperty('--editor-font-size', `${settings.font_size}px`);
  root.style.setProperty('--editor-line-height', String(settings.line_height));
}

// Toggle preview mode (client-side only, no backend persistence)
function togglePreviewMode(): void {
  settings = { ...settings, preview_mode: !settings.preview_mode };
}

// Export store
export const settingsStore = {
  get settings() { return settings; },
  get isLoading() { return isLoading; },
  get lastError() { return lastError; },

  loadSettings,
  updateSetting,
  togglePreviewMode,
};
