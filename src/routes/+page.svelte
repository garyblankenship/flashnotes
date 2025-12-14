<script lang="ts">
  import { onMount } from 'svelte';
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import { invoke } from '@tauri-apps/api/core';
  import Editor from '$lib/components/Editor.svelte';
  import Sidebar from '$lib/components/Sidebar.svelte';
  import EditorHeader from '$lib/components/EditorHeader.svelte';
  import CommandPalette from '$lib/components/CommandPalette.svelte';
  import SettingsModal from '$lib/components/SettingsModal.svelte';
  import { bufferStore } from '$lib/stores/buffers.svelte';
  import { settingsStore } from '$lib/stores/settings.svelte';
  import { debounce } from '$lib/utils/debounce';

  let editorRef: Editor | null = $state(null);
  let isPaletteOpen = $state(false);
  let isSettingsOpen = $state(false);
  let toastMessage = $state<string | null>(null);
  let toastTimeout: ReturnType<typeof setTimeout> | null = null;

  function showToast(message: string, duration = 2000) {
    if (toastTimeout) clearTimeout(toastTimeout);
    toastMessage = message;
    toastTimeout = setTimeout(() => {
      toastMessage = null;
    }, duration);
  }

  async function handleCopyToClipboard() {
    const content = editorRef?.getContent() ?? bufferStore.activeContent;
    if (content) {
      try {
        await navigator.clipboard.writeText(content);
        showToast('Copied to clipboard');
      } catch {
        showToast('Failed to copy');
      }
    }
  }

  async function handleToggleAlwaysOnTop() {
    try {
      const newState = await invoke<boolean>('toggle_always_on_top');
      await settingsStore.loadSettings(); // Sync frontend state
      showToast(newState ? 'Stay on Top: On' : 'Stay on Top: Off');
    } catch (error) {
      console.error('Failed to toggle always on top:', error);
    }
  }

  const debouncedSave = debounce(() => bufferStore.saveCurrentBuffer(), 500);

  // Derived state
  const activeTitle = $derived(
    bufferStore.sidebarBuffers.find(b => b.id === bufferStore.activeBufferId)?.title ?? ''
  );

  // Event handlers
  function handleEditorChange(content: string) {
    bufferStore.updateContent(content);
    debouncedSave();
  }

  function handleSearch(query: string) {
    bufferStore.searchBuffers(query);
  }

  function handleSelectBuffer(id: string) {
    bufferStore.selectBuffer(id);
  }

  async function handleCreateBuffer() {
    await bufferStore.createBuffer();
    editorRef?.focus();
  }

  async function handleDeleteBuffer(id: string) {
    await bufferStore.deleteBuffer(id);
  }

  async function handleTogglePin(id: string) {
    await bufferStore.togglePin(id);
  }

  async function handleReorder(ids: string[]) {
    await bufferStore.reorderBuffers(ids);
  }

  function handlePaletteSelect(id: string) {
    bufferStore.selectBuffer(id);
    isPaletteOpen = false;
    setTimeout(() => editorRef?.focus(), 50);
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'p' && e.metaKey && !e.shiftKey) {
      e.preventDefault();
      isPaletteOpen = true;
    } else if (e.key === 'n' && e.metaKey && !e.shiftKey) {
      e.preventDefault();
      handleCreateBuffer();
    } else if (e.key === 'w' && e.metaKey && !e.shiftKey) {
      e.preventDefault();
      if (bufferStore.activeBufferId) {
        handleDeleteBuffer(bufferStore.activeBufferId);
      }
    } else if (e.key === ',' && e.metaKey) {
      e.preventDefault();
      isSettingsOpen = true;
    } else if (e.key === 'e' && e.metaKey && !e.shiftKey) {
      e.preventDefault();
      settingsStore.togglePreviewMode();
    } else if (e.key === 'ArrowUp' && e.metaKey && e.shiftKey) {
      e.preventDefault();
      bufferStore.moveBufferUp();
    } else if (e.key === 'ArrowDown' && e.metaKey && e.shiftKey) {
      e.preventDefault();
      bufferStore.moveBufferDown();
    } else if (e.key === 'Escape' && !isPaletteOpen && !isSettingsOpen) {
      bufferStore.clearSearch();
      editorRef?.focus();
    } else if (e.key === 't' && e.metaKey && e.shiftKey) {
      e.preventDefault();
      handleToggleAlwaysOnTop();
    } else if (e.key === 'c' && e.metaKey && e.shiftKey) {
      e.preventDefault();
      handleCopyToClipboard();
    } else if (e.key === 'b' && e.metaKey && !e.shiftKey) {
      e.preventDefault();
      settingsStore.toggleSidebarCollapsed();
    }
  }

  // Welcome content for first-run experience
  const WELCOME_CONTENT = `# Welcome to Flashnotes

Your always-ready scratchpad. No files, no saving—everything persists automatically.

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| \`Cmd+P\` | Command palette |
| \`Cmd+N\` | New note |
| \`Cmd+W\` | Delete note |
| \`Cmd+B\` | Toggle sidebar |
| \`Cmd+Shift+↑/↓\` | Move note up/down |
| \`Cmd+E\` | Toggle markdown preview |
| \`Cmd+Shift+T\` | Toggle stay on top |
| \`Cmd+Shift+C\` | Copy note to clipboard |
| \`Cmd+,\` | Settings |
| \`Escape\` | Clear search |

## Tips

- **Search** is instant—just start typing in the sidebar
- **Drag** notes to reorder them, or use \`Cmd+Shift+↑/↓\`
- **Pin** important notes to keep them at the top
- **Vim mode** can be enabled in Settings
- Notes auto-save as you type
- Only one instance of Flashnotes can run at a time

Happy writing!
`;

  onMount(async () => {
    // Load settings first so fonts are ready
    await settingsStore.loadSettings();
    await bufferStore.loadSidebarData();

    if (bufferStore.sidebarBuffers.length === 0) {
      // First run - create welcome buffer
      await bufferStore.createBuffer(WELCOME_CONTENT);
    } else {
      await bufferStore.selectBuffer(bufferStore.sidebarBuffers[0].id);
    }

    // Safety net: save on blur and cleanup empty buffers
    try {
      const appWindow = getCurrentWindow();
      appWindow.onFocusChanged(async ({ payload: focused }) => {
        if (!focused) {
          if (bufferStore.isDirty) {
            await bufferStore.saveCurrentBuffer();
          }
          // Cleanup empty buffers when app loses focus
          await invoke('cleanup_empty_buffers');
          await bufferStore.loadSidebarData();
        }
      });
    } catch (error) {
      console.error('Failed to set up window focus listener:', error);
    }
  });
