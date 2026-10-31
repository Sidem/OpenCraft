// Sign editor: opens when the player right-clicks a note sign (main.ts, through `game.take_panel_request()` like a
// logbook). Four lines, each cut to what a sign's line holds (`game.sign_line_bytes()`), the board's and the ink's colour
// from a colour wheel (ui/colour-wheel.ts) or a swatch, and a preview of the board. Done, Enter on the last line or a
// click outside writes it (`game.edit_sign`: an engine action, so every player sees it); Escape leaves it as it was.
// The engine keeps the sign (`signs/`); this only edits it. The frame is the machine panel's (machine.css).

import './machine.css';
import './sign.css';
import type { Game } from '../wasm/engine.js';
import { fitBytes, hex } from './colour';
import { ColourWheel } from './colour-wheel';
import { button, h } from './dom';

const ROWS = 4; // the engine's `signs::LINES`
/** Quick picks: the default pale wood and dark ink, paper, then a few strong colours. */
const SWATCHES = [0xe6d3a3, 0x2b2118, 0xf5f1e6, 0x1f3a5f, 0x2f6d3a, 0x9c2f2f, 0xf2c14e, 0x6b4f8a, 0x111111, 0xffffff];
const PARTS = ['Board', 'Text'];

export class SignPanel {
  /** Called after the screen closes. `resume` is true when play should continue. */
  onClose: (resume: boolean) => void = () => {};

  private readonly backdrop = h('div', 'mp-backdrop hidden');
  private readonly dialog = h('div', 'mp sign-editor');
  private readonly preview = h('div', 'sign-preview');
  private readonly inputs: HTMLInputElement[] = [];
  private readonly parts: HTMLButtonElement[];
  private readonly wheel = new ColourWheel();
  private at: [number, number, number] = [0, 0, 0];
  /** Board and ink, and which of them the wheel sets. */
  private colours = [0, 0];
  private part = 0;
  /** What the sign said when opened, so an unchanged sign is not written. */
  private before = '';

  constructor(private readonly game: Game) {
    this.dialog.setAttribute('role', 'dialog');
    this.dialog.setAttribute('aria-modal', 'true');
    this.dialog.tabIndex = -1;
    const head = h('div', 'mp-head');
    const close = button('close-btn', '×', () => this.close(true));
    close.setAttribute('aria-label', 'Close');
    head.append(h('h2', '', 'Sign'), h('span', 'mp-keys', 'Enter when done · Esc leaves it as it was'), close);
    const lines = h('div', 'sign-lines');
    for (let i = 0; i < ROWS; i++) {
      const input = h('input', 'sign-line');
      input.type = 'text';
      input.placeholder = `Line ${i + 1}`;
      input.addEventListener('input', () => {
        const fit = fitBytes(input.value, game.sign_line_bytes());
        if (fit !== input.value) input.value = fit;
        this.show();
      });
      input.addEventListener('keydown', (e) => {
        if (e.key !== 'Enter') return;
        e.preventDefault();
        if (i + 1 < ROWS) this.inputs[i + 1].focus();
        else this.close(true);
      });
      this.inputs.push(input);
      lines.append(input);
    }
    this.parts = PARTS.map((name, i) => button('secondary-btn sign-part', name, () => this.pick(i)));
    const swatches = h('div', 'sign-swatches');
    for (const rgb of SWATCHES) {
      const s = button('sign-swatch', '', () => this.set(rgb));
      s.style.background = hex(rgb);
      s.setAttribute('aria-label', hex(rgb));
      swatches.append(s);
    }
    this.wheel.onChange = (rgb) => this.set(rgb, false);
    const pickers = h('div', 'sign-colours');
    const side = h('div', 'sign-side');
    side.append(h('div', 'sign-parts'), swatches, button('secondary-btn', 'Done', () => this.close(true)));
    side.firstElementChild!.append(...this.parts);
    pickers.append(this.wheel.el, side);
    const body = h('div', 'mp-body');
    body.append(this.preview, lines, pickers);
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

  /** Opens the sign with a cell at a block; false (and nothing opens) when no sign stands there. */
  open(x: number, y: number, z: number): boolean {
    const colours = this.game.sign_colours(x, y, z);
    if (colours.length < 2) return false;
    this.at = [x, y, z];
    this.colours = [colours[0], colours[1]];
    const text = this.game.sign_text(x, y, z).split('\n');
    this.inputs.forEach((input, i) => (input.value = text[i] ?? ''));
    this.before = this.written();
    this.pick(0);
    this.backdrop.classList.remove('hidden');
    this.inputs[0].focus();
    return true;
  }

  /** Closes the editor, writing the sign first unless `write` is false. */
  close(resume: boolean, write = true): void {
    if (!this.isOpen) return;
    if (write && this.written() !== this.before) {
      const [x, y, z] = this.at;
      this.game.edit_sign(x, y, z, this.inputs.map((i) => i.value).join('\n'), this.colours[0], this.colours[1]);
    }
    this.backdrop.classList.add('hidden');
    this.onClose(resume);
  }

  /** Lets the wheel and the swatches set the board's colour (0) or the ink's (1). */
  private pick(part: number): void {
    this.part = part;
    this.parts.forEach((b, i) => b.classList.toggle('active', i === part));
    this.wheel.set(this.colours[part]);
    this.show();
  }

  private set(rgb: number, moveWheel = true): void {
    this.colours[this.part] = rgb;
    if (moveWheel) this.wheel.set(rgb);
    this.show();
  }

  /** Draws the preview board as the sign will look. */
  private show(): void {
    this.preview.style.background = hex(this.colours[0]);
    this.preview.style.color = hex(this.colours[1]);
    const lines = this.inputs.map((i) => i.value);
    let used = ROWS;
    while (used > 1 && lines[used - 1] === '') used--;
    this.preview.replaceChildren(...lines.slice(0, used).map((l) => h('div', 'sign-preview-line', l || ' ')));
  }

  private written(): string {
    return [...this.inputs.map((i) => i.value), ...this.colours].join('\n');
  }
}
