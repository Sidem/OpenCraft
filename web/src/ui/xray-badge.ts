// The X-ray goggles' state on screen, and the player's choice of on or off. Shows nothing until goggles are worn (the
// engine's `xray_worn`); then a goggles icon with the key cap beside it, lit while the view is on and dim while it is off
// (also dim in the strategy view, where the engine shows none), and a faint cyan rim round the whole view while it is on,
// so the player never wonders whether they are looking through the ground. The choice is kept in this browser and sent to
// the engine at start (`set_xray_enabled`); wearing the goggles is core state, switching them is not. Words stay in the
// tooltip (the "show, don't write" rule). To change the look: `xray-badge.css`; the key itself is in `input.ts`.

import './xray-badge.css';
import type { Game } from '../wasm/engine.js';
import { h } from './dom';
import { keyLabels } from './keyboard-layout';

const STORE_KEY = 'opencraft.xray';
const SVG = 'http://www.w3.org/2000/svg';

type State = 'hidden' | 'on' | 'off';

export class XrayBadge {
  readonly el = h('div', 'hidden');
  private readonly rim = h('div');
  private readonly cap = h('kbd', '');
  private state: State = 'hidden';

  constructor(private readonly game: Game) {
    this.el.id = 'xray-badge';
    this.rim.id = 'xray-rim';
    this.el.append(icon(), this.cap);
    keyLabels.subscribe(() => this.drawCap());
    this.drawCap();
    game.set_xray_enabled(load());
    // The badge sits at the top of the left HUD column (hud-columns.css); the rim covers the whole view.
    document.getElementById('hud')!.append(this.rim);
    document.getElementById('hud-left')!.append(this.el);
  }

  /** The toggle key. Returns a line to show (the goggles are not worn), or "" after switching. */
  toggle(): string {
    const notice = this.game.toggle_xray();
    if (notice === '') {
      save(this.game.xray_enabled());
      this.el.classList.remove('flip');
      void this.el.offsetWidth; // restart the animation
      this.el.classList.add('flip');
    }
    return notice;
  }

  /** Once a frame: follows what the engine says is worn and active. */
  update(): void {
    const next: State = !this.game.xray_worn() ? 'hidden' : this.game.xray_active() ? 'on' : 'off';
    if (next === this.state) return;
    this.state = next;
    this.el.classList.toggle('hidden', next === 'hidden');
    this.el.classList.toggle('on', next === 'on');
    this.rim.classList.toggle('on', next === 'on');
    this.el.title =
      next === 'on' ? 'X-ray goggles: on. Shift+X switches them off' : 'X-ray goggles: off. Shift+X switches them on';
  }

  private drawCap(): void {
    this.cap.textContent = `⇧ ${keyLabels.label('KeyX')}`;
  }
}

/** Goggles seen from the front: a strap, two round lenses and a bridge. */
function icon(): SVGElement {
  const svg = document.createElementNS(SVG, 'svg');
  svg.setAttribute('viewBox', '0 0 32 16');
  svg.setAttribute('aria-hidden', 'true');
  svg.innerHTML =
    '<path class="strap" d="M1 8h4M27 8h4" />' +
    '<circle class="lens" cx="10" cy="8" r="5.5" /><circle class="lens" cx="22" cy="8" r="5.5" />' +
    '<path class="bridge" d="M15.5 7.5h1" /><path class="glint" d="M7.5 6.2a3 3 0 0 1 2-1.2M19.5 6.2a3 3 0 0 1 2-1.2" />';
  return svg;
}

function load(): boolean {
  try {
    return localStorage.getItem(STORE_KEY) !== '0';
  } catch {
    return true;
  }
}

function save(on: boolean): void {
  try {
    localStorage.setItem(STORE_KEY, on ? '1' : '0');
  } catch {
    // Private windows and blocked storage: the choice just lasts until the page closes.
  }
}
