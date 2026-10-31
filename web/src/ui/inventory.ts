// Inventory and build screen (E): every slot (36, up to 135 with a hauler pack) with a held stack on the cursor, the
// backpack in pages of 27 (never scrolled: the equipment and hotbar stay in view), the equipment slots (Shift-click gear in the pack to wear it; a button raises the worn pack with kits,
// `pack-upgrade.ts`), and the build menu
// (`crafting.ts`; creative worlds add the item picker, `creative.ts`). Opened on a storage box (right-click), it shows the box's slots above the inventory
// instead of the build menu, like a chest: clicks move stacks with the cursor, shift-clicks move whole
// stacks between the box and the inventory. Right-click takes half of a stack (again: half of what is left, added to
// the held stack); Ctrl-click takes one item onto the cursor and Ctrl-right-click puts one down (so a stack can be
// built or split a single item at a time); Shift-right-click with a stack held puts one item down too (outside the
// screen: throws one out), and with an empty hand moves every stack of that item. Pressing a stack and letting go
// outside the panel (a drag) throws the whole stack. Opened with a `Beside` (a machine's
// panel, `machine.ts`), that panel takes the build menu's place and Shift-click puts a stack into it.
// The clicks are taught by the controls bar along the bottom (`slot-controls.ts`): it shows what applies to the
// hovered slot (`data-slot`, `data-item`) and the held stack, so the screen needs no wall of instructions.

import './inventory.css';
import type { Game } from '../wasm/engine.js';
import { BuildMenu } from './crafting';
import { CreativeMenu, withCreativeTabs } from './creative';
import { DragThrow } from './drag-throw';
import { h } from './dom';
import { showAmount } from './hud';
import { PackUpgrade } from './pack-upgrade';
import { Pager } from './pager';
import { type SlotContext, SlotControls } from './slot-controls';

const ICON_PX = 64;
/** Backpack slots on a page: three rows of nine, so the equipment and hotbar never scroll out of view. */
const PAGE_SLOTS = 27;

/** A panel shown beside the inventory in place of the build menu (a machine's: `machine.ts`). */
export interface Beside {
  readonly el: HTMLElement;
  readonly title: string;
  /** Redraws what changed; false once what it shows is gone (the screen then closes). Called every frame. */
  update(): boolean;
  /** Shift-click on inventory slot `slot`: true if it took the stack. */
  shiftSlot(slot: number): boolean;
  /** Whether Shift-click would put `item` in (for the controls bar). */
  wants(item: number): boolean;
  /** The screen closed. */
  closed(): void;
}

interface SlotView {
  root: HTMLDivElement;
  icon: HTMLCanvasElement;
  count: HTMLSpanElement;
  item: number;
  n: number;
}

export class InventoryPanel {
  /** Called after the panel closes. `resume` is true when play should continue (E, or a click outside). */
  onClose: (resume: boolean) => void = () => {};
  /** Called after something was crafted. */
  onCraft: () => void = () => {};

  private readonly backdrop = h('div', 'inv-backdrop hidden');
  private readonly dialog = h('div', 'inv');
  private readonly slots: SlotView[] = [];
  private readonly gear: SlotView[] = [];
  /** The left hand's slot (`offhand.ts` shows the same stack in the HUD). */
  private readonly offhand: SlotView;
  private readonly packUpgrade: PackUpgrade;
  private readonly packPager = new Pager();
  private readonly cursor: SlotView;
  private version = -1;
  private readonly title = h('h2', '', 'Inventory & build');
  private readonly menu: BuildMenu;
  /** The column beside the backpack: the build menu, with the item picker's tabs in a creative world. */
  private readonly right: HTMLElement;
  private readonly boxSection = h('section', 'inv-box hidden');
  private readonly boxGrid = h('div', 'inv-grid inv-box-grid');
  private readonly boxSlots: SlotView[] = [];
  /** The open box's position, or null for the plain inventory screen. */
  private box: [number, number, number] | null = null;
  /** The panel shown beside the inventory, if any. */
  private beside: Beside | null = null;
  private readonly body = h('div', 'inv-body');
  /** Whether Shift was held for the slot click being handled. */
  private shift = false;
  private readonly controls = new SlotControls();
  /** The pointer's last position, to find the slot under it (`showControls`). */
  private pointer: [number, number] = [-1, -1];
  private boxKey = '';
  /** Dragging a stack out of the panel throws it. */
  private readonly drag: DragThrow;

