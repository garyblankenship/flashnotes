import { ViewPlugin, Decoration, type DecorationSet, EditorView, type ViewUpdate } from '@codemirror/view';
import { syntaxTree } from '@codemirror/language';
import { RangeSetBuilder } from '@codemirror/state';

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
      const builder = new RangeSetBuilder<Decoration>();
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
                  builder.add(
                    node.from,
                    node.from + match[0].length,
                    Decoration.replace({})
                  );
                }
              }
              // Apply heading style to the whole line
              builder.add(
                node.from,
                node.to,
                Decoration.mark({ class: headingClasses[node.name] })
              );
            }

            // Bold - StrongEmphasis
            if (node.name === 'StrongEmphasis') {
              if (!isActiveLine) {
                // Hide ** or __ markers
                builder.add(node.from, node.from + 2, Decoration.replace({}));
                builder.add(node.to - 2, node.to, Decoration.replace({}));
              }
              builder.add(
                node.from,
                node.to,
                Decoration.mark({ class: 'cm-md-bold' })
              );
            }

            // Italic - Emphasis
            if (node.name === 'Emphasis') {
              if (!isActiveLine) {
                // Hide * or _ markers
                builder.add(node.from, node.from + 1, Decoration.replace({}));
                builder.add(node.to - 1, node.to, Decoration.replace({}));
              }
              builder.add(
                node.from,
                node.to,
                Decoration.mark({ class: 'cm-md-italic' })
              );
            }

            // Strikethrough
            if (node.name === 'Strikethrough') {
              if (!isActiveLine) {
                builder.add(node.from, node.from + 2, Decoration.replace({}));
                builder.add(node.to - 2, node.to, Decoration.replace({}));
              }
              builder.add(
                node.from,
                node.to,
                Decoration.mark({ class: 'cm-md-strikethrough' })
              );
            }

            // Inline code
            if (node.name === 'InlineCode') {
              if (!isActiveLine) {
                // Hide backticks
                builder.add(node.from, node.from + 1, Decoration.replace({}));
                builder.add(node.to - 1, node.to, Decoration.replace({}));
              }
              builder.add(
                node.from,
                node.to,
                Decoration.mark({ class: 'cm-md-inline-code' })
              );
            }

            // Code blocks
            if (node.name === 'FencedCode' || node.name === 'CodeBlock') {
              builder.add(
                node.from,
                node.to,
                Decoration.mark({ class: 'cm-md-code-block' })
              );
            }

            // Blockquotes
            if (node.name === 'Blockquote') {
              builder.add(
                node.from,
                node.to,
                Decoration.mark({ class: 'cm-md-blockquote' })
              );
            }

            // Links - style the visible text
            if (node.name === 'Link') {
              builder.add(
                node.from,
                node.to,
                Decoration.mark({ class: 'cm-md-link' })
              );
            }

            // Horizontal rules
            if (node.name === 'HorizontalRule') {
              builder.add(
                node.from,
                node.to,
                Decoration.mark({ class: 'cm-md-hr' })
              );
            }

            // List markers
            if (node.name === 'ListMark') {
              builder.add(
                node.from,
                node.to,
                Decoration.mark({ class: 'cm-md-list-marker' })
              );
            }
          },
        });
      }

      return builder.finish();
    }
  },
  {
    decorations: (v) => v.decorations,
  }
);
