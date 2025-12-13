<script lang="ts">
  import { getCurrentWindow } from '@tauri-apps/api/window';

  interface Props {
    title: string;
    isDirty: boolean;
  }

  let { title, isDirty }: Props = $props();

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
</script>

<header
  onmousedown={handleMouseDown}
  class="h-10 flex items-center px-6 border-b border-[--border-subtle] select-none cursor-move">
  <span class="text-xs text-[--text-muted] pointer-events-none">buffers /</span>
  <span class="text-xs ml-2 font-medium pointer-events-none">
    {title || 'Untitled'}
  </span>
  {#if isDirty}
    <span class="ml-2 w-2 h-2 rounded-full bg-[--accent] pointer-events-none" title="Unsaved changes"></span>
  {/if}
</header>
