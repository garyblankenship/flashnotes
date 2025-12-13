<script lang="ts">
  import { onMount } from 'svelte';
  import { bufferStore } from '$lib/stores/buffers.svelte';
  import { formatRelativeTime } from '$lib/utils/time';
  import type { BufferSummary, SearchResult } from '$lib/types';

  interface Props {
    isOpen: boolean;
    onclose: () => void;
    onselect: (id: string) => void;
  }

  let { isOpen, onclose, onselect }: Props = $props();

  let searchInput = $state('');
  let selectedIndex = $state(0);
  let inputRef: HTMLInputElement | null = $state(null);

  // Get filtered results
  const results = $derived.by(() => {
    if (searchInput.length === 0) {
      return bufferStore.sidebarBuffers;
    }
    const query = searchInput.toLowerCase();
    return bufferStore.sidebarBuffers.filter(buffer =>
      buffer.title.toLowerCase().includes(query) ||
      buffer.preview.toLowerCase().includes(query)
    );
  });

  // Reset selection when results change
  $effect(() => {
    if (results.length > 0 && selectedIndex >= results.length) {
      selectedIndex = 0;
    }
  });

  // Focus input when opened
  $effect(() => {
    if (isOpen && inputRef) {
      inputRef.focus();
      searchInput = '';
      selectedIndex = 0;
    }
  });

  function handleKeydown(e: KeyboardEvent) {
    switch (e.key) {
      case 'ArrowDown':
        e.preventDefault();
        selectedIndex = Math.min(selectedIndex + 1, results.length - 1);
        break;
      case 'ArrowUp':
        e.preventDefault();
        selectedIndex = Math.max(selectedIndex - 1, 0);
        break;
      case 'Enter':
        e.preventDefault();
        if (results[selectedIndex]) {
          handleSelect(results[selectedIndex].id);
        }
        break;
      case 'Escape':
        e.preventDefault();
        onclose();
        break;
    }
  }

  function handleSelect(id: string) {
    onselect(id);
    onclose();
  }

  function handleBackdropClick(e: MouseEvent) {
    if (e.target === e.currentTarget) {
      onclose();
    }
  }
</script>

{#if isOpen}
  <!-- Backdrop -->
  <div
    class="fixed inset-0 bg-black/50 backdrop-blur-sm z-50 flex items-start justify-center pt-[15vh]"
    onclick={handleBackdropClick}
    onkeydown={handleKeydown}
    role="dialog"
    aria-modal="true"
    tabindex="-1"
  >
    <!-- Palette Container -->
    <div class="w-[500px] max-h-[60vh] bg-[--bg-app] border border-[--border-subtle] rounded-lg shadow-2xl overflow-hidden">
      <!-- Search Input -->
      <div class="p-3 border-b border-[--border-subtle]">
        <input
          bind:this={inputRef}
          type="text"
          placeholder="Search buffers..."
          value={searchInput}
          oninput={(e) => searchInput = (e.target as HTMLInputElement).value}
          class="w-full px-3 py-2 text-sm bg-transparent text-[--text-main] placeholder:text-[--text-muted] outline-none"
        />
      </div>

      <!-- Results List -->
      <div class="max-h-[400px] overflow-y-auto">
        {#if results.length === 0}
          <div class="p-4 text-center text-sm text-[--text-muted]">
            No buffers found
          </div>
        {:else}
          {#each results as buffer, index (buffer.id)}
            <button
              class="w-full text-left px-4 py-3 flex items-center gap-3 cursor-pointer transition-colors"
              class:bg-[--bg-active]={index === selectedIndex}
              onmouseenter={() => selectedIndex = index}
              onclick={() => handleSelect(buffer.id)}
            >
              <div class="flex-1 min-w-0">
                <div class="flex items-center gap-2">
                  {#if buffer.is_pinned}
                    <span class="text-[--accent] text-[10px]">●</span>
                  {/if}
                  <span class="text-sm font-medium truncate">
                    {buffer.title || 'Untitled'}
                  </span>
                </div>
                {#if buffer.preview}
                  <div class="text-xs text-[--text-muted] truncate mt-0.5">
                    {buffer.preview}
                  </div>
                {/if}
              </div>
              <span class="text-[10px] text-[--text-muted] flex-shrink-0">
                {formatRelativeTime(buffer.updated_at)}
              </span>
            </button>
          {/each}
        {/if}
      </div>

      <!-- Footer Hints -->
      <div class="px-4 py-2 border-t border-[--border-subtle] flex items-center gap-4 text-[10px] text-[--text-muted]">
        <span>↑↓ Navigate</span>
        <span>↵ Select</span>
        <span>esc Close</span>
      </div>
    </div>
  </div>
{/if}
