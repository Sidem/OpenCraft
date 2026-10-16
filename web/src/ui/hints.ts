// Onboarding hints: one short tip at a time in the HUD, in the engine's order (`hint_text`). The engine
// says how far the player got (`hint_progress`), so a tip goes away by itself once it's done. H skips the
// tip on screen. Skipped tips are UI state kept in localStorage (per browser, not per world); the menu's
// controls page gets a "Show tips again" button once any were skipped. New tips are engine rows (`hints.rs`).
// Tip text is drawn to be skimmed: a lead-in before the first colon becomes the title, `*words*` are
// highlighted and `[keys]` drawn as key caps (`richText`).
// A new world's opening scene comes first (`opening_step`, `opening.rs`): its tips replace the list until it
// is done, a floating prompt points at what to do (`key-prompt.ts`), the card cheers briefly each time one of
// its steps is done, and `onLineRuns` fires when its line starts again.

import './hints.css';
import type { Game } from '../wasm/engine.js';
import type { Renderer } from '../render/renderer';
import { button, h } from './dom';
import { KeyPrompt } from './key-prompt';

const STORAGE_KEY = 'opencraft.hints.skipped';
/** `shown` and the skipped set use this for the opening scene's tips (-1 is no tip). */
const OPENING = -2;
/** A lead-in this short before the first ": " is the tip's title. */
const TITLE_MAX = 40;

export class Hints {
  private readonly el = h('div', 'hint hidden');
  private readonly counter = h('span', 'hint-count');
  private readonly text = h('div', 'hint-text');
  private readonly prompt = new KeyPrompt();
  private readonly again: HTMLButtonElement;
  private readonly skipped: Set<number>;
  private shown = -1;
  private openingStep = 0;
  /** Whether the opening's line ran last frame (null before the first, so loading a world never cheers). */
  private running: boolean | null = null;

  constructor(
    private readonly game: Game,
    private readonly renderer: Renderer,
    private readonly onLineRuns: () => void = () => {},
  ) {
    this.skipped = loadSkipped();
    const head = h('div', 'hint-head');
    const key = h('span', 'hint-key');
    richText(key, '[H] skip');
    head.append(this.counter, key);
    this.el.append(head, this.text);
    this.el.addEventListener('animationend', () => this.el.classList.remove('hint-cheer'));
    document.getElementById('hud-left')!.append(this.el);
    this.again = button('secondary-btn hidden', 'Show tips again', () => {
      this.skipped.clear();
      saveSkipped(this.skipped);
      this.update();
    });
    document.getElementById('menu-tips')?.append(this.again);
  }

  /** Skips the tip on screen for good. */
  skip(): void {
    if (this.shown === -1) return;
    this.skipped.add(this.shown);
    saveSkipped(this.skipped);
    this.update();
  }

  /** Call every frame, after rendering; redraws the card only when the tip changes. */
  update(): void {
    const g = this.game;
    this.again.classList.toggle('hidden', this.skipped.size === 0);
    const step = this.skipped.has(OPENING) ? 0 : g.opening_step();
    this.prompt.update(step > 0 ? g.opening_marker() : [], this.renderer);
    const running = step > 0 && g.opening_runs();
    if (running && this.running === false) this.onLineRuns();
    this.running = running;
    if (step > 0) {
      if (this.shown === OPENING && step === this.openingStep) return;
      this.show(OPENING, 'Getting started', g.opening_text(step));
      this.openingStep = step;
      return;
    }
    const count = g.hint_count();
    let tip = -1;
    for (let i = g.hint_progress(); i < count; i++) {
      if (!this.skipped.has(i)) {
        tip = i;
        break;
      }
    }
    if (tip === this.shown) return;
    this.show(tip, `Tip ${tip + 1} of ${count}`, tip < 0 ? '' : g.hint_text(tip));
  }

  private show(tip: number, head: string, text: string): void {
    const stepDone = this.shown === OPENING && !this.skipped.has(OPENING);
    this.shown = tip;
    this.el.classList.toggle('hidden', tip === -1);
    if (tip === -1) return;
    this.counter.textContent = head;
    this.text.replaceChildren();
    const lead = text.indexOf(': ');
    if (lead > 0 && lead <= TITLE_MAX && !/[*[]/.test(text.slice(0, lead))) {
      this.text.append(h('div', 'hint-title', text.slice(0, lead)));
      text = text.slice(lead + 2);
      text = text.charAt(0).toUpperCase() + text.slice(1);
    }
    const body = h('p', 'hint-body');
    richText(body, text);
    this.text.append(body);
    this.el.classList.remove('hint-cheer', 'hint-new');
    void this.el.offsetWidth; // restarts the animation
    this.el.classList.add(stepDone ? 'hint-cheer' : 'hint-new');
  }
}

/** Appends `text` to `el`, with `*words*` highlighted and `[keys]` as key caps. Never parses HTML. */
function richText(el: HTMLElement, text: string): void {
  const marks = /\*([^*]+)\*|\[([^\]]+)\]/g;
  let at = 0;
  for (const m of text.matchAll(marks)) {
    if (m.index > at) el.append(text.slice(at, m.index));
    el.append(m[1] !== undefined ? h('strong', 'hint-em', m[1]) : h('kbd', 'hint-cap', m[2]));
    at = m.index + m[0].length;
  }
  if (at < text.length) el.append(text.slice(at));
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
