// Pinned HUD shopping list ("To-do", in #hud-left): lets the player pin buildings or parts from the build menu
// (Alt-click a tile, Shift+Alt-click for +5) or the item guide (+ To-do), and shows live progress for both the
// pinned targets and the combined materials still needed to craft or assemble them. Presentation only: kept per
// browser in localStorage, reading counts and recipes from `game` only when `inventory_version` moves.

import './todo.css';
import type { Game } from '../wasm/engine.js';
import { button, h } from './dom';
import { waysOf } from './item-guide';

const STORAGE_KEY = 'opencraft.todo';
const ICON_PX = 64;
const MAX_NEEDS = 8;

export interface TodoEntry {
  item: number;
  count: number;
}

export class TodoList {
  readonly el = h('section', 'todo hidden');
  private readonly targets = h('div', 'todo-targets');
  private readonly needsHead = h('div', 'todo-sub', 'Materials needed');
  private readonly needs = h('div', 'todo-needs');
  private entries: TodoEntry[];
  private version = -1;
  private recipeMap = new Map<number, number>();
  private onChange: (() => void)[] = [];

  constructor(
    private readonly game: Game,
    private readonly icon: (item: number) => HTMLCanvasElement,
  ) {
    this.entries = loadEntries();
    for (let r = 0; r < game.recipe_count(); r++) {
      const out = game.recipe_output(r);
      if (!this.recipeMap.has(out)) this.recipeMap.set(out, r);
    }
    const head = h('div', 'todo-head');
    const clear = button('todo-clear', 'Clear', () => this.clear());
    clear.title = 'Clear pinned to-do list';
    head.append(h('span', 'todo-title', 'To-do'), clear);
    this.el.append(head, this.targets, this.needsHead, this.needs);
    document.getElementById('hud-left')!.append(this.el);
  }

  subscribe(fn: () => void): void {
    this.onChange.push(fn);
  }

  countOf(item: number): number {
    return this.entries.find((e) => e.item === item)?.count ?? 0;
  }

  add(item: number, delta: number): void {
    const cur = this.entries.find((e) => e.item === item);
    if (cur) {
      cur.count = Math.max(0, Math.min(999, cur.count + delta));
      if (cur.count === 0) this.entries = this.entries.filter((e) => e.item !== item);
    } else if (delta > 0) {
      this.entries.push({ item, count: Math.min(999, delta) });
    }
    this.changed();
  }

  clear(): void {
    if (this.entries.length === 0) return;
    this.entries = [];
    this.changed();
  }

  update(): void {
    if (this.entries.length === 0) {
      this.el.classList.add('hidden');
      return;
    }
    this.el.classList.remove('hidden');
    const v = this.game.inventory_version();
    if (v === this.version) return;
    this.version = v;
    this.render();
  }

  private changed(): void {
    saveEntries(this.entries);
    this.version = -1;
    this.update();
    for (const fn of this.onChange) fn();
  }

  private render(): void {
    const g = this.game;
    const matNeed = new Map<number, number>();
    this.targets.replaceChildren(
      ...this.entries.map(({ item, count }) => {
        const have = g.item_total(item);
        const done = have >= count;
        const row = h('div', `todo-row${done ? ' done' : ''}`);
        const step = h('span', 'todo-steps');
        step.append(
          button('todo-step', '−', () => this.add(item, -1)),
          button('todo-step', '+', () => this.add(item, 1)),
          button('todo-step', '×', () => this.add(item, -count)),
        );
        row.append(
          this.canvas(item),
          h('span', 'todo-name', g.item_name(item)),
          h('span', 'todo-count', `${done ? '✓ ' : ''}${have}/${count}`),
          step,
        );
        const short = Math.max(0, count - have);
        if (short > 0) this.accumulateInputs(item, short, matNeed);
        return row;
      }),
    );

    const rows: HTMLElement[] = [];
    const entries = [...matNeed.entries()];
    for (const [mat, need] of entries.slice(0, MAX_NEEDS)) {
      const have = g.item_total(mat);
      const ok = have >= need;
      const r = h('div', `todo-need${ok ? ' done' : ''}`);
      r.append(
        this.canvas(mat),
        h('span', 'todo-name', g.item_name(mat)),
        h('span', 'todo-count', `${ok ? '✓ ' : ''}${have}/${need}`),
      );
      rows.push(r);
    }
    if (entries.length > MAX_NEEDS) {
      rows.push(h('div', 'todo-more', `+${entries.length - MAX_NEEDS} more materials`));
    }
    this.needsHead.classList.toggle('hidden', rows.length === 0);
    this.needs.classList.toggle('hidden', rows.length === 0);
    this.needs.replaceChildren(...rows);
  }

  private accumulateInputs(item: number, missing: number, out: Map<number, number>): void {
    const g = this.game;
    const hand = this.recipeMap.get(item);
    if (hand !== undefined) {
      const yieldN = g.tool_uses(item) > 0 ? 1 : Math.max(1, g.recipe_output_count(hand));
      const batches = Math.ceil(missing / yieldN);
      const flat = g.recipe_inputs(hand);
      for (let k = 0; k + 1 < flat.length; k += 2) {
        out.set(flat[k], (out.get(flat[k]) ?? 0) + flat[k + 1] * batches);
      }
      return;
    }
    const mach = waysOf(g, item).find((w) => w.kind === 'machine');
    if (mach && mach.kind === 'machine') {
      const outs = g.machine_recipe_output(mach.recipe);
      const at = outs.findIndex((o, i) => i % 2 === 0 && o === item);
      const yieldN = at >= 0 ? Math.max(1, outs[at + 1]) : 1;
      const batches = Math.ceil(missing / yieldN);
      const flat = g.machine_recipe_inputs(mach.recipe);
      for (let k = 0; k + 1 < flat.length; k += 2) {
        out.set(flat[k], (out.get(flat[k]) ?? 0) + flat[k + 1] * batches);
      }
    }
  }

  private canvas(item: number): HTMLCanvasElement {
    const c = h('canvas', 'todo-icon');
    c.width = c.height = ICON_PX;
    c.getContext('2d')!.drawImage(this.icon(item), 0, 0);
    return c;
  }
}

function loadEntries(): TodoEntry[] {
  try {
    const raw: unknown = JSON.parse(localStorage.getItem(STORAGE_KEY) ?? '[]');
    if (!Array.isArray(raw)) return [];
    return raw.filter(
      (x): x is TodoEntry =>
        typeof x === 'object' && x !== null && typeof (x as TodoEntry).item === 'number' && (x as TodoEntry).count > 0,
    );
  } catch {
    return [];
  }
}

function saveEntries(entries: TodoEntry[]): void {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(entries));
  } catch {
    // Storage unavailable in private mode.
  }
}
