// A building's Mks as one build-menu tile. `Families.fold` turns the tiles on show (in menu order) into the grid's list:
// a building with two or more Mks on show becomes one tile (the icon, the name without its Mk, the Mks it covers, the
// best state of its Mks) and anything else stays as it is. Clicking the tile opens a small popover beside it with the
// Mks' own tiles, so crafting, Shift-click, right-click take-apart and the hover card all work as in the grid.
// The popover never blocks: no backdrop, it stays open while you craft several Mks, a click anywhere else (which still
// does what it was aimed at), Escape, another page or closing the inventory closes it, and the arrow keys move between
// its Mks. Family tiles live as long as the menu, so an open popover keeps its anchor through every redraw.
// Grouping rules (pure, tested): craft-tiers.ts. A new building needs nothing here: the engine's `recipe_order` groups it.

import './craft-family.css';
import type { Tile } from './crafting';
import { baseName, bestState, familyKey, foldFamilies, mkOf, mkRange } from './craft-tiers';
import { h } from './dom';

interface FamilyTile extends Tile {
  members: Tile[];
  /** The label with the Mks it covers. */
  mks: HTMLElement;
}

export class Families {
  private readonly tiles = new Map<string, FamilyTile>();
  private readonly pop = h('div', 'craft-family-pop hidden');
  private readonly title = h('div', 'craft-pop-title');
  private readonly row = h('div', 'craft-pop-row');
  private open: FamilyTile | null = null;

  /** `host` is the positioned element the popover goes in; `onClose` runs when it closes (to drop a hover card). */
  constructor(
    private readonly host: HTMLElement,
    private readonly onClose: () => void,
    private readonly onOpen: () => void,
  ) {
    this.pop.setAttribute('role', 'group');
    this.pop.append(this.title, this.row);
    host.append(this.pop);
    this.row.addEventListener('keydown', (e) => this.arrow(e));
    document.addEventListener('pointerdown', (e) => this.outside(e), true);
    document.addEventListener('keydown', (e) => this.escape(e), true);
  }

  /** The grid's list for the tiles on show: one tile per building with several Mks. */
  fold(visible: Tile[]): Tile[] {
    return foldFamilies(visible).map((list) => (list.length > 1 ? this.family(list) : list[0]));
  }

  /** After the grid was filled with `shown`: closes the popover if its tile left, else keeps it up to date. */
  sync(shown: Tile[]): void {
    const f = this.open;
    if (!f) return;
    if (!shown.includes(f)) this.close();
    else {
      this.fill(f);
      this.place(f);
    }
  }

  /** The popover when `t` is one of the Mks in it, so the hover card can go beside the whole popover. */
  anchorOf(t: Tile): HTMLElement | undefined {
    return this.open?.members.includes(t) ? this.pop : undefined;
  }

  close(): void {
    const f = this.open;
    if (!f) return;
    f.root.setAttribute('aria-expanded', 'false');
    this.open = null;
    this.pop.classList.add('hidden');
    this.onClose();
  }

  /** The tile of the building whose Mks are `members`, made on first need and brought up to date. */
  private family(members: Tile[]): FamilyTile {
    const key = familyKey(members[0]);
    const f = this.tiles.get(key) ?? this.make(members);
    this.tiles.set(key, f);
    f.members = members;
    const lead = members.reduce((a, t) => (t.score < a.score ? t : a));
    f.item = lead.item;
    f.id = lead.id;
    f.score = lead.score;
    f.order = members[0].order;
    f.state = bestState(members.map((t) => t.state));
    f.root.dataset.state = f.state;
    f.root.classList.toggle('craft-goal', members.some((t) => t.root.classList.contains('craft-goal')));
    const range = mkRange(members.map(mkOf));
    if (f.mks.textContent !== range) f.mks.textContent = range;
    f.root.setAttribute('aria-label', `${f.name}, ${range}: pick a Mk`);
    return f;
  }

  private make(members: Tile[]): FamilyTile {
    const first = members[0];
    const root = h('button', 'craft-tile craft-family');
    root.type = 'button';
    root.setAttribute('aria-haspopup', 'true');
    root.setAttribute('aria-expanded', 'false');
    const canvas = h('canvas', 'craft-icon');
    const from = first.root.querySelector('canvas');
    if (from) {
      canvas.width = from.width;
      canvas.height = from.height;
      canvas.getContext('2d')!.drawImage(from, 0, 0);
    }
    const name = baseName(first.name);
    const mks = h('span', 'craft-mks');
    root.append(canvas, h('span', 'craft-name', name), mks, h('span', 'craft-lock'));
    const f: FamilyTile = { ...first, name, words: '', root, members, mks };
    // A detail of 0 is a click from the keyboard: move into the popover then.
    root.addEventListener('click', (e) => (this.open === f ? this.close() : this.show(f, e.detail === 0)));
    return f;
  }

  private show(f: FamilyTile, keyboard: boolean): void {
    this.close();
    this.open = f;
    f.root.setAttribute('aria-expanded', 'true');
    this.title.textContent = `${f.name}: pick a Mk`;
    this.onOpen();
    this.pop.classList.remove('hidden');
    this.fill(f);
    this.place(f);
    if (keyboard) (this.row.querySelector<HTMLElement>('[data-state="ready"]') ?? (this.row.firstElementChild as HTMLElement | null))?.focus();
  }

  /** Puts the Mks' tiles in the popover (each marked with its Mk), leaving them alone when they are already there. */
  private fill(f: FamilyTile): void {
    const roots = f.members.map((m) => m.root);
    for (const m of f.members) {
      if (!m.root.querySelector('.craft-mk')) m.root.prepend(h('span', 'craft-mk', `Mk${mkOf(m)}`));
    }
    const same = roots.length === this.row.children.length && roots.every((r, i) => r === this.row.children[i]);
    if (!same) this.row.replaceChildren(...roots);
  }

  /** Below the family's tile, or above it when there is no room below, kept inside the host. */
  private place(f: FamilyTile): void {
    const host = this.host.getBoundingClientRect();
    const at = f.root.getBoundingClientRect();
    const w = this.pop.offsetWidth;
    const hgt = this.pop.offsetHeight;
    const left = Math.max(0, Math.min(at.left - host.left, host.width - w));
    let top = at.bottom - host.top + 4;
    if (top + hgt > host.height && at.top - host.top - hgt - 4 >= 0) top = at.top - host.top - hgt - 4;
    this.pop.style.left = `${left}px`;
    this.pop.style.top = `${Math.max(0, top)}px`;
  }

  /** A press anywhere but the popover and its tile closes it, and goes on to whatever it was aimed at. */
  private outside(e: PointerEvent): void {
    const f = this.open;
    const at = e.target as Node;
    if (f && !this.pop.contains(at) && !f.root.contains(at)) this.close();
  }

  /** Escape closes only the popover (not the inventory behind it) and gives focus back to its tile. */
  private escape(e: KeyboardEvent): void {
    const f = this.open;
    if (e.key !== 'Escape' || !f) return;
    e.preventDefault();
    e.stopPropagation();
    const inside = this.pop.contains(document.activeElement);
    this.close();
    if (inside) f.root.focus();
  }

  private arrow(e: KeyboardEvent): void {
    const step = { ArrowRight: 1, ArrowDown: 1, ArrowLeft: -1, ArrowUp: -1 }[e.key];
    if (!step) return;
    const all = Array.from(this.row.children) as HTMLElement[];
    const at = all.indexOf(document.activeElement as HTMLElement);
    all[Math.max(0, Math.min(all.length - 1, at + step))]?.focus();
    e.preventDefault();
  }
}