  constructor(
    private readonly game: Game,
    private readonly icon: (item: number) => HTMLCanvasElement,
  ) {
    this.drag = new DragThrow(this.backdrop, game, () => this.isOpen);
    this.dialog.setAttribute('role', 'dialog');
    this.dialog.setAttribute('aria-modal', 'true');
    this.dialog.setAttribute('aria-labelledby', 'inv-title');
    this.dialog.tabIndex = -1;

    const head = h('div', 'inv-head');
    const title = this.title;
    title.id = 'inv-title';
    const close = h('button', 'close-btn', '×');
    close.type = 'button';
    close.setAttribute('aria-label', 'Close inventory');
    close.addEventListener('click', () => this.close(true));
    head.append(title, close);

    // Backpack (slots 9..35) above the hotbar (0..8), as in the HUD.
    const size = game.inventory_max_size(), hotbar = game.hotbar_size();
    for (let i = 0; i < size; i++) {
      const slot = this.makeSlot(
        () => {
          if (this.shift && this.beside?.shiftSlot(i)) return;
          if (this.box && this.shift) game.store_slot(...this.box, i);
          else game.click_slot(i, this.shift);
        },
        (shift) => {
          if (shift && !this.holding()) this.box ? game.store_all(...this.box, i) : game.quick_move_all(i);
          else game.right_click_slot(i, shift);
        },
        { takeOne: () => game.take_one_slot(i), putOne: () => game.right_click_slot(i, true) },
      );
      if (i < hotbar) slot.root.append(h('span', 'key', String(i + 1)));
      slot.root.dataset.slot = i < hotbar ? 'hotbar' : 'pack';
      this.slots.push(slot);
    }
    const pack = h('div', 'inv-grid');
    pack.append(...this.slots.slice(hotbar).map((s) => s.root));
    const bar = h('div', 'inv-grid inv-hotbar');
    bar.append(...this.slots.slice(0, hotbar).map((s) => s.root));
    const gearRow = h('div', 'inv-gear');
    for (let i = 0; i < game.gear_slots(); i++) {
      const slot = this.makeSlot(
        () => game.click_gear(i, this.shift),
        (shift) => shift && game.click_gear(i, true),
      );
      slot.root.dataset.slot = 'gear';
      this.gear.push(slot);
      const cell = h('div', 'inv-gear-cell');
      cell.append(slot.root, h('span', 'inv-gear-name', game.gear_slot_name(i)));
      gearRow.append(cell);
    }
    // The left hand sits with the gear: a click puts the cursor stack in or takes it out (J swaps it with the hotbar).
    this.offhand = this.makeSlot(() => game.click_offhand(), () => {});
    this.offhand.root.dataset.slot = 'hand';
    const handCell = h('div', 'inv-gear-cell');
    handCell.append(this.offhand.root, h('span', 'inv-gear-name', 'Left hand (J)'));
    gearRow.append(handCell);
    const note = h(
      'p',
      'inv-note',
      `Mining ore by hand keeps ${game.hand_yield()} per block; the rest of that block's share of its deposit is lost. ` +
        `A miner recovers ${Math.round(game.miner_recovery() * 100)}% of what it drills, so build one early. ` +
        'Look at ore to see how big its deposit is.',
    );
    const items = h('section', 'inv-items');
    const boxHead = h('div', 'inv-box-head');
    const takeAll = h('button', 'secondary-btn', 'Take all');
    takeAll.type = 'button';
    takeAll.addEventListener('click', () => this.box && game.take_machine_output(...this.box));
    const sortBox = h('button', 'secondary-btn', 'Sort');
    sortBox.type = 'button';
    sortBox.title = `Merge stacks and order them by kind: ${game.sort_kinds()}`;
    sortBox.addEventListener('click', () => this.box && game.sort_box(...this.box));
    boxHead.append(h('h3', '', 'Storage box'), sortBox, takeAll);
    this.boxSection.append(boxHead, this.boxGrid);
    const sortPack = h('button', 'secondary-btn', 'Sort');
    sortPack.type = 'button';
    sortPack.title = `Merge stacks and order the backpack by kind: ${game.sort_kinds()} (the hotbar stays as you set it)`;
    sortPack.addEventListener('click', () => game.sort_inventory());
    const packHead = h('div', 'inv-box-head');
    packHead.append(h('h3', '', 'Backpack'), this.packPager.el, sortPack);
    this.packPager.onChange = () => this.showPackPage();
    this.packPager.wheel(pack);
    this.packUpgrade = new PackUpgrade(game);
    items.append(
      h('h3', '', 'Equipment'), gearRow, this.packUpgrade.el, packHead, pack, h('h3', '', 'Hotbar'), bar, note,
    );

    this.menu = new BuildMenu(game, icon);
    this.menu.onCraft = () => this.onCraft();
    // A creative world adds an "All items" tab beside the build menu.
    this.right = game.is_creative() ? withCreativeTabs(this.menu.el, new CreativeMenu(game, icon)) : this.menu.el;

    this.body.append(items, this.right, this.boxSection);
    this.dialog.append(head, this.body, this.controls.el);
    this.backdrop.append(this.dialog);

    const cur = h('div', 'slot inv-cursor hidden');
    const curIcon = h('canvas');
    curIcon.width = curIcon.height = ICON_PX;
    const curCount = h('span', 'count');
    cur.append(curIcon, curCount);
    this.cursor = { root: cur, icon: curIcon, count: curCount, item: -1, n: -1 };
    this.backdrop.append(cur);

    this.backdrop.addEventListener('pointerdown', (e) => {
      if (e.target !== this.backdrop) return;
      // Shift-right-click outside, holding a stack: throw one item out and stay.
      if (e.button === 2 && e.shiftKey && this.holding()) game.throw_cursor();
      else this.close(true);
    });
    this.backdrop.addEventListener('pointermove', (e) => this.moveCursor(e.clientX, e.clientY));
    this.backdrop.addEventListener('contextmenu', (e) => e.preventDefault());
    window.addEventListener('keydown', (e) => {
      if (!this.isOpen || e.repeat) return;
      if (e.code === 'KeyE' && !(e.target instanceof HTMLInputElement)) {
        e.preventDefault();
        this.close(true);
      } else if (e.key === 'Escape' && !e.defaultPrevented) {
        // (A search box's first Escape only empties it and prevents the default.)
        e.preventDefault();
        this.close(false);
      }
    });
    document.body.append(this.backdrop);
  }

