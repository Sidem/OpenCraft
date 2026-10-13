// Position readout under the minimap: the player's block X, Y and Z (red, green, blue) beside a small
// 3D gizmo. The gizmo is fixed to the world and drawn from the south and above, like the map: red X
// points east, green Y up, blue Z south (towards the viewer). A white arrow shows where the player is
// looking, in the same projection, so looking up, down or along an axis reads at a glance. Owned and
// updated by the Minimap (it shows and hides with it); the layout size is mirrored as COMPASS_PX there.

import './compass.css';
import type { Game } from '../wasm/engine.js';
import { h } from './dom';

export const COMPASS_PX = 64; // must match .compass height in compass.css

const SIZE = 64; // gizmo canvas, CSS pixels
const AXES = [
  { label: 'X', colour: '#ff5d5d', dir: [1, 0, 0] },
  { label: 'Y', colour: '#6ee66e', dir: [0, 1, 0] },
  { label: 'Z', colour: '#5fa4ff', dir: [0, 0, 1] },
] as const;

/** Screen offset (right, down) of a world direction seen from the south and above. */
function project(x: number, y: number, z: number): [number, number] {
  return [x - 0.55 * z, -y + 0.45 * z];
}

export class Compass {
  readonly el = h('div', 'compass');
  private readonly canvas = h('canvas', 'cp-gizmo');
  private readonly ctx: CanvasRenderingContext2D;
  private readonly values = AXES.map(() => h('span', 'cp-value', '0'));
  private readonly dpr = window.devicePixelRatio || 1;
  private text = '';

  constructor(private readonly game: Game) {
    this.canvas.width = this.canvas.height = Math.round(SIZE * this.dpr);
    this.ctx = this.canvas.getContext('2d')!;
    const rows = h('div', 'cp-rows');
    AXES.forEach((a, i) => {
      const label = h('span', 'cp-label', a.label);
      label.style.color = a.colour;
      const row = h('div', 'cp-row');
      row.append(label, this.values[i]);
      rows.append(row);
    });
    this.el.append(this.canvas, rows);
  }

  /** Call every frame while shown. */
  update(): void {
    const g = this.game;
    const pos = [g.player_x(), g.player_y(), g.player_z()].map(Math.floor);
    const text = pos.join(',');
    if (text !== this.text) {
      this.text = text;
      pos.forEach((v, i) => (this.values[i].textContent = String(v)));
    }
    this.drawGizmo(g.yaw(), g.pitch());
  }

  private drawGizmo(yaw: number, pitch: number): void {
    const c = this.ctx, s = this.dpr, len = 21 * s;
    const ox = (SIZE / 2) * s, oy = (SIZE / 2 + 1) * s;
    c.clearRect(0, 0, SIZE * s, SIZE * s);
    c.lineCap = 'round';
    c.font = `700 ${10 * s}px sans-serif`;
    c.textAlign = 'center';
    c.textBaseline = 'middle';
    for (const a of AXES) {
      const [dx, dy] = project(a.dir[0], a.dir[1], a.dir[2]);
      c.strokeStyle = a.colour;
      c.globalAlpha = 0.3; // the negative half, shorter and faint
      c.lineWidth = 1.5 * s;
      c.beginPath();
      c.moveTo(ox, oy);
      c.lineTo(ox - dx * len * 0.4, oy - dy * len * 0.4);
      c.stroke();
      c.globalAlpha = 1;
      c.lineWidth = 2 * s;
      c.beginPath();
      c.moveTo(ox, oy);
      c.lineTo(ox + dx * len * 0.8, oy + dy * len * 0.8);
      c.stroke();
      c.fillStyle = a.colour;
      c.fillText(a.label, ox + dx * len, oy + dy * len);
    }
    // The look direction (yaw 0 looks towards -Z, see Player::look_dir).
    const cp = Math.cos(pitch);
    const [lx, ly] = project(Math.sin(yaw) * cp, Math.sin(pitch), -Math.cos(yaw) * cp);
    const tx = ox + lx * len * 0.85, ty = oy + ly * len * 0.85;
    const angle = Math.atan2(ly, lx);
    c.strokeStyle = c.fillStyle = '#fff';
    c.shadowColor = '#000';
    c.shadowBlur = 3 * s;
    c.lineWidth = 2.5 * s;
    c.beginPath();
    c.moveTo(ox, oy);
    c.lineTo(tx, ty);
    c.stroke();
    c.beginPath();
    c.moveTo(tx + Math.cos(angle) * 5 * s, ty + Math.sin(angle) * 5 * s);
    c.lineTo(tx + Math.cos(angle + 2.5) * 5 * s, ty + Math.sin(angle + 2.5) * 5 * s);
    c.lineTo(tx + Math.cos(angle - 2.5) * 5 * s, ty + Math.sin(angle - 2.5) * 5 * s);
    c.fill();
    c.shadowBlur = 0;
  }
}
