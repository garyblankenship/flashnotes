<script lang="ts">
  import { onMount } from 'svelte';
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import { invoke } from '@tauri-apps/api/core';
  import Editor from '$lib/components/Editor.svelte';
  import Sidebar from '$lib/components/Sidebar.svelte';
  import EditorHeader from '$lib/components/EditorHeader.svelte';
  import CommandPalette from '$lib/components/CommandPalette.svelte';
  import SettingsModal from '$lib/components/SettingsModal.svelte';
  import Toast from '$lib/components/Toast.svelte';
  import { bufferStore } from '$lib/stores/buffers.svelte';
  import { settingsStore } from '$lib/stores/settings.svelte';
  import { toastStore } from '$lib/stores/toast.svelte';
  import { debounce } from '$lib/utils/debounce';
  import { createKeyboardHandler } from '$lib/hooks/useKeyboardShortcuts.svelte';
  import { WELCOME_CONTENT } from '$lib/constants/welcome';

  let editorRef: Editor | null = $state(null);
  let isPaletteOpen = $state(false);
  let isSettingsOpen = $state(false);

  // Clipboard and always-on-top handlers
  async function handleCopyToClipboard() {
    const content = editorRef?.getContent() ?? bufferStore.activeContent;
    if (content) {
      try {
        await navigator.clipboard.writeText(content);
        toastStore.show('Copied to clipboard');
      } catch {
        toastStore.show('Failed to copy');
      }
    }
  }

  async function handleToggleAlwaysOnTop() {
    try {
      const newState = await invoke<boolean>('toggle_always_on_top');
      await settingsStore.loadSettings();
      toastStore.show(newState ? 'Stay on Top: On' : 'Stay on Top: Off');
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

  // Keyboard shortcuts via extracted hook
  const handleKeydown = createKeyboardHandler(
    {
      onOpenPalette: () => { isPaletteOpen = true; },
      onCreateBuffer: handleCreateBuffer,
      onDeleteBuffer: () => {
        if (bufferStore.activeBufferId) {
          handleDeleteBuffer(bufferStore.activeBufferId);
        }
      },
      onOpenSettings: () => { isSettingsOpen = true; },
      onTogglePreview: () => settingsStore.togglePreviewMode(),
      onMoveBufferUp: () => bufferStore.moveBufferUp(),
      onMoveBufferDown: () => bufferStore.moveBufferDown(),
      onClearSearch: () => {
        bufferStore.clearSearch();
        editorRef?.focus();
      },
      onToggleAlwaysOnTop: handleToggleAlwaysOnTop,
      onCopyToClipboard: handleCopyToClipboard,
      onToggleSidebar: () => settingsStore.toggleSidebarCollapsed(),
    },
    () => ({ isPaletteOpen, isSettingsOpen })
  );

  onMount(async () => {
    await settingsStore.loadSettings();
    await bufferStore.loadSidebarData();

    if (bufferStore.sidebarBuffers.length === 0) {
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
      onCopy={handleCopyToClipboard}
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

<Toast />
