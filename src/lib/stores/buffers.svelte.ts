import { invoke } from '@tauri-apps/api/core';
import type { BufferSummary, SearchResult } from '$lib/types';

// Reactive state using Svelte 5 runes
let sidebarBuffers = $state<BufferSummary[]>([]);
let searchResults = $state<SearchResult[]>([]);
let searchQuery = $state('');
let activeBufferId = $state<string | null>(null);
let activeContent = $state('');
let isDirty = $state(false);
let isLoading = $state(false);
let bufferCount = $state(0);
let lastError = $state<string | null>(null);

// Derived state
const displayList = $derived(searchQuery.length > 0 ? searchResults : sidebarBuffers);
const isSearching = $derived(searchQuery.length > 0);

// Error handling utility
function handleError(message: string, error: unknown): void {
  const errorMsg = error instanceof Error ? error.message : String(error);
  lastError = `${message}: ${errorMsg}`;
  console.error(lastError);
}

// Actions
async function loadSidebarData(): Promise<void> {
  try {
    lastError = null;
    sidebarBuffers = await invoke<BufferSummary[]>('get_sidebar_data');
    bufferCount = sidebarBuffers.length;
  } catch (error) {
    handleError('Failed to load sidebar data', error);
  }
}

async function searchBuffers(query: string): Promise<void> {
  searchQuery = query;
  if (query.length > 0) {
    try {
      lastError = null;
      searchResults = await invoke<SearchResult[]>('search_buffers', { query });
    } catch (error) {
      handleError('Failed to search buffers', error);
      searchResults = [];
    }
  } else {
    searchResults = [];
  }
}

function clearSearch(): void {
  searchQuery = '';
  searchResults = [];
}

async function selectBuffer(id: string): Promise<void> {
  if (id === activeBufferId) return;

  // Save current buffer if dirty before switching
  if (isDirty && activeBufferId) {
    await saveCurrentBuffer();
  }

  try {
    isLoading = true;
    lastError = null;
    activeContent = await invoke<string>('get_buffer_content', { id });
    activeBufferId = id;
    isDirty = false;
  } catch (error) {
    handleError('Failed to get buffer content', error);
  } finally {
    isLoading = false;
  }
}

async function createBuffer(initialContent?: string): Promise<string | null> {
  try {
    lastError = null;
    const id = await invoke<string>('create_buffer');

    // If initial content provided, save it immediately
    if (initialContent) {
      await invoke('save_buffer', { id, content: initialContent });
    }

    await loadSidebarData();
    await selectBuffer(id);
    return id;
  } catch (error) {
    handleError('Failed to create buffer', error);
    return null;
  }
}

async function saveCurrentBuffer(): Promise<void> {
  if (!activeBufferId || !isDirty) return;

  try {
    lastError = null;
    await invoke('save_buffer', { id: activeBufferId, content: activeContent });
    isDirty = false;
    await loadSidebarData();
  } catch (error) {
    handleError('Failed to save buffer', error);
  }
}

function updateContent(content: string): void {
  if (content !== activeContent) {
    activeContent = content;
    isDirty = true;
  }
}

async function archiveBuffer(id: string): Promise<void> {
  try {
    lastError = null;
    await invoke('archive_buffer', { id });
    if (activeBufferId === id) {
      activeBufferId = null;
      activeContent = '';
    }
    await loadSidebarData();
  } catch (error) {
    handleError('Failed to archive buffer', error);
  }
}

async function deleteBuffer(id: string): Promise<void> {
  try {
    lastError = null;
    await invoke('delete_buffer_permanently', { id });
    if (activeBufferId === id) {
      activeBufferId = null;
      activeContent = '';
    }
    await loadSidebarData();
  } catch (error) {
    handleError('Failed to delete buffer', error);
  }
}

async function togglePin(id: string): Promise<void> {
  try {
    lastError = null;
    await invoke('toggle_pin', { id });
    await loadSidebarData();
  } catch (error) {
    handleError('Failed to toggle pin', error);
  }
}

// Export store as object with getters for reactive access
export const bufferStore = {
  // Reactive getters
  get sidebarBuffers() { return sidebarBuffers; },
  get searchResults() { return searchResults; },
  get searchQuery() { return searchQuery; },
  get activeBufferId() { return activeBufferId; },
  get activeContent() { return activeContent; },
  get isDirty() { return isDirty; },
  get isLoading() { return isLoading; },
  get bufferCount() { return bufferCount; },
  get lastError() { return lastError; },
  get displayList() { return displayList; },
  get isSearching() { return isSearching; },

  // Actions
  loadSidebarData,
  searchBuffers,
  clearSearch,
  selectBuffer,
  createBuffer,
  saveCurrentBuffer,
  updateContent,
  archiveBuffer,
  deleteBuffer,
  togglePin,
};
