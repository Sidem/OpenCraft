// Minimap in the HUD's top-right corner: the engine's top-down image of the loaded world (minimap.rs,
// one pixel per block column, north up, centred on the local player) in a round frame, an arrow for the
// local player's facing and one per co-op player (pinned to the rim when out of range). The image is
// redrawn at most 4 times a second and only when the engine says it changed. N toggles it; whether it
// shows is UI state kept in localStorage. While shown it sets `--map-space` on #hud so the other
// top-right HUD items move below it.

import './minimap.css';
import type { Game } from '../wasm/engine.js';
import { h } from './dom';

const STORAGE_KEY = 'opencraft.minimap.hidden';
const REDRAW_MS = 250;
const FRAME_PX = 150; // must match .minimap in minimap.css
const MAP_SPACE_PX = FRAME_PX + 12;

export class Minimap {
  private readonly el = h('div', 'minimap');
  private readonly canvas = h('canvas', 'mm-image');
  private readonly ctx: CanvasRenderingContext2D;
  private readonly me = h('div', 'mm-arrow mm-me');
  private readonly others: HTMLDivElement[] = [];
  private readonly size: number;
  private lastDraw = -Infinity;
  private shown: boolean;

  constructor(private readonly game: Game, private readonly memory: WebAssembly.Memory) {
    this.size = game.minimap_size();
    this.canvas.width = this.canvas.height = this.size;
    this.ctx = this.canvas.getContext('2d')!;
    this.el.append(this.canvas, this.me, h('span', 'mm-north', 'N'));
    document.getElementById('hud')!.append(this.el);
    this.shown = !loadHidden();
    this.show(this.shown);
  }

  toggle(): void {
    this.shown = !this.shown;
    this.show(this.shown);
    saveHidden(!this.shown);
  }

  /** Call every frame: turns the arrows, and redraws the image when it's due and changed. */
  update(now: number): void {
    if (!this.shown) return;
    const g = this.game;
    this.me.style.transform = `rotate(${g.yaw()}rad)`;
    if (now - this.lastDraw < REDRAW_MS) return;
    this.lastDraw = now;
    if (g.minimap_redraw()) {
      const bytes = this.size * this.size * 4;
      const view = new Uint8ClampedArray(this.memory.buffer, g.minimap_ptr(), bytes);
      this.ctx.putImageData(new ImageData(view, this.size, this.size), 0, 0);
    }
    this.placeOthers(g.minimap_players());
  }

  private show(on: boolean): void {
    this.el.classList.toggle('hidden', !on);
    this.lastDraw = -Infinity;
    document.getElementById('hud')!.style.setProperty('--map-space', on ? `${MAP_SPACE_PX}px` : '0px');
  }

  /** One arrow per other player: (x offset, z offset, yaw) triples in blocks. */
  private placeOthers(marks: Float32Array): void {
    const count = marks.length / 3;
    while (this.others.length < count) {
      const a = h('div', 'mm-arrow mm-other');
      this.el.append(a);
      this.others.push(a);
    }
    const scale = FRAME_PX / this.size;
    const rim = FRAME_PX / 2 - 8;
    this.others.forEach((a, i) => {
      a.classList.toggle('hidden', i >= count);
      if (i >= count) return;
      let x = marks[i * 3] * scale, y = marks[i * 3 + 1] * scale;
      const d = Math.hypot(x, y);
      if (d > rim) [x, y] = [(x / d) * rim, (y / d) * rim];
      a.style.transform = `translate(${x}px, ${y}px) rotate(${marks[i * 3 + 2]}rad)`;
    });
  }
}

// Storage can be unavailable (private windows, blocked site data); the map then just shows.
function loadHidden(): boolean {
  try {
    return localStorage.getItem(STORAGE_KEY) === '1';
  } catch {
    return false;
  }
}

function saveHidden(hidden: boolean): void {
  try {
    localStorage.setItem(STORAGE_KEY, hidden ? '1' : '0');
  } catch {
    // Not remembered; see loadHidden.
  }
}
