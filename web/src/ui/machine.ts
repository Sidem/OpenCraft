// Machine panel: opens when the player right-clicks a smelter, constructor or filter (the engine asks
// with `take_panel_request`). Shows what the machine is doing, its buffers, the recipe choice for
// machines where the player picks it, a filter's item choice, buttons to put items in from the
// inventory, and a take-output button. Sections a machine doesn't use are hidden.
// Everything shown is read from the engine each frame (`machine_panel`); buttons queue engine actions.
// A new machine with a panel needs no change here unless it adds a buffer role (`ROLE_LABELS`).

import './machine.css';
import type { Game } from '../wasm/engine.js';
import { button, h } from './dom';

const ICON_PX = 64;

interface RecipeButton {
  id: number;
  el: HTMLButtonElement;
}

export class MachinePanel {
  /** Called after the panel closes. `resume` is true when play should continue (E, or a click outside). */
  onClose: (resume: boolean) => void = () => {};

  private readonly backdrop = h('div', 'mp-backdrop hidden');
  private readonly dialog = h('div', 'mp');
  private readonly title = h('h2', '');
  private readonly status = h('p', 'mp-status');
  private readonly bar = h('div', 'mp-bar-fill');
  private readonly fire = h('span', 'mp-fire');
  private readonly slots = h('div', 'mp-slots');
  private readonly recipeHead = h('h3', '', 'Recipe');
  private readonly recipeList = h('div', 'mp-recipes');
  private readonly inserts = h('div', 'mp-inserts');
  private readonly put = h('section', 'mp-put');
  private readonly progress = h('div', 'mp-bar');
  private readonly filterSection = h('section', 'mp-filter');
  private readonly filterList = h('div', 'mp-inserts');
  private readonly take: HTMLButtonElement;
  private readonly roleLabels: Map<number, string>;
  private recipes: RecipeButton[] = [];
  private pos: [number, number, number] = [0, 0, 0];
  private block = -1;
  private drawn = '';

  constructor(
    private readonly game: Game,
    private readonly icon: (item: number) => HTMLCanvasElement,
  ) {
    this.roleLabels = new Map([
      [game.panel_role_input(), 'In'],
      [game.panel_role_fuel(), 'Fuel'],
      [game.panel_role_output(), 'Out'],
    ]);
    this.dialog.setAttribute('role', 'dialog');
    this.dialog.setAttribute('aria-modal', 'true');
    this.dialog.tabIndex = -1;

    const head = h('div', 'mp-head');
    const close = button('close-btn', '×', () => this.close(true));
    close.setAttribute('aria-label', 'Close');
    head.append(this.title, h('span', 'mp-keys', 'E or click outside to return'), close);

    this.progress.append(this.bar);
    this.take = button('secondary-btn mp-take', 'Take output', () => {
      this.game.take_machine_output(...this.pos);
    });
    const state = h('section', 'mp-state');
    state.append(this.status, this.progress, this.fire, this.slots, this.take);

    this.put.append(h('h3', '', 'Put in from your inventory'), this.inserts);
    this.filterSection.append(h('h3', '', 'Goes straight on'), this.filterList);
    const body = h('div', 'mp-body');
    body.append(state, this.filterSection, this.recipeHead, this.recipeList, this.put);
    this.dialog.append(head, body);
    this.backdrop.append(this.dialog);

    this.backdrop.addEventListener('pointerdown', (e) => {
      if (e.target === this.backdrop) this.close(true);
    });
    this.backdrop.addEventListener('contextmenu', (e) => e.preventDefault());
    window.addEventListener('keydown', (e) => {
      if (!this.isOpen || e.repeat) return;
      if (e.code === 'KeyE') {
        e.preventDefault();
        this.close(true);
      } else if (e.key === 'Escape') {
        e.preventDefault();
        this.close(false);
      }
    });
    document.body.append(this.backdrop);
  }

  get isOpen(): boolean {
    return !this.backdrop.classList.contains('hidden');
  }

  /** Opens the panel of the machine at a position (does nothing if it has none). */
  open(x: number, y: number, z: number): void {
    const data = this.game.machine_panel(x, y, z);
    if (data.length === 0) return;
    this.pos = [x, y, z];
    if (data[0] !== this.block) this.buildRecipes(data[0]);
    this.title.textContent = this.game.block_name(data[0]);
    this.backdrop.classList.remove('hidden');
    this.drawn = '';
    this.update();
    this.dialog.focus();
  }

  close(resume: boolean): void {
    if (!this.isOpen) return;
    this.backdrop.classList.add('hidden');
    this.onClose(resume);
  }

