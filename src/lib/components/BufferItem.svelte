<script lang="ts">
  import { formatRelativeTime } from '$lib/utils/time';
  import type { BufferSummary } from '$lib/types';

  interface Props {
    buffer: BufferSummary;
    isActive: boolean;
    onSelect: () => void;
    onDelete: () => void;
    onTogglePin: () => void;
  }

  let { buffer, isActive, onSelect, onDelete, onTogglePin }: Props = $props();

  function handleDelete(e: MouseEvent) {
    e.stopPropagation();
    if (confirm('Delete this buffer?')) {
      onDelete();
    }
  }

  function handleTogglePin(e: MouseEvent) {
    e.stopPropagation();
    onTogglePin();
  }
</script>

<div
  class="group w-full text-left px-4 py-2 flex items-center justify-between cursor-pointer hover:bg-[--bg-active] transition-colors"
  class:bg-[--bg-active]={isActive}
  class:border-l-2={isActive}
  class:border-[--accent]={isActive}
  onclick={onSelect}
  onkeydown={(e) => e.key === 'Enter' && onSelect()}
  role="button"
  tabindex="0"
>
  <div class="flex-1 min-w-0">
    <div class="truncate text-[13px]" class:font-medium={isActive}>
      {buffer.title || 'Untitled'}
    </div>
    <div class="truncate text-[11px] text-[--text-muted] mt-0.5">
      {buffer.preview || 'Empty buffer'}
    </div>
  </div>
  <div class="flex items-center gap-1 ml-2 flex-shrink-0">
    <button
      class="p-1 transition-opacity {buffer.is_pinned ? 'text-[--accent]' : 'opacity-0 group-hover:opacity-100 text-[--text-muted] hover:text-[--accent]'}"
      onclick={handleTogglePin}
      title={buffer.is_pinned ? 'Unpin buffer' : 'Pin buffer'}
    >
      <svg class="w-3 h-3" fill={buffer.is_pinned ? 'currentColor' : 'none'} stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 5a2 2 0 012-2h10a2 2 0 012 2v16l-7-3.5L5 21V5z" />
      </svg>
    </button>
    <button
      class="opacity-0 group-hover:opacity-100 text-[--text-muted] hover:text-red-400 transition-opacity p-1"
      onclick={handleDelete}
      title="Delete buffer"
    >
      <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
      </svg>
    </button>
    <span class="text-[10px] text-[--text-muted] ml-1">
      {formatRelativeTime(buffer.updated_at)}
    </span>
  </div>
</div>
