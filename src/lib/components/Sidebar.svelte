<script lang="ts">
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import BufferItem from './BufferItem.svelte';
  import SearchResults from './SearchResults.svelte';
  import { bufferStore } from '$lib/stores/buffers.svelte';

  async function startWindowDrag(e: MouseEvent) {
    if (e.button === 0) {
      e.preventDefault();
      try {
        await getCurrentWindow().startDragging();
      } catch (err) {
        console.error('Failed to start window drag:', err);
      }
    }
  }

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
  let insertPosition = $state<'before' | 'after' | null>(null);
  let liveAnnouncement = $state('');
  let mouseDownTime = $state(0);
  let isDragging = $state(false);

  function handleSearchInput(e: Event) {
    const value = (e.target as HTMLInputElement).value;
    searchInput = value;
    onSearch(value);
  }

  // Announce to screen readers
  function announce(message: string) {
    liveAnnouncement = '';
    setTimeout(() => { liveAnnouncement = message; }, 50);
  }

  // Mouse-based drag (more reliable than HTML5 drag API)
  function handleMouseDown(e: MouseEvent, id: string, isPinned: boolean) {
    if (isPinned || e.button !== 0) return;

    // Record time to differentiate click vs drag
    mouseDownTime = Date.now();
    draggedId = id;

    const handleMouseMove = (moveEvent: MouseEvent) => {
      // Only start visual drag after 100ms or 5px movement
      if (!isDragging && (Date.now() - mouseDownTime > 100)) {
        isDragging = true;
      }

      if (!isDragging) return;

      // Find element under cursor
      const elements = document.elementsFromPoint(moveEvent.clientX, moveEvent.clientY);
      const bufferEl = elements.find(el => el.getAttribute('data-buffer-id'));

      if (bufferEl) {
        const targetId = bufferEl.getAttribute('data-buffer-id')!;
        const targetBuffer = buffers.find(b => b.id === targetId);

        if (targetId !== draggedId && targetBuffer && !targetBuffer.is_pinned) {
          dropTargetId = targetId;
          const rect = bufferEl.getBoundingClientRect();
          const midY = rect.top + rect.height / 2;
          insertPosition = moveEvent.clientY < midY ? 'before' : 'after';
        }
      } else {
        dropTargetId = null;
      }
    };

    const handleMouseUp = () => {
      document.removeEventListener('mousemove', handleMouseMove);
      document.removeEventListener('mouseup', handleMouseUp);

      // If it was a quick click (not a drag), select the buffer
      if (!isDragging || Date.now() - mouseDownTime < 150) {
        onSelect(id);
        resetDragState();
        return;
      }

      // Complete the reorder
      if (dropTargetId && draggedId && dropTargetId !== draggedId) {
        const fromIndex = buffers.findIndex(b => b.id === draggedId);
        const toIndex = buffers.findIndex(b => b.id === dropTargetId);

        if (fromIndex !== -1 && toIndex !== -1) {
          const ids = buffers.map(b => b.id);
          ids.splice(fromIndex, 1);

          let insertAt = toIndex;
          if (fromIndex < toIndex) {
            insertAt = insertPosition === 'after' ? toIndex : toIndex - 1;
          } else {
            insertAt = insertPosition === 'after' ? toIndex + 1 : toIndex;
          }

          ids.splice(insertAt, 0, draggedId);
          onReorder(ids);
        }
      }

      resetDragState();
    };

    document.addEventListener('mousemove', handleMouseMove);
    document.addEventListener('mouseup', handleMouseUp);
  }

  function resetDragState() {
    dropTargetId = null;
    draggedId = null;
    insertPosition = null;
    isDragging = false;
    mouseDownTime = 0;
  }

</script>

<aside class="w-64 flex flex-col border-r border-[--border-subtle]" aria-label="Buffer sidebar">
  <!-- Screen reader announcements -->
  <div class="sr-only" role="status" aria-live="polite" aria-atomic="true">
    {liveAnnouncement}
  </div>

  <!-- Drag region for traffic lights area -->
  <div onmousedown={startWindowDrag} class="h-10 flex-shrink-0 cursor-move"></div>

  <!-- Search Bar -->
  <div class="px-3 pb-2">
    <input
      type="search"
      placeholder="Search buffers..."
      value={searchInput}
      oninput={handleSearchInput}
      aria-label="Search buffers"
      class="w-full px-3 py-1.5 text-xs rounded bg-[--bg-active] border border-[--border-subtle] text-[--text-main] placeholder:text-[--text-muted] focus:outline-none focus:border-[--accent]"
    />
  </div>

  <!-- Buffer List -->
  <div class="flex-1 overflow-y-auto" role="list" aria-label="Buffers">
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
          data-buffer-id={buffer.id}
          aria-label="{buffer.title}{buffer.is_pinned ? ' (pinned)' : ''}"
          onmousedown={(e) => handleMouseDown(e, buffer.id, buffer.is_pinned)}
          class="transition-all select-none"
          class:opacity-50={draggedId === buffer.id && isDragging}
          class:border-t-2={dropTargetId === buffer.id && insertPosition === 'before'}
          class:border-b-2={dropTargetId === buffer.id && insertPosition === 'after'}
          class:border-[--accent]={dropTargetId === buffer.id}
          class:cursor-grab={!buffer.is_pinned && !isDragging}
          class:cursor-grabbing={isDragging}
        >
          <BufferItem
            {buffer}
            isActive={buffer.id === activeBufferId}
            isDragging={draggedId === buffer.id && isDragging}
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
