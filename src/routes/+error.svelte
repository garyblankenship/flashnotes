<script lang="ts">
  import { page } from '$app/stores';

  function handleReload() {
    window.location.reload();
  }
</script>

<div class="h-screen w-screen flex items-center justify-center bg-[--bg-app] text-[--text-main]">
  <div class="max-w-md p-8 text-center">
    <h1 class="text-2xl font-bold mb-4 text-[--accent]">Something went wrong</h1>

    <p class="text-[--text-muted] mb-6">
      {$page.error?.message || 'An unexpected error occurred'}
    </p>

    <div class="space-y-3">
      <button
        onclick={handleReload}
        class="w-full px-4 py-2 bg-[--accent] text-[--bg-app] rounded hover:opacity-90 transition-opacity"
      >
        Reload Application
      </button>

      <p class="text-xs text-[--text-muted]">
        If this problem persists, check the logs at:<br />
        <code class="text-[--accent-secondary]">~/Library/Application Support/com.flashnotes.notes/logs/</code>
      </p>
    </div>
  </div>
</div>