  get isOpen(): boolean {
    return !this.backdrop.classList.contains('hidden');
  }

  /** Opens the screen, on the box at `box` if given, or with `beside` in place of the build menu. */
  open(box: [number, number, number] | null = null, beside: Beside | null = null): void {
    if (this.beside !== beside) {
      this.beside?.el.remove();
      this.beside?.closed();
      if (beside) this.body.append(beside.el);
    }
    this.beside = beside;
    this.box = box;
    const n = box ? this.game.box_slots(...box).length / 2 : 0;
    while (this.boxSlots.length < n) {
      const i = this.boxSlots.length;
      const slot = this.makeSlot(
        () => this.box && this.game.click_box(...this.box, i, this.shift),
        (shift) => {
          if (!this.box) return;
          if (shift && !this.holding()) this.game.take_all(...this.box, i);
          else this.game.right_click_box(...this.box, i, shift);
        },
        {
          takeOne: () => this.box && this.game.take_one_box(...this.box, i),
          putOne: () => this.box && this.game.right_click_box(...this.box, i, true),
        },
      );
      slot.root.dataset.slot = 'box';
      this.boxSlots.push(slot);
      this.boxGrid.append(slot.root);
    }
    // Slots are kept between boxes; a smaller box hides the extras (an upgraded box has more).
    this.boxSlots.forEach((s, i) => s.root.classList.toggle('hidden', i >= n));
    this.boxSection.classList.toggle('hidden', !box);
    this.right.classList.toggle('hidden', !!box || !!beside);
    this.dialog.classList.toggle('inv-with-box', !!box || !!beside);
    this.title.textContent = beside ? beside.title : box ? 'Storage box' : 'Inventory & build';
    this.backdrop.classList.remove('hidden');
    this.version = -1;
    this.boxKey = '';
    this.update();
    this.dialog.focus();
  }

  close(resume: boolean): void {
    if (!this.isOpen) return;
    this.backdrop.classList.add('hidden');
    this.menu.hideInfo();
    this.beside?.el.remove();
    this.beside?.closed();
    this.beside = null;
    this.onClose(resume);
  }

  /** Redraws whatever changed since the last call. Cheap when nothing did; call every frame. */
  update(): void {
    if (!this.isOpen) return;
    if (this.beside && !this.beside.update()) {
      this.close(true); // the machine is gone
      return;
    }
    const g = this.game;
    const v = g.inventory_version();
    const boxData = this.box ? g.box_slots(...this.box) : null;
    if (boxData && boxData.length === 0) {
      this.close(false); // the box is gone
      return;
    }
    const boxKey = boxData ? boxData.join(',') : '';
    if (v === this.version && boxKey === this.boxKey) return this.showControls();
    this.version = v;
    this.boxKey = boxKey;
    if (boxData) {
      for (let i = 0; i < boxData.length / 2; i++) this.drawSlot(this.boxSlots[i], boxData[i * 2], boxData[i * 2 + 1]);
    }

    const selected = g.selected_slot();
    this.slots.forEach((s, i) => {
      this.drawSlot(s, g.slot_item(i), g.slot_count(i));
      s.root.classList.toggle('selected', i === selected);
    });
    this.showPackPage();
    this.gear.forEach((s, i) => {
      const item = g.worn_item(i);
      this.drawSlot(s, item, item ? 1 : 0);
      if (!item) s.root.title = `${g.gear_slot_name(i)}: nothing worn`;
    });
    this.drawSlot(this.offhand, g.offhand_item(), g.offhand_count());
    this.packUpgrade.update();
    this.drawSlot(this.cursor, g.cursor_item(), g.cursor_count());
    this.cursor.root.classList.toggle('hidden', g.cursor_count() === 0);

    if (!this.box && !this.beside) this.menu.update();
    this.showControls();
  }

