// Onboarding hints: one short tip at a time in the HUD, in the engine's order (`hint_text`). The engine
// says how far the player got (`hint_progress`), so a tip goes away by itself once it's done. H skips the
// tip on screen. Skipped tips are UI state kept in localStorage (per browser, not per world); the pause
// menu gets a "Show tips again" button once any were skipped. New tips are engine rows (`hints.rs`).

import './hints.css';
import type { Game } from '../wasm/engine.js';
import { button, h } from './dom';

const STORAGE_KEY = 'opencraft.hints.skipped';

export class Hints {
  private readonly el = h('div', 'hint hidden');
  private readonly counter = h('span', 'hint-count');
  private readonly text = h('p', 'hint-text');
  private readonly again: HTMLButtonElement;
  private readonly skipped: Set<number>;
  private shown = -1;

  constructor(private readonly game: Game) {
    this.skipped = loadSkipped();
    const head = h('div', 'hint-head');
    head.append(this.counter, h('span', 'hint-key', 'H: skip'));
    this.el.append(head, this.text);
    document.getElementById('hud')!.append(this.el);
    this.again = button('secondary-btn hidden', 'Show tips again', () => {
      this.skipped.clear();
      saveSkipped(this.skipped);
      this.update();
    });
    document.querySelector('#menu .sound-row')?.append(this.again);
  }

  /** Skips the tip on screen for good. */
  skip(): void {
    if (this.shown < 0) return;
    this.skipped.add(this.shown);
    saveSkipped(this.skipped);
    this.update();
  }

  /** Call every frame; redraws only when the tip changes. */
  update(): void {
    const g = this.game;
    const count = g.hint_count();
    let tip = -1;
    for (let i = g.hint_progress(); i < count; i++) {
      if (!this.skipped.has(i)) {
        tip = i;
        break;
      }
    }
    this.again.classList.toggle('hidden', this.skipped.size === 0);
    if (tip === this.shown) return;
    this.shown = tip;
    this.el.classList.toggle('hidden', tip < 0);
    if (tip < 0) return;
    this.counter.textContent = `Tip ${tip + 1} of ${count}`;
    this.text.textContent = g.hint_text(tip);
  }
}

// Storage can be unavailable (private windows, blocked site data); tips then just aren't remembered.
function loadSkipped(): Set<number> {
  try {
    const raw: unknown = JSON.parse(localStorage.getItem(STORAGE_KEY) ?? '[]');
    return new Set(Array.isArray(raw) ? raw.filter((n): n is number => typeof n === 'number') : []);
  } catch {
    return new Set();
  }
}

function saveSkipped(skipped: Set<number>): void {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify([...skipped]));
  } catch {
    // Not remembered; see loadSkipped.
  }
}
