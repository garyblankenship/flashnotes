<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { EditorView, keymap, placeholder, lineNumbers, highlightActiveLine, drawSelection, dropCursor } from '@codemirror/view';
  import { EditorState, Compartment } from '@codemirror/state';
  import { defaultKeymap, history, historyKeymap, indentWithTab } from '@codemirror/commands';
  import { bracketMatching, indentOnInput } from '@codemirror/language';
  import { closeBrackets, closeBracketsKeymap } from '@codemirror/autocomplete';
  import { searchKeymap, highlightSelectionMatches } from '@codemirror/search';
  import { markdown } from '@codemirror/lang-markdown';
  import { GFM } from '@lezer/markdown';
  import { markdownPreviewPlugin } from '$lib/codemirror/markdown-preview';
  import { markdownPreviewTheme } from '$lib/codemirror/markdown-styles';

  interface Props {
    content: string;
    onchange: (content: string) => void;
    previewMode?: boolean;
  }

  let { content, onchange, previewMode = false }: Props = $props();

  let container: HTMLDivElement;
  let view: EditorView | null = $state(null);
  let isUpdatingFromProp = false;

  // Nord theme with CSS variable support for dynamic font settings
  const nordTheme = EditorView.theme({
    '&': {
      height: '100%',
      backgroundColor: 'var(--bg-editor)',
      color: 'var(--text-main)',
      fontSize: 'var(--editor-font-size, 13px)',
    },
    '.cm-content': {
      fontFamily: "var(--editor-font-family, 'JetBrains Mono', monospace)",
      padding: '16px 0',
      caretColor: 'var(--accent)',
      lineHeight: 'var(--editor-line-height, 1.5)',
    },
    '.cm-cursor, .cm-dropCursor': {
      borderLeftColor: 'var(--accent)',
      borderLeftWidth: '2px',
    },
    '.cm-scroller': {
      overflow: 'auto',
      fontFamily: "var(--editor-font-family, 'JetBrains Mono', monospace)",
    },
    '.cm-gutters': {
      backgroundColor: 'transparent',
      borderRight: 'none',
      color: 'var(--text-muted)',
      paddingRight: '8px',
    },
    '.cm-gutter': {
      minWidth: '40px',
    },
    '.cm-lineNumbers .cm-gutterElement': {
      padding: '0 8px 0 16px',
      minWidth: '32px',
      textAlign: 'right',
    },
    '.cm-activeLine': {
      backgroundColor: 'rgba(67, 76, 94, 0.5)',
    },
    '.cm-activeLineGutter': {
      backgroundColor: 'transparent',
      color: 'var(--text-main)',
    },
    '.cm-selectionBackground, ::selection': {
      backgroundColor: 'rgba(136, 192, 208, 0.3) !important',
    },
    '.cm-focused .cm-selectionBackground': {
      backgroundColor: 'rgba(136, 192, 208, 0.3)',
    },
    '.cm-matchingBracket': {
      backgroundColor: 'rgba(129, 161, 193, 0.3)',
      outline: 'none',
    },
    '.cm-placeholder': {
      color: 'var(--text-muted)',
      fontStyle: 'italic',
    },
    '.cm-line': {
      padding: '0 16px',
    },
    '.cm-foldPlaceholder': {
      backgroundColor: 'var(--bg-active)',
      border: 'none',
      color: 'var(--text-muted)',
    },
    '.cm-tooltip': {
      backgroundColor: 'var(--bg-active)',
      border: '1px solid var(--border-subtle)',
      borderRadius: '4px',
    },
    '&.cm-focused': {
      outline: 'none',
    },
  }, { dark: true });

  const readOnlyCompartment = new Compartment();
  const previewCompartment = new Compartment();

  onMount(() => {
    const extensions = [
      history(),
      drawSelection(),
      dropCursor(),
      indentOnInput(),
      bracketMatching(),
      closeBrackets(),
      highlightActiveLine(),
      highlightSelectionMatches(),
      lineNumbers(),
      placeholder('Start typing...'),
      keymap.of([
        ...closeBracketsKeymap,
        ...defaultKeymap,
        ...searchKeymap,
        ...historyKeymap,
        indentWithTab,
      ]),
      nordTheme,
      EditorView.lineWrapping,
      EditorView.updateListener.of((update) => {
        if (update.docChanged && !isUpdatingFromProp) {
          const newContent = update.state.doc.toString();
          onchange(newContent);
        }
      }),
      // Markdown language support with GFM (tables, strikethrough, etc.)
      markdown({ extensions: GFM }),
      // Preview mode extensions (toggleable)
      previewCompartment.of(previewMode ? [markdownPreviewPlugin, markdownPreviewTheme] : []),
      readOnlyCompartment.of([]),
    ];

    const state = EditorState.create({
      doc: content,
      extensions,
    });

    view = new EditorView({
      state,
      parent: container,
    });

    // Focus the editor
    view.focus();
  });

  // Update editor when content prop changes (buffer switch)
  $effect(() => {
    if (view && content !== view.state.doc.toString()) {
      isUpdatingFromProp = true;
      view.dispatch({
        changes: {
          from: 0,
          to: view.state.doc.length,
          insert: content,
        },
      });
      isUpdatingFromProp = false;
    }
  });

  // Toggle preview mode when prop changes
  $effect(() => {
    if (view) {
      view.dispatch({
        effects: previewCompartment.reconfigure(
          previewMode ? [markdownPreviewPlugin, markdownPreviewTheme] : []
        ),
      });
    }
  });

  onDestroy(() => {
    view?.destroy();
  });

  export function focus() {
    view?.focus();
  }

  export function getContent(): string {
    return view?.state.doc.toString() ?? '';
  }
</script>

<div bind:this={container} class="h-full w-full"></div>
