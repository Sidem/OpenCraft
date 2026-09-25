// Research screen (R): the tech tree as one card per tech, in table order, each showing its state,
// what it unlocks, what one unit costs and how far it got. Clicking Research on an available tech sets
// what every lab in the world works on. Also the HUD tracker (the tech being researched and its
// progress) and a short notice when a tech is done. Everything is read from the engine (`tech_*`,
// `current_research`); choosing queues `set_research`. A new tech in the engine needs no change here.
// The dialog frame reuses the machine panel's styles (machine.css), the item chips the build menu's
// (inventory.css).

import './inventory.css';
import './machine.css';
import './research.css';
import type { Game } from '../wasm/engine.js';
import { button, h } from './dom';

const ICON_PX = 64;
const NOTICE_MS = 6000;

interface TechView {
  root: HTMLDivElement;
  state: HTMLSpanElement;
  bar: HTMLDivElement;
  count: HTMLSpanElement;
  choose: HTMLButtonElement;
}

export class ResearchPanel {
  /** Called after the screen closes. `resume` is true when play should continue (R, E, or a click outside). */
  onClose: (resume: boolean) => void = () => {};
  /** Called when a tech is done. */
  onDone: () => void = () => {};

  private readonly backdrop = h('div', 'mp-backdrop hidden');
  private readonly dialog = h('div', 'mp rs');
  private readonly current = h('p', 'mp-status');
  private readonly tracker = h('div', 'rs-tracker hidden');
  private readonly trackerText = h('span', '');
  private readonly trackerBar = h('div', 'mp-bar-fill');
  private readonly views: TechView[];
  private done: boolean[];
  private noticeUntil = 0;
  private drawn = '';

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
    head.append(h('h2', '', 'Research'), h('span', 'mp-keys', 'R, E or click outside to return'), close);

    const count = game.tech_count();
    this.views = Array.from({ length: count }, (_, t) => this.makeTech(t));
    this.done = Array.from({ length: count }, (_, t) => game.tech_done(t));
    const intro = h('p', 'rs-intro',
      'Labs use science packs to research new machines. Choose a tech here; every powered lab with the right packs works on it.');
    const body = h('div', 'mp-body');
    body.append(intro, this.current, ...this.views.map((v) => v.root));
    this.dialog.append(head, body);
    this.backdrop.append(this.dialog);

    const bar = h('div', 'mp-bar');
    bar.append(this.trackerBar);
    this.tracker.append(this.trackerText, bar);
    document.getElementById('hud')!.append(this.tracker);

    this.backdrop.addEventListener('pointerdown', (e) => {
      if (e.target === this.backdrop) this.close(true);
    });
    this.backdrop.addEventListener('contextmenu', (e) => e.preventDefault());
    window.addEventListener('keydown', (e) => {
      if (!this.isOpen || e.repeat) return;
      if (e.code === 'KeyR' || e.code === 'KeyE') {
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
    this.backdrop.classList.remove('hidden');
    this.drawn = '';
    this.update(performance.now());
    this.dialog.focus();
  }

  close(resume: boolean): void {
    if (!this.isOpen) return;
    this.backdrop.classList.add('hidden');
    this.onClose(resume);
  }

  /** Call every frame: the HUD tracker, notices for finished techs, and the screen when open. */
  update(now: number): void {
    const g = this.game;
    const current = g.current_research();
    for (let t = 0; t < this.done.length; t++) {
      if (this.done[t] || !g.tech_done(t)) continue;
      this.done[t] = true;
      this.noticeUntil = now + NOTICE_MS;
      this.trackerText.textContent = `Research done: ${g.tech_name(t)}. ${this.unlockText(t)}`;
      this.trackerBar.style.transform = 'scaleX(1)';
      this.onDone();
    }
    const notice = now < this.noticeUntil;
    this.tracker.classList.toggle('hidden', !notice && (current < 0 || this.isOpen));
    this.tracker.classList.toggle('rs-notice', notice);
    if (!notice && current >= 0) {
      const [done, units] = [g.tech_progress(current), g.tech_units(current)];
      this.trackerText.textContent = `Researching ${g.tech_name(current)} · ${done} of ${units}`;
      this.trackerBar.style.transform = `scaleX(${done / units})`;
    }
    if (this.isOpen) this.draw(current);
  }

  /** Redraws the cards when the current tech or any progress changed. */
  private draw(current: number): void {
    const g = this.game;
    const progress = this.views.map((_, t) => g.tech_progress(t));
    const key = `${current}|${progress.join(',')}|${this.views.map((_, t) => g.tech_available(t)).join(',')}`;
    if (key === this.drawn) return;
    this.drawn = key;
    this.current.textContent = current >= 0
      ? `Labs are researching ${g.tech_name(current)}.`
      : 'Nothing is being researched. Choose an available tech below.';
    this.views.forEach((v, t) => {
      const [done, available, units] = [g.tech_done(t), g.tech_available(t), g.tech_units(t)];
      const chosen = t === current;
      v.root.className = `rs-tech ${done ? 'done' : available ? 'available' : 'locked'}${chosen ? ' chosen' : ''}`;
      v.state.textContent = done ? 'Done' : chosen ? 'Researching' : available ? 'Available' : this.needsText(t);
      v.bar.style.transform = `scaleX(${progress[t] / units})`;
      v.count.textContent = `${progress[t]} of ${units} units`;
      v.choose.classList.toggle('hidden', done || !available);
      v.choose.textContent = chosen ? 'Stop' : 'Research';
    });
  }

  private makeTech(t: number): TechView {
    const g = this.game;
    const root = h('div', 'rs-tech');
    const title = h('div', 'rs-title');
    const state = h('span', 'rs-state');
    title.append(h('h4', '', g.tech_name(t)), state);
    const unlocks = h('div', 'rs-chips');
    unlocks.append(h('span', 'rs-label', 'Unlocks'));
    for (const item of g.tech_unlocks(t)) unlocks.append(this.chip(item, g.item_name(item)));
    const cost = h('div', 'rs-chips');
    cost.append(h('span', 'rs-label', 'Each unit'));
    for (const item of g.tech_packs(t)) cost.append(this.chip(item, `1 ${g.item_name(item)}`));
    cost.append(h('span', 'rs-label', `${g.tech_seconds(t)} s in a lab`));
    const progress = h('div', 'rs-progress');
    const barBox = h('div', 'mp-bar');
    const bar = h('div', 'mp-bar-fill');
    barBox.append(bar);
    const count = h('span', 'rs-count');
    progress.append(barBox, count);
    const choose = button('secondary-btn rs-choose', 'Research', () => {
      g.set_research(g.current_research() === t ? -1 : t);
    });
    root.append(title, h('p', 'rs-blurb', g.tech_blurb(t)), unlocks, cost, progress, choose);
    return { root, state, bar, count, choose };
  }

  private needsText(t: number): string {
    const g = this.game;
    const missing = Array.from(g.tech_needs(t)).filter((n) => !g.tech_done(n));
    return `Needs ${missing.map((n) => g.tech_name(n)).join(' and ')}`;
  }

  private unlockText(t: number): string {
    const names = Array.from(this.game.tech_unlocks(t), (item) => this.game.item_name(item));
    return names.length > 0 ? `You can now build: ${names.join(', ')}.` : '';
  }

  private chip(item: number, text: string): HTMLSpanElement {
    const el = h('span', 'chip');
    const c = h('canvas', 'chip-icon');
    c.width = c.height = ICON_PX;
    c.getContext('2d')!.drawImage(this.icon(item), 0, 0);
    el.append(c, text);
    return el;
  }
}
