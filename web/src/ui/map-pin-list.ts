// The world map's list of pins (beside the map, worldmap.ts): each pin the filter shows, with how far it is and which
// way from the player; clicking one is handed back to the map. Styles live in worldmap.css.

import type { Game } from '../wasm/engine.js';
import { button, h } from './dom';
import type { MapFilter } from './map-filter';
import type { Pin, Pins } from './pins';

/** Fills `list` with the pins `filter` shows; `onPick` gets the clicked one. */
export function fillPinList(list: HTMLElement, game: Game, pins: Pins, filter: MapFilter, onPick: (pin: Pin) => void): void {
  list.replaceChildren();
  const shown = pins.list.filter((p) => filter.showsPin(p.kind));
  if (shown.length === 0) {
    const none = pins.list.length === 0 ? 'No pins yet. Click the map to pin a spot.' : 'All your pins are hidden by the filter.';
    list.append(h('p', 'wm-empty', none));
    return;
  }
  const px = game.player_x(), pz = game.player_z();
  for (const pin of shown) {
    const row = button('wm-pin', '', () => onPick(pin));
    const dot = h('span', 'og-swatch');
    dot.style.background = pins.kind(pin.kind).color;
    const dx = pin.x + 0.5 - px, dz = pin.z + 0.5 - pz, d = Math.round(Math.hypot(dx, dz));
    const far = d < 4 ? 'here' : `${d} blocks ${compass(dx, dz)}`;
    row.append(dot, h('span', 'wm-pin-label', pins.label(pin)), h('span', 'wm-pin-far', far));
    list.append(row);
  }
}

/** The compass direction of an offset (north is -z). */
function compass(dx: number, dz: number): string {
  const names = ['N', 'NE', 'E', 'SE', 'S', 'SW', 'W', 'NW'];
  const turn = Math.atan2(dx, -dz) / (Math.PI / 4);
  return names[(Math.round(turn) + 8) % 8];
}
