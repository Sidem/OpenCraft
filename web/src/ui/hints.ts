// Onboarding hints: one short tip at a time in the HUD, in the engine's order (`hint_text`). The engine
// says how far the player got (`hint_progress`), so a tip goes away by itself once it's done. H skips the
// tip on screen. Skipped tips are UI state kept in localStorage (per browser, not per world); the menu's
// controls page gets a "Show tips again" button once any were skipped. New tips are engine rows (`hints.rs`).
// Tip text is drawn to be skimmed: a lead-in before the first colon becomes the title, `*words*` are
// highlighted and `[keys]` drawn as key caps (`richText`).
// A new world's opening scene comes first (`opening_step`, `opening.rs`): its tips replace the list until it
// is done, a floating prompt points at what to do (`key-prompt.ts`), the card cheers briefly each time one of
// its steps is done, and `onLineRuns` fires when its line starts again.
// Hand mining is taught as it happens (`hand_waste.rs`): each ore block broken by hand flashes what was kept and
// lost under the crosshair, and after a few a warning card with the totals jumps ahead of every tip (the opening's
// too) while the player keeps at it (H snoozes it for `SNOOZE_BLOCKS` more). Creative worlds skip both.
// Building the same machine a third time nearby brings the blueprint prompt (`blueprint_prompt`,
// `blueprint/repeat.rs`) ahead of the tips; the engine ends it, and H dismisses it. A machine that has stood stalled near
// the player brings its lesson (`lesson_now`, `stall_lesson.rs`): the card names the cause and the fix, a prompt hangs over
// the machine, the engine ends it when the machine works again, and H teaches it away for the world. Side tips
// (`hint_side`: towns, once Settlements is researched) come next, ahead of the main order, until done or skipped.

import './hints.css';
import type { Game } from '../wasm/engine.js';
import type { Renderer } from '../render/renderer';
import { button, h } from './dom';
import { KeyPrompt } from './key-prompt';

const STORAGE_KEY = 'opencraft.hints.skipped';
/** `shown` and the skipped set use this for the opening scene's tips (-1 is no tip). */
const OPENING = -2;
/** `shown` for the hand-mining warning. */
const WASTE = -3;
/** `shown` for the blueprint prompt. */
const REPEAT = -4;
/** `shown` for a machine-stopped lesson. */
const LESSON = -5;
/** A lead-in this short before the first ": " is the tip's title. */
const TITLE_MAX = 40;
/** The hand-mining warning stays this long after the latest ore block broken by hand. */
const WASTE_MS = 45_000;
/** Skipping the warning hides it until this many more ore blocks are broken by hand. */
const SNOOZE_BLOCKS = 10;
export class Hints {
  private readonly el = h('div', 'hint hidden');
  private readonly counter = h('span', 'hint-count');
  private readonly text = h('div', 'hint-text');
  private readonly prompt = new KeyPrompt();
  private readonly again: HTMLButtonElement;
  private readonly skipped: Set<number>;
  private shown = -1;
  private openingStep = 0;
  /** The lesson on the card (`lesson_now`'s number), so another cause redraws it. */
  private lesson = 0;
  /** Whether the opening's line ran last frame (null before the first, so loading a world never cheers). */
  private running: boolean | null = null;
  private readonly notice = h('div', 'waste-notice');
  private readonly creative: boolean;
  /** Ore blocks broken by hand as last seen, when the latest was, and the count the warning is snoozed until. */
  private mined = 0;
  private minedAt = -Infinity;
  private snoozed = 0;
  private wasteText = '';

  constructor(
    private readonly game: Game,
    private readonly renderer: Renderer,
    private readonly onLineRuns: () => void = () => {},
  ) {
    this.skipped = loadSkipped();
    this.creative = game.is_creative();
    document.getElementById('hud')!.append(this.notice);
    this.notice.addEventListener('animationend', () => this.notice.classList.remove('waste-show'));
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
    if (this.shown === WASTE) {
      this.snoozed = this.mined + SNOOZE_BLOCKS;
      this.update();
      return;
    }
    if (this.shown === REPEAT) {
      this.game.blueprint_prompt_dismiss();
      this.update();
      return;
    }
    if (this.shown === LESSON) {
      this.game.lesson_dismiss();
      this.update();
      return;
    }
    this.skipped.add(this.shown);
    saveSkipped(this.skipped);
    this.update();
  }

  /** Call every frame, after rendering; redraws the card only when the tip changes. */
  update(): void {
    const g = this.game;
    this.again.classList.toggle('hidden', this.skipped.size === 0);
    const step = this.skipped.has(OPENING) ? 0 : g.opening_step();
    const marker = step > 0 ? g.opening_marker() : this.shown === LESSON ? g.lesson_marker() : [];
    this.prompt.update(marker, this.renderer);
    const running = step > 0 && g.opening_runs();
    if (running && this.running === false) this.onLineRuns();
    this.running = running;
    const waste = this.wasteTip();
    if (waste) {
      if (this.shown !== WASTE) this.show(WASTE, 'Heads up', waste);
      else if (waste !== this.wasteText) this.draw(waste); // the totals grew: no new animation
      this.wasteText = waste;
      return;
    }
    const repeat = g.blueprint_prompt();
    if (repeat) {
      if (this.shown !== REPEAT) this.show(REPEAT, 'Save some work', repeat);
      return;
    }
    if (step > 0) {
      if (this.shown === OPENING && step === this.openingStep) return;
      this.show(OPENING, 'Getting started', g.opening_text(step));
      this.openingStep = step;
      return;
    }
    // A machine that has stood stalled near the player: its lesson goes ahead of the side and main tips.
    const lesson = g.lesson_now();
    if (lesson) {
      if (this.shown !== LESSON || lesson !== this.lesson) this.show(LESSON, 'Machine stopped', g.lesson_text());
      this.lesson = lesson;
      return;
    }
    // A side tip (towns, once researched) comes ahead of the main order.
    const side = [...g.hint_side()].find((i) => !this.skipped.has(i));
    if (side !== undefined) {
      if (side !== this.shown) this.show(side, 'New to try', g.hint_text(side));
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

  /** Flashes the latest hand-mined block's loss, and returns the warning card's text while it should show. */
  private wasteTip(): string {
    const g = this.game;
    if (this.creative) return '';
    const mined = g.hand_mined_blocks();
    if (mined !== this.mined) {
      this.mined = mined;
      this.minedAt = performance.now();
      this.notice.replaceChildren();
      richText(this.notice, g.hand_waste_notice());
      this.notice.classList.remove('waste-show');
      void this.notice.offsetWidth; // restarts the animation
      this.notice.classList.add('waste-show');
    }
    if (mined < this.snoozed || performance.now() - this.minedAt > WASTE_MS) return '';
    return g.hand_waste_tip();
  }

  private show(tip: number, head: string, text: string): void {
    // The card the player just finished (an opening step, a stalled machine mended) is followed by a cheer.
    const stepDone = (this.shown === OPENING || this.shown === LESSON) && tip !== WASTE && !this.skipped.has(OPENING);
    this.shown = tip;
    this.el.classList.toggle('hidden', tip === -1);
    if (tip === -1) return;
    this.counter.textContent = head;
    this.el.classList.toggle('hint-warn', tip === WASTE);
    this.draw(text);
    this.el.classList.remove('hint-cheer', 'hint-new');
    void this.el.offsetWidth; // restarts the animation
    this.el.classList.add(stepDone ? 'hint-cheer' : 'hint-new');
  }

  /** The card's title and body from a tip's text. */
  private draw(text: string): void {
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
