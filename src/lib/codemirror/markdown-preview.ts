import { ViewPlugin, Decoration, type DecorationSet, EditorView, type ViewUpdate, WidgetType } from '@codemirror/view';
import { syntaxTree } from '@codemirror/language';
import { RangeSetBuilder } from '@codemirror/state';

// Widget to render a table as HTML
class TableWidget extends WidgetType {
  constructor(private tableText: string) {
    super();
  }

  eq(other: TableWidget) {
    return this.tableText === other.tableText;
  }

  toDOM() {
    const wrapper = document.createElement('div');
    wrapper.className = 'cm-md-table-widget';

    const lines = this.tableText.split('\n').filter(line => line.trim());
    if (lines.length < 2) {
      wrapper.textContent = this.tableText;
      return wrapper;
    }

    const table = document.createElement('table');
    table.className = 'cm-md-table-rendered';

    // Parse header
    const headerCells = this.parseCells(lines[0]);
    const thead = document.createElement('thead');
    const headerRow = document.createElement('tr');
    for (const cell of headerCells) {
      const th = document.createElement('th');
      th.textContent = cell.trim();
      headerRow.appendChild(th);
    }
    thead.appendChild(headerRow);
    table.appendChild(thead);

    // Skip separator line (lines[1]), parse body
    const tbody = document.createElement('tbody');
    for (let i = 2; i < lines.length; i++) {
      const cells = this.parseCells(lines[i]);
      const tr = document.createElement('tr');
      for (let j = 0; j < headerCells.length; j++) {
        const td = document.createElement('td');
        td.textContent = (cells[j] || '').trim();
        tr.appendChild(td);
      }
      tbody.appendChild(tr);
    }
    table.appendChild(tbody);
    wrapper.appendChild(table);

    return wrapper;
  }

  private parseCells(line: string): string[] {
    // Remove leading/trailing pipes and split
    const trimmed = line.replace(/^\||\|$/g, '');
    return trimmed.split('|');
  }

  ignoreEvent() {
    return false;
  }
}

// Map heading levels to CSS classes
const headingClasses: Record<string, string> = {
  ATXHeading1: 'cm-md-h1',
  ATXHeading2: 'cm-md-h2',
  ATXHeading3: 'cm-md-h3',
  ATXHeading4: 'cm-md-h4',
  ATXHeading5: 'cm-md-h5',
  ATXHeading6: 'cm-md-h6',
  SetextHeading1: 'cm-md-h1',
  SetextHeading2: 'cm-md-h2',
};

interface DecorationSpec {
  from: number;
  to: number;
  decoration: Decoration;
}

