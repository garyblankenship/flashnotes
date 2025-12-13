<script lang="ts">
  interface Props {
    title: string;
    isDirty: boolean;
  }

  let { title, isDirty }: Props = $props();
</script>

<header data-tauri-drag-region class="h-10 flex items-center px-6 border-b border-[--border-subtle] select-none">
  <span class="text-xs text-[--text-muted]">buffers /</span>
  <span class="text-xs ml-2 font-medium">
    {title || 'Untitled'}
  </span>
  {#if isDirty}
    <span class="ml-2 w-2 h-2 rounded-full bg-[--accent]" title="Unsaved changes"></span>
  {/if}
</header>
