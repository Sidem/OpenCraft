// The chips beside the world map that choose what the maps show (the rules and the saved state are map-filter.ts):
// one chip each for machines, towns, terraforming sites and satellite cover, then a row of chips per ore and per pin
// kind, with the row's name toggling the whole row. A hidden chip is struck through. To add a group of marks: a
// category in `markCategory` and a line in `GROUPS`.

import './map-filter-panel.css';
import type { Game } from '../wasm/engine.js';
import { button, h } from './dom';
import { type MapFilter, oreCategory, pinCategory } from './map-filter.ts';
import { hex, type PinKind } from './pins';

/** The chips that stand for one category of marks each. */
const GROUPS: [category: string, label: string][] = [
  ['machines', 'Machines'],
  ['towns', 'Towns'],
  ['sites', 'Sites'],
  ['cover', 'Satellite cover'],
  ['dry', 'Worked out'],
];

export class FilterPanel {
  readonly el = h('section', 'mf');
  private readonly chips: [HTMLElement, string][] = [];

  constructor(game: Game, private readonly filter: MapFilter, pinKinds: readonly PinKind[]) {
    const ores = game.ore_guide().trim().split('\n').map((line) => line.split('\t'));
    this.el.append(h('h3', '', 'Show on the map'));
    const marks = h('div', 'mf-row');
    for (const [category, label] of GROUPS) marks.append(this.chip(category, label, null));
    this.el.append(marks);
    this.row('Ores', ores.map(([, colour, name]) => [oreCategory(Number(colour)), name, hex(Number(colour))]));
    this.row('Pins', pinKinds.map((k) => [pinCategory(k.id), k.label, k.color]));
    filter.subscribe(() => this.refresh());
    this.refresh();
  }

  /** A row of chips under a name that shows or hides them all. */
  private row(name: string, items: [category: string, label: string, colour: string][]): void {
    const categories = items.map((i) => i[0]);
    const head = button('mf-name', name, () => this.filter.set(categories, categories.some((c) => !this.filter.shows(c))));
    head.title = `Show or hide all ${name.toLowerCase()}`;
    const row = h('div', 'mf-row');
    for (const [category, label, colour] of items) row.append(this.chip(category, label, colour));
    this.el.append(head, row);
  }

  private chip(category: string, label: string, colour: string | null): HTMLElement {
    const chip = button('mf-chip', label, () => this.filter.toggle(category));
    if (colour) chip.style.setProperty('--kind', colour);
    chip.classList.toggle('mf-swatch', colour !== null);
    this.chips.push([chip, category]);
    return chip;
  }

  private refresh(): void {
    for (const [chip, category] of this.chips) {
      const on = this.filter.shows(category);
      chip.classList.toggle('off', !on);
      chip.setAttribute('aria-pressed', String(on));
    }
  }
}
