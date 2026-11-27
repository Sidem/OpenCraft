// Calendar panel (Shift+N): the year as a ring of the four seasons with each day's daylight as a bar and today's mark,
// today's day and night as a 24-hour bar, the weather now and the days ahead (forecast.ts), and the seasons to come
// with the days to go and how fast plants grow then.
// Everything is read from the engine's calendar getters (api/calendar.rs, daytime.rs) when it opens and once a
// second while open; words only in tooltips. The frame is the machine panel's (base.css `.mp`).

import './calendar.css';
import type { Game } from '../wasm/engine.js';
import { clock } from '../render/sky';
import { button, h } from './dom';
import { drawForecast } from './forecast';
import { icon, onRing, seasonIcon, seasonRing, SEASONS, svgEl } from './seasons';

const RING = 240; // the year ring's view box
const C = RING / 2;
const REFRESH_MS = 1000;

export class CalendarPanel {
  /** Called after the screen closes. `resume` is true when play should continue. */
  onClose: (resume: boolean) => void = () => {};

  private readonly backdrop = h('div', 'mp-backdrop hidden');
  private readonly dialog = h('div', 'mp calendar');
  private readonly ring = svgEl('svg', { viewBox: `0 0 ${RING} ${RING}`, class: 'cal-ring' });
  private readonly centre = h('div', 'cal-centre');
  private readonly dayBar = h('div', 'cal-daybar');
  private readonly dayRow = h('div', 'cal-dayrow');
  private readonly sky = h('div', 'cal-sky');
  private readonly next = h('div', 'cal-next');
  private timer = 0;

