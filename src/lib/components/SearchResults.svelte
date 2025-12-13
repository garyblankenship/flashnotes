<script lang="ts">
  import { formatRelativeTime } from '$lib/utils/time';
  import type { SearchResult } from '$lib/types';

  interface Props {
    results: SearchResult[];
    activeBufferId: string | null;
    onSelect: (id: string) => void;
  }

  let { results, activeBufferId, onSelect }: Props = $props();
</script>

{#each results as result (result.id)}
  <button
    class="w-full text-left px-4 py-2 cursor-pointer hover:bg-[--bg-active] transition-colors"
    class:bg-[--bg-active]={result.id === activeBufferId}
    class:border-l-2={result.id === activeBufferId}
    class:border-[--accent]={result.id === activeBufferId}
    onclick={() => onSelect(result.id)}
  >
    <div class="text-[12px] line-clamp-2">
      {@html result.snippet}
    </div>
    <div class="text-[10px] text-[--text-muted] mt-1">
      {formatRelativeTime(result.updated_at)}
    </div>
  </button>
{/each}

{#if results.length === 0}
  <div class="px-4 py-8 text-center text-xs text-[--text-muted]">
    No results found
  </div>
{/if}

<style>
  .line-clamp-2 {
    display: -webkit-box;
    -webkit-line-clamp: 2;
    line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }

  :global(mark) {
    background-color: rgba(88, 152, 248, 0.3);
    color: inherit;
    padding: 0 2px;
    border-radius: 2px;
  }
</style>
