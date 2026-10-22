// The telescope's host side (the engine's hands are `telescope.rs`): the view it draws and the pins it makes.
// - Zoom: the engine answers with an eased zoom (`scope_zoom`, never a jump); the field of view is the comfort setting
//   divided by it, in the tangent so a 4× zoom looks 4× closer. Mouse look is divided by it too (main.ts), so the view
//   turns as fast on screen as it always does. A static dark ring and hairlines frame it (scope.css).
// - Pins: a left-click makes the engine ask for a pin (`take_pin_request`): a Note with no words on the ground it looked at,
//   kept like any pin (pins.ts), editable on the map. Every pin, filtered or not, goes back to the engine as a coloured
//   column (`set_scope_pins`) whenever the list changes.
// - The pointer being freed (a panel, the menu) lowers the telescope.

import './scope.css';
import type { Game } from '../wasm/engine.js';
import type { Renderer } from '../render/renderer';
import type { ComfortStore } from '../comfort/settings';
import { h } from './dom';
import type { Pins } from './pins';

/** A pin made within this many blocks of another is the same pin. */
const SAME_SPOT = 2;
/** The ring shows from this zoom on and is whole by this one, so it fades in with the view. */
const RING_FROM = 1.05, RING_FULL = 1.6;

export class Scope {
  readonly el = h('div', 'scope');
  private zoomed = false;

  constructor(
    private readonly game: Game,
    private readonly renderer: Renderer,
    private readonly comfort: ComfortStore,
    private readonly pins: Pins,
    private readonly locked: () => boolean,
    private readonly onPin: () => void,
  ) {
    this.el.append(h('div', 'scope-hair scope-h'), h('div', 'scope-hair scope-v'));
    document.getElementById('hud')!.prepend(this.el);
    pins.subscribe(() => this.send());
    this.send();
  }

  /** Once a frame. */
  update(): void {
    if (!this.locked() && this.game.scope_on()) this.game.close_scope();
    const zoom = this.game.scope_zoom();
    if (zoom !== 1 || this.zoomed) {
      const base = (this.comfort.settings.fov * Math.PI) / 180;
      this.renderer.fovY = 2 * Math.atan(Math.tan(base / 2) / zoom);
      this.zoomed = zoom !== 1;
      this.el.style.opacity = String(Math.min(1, Math.max(0, (zoom - RING_FROM) / (RING_FULL - RING_FROM))));
    }
    const pin = this.game.take_pin_request();
    if (pin.length === 3) this.pin(pin[0], pin[1], pin[2]);
  }

  private pin(x: number, y: number, z: number): void {
    if (this.pins.list.some((p) => Math.abs(p.x - x) <= SAME_SPOT && Math.abs(p.z - z) <= SAME_SPOT)) return;
    this.pins.add({ x, y, z, kind: 'note', note: '' });
    this.onPin();
    this.el.classList.remove('pinned');
    void this.el.offsetWidth; // restart the pulse
    this.el.classList.add('pinned');
  }

  /** Every pin as `[x, y, z, 0xRRGGBB]`, `y` the marker for "no height" when the pin has none. */
  private send(): void {
    const none = this.game.scope_no_height();
    const flat = new Int32Array(this.pins.list.length * 4);
    this.pins.list.forEach((p, i) => {
      flat.set([p.x, Number.isFinite(p.y) ? (p.y as number) : none, p.z, parseInt(this.pins.kind(p.kind).color.slice(1), 16)], i * 4);
    });
    this.game.set_scope_pins(flat);
  }
}
