// "Show on map" on the world map (worldmap.ts): after the analytics hub names a kind of machine, every machine of that
// kind gets a ring in its health colour (green at full speed, amber slowed, red badly held back, a grey dashed ring
// when idle), one picked machine a second ring, and a chip at the top says what is shown and clears it. Resting on a
// ring names its pace and what holds it back. The list is read live from the engine (`analytics_machines_of`), so rings
// follow the machines; the parsing and the view fit are pure (map-focus.ts).

import './map-focus.css';
import type { Game } from '../wasm/engine.js';
import { CAUSES, healthColour } from './analytics-machines';
import { button, h } from './dom';
import type { Info } from './map-filter';
import { type Focused, parseMachines, tally } from './map-focus';

/** Ring radius and width in screen pixels (a machine mark is about 5 px wide). */
const RING_PX = 9;
const RING_WIDTH = 2.5;
const IDLE = '#a8a29e';

export class MapFocus {
  /** The chip saying what is shown, for the map to place over its view. */
  readonly chip = h('div', 'fo-chip hidden');
  /** Called when the chip's Clear is pressed. */
  onClear: () => void = () => {};
  private block = -1;
  private picked: { x: number; z: number } | null = null;
  private list: Focused[] = [];

  constructor(private readonly game: Game) {}

  get active(): boolean {
    return this.block >= 0;
  }

  get machines(): readonly Focused[] {
    return this.list;
  }

  /** Shows every machine of kind `block` (`picked` gets a second ring). */
  show(block: number, picked?: { x: number; z: number }): void {
    this.block = block;
    this.picked = picked ?? null;
    this.refresh();
  }

  clear(): void {
    this.block = -1;
    this.list = [];
    this.chip.classList.add('hidden');
  }

  /** Reads the machines again and rewrites the chip. */
  refresh(): void {
    if (!this.active) return;
    this.list = parseMachines(this.game.analytics_machines_of(this.block), this.game.analytics_kind_fields());
    const name = this.game.item_name(this.block);
    const t = tally(this.list);
    const parts = [
      t.full > 0 ? `${t.full} at full speed` : '',
      ...CAUSES.map((c, i) => (t.held[i] ? `${t.held[i]} ${c.what}` : '')),
      t.idle > 0 ? `${t.idle} idle` : '',
    ];
    const text = this.list.length === 0 ? `No ${name} has run lately` : `${name} ×${this.list.length}: ${parts.filter(Boolean).join(', ')}`;
    if (this.chip.firstChild?.textContent !== text) {
      this.chip.replaceChildren(h('span', 'fo-text', text), button('secondary-btn fo-clear', 'Clear', () => this.onClear()));
    }
    this.chip.classList.remove('hidden');
  }

  /** Draws the rings; `at` turns a world column into overlay pixels. */
  draw(c: CanvasRenderingContext2D, at: (x: number, z: number) => [number, number], dpr: number): void {
    if (!this.active) return;
    const r = RING_PX * dpr;
    c.save();
    for (const m of this.list) {
      const [x, y] = at(m.x + 0.5, m.z + 0.5);
      c.beginPath();
      c.arc(x, y, r, 0, Math.PI * 2);
      c.lineWidth = (RING_WIDTH + 2) * dpr;
      c.strokeStyle = 'rgba(0, 0, 0, 0.6)';
      c.setLineDash([]);
      c.stroke();
      c.lineWidth = RING_WIDTH * dpr;
      c.strokeStyle = m.percent < 0 ? IDLE : healthColour(m.percent);
      c.setLineDash(m.percent < 0 ? [3 * dpr, 3 * dpr] : []);
      c.stroke();
      if (this.picked && this.picked.x === m.x && this.picked.z === m.z) {
        c.beginPath();
        c.arc(x, y, r * 1.6, 0, Math.PI * 2);
        c.lineWidth = 1.5 * dpr;
        c.strokeStyle = '#ffffff';
        c.setLineDash([]);
        c.stroke();
      }
    }
    c.restore();
  }

  /** Hover lines for the ringed machines within `reach` blocks of column (x, z), nearest first. */
  near(x: number, z: number, reach: number): Info[] {
    if (!this.active) return [];
    const name = this.game.item_name(this.block);
    return this.list
      .map((m) => ({ m, d: Math.hypot(m.x + 0.5 - x, m.z + 0.5 - z) }))
      .filter((e) => e.d <= reach)
      .sort((a, b) => a.d - b.d)
      .map(({ m }) => {
        const state = m.percent < 0 ? 'Nothing to do right now.' : m.cause < 0 ? 'At full speed.' : `Mostly ${CAUSES[m.cause]?.what ?? 'held back'}.`;
        return {
          colour: m.percent < 0 ? IDLE : healthColour(m.percent),
          title: m.percent < 0 ? `${name}: idle` : `${name}: ${m.percent}%`,
          detail: `${state} At ${m.x}, ${m.y}, ${m.z}.`,
        };
      });
  }

  /** The name the engine gives these machines' map marks, so the map can drop its own line for them. */
  get name(): string {
    return this.active ? this.game.item_name(this.block) : '';
  }
}
