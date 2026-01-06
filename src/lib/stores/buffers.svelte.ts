import { invoke } from '@tauri-apps/api/core';
import type { BufferSummary, SearchResult } from '$lib/types';

// Extract title and preview from content (mirrors backend logic)
function extractTitlePreview(content: string): { title: string; preview: string } {
  const lines = content.split('\n');

  const titleLine = lines.find(line => line.trim().length > 0);
  const title = titleLine?.trim().slice(0, 100) || 'Untitled';

  // Find first non-empty line after title
  let foundTitle = false;
  let preview = '';
  for (const line of lines) {
    if (!foundTitle) {
      if (line.trim().length > 0) foundTitle = true;
      continue;
    }
    if (line.trim().length > 0) {
      preview = line.trim().slice(0, 100);
      break;
    }
  }

  return { title, preview };
}

// Retry helper for save operations
async function withRetry<T>(
  operation: () => Promise<T>,
  maxRetries: number = 3,
  delayMs: number = 1000
): Promise<T> {
  let lastError: unknown;
  for (let attempt = 1; attempt <= maxRetries; attempt++) {
    try {
      return await operation();
    } catch (error) {
      lastError = error;
      if (attempt < maxRetries) {
        await new Promise(resolve => setTimeout(resolve, delayMs * attempt));
      }
    }
  }
  throw lastError;
}

// Class-based reactive store for Svelte 5
class BufferStore {
  sidebarBuffers = $state<BufferSummary[]>([]);
  searchResults = $state<SearchResult[]>([]);
  searchQuery = $state('');
  activeBufferId = $state<string | null>(null);
  activeContent = $state('');
  isDirty = $state(false);
  isLoading = $state(false);
  isLoadingMore = $state(false);
  hasMoreBuffers = $state(true);
  lastError = $state<string | null>(null);
  saveError = $state<string | null>(null);
  saveRetryCount = $state(0);

  // Derived state
  displayList = $derived(this.searchQuery.length > 0 ? this.searchResults : this.sidebarBuffers);
  isSearching = $derived(this.searchQuery.length > 0);
  bufferCount = $derived(this.sidebarBuffers.length);

  // Error handling utility
  private handleError(message: string, error: unknown): void {
    const errorMsg = error instanceof Error ? error.message : String(error);
    this.lastError = `${message}: ${errorMsg}`;
    console.error(this.lastError);
  }

  // Clear errors
  clearErrors(): void {
    this.lastError = null;
    this.saveError = null;
    this.saveRetryCount = 0;
  }

  // Actions
  async loadSidebarData(): Promise<void> {
    try {
      this.lastError = null;
      this.sidebarBuffers = await invoke<BufferSummary[]>('get_sidebar_data', { offset: null });
      this.hasMoreBuffers = this.sidebarBuffers.length >= 100;
    } catch (error) {
      this.handleError('Failed to load sidebar data', error);
    }
  }

  async loadMoreBuffers(): Promise<void> {
    if (this.isLoadingMore || !this.hasMoreBuffers || this.isSearching) return;

    try {
      this.isLoadingMore = true;
      this.lastError = null;
      const offset = this.sidebarBuffers.length;
      const moreBuffers = await invoke<BufferSummary[]>('get_sidebar_data', { offset });

      if (moreBuffers.length < 100) {
        this.hasMoreBuffers = false;
      }

      if (moreBuffers.length > 0) {
        this.sidebarBuffers = [...this.sidebarBuffers, ...moreBuffers];
      }
    } catch (error) {
      this.handleError('Failed to load more buffers', error);
    } finally {
      this.isLoadingMore = false;
    }
  }

  async searchBuffers(query: string): Promise<void> {
    this.searchQuery = query;
    if (query.length > 0) {
      try {
        this.lastError = null;
        this.searchResults = await invoke<SearchResult[]>('search_buffers', { query });
      } catch (error) {
        this.handleError('Failed to search buffers', error);
        this.searchResults = [];
      }
    } else {
      this.searchResults = [];
    }
  }

  clearSearch(): void {
    this.searchQuery = '';
    this.searchResults = [];
  }

  async selectBuffer(id: string): Promise<void> {
    if (id === this.activeBufferId) return;

    // Save current buffer if dirty before switching
    if (this.isDirty && this.activeBufferId) {
      await this.saveCurrentBuffer();
    }

    try {
      this.isLoading = true;
      this.lastError = null;
      this.activeContent = await invoke<string>('get_buffer_content', { id });
      this.activeBufferId = id;
      this.isDirty = false;
    } catch (error) {
      this.handleError('Failed to get buffer content', error);
    } finally {
      this.isLoading = false;
    }
  }

  async createBuffer(initialContent?: string): Promise<string | null> {
    try {
      this.lastError = null;
      // Backend creates buffer and returns summary in one call
      const summary = await invoke<BufferSummary>('create_buffer', {
        content: initialContent ?? null
      });

      // Update UI directly - no refetch needed
      this.sidebarBuffers = [summary, ...this.sidebarBuffers];
      this.activeBufferId = summary.id;
      this.activeContent = initialContent ?? '';
      this.isDirty = false;

      return summary.id;
    } catch (error) {
      this.handleError('Failed to create buffer', error);
      return null;
    }
  }

