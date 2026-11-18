// LED lamp panel: opens when the player right-clicks an LED lamp (main.ts, through `game.take_panel_request()` like a
// sign's editor). Brightness is three bars (the lamp's block ids; engine `block::LED_STEPS`), colour a wheel
// (ui/colour-wheel.ts) or a swatch, with a lit preview. Done, or a click outside, writes it (`game.set_led`: an engine
// action, so every player sees it); Escape leaves the lamp as it was. The engine keeps the lamp (`led/`); this only
// edits it. The frame is the machine panel's (machine.css).

import './machine.css';
import './led-panel.css';
import type { Game } from '../wasm/engine.js';
import { hex } from './colour';
import { ColourWheel } from './colour-wheel';
import { button, h } from './dom';

/** Quick picks: warm white (the unpicked default), cool white, then the strong colours. */
const SWATCHES = [0xffe2b0, 0xf4f8ff, 0xff4a3d, 0xffa530, 0xffe14a, 0x4be06a, 0x32d4e0, 0x4a7dff, 0xa05cff, 0xff5cc8];

export class LedPanel {
  /** Called after the screen closes. `resume` is true when play should continue. */
  onClose: (resume: boolean) => void = () => {};

  private readonly backdrop = h('div', 'mp-backdrop hidden');
  private readonly dialog = h('div', 'mp led-panel');
  private readonly preview = h('div', 'led-preview');
  private readonly bars: HTMLButtonElement[] = [];
  private readonly wheel = new ColourWheel();
  private at: [number, number, number] = [0, 0, 0];
  private step = 0;
  private colour = 0;
  /** What the lamp was set to when opened, so an unchanged lamp is not written. */
  private before = '';

  constructor(private readonly game: Game) {
    this.dialog.setAttribute('role', 'dialog');
    this.dialog.setAttribute('aria-modal', 'true');
    this.dialog.tabIndex = -1;
    const head = h('div', 'mp-head');
    const close = button('close-btn', '×', () => this.close(true));
    close.setAttribute('aria-label', 'Close');
    head.append(h('h2', '', 'LED lamp'), h('span', 'mp-keys', 'Esc leaves it as it was'), close);
    const levels = h('div', 'led-levels');
    for (let i = 0; i < game.led_steps(); i++) {
      const bar = button('led-level', '', () => this.setStep(i));
      bar.title = ['Dim', 'Bright', 'Brightest'][i] ?? `Step ${i + 1}`;
      bar.setAttribute('aria-label', bar.title);
      for (let k = 0; k <= i; k++) bar.append(h('span', 'led-pip'));
      this.bars.push(bar);
      levels.append(bar);
    }
    const swatches = h('div', 'led-swatches');
    for (const rgb of SWATCHES) {
      const s = button('led-swatch', '', () => this.setColour(rgb));
      s.style.background = hex(rgb);
      s.setAttribute('aria-label', hex(rgb));
      swatches.append(s);
    }
    this.wheel.onChange = (rgb) => this.setColour(rgb, false);
    const side = h('div', 'led-side');
    side.append(levels, swatches, button('secondary-btn', 'Done', () => this.close(true)));
    const pickers = h('div', 'led-colours');
    pickers.append(this.wheel.el, side);
    const body = h('div', 'mp-body');
    body.append(this.preview, pickers);
    this.dialog.append(head, body);
    this.backdrop.append(this.dialog);
    this.backdrop.addEventListener('pointerdown', (e) => {
      if (e.target === this.backdrop) this.close(true);
    });
    this.backdrop.addEventListener('contextmenu', (e) => e.preventDefault());
    window.addEventListener('keydown', (e) => {
      if (this.isOpen && e.key === 'Escape') {
        e.preventDefault();
        this.close(false, false);
      }
    });
    document.body.append(this.backdrop);
  }

  get isOpen(): boolean {
    return !this.backdrop.classList.contains('hidden');
  }

  /** Opens the LED lamp at a block; false (and nothing opens) when none stands there. */
  open(x: number, y: number, z: number): boolean {
    const state = this.game.led_state(x, y, z);
    if (state.length < 2) return false;
    this.at = [x, y, z];
    [this.step, this.colour] = [state[0], state[1]];
    this.before = this.written();
    this.wheel.set(this.colour);
    this.show();
    this.backdrop.classList.remove('hidden');
    this.dialog.focus();
    return true;
  }

  /** Closes the panel, writing the lamp first unless `write` is false. */
  close(resume: boolean, write = true): void {
    if (!this.isOpen) return;
    if (write && this.written() !== this.before) {
      const [x, y, z] = this.at;
      this.game.set_led(x, y, z, this.step, this.colour);
    }
    this.backdrop.classList.add('hidden');
    this.onClose(resume);
  }

  private setStep(step: number): void {
    this.step = step;
    this.show();
  }

  private setColour(rgb: number, moveWheel = true): void {
    this.colour = rgb;
    if (moveWheel) this.wheel.set(rgb);
    this.show();
  }

  /** Draws the preview as the lamp will look: its colour, glowing wider with each step. */
  private show(): void {
    const glow = 10 + this.step * 14;
    this.preview.style.background = hex(this.colour);
    this.preview.style.boxShadow = `0 0 ${glow}px ${glow / 2}px ${hex(this.colour)}`;
    this.bars.forEach((b, i) => b.classList.toggle('active', i === this.step));
  }

  private written(): string {
    return `${this.step}:${this.colour}`;
  }
}
