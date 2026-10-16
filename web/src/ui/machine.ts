// Machine panel: opens when the player right-clicks a smelter, constructor, filter or quarry (the engine asks with
// `take_panel_request`). It shows inside the inventory screen in place of the build menu (`InventoryPanel.open`'s
// `beside`), so items move between the two like a chest: click a slot with a stack held to put it in, click a filled
// slot to take it, Shift-click an inventory stack to put it in. Shows what the machine is doing, its buffers, the
// recipe choice for machines where the player picks it, a filter's item choice, one-click put-in buttons, a
// take-output button, and a quarry's choices (ui/quarry.ts; `quarryBox` for its outline). Sections a machine doesn't
// use are hidden.
// Everything shown is read from the engine each frame (`machine_panel`); buttons queue engine actions.
// A new machine with a panel needs no change here unless it adds a buffer role (`ROLE_LABELS`).

import './machine.css';
import type { Game } from '../wasm/engine.js';
import { button, h } from './dom';
import type { Beside, InventoryPanel } from './inventory';
import { RecipePicker } from './machine-recipes';
import { QuarrySection } from './quarry';

const ICON_PX = 64;

export class MachinePanel implements Beside {
  readonly el = h('section', 'mp');
  title = '';
  private readonly status = h('p', 'mp-status');
  private readonly efficiency = h('p', 'mp-efficiency');
  private readonly bar = h('div', 'mp-bar-fill');
  private readonly fire = h('span', 'mp-fire');
  private readonly slots = h('div', 'mp-slots');
  private readonly recipeHead = h('h3', '', 'Recipe');
  private readonly recipes: RecipePicker;
  private readonly inserts = h('div', 'mp-inserts');
  private readonly put = h('section', 'mp-put');
  private readonly progress = h('div', 'mp-bar');
  private readonly filterSection = h('section', 'mp-filter');
  private readonly filterList = h('div', 'mp-inserts');
  private readonly take: HTMLButtonElement;
  private readonly quarry: QuarrySection;
  private readonly roleLabels: Map<number, string>;
  private pos: [number, number, number] = [0, 0, 0];
  private block = -1;
  private drawn = '';
  private shown = false;

  constructor(
    private readonly game: Game,
    private readonly icon: (item: number) => HTMLCanvasElement,
    private readonly inventory: InventoryPanel,
  ) {
    this.roleLabels = new Map([
      [game.panel_role_input(), 'In'],
      [game.panel_role_fuel(), 'Fuel'],
      [game.panel_role_output(), 'Out'],
    ]);
    this.progress.append(this.bar);
    this.take = button('secondary-btn mp-take', 'Take output', () => {
      this.game.take_machine_output(...this.pos);
    });
    const state = h('section', 'mp-state');
    state.append(this.status, this.efficiency, this.progress, this.fire, this.slots, this.take);

    this.put.append(h('h3', '', 'Put in from your inventory'), this.inserts);
    this.filterSection.append(h('h3', '', 'Goes straight on'), this.filterList);
    this.quarry = new QuarrySection(game);
    this.recipes = new RecipePicker(game, icon, (id) => {
      const current = this.game.machine_panel(...this.pos)[1];
      this.game.set_machine_recipe(...this.pos, current === id ? -1 : id);
    });
    this.el.append(state, this.quarry.el, this.filterSection, this.recipeHead, this.recipes.el, this.put);
  }

  get isOpen(): boolean {
    return this.shown;
  }

  /** Opens the inventory screen with the panel of the machine at a position (does nothing if it has none). */
  open(x: number, y: number, z: number): void {
    const data = this.game.machine_panel(x, y, z);
    if (data.length === 0) return;
    this.pos = [x, y, z];
    if (data[0] !== this.block) {
      this.block = data[0];
      this.recipes.build(data[0]);
    }
    this.recipes.reveal();
    this.title = this.game.block_name(data[0]);
    this.drawn = '';
    this.shown = true;
    this.inventory.open(null, this);
  }

  /** The open quarry's box, as lowest and highest cells, or null when no quarry panel is open. */
  quarryBox(): Int32Array | null {
    return this.shown && this.quarry.visible ? this.game.quarry_box(...this.pos) : null;
  }

