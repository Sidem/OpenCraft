// Logbook panel: opens when the player right-clicks a ruin's logbook (the request comes through
// `game.take_panel_request()`, main.ts asks `game.logbook(x, y, z)` and opens this when it answers). A page is a title, its
// text and, when it points at ground, a last `pin` line (x, z, ore block id) the player can put on the map (`Pins`); the engine
// writes pages (`worldgen/logbook.rs`), this only shows them. The frame is the machine panel's (machine.css).

import './machine.css';
import './logbook.css';
import type { Game } from '../wasm/engine.js';
import { button, h } from './dom';
import type { Pins } from './pins';

export class LogbookPanel {
  /** Called after the screen closes. `resume` is true when play should continue. */
  onClose: (resume: boolean) => void = () => {};

  private readonly backdrop = h('div', 'mp-backdrop hidden');
  private readonly dialog = h('div', 'mp logbook');
  private readonly title = h('h2', '', 'Logbook');
  private readonly body = h('div', 'mp-body');

  constructor(private readonly game: Game, private readonly pins: Pins) {
    this.dialog.setAttribute('role', 'dialog');
    this.dialog.setAttribute('aria-modal', 'true');
    this.dialog.tabIndex = -1;
    const head = h('div', 'mp-head');
    const close = button('close-btn', '×', () => this.close(true));
    close.setAttribute('aria-label', 'Close');
    head.append(this.title, h('span', 'mp-keys', 'E or click outside to return'), close);
    this.dialog.append(head, this.body);
    this.backdrop.append(this.dialog);
    this.backdrop.addEventListener('pointerdown', (e) => {
      if (e.target === this.backdrop) this.close(true);
    });
    this.backdrop.addEventListener('contextmenu', (e) => e.preventDefault());
    window.addEventListener('keydown', (e) => {
      if (!this.isOpen || e.repeat) return;
      if (e.code === 'KeyE') {
        e.preventDefault();
        this.close(true);
      } else if (e.key === 'Escape') {
        e.preventDefault();
        this.close(false);
      }
    });
    document.body.append(this.backdrop);
  }

  get isOpen(): boolean {
    return !this.backdrop.classList.contains('hidden');
  }

  /** Shows the page of the logbook at a block; false (and nothing opens) when no logbook stands there. */
  open(x: number, y: number, z: number): boolean {
    const page = this.game.logbook(x, y, z);
    if (!page) return false;
    const lines = page.split('\n');
    const pin = lines[lines.length - 1].startsWith('pin\t') ? lines.pop()!.split('\t') : null;
    this.title.textContent = lines[0];
    this.body.replaceChildren(h('p', 'logbook-text', lines.slice(1).join('\n')));
    if (pin) this.body.append(this.pinButton(Number(pin[1]), Number(pin[2]), Number(pin[3])));
    this.backdrop.classList.remove('hidden');
    this.dialog.focus();
    return true;
  }

  close(resume: boolean): void {
    if (!this.isOpen) return;
    this.backdrop.classList.add('hidden');
    this.onClose(resume);
  }

  /** A button that puts the page's ground on the map as an ore pin (once). */
  private pinButton(x: number, z: number, ore: number): HTMLElement {
    const kind = this.pins.kinds.some((k) => k.id === `ore-${ore}`) ? `ore-${ore}` : 'note';
    const done = () => this.pins.list.some((p) => p.x === x && p.z === z && p.note === 'Logbook');
    const b = button('secondary-btn', done() ? 'Pinned on the map' : 'Pin on the map', () => {
      this.pins.add({ x, z, kind, note: 'Logbook' });
      b.textContent = 'Pinned on the map';
      b.disabled = true;
    });
    b.disabled = done();
    return b;
  }
}
