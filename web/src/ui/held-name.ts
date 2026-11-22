// The selected hotbar item's name, resting on the bar's top edge (the HUD appends `el` to the hotbar). It shows when
// the selection or the item in the selected slot changes, stays `HELD_NAME_MS` and fades; an empty slot shows nothing.

import './held-name.css';
import type { Game } from '../wasm/engine.js';

/** How long the name stays after the selection changes. */
const HELD_NAME_MS = 2500;

export class HeldName {
  readonly el = Object.assign(document.createElement('div'), { className: 'held-name' });
  private readonly swatch = Object.assign(document.createElement('span'), { className: 'paint-swatch' });
  private readonly label = document.createElement('span');
  private key = '';
  private shownAt = 0;

  constructor(private readonly game: Game) {
    this.el.append(this.swatch, this.label);
  }

  /** Call every frame with the frame's time in milliseconds. */
  update(now: number): void {
    const g = this.game;
    const slot = g.selected_slot(), item = g.slot_item(slot);
    const shapeMode = g.held_shapeable_block() ? g.held_shape_mode() : -1;
    const paintInfo = g.held_paint_info();
    const paintHex = paintInfo ? g.held_paint_hex() : '';
    const key = g.slot_count(slot) > 0 ? `${slot}:${item}:${shapeMode}:${paintInfo}` : `${slot}:`;
    if (key !== this.key) {
      this.key = key;
      this.shownAt = now;
      this.swatch.style.display = paintHex ? 'inline-block' : 'none';
      if (paintHex) this.swatch.style.backgroundColor = paintHex;
      if (key.endsWith(':')) this.label.textContent = '';
      else if (paintInfo) this.label.textContent = `${g.item_name(item)} · ${paintInfo}`;
      else if (shapeMode === 0) this.label.textContent = `${g.item_name(item)} · Shift+R shapes`;
      else if (shapeMode > 0) this.label.textContent = `${g.item_name(item)} · ${g.held_shape_name()}`;
      else this.label.textContent = g.item_name(item);
    }
    this.el.classList.toggle('shown', this.label.textContent !== '' && now - this.shownAt < HELD_NAME_MS);
  }
}
