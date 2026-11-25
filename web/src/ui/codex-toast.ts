// Codex unlock popups: a card that slides in at the top left when the world's Codex records a new element, completes
// one (every form obtained) or unlocks an achievement, with a chime (audio/synth-chime.ts). Elements show their
// periodic-table tile in their category colour; achievements a gold medal with the achievement's icon, light rays and
// sparks. Clicking a card opens the Codex on it. Everything is read from the engine (`codex_*`): each frame
// `update` checks the cheap `codex_obtained_count` and, only when it moved, compares elements and the achievement
// mask with what it saw last. What is already known when the world loads never pops up.
// To add a kind of popup: a `Pop` variant, its card in `card`, its chime in `chime`.

import './codex.css';
import './codex-toast.css';
import './codex-toast-art.css';
import type { Game } from '../wasm/engine.js';
import { CATEGORY_LABELS, ELEMENT_BY_Z } from './codex-elements';
import { h } from './dom';

/** Cards on screen at once; the rest wait their turn. */
const MAX_SHOWN = 3;
/** Waiting element cards past this many fold into one "and N more" card. */
const MAX_WAITING = 4;
/** How long a card stays, in ms (hovering pauses it). */
const HOLD_MS = { element: 5200, complete: 5200, more: 5200, achievement: 7500 };
/** Chimes closer together than this play once. */
const CHIME_GAP_MS = 350;
const ICON_PX = 64;
const SPARKS = 10;

interface ElementPop {
  kind: 'element';
  z: number;
  done: number;
  total: number;
}
interface MorePop {
  kind: 'more';
  zs: number[];
}
type CompletePop = { kind: 'complete'; z: number; total: number };
type Pop = ElementPop | MorePop | CompletePop | { kind: 'achievement'; index: number };

/** What a click on a card shows in the Codex. */
export type CodexTarget = { z: number } | { achievement: number };

export class CodexToasts {
  /** Opens the Codex on an element or an achievement. */
  onOpen: (target: CodexTarget) => void = () => {};
  /** Plays a chime. */
  onChime: (kind: 'discover' | 'achieve') => void = () => {};

  private readonly stack = h('div', 'cxt-stack');
  private readonly waiting: Pop[] = [];
  /** Codex version last compared (-1: not yet, so the first look only records). */
  private seen = -1;
  private forms = new Map<number, number>();
  private mask = 0;
  private lastChime = 0;

  constructor(
    private readonly game: Game,
    private readonly icon: (item: number) => HTMLCanvasElement,
  ) {
    this.stack.setAttribute('aria-live', 'polite');
    document.getElementById('hud-left')!.append(this.stack);
  }

  /** Call every frame. */
  update(): void {
    const g = this.game;
    const version = g.codex_obtained_count();
    if (version === this.seen) return;
    const first = this.seen < 0;
    this.seen = version;
    const raw = g.codex_reachable_elements();
    const mask = g.codex_unlocked_mask();
    const found: Pop[] = [];
    for (let i = 0; i < raw.length; i += 3) {
      const [z, done, total] = [raw[i], raw[i + 1], raw[i + 2]];
      const before = this.forms.get(z) ?? 0;
      this.forms.set(z, done);
      if (first || done === before) continue;
      if (before === 0) found.push({ kind: 'element', z, done, total });
      else if (done >= total && total > 1) found.push({ kind: 'complete', z, total });
    }
    for (let i = 0; i < 32; i++) {
      if (!first && mask & (1 << i) && !(this.mask & (1 << i))) found.push({ kind: 'achievement', index: i });
    }
    this.mask = mask;
    // Achievements first: they are rarer and carry the bigger moment.
    found.sort((a, b) => Number(b.kind === 'achievement') - Number(a.kind === 'achievement'));
    this.waiting.push(...found);
    this.fold();
    this.next();
  }

  /** Folds element cards waiting past `MAX_WAITING` into one "and N more" card: a big haul doesn't take a minute. */
  private fold(): void {
    const elements = this.waiting.filter((p): p is ElementPop => p.kind === 'element');
    if (elements.length <= MAX_WAITING) return;
    const extra = new Set<Pop>(elements.slice(MAX_WAITING - 1));
    const zs = elements.slice(MAX_WAITING - 1).map((p) => p.z);
    const kept = this.waiting.filter((p) => !extra.has(p));
    const more = kept.find((p): p is MorePop => p.kind === 'more');
    if (more) more.zs.push(...zs);
    else kept.push({ kind: 'more', zs });
    this.waiting.splice(0, this.waiting.length, ...kept);
  }

  /** Shows waiting cards while there is room. */
  private next(): void {
    while (this.stack.children.length < MAX_SHOWN && this.waiting.length > 0) this.show(this.waiting.shift()!);
  }

