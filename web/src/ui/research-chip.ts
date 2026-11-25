// One tech on the research grid (`research-grid.ts`): a chip (icon, name, and one line of what matters now) for a
// wide ring column, and a dot for a folded one. State colours carry one meaning each: available glows blue,
// researching is orange, done recedes to a quiet green tick, a "next" teaser is faint and dashed.
// To change what a chip says in a state: `paintChip`.

import './research-chip.css';
import type { Game } from '../wasm/engine.js';
import { h } from './dom';
import type { Seen } from './research-frontier';
import { glyphSvg, kindOf, KINDS } from './research-kinds';

const ICONS = 4;
const ICON_PX = 64;

export interface ChipState {
  seen: Seen;
  done: boolean;
  available: boolean;
  /** Being researched now. */
  chosen: boolean;
  /** Place in the queue from 1, or 0. */
  place: number;
  progress: number;
  units: number;
  endless: boolean;
  /** A deed's job ("Lay 40 belts"), or empty for a lab tech. */
  deed: string;
  deedProgress: number;
  deedTarget: number;
  /** The first missing need's name, for a locked tech. */
  need: string;
}

export interface Chip {
  tech: number;
  el: HTMLButtonElement;
  dot: HTMLSpanElement;
  sub: HTMLSpanElement;
  side: HTMLSpanElement;
  icons: HTMLSpanElement;
  bar: HTMLDivElement;
  /** Lower-case words the search matches: name and unlocked items. */
  words: string;
}

/** An item's icon as its own small canvas (`cls` sizes it). */
export function itemIcon(icon: (item: number) => HTMLCanvasElement, item: number, cls: string): HTMLCanvasElement {
  const c = h('canvas', cls) as HTMLCanvasElement;
  c.width = c.height = ICON_PX;
  c.getContext('2d')!.drawImage(icon(item), 0, 0);
  return c;
}

/** The kind's line icon in a round medal (`--kind` colours it). */
export function medal(name: string, cls: string): HTMLSpanElement {
  const kind = kindOf(name);
  const el = h('span', cls);
  el.innerHTML = glyphSvg(kind);
  el.style.setProperty('--kind', KINDS[kind].colour);
  return el;
}

export function makeChip(game: Game, t: number, icon: (item: number) => HTMLCanvasElement): Chip {
  const name = game.tech_name(t);
  const el = h('button', 'rc') as HTMLButtonElement;
  el.type = 'button';
  el.dataset.tech = String(t);
  const main = h('span', 'rc-main');
  const sub = h('span', 'rc-sub');
  const icons = h('span', 'rc-icons');
  const unlocks = Array.from(game.tech_unlocks(t));
  for (const item of unlocks.slice(0, ICONS)) {
    const c = itemIcon(icon, item, 'rc-icon');
    c.title = game.item_name(item);
    icons.append(c);
  }
  if (unlocks.length > ICONS) icons.append(h('span', 'rc-more', `+${unlocks.length - ICONS}`));
  main.append(h('span', 'rc-name', name), sub);
  const side = h('span', 'rc-side');
  const track = h('div', 'rc-bar');
  const bar = h('div', 'rc-fill');
  track.append(bar);
  el.append(medal(name, 'rc-medal'), main, side, track);
  const dot = h('span', 'rc-dot');
  dot.dataset.tech = String(t);
  dot.title = name;
  const words = [name, ...unlocks.map((i) => game.item_name(i))].join(' ').toLowerCase();
  return { tech: t, el, dot, sub, side, icons, bar, words };
}

/** Sets the chip's and the dot's look and words from the tech's state. */
export function paintChip(c: Chip, s: ChipState): void {
  const state = s.done ? 'done' : s.chosen ? 'chosen' : s.place > 0 ? 'queued' : s.available ? 'available' : 'locked';
  const cls = `${state}${s.seen === 'next' ? ' next' : ''}${s.deed !== '' ? ' deed' : ''}`;
  c.el.className = `rc ${cls}`;
  c.dot.className = `rc-dot ${cls}`;
  c.el.classList.toggle('hidden', s.seen === 'hidden');
  c.dot.classList.toggle('hidden', s.seen === 'hidden');
  let share = 0;
  if (s.deed !== '') {
    const target = Math.max(1, s.deedTarget);
    share = s.done ? 1 : s.deedProgress / target;
    c.sub.replaceChildren(s.done ? c.icons : s.deed);
    c.side.textContent = s.done ? '✓' : target > 1 ? `${s.deedProgress}/${target}` : '';
  } else {
    share = s.done ? 1 : s.endless ? 0 : s.progress / Math.max(1, s.units);
    c.sub.replaceChildren(s.available || s.done || s.chosen || s.place > 0 ? c.icons : `Needs ${s.need}`);
    c.side.textContent = s.done
      ? s.endless ? `Lv ${s.progress}` : '✓'
      : s.chosen
        ? s.endless ? `Lv ${s.progress}` : `${s.progress}/${s.units}`
        : s.place > 0 ? `#${s.place}` : '';
  }
  c.bar.style.transform = `scaleX(${s.done ? 0 : share})`;
  c.el.setAttribute('aria-label', `${c.el.querySelector('.rc-name')!.textContent}, ${state}`);
}
