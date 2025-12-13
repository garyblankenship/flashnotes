<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import { listen, type UnlistenFn } from '@tauri-apps/api/event';
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
  let unlistenFocusEditor: UnlistenFn | null = null;
  let unlistenImportSublime: UnlistenFn | null = null;

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
    bufferStore.clearSearch();
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
      if (bufferStore.activeBufferId && confirm('Delete this buffer?')) {
        handleDeleteBuffer(bufferStore.activeBufferId);
      }
    } else if (e.key === ',' && e.metaKey) {
      e.preventDefault();
      isSettingsOpen = true;
    } else if (e.key === 'e' && e.metaKey && !e.shiftKey) {
      e.preventDefault();
      settingsStore.togglePreviewMode();
    } else if (e.key === 'Escape' && !isPaletteOpen && !isSettingsOpen) {
      bufferStore.clearSearch();
      editorRef?.focus();
    }
  }

  const welcomeContent = `# Welcome to Flashnotes

Your infinite-buffer scratchpad. No files, no saving—everything persists automatically.

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| \`Cmd+Shift+Space\` | Toggle window (global) |
| \`Cmd+P\` | Command palette |
| \`Cmd+N\` | New buffer |
| \`Cmd+W\` | Delete buffer |
| \`Cmd+E\` | Toggle markdown preview |
| \`Cmd+,\` | Settings |
| \`Escape\` | Clear search |

## Tips

- **Search** is instant—just start typing in the sidebar
- **Pin** important buffers to keep them at the top
- Buffers auto-save as you type (500ms debounce)
- This buffer will close when you create a new one, or delete it anytime

Happy writing!
`;

  onMount(async () => {
    // Load settings first so fonts are ready
    await settingsStore.loadSettings();
    await bufferStore.loadSidebarData();

    if (bufferStore.sidebarBuffers.length === 0) {
      // First run - create welcome buffer
      await bufferStore.createBuffer(welcomeContent);
    } else {
      await bufferStore.selectBuffer(bufferStore.sidebarBuffers[0].id);
    }

    // Safety net: save on blur
    try {
      const appWindow = getCurrentWindow();
      appWindow.onFocusChanged(({ payload: focused }) => {
        if (!focused && bufferStore.isDirty) {
          bufferStore.saveCurrentBuffer();
        }
      });
    } catch (error) {
      console.error('Failed to set up window focus listener:', error);
    }

    // Listen for focus-editor event from global shortcut
    unlistenFocusEditor = await listen('focus-editor', () => {
      setTimeout(() => editorRef?.focus(), 50);
    });

    // Listen for import-sublime event from menu
    unlistenImportSublime = await listen('import-sublime', async () => {
      try {
        const count = await invoke<number>('import_sublime_buffers');
        await bufferStore.loadSidebarData();
        if (count > 0 && bufferStore.sidebarBuffers.length > 0) {
          await bufferStore.selectBuffer(bufferStore.sidebarBuffers[0].id);
        }
        alert(count > 0 ? `Imported ${count} buffer${count === 1 ? '' : 's'} from Sublime Text` : 'No unsaved buffers found in Sublime Text');
      } catch (error) {
        console.error('Failed to import from Sublime:', error);
        alert(`Import failed: ${error}`);
      }
    });
  });

  onDestroy(() => {
    unlistenFocusEditor?.();
    unlistenImportSublime?.();
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
    buffers={bufferStore.sidebarBuffers}
    searchResults={bufferStore.searchResults}
    searchQuery={bufferStore.searchQuery}
    activeBufferId={bufferStore.activeBufferId}
    bufferCount={bufferStore.bufferCount}
    onSearch={handleSearch}
    onSelect={handleSelectBuffer}
    onCreate={handleCreateBuffer}
    onDelete={handleDeleteBuffer}
    onTogglePin={handleTogglePin}
  />

  <main class="flex-1 flex flex-col bg-[--bg-editor]">
    <EditorHeader title={activeTitle} isDirty={bufferStore.isDirty} />

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
          />
        {/if}
      {:else}
        <div class="flex items-center justify-center h-full text-[--text-muted]">
          <div class="text-center">
            <p>No buffer selected</p>
            <button class="mt-2 text-[--accent] hover:underline" onclick={handleCreateBuffer}>
              Create a new buffer
            </button>
          </div>
        </div>
      {/if}
    </div>
  </main>
</div>