  private show(pop: Pop): void {
    const el = this.card(pop);
    const timer = h('i', 'cxt-timer');
    timer.style.animationDuration = `${HOLD_MS[pop.kind]}ms`;
    timer.addEventListener('animationend', () => this.leave(el));
    el.append(timer);
    el.addEventListener('click', () => {
      if (pop.kind === 'achievement') this.onOpen({ achievement: pop.index });
      else this.onOpen({ z: pop.kind === 'more' ? pop.zs[0] : pop.z });
      this.leave(el);
    });
    this.stack.append(el);
    const now = performance.now();
    if (now - this.lastChime > CHIME_GAP_MS) {
      this.lastChime = now;
      this.onChime(pop.kind === 'achievement' ? 'achieve' : 'discover');
    }
  }

  private leave(el: HTMLElement): void {
    if (el.classList.contains('cxt-leaving')) return;
    el.classList.add('cxt-leaving');
    const gone = () => {
      el.remove();
      this.next();
    };
    el.addEventListener('animationend', (e) => e.target === el && gone());
    // Without animations (reduced motion, a hidden tab) the end may never come.
    setTimeout(gone, 600);
  }

  private card(pop: Pop): HTMLElement {
    const g = this.game;
    const [found, reachable, unlocked, achievements] = g.codex_summary();
    if (pop.kind === 'achievement') {
      const [title, desc, icon] = g.codex_achievement(pop.index).split('\n');
      const el = h('button', 'cxt cxt-achievement');
      const wrap = h('div', 'cxt-medal-wrap');
      const medal = h('div', 'cxt-medal');
      medal.append(this.iconCanvas(Number(icon) || 0));
      wrap.append(h('span', 'cxt-rays'), medal);
      for (let i = 0; i < SPARKS; i++) {
        const spark = h('span', 'cxt-spark');
        spark.style.setProperty('--a', `${(360 / SPARKS) * i + 9}deg`);
        spark.style.setProperty('--d', `${(i % 3) * 70}ms`);
        wrap.append(spark);
      }
      el.append(wrap, body('Achievement unlocked', title, desc, `${unlocked} of ${achievements} achievements`));
      return el;
    }
    const elements = `${found} of ${reachable} elements`;
    if (pop.kind === 'more') {
      const el = elementCard(pop.zs[0], '');
      const tiles = h('div', 'cxt-minis');
      for (const z of pop.zs.slice(0, 6)) tiles.append(tile(z, 'cxt-mini'));
      const names = pop.zs.map((z) => ELEMENT_BY_Z.get(z)?.name ?? '').join(', ');
      el.append(tiles, body('New elements', `And ${pop.zs.length} more`, names, elements));
      return el;
    }
    const info = ELEMENT_BY_Z.get(pop.z);
    const complete = pop.kind === 'complete';
    const el = elementCard(pop.z, complete ? ' cxt-complete' : '');
    const forms = `${pop.kind === 'element' ? pop.done : pop.total} of ${pop.total} form${pop.total === 1 ? '' : 's'}`;
    const cat = info ? CATEGORY_LABELS[info.cat] : 'Element';
    const sub = complete ? `All ${pop.total} forms recorded` : `${cat} · ${forms}`;
    const name = info?.name ?? `Element ${pop.z}`;
    el.append(tile(pop.z, 'cxt-tile'), body(complete ? 'Element complete' : 'New element', name, sub, elements));
    return el;
  }

  private iconCanvas(item: number): HTMLCanvasElement {
    const c = h('canvas', 'cxt-icon') as HTMLCanvasElement;
    c.width = c.height = ICON_PX;
    c.getContext('2d')!.drawImage(this.icon(item), 0, 0);
    return c;
  }
}

/** A card that glows in element `z`'s category colour. */
function elementCard(z: number, extra: string): HTMLElement {
  const el = h('button', `cxt cxt-element cx-cat${extra}`);
  const cat = ELEMENT_BY_Z.get(z)?.cat;
  if (cat) el.dataset.cat = cat;
  return el;
}

/** An element's periodic-table tile in its category colour, with a sweep of light across it. */
function tile(z: number, cls: string): HTMLElement {
  const info = ELEMENT_BY_Z.get(z);
  const el = h('div', `${cls} cx-cat`);
  if (info) el.dataset.cat = info.cat;
  el.append(h('span', 'cxt-z', String(z)), h('span', 'cxt-sym', info?.symbol ?? '?'), h('span', 'cxt-shine'));
  return el;
}

/** The words of a card: a small coloured kicker, the title, one line under it and the Codex's count. */
function body(kicker: string, title: string, sub: string, count: string): HTMLElement {
  const el = h('div', 'cxt-body');
  el.append(h('span', 'cxt-kicker', kicker), h('span', 'cxt-title', title), h('span', 'cxt-sub', sub));
  el.append(h('span', 'cxt-count', count));
  return el;
}
