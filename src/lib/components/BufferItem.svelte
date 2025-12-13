<script lang="ts">
  import { formatRelativeTime } from '$lib/utils/time';
  import type { BufferSummary } from '$lib/types';

  interface Props {
    buffer: BufferSummary;
    isActive: boolean;
    onSelect: () => void;
  }

  let { buffer, isActive, onSelect }: Props = $props();
</script>

<button
  class="w-full text-left px-4 py-2 flex items-center justify-between cursor-pointer hover:bg-[--bg-active] transition-colors"
  class:bg-[--bg-active]={isActive}
  class:border-l-2={isActive}
  class:border-[--accent]={isActive}
  onclick={onSelect}
>
  <div class="flex-1 min-w-0">
    <div class="flex items-center gap-1">
      {#if buffer.is_pinned}
        <span class="text-[--accent] text-[10px]">●</span>
      {/if}
      <div class="truncate text-[13px]" class:font-medium={isActive}>
        {buffer.title || 'Untitled'}
      </div>
    </div>
    <div class="truncate text-[11px] text-[--text-muted] mt-0.5">
      {buffer.preview || 'Empty buffer'}
    </div>
  </div>
  <span class="text-[10px] text-[--text-muted] ml-2 flex-shrink-0">
    {formatRelativeTime(buffer.updated_at)}
  </span>
</button>
