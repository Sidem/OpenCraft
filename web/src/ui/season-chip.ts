// The season chip under the XYZ readout (compass.ts): a tiny year ring with today's mark, the season's icon, the day
// of the season ("3/9") and the clock, gold by day and blue by night. Owned and updated by the Minimap (it shows and
// hides with it); the layout height is mirrored as CHIP_PX there. Words only in the tooltip; the full calendar is
// calendar.ts.

import './season-chip.css';
import type { Game } from '../wasm/engine.js';
import { clock } from '../render/sky';
import { h } from './dom';
import { onRing, seasonIcon, seasonRing, SEASONS, svgEl } from './seasons';

export const CHIP_PX = 30; // must match .season-chip height in season-chip.css

export class SeasonChip {
  readonly el = h('div', 'season-chip');
  private readonly mark = svgEl('circle', { r: 2.4, class: 'sc-mark' });
  private readonly seasonSlot = h('span', 'sc-season');
  private readonly day = h('span', 'sc-day');
  private readonly time = h('span', 'sc-time');
  private key = '';

  constructor(private readonly game: Game) {
    const dial = svgEl('svg', { viewBox: '0 0 22 22', class: 'sc-dial', 'aria-hidden': 'true' });
    dial.append(...seasonRing(11, 8, 3.2, 0.012), this.mark);
    this.el.append(dial, this.seasonSlot, this.day, h('span', 'sc-gap'), this.time);
  }

  /** Call every frame while shown; touches the DOM only when something it shows changed. */
  update(): void {
    const g = this.game;
    const season = g.season(), dos = g.day_of_season(), per = g.days_per_season();
    const t = g.time_of_day(), sun = g.sun_time(), up = sun >= 0.25 && sun < 0.75;
    const turn = Math.round(g.year_fraction() * 360);
    const time = clock(t);
    const key = `${season}|${dos}|${time}|${turn}|${up}`;
    if (key === this.key) return;
    const was = this.key.split('|');
    this.key = key;
    const [x, y] = onRing(11, 8, turn / 360);
    this.mark.setAttribute('cx', x.toFixed(2));
    this.mark.setAttribute('cy', y.toFixed(2));
    if (was[0] !== String(season)) this.seasonSlot.replaceChildren(seasonIcon(season));
    this.time.classList.toggle('night', !up);
    this.day.textContent = `${dos}/${per}`;
    this.time.textContent = time;
    this.el.title = `${SEASONS[season].name}, day ${dos} of ${per} (day ${g.day_number()} of the world), ${time}. Shift+N: calendar`;
  }
}
