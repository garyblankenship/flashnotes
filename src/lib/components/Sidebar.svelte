<script lang="ts">
  import BufferItem from './BufferItem.svelte';
  import SearchResults from './SearchResults.svelte';
  import type { BufferSummary, SearchResult } from '$lib/types';

  interface Props {
    buffers: BufferSummary[];
    searchResults: SearchResult[];
    searchQuery: string;
    activeBufferId: string | null;
    bufferCount: number;
    onSearch: (query: string) => void;
    onSelect: (id: string) => void;
    onCreate: () => void;
    onDelete: (id: string) => void;
  }

  let {
    buffers,
    searchResults,
    searchQuery,
    activeBufferId,
    bufferCount,
    onSearch,
    onSelect,
    onCreate,
    onDelete,
  }: Props = $props();

  let searchInput = $state('');

  function handleSearchInput(e: Event) {
    const value = (e.target as HTMLInputElement).value;
    searchInput = value;
    onSearch(value);
  }

  const isSearching = $derived(searchQuery.length > 0);
</script>

<aside class="w-64 flex flex-col border-r border-[--border-subtle] pt-10">
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
        <BufferItem
          {buffer}
          isActive={buffer.id === activeBufferId}
          onSelect={() => onSelect(buffer.id)}
          onDelete={() => onDelete(buffer.id)}
        />
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
