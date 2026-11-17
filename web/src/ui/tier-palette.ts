// "Colour-blind colours" on the menu's Picture settings: which palette the tier colours (Mk1 to Mk5) wear. The choice is a
// local setting kept in localStorage; `TierPaletteStore` tells subscribers (main.ts, which sends it to the engine and
// repaints the baked layers) when it changes. Presentation only. The row shows each palette as five colour chips, so
// no sentence is needed beyond the tooltip. To add a palette: a table in the engine's `tier_palette.rs`, a name here.

import { button, h } from './dom';
import './tier-palette.css';

export const PALETTE_NAMES = ['Standard', 'Colour-blind'] as const;
const STORAGE_KEY = 'opencraft.tierPalette';

export class TierPaletteStore {
  /** Index into `PALETTE_NAMES`, and the engine's palette number. */
  palette = 0;
  private readonly listeners: (() => void)[] = [];

  constructor() {
    try {
      const saved = Number(localStorage.getItem(STORAGE_KEY));
      if (Number.isInteger(saved) && saved >= 0 && saved < PALETTE_NAMES.length) this.palette = saved;
    } catch {
      // Storage unavailable: the standard palette.
    }
  }

  /** Calls `fn` now and after every change. */
  subscribe(fn: () => void): void {
    this.listeners.push(fn);
    fn();
  }

  set(palette: number): void {
    if (palette === this.palette) return;
    this.palette = palette;
    try {
      localStorage.setItem(STORAGE_KEY, String(palette));
    } catch {
      // Not persisted; the choice still applies for this session.
    }
    for (const fn of this.listeners) fn();
  }
}

export class TierPalettePanel {
  readonly el = h('div', 'comfort');

  /** `colours[i]` are the five 0xRRGGBB of palette `i`. */
  constructor(store: TierPaletteStore, colours: number[][]) {
    const row = h('div', 'comfort-row');
    const choices = h('div', 'comfort-choices');
    const buttons = PALETTE_NAMES.map((name, i) => {
      const b = button('secondary-btn', '', () => store.set(i));
      b.title = i === 0 ? 'The usual tier colours' : 'Five colours that stay apart for red-green and blue-yellow colour blindness, dark to light';
      b.append(...colours[i].map((c) => this.chip(c)), h('span', 'tier-chip-name', name));
      return b;
    });
    choices.append(...buttons);
    row.append(h('span', 'comfort-label', 'Tier colours'), choices);
    this.el.append(row);
    store.subscribe(() => buttons.forEach((b, i) => b.classList.toggle('active', i === store.palette)));
  }

  private chip(colour: number): HTMLElement {
    const chip = h('span', 'tier-chip');
    chip.style.background = `#${colour.toString(16).padStart(6, '0')}`;
    return chip;
  }
}
