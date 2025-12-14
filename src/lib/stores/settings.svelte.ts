/**
 * Settings store - manages app settings with backend persistence
 * Uses Svelte 5 class-based reactive pattern (consistent with BufferStore)
 */
import { invoke } from '@tauri-apps/api/core';
import type { AppSettings } from '$lib/types';

// Default settings
const defaults: AppSettings = {
  font_family: 'JetBrains Mono',
  font_size: 13,
  line_height: 1.5,
  preview_mode: false,
  sidebar_width: 256,
  sidebar_collapsed: false,
  vim_mode: false,
  always_on_top: false,
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

class SettingsStore {
  settings = $state<AppSettings>({ ...defaults });
  isLoading = $state(false);
  lastError = $state<string | null>(null);

  private handleError(message: string, error: unknown): void {
    const errorMsg = error instanceof Error ? error.message : String(error);
    this.lastError = `${message}: ${errorMsg}`;
    console.error(this.lastError);
  }

  async loadSettings(): Promise<void> {
    try {
      this.isLoading = true;
      this.lastError = null;
      this.settings = await invoke<AppSettings>('get_settings');
      this.applyToDocument();
    } catch (error) {
      this.handleError('Failed to load settings', error);
      this.settings = { ...defaults };
    } finally {
      this.isLoading = false;
    }
  }

  async updateSetting<K extends keyof AppSettings>(
    key: K,
    value: AppSettings[K]
  ): Promise<void> {
    try {
      this.lastError = null;
      await invoke('set_setting', { key, value: String(value) });
      this.settings = { ...this.settings, [key]: value };
      this.applyToDocument();
    } catch (error) {
      this.handleError(`Failed to update ${key}`, error);
    }
  }

  private applyToDocument(): void {
    const root = document.documentElement;
    root.style.setProperty('--editor-font-family', `'${this.settings.font_family}', monospace`);
    root.style.setProperty('--editor-font-size', `${this.settings.font_size}px`);
    root.style.setProperty('--editor-line-height', String(this.settings.line_height));
  }

  // Toggle preview mode (client-side only, no backend persistence)
  togglePreviewMode(): void {
    this.settings = { ...this.settings, preview_mode: !this.settings.preview_mode };
  }

  // Toggle vim mode (persisted)
  async toggleVimMode(): Promise<void> {
    const newValue = !this.settings.vim_mode;
    await this.updateSetting('vim_mode', newValue);
  }

  // Toggle always on top (persisted)
  async toggleAlwaysOnTop(): Promise<void> {
    const newValue = !this.settings.always_on_top;
    await this.updateSetting('always_on_top', newValue);
  }

  // Toggle sidebar collapsed (persisted)
  async toggleSidebarCollapsed(): Promise<void> {
    const newValue = !this.settings.sidebar_collapsed;
    await this.updateSetting('sidebar_collapsed', newValue);
  }
}

export const settingsStore = new SettingsStore();
