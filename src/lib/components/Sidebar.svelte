<script lang="ts">
  import BufferItem from './BufferItem.svelte';
  import SearchResults from './SearchResults.svelte';
  import { bufferStore } from '$lib/stores/buffers.svelte';

  interface Props {
    onSearch: (query: string) => void;
    onSelect: (id: string) => void;
    onCreate: () => void;
    onDelete: (id: string) => void;
    onTogglePin: (id: string) => void;
    onReorder: (ids: string[]) => void;
  }

  let { onSearch, onSelect, onCreate, onDelete, onTogglePin, onReorder }: Props = $props();

  // Reactive access to store - $derived creates subscription to store changes
  const buffers = $derived(bufferStore.sidebarBuffers);
  const searchResults = $derived(bufferStore.searchResults);
  const searchQuery = $derived(bufferStore.searchQuery);
  const activeBufferId = $derived(bufferStore.activeBufferId);
  const bufferCount = $derived(bufferStore.bufferCount);
  const isSearching = $derived(searchQuery.length > 0);

  let searchInput = $state('');
  let draggedId = $state<string | null>(null);
  let dropTargetId = $state<string | null>(null);

  function handleSearchInput(e: Event) {
    const value = (e.target as HTMLInputElement).value;
    searchInput = value;
    onSearch(value);
  }

  // Drag/drop handlers
  function handleDragStart(e: DragEvent, id: string, isPinned: boolean) {
    // Don't allow dragging pinned items (they stay at top)
    if (isPinned) {
      e.preventDefault();
      return;
    }
    draggedId = id;
    if (e.dataTransfer) {
      e.dataTransfer.effectAllowed = 'move';
      e.dataTransfer.setData('text/plain', id);
    }
  }

  function handleDragOver(e: DragEvent, id: string, isPinned: boolean) {
    e.preventDefault();
    // Can't drop onto pinned items
    if (isPinned || !draggedId || draggedId === id) return;
    dropTargetId = id;
  }

  function handleDragLeave() {
    dropTargetId = null;
  }

  function handleDrop(e: DragEvent, targetId: string) {
    e.preventDefault();
    if (!draggedId || draggedId === targetId) {
      resetDragState();
      return;
    }

    // Calculate new order and delegate to store
    const fromIndex = buffers.findIndex(b => b.id === draggedId);
    const toIndex = buffers.findIndex(b => b.id === targetId);

    if (fromIndex !== -1 && toIndex !== -1) {
      const ids = buffers.map(b => b.id);
      ids.splice(fromIndex, 1);
      // Adjust target index if we removed from before it
      const insertAt = fromIndex < toIndex ? toIndex - 1 : toIndex;
      ids.splice(insertAt, 0, draggedId);
      onReorder(ids);
    }

    resetDragState();
  }

  function resetDragState() {
    dropTargetId = null;
    draggedId = null;
  }
</script>

<aside class="w-64 flex flex-col border-r border-[--border-subtle]">
  <!-- Drag region for traffic lights area -->
  <div data-tauri-drag-region class="h-10 flex-shrink-0"></div>

  <!-- Search Bar -->
  <div class="px-3 pb-2">
    <input
      type="search"
      placeholder="Search buffers..."
      value={searchInput}
      oninput={handleSearchInput}
      class="w-full px-3 py-1.5 text-xs rounded bg-[--bg-active] border border-[--border-subtle] text-[--text-main] placeholder:text-[--text-muted] focus:outline-none focus:border-[--accent]"
    />
  </div>

  <!-- Buffer List -->
  <div class="flex-1 overflow-y-auto">
    {#if isSearching}
      <SearchResults
        results={searchResults}
        {activeBufferId}
        {onSelect}
      />
    {:else}
      {#each buffers as buffer (buffer.id)}
        <!-- svelte-ignore a11y_no_static_element_interactions -->
        <div
          role="listitem"
          draggable={!buffer.is_pinned}
          ondragstart={(e) => handleDragStart(e, buffer.id, buffer.is_pinned)}
          ondragover={(e) => handleDragOver(e, buffer.id, buffer.is_pinned)}
          ondragleave={handleDragLeave}
          ondrop={(e) => handleDrop(e, buffer.id)}
          ondragend={resetDragState}
          class:opacity-50={draggedId === buffer.id}
          class:border-t-2={dropTargetId === buffer.id}
          class:border-[--accent]={dropTargetId === buffer.id}
        >
          <BufferItem
            {buffer}
            isActive={buffer.id === activeBufferId}
            onSelect={() => onSelect(buffer.id)}
            onDelete={() => onDelete(buffer.id)}
            onTogglePin={() => onTogglePin(buffer.id)}
          />
        </div>
      {/each}
      {#if buffers.length === 0}
        <div class="px-4 py-8 text-center text-xs text-[--text-muted]">
          No buffers yet
          <br />
          <button
            class="mt-2 text-[--accent] hover:underline"
            onclick={onCreate}
          >
            Create one
          </button>
        </div>
      {/if}
    {/if}
  </div>

  <!-- Sidebar Footer -->
  <div class="h-10 border-t border-[--border-subtle] flex items-center justify-between px-4">
    <span class="text-xs text-[--text-muted]">{bufferCount} buffers</span>
    <button
      class="text-xs text-[--text-muted] hover:text-[--accent] transition-colors"
      onclick={onCreate}
      title="New buffer (⌘N)"
    >
      + New
    </button>
  </div>
</aside>