</script>

<svelte:window onkeydown={handleKeydown} />

<CommandPalette
  isOpen={isPaletteOpen}
  onclose={() => { isPaletteOpen = false; editorRef?.focus(); }}
  onselect={handlePaletteSelect}
/>

<SettingsModal
  isOpen={isSettingsOpen}
  onclose={() => { isSettingsOpen = false; editorRef?.focus(); }}
/>

<div class="h-screen w-screen flex font-mono text-sm antialiased bg-[--bg-app] text-[--text-main]">
  <Sidebar
    onSearch={handleSearch}
    onSelect={handleSelectBuffer}
    onCreate={handleCreateBuffer}
    onDelete={handleDeleteBuffer}
    onTogglePin={handleTogglePin}
    onReorder={handleReorder}
  />

  <main class="flex-1 flex flex-col bg-[--bg-editor]">
    <EditorHeader
      title={activeTitle}
      isDirty={bufferStore.isDirty}
      alwaysOnTop={settingsStore.settings.always_on_top}
      onToggleAlwaysOnTop={handleToggleAlwaysOnTop}
    />

    <div class="flex-1 overflow-hidden">
      {#if bufferStore.activeBufferId}
        {#if bufferStore.isLoading}
          <div class="flex items-center justify-center h-full text-[--text-muted]">
            Loading...
          </div>
        {:else}
          <Editor
            bind:this={editorRef}
            content={bufferStore.activeContent}
            onchange={handleEditorChange}
            previewMode={settingsStore.settings.preview_mode}
            vimMode={settingsStore.settings.vim_mode}
          />
        {/if}
      {:else}
        <div class="flex items-center justify-center h-full text-[--text-muted]">
          <div class="text-center">
            <p>No note selected</p>
            <button class="mt-2 text-[--accent] hover:underline" onclick={handleCreateBuffer}>
              Create a new note
            </button>
          </div>
        </div>
      {/if}
    </div>
  </main>
</div>

<!-- Toast notification -->
{#if toastMessage}
  <div
    class="fixed bottom-6 left-1/2 -translate-x-1/2 bg-[--bg-active] border border-[--border-subtle] text-[--text-main] px-4 py-2 rounded-lg shadow-lg text-sm font-medium z-50 animate-fade-in"
  >
    {toastMessage}
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
