// Analytics hub (P): how the factory is running, split into tabs that each answer one question:
//   - Overview: tiles, what needs attention, a small power chart and the busiest materials (analytics-overview.ts);
//   - Power: supply by source and demand by kind of machine, stacked over time (analytics-power.ts);
//   - Materials: what is made and what is used up, grouped and filterable (analytics-items.ts);
//   - Machines: efficiency by kind, what holds machines back, the slowest machines (analytics-machines.ts).
// This file is the shell: the dialog, the tab bar, the timescale (1m to 3h, shared by every chart) and the once-a-second
// refresh of the tab showing. All numbers come from the engine (`analytics_*`); history starts when the world is opened
// (it is not saved). Keys: P or E closes, Esc closes and keeps the mouse, 1-4 pick a tab. Dialog frame styles: machine.css.
// To add a tab: implement `Tab` (analytics-tab.ts) and add it to TABS.

import './machine.css';
import './analytics.css';
import type { Game } from '../wasm/engine.js';
import { OverviewTab } from './analytics-overview';
import { ItemsTab } from './analytics-items';
import { MachinesTab } from './analytics-machines';
import { PowerTab } from './analytics-power';
import type { Ctx, Tab } from './analytics-tab';
import { button, h } from './dom';

// `ticks`: how many time-axis divisions, so the labels fall on round times (15 s, 1 min, 5 min, 10 min, 30 min).
const SCALES = [
  { label: '1m', secs: 60, ticks: 4 },
  { label: '5m', secs: 300, ticks: 5 },
  { label: '30m', secs: 1800, ticks: 6 },
  { label: '1h', secs: 3600, ticks: 6 },
  { label: '3h', secs: 10800, ticks: 6 },
];
const DEFAULT_SCALE = 1;
const REFRESH_MS = 1000;

const TABS: { id: string; label: string; make: () => Tab }[] = [
  { id: 'overview', label: 'Overview', make: () => new OverviewTab() },
  { id: 'power', label: 'Power', make: () => new PowerTab() },
  { id: 'items', label: 'Materials', make: () => new ItemsTab() },
  { id: 'machines', label: 'Machines', make: () => new MachinesTab() },
];

export class AnalyticsPanel {
  /** Called after the screen closes. `resume` is true when play should continue (P, E, or a click outside). */
  onClose: (resume: boolean) => void = () => {};
  /** A tab's "Show on map": main.ts closes this screen and opens the world map on those machines. */
  onShowOnMap: (block: number, at?: { x: number; z: number }) => void = () => {};

  private readonly backdrop = h('div', 'mp-backdrop hidden');
  private readonly dialog = h('div', 'mp an');
  private readonly tabs = TABS.map((t) => ({ ...t, tab: t.make(), button: null as HTMLButtonElement | null }));
  private readonly scaleButtons: HTMLButtonElement[];
  private active = 0;
  private scale = DEFAULT_SCALE;
  private last = -Infinity;

  constructor(
    private readonly game: Game,
    private readonly icon: (item: number) => HTMLCanvasElement,
  ) {
    this.dialog.setAttribute('role', 'dialog');
    this.dialog.setAttribute('aria-modal', 'true');
    this.dialog.tabIndex = -1;
    const head = h('div', 'mp-head');
    const close = button('close-btn', '×', () => this.close(true));
    close.setAttribute('aria-label', 'Close');
    head.append(h('h2', '', 'Analytics'), h('span', 'mp-keys', 'P, E or click outside to return'), close);

    const bar = h('div', 'an-bar');
    const nav = h('div', 'an-tabs');
    nav.setAttribute('role', 'tablist');
    this.tabs.forEach((t, i) => {
      t.button = button('an-tab', t.label, () => this.show(i));
      t.button.setAttribute('role', 'tab');
      nav.append(t.button);
    });
    const scales = h('div', 'an-scales');
    scales.setAttribute('role', 'group');
    scales.setAttribute('aria-label', 'Timescale');
    this.scaleButtons = SCALES.map((s, i) =>
      button('an-scale', s.label, () => {
        this.scale = i;
        this.refresh();
      }),
    );
    scales.append(h('span', 'an-scales-label', 'Last'), ...this.scaleButtons);
    bar.append(nav, scales);

    const body = h('div', 'mp-body');
    body.append(bar, ...this.tabs.map((t) => t.tab.el));
    this.dialog.append(head, body);
    this.backdrop.append(this.dialog);

    this.backdrop.addEventListener('pointerdown', (e) => {
      if (e.target === this.backdrop) this.close(true);
    });
    this.backdrop.addEventListener('contextmenu', (e) => e.preventDefault());
    window.addEventListener('keydown', (e) => this.onKey(e));
    window.addEventListener('resize', () => {
      if (this.isOpen) this.refresh();
    });
    document.body.append(this.backdrop);
    this.arrange();
  }

  get isOpen(): boolean {
    return !this.backdrop.classList.contains('hidden');
  }

  open(): void {
    this.backdrop.classList.remove('hidden');
    this.tabs[this.active].tab.open?.();
    this.refresh();
    this.dialog.focus();
  }

  close(resume: boolean): void {
    if (!this.isOpen) return;
    this.backdrop.classList.add('hidden');
    this.onClose(resume);
  }

  /** Call every frame: refreshes the tab showing once a second while open. */
  update(now: number): void {
    if (this.isOpen && now - this.last >= REFRESH_MS) this.refresh(now);
  }

  private onKey(e: KeyboardEvent): void {
    if (!this.isOpen || e.repeat) return;
    // Typing in a search box must not close the screen or switch tab.
    if ((e.target as HTMLElement | null)?.tagName === 'INPUT') {
      if (e.key === 'Escape') (e.target as HTMLElement).blur();
      return;
    }
    const digit = /^Digit([1-9])$/.exec(e.code);
    if (e.code === 'KeyP' || e.code === 'KeyE') {
      e.preventDefault();
      this.close(true);
    } else if (e.key === 'Escape') {
      e.preventDefault();
      this.close(false);
    } else if (digit && Number(digit[1]) <= this.tabs.length) {
      e.preventDefault();
      this.show(Number(digit[1]) - 1);
    }
  }

  private show(i: number): void {
    if (i !== this.active) {
      this.active = i;
      this.arrange();
      this.tabs[i].tab.open?.();
    }
    this.refresh();
  }

  /** Shows the active tab's panel and marks its button. */
  private arrange(): void {
    this.tabs.forEach((t, i) => {
      t.tab.el.classList.toggle('hidden', i !== this.active);
      t.button?.setAttribute('aria-selected', String(i === this.active));
    });
  }

  private refresh(now = performance.now()): void {
    this.last = now;
    this.scaleButtons.forEach((b, i) => b.setAttribute('aria-pressed', String(i === this.scale)));
    const { secs, ticks } = SCALES[this.scale];
    const go = (id: string) => this.show(Math.max(0, TABS.findIndex((t) => t.id === id)));
    const ctx: Ctx = { game: this.game, icon: this.icon, secs, ticks, go, showOnMap: (block, at) => this.onShowOnMap(block, at) };
    this.tabs[this.active].tab.refresh(ctx);
  }
}
