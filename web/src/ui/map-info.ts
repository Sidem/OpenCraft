// The info box when the mouse rests on a point of interest on the world map or the corner minimap: what the engine
// says the nearest marks are (`world_map_describe`: only what the player knows) or the pin's kind and note, for the
// marks the filters let through (map-filter.ts, which also reads the engine's lines). One box, shared by both maps. To
// say more about a mark, change its words in the engine's `minimap/describe.rs`.

import './map-info.css';
import type { Game } from '../wasm/engine.js';
import { h } from './dom';
import { type Info, type MapFilter, parseInfo } from './map-filter.ts';
import type { Pin, Pins } from './pins.ts';

/** Most marks one box names. */
const MAX_LINES = 3;

/** What the maps say at world column (x, z), within `reach` blocks: the pin (if one was hit) first, then the engine's marks. */
export function infoAt(game: Game, filter: MapFilter, pins: Pins, pin: Pin | null, x: number, z: number, reach: number): Info[] {
  const out: Info[] = [];
  if (pin) {
    const kind = pins.kind(pin.kind);
    out.push({ colour: kind.color, title: `Pin: ${kind.label}`, detail: pin.note || 'Your pin.' });
  }
  const marks = game.world_map_describe(Math.floor(x), Math.floor(z), Math.ceil(reach));
  return [...out, ...parseInfo(marks, filter)].slice(0, MAX_LINES);
}

export class MapInfo {
  private readonly el = h('div', 'mi hidden');

  constructor() {
    this.el.setAttribute('role', 'tooltip');
    document.body.append(this.el);
  }

  /** Shows `lines` near the pointer, or hides the box when there are none. */
  show(lines: readonly Info[], clientX: number, clientY: number): void {
    if (lines.length === 0) return this.hide();
    const e = this.el;
    e.replaceChildren(...lines.map(line));
    e.classList.remove('hidden');
    const gap = 16;
    const left = clientX + gap + e.offsetWidth > window.innerWidth ? clientX - gap - e.offsetWidth : clientX + gap;
    const top = clientY + gap + e.offsetHeight > window.innerHeight ? clientY - gap - e.offsetHeight : clientY + gap;
    e.style.left = `${Math.max(4, left)}px`;
    e.style.top = `${Math.max(4, top)}px`;
  }

  hide(): void {
    this.el.classList.add('hidden');
  }
}

function line(info: Info): HTMLElement {
  const row = h('div', 'mi-line');
  const title = h('div', 'mi-title');
  if (info.colour !== null) {
    const swatch = h('span', 'mi-swatch');
    swatch.style.background = info.colour;
    title.append(swatch);
  }
  title.append(info.title);
  row.append(title, h('div', 'mi-detail', info.detail));
  return row;
}
