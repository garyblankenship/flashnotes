/**
 * Drag-to-reorder hook for list items
 * Uses mouse events instead of HTML5 drag API for better control
 */

export interface DragItem {
  id: string;
  is_pinned?: boolean;
}

export interface DragState {
  draggedId: string | null;
  dropTargetId: string | null;
  insertPosition: 'before' | 'after' | null;
  isDragging: boolean;
}

export interface UseDragToReorderOptions<T extends DragItem> {
  getItems: () => T[];
  onReorder: (ids: string[]) => void;
  onSelect: (id: string) => void;
  isPinned?: (item: T) => boolean;
}

/**
 * Creates drag-to-reorder state and handlers
 */
export function createDragToReorder<T extends DragItem>(
  options: UseDragToReorderOptions<T>
) {
  let draggedId = $state<string | null>(null);
  let dropTargetId = $state<string | null>(null);
  let insertPosition = $state<'before' | 'after' | null>(null);
  let isDragging = $state(false);
  let mouseDownTime = $state(0);

  const isPinned = options.isPinned ?? ((item: T) => item.is_pinned ?? false);

  function resetDragState() {
    dropTargetId = null;
    draggedId = null;
    insertPosition = null;
    isDragging = false;
    mouseDownTime = 0;
  }

  function handleMouseDown(e: MouseEvent, id: string, item: T) {
    if (e.button !== 0) return;

    mouseDownTime = Date.now();

    // Pinned items can be selected but not dragged
    if (isPinned(item)) {
      const handlePinnedMouseUp = () => {
        document.removeEventListener('mouseup', handlePinnedMouseUp);
        if (Date.now() - mouseDownTime < 300) {
          options.onSelect(id);
        }
        mouseDownTime = 0;
      };
      document.addEventListener('mouseup', handlePinnedMouseUp);
      return;
    }

    draggedId = id;

    const handleMouseMove = (moveEvent: MouseEvent) => {
      // Only start visual drag after 100ms
      if (!isDragging && (Date.now() - mouseDownTime > 100)) {
        isDragging = true;
      }

      if (!isDragging) return;

      // Find element under cursor
      const elements = document.elementsFromPoint(moveEvent.clientX, moveEvent.clientY);
      const bufferEl = elements.find(el => el.getAttribute('data-buffer-id'));

      if (bufferEl) {
        const targetId = bufferEl.getAttribute('data-buffer-id')!;
        const items = options.getItems();
        const targetItem = items.find(b => b.id === targetId);

        if (targetId !== draggedId && targetItem && !isPinned(targetItem)) {
          dropTargetId = targetId;
          const rect = bufferEl.getBoundingClientRect();
          const midY = rect.top + rect.height / 2;
          insertPosition = moveEvent.clientY < midY ? 'before' : 'after';
        }
      } else {
        dropTargetId = null;
      }
    };

    const handleMouseUp = () => {
      document.removeEventListener('mousemove', handleMouseMove);
      document.removeEventListener('mouseup', handleMouseUp);

      // If it was a quick click (not a drag), select the buffer
      if (!isDragging || Date.now() - mouseDownTime < 150) {
        options.onSelect(id);
        resetDragState();
        return;
      }

      // Complete the reorder
      if (dropTargetId && draggedId && dropTargetId !== draggedId) {
        const items = options.getItems();
        const fromIndex = items.findIndex(b => b.id === draggedId);
        const toIndex = items.findIndex(b => b.id === dropTargetId);

        if (fromIndex !== -1 && toIndex !== -1) {
          const ids = items.map(b => b.id);
          ids.splice(fromIndex, 1);

          let insertAt = toIndex;
          if (fromIndex < toIndex) {
            insertAt = insertPosition === 'after' ? toIndex : toIndex - 1;
          } else {
            insertAt = insertPosition === 'after' ? toIndex + 1 : toIndex;
          }

          ids.splice(insertAt, 0, draggedId);
          options.onReorder(ids);
        }
      }

      resetDragState();
    };

    document.addEventListener('mousemove', handleMouseMove);
    document.addEventListener('mouseup', handleMouseUp);
  }

  return {
    get draggedId() { return draggedId; },
    get dropTargetId() { return dropTargetId; },
    get insertPosition() { return insertPosition; },
    get isDragging() { return isDragging; },
    handleMouseDown,
    resetDragState,
  };
}
