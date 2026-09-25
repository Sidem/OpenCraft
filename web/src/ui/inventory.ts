// Inventory and build screen (E): all 36 slots with a held stack on the cursor, and the
// hand-crafting recipes for the first factory machines.

import './inventory.css';
import type { Game } from '../wasm/engine.js';
import { h } from './dom';

const ICON_PX = 64;
/** Shift-clicking Craft makes up to this many at once. */
const BULK_CRAFT = 5;

interface SlotView {
  root: HTMLDivElement;
  icon: HTMLCanvasElement;
  count: HTMLSpanElement;
  item: number;
  n: number;
}

interface Chip {
  item: number;
  need: number;
  el: HTMLSpanElement;
  have: HTMLSpanElement;
}

interface RecipeView {
  id: number;
  root: HTMLDivElement;
  craft: HTMLButtonElement;
  chips: Chip[];
}

export class InventoryPanel {
  /** Called after the panel closes. `resume` is true when play should continue (E, or a click outside). */
  onClose: (resume: boolean) => void = () => {};
  /** Called after something was crafted. */
  onCraft: () => void = () => {};

  private readonly backdrop = h('div', 'inv-backdrop hidden');
  private readonly dialog = h('div', 'inv');
  private readonly slots: SlotView[] = [];
  private readonly recipes: RecipeView[] = [];
  private readonly cursor: SlotView;
  private version = -1;

