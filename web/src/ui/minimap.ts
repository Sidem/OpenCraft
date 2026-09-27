// Minimap in the HUD's top-right corner: the engine's top-down image of the loaded world (minimap.rs,
// one pixel per block column, north up, centred on the local player) in a round frame, an arrow for the
// local player's facing and one per co-op player (pinned to the rim when out of range). The image is
// redrawn at most 4 times a second and only when the engine says it changed; the ore, deposit and
// machine marks (`minimap_marks`, colours from the engine) and the player's pins (pins.ts; those out of
// range sit on the rim, pointing the way) go on an overlay canvas at the same rate, clipped by the round
// frame. N toggles it; whether it shows is UI state kept in localStorage. While shown it sets
// `--map-space` on #hud so the other top-right HUD items move below it.

import './minimap.css';
import type { Game } from '../wasm/engine.js';
import { h } from './dom';
import { drawMark, drawPin, type Pins } from './pins';

const STORAGE_KEY = 'opencraft.minimap.hidden';
const REDRAW_MS = 250;
const FRAME_PX = 150; // must match .minimap in minimap.css
const MAP_SPACE_PX = FRAME_PX + 12;

export class Minimap {
  private readonly el = h('div', 'minimap');
  private readonly canvas = h('canvas', 'mm-image');
  private readonly ctx: CanvasRenderingContext2D;
  /** Deposit and machine marks, drawn at the screen's resolution over the image. */
  private readonly overlay = h('canvas', 'mm-marks');
  private readonly marks: CanvasRenderingContext2D;
  private readonly me = h('div', 'mm-arrow mm-me');
  private readonly others: HTMLDivElement[] = [];
  private readonly size: number;
  private lastDraw = -Infinity;
  private shown: boolean;

  constructor(
    private readonly game: Game,
    private readonly memory: WebAssembly.Memory,
    private readonly pins: Pins,
  ) {
    this.size = game.minimap_size();
    this.canvas.width = this.canvas.height = this.size;
    this.ctx = this.canvas.getContext('2d')!;
    this.overlay.width = this.overlay.height = Math.round(FRAME_PX * (window.devicePixelRatio || 1));
    this.marks = this.overlay.getContext('2d')!;
    this.el.append(this.canvas, this.overlay, this.me, h('span', 'mm-north', 'N'));
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
    this.drawMarks(g.minimap_marks());
    this.placeOthers(g.minimap_players());
  }

  private show(on: boolean): void {
    this.el.classList.toggle('hidden', !on);
    this.lastDraw = -Infinity;
    document.getElementById('hud')!.style.setProperty('--map-space', on ? `${MAP_SPACE_PX}px` : '0px');
  }

  /** Engine marks at their columns' centres on the image, then pins (on the rim when out of range).
   * Marks are relative to the column the image was drawn around: the player's, this frame. */
  private drawMarks(marks: Int32Array): void {
    const c = this.marks, n = this.game.minimap_mark_fields();
    const w = this.overlay.width, px = w / this.size, half = this.size / 2;
    c.clearRect(0, 0, w, w);
    c.lineWidth = Math.max(1, px);
    for (let i = 0; i < marks.length; i += n) {
      drawMark(c, (marks[i] + half + 0.5) * px, (marks[i + 1] + half + 0.5) * px, marks[i + 2], marks[i + 3], px);
    }
    const cx = Math.floor(this.game.player_x()), cz = Math.floor(this.game.player_z());
    const rim = half - 6;
    const dpr = w / FRAME_PX;
    for (const pin of this.pins.list) {
      let dx = pin.x - cx, dz = pin.z - cz;
      const d = Math.hypot(dx, dz);
      if (d > rim) [dx, dz] = [(dx / d) * rim, (dz / d) * rim];
      drawPin(c, (dx + half + 0.5) * px, (dz + half + 0.5) * px, this.pins.kind(pin.kind).color, dpr * 0.7);
    }
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
