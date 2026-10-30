// Blueprint library (L): the player's copied machine layouts. Each blueprint is a card: a picture of it from
// above (blueprint-plan.ts), its name (click to rename), what it holds as item chips (red where you carry too
// few to build it), a Hold button (takes it in hand: ghost mode on, the use button stamps its ghosts) and
// Delete. Copying is done in the world (ghost mode, B: Z marks a box, Enter copies it); everything is read from
// the engine (`blueprint_*`), and the engine's bytes are kept with the world's record by save/session.ts. The
// dialog frame reuses the machine panel's styles (machine.css).

import './machine.css';
import './blueprints.css';
import type { Game } from '../wasm/engine.js';
import { drawPlan } from './blueprint-plan';
import { button, h } from './dom';

/** The picture's size in CSS pixels: wider than tall, as most layouts are lines. */
const PLAN_PX: [number, number] = [144, 96];
/** Item chips shown on a card before "+N more". */
const MAX_CHIPS = 6;
const CHIP_ICON_PX = 20;

export class BlueprintPanel {
  /** Called after the screen closes. `resume` is true when play should continue. */
  onClose: (resume: boolean) => void = () => {};

  private readonly backdrop = h('div', 'mp-backdrop hidden');
  private readonly dialog = h('div', 'mp bp');
  private readonly list = h('div', 'bp-list');

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
    head.append(h('h2', '', 'Blueprints'), h('span', 'mp-keys', 'L, E or click outside to return'), close);
    const help = h(
      'p',
      'bp-help',
      'Copy a layout: press B for ghost mode, aim at one corner and press Z, aim at the opposite corner and press Z, ' +
        'then Enter. Machines, belts and poles are copied, not the ground and not recipes or filters. Hold a ' +
        'blueprint, then right-click to stamp it as ghosts (R turns it); build over the ghosts to make them real.',
    );
    const body = h('div', 'mp-body');
    body.append(help, this.list);
    this.dialog.append(head, body);
    this.backdrop.append(this.dialog);
    this.backdrop.addEventListener('pointerdown', (e) => {
      if (e.target === this.backdrop) this.close(true);
    });
    this.backdrop.addEventListener('contextmenu', (e) => e.preventDefault());
    window.addEventListener('keydown', (e) => {
      if (!this.isOpen || e.repeat) return;
      const typing = e.target instanceof HTMLInputElement;
      if (!typing && (e.code === 'KeyL' || e.code === 'KeyE')) {
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
    this.render();
    this.dialog.focus();
  }

  close(resume: boolean): void {
    if (!this.isOpen) return;
    this.backdrop.classList.add('hidden');
    this.onClose(resume);
  }

  private render(): void {
    const rows = this.game.blueprint_list();
    const held = this.game.blueprint_held();
    this.list.replaceChildren();
    if (!rows) {
      this.list.append(h('p', 'bp-empty', 'No blueprints yet.'));
      return;
    }
    const fields = this.game.blueprint_plan_fields();
    rows.split('\n').forEach((row, i) => {
      const [name, count] = row.split('\t');
      const input = h('input', 'bp-name');
      input.value = name;
      input.maxLength = 32;
      input.setAttribute('aria-label', 'Blueprint name');
      input.addEventListener('change', () => this.game.blueprint_rename(i, input.value));
      const hold = button('secondary-btn', i === held ? 'In hand' : 'Hold', () => {
        this.game.blueprint_hold(i);
        this.close(true);
      });
      const del = button('secondary-btn', 'Delete', () => {
        this.game.blueprint_delete(i);
        this.render();
      });
      const plan = h('canvas', 'bp-plan');
      plan.setAttribute('role', 'img');
      plan.setAttribute('aria-label', `${name} seen from above`);
      drawPlan(plan, this.game.blueprint_plan(i), fields, PLAN_PX, this.icon);
      const head = h('div', 'bp-head');
      head.append(input, h('span', 'bp-count', `${count} ${count === '1' ? 'piece' : 'pieces'}`));
      const info = h('div', 'bp-info');
      info.append(head, ...this.contents(i));
      const actions = h('div', 'bp-actions');
      actions.append(hold, del);
      const card = h('div', 'bp-row');
      card.append(plan, info, actions);
      this.list.append(card);
    });
  }

  /** Blueprint `i`'s items as chips, then what you are short of to build it all. */
  private contents(i: number): HTMLElement[] {
    const needs = this.game.blueprint_needs(i);
    const chips = h('div', 'bp-items');
    const short: string[] = [];
    for (let k = 0; k < needs.length; k += 3) {
      const [item, count, carried] = [needs[k], needs[k + 1], needs[k + 2]];
      const name = this.game.item_name(item);
      if (carried < count) short.push(`${count - carried} ${name}`);
      if (k / 3 >= MAX_CHIPS) continue;
      const icon = h('canvas', 'bp-icon');
      icon.width = icon.height = CHIP_ICON_PX;
      icon.getContext('2d')!.drawImage(this.icon(item), 0, 0, CHIP_ICON_PX, CHIP_ICON_PX);
      const chip = h('span', carried < count ? 'bp-item bp-short' : 'bp-item');
      chip.title = carried < count ? `${name}: you carry ${carried} of ${count}` : `${name}: you carry enough`;
      chip.append(icon, `${count} ${name}`);
      chips.append(chip);
    }
    const kinds = needs.length / 3;
    if (kinds > MAX_CHIPS) chips.append(h('span', 'bp-more', `+${kinds - MAX_CHIPS} more`));
    const status = short.length ? `Missing ${short.join(', ')}` : 'You carry everything it needs';
    return [chips, h('p', short.length ? 'bp-status bp-missing' : 'bp-status', status)];
  }
}
