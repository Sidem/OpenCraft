// Movement vignette: the edges of the view darken while the camera turns or the player moves fast,
// which narrows what moves across the retina. This is one of the best-studied comfort aids (dynamic
// field-of-view restriction) and it is only visible while moving. Presentation only: `update` is
// called once per frame with the camera; the strength follows the comfort setting. To tune what counts
// as fast: the constants below.

import './vignette.css';
import type { ComfortStore } from '../comfort/settings';

/** How far the clear centre shrinks at full strength, in percent of the corner distance, per level. */
const SHRINK = [0, 28, 45, 60];
/** Turning slower than this (rad/s) and walking slower than this (blocks/s) does not count. */
const TURN_FREE = 0.6, TURN_FULL = 3.0;
const MOVE_FREE = 3.0, MOVE_FULL = 9.0;
/** A jump this long in one frame is a teleport, not movement. */
const TELEPORT = 8;
/** Rates (per second) at which the strength rises and falls. */
const RISE = 10, FALL = 2.5;

export class Vignette {
  readonly el = document.createElement('div');
  /** True while the menu previews the setting: the vignette shows at its level even at rest. */
  previewing: () => boolean = () => false;
  private strength = 0;
  private shown = -1;
  private last: [number, number, number, number, number] | null = null;

  constructor(private readonly comfort: ComfortStore) {
    this.el.id = 'vignette';
    document.getElementById('hud')!.prepend(this.el);
  }

  update(dt: number, yaw: number, pitch: number, x: number, y: number, z: number): void {
    const level = this.comfort.settings.vignette;
    let target = this.previewing() ? 1 : 0;
    const prev = this.last;
    this.last = [yaw, pitch, x, y, z];
    if (prev && dt > 0 && level > 0) {
      let dyaw = Math.abs(yaw - prev[0]);
      if (dyaw > Math.PI) dyaw = 2 * Math.PI - dyaw;
      const turn = Math.hypot(dyaw, pitch - prev[1]) / dt;
      const dist = Math.hypot(x - prev[2], y - prev[3], z - prev[4]);
      const move = dist < TELEPORT ? dist / dt : 0;
      const t = Math.max(ramp(turn, TURN_FREE, TURN_FULL), ramp(move, MOVE_FREE, MOVE_FULL));
      target = Math.max(target, t);
    }
    if (level === 0) target = 0;
    this.strength += (target - this.strength) * (1 - Math.exp(-(target > this.strength ? RISE : FALL) * dt));
    const clear = Math.round(100 - this.strength * SHRINK[level]);
    if (clear !== this.shown) {
      this.shown = clear;
      this.el.style.setProperty('--vignette-clear', `${clear}%`);
    }
  }
}

const ramp = (v: number, free: number, full: number) => Math.min(1, Math.max(0, (v - free) / (full - free)));
