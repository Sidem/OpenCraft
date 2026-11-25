// Codex screen (Shift+T, or from the Research header / Home menu): an unlocking 118-element periodic
// table (`codex-elements.ts`) showing every discovered element and its in-game chemical forms, plus
// the world's achievements tab. Everything is read from the engine (`codex_*`); no state is kept here. It also owns
// the unlock popups (`codex-toast.ts`), which open it on what they show.

import './machine.css';
import './codex.css';
import type { Game } from '../wasm/engine.js';
import { CATEGORY_LABELS, ELEMENT_BY_Z, FORM_LABELS, PERIODIC_ELEMENTS } from './codex-elements';
import { CodexToasts } from './codex-toast';
import { button, h } from './dom';

const ICON_PX = 64;

type TabId = 'table' | 'achievements';

export class CodexPanel {
  /** Called after the screen closes. `resume` is true when play should continue. */
  onClose: (resume: boolean) => void = () => {};

  private readonly backdrop = h('div', 'mp-backdrop hidden');
  private readonly dialog = h('div', 'mp cx');
  private readonly summary = h('span', 'cx-summary');
  private readonly tableBtn: HTMLButtonElement;
  private readonly achBtn: HTMLButtonElement;
  private readonly tablePane = h('div', '');
  private readonly grid = h('div', 'cx-grid');
  private readonly card = h('div', 'cx-card');
  private readonly achPane = h('div', 'cx-achs hidden');
  private readonly cells = new Map<number, HTMLButtonElement>();
  private selectedZ = 26; // Iron by default
  private drawnVersion = -1;

  private readonly toasts: CodexToasts;

  constructor(
    private readonly game: Game,
    private readonly icon: (item: number) => HTMLCanvasElement,
    chime: (kind: 'discover' | 'achieve') => void,
  ) {
    this.toasts = new CodexToasts(game, icon);
    this.toasts.onChime = chime;
    this.toasts.onOpen = (target) => this.openAt(target);
    this.dialog.setAttribute('role', 'dialog');
    this.dialog.setAttribute('aria-modal', 'true');
    this.dialog.tabIndex = -1;

    const head = h('div', 'mp-head');
    const close = button('close-btn', '×', () => this.close(true));
    close.setAttribute('aria-label', 'Close');
    head.append(h('h2', '', 'Codex'), h('span', 'mp-keys', 'Shift+T, E or click outside to return'), close);

    const bar = h('div', 'cx-bar');
    const tabs = h('div', 'cx-tabs');
    tabs.setAttribute('role', 'tablist');
    this.tableBtn = button('cx-tab active', 'Periodic Table', () => this.showTab('table'));
    this.achBtn = button('cx-tab', 'Achievements', () => this.showTab('achievements'));
    this.tableBtn.setAttribute('role', 'tab');
    this.achBtn.setAttribute('role', 'tab');
    tabs.append(this.tableBtn, this.achBtn);
    bar.append(tabs, this.summary);

    for (const el of PERIODIC_ELEMENTS) {
      const cell = button('cx-cell cx-cat', '', () => {
        this.selectedZ = el.z;
        this.highlightSelected();
        this.drawCard();
      });
      cell.style.gridColumn = String(el.col);
      cell.style.gridRow = String(el.row);
      cell.dataset.cat = el.cat;
      cell.title = `${el.z} ${el.name}`;
      cell.append(h('span', 'cx-z', String(el.z)), h('span', 'cx-sym', el.symbol), h('div', 'cx-dots'));
      this.cells.set(el.z, cell);
      this.grid.append(cell);
    }

    this.tablePane.append(this.grid, this.card);
    const body = h('div', 'mp-body');
    body.append(bar, this.tablePane, this.achPane);
    this.dialog.append(head, body);
    this.backdrop.append(this.dialog);

    this.backdrop.addEventListener('pointerdown', (e) => {
      if (e.target === this.backdrop) this.close(true);
    });
    window.addEventListener('keydown', (e) => {
      if (!this.isOpen) return;
      if (e.code === 'Escape') {
        e.preventDefault();
        this.close(false);
      } else if (e.code === 'KeyE' || (e.code === 'KeyT' && e.shiftKey)) {
        e.preventDefault();
        this.close(true);
      } else if (e.code === 'Digit1') {
        this.showTab('table');
      } else if (e.code === 'Digit2') {
        this.showTab('achievements');
      }
    });
    document.body.append(this.backdrop);
  }

  get isOpen(): boolean {
    return !this.backdrop.classList.contains('hidden');
  }

  open(): void {
    this.backdrop.classList.remove('hidden');
    this.drawnVersion = -1;
    this.refresh();
    this.dialog.focus({ preventScroll: true });
  }

  /** Opens on element `z` in the table, or on achievement `achievement`, glowing (an unlock popup was clicked). */
  openAt(target: { z: number } | { achievement: number }): void {
    if ('z' in target) this.selectedZ = target.z;
    this.open();
    this.showTab('z' in target ? 'table' : 'achievements');
    const el = 'z' in target ? this.cells.get(target.z) : this.achPane.children[target.achievement];
    el?.scrollIntoView({ block: 'nearest' });
    el?.classList.add('cx-flash');
    el?.addEventListener('animationend', () => el.classList.remove('cx-flash'), { once: true });
  }

  close(resume: boolean): void {
    if (!this.isOpen) return;
    this.backdrop.classList.add('hidden');
    this.onClose(resume);
  }

