// What the analytics hub (analytics.ts) hands each of its tabs: the engine, the timescale and the item icons. A tab is
// one panel of the hub (overview, power, materials, machines); it builds its DOM once and `refresh`es it once a second
// while it is the one showing. To add a tab: implement `Tab` in its own file and list it in analytics.ts.

import type { Game } from '../wasm/engine.js';
import { h } from './dom';

export interface Ctx {
  game: Game;
  /** The icon canvas of an item id (blocks are items too). */
  icon: (item: number) => HTMLCanvasElement;
  /** Seconds the charts cover, and their time-axis divisions. */
  secs: number;
  ticks: number;
  /** Switches the hub to another tab (by name). */
  go: (tab: string) => void;
}

export interface Tab {
  readonly el: HTMLElement;
  /** Called each time the tab comes into view, before its first refresh. */
  open?(): void;
  /** Called when the tab is shown and then every second. */
  refresh(ctx: Ctx): void;
}

export const ICON_PX = 22;

/** A small copy of an item icon. */
export function iconCanvas(source: HTMLCanvasElement, px = ICON_PX): HTMLCanvasElement {
  const canvas = h('canvas', 'an-icon');
  canvas.width = canvas.height = px;
  canvas.getContext('2d')!.drawImage(source, 0, 0, px, px);
  return canvas;
}

/** How many chart points fit `width` pixels (about one per three pixels, at least 30). */
export function pointsFor(width: number): number {
  return Math.max(30, Math.floor(width / 3));
}

/** A coloured dot. */
export function dot(colour: string): HTMLSpanElement {
  const d = h('span', 'an-dot');
  d.style.background = colour;
  return d;
}

/** A segmented row of buttons, one pressed; `onPick` gets the index. */
export function segmented(labels: string[], onPick: (i: number) => void, label: string): { el: HTMLElement; set(i: number): void } {
  const el = h('div', 'an-seg');
  el.setAttribute('role', 'group');
  el.setAttribute('aria-label', label);
  const buttons = labels.map((text, i) => {
    const b = h('button', 'an-seg-btn', text);
    b.type = 'button';
    b.addEventListener('click', () => onPick(i));
    return b;
  });
  el.append(...buttons);
  return { el, set: (i) => buttons.forEach((b, k) => b.setAttribute('aria-pressed', String(k === i))) };
}
