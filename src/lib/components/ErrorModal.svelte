<script lang="ts">
  interface Props {
    error: string | null;
    onRetry?: () => void;
    onDismiss: () => void;
  }

  let { error, onRetry, onDismiss }: Props = $props();
</script>

{#if error}
  <div
    class="fixed inset-0 z-50 flex items-center justify-center bg-black/50"
    onclick={onDismiss}
    onkeydown={(e) => e.key === 'Escape' && onDismiss()}
    role="dialog"
    aria-modal="true"
    aria-labelledby="error-title"
    tabindex="-1"
  >
    <!-- svelte-ignore a11y_no_static_element_interactions a11y_click_events_have_key_events a11y_no_noninteractive_element_interactions -->
    <div
      class="bg-[--bg-editor] rounded-lg shadow-xl max-w-md w-full mx-4 p-6"
      onclick={(e) => e.stopPropagation()}
      role="document"
    >
      <h2 id="error-title" class="text-lg font-semibold text-[--text-main] mb-3">
        Error
      </h2>

      <p class="text-[--text-muted] mb-6 text-sm">
        {error}
      </p>

      <div class="flex gap-3 justify-end">
        <button
          onclick={onDismiss}
          class="px-4 py-2 text-sm text-[--text-muted] hover:text-[--text-main] transition-colors"
        >
          Dismiss
        </button>

        {#if onRetry}
          <button
            onclick={onRetry}
            class="px-4 py-2 text-sm bg-[--accent] text-[--bg-app] rounded hover:opacity-90 transition-opacity"
          >
            Retry
          </button>
        {/if}
      </div>
    </div>
  </div>
{/if}