  /** Redraws whatever changed; cheap when nothing did. Call every frame. Closes if the machine is gone. */
  update(): void {
    if (!this.isOpen) return;
    const g = this.game;
    const data = g.machine_panel(...this.pos);
    if (data.length === 0) {
      this.close(true);
      return;
    }
    const [, recipe, progress, fire, choosable, count] = data;
    this.bar.style.transform = `scaleX(${progress / 1000})`;
    // Everything else changes rarely: redraw it only when the rest of the data or the inventory does.
    data[2] = 0;
    const key = `${data.join(',')}|${g.inventory_version()}`;
    if (key === this.drawn) return;
    this.drawn = key;

    this.status.textContent = g.machine_status(...this.pos);
    this.fire.textContent = fire > 0 ? `Fire: ${fire} s left` : '';
    this.slots.replaceChildren();
    let output = -1;
    let held = 0;
    for (let i = 0; i < count; i++) {
      const role = data[6 + i * 3], item = data[7 + i * 3], n = data[8 + i * 3];
      if (role === g.panel_role_output()) output = Math.max(output, 0) + n;
      else if (n > 0) held = item;
      this.slots.append(this.slotView(this.roleLabels.get(role) ?? '', item, n));
    }
    this.take.disabled = output <= 0;
    this.take.classList.toggle('hidden', output < 0);

    const filter = g.machine_filter(...this.pos);
    const filtering = filter !== 0xffffffff;
    this.filterSection.classList.toggle('hidden', !filtering);
    this.put.classList.toggle('hidden', filtering);
    this.progress.classList.toggle('hidden', filtering);
    if (filtering) this.drawFilter(filter, held);
    this.recipeHead.classList.toggle('hidden', this.recipes.length === 0);

    for (const r of this.recipes) {
      r.el.classList.toggle('active', r.id === recipe);
      r.el.disabled = !choosable;
    }
    this.recipeHead.textContent = choosable ? 'Choose what it makes' : 'It makes (from the ore it gets)';
    if (!filtering) this.drawInserts();
  }

  /** The filter's choice: nothing, or any item in the inventory, the current choice or the held item. */
  private drawFilter(filter: number, held: number): void {
    const g = this.game;
    const items = new Set(this.inventoryTotals().keys());
    if (filter !== 0) items.add(filter);
    if (held !== 0) items.add(held);
    const none = button('secondary-btn mp-insert', 'Nothing', () => g.set_machine_filter(...this.pos, 0));
    none.classList.toggle('active', filter === 0);
    const buttons = [none];
    for (const item of items) {
      const b = button('secondary-btn mp-insert', '', () => g.set_machine_filter(...this.pos, item));
      b.classList.toggle('active', item === filter);
      b.append(this.iconCanvas(item, 'mp-chip-icon'), g.item_name(item));
      buttons.push(b);
    }
    this.filterList.replaceChildren(...buttons);
  }

  /** Each item in the inventory with how many there are. */
  private inventoryTotals(): Map<number, number> {
    const g = this.game;
    const totals = new Map<number, number>();
    for (let s = 0; s < g.inventory_size(); s++) {
      const n = g.slot_count(s);
      if (n > 0) totals.set(g.slot_item(s), (totals.get(g.slot_item(s)) ?? 0) + n);
    }
    return totals;
  }

  private buildRecipes(block: number): void {
    const g = this.game;
    this.block = block;
    this.recipes = Array.from(g.machine_recipes(block), (id) => {
      const [out, n] = g.machine_recipe_output(id);
      const ins = g.machine_recipe_inputs(id);
      const text = h('span', 'mp-recipe-text');
      text.append(h('b', '', `${g.item_name(out)}${n > 1 ? ` ×${n}` : ''}`));
      const parts: string[] = [];
      for (let k = 0; k + 1 < ins.length; k += 2) parts.push(`${ins[k + 1]} ${g.item_name(ins[k])}`);
      text.append(h('small', '', `from ${parts.join(' + ')} · ${g.machine_recipe_seconds(id)} s`));
      const el = button('mp-recipe', '', () => {
        const current = this.game.machine_panel(...this.pos)[1];
        this.game.set_machine_recipe(...this.pos, current === id ? -1 : id);
      });
      el.title = 'Click again to clear. Items it holds come back to you.';
      el.append(this.iconCanvas(out, 'mp-recipe-icon'), text);
      return { id, el };
    });
    this.recipeList.replaceChildren(...this.recipes.map((r) => r.el));
  }

  /** One button per item in the inventory that the machine would take now. */
  private drawInserts(): void {
    const g = this.game;
    const buttons: HTMLElement[] = [];
    for (const [item, n] of this.inventoryTotals()) {
      if (!g.machine_wants(...this.pos, item)) continue;
      const b = button('secondary-btn mp-insert', '', () => g.insert_into_machine(...this.pos, item));
      b.append(this.iconCanvas(item, 'mp-chip-icon'), `${g.item_name(item)} ×${n}`);
      buttons.push(b);
    }
    if (buttons.length === 0) buttons.push(h('p', 'mp-note', 'Nothing in your inventory goes in here right now.'));
    this.inserts.replaceChildren(...buttons);
  }

  private slotView(label: string, item: number, n: number): HTMLElement {
    const box = h('div', 'mp-slot');
    const slot = h('div', 'slot');
    if (n > 0) {
      slot.append(this.iconCanvas(item, ''), h('span', 'count', n > 1 ? String(n) : ''));
      slot.title = this.game.item_name(item);
    }
    box.append(slot, h('span', 'mp-slot-label', label));
    return box;
  }

  private iconCanvas(item: number, className: string): HTMLCanvasElement {
    const c = h('canvas', className);
    c.width = c.height = ICON_PX;
    c.getContext('2d')!.drawImage(this.icon(item), 0, 0);
    return c;
  }
}
