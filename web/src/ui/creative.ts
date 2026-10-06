// Creative item picker (creative worlds only): in the inventory screen, an "All items" tab beside the build
// menu. Every item the engine offers is a tile (icon, name) with a search box; click takes a full stack,
// Shift-click takes one, straight into the backpack. The tiles reuse the build menu's look (crafting.css).
// `withCreativeTabs` joins the two menus into one column with a tab row; normal worlds never build any of it.

import './crafting.css';
import './creative.css';
import type { Game } from '../wasm/engine.js';
import { h } from './dom';

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
      grid.append(root);
    }
    search.addEventListener('input', () => {
      const words = search.value.trim().toLowerCase().split(/\s+/).filter(Boolean);
      for (const t of tiles) t.root.classList.toggle('hidden', !words.every((w) => t.name.includes(w)));
    });
    this.el.append(head, grid);
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