  /** The controls bar for the slot under the pointer (looked up every frame: machine slots are rebuilt as they change)
   * and the held stack. */
  private showControls(): void {
    const el = document.elementFromPoint(...this.pointer)?.closest<HTMLElement>('[data-slot]');
    const kind = (el?.dataset.slot ?? null) as SlotContext['kind'];
    const item = Number(el?.dataset.item || 0);
    const held = this.holding() ? this.game.cursor_item() : 0;
    this.controls.show({ kind, item, held, shift: kind ? this.shiftLabel(kind, item) : '' });
  }

  /** What Shift-click does with the stack in a slot of `kind`, in the controls bar's words. */
  private shiftLabel(kind: string, item: number): string {
    if (kind === 'box') return 'Move to inventory';
    if (this.box) return 'Move to the box';
    if (this.beside?.wants(item)) return `Put in the ${this.beside.title}`;
    if (this.game.gear_blurb(item)) return 'Wear it';
    return kind === 'hotbar' ? 'Move to backpack' : 'Move to hotbar';
  }

  /** Shows the backpack slots of the current page (the pack grows with a hauler pack, so pages come and go). */
  private showPackPage(): void {
    const hotbar = this.game.hotbar_size(), open = this.game.inventory_size();
    const pages = Math.max(1, Math.ceil((open - hotbar) / PAGE_SLOTS));
    this.packPager.set(Array.from({ length: pages }, (_, i) => ({ title: `Backpack page ${i + 1} of ${pages}` })));
    const first = hotbar + this.packPager.page * PAGE_SLOTS;
    this.slots.forEach((s, i) => {
      if (i >= hotbar) s.root.classList.toggle('hidden', i >= open || i < first || i >= first + PAGE_SLOTS);
    });
  }

  /** Whether the cursor holds a stack. */
  private holding(): boolean {
    return this.game.cursor_count() > 0;
  }

  /** A slot that calls `click` on a left click (`this.shift` tells whether Shift was held) and `rightClick`
   * on a right click, with whether Shift was held. With Ctrl (or Cmd) held instead, `ctrl` takes over: a left click
   * takes one item onto the cursor, a right click puts one down (nothing when the hand is empty). */
  private makeSlot(
    click: () => void,
    rightClick: (shift: boolean) => void,
    ctrl?: { takeOne: () => void; putOne: () => void },
  ): SlotView {
    const root = h('div', 'slot inv-slot');
    const icon = h('canvas');
    icon.width = icon.height = ICON_PX;
    const count = h('span', 'count');
    root.append(icon, count);
    root.addEventListener('pointerdown', (e) => {
      if (e.button !== 0 && e.button !== 2) return;
      e.preventDefault();
      this.drag.press(e, !this.holding() && !!root.dataset.item);
      this.shift = e.shiftKey;
      if (ctrl && !e.shiftKey && (e.ctrlKey || e.metaKey)) {
        if (e.button === 0) ctrl.takeOne();
        else if (this.holding()) ctrl.putOne();
      } else if (e.button === 2) rightClick(e.shiftKey);
      else click();
      this.moveCursor(e.clientX, e.clientY);
      this.update();
    });
    return { root, icon, count, item: -1, n: -1 };
  }

  private drawSlot(s: SlotView, item: number, n: number): void {
    const uses = this.game.tool_uses(item);
    const amount = uses > 0 ? ` (${n} of ${uses} uses left)` : n > 1 ? ` ×${n}` : '';
    const effect = n > 0 ? this.game.gear_blurb(item) : '';
    s.root.title = n > 0 ? `${this.game.item_name(item)}${amount}${effect ? `: ${effect}` : ''}` : '';
    s.root.dataset.item = n > 0 ? String(item) : '';
    if (item === s.item && n === s.n) return;
    const ctx = s.icon.getContext('2d')!;
    ctx.clearRect(0, 0, ICON_PX, ICON_PX);
    if (n > 0) ctx.drawImage(this.icon(item), 0, 0);
    showAmount(this.game, s.root, s.count, item, n);
    s.item = item;
    s.n = n;
  }

  /** Follows the pointer (on moves and clicks): the held stack, and where `showControls` looks. */
  private moveCursor(x: number, y: number): void {
    this.pointer = [x, y];
    this.cursor.root.style.transform = `translate(${x}px, ${y}px) translate(-50%, -50%)`;
  }
}