  constructor(
    private readonly game: Game,
    private readonly icon: (item: number) => HTMLCanvasElement,
  ) {
    this.dialog.setAttribute('role', 'dialog');
    this.dialog.setAttribute('aria-modal', 'true');
    this.dialog.setAttribute('aria-labelledby', 'inv-title');
    this.dialog.tabIndex = -1;

    const head = h('div', 'inv-head');
    const title = h('h2', '', 'Inventory & build');
    title.id = 'inv-title';
    const close = h('button', 'close-btn', '×');
    close.type = 'button';
    close.setAttribute('aria-label', 'Close inventory');
    close.addEventListener('click', () => this.close(true));
    head.append(title, h('span', 'inv-keys', 'E or click outside to return · Shift-click moves a stack'), close);

    // Backpack (slots 9..35) above the hotbar (0..8), as in the HUD.
    const size = game.inventory_size(), hotbar = game.hotbar_size();
    for (let i = 0; i < size; i++) this.slots.push(this.makeSlot(i));
    const pack = h('div', 'inv-grid');
    pack.append(...this.slots.slice(hotbar).map((s) => s.root));
    const bar = h('div', 'inv-grid inv-hotbar');
    bar.append(...this.slots.slice(0, hotbar).map((s) => s.root));
    const note = h(
      'p',
      'inv-note',
      `Mining ore by hand keeps ${game.hand_yield()} per block; the rest of that block's share of its deposit is lost. ` +
        `A miner recovers ${Math.round(game.miner_recovery() * 100)}% of what it drills, so build one early. ` +
        'Look at ore to see how big its deposit is.',
    );
    const items = h('section', 'inv-items');
    items.append(h('h3', '', 'Backpack'), pack, h('h3', '', 'Hotbar'), bar, note);

    const build = h('section', 'inv-build');
    const list = h('div', 'recipe-list');
    for (let r = 0; r < game.recipe_count(); r++) {
      const view = this.makeRecipe(r);
      this.recipes.push(view);
      list.append(view.root);
    }
    build.append(h('h3', '', 'Build'), list, h('p', 'inv-note', 'Shift-click Craft to make up to 5 at once.'));

    const body = h('div', 'inv-body');
    body.append(items, build);
    this.dialog.append(head, body);
    this.backdrop.append(this.dialog);

    const cur = h('div', 'slot inv-cursor hidden');
    const curIcon = h('canvas');
    curIcon.width = curIcon.height = ICON_PX;
    const curCount = h('span', 'count');
    cur.append(curIcon, curCount);
    this.cursor = { root: cur, icon: curIcon, count: curCount, item: -1, n: -1 };
    this.backdrop.append(cur);

    this.backdrop.addEventListener('pointerdown', (e) => {
      if (e.target === this.backdrop) this.close(true);
    });
    this.backdrop.addEventListener('pointermove', (e) => this.moveCursor(e.clientX, e.clientY));
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

  open(): void {
    this.backdrop.classList.remove('hidden');
    this.version = -1;
    this.update();
    this.dialog.focus();
  }

  close(resume: boolean): void {
    if (!this.isOpen) return;
    this.backdrop.classList.add('hidden');
    this.onClose(resume);
  }

  /** Redraws whatever changed since the last call. Cheap when nothing did; call every frame. */
  update(): void {
    if (!this.isOpen) return;
    const g = this.game;
    const v = g.inventory_version();
    if (v === this.version) return;
    this.version = v;

    const selected = g.selected_slot();
    this.slots.forEach((s, i) => {
      this.drawSlot(s, g.slot_item(i), g.slot_count(i));
      s.root.classList.toggle('selected', i === selected);
    });
    this.drawSlot(this.cursor, g.cursor_item(), g.cursor_count());
    this.cursor.root.classList.toggle('hidden', g.cursor_count() === 0);

    for (const r of this.recipes) {
      for (const c of r.chips) {
        const have = g.item_total(c.item);
        c.have.textContent = `${have}/${c.need}`;
        c.el.classList.toggle('short', have < c.need);
      }
      const ok = g.can_craft(r.id);
      r.craft.disabled = !ok;
      r.root.classList.toggle('ready', ok);
    }
  }

  private makeSlot(i: number): SlotView {
    const root = h('div', 'slot inv-slot');
    const icon = h('canvas');
    icon.width = icon.height = ICON_PX;
    const count = h('span', 'count');
    root.append(icon, count);
    if (i < this.game.hotbar_size()) root.append(h('span', 'key', String(i + 1)));
    root.addEventListener('pointerdown', (e) => {
      if (e.button !== 0) return;
      e.preventDefault();
      this.game.click_slot(i, e.shiftKey);
      this.moveCursor(e.clientX, e.clientY);
      this.update();
    });
    return { root, icon, count, item: -1, n: -1 };
  }

  private makeRecipe(r: number): RecipeView {
    const g = this.game;
    const out = g.recipe_output(r);
    const root = h('div', 'recipe');
    const text = h('div', 'recipe-text');
    const title = h('h4', '', g.item_name(out));
    const n = g.recipe_output_count(r);
    if (n > 1) title.append(h('span', 'recipe-yield', ` ×${n}`));
    const inputs = h('div', 'recipe-inputs');
    const chips: Chip[] = [];
    const flat = g.recipe_inputs(r);
    for (let k = 0; k + 1 < flat.length; k += 2) {
      const item = flat[k], need = flat[k + 1];
      const el = h('span', 'chip');
      el.title = this.game.item_name(item);
      const have = h('span', 'chip-have');
      el.append(this.iconCanvas(item, 'chip-icon'), this.game.item_name(item), have);
      inputs.append(el);
      chips.push({ item, need, el, have });
    }
    text.append(title, h('p', '', g.recipe_blurb(r)), inputs);

    const craft = h('button', 'secondary-btn craft-btn', 'Craft');
    craft.type = 'button';
    craft.title = `Shift-click to craft up to ${BULK_CRAFT}`;
    craft.addEventListener('click', (e) => {
      if (g.craft(r, e.shiftKey ? BULK_CRAFT : 1) > 0) this.onCraft();
      this.update();
    });
    root.append(this.iconCanvas(out, 'recipe-icon'), text, craft);
    return { id: r, root, craft, chips };
  }

  private iconCanvas(item: number, className: string): HTMLCanvasElement {
    const c = h('canvas', className);
    c.width = c.height = ICON_PX;
    c.getContext('2d')!.drawImage(this.icon(item), 0, 0);
    return c;
  }

  private drawSlot(s: SlotView, item: number, n: number): void {
    s.root.title = n > 0 ? `${this.game.item_name(item)}${n > 1 ? ` ×${n}` : ''}` : '';
    if (item === s.item && n === s.n) return;
    const ctx = s.icon.getContext('2d')!;
    ctx.clearRect(0, 0, ICON_PX, ICON_PX);
    if (n > 0) ctx.drawImage(this.icon(item), 0, 0);
    s.count.textContent = n > 1 ? String(n) : '';
    s.item = item;
    s.n = n;
  }

  private moveCursor(x: number, y: number): void {
    this.cursor.root.style.transform = `translate(${x}px, ${y}px) translate(-50%, -50%)`;
  }
}
