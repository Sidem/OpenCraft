// Position readout under the minimap: the player's block X, Y and Z (red, green, blue), and at the end of
// each row which way the player faces along that axis: a "+" or "−" (the way the number grows or shrinks).
// The axis faced most is a solid pill, an axis also faced noticeably (diagonal or sloped look) is a faint
// sign, an axis faced barely at all is blank. Owned and updated by the Minimap (it shows and hides with
// it); the layout size is mirrored as COMPASS_PX there.

import './compass.css';
import type { Game } from '../wasm/engine.js';
import { h } from './dom';

export const COMPASS_PX = 64; // must match .compass height in compass.css

const AXES = [
  { label: 'X', colour: '#ff5d5d' },
  { label: 'Y', colour: '#6ee66e' },
  { label: 'Z', colour: '#5fa4ff' },
] as const;
const FACED = 0.38; // a look component this strong counts as facing along that axis

/** Per axis: '' (not facing it), '+' or '−' as a faint sign, '+!' or '−!' for the main one. */
export function facing(yaw: number, pitch: number): string[] {
  // The look direction (yaw 0 looks towards -Z, see Player::look_dir).
  const cp = Math.cos(pitch);
  const look = [Math.sin(yaw) * cp, Math.sin(pitch), -Math.cos(yaw) * cp];
  const main = look.reduce((best, v, i) => (Math.abs(v) > Math.abs(look[best]) ? i : best), 0);
  return look.map((v, i) => {
    if (i !== main && Math.abs(v) < FACED) return '';
    return (v < 0 ? '−' : '+') + (i === main ? '!' : '');
  });
}

export class Compass {
  readonly el = h('div', 'compass');
  private readonly values = AXES.map(() => h('span', 'cp-value', '0'));
  private readonly tags = AXES.map(() => h('span', 'cp-tag'));
  private text = '';
  private tagText = '';

  constructor(private readonly game: Game) {
    const rows = h('div', 'cp-rows');
    AXES.forEach((a, i) => {
      const label = h('span', 'cp-label', a.label);
      label.style.color = a.colour;
      this.tags[i].style.setProperty('--axis', a.colour);
      const row = h('div', 'cp-row');
      row.append(label, this.values[i], this.tags[i]);
      rows.append(row);
    });
    this.el.append(rows);
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
    const tags = facing(g.yaw(), g.pitch());
    const tagText = tags.join(',');
    if (tagText !== this.tagText) {
      this.tagText = tagText;
      tags.forEach((t, i) => {
        this.tags[i].textContent = t.slice(0, 1);
        this.tags[i].classList.toggle('main', t.endsWith('!'));
      });
    }
  }
}
