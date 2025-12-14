<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { EditorView, keymap, placeholder, lineNumbers, highlightActiveLine, drawSelection, dropCursor } from '@codemirror/view';
  import { EditorState, Compartment } from '@codemirror/state';
  import { defaultKeymap, history, historyKeymap, indentWithTab } from '@codemirror/commands';
  import { bracketMatching, indentOnInput } from '@codemirror/language';
  import { closeBrackets, closeBracketsKeymap } from '@codemirror/autocomplete';
  import { searchKeymap, highlightSelectionMatches } from '@codemirror/search';
  import { markdown } from '@codemirror/lang-markdown';
  import { languages } from '@codemirror/language-data';
  import { GFM } from '@lezer/markdown';
  import { vim } from '@replit/codemirror-vim';
  import { markdownPreviewPlugin } from '$lib/codemirror/markdown-preview';
  import { markdownPreviewTheme } from '$lib/codemirror/markdown-styles';
  import { nordExtensions } from '$lib/codemirror/themes/nord';

  interface Props {
    content: string;
    onchange: (content: string) => void;
    previewMode?: boolean;
    vimMode?: boolean;
  }

  let { content, onchange, previewMode = false, vimMode = false }: Props = $props();

  let container: HTMLDivElement;
  let view: EditorView | null = $state(null);
  let isUpdatingFromProp = false;

  const readOnlyCompartment = new Compartment();
  const previewCompartment = new Compartment();
  const vimCompartment = new Compartment();

  onMount(() => {
    const extensions = [
      // Vim mode must be first for proper key handling
      vimCompartment.of(vimMode ? vim() : []),
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
      ...nordExtensions,
      EditorView.lineWrapping,
      EditorView.updateListener.of((update) => {
        if (update.docChanged && !isUpdatingFromProp) {
          const newContent = update.state.doc.toString();
          onchange(newContent);
        }
      }),
      // Markdown language support with GFM and code block highlighting
      markdown({
        extensions: GFM,
        codeLanguages: languages,
      }),
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

  // Toggle vim mode when prop changes
  $effect(() => {
    if (view) {
      view.dispatch({
        effects: vimCompartment.reconfigure(vimMode ? vim() : []),
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
