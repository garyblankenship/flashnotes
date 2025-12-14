<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { EditorView, keymap, placeholder, lineNumbers, highlightActiveLine, drawSelection, dropCursor } from '@codemirror/view';
  import { EditorState, Compartment } from '@codemirror/state';
  import { defaultKeymap, history, historyKeymap, indentWithTab } from '@codemirror/commands';
  import { bracketMatching, indentOnInput, syntaxHighlighting, HighlightStyle } from '@codemirror/language';
  import { closeBrackets, closeBracketsKeymap } from '@codemirror/autocomplete';
  import { searchKeymap, highlightSelectionMatches } from '@codemirror/search';
  import { markdown } from '@codemirror/lang-markdown';
  import { languages } from '@codemirror/language-data';
  import { GFM } from '@lezer/markdown';
  import { tags } from '@lezer/highlight';
  import { vim } from '@replit/codemirror-vim';
  import { markdownPreviewPlugin } from '$lib/codemirror/markdown-preview';
  import { markdownPreviewTheme } from '$lib/codemirror/markdown-styles';

  // Nord-inspired syntax highlighting
  const nordHighlightStyle = HighlightStyle.define([
    { tag: tags.keyword, color: '#81A1C1' },
    { tag: tags.operator, color: '#81A1C1' },
    { tag: tags.special(tags.variableName), color: '#88C0D0' },
    { tag: tags.typeName, color: '#8FBCBB' },
    { tag: tags.atom, color: '#D08770' },
    { tag: tags.number, color: '#B48EAD' },
    { tag: tags.bool, color: '#D08770' },
    { tag: tags.string, color: '#A3BE8C' },
    { tag: tags.regexp, color: '#EBCB8B' },
    { tag: tags.escape, color: '#D08770' },
    { tag: tags.definition(tags.variableName), color: '#88C0D0' },
    { tag: tags.function(tags.variableName), color: '#88C0D0' },
    { tag: tags.labelName, color: '#81A1C1' },
    { tag: tags.comment, color: '#616E88', fontStyle: 'italic' },
    { tag: tags.meta, color: '#5E81AC' },
    { tag: tags.invalid, color: '#BF616A' },
    { tag: tags.punctuation, color: '#ECEFF4' },
    { tag: tags.heading, color: '#88C0D0', fontWeight: 'bold' },
    { tag: tags.heading1, color: '#88C0D0', fontWeight: 'bold', fontSize: '1.4em' },
    { tag: tags.heading2, color: '#88C0D0', fontWeight: 'bold', fontSize: '1.2em' },
    { tag: tags.heading3, color: '#88C0D0', fontWeight: 'bold', fontSize: '1.1em' },
    { tag: tags.link, color: '#81A1C1', textDecoration: 'underline' },
    { tag: tags.url, color: '#5E81AC' },
    { tag: tags.emphasis, fontStyle: 'italic', color: '#EBCB8B' },
    { tag: tags.strong, fontWeight: 'bold', color: '#D8DEE9' },
    { tag: tags.strikethrough, textDecoration: 'line-through' },
    { tag: tags.content, color: '#D8DEE9' },
    { tag: tags.monospace, color: '#A3BE8C', fontFamily: 'inherit' },
    { tag: tags.className, color: '#8FBCBB' },
    { tag: tags.propertyName, color: '#88C0D0' },
    { tag: tags.variableName, color: '#D8DEE9' },
    { tag: tags.attributeName, color: '#8FBCBB' },
    { tag: tags.attributeValue, color: '#A3BE8C' },
  ]);

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
    // Vim mode styles
    '.cm-vim-panel': {
      backgroundColor: 'var(--bg-active)',
      color: 'var(--text-main)',
      padding: '2px 8px',
      fontFamily: "var(--editor-font-family, 'JetBrains Mono', monospace)",
      fontSize: '12px',
    },
    '.cm-vim-panel input': {
      backgroundColor: 'transparent',
      color: 'var(--text-main)',
      border: 'none',
      outline: 'none',
      fontFamily: "var(--editor-font-family, 'JetBrains Mono', monospace)",
    },
    '.cm-fat-cursor': {
      background: 'var(--accent) !important',
    },
    '&:not(.cm-focused) .cm-fat-cursor': {
      background: 'none !important',
      outline: '1px solid var(--accent) !important',
    },
  }, { dark: true });

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
      nordTheme,
      syntaxHighlighting(nordHighlightStyle),
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