// Create the ViewPlugin for markdown preview decorations
export const markdownPreviewPlugin = ViewPlugin.fromClass(
  class {
    decorations: DecorationSet;

    constructor(view: EditorView) {
      this.decorations = this.buildDecorations(view);
    }

    update(update: ViewUpdate) {
      if (update.docChanged || update.viewportChanged || update.selectionSet) {
        this.decorations = this.buildDecorations(update.view);
      }
    }

    buildDecorations(view: EditorView): DecorationSet {
      const specs: DecorationSpec[] = [];
      const cursorPos = view.state.selection.main.head;

      // Process only visible ranges for performance
      for (const { from, to } of view.visibleRanges) {
        syntaxTree(view.state).iterate({
          from,
          to,
          enter: (node) => {
            // Skip decorations on the line where cursor is (allows editing)
            const line = view.state.doc.lineAt(node.from);
            const cursorLine = view.state.doc.lineAt(cursorPos);
            const isActiveLine = line.number === cursorLine.number;

            // Headers
            if (headingClasses[node.name]) {
              if (!isActiveLine) {
                // Hide the # marks
                const text = view.state.doc.sliceString(node.from, node.to);
                const match = text.match(/^(#{1,6})\s/);
                if (match) {
                  specs.push({
                    from: node.from,
                    to: node.from + match[0].length,
                    decoration: Decoration.replace({}),
                  });
                }
              }
              // Apply heading style to the whole line
              specs.push({
                from: node.from,
                to: node.to,
                decoration: Decoration.mark({ class: headingClasses[node.name] }),
              });
            }

            // Bold - StrongEmphasis
            if (node.name === 'StrongEmphasis') {
              if (!isActiveLine) {
                // Hide ** or __ markers
                specs.push({
                  from: node.from,
                  to: node.from + 2,
                  decoration: Decoration.replace({}),
                });
                specs.push({
                  from: node.to - 2,
                  to: node.to,
                  decoration: Decoration.replace({}),
                });
              }
              specs.push({
                from: node.from,
                to: node.to,
                decoration: Decoration.mark({ class: 'cm-md-bold' }),
              });
            }

            // Italic - Emphasis
            if (node.name === 'Emphasis') {
              if (!isActiveLine) {
                // Hide * or _ markers
                specs.push({
                  from: node.from,
                  to: node.from + 1,
                  decoration: Decoration.replace({}),
                });
                specs.push({
                  from: node.to - 1,
                  to: node.to,
                  decoration: Decoration.replace({}),
                });
              }
              specs.push({
                from: node.from,
                to: node.to,
                decoration: Decoration.mark({ class: 'cm-md-italic' }),
              });
            }

            // Strikethrough
            if (node.name === 'Strikethrough') {
              if (!isActiveLine) {
                specs.push({
                  from: node.from,
                  to: node.from + 2,
                  decoration: Decoration.replace({}),
                });
                specs.push({
                  from: node.to - 2,
                  to: node.to,
                  decoration: Decoration.replace({}),
                });
              }
              specs.push({
                from: node.from,
                to: node.to,
                decoration: Decoration.mark({ class: 'cm-md-strikethrough' }),
              });
            }

            // Inline code
            if (node.name === 'InlineCode') {
              if (!isActiveLine) {
                // Hide backticks
                specs.push({
                  from: node.from,
                  to: node.from + 1,
                  decoration: Decoration.replace({}),
                });
                specs.push({
                  from: node.to - 1,
                  to: node.to,
                  decoration: Decoration.replace({}),
                });
              }
              specs.push({
                from: node.from,
                to: node.to,
                decoration: Decoration.mark({ class: 'cm-md-inline-code' }),
              });
            }

            // Code blocks
            if (node.name === 'FencedCode' || node.name === 'CodeBlock') {
              specs.push({
                from: node.from,
                to: node.to,
                decoration: Decoration.mark({ class: 'cm-md-code-block' }),
              });
            }

            // Blockquotes
            if (node.name === 'Blockquote') {
              specs.push({
                from: node.from,
                to: node.to,
                decoration: Decoration.mark({ class: 'cm-md-blockquote' }),
              });
            }

            // Links - style the visible text
            if (node.name === 'Link') {
              specs.push({
                from: node.from,
                to: node.to,
                decoration: Decoration.mark({ class: 'cm-md-link' }),
              });
            }

            // Horizontal rules
            if (node.name === 'HorizontalRule') {
              specs.push({
                from: node.from,
                to: node.to,
                decoration: Decoration.mark({ class: 'cm-md-hr' }),
              });
            }

            // List markers
            if (node.name === 'ListMark') {
              specs.push({
                from: node.from,
                to: node.to,
                decoration: Decoration.mark({ class: 'cm-md-list-marker' }),
              });
            }

            // Tables (GFM) - render as HTML table widget when not editing
            if (node.name === 'Table') {
              const tableStart = view.state.doc.lineAt(node.from);
              const tableEnd = view.state.doc.lineAt(node.to);
              const cursorLine = view.state.doc.lineAt(cursorPos);
              const isCursorInTable = cursorLine.number >= tableStart.number && cursorLine.number <= tableEnd.number;

              if (!isCursorInTable) {
                // Replace entire table with rendered widget
                const tableText = view.state.doc.sliceString(node.from, node.to);
                specs.push({
                  from: node.from,
                  to: node.to,
                  decoration: Decoration.replace({
                    widget: new TableWidget(tableText),
                  }),
                });
              } else {
                // Just apply basic styling when editing
                specs.push({
                  from: node.from,
                  to: node.to,
                  decoration: Decoration.mark({ class: 'cm-md-table' }),
                });
              }
            }
          },
        });
      }

      // Sort decorations by from position, then by to position (for stability)
      specs.sort((a, b) => a.from - b.from || a.to - b.to);

      // Build the decoration set
      const builder = new RangeSetBuilder<Decoration>();
      for (const spec of specs) {
        builder.add(spec.from, spec.to, spec.decoration);
      }

      return builder.finish();
    }
  },
  {
    decorations: (v) => v.decorations,
  }
);
