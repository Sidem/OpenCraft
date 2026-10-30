// The item guide: one page per item that answers "how do I get it?" and "what is it for?", shown over the build menu
// (crafting.ts), whose search finds every item and where clicking anything you cannot craft yet opens its page. Every
// ingredient, machine and use on a page links to that item's page, and Back retraces the steps, so any chain
// (concrete, quicklime, limestone, where to dig) can be walked down to the ground. The ways come from the engine
// (`item_ways`, `item_uses`: `item_guide.rs`), and a test there holds them to cover every item anything asks for.
// To show another kind of way: a case in `wayRow` (and `sourceHint`).

import './item-guide.css';
import type { Game } from '../wasm/engine.js';
import { button, h } from './dom';
import { formatSeconds } from './machine-recipes';

const ICON_PX = 64;
/** A page lists this many uses, then says how many more there are. */
const MAX_USES = 24;

type Way = { kind: 'found'; text: string } | { kind: 'hand'; recipe: number } | { kind: 'machine'; recipe: number; machine: number };

/** The engine's ways to get `item`, parsed. */
export function waysOf(game: Game, item: number): Way[] {
  const text = game.item_ways(item);
  if (!text) return [];
  return text.split('\n').map((line): Way => {
    const [kind, a, b] = line.split('\t');
    if (kind === 'found') return { kind, text: a };
    if (kind === 'hand') return { kind, recipe: Number(a) };
    return { kind: 'machine', recipe: Number(a), machine: Number(b) };
  });
}

/** Where `item` mainly comes from, in a few words: "found in the world", "made in a Smelter", "crafted by hand". */
export function sourceHint(game: Game, item: number): string {
  const way = waysOf(game, item)[0];
  if (!way) return '';
  if (way.kind === 'found') return 'found in the world';
  if (way.kind === 'hand') return 'crafted by hand';
  return `made in ${withArticle(game.item_name(way.machine))}`;
}

/** Each way to get `item` in a line, for the build menu's hover card: "In an Assembler from 1 Quicklime, 2 Sand". */
export function waySummary(game: Game, item: number): string[] {
  const names = (flat: Uint32Array) => {
    const out: string[] = [];
    for (let k = 0; k + 1 < flat.length; k += 2) out.push(`${flat[k + 1]} ${game.item_name(flat[k])}`);
    return out.join(', ');
  };
  return waysOf(game, item).map((w) => {
    if (w.kind === 'found') return w.text;
    if (w.kind === 'hand') return `By hand from ${names(game.recipe_inputs(w.recipe))}`;
    const tech = game.machine_recipe_locked_by(w.recipe);
    const lockNote = tech >= 0 ? ` (research ${game.tech_name(tech)})` : '';
    return `In ${withArticle(game.item_name(w.machine))} from ${names(game.machine_recipe_inputs(w.recipe))}${lockNote}`;
  });
}

/** The names of the machines and ingredients of every recipe that makes `item`, for the build menu's search. */
export function wayWords(game: Game, item: number): string {
  const names: string[] = [];
  for (const w of waysOf(game, item)) {
    if (w.kind === 'found') continue;
    if (w.kind === 'machine') names.push(game.item_name(w.machine));
    const flat = w.kind === 'hand' ? game.recipe_inputs(w.recipe) : game.machine_recipe_inputs(w.recipe);
    for (let k = 0; k < flat.length; k += 2) names.push(game.item_name(flat[k]));
  }
  return names.join(' ');
}

/** "a Smelter", "an Assembler". */
export function withArticle(name: string): string {
  return `${/^[aeiou]/i.test(name) ? 'an' : 'a'} ${name}`;
}

export class ItemGuide {
  readonly el = h('section', 'guide hidden');
  private readonly back = button('guide-back', '‹ Back', () => this.step());
  private readonly body = h('div', 'guide-body');
  /** The pages walked through, the one on show last. */
  private trail: number[] = [];

  constructor(
    private readonly game: Game,
    private readonly icon: (item: number) => HTMLCanvasElement,
  ) {
    const close = button('guide-close', 'Build menu', () => this.close());
    close.title = 'Back to the build menu';
    const bar = h('div', 'guide-bar');
    bar.append(this.back, close);
    this.el.append(bar, this.body);
    this.el.addEventListener('keydown', (e) => {
      if (e.key === 'Backspace' && !(e.target instanceof HTMLInputElement)) this.step();
    });
  }

  get isOpen(): boolean {
    return !this.el.classList.contains('hidden');
  }

  /** Shows `item`'s page, after the one on show (Back returns to it). */
  open(item: number): void {
    if (this.trail[this.trail.length - 1] !== item) this.trail.push(item);
    this.el.classList.remove('hidden');
    this.render();
  }

  close(): void {
    this.trail = [];
    this.el.classList.add('hidden');
  }

  /** Redraws the page on show (crafting.ts calls it when the inventory changes: have counts, craft buttons). */
  refresh(): void {
    if (this.isOpen) this.render();
  }

  /** Back one page, or to the build menu from the first. */
  private step(): void {
    this.trail.pop();
    if (this.trail.length === 0) this.close();
    else this.render();
  }

