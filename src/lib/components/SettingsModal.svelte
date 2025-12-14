<script lang="ts">
  import { settingsStore, fontFamilies, fontSizes, lineHeights } from '$lib/stores/settings.svelte';

  interface Props {
    isOpen: boolean;
    onclose: () => void;
  }

  let { isOpen, onclose }: Props = $props();

  let dialogRef: HTMLDialogElement | null = $state(null);

  $effect(() => {
    if (isOpen && dialogRef) {
      dialogRef.showModal();
    } else if (!isOpen && dialogRef?.open) {
      dialogRef.close();
    }
  });

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      e.preventDefault();
      onclose();
    }
  }

  function handleBackdropClick(e: MouseEvent) {
    if (e.target === dialogRef) {
      onclose();
    }
  }
</script>

<dialog
  bind:this={dialogRef}
  onkeydown={handleKeydown}
  onclick={handleBackdropClick}
  onclose={onclose}
  class="bg-transparent p-0 m-0 max-w-none max-h-none w-full h-full backdrop:bg-black/50"
>
  <div class="fixed inset-0 flex items-center justify-center p-4 text-[--text-main]">
    <!-- svelte-ignore a11y_no_static_element_interactions a11y_click_events_have_key_events -->
    <div
      class="bg-[--bg-sidebar] border border-[--border-subtle] rounded-lg shadow-2xl w-full max-w-md"
      onclick={(e) => e.stopPropagation()}
    >
      <div class="flex items-center justify-between px-4 py-3 border-b border-[--border-subtle]">
        <h2 class="text-sm font-medium text-[--text-main]">Settings</h2>
        <button
          onclick={onclose}
          aria-label="Close settings"
          class="text-[--text-muted] hover:text-[--text-main] transition-colors"
        >
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
      </div>

      <div class="p-4 space-y-4">
        <div>
          <label for="font-family" class="block text-xs text-[--text-muted] mb-2">Font Family</label>
          <select
            id="font-family"
            value={settingsStore.settings.font_family}
            onchange={(e) => settingsStore.updateSetting('font_family', e.currentTarget.value)}
            class="w-full bg-[--bg-active] border border-[--border-subtle] rounded px-3 py-2.5 text-sm text-[--text-main] focus:outline-none focus:border-[--accent] cursor-pointer appearance-none"
            style="background-image: url('data:image/svg+xml;charset=UTF-8,%3csvg xmlns=%27http://www.w3.org/2000/svg%27 viewBox=%270 0 24 24%27 fill=%27none%27 stroke=%27%23D8DEE9%27 stroke-width=%272%27 stroke-linecap=%27round%27 stroke-linejoin=%27round%27%3e%3cpolyline points=%276 9 12 15 18 9%27%3e%3c/polyline%3e%3c/svg%3e'); background-repeat: no-repeat; background-position: right 0.75rem center; background-size: 1rem;"
          >
            {#each fontFamilies as font}
              <option value={font}>{font}</option>
            {/each}
          </select>
        </div>

        <div>
          <label for="font-size" class="block text-xs text-[--text-muted] mb-2">Font Size</label>
          <div class="flex items-center gap-2">
            <input
              id="font-size"
              type="range"
              min="10"
              max="20"
              step="1"
              value={settingsStore.settings.font_size}
              oninput={(e) => settingsStore.updateSetting('font_size', Number(e.currentTarget.value))}
              class="flex-1 accent-[--accent]"
            />
            <span class="text-sm w-12 text-right text-[--text-main]">{settingsStore.settings.font_size}px</span>
          </div>
        </div>

        <div>
          <label for="line-height" class="block text-xs text-[--text-muted] mb-2">Line Height</label>
          <div class="flex items-center gap-2">
            <input
              id="line-height"
              type="range"
              min="1.2"
              max="2.0"
              step="0.1"
              value={settingsStore.settings.line_height}
              oninput={(e) => settingsStore.updateSetting('line_height', Number(e.currentTarget.value))}
              class="flex-1 accent-[--accent]"
            />
            <span class="text-sm w-12 text-right text-[--text-main]">{settingsStore.settings.line_height.toFixed(1)}</span>
          </div>
        </div>

        <div>
          <div class="flex items-center justify-between">
            <label for="vim-mode" class="text-xs text-[--text-muted]">Vim Mode</label>
            <button
              id="vim-mode"
              type="button"
              role="switch"
              aria-label="Toggle vim mode"
              aria-checked={settingsStore.settings.vim_mode}
              onclick={() => settingsStore.toggleVimMode()}
              class="relative inline-flex h-5 w-9 flex-shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors duration-200 ease-in-out focus:outline-none focus:ring-2 focus:ring-[--accent] focus:ring-offset-2 focus:ring-offset-[--bg-sidebar]"
              class:bg-[--accent]={settingsStore.settings.vim_mode}
              class:bg-[--border-subtle]={!settingsStore.settings.vim_mode}
            >
              <span
                class="pointer-events-none inline-block h-4 w-4 transform rounded-full bg-white shadow ring-0 transition duration-200 ease-in-out"
                class:translate-x-4={settingsStore.settings.vim_mode}
                class:translate-x-0={!settingsStore.settings.vim_mode}
              ></span>
            </button>
          </div>
          <p class="text-xs text-[--text-muted] mt-1 opacity-60">Enable vim keybindings in the editor</p>
        </div>

        <div class="pt-3 border-t border-[--border-subtle]">
          <p class="text-xs text-[--text-muted] mb-2">Preview</p>
          <div class="bg-[--bg-editor] rounded p-3 border border-[--border-subtle]">
            <p style="font-family: var(--editor-font-family); font-size: var(--editor-font-size); line-height: var(--editor-line-height);" class="text-[--text-main]">The quick brown fox jumps over the lazy dog.</p>
          </div>
        </div>
      </div>

      <div class="px-4 py-3 border-t border-[--border-subtle] flex justify-end">
        <button
          onclick={onclose}
          class="px-4 py-1.5 text-sm bg-[--accent] text-white rounded hover:opacity-90 transition-opacity"
        >
          Done
        </button>
      </div>
    </div>
  </div>
</dialog>

<style>
  dialog::backdrop {
    background: rgba(0, 0, 0, 0.5);
  }

  select option {
    background: #434C5E;
    color: #ECEFF4;
    padding: 8px 12px;
  }

  input[type="range"] {
    height: 4px;
    background: var(--border-subtle);
    border-radius: 2px;
    -webkit-appearance: none;
    appearance: none;
  }

  input[type="range"]::-webkit-slider-thumb {
    -webkit-appearance: none;
    appearance: none;
    width: 14px;
    height: 14px;
    background: var(--accent);
    border-radius: 50%;
    cursor: pointer;
  }

  input[type="range"]::-webkit-slider-thumb:hover {
    background: #5E81AC;
    transform: scale(1.1);
  }
</style>
