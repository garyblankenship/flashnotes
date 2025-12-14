<script lang="ts">
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import BufferItem from './BufferItem.svelte';
  import SearchResults from './SearchResults.svelte';
  import { bufferStore } from '$lib/stores/buffers.svelte';
  import { settingsStore } from '$lib/stores/settings.svelte';

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
    if (e.button !== 0) return;

    // Record time to differentiate click vs drag
    mouseDownTime = Date.now();

    // Pinned items can be selected but not dragged
    if (isPinned) {
      // Use mouseup for selection to match non-pinned behavior
      const handlePinnedMouseUp = () => {
        document.removeEventListener('mouseup', handlePinnedMouseUp);
        if (Date.now() - mouseDownTime < 300) {
          onSelect(id);
        }
        mouseDownTime = 0;
      };
      document.addEventListener('mouseup', handlePinnedMouseUp);
      return;
    }

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

  // Sidebar resize
  const MIN_WIDTH = 180;
  const MAX_WIDTH = 400;
  let isResizing = $state(false);
  const sidebarWidth = $derived(settingsStore.settings.sidebar_width);
  const isCollapsed = $derived(settingsStore.settings.sidebar_collapsed);

  function handleResizeStart(e: MouseEvent) {
    if (e.button !== 0) return;
    e.preventDefault();
    isResizing = true;

    const handleResizeMove = (moveEvent: MouseEvent) => {
      const newWidth = Math.min(MAX_WIDTH, Math.max(MIN_WIDTH, moveEvent.clientX));
      settingsStore.settings.sidebar_width = newWidth;
    };

    const handleResizeEnd = () => {
      document.removeEventListener('mousemove', handleResizeMove);
      document.removeEventListener('mouseup', handleResizeEnd);
      isResizing = false;
      settingsStore.updateSetting('sidebar_width', settingsStore.settings.sidebar_width);
    };

    document.addEventListener('mousemove', handleResizeMove);
    document.addEventListener('mouseup', handleResizeEnd);
  }

</script>

{#if isCollapsed}
  <!-- Collapsed sidebar - just a thin strip with expand button -->
  <aside class="flex flex-col border-r border-[--border-subtle] w-10" aria-label="Sidebar (collapsed)">
    <!-- Drag region for traffic lights area -->
    <div onmousedown={startWindowDrag} class="h-10 flex-shrink-0 cursor-move"></div>

    <!-- Expand button -->
    <button
      onclick={() => settingsStore.toggleSidebarCollapsed()}
      class="flex-1 flex items-start justify-center pt-2 text-[--text-muted] hover:text-[--accent] transition-colors"
      title="Expand sidebar (⌘B)"
    >
      <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 5l7 7-7 7M5 5l7 7-7 7" />
      </svg>
    </button>
  </aside>
{:else}
  <aside class="flex flex-col border-r border-[--border-subtle] relative" style:width="{sidebarWidth}px" aria-label="Note sidebar">
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
        placeholder="Search notes..."
        value={searchInput}
        oninput={handleSearchInput}
        aria-label="Search notes"
        class="w-full px-3 py-2 text-sm rounded bg-[--bg-active] border border-[--border-subtle] text-[--text-main] placeholder:text-[--text-muted] focus:outline-none focus:border-[--accent]"
      />
    </div>

    <!-- Note List -->
    <div class="flex-1 overflow-y-auto" role="list" aria-label="Notes">
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
            No notes yet
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
      <span class="text-xs text-[--text-muted]">{bufferCount} notes</span>
      <button
        class="text-xs text-[--text-muted] hover:text-[--accent] transition-colors"
        onclick={onCreate}
        title="New note (⌘N)"
      >
        + New
      </button>
    </div>

    <!-- Resize Handle -->
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div
      onmousedown={handleResizeStart}
      class="absolute top-0 right-0 w-1 h-full cursor-col-resize hover:bg-[--accent] transition-colors"
      class:bg-[--accent]={isResizing}
    ></div>
  </aside>
{/if}
