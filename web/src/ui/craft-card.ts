// The facts block of a build-menu info card (crafting.ts): what a building is at a glance. The engine's `item_facts`
// gives one row a line, `kind<TAB>label<TAB>value`: size, use (electricity drawn), fuel (what it burns), make (power
// produced), none (needs no energy) and stat (any other number). The first four get a field of their own in a fixed
// place, so the player finds the size, the fuel and the kW without reading the description; stats are a short list.
// To show a new kind of row: give it a case in `FIELD_NAMES` (and a colour in craft-card.css).

import './craft-card.css';
import type { Game } from '../wasm/engine.js';
import { h } from './dom';

/** The kinds that get a field, in the order they are drawn. */
const FIELD_NAMES: Record<string, string> = { size: 'Size', use: 'Electricity', fuel: 'Fuel', make: 'Produces', none: 'Energy' };
/** Values this long take a whole row (a fuel list). */
const WIDE = 16;

interface Row {
  kind: string;
  label: string;
  value: string;
}

/** The fields and stats of the building recipe output `item` places, or null when it is not a machine. */
export function factsBlock(game: Game, item: number): HTMLElement | null {
  const text = game.item_facts(item);
  if (!text) return null;
  const rows: Row[] = text.split('\n').map((line) => {
    const [kind, label, value] = line.split('\t');
    return { kind, label, value };
  });
  const root = h('div', 'card-facts');
  const fields = h('div', 'card-fields');
  for (const r of rows.filter((r) => r.kind in FIELD_NAMES)) {
    const field = h('div', `card-field ${r.kind}${r.value.length > WIDE ? ' wide' : ''}`);
    field.append(h('span', 'card-field-name', FIELD_NAMES[r.kind]), h('span', 'card-field-value', r.value));
    fields.append(field);
  }
  root.append(fields);
  const stats = rows.filter((r) => r.kind === 'stat');
  if (stats.length > 0) {
    const list = h('dl', 'card-stats');
    for (const s of stats) list.append(h('dt', '', s.label), h('dd', '', s.value));
    root.append(list);
  }
  return root;
}
