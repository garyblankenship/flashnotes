<script lang="ts">
  import { toastStore } from '$lib/stores/toast.svelte';
</script>

{#if toastStore.message}
  <div
    class="fixed bottom-6 left-1/2 -translate-x-1/2 bg-[--bg-active] border border-[--border-subtle] text-[--text-main] px-4 py-2 rounded-lg shadow-lg text-sm font-medium z-50 animate-fade-in"
  >
    {toastStore.message}
  </div>
{/if}

<style>
  @keyframes fade-in {
    from {
      opacity: 0;
      transform: translate(-50%, 10px);
    }
    to {
      opacity: 1;
      transform: translate(-50%, 0);
    }
  }

  .animate-fade-in {
    animation: fade-in 0.2s ease-out;
  }
</style>
