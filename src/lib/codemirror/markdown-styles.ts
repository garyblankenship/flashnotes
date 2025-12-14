import { EditorView } from '@codemirror/view';

export const markdownPreviewTheme = EditorView.theme({
  // Headers - scaled from base font
  '.cm-md-h1': {
    fontSize: '1.75em',
    fontWeight: '700',
    lineHeight: '1.3',
  },
  '.cm-md-h2': {
    fontSize: '1.5em',
    fontWeight: '600',
    lineHeight: '1.35',
  },
  '.cm-md-h3': {
    fontSize: '1.25em',
    fontWeight: '600',
  },
  '.cm-md-h4': {
    fontSize: '1.1em',
    fontWeight: '600',
  },
  '.cm-md-h5': {
    fontSize: '1.05em',
    fontWeight: '600',
  },
  '.cm-md-h6': {
    fontSize: '1em',
    fontWeight: '600',
    color: 'var(--text-muted)',
  },

  // Text styling
  '.cm-md-bold': {
    fontWeight: '700',
  },
  '.cm-md-italic': {
    fontStyle: 'italic',
  },
  '.cm-md-strikethrough': {
    textDecoration: 'line-through',
    color: 'var(--text-muted)',
  },

  // Inline code
  '.cm-md-inline-code': {
    backgroundColor: 'var(--bg-active)',
    padding: '2px 6px',
    borderRadius: '4px',
    fontSize: '0.9em',
  },

  // Code blocks
  '.cm-md-code-block': {
    backgroundColor: 'var(--bg-app)',
    borderRadius: '6px',
  },

  // Links
  '.cm-md-link': {
    color: 'var(--accent)',
    textDecoration: 'underline',
    textUnderlineOffset: '2px',
  },
  '.cm-md-link-url': {
    color: 'var(--text-muted)',
    fontSize: '0.85em',
  },

  // Blockquotes
  '.cm-md-blockquote': {
    borderLeft: '3px solid var(--accent)',
    paddingLeft: '12px',
    color: 'var(--text-muted)',
    fontStyle: 'italic',
  },

  // Lists
  '.cm-md-list-marker': {
    color: 'var(--accent)',
  },

  // Horizontal rule
  '.cm-md-hr': {
    borderTop: '1px solid var(--border-subtle)',
    display: 'block',
    margin: '8px 0',
  },

  // Task lists
  '.cm-md-task-checked': {
    textDecoration: 'line-through',
    color: 'var(--text-muted)',
  },

  // Tables (GFM)
  '.cm-md-table': {
    fontFamily: 'inherit',
  },
  '.cm-md-table-header': {
    fontWeight: '600',
  },
}, { dark: true });