  private render(): void {
    const g = this.game;
    const item = this.trail[this.trail.length - 1];
    this.back.textContent = this.trail.length > 1 ? `‹ ${g.item_name(this.trail[this.trail.length - 2])}` : '‹ Back';
    const head = h('div', 'guide-head');
    head.append(this.canvas(item, 'guide-icon'), h('h3', '', g.item_name(item)), h('span', 'guide-have', `You have ${g.item_total(item)}`));

    const ways = waysOf(g, item);
    const get = h('div', 'guide-ways');
    get.append(...ways.map((w) => this.wayRow(item, w)));
    if (ways.length === 0) get.append(h('p', 'guide-note', 'Nothing makes this, and the world does not give it.'));

    const uses = g.item_uses(item);
    const used = h('div', 'guide-uses');
    const outs = new Set<number>();
    for (let k = 0; k + 1 < uses.length; k += 2) {
      outs.add(uses[k] === 0 ? g.recipe_output(uses[k + 1]) : this.machineProduct(uses[k + 1]));
    }
    const list = [...outs].filter((o) => o !== item);
    used.append(...list.slice(0, MAX_USES).map((o) => this.link(o)));
    if (list.length > MAX_USES) used.append(h('span', 'guide-note', `and ${list.length - MAX_USES} more`));
    if (list.length === 0) used.append(h('p', 'guide-note', 'No recipe uses it.'));

    this.body.replaceChildren(head, h('h4', 'guide-title', 'How to get it'), get, h('h4', 'guide-title', 'Used to make'), used);
    this.body.scrollTop = 0;
  }

  /** One way to get `item`: what it is, what goes in (each a link) and what stands in the way. */
  private wayRow(item: number, way: Way): HTMLElement {
    const g = this.game;
    const row = h('div', `guide-way ${way.kind}`);
    if (way.kind === 'found') {
      row.append(h('span', 'guide-label', 'In the world'), h('p', 'guide-text', way.text));
      return row;
    }
    const label = h('span', 'guide-label');
    const flat = way.kind === 'hand' ? g.recipe_inputs(way.recipe) : g.machine_recipe_inputs(way.recipe);
    const ins = h('div', 'guide-inputs');
    for (let k = 0; k + 1 < flat.length; k += 2) ins.append(this.link(flat[k], flat[k + 1]));
    const foot = h('p', 'guide-foot');
    if (way.kind === 'hand') {
      label.textContent = 'By hand';
      const n = g.recipe_output_count(way.recipe);
      const tech = g.recipe_locked_by(way.recipe);
      foot.textContent = `Makes ${n} · ${g.recipe_tenths(way.recipe) / 10} s`;
      if (tech >= 0) foot.append(lock(`Research ${g.tech_name(tech)} to unlock it (T)`));
      else if (g.craftable_times(way.recipe) > 0) {
        foot.append(button('guide-craft', 'Craft one', () => { g.craft(way.recipe, 1); }));
      } else foot.append(h('span', 'guide-short', ' · Gather what is missing first'));
    } else {
      label.append('In ', this.link(way.machine));
      const outs = g.machine_recipe_output(way.recipe);
      const at = outs.findIndex((o, i) => i % 2 === 0 && o === item);
      const side = at > 0 ? ' as a byproduct' : '';
      foot.textContent = `Makes ${outs[at + 1]}${side} · ${formatSeconds(g.machine_recipe_seconds(way.recipe))} a batch`;
      const tech = g.machine_recipe_locked_by(way.recipe);
      if (tech >= 0) foot.append(lock(`Research ${g.tech_name(tech)} to unlock it (T)`));
    }
    row.append(label, ins, foot);
    // How many machines feed how many, to plan a line before building it.
    const ratios = way.kind === 'machine' ? g.machine_recipe_ratios(way.recipe) : '';
    if (ratios) row.append(h('p', 'guide-ratio', `At full speed: ${ratios}`));
    return row;
  }

  /** What a machine recipe is for: its main product, or a launch's payload. */
  private machineProduct(recipe: number): number {
    return this.game.machine_recipe_payload(recipe) || this.game.machine_recipe_output(recipe)[0];
  }

  /** An item as a link to its page: icon, count when given, name; short when the inventory holds fewer. */
  private link(item: number, count = 0): HTMLButtonElement {
    const g = this.game;
    const b = button('guide-link', '', () => this.open(item));
    if (count > 0 && g.item_total(item) < count) b.classList.add('short');
    b.append(this.canvas(item, 'guide-link-icon'));
    if (count > 0) b.append(h('b', '', String(count)));
    b.append(g.item_name(item));
    b.title = `How to get ${g.item_name(item)}`;
    return b;
  }

  private canvas(item: number, className: string): HTMLCanvasElement {
    const c = h('canvas', className);
    c.width = c.height = ICON_PX;
    c.getContext('2d')!.drawImage(this.icon(item), 0, 0);
    return c;
  }
}

function lock(text: string): HTMLElement {
  return h('span', 'guide-lock', ` · ${text}`);
}
