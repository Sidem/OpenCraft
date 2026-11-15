// The keyboard picture on the Controls page: a cap for every place in keyboard-geometry.ts, printed with the player's labels
// (keyboard-layout.ts). A key the game uses is a button in its category's colour; the rest are dim caps. The page (controls.ts)
// says what to light and what a hover or focus does; this file only draws. To change the look: keyboard-view.css.

import './keyboard-view.css';
import { CATEGORIES, categoryOf } from './control-bindings';
import { h } from './dom';
import { KB_HEIGHT, KB_WIDTH, KEY_PLACES } from './keyboard-geometry';
import type { KeyLabels } from './keyboard-layout';

export interface KeyHooks {
  /** The pointer or focus is on `code` (null: left it). */
  point(code: string | null): void;
  /** `code` was clicked or pressed. */
  pick(code: string): void;
  /** The words a screen reader says for `code`. */
  describe(code: string): string;
}

export class KeyboardView {
  readonly el = h('div', 'kb-wrap');
  private readonly caps = new Map<string, HTMLElement>();

  constructor(private readonly labels: KeyLabels, private readonly hooks: KeyHooks) {
    const board = h('div', 'kb');
    board.style.setProperty('--cols', String(KB_WIDTH));
    board.style.setProperty('--rows', String(KB_HEIGHT));
    for (const place of KEY_PLACES) {
      const cat = categoryOf(place.code);
      const cap = h(cat ? 'button' : 'div', cat ? 'kb-key' : 'kb-key kb-unused');
      cap.style.setProperty('--x', String(place.x));
      cap.style.setProperty('--y', String(place.y));
      cap.style.setProperty('--w', String(place.w));
      if (cat) {
        cap.style.setProperty('--cat', CATEGORIES.find((c) => c.id === cat)!.colour);
        cap.dataset.cat = cat;
        cap.addEventListener('pointerenter', () => hooks.point(place.code));
        cap.addEventListener('pointerleave', () => hooks.point(null));
        cap.addEventListener('focus', () => hooks.point(place.code));
        cap.addEventListener('blur', () => hooks.point(null));
        cap.addEventListener('click', () => hooks.pick(place.code));
      }
      this.caps.set(place.code, cap);
      board.append(cap);
    }
    this.el.append(board);
    this.sync();
  }

  /** Prints the current labels on the caps. */
  sync(): void {
    for (const [code, cap] of this.caps) {
      const label = this.labels.label(code);
      cap.textContent = label;
      cap.classList.toggle('kb-long', label.length > 2);
      if (cap instanceof HTMLButtonElement) cap.setAttribute('aria-label', `${label} key: ${this.hooks.describe(code)}`);
    }
  }

  has(code: string): boolean {
    return this.caps.has(code);
  }

  /** Rings these keys (an empty set: none). */
  light(codes: ReadonlySet<string>): void {
    for (const [code, cap] of this.caps) cap.classList.toggle('kb-hot', codes.has(code));
  }

  /** Shows a key as held down. */
  press(code: string, down: boolean): void {
    this.caps.get(code)?.classList.toggle('kb-pressed', down);
  }

  /** Fades the keys outside one category (null: none). */
  filter(category: string | null): void {
    for (const cap of this.caps.values()) cap.classList.toggle('kb-dim', category !== null && cap.dataset.cat !== category);
  }
}