  constructor(private readonly game: Game) {
    this.dialog.setAttribute('role', 'dialog');
    this.dialog.setAttribute('aria-modal', 'true');
    this.dialog.tabIndex = -1;
    const head = h('div', 'mp-head');
    const close = button('close-btn', '×', () => this.close(true));
    close.setAttribute('aria-label', 'Close');
    head.append(h('h2', '', 'Calendar'), h('span', 'mp-keys', 'N or click outside to return'), close);
    const wheel = h('div', 'cal-wheel');
    wheel.append(this.ring, this.centre);
    const body = h('div', 'mp-body');
    body.append(wheel, this.dayBar, this.dayRow, this.sky, this.next);
    this.dialog.append(head, body);
    this.backdrop.append(this.dialog);
    this.backdrop.addEventListener('pointerdown', (e) => {
      if (e.target === this.backdrop) this.close(true);
    });
    this.backdrop.addEventListener('contextmenu', (e) => e.preventDefault());
    window.addEventListener('keydown', (e) => {
      if (!this.isOpen || e.repeat) return;
      if (e.code === 'KeyN') {
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
    this.draw();
    this.backdrop.classList.remove('hidden');
    this.dialog.focus();
    this.timer = window.setInterval(() => this.draw(), REFRESH_MS);
  }

  close(resume: boolean): void {
    if (!this.isOpen) return;
    window.clearInterval(this.timer);
    this.backdrop.classList.add('hidden');
    this.onClose(resume);
  }

  private draw(): void {
    const g = this.game;
    const per = g.days_per_season(), year = 4 * per;
    const season = g.season(), dos = g.day_of_season();
    const today = season * per + dos - 1; // day of the year, from 0
    this.drawRing(year, today, g.year_fraction());
    const worldYear = Math.floor((g.day_number() - 1) / year) + 1;
    const big = seasonIcon(season);
    big.classList.add('cal-big');
    this.centre.replaceChildren(big, h('div', 'cal-dos', `${dos}/${per}`), h('div', 'cal-year', `Y${worldYear}`));
    this.centre.title = `${SEASONS[season].name}, day ${dos} of ${per}; year ${worldYear}, day ${g.day_number()} of the world`;
    this.drawDay(g.daylight_share(today), g.time_of_day());
    drawForecast(g, this.sky);
    this.drawNext(season, per - dos + 1, per);
  }

  /** The season band, a daylight bar per day pointing in from it, and today's mark on the rim. */
  private drawRing(year: number, today: number, at: number): void {
    const g = this.game;
    const parts: SVGElement[] = [svgEl('circle', { cx: C, cy: C, r: 103, class: 'cal-track' }), ...seasonRing(C, 103, 14)];
    for (let d = 0; d < year; d++) {
      const share = g.daylight_share(d), mid = (d + 0.5) / year;
      const [x0, y0] = onRing(C, 92, mid);
      const [x1, y1] = onRing(C, 92 - share * 60, mid);
      const bar = svgEl('line', { x1: x0, y1: y0, x2: x1, y2: y1, class: d === today ? 'cal-sun today' : 'cal-sun' });
      const s = Math.floor(d / (year / 4));
      bar.append(svgEl('title', {}));
      bar.firstChild!.textContent = `${SEASONS[s].name} day ${(d % (year / 4)) + 1}: ${hours(share)} h of daylight`;
      parts.push(bar);
    }
    for (let s = 0; s < 4; s++) {
      const [x, y] = onRing(C, 103, (s + 0.5) / 4);
      const i = seasonIcon(s);
      for (const [k, v] of Object.entries({ x: x - 6, y: y - 6, width: 12, height: 12 })) i.setAttribute(k, String(v));
      i.classList.add('cal-band-icon');
      parts.push(i);
    }
    const [tx, ty] = onRing(C, 118, at);
    const [lx, ly] = onRing(C, 112, at - 0.012);
    const [rx, ry] = onRing(C, 112, at + 0.012);
    const [ix, iy] = onRing(C, 96, at);
    parts.push(svgEl('path', { d: `M${lx} ${ly}L${ix} ${iy}L${rx} ${ry}L${tx} ${ty}Z`, class: 'cal-now' }));
    this.ring.replaceChildren(...parts);
  }

  /** Today's 24 hours: night, the sunlit middle and the clock's place in it; under it the hours of each. */
  private drawDay(share: number, t: number): void {
    const rise = (1 - share) / 2;
    const sun = h('div', 'cal-daylight');
    sun.style.left = `${rise * 100}%`;
    sun.style.width = `${share * 100}%`;
    const now = h('div', 'cal-clock');
    now.style.left = `${t * 100}%`;
    this.dayBar.replaceChildren(sun, now);
    this.dayBar.title = `Sunrise ${clock(rise)}, sunset ${clock(1 - rise)}; now ${clock(t)}`;
    const day = h('span', 'cal-hours'), night = h('span', 'cal-hours night');
    day.append(icon('sun'), `${hours(share)} h`);
    night.append(`${hours(1 - share)} h`, icon('moon'));
    day.title = 'Daylight today';
    night.title = 'Night today';
    this.dayRow.replaceChildren(day, h('span', 'cal-now-time', clock(t)), night);
  }

  /** The current season with its day and a bar of how far it has gone, then the three to come with the days until each; each with its growth. */
  private drawNext(season: number, left: number, per: number): void {
    const cells = [0, 1, 2, 3].map((k) => {
      const s = (season + k) % 4, days = k === 0 ? left : left + (k - 1) * per;
      const growth = this.game.growth_percent(s);
      const cell = h('div', k === 0 ? 'cal-season current' : 'cal-season');
      cell.style.setProperty('--season', SEASONS[s].colour);
      const grow = h('span', growth === 0 ? 'cal-grow stopped' : 'cal-grow');
      grow.append(icon('spring'), `×${growth / 100}`);
      if (k === 0) cell.style.setProperty('--done', `${((per - left) / per) * 100}%`);
      cell.append(seasonIcon(s), h('span', 'cal-days', k === 0 ? `${per - left + 1}/${per}` : `${days}d`), grow);
      const when = k === 0 ? `${left} more day${left === 1 ? '' : 's'}` : `in ${days} day${days === 1 ? '' : 's'}`;
      cell.title = `${SEASONS[s].name} ${when}: plants grow ${growth === 0 ? 'not at all' : `at ${growth}% speed`}`;
      return cell;
    });
    this.next.replaceChildren(...cells);
  }
}

function hours(share: number): string {
  return String(Math.round(share * 24 * 10) / 10);
}