  closed(): void {
    this.shown = false;
  }

  /** Shift-click on inventory slot `slot`: puts that item in if the machine takes it. */
  shiftSlot(slot: number): boolean {
    const g = this.game;
    if (g.slot_count(slot) === 0 || !g.machine_wants(...this.pos, g.slot_item(slot))) return false;
    g.insert_into_machine(...this.pos, g.slot_item(slot));
    return true;
  }

  wants(item: number): boolean {
    return this.game.machine_wants(...this.pos, item);
  }

  /** Redraws whatever changed; cheap when nothing did. Called every frame by the inventory screen; false once the
   * machine is gone. */
  update(): boolean {
    const g = this.game;
    const data = g.machine_panel(...this.pos);
    if (data.length === 0) return false;
    const [, recipe, progress, fire, choosable, count] = data;
    this.bar.style.transform = `scaleX(${progress / 1000})`;
    const quarry = this.quarry.update(this.pos);
    // The running average moves a little every frame, so it is outside the redraw key below.
    const efficiency = g.machine_efficiency(...this.pos);
    if (this.efficiency.textContent !== efficiency) this.efficiency.textContent = efficiency;
    const percent = g.machine_efficiency_percent(...this.pos);
    this.efficiency.dataset.level = percent < 0 ? 'idle' : percent >= 98 ? 'full' : percent >= 60 ? 'ok' : 'low';
    // Everything else changes rarely: redraw it only when the rest of the data or the inventory does.
    data[2] = 0;
    const key = `${data.join(',')}|${g.inventory_version()}`;
    if (key === this.drawn) return true;
    this.drawn = key;

    this.status.textContent = g.machine_status(...this.pos);
    this.fire.textContent = fire > 0 ? `Fire: ${fire} s left` : '';
    this.slots.replaceChildren();
    let output = -1;
    let held = 0;
    // A silo has 144 slots: show only the ones that hold something.
    const many = count > 12;
    for (let i = 0; i < count; i++) {
      const role = data[6 + i * 3], item = data[7 + i * 3], n = data[8 + i * 3];
      if (role === g.panel_role_output()) output = Math.max(output, 0) + n;
      else if (n > 0) held = item;
      if (many && n === 0) continue;
      this.slots.append(this.slotView(this.roleLabels.get(role) ?? '', item, n, i));
    }
    this.take.disabled = output <= 0;
    this.take.classList.toggle('hidden', output < 0);

    const filter = g.machine_filter(...this.pos);
    const filtering = filter !== 0xffffffff;
    this.filterSection.classList.toggle('hidden', !filtering);
    this.put.classList.toggle('hidden', filtering || quarry);
    this.progress.classList.toggle('hidden', filtering);
    if (filtering) this.drawFilter(filter, held);
    this.recipeHead.classList.toggle('hidden', this.recipes.isEmpty);
    this.recipes.el.classList.toggle('hidden', this.recipes.isEmpty);
    this.recipes.update(recipe === 0xffffffff ? -1 : recipe, choosable === 1);
    this.recipeHead.textContent = choosable ? 'Choose what it makes' : 'It makes (from what it gets)';
    if (!filtering && !quarry) this.drawInserts();
    return true;
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

  /** A slot of the panel: with a stack held, a click puts it in (as far as the machine takes it); otherwise a click
   * on a filled slot takes its stack into the inventory (a stuck input too). */
  private slotView(label: string, item: number, n: number, index: number): HTMLElement {
    const box = h('div', 'mp-slot');
    const slot = h('div', 'slot mp-takeable');
    slot.dataset.slot = 'machine'; // for the inventory screen's controls bar
    slot.dataset.item = n > 0 ? String(item) : '';
    if (n > 0) {
      slot.append(this.iconCanvas(item, ''), h('span', 'count', n > 1 ? String(n) : ''));
      slot.title = this.game.item_name(item);
    }
    slot.addEventListener('pointerdown', (e) => {
      if (e.button !== 0) return;
      e.preventDefault();
      if (this.game.cursor_count() > 0) this.game.insert_cursor_into_machine(...this.pos);
      else if (n > 0) this.game.take_machine_slot(...this.pos, index);
    });
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
