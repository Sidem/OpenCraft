// The selected hotbar item's name, resting on the bar's top edge (the HUD appends `el` to the hotbar). It shows when
// the selection or the item in the selected slot changes, stays `HELD_NAME_MS` and fades; an empty slot shows nothing.

import './held-name.css';
import type { Game } from '../wasm/engine.js';

/** How long the name stays after the selection changes. */
const HELD_NAME_MS = 2500;

export class HeldName {
  readonly el = Object.assign(document.createElement('div'), { className: 'held-name' });
  private key = '';
  private shownAt = 0;

  constructor(private readonly game: Game) {}

  /** Call every frame with the frame's time in milliseconds. */
  update(now: number): void {
    const g = this.game;
    const slot = g.selected_slot(), item = g.slot_item(slot);
    const key = g.slot_count(slot) > 0 ? `${slot}:${item}` : `${slot}:`;
    if (key !== this.key) {
      this.key = key;
      this.shownAt = now;
      this.el.textContent = key.endsWith(':') ? '' : g.item_name(item);
    }
    this.el.classList.toggle('shown', this.el.textContent !== '' && now - this.shownAt < HELD_NAME_MS);
  }
}
