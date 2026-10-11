// Creative item picker (creative worlds only): in the inventory screen, an "All items" tab beside the build
// menu. Every item the engine offers is a tile (icon, name) with a search box; click takes a full stack,
// Shift-click takes one, straight into the backpack. The tiles reuse the build menu's look (crafting.css) and, like it,\n// come in pages that fit the height (no scrollbar; the wheel and the pager flip them).
// `withCreativeTabs` joins the two menus into one column with a tab row; normal worlds never build any of it.

import './crafting.css';
import './creative.css';
import type { Game } from '../wasm/engine.js';
import { TILE_GAP, TILE_H } from './crafting';
import { h } from './dom';
import { Pager, tilesPerPage } from './pager';

const ICON_PX = 64;

export class CreativeMenu {
  readonly el = h('section', 'inv-creative');

  constructor(game: Game, icon: (item: number) => HTMLCanvasElement) {
    const search = h('input', 'craft-search');
    search.type = 'search';
    search.placeholder = 'Search all items';
    search.setAttribute('aria-label', 'Search all items');
    const head = h('div', 'craft-head');
    head.append(search);
    const grid = h('div', 'craft-grid');
    const tiles: { root: HTMLButtonElement; name: string }[] = [];
    for (const item of game.creative_items()) {
      const name = game.item_name(item);
      const root = h('button', 'craft-tile');
      root.type = 'button';
      root.dataset.state = 'ready';
      root.title = `${name}: click for a stack, Shift-click for one`;
      const canvas = h('canvas', 'craft-icon');
      canvas.width = canvas.height = ICON_PX;
      canvas.getContext('2d')!.drawImage(icon(item), 0, 0);
      root.append(canvas, h('span', 'craft-name', name));
      root.addEventListener('click', (e) => game.creative_give(item, !e.shiftKey));
      tiles.push({ root, name: name.toLowerCase() });
    }
    const pager = new Pager();
    // The tiles that match the search, a page of them in the grid.
    const show = () => {
      const words = search.value.trim().toLowerCase().split(/\s+/).filter(Boolean);
      const found = tiles.filter((t) => words.every((w) => t.name.includes(w)));
      const size = tilesPerPage(grid, TILE_H, TILE_GAP);
      if (size === 0) return; // hidden: laid out when the grid gets a size
      pager.set(Array.from({ length: Math.ceil(found.length / size) }, (_, i) => ({ title: `Page ${i + 1}` })));
      grid.replaceChildren(...found.slice(pager.page * size, (pager.page + 1) * size).map((t) => t.root));
    };
    pager.onChange = show;
    pager.wheel(grid);
    search.addEventListener('input', () => {
      pager.page = 0;
      show();
    });
    new ResizeObserver(show).observe(grid);
    this.el.append(head, grid, pager.el);
  }
}

/** One column holding the build menu and the item picker, with tabs to switch (the picker shows first). */
export function withCreativeTabs(build: HTMLElement, items: CreativeMenu): HTMLElement {
  const tabs = h('div', 'craft-filters');
  const show = (picker: boolean, buttons: HTMLButtonElement[]) => {
    items.el.classList.toggle('hidden', !picker);
    build.classList.toggle('hidden', picker);
    buttons.forEach((b, i) => b.setAttribute('aria-pressed', String((i === 0) === picker)));
  };
  const buttons = [h('button', 'craft-filter', 'All items'), h('button', 'craft-filter', 'Build')];
  buttons.forEach((b, i) => {
    b.type = 'button';
    b.addEventListener('click', () => show(i === 0, buttons));
  });
  tabs.append(...buttons);
  show(true, buttons);
  const column = h('div', 'inv-right');
  column.append(tabs, items.el, build);
  return column;
}
