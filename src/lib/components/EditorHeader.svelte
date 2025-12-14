<script lang="ts">
  import { getCurrentWindow } from '@tauri-apps/api/window';

  interface Props {
    title: string;
    isDirty: boolean;
    alwaysOnTop: boolean;
    onToggleAlwaysOnTop: () => void;
  }

  let { title, isDirty, alwaysOnTop, onToggleAlwaysOnTop }: Props = $props();

  async function handleMouseDown(e: MouseEvent) {
    // Only drag on left click
    if (e.button === 0) {
      e.preventDefault();
      try {
        await getCurrentWindow().startDragging();
      } catch (err) {
        console.error('Failed to start window drag:', err);
      }
    }
  }

  function handlePinClick(e: MouseEvent) {
    e.stopPropagation();
    onToggleAlwaysOnTop();
  }
</script>

<header
  onmousedown={handleMouseDown}
  class="h-10 flex items-center justify-between px-6 border-b border-[--border-subtle] select-none cursor-move">
  <div class="flex items-center pointer-events-none">
    <span class="text-xs text-[--text-muted]">notes /</span>
    <span class="text-xs ml-2 font-medium">
      {title || 'Untitled'}
    </span>
    {#if isDirty}
      <span class="ml-2 w-2 h-2 rounded-full bg-[--accent]" title="Unsaved changes"></span>
    {/if}
  </div>
  <button
    onclick={handlePinClick}
    class="pointer-events-auto p-1.5 rounded transition-colors {alwaysOnTop ? 'text-[--accent]' : 'text-[--text-muted] hover:text-[--text-main]'}"
    title="Stay on top (⌘⇧T)"
  >
    <svg class="w-4 h-4" fill={alwaysOnTop ? 'currentColor' : 'none'} stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
      <path stroke-linecap="round" stroke-linejoin="round" d="M16 4V7L18 9V11H13L13 19L12 22L11 19V11H6V9L8 7V4H16Z" />
    </svg>
  </button>
</header>