  async saveCurrentBuffer(): Promise<boolean> {
    if (!this.activeBufferId || !this.isDirty) return true;

    try {
      this.lastError = null;
      this.saveError = null;

      // Use retry for save operations
      const [title, preview] = await withRetry(
        () => invoke<[string, string]>('save_buffer', {
          id: this.activeBufferId,
          content: this.activeContent
        }),
        3, // max retries
        1000 // initial delay
      );

      this.isDirty = false;
      this.saveRetryCount = 0;

      // Update sidebar locally - no refetch needed
      this.sidebarBuffers = this.sidebarBuffers.map(b =>
        b.id === this.activeBufferId
          ? { ...b, title, preview, updated_at: Date.now() / 1000 }
          : b
      );

      return true;
    } catch (error) {
      const errorMsg = error instanceof Error ? error.message : String(error);
      this.saveError = `Failed to save: ${errorMsg}`;
      this.saveRetryCount++;
      console.error('Save failed after retries:', error);
      return false;
    }
  }

  // Manual retry for save (user-initiated)
  async retrySave(): Promise<boolean> {
    if (!this.saveError) return true;
    return this.saveCurrentBuffer();
  }

  updateContent(content: string): void {
    if (content !== this.activeContent) {
      this.activeContent = content;
      this.isDirty = true;

      // Update sidebar title/preview immediately (optimistic)
      if (this.activeBufferId) {
        const { title, preview } = extractTitlePreview(content);
        this.sidebarBuffers = this.sidebarBuffers.map(b =>
          b.id === this.activeBufferId ? { ...b, title, preview } : b
        );
      }
    }
  }

  async deleteBuffer(id: string): Promise<void> {
    try {
      this.lastError = null;
      const wasActive = this.activeBufferId === id;

      // Delete and get next buffer to select
      const nextId = await invoke<string | null>('delete_buffer', { id });

      // Update sidebar locally
      this.sidebarBuffers = this.sidebarBuffers.filter(b => b.id !== id);

      // Select next buffer if we deleted the active one
      if (wasActive) {
        if (nextId) {
          await this.selectBuffer(nextId);
        } else {
          this.activeBufferId = null;
          this.activeContent = '';
          this.isDirty = false;
        }
      }
    } catch (error) {
      this.handleError('Failed to delete buffer', error);
      throw error;
    }
  }

  async togglePin(id: string): Promise<void> {
    try {
      this.lastError = null;
      // Backend returns new pin state
      const isPinned = await invoke<boolean>('toggle_pin', { id });

      // Update locally and re-sort (pinned items first)
      this.sidebarBuffers = this.sidebarBuffers
        .map(b => b.id === id ? { ...b, is_pinned: isPinned } : b)
        .sort((a, b) => {
          if (a.is_pinned !== b.is_pinned) return a.is_pinned ? -1 : 1;
          return 0; // Keep relative order otherwise
        });
    } catch (error) {
      this.handleError('Failed to toggle pin', error);
    }
  }

  async reorderBuffers(ids: string[]): Promise<void> {
    try {
      this.lastError = null;
      // Update UI immediately (optimistic update)
      const idToBuffer = new Map(this.sidebarBuffers.map(b => [b.id, b]));
      this.sidebarBuffers = ids.map(id => idToBuffer.get(id)!).filter(Boolean);

      // Persist to backend (no refetch needed)
      await invoke('reorder_buffers', { ids });
    } catch (error) {
      this.handleError('Failed to reorder buffers', error);
      // Refetch on error to restore correct state
      await this.loadSidebarData();
    }
  }

  // Move active buffer up in the list (Cmd+Shift+Up)
  async moveBufferUp(): Promise<boolean> {
    if (!this.activeBufferId) return false;

    const index = this.sidebarBuffers.findIndex(b => b.id === this.activeBufferId);
    // Can't move up if first, pinned, or buffer above is pinned
    if (index <= 0) return false;
    if (this.sidebarBuffers[index].is_pinned) return false;
    if (this.sidebarBuffers[index - 1].is_pinned) return false;

    const ids = this.sidebarBuffers.map(b => b.id);
    [ids[index - 1], ids[index]] = [ids[index], ids[index - 1]];
    await this.reorderBuffers(ids);
    return true;
  }

  // Move active buffer down in the list (Cmd+Shift+Down)
  async moveBufferDown(): Promise<boolean> {
    if (!this.activeBufferId) return false;

    const index = this.sidebarBuffers.findIndex(b => b.id === this.activeBufferId);
    // Can't move down if last or pinned
    if (index === -1 || index >= this.sidebarBuffers.length - 1) return false;
    if (this.sidebarBuffers[index].is_pinned) return false;

    const ids = this.sidebarBuffers.map(b => b.id);
    [ids[index], ids[index + 1]] = [ids[index + 1], ids[index]];
    await this.reorderBuffers(ids);
    return true;
  }
}

// Export singleton instance
export const bufferStore = new BufferStore();