  /** Call every frame: the unlock popups, and the screen when open. */
  update(): void {
    this.toasts.update();
    if (!this.isOpen) return;
    const v = this.game.codex_obtained_count();
    if (v !== this.drawnVersion) this.refresh();
  }

  private showTab(tab: TabId): void {
    this.tableBtn.classList.toggle('active', tab === 'table');
    this.achBtn.classList.toggle('active', tab === 'achievements');
    this.tablePane.classList.toggle('hidden', tab !== 'table');
    this.achPane.classList.toggle('hidden', tab !== 'achievements');
  }

  private refresh(): void {
    this.drawnVersion = this.game.codex_obtained_count();
    const [disc, reach, unlocked, totalAch] = this.game.codex_summary();
    this.summary.textContent = `${disc} / ${reach} elements · ${unlocked} / ${totalAch} achievements`;

    const reachable = new Map<number, [number, number]>();
    const raw = this.game.codex_reachable_elements();
    for (let i = 0; i < raw.length; i += 3) {
      reachable.set(raw[i], [raw[i + 1], raw[i + 2]]);
    }

    for (const [z, cell] of this.cells) {
      const info = reachable.get(z);
      const done = info?.[0] ?? 0;
      const total = info?.[1] ?? 0;
      cell.classList.toggle('reachable', info !== undefined && done === 0);
      cell.classList.toggle('discovered', done > 0);
      cell.classList.toggle('mastered', total > 0 && done >= total);
      const dots = cell.querySelector('.cx-dots')!;
      dots.replaceChildren();
      if (total > 0) {
        const shown = Math.min(total, 6);
        const lit = Math.round((done / total) * shown);
        for (let i = 0; i < shown; i++) dots.append(h('span', i < lit ? 'cx-dot on' : 'cx-dot'));
      }
    }
    this.highlightSelected();
    this.drawCard();
    this.drawAchievements();
  }

  private highlightSelected(): void {
    for (const [z, cell] of this.cells) {
      cell.classList.toggle('selected', z === this.selectedZ);
    }
  }

  private drawCard(): void {
    const el = ELEMENT_BY_Z.get(this.selectedZ);
    if (!el) return;
    const left = h('div', '');
    const head = h('div', 'cx-card-head');
    const badge = h('div', 'cx-badge');
    badge.append(h('span', 'cx-badge-z', String(el.z)), h('span', 'cx-badge-sym', el.symbol));
    const titleBox = h('div', '');
    titleBox.append(h('h3', 'cx-card-name', el.name), h('div', 'cx-card-cat', CATEGORY_LABELS[el.cat]));
    head.append(badge, titleBox);
    left.append(head);

    const info = this.game.codex_element_info(el.z);
    if (info) {
      const [fact, role] = info.split('\n');
      if (fact) left.append(h('p', 'cx-card-fact', fact));
      if (role) left.append(h('p', 'cx-card-role', role));
    } else {
      left.append(h('p', 'cx-card-fact', 'Not in this world yet. Future chemistry updates may unlock this element.'));
    }

    const forms = h('div', 'cx-forms');
    const data = this.game.codex_element_forms(el.z);
    const notes = this.game.codex_element_form_notes(el.z).split('\n');
    for (let i = 0; i < data.length; i += 3) {
      const itemId = data[i];
      const formCode = data[i + 1];
      const made = data[i + 2] === 1;
      const note = notes[i / 3] ?? '';
      const row = h('div', made ? 'cx-form made' : 'cx-form unknown');
      row.append(this.iconCanvas(itemId, 'cx-form-icon'));
      const meta = h('div', 'cx-form-meta');
      const top = h('div', 'cx-form-top');
      top.append(
        h('span', '', this.game.item_name(itemId)),
        h('span', 'cx-form-tag', FORM_LABELS[formCode] ?? 'Form'),
      );
      meta.append(top, h('div', 'cx-form-note', made ? note : 'Undiscovered — obtain or produce to record'));
      row.append(meta);
      forms.append(row);
    }
    this.card.replaceChildren(left, forms);
  }

  private drawAchievements(): void {
    const count = this.game.codex_achievement_count();
    const nodes: HTMLElement[] = [];
    for (let i = 0; i < count; i++) {
      const [title, desc, iconStr, curStr, targetStr, doneStr] = this.game.codex_achievement(i).split('\n');
      const cur = Number(curStr) || 0;
      const target = Math.max(1, Number(targetStr) || 1);
      const done = doneStr === '1';
      const card = h('div', done ? 'cx-ach done' : 'cx-ach');
      card.append(this.iconCanvas(Number(iconStr) || 0, 'cx-form-icon'));
      const body = h('div', 'cx-ach-body');
      const bar = h('div', 'cx-ach-bar');
      const fill = h('div', 'cx-ach-fill');
      fill.style.width = `${Math.min(100, (cur / target) * 100)}%`;
      bar.append(fill);
      body.append(
        h('div', 'cx-ach-title', title),
        h('div', 'cx-ach-desc', desc),
        bar,
        h('div', 'cx-ach-stat', done ? 'Unlocked' : `${cur} / ${target}`),
      );
      card.append(body);
      nodes.push(card);
    }
    this.achPane.replaceChildren(...nodes);
  }

  private iconCanvas(item: number, className: string): HTMLCanvasElement {
    const c = h('canvas', className);
    c.width = c.height = ICON_PX;
    c.getContext('2d')!.drawImage(this.icon(item), 0, 0);
    return c;
  }
}
