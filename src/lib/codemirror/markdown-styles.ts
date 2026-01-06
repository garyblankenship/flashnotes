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

  // Tables (GFM) - raw markdown when editing
  '.cm-md-table': {
    fontFamily: 'var(--font-mono)',
    fontSize: '0.9em',
  },

  // Rendered table widget
  '.cm-md-table-widget': {
    display: 'block',
    margin: '8px 0',
    overflow: 'auto',
  },
  '.cm-md-table-rendered': {
    borderCollapse: 'collapse',
    width: '100%',
    fontSize: '0.9em',
  },
  '.cm-md-table-rendered th': {
    backgroundColor: 'var(--bg-active)',
    fontWeight: '600',
    textAlign: 'left',
    padding: '8px 12px',
    borderBottom: '2px solid var(--border-subtle)',
  },
  '.cm-md-table-rendered td': {
    padding: '6px 12px',
    borderBottom: '1px solid var(--border-subtle)',
  },
  '.cm-md-table-rendered tr:hover td': {
    backgroundColor: 'var(--bg-hover)',
  },
}, { dark: true });
