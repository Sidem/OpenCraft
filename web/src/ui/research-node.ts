// One tech on the research screen: the node on the tree (name, a few icons of what it unlocks, its state and a
// progress bar) and the hover card with the full story (research.ts places the nodes and shows the card).
// `makeTech` builds both from the engine's `tech_*` data; `paintTech` sets their state from the engine each time
// something changed. Item icons are canvases from the renderer's icon cache, drawn small.

import './research-node.css';
import type { Game } from '../wasm/engine.js';
import { h } from './dom';

const ICON_PX = 64;
const NODE_ICONS = 3;

export interface TechView {
  node: HTMLButtonElement;
  nodeState: HTMLSpanElement;
  nodeBar: HTMLElement;
  /** The hover card. */
  card: HTMLDivElement;
  state: HTMLSpanElement;
  bar: HTMLDivElement;
  count: HTMLSpanElement;
  /** The lab time of the next unit (it grows with an endless tech's level). */
  seconds: HTMLSpanElement;
  action: HTMLParagraphElement;
  /** Lower-case words the search matches: the name and what the tech unlocks. */
  words: string;
}

export interface TechState {
  done: boolean;
  available: boolean;
  /** Being researched now. */
  chosen: boolean;
  /** Place in the queue, from 1, or 0 when not queued. */
  place: number;
  progress: number;
  units: number;
  endless: boolean;
  seconds: number;
  /** "Needs X first." for a locked tech. */
  needs: string;
}

export function itemIcon(icon: (item: number) => HTMLCanvasElement, item: number, cls: string): HTMLCanvasElement {
  const c = h('canvas', cls) as HTMLCanvasElement;
  c.width = c.height = ICON_PX;
  c.getContext('2d')!.drawImage(icon(item), 0, 0);
  return c;
}

export function makeTech(game: Game, t: number, icon: (item: number) => HTMLCanvasElement): TechView {
  const g = game;
  const unlocked = Array.from(g.tech_unlocks(t));
  const node = h('button', 'rs-node') as HTMLButtonElement;
  node.type = 'button';
  node.setAttribute('aria-label', g.tech_name(t));
  const unlocks = h('span', 'rs-unlocks');
  for (const item of unlocked.slice(0, NODE_ICONS)) {
    const c = itemIcon(icon, item, 'rs-pack');
    c.title = g.item_name(item);
    unlocks.append(c);
  }
  if (unlocked.length > NODE_ICONS) unlocks.append(h('span', 'rs-more', `+${unlocked.length - NODE_ICONS}`));
  const nodeState = h('span', 'rs-node-state');
  const meta = h('span', 'rs-node-meta');
  meta.append(unlocks, nodeState);
  const track = h('span', 'rs-node-track');
  const nodeBar = h('i', '');
  track.append(nodeBar);
  node.append(h('span', 'rs-node-name', g.tech_name(t)), meta, track);

  const card = h('div', 'rs-card');
  const title = h('div', 'rs-title');
  const state = h('span', 'rs-state');
  title.append(h('h4', '', g.tech_name(t)), state);
  const chips = h('div', 'rs-chips');
  chips.append(h('span', 'rs-label', 'Unlocks'));
  for (const item of unlocked) chips.append(chip(icon, item, g.item_name(item)));
  const cost = h('div', 'rs-chips');
  cost.append(h('span', 'rs-label', 'Each unit'));
  for (const item of g.tech_packs(t)) cost.append(chip(icon, item, `1 ${g.item_name(item)}`));
  const seconds = h('span', 'rs-label', `${g.tech_seconds(t)} s in a lab`);
  cost.append(seconds);
  const progress = h('div', 'rs-progress');
  const barBox = h('div', 'mp-bar');
  const bar = h('div', 'mp-bar-fill');
  barBox.append(bar);
  const count = h('span', 'rs-count');
  progress.append(barBox, count);
  const action = h('p', 'rs-action');
  card.append(title, h('p', 'rs-blurb', g.tech_blurb(t)), chips, cost, progress, action);
  const words = [g.tech_name(t), ...unlocked.map((i) => g.item_name(i))].join(' ').toLowerCase();
  return { node, nodeState, nodeBar, card, state, bar, count, seconds, action, words };
}

/** Sets the node's and the card's look and words from the tech's current state. */
export function paintTech(v: TechView, s: TechState): void {
  const amount = s.endless ? `level ${s.progress}` : `${s.progress} of ${s.units}`;
  const kind = s.done ? 'done' : s.available ? 'available' : 'locked';
  const label = s.done ? 'Done' : s.chosen ? 'Researching' : s.place > 0 ? `Queued #${s.place}` : s.available ? 'Available' : 'Locked';
  const extra = `${s.chosen ? ' chosen' : ''}${s.place > 0 ? ' queued' : ''}`;
  v.node.className = `rs-node ${kind}${extra}`;
  v.card.className = `rs-card ${kind}${extra}`;
  const started = s.chosen || (s.available && s.progress > 0);
  v.nodeState.textContent = started ? amount : s.place > 0 ? label : s.done ? 'Done' : s.endless ? 'endless' : `${s.units} units`;
  v.state.textContent = label;
  const fill = `scaleX(${s.endless ? 0 : s.progress / s.units})`;
  v.nodeBar.style.transform = fill;
  v.bar.style.transform = fill;
  v.count.textContent = s.endless ? `${amount} · endless` : `${amount} units`;
  v.seconds.textContent = `${Math.round(s.seconds)} s in a ${s.endless ? 'AI lab, using compute' : 'lab'}`;
  v.action.textContent = s.done
    ? 'Researched: what comes after it builds on this.'
    : s.chosen
      ? 'Click to give this up and move on to the next in the queue.'
      : s.place > 0
        ? 'Click to take this out of the queue.'
        : s.available
          ? 'Click to research this now, Shift-click to add it to the queue.'
          : `${s.needs} Click to queue it with everything it needs.`;
}

function chip(icon: (item: number) => HTMLCanvasElement, item: number, text: string): HTMLSpanElement {
  const el = h('span', 'chip');
  el.append(itemIcon(icon, item, 'chip-icon'), text);
  return el;
}
