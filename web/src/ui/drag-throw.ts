// Dragging a stack out of the inventory screen: a plain left press on a filled slot with an empty hand picks the stack
// up (the screen's own click), and letting go over the dimmed backdrop, outside the panel, after moving the pointer,
// throws the whole stack (`Action::ThrowCursorAll`, queued after the pick-up so the engine sees them in order).

import type { Game } from '../wasm/engine.js';

/** A press that moved less than this before letting go outside the panel was a click, not a throw. */
const DRAG_PX = 8;

export class DragThrow {
  /** Where the stack was picked up, until the button comes up. */
  private from: [number, number] | null = null;

  constructor(backdrop: HTMLElement, game: Game, isOpen: () => boolean) {
    window.addEventListener('pointerup', (e) => {
      const from = this.from;
      this.from = null;
      if (!from || !isOpen() || e.button !== 0 || e.target !== backdrop) return;
      if (Math.hypot(e.clientX - from[0], e.clientY - from[1]) > DRAG_PX) game.throw_cursor_all();
    });
  }

  /** A press on a slot: `filled` says the slot holds a stack and the hand is empty. Only a plain left press may drag. */
  press(e: PointerEvent, filled: boolean): void {
    const plain = e.button === 0 && !e.shiftKey && !e.ctrlKey && !e.metaKey;
    this.from = plain && filled ? [e.clientX, e.clientY] : null;
  }
}
