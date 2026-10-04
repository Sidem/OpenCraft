// Blueprint library (L): the player's copied machine layouts. Each row names a blueprint (click the name to
// rename it) with how many machines it holds, a Hold button (takes it in hand: ghost mode on, the use button
// stamps its ghosts) and Delete. Copying is done in the world (ghost mode, B: Z marks a box, Enter copies it);
// everything is read from the engine (`blueprint_*`), and the engine's bytes are kept with the world's
// record by save/session.ts. The dialog frame reuses the machine panel's styles (machine.css).

import './machine.css';
import './blueprints.css';
import type { Game } from '../wasm/engine.js';
import { button, h } from './dom';

export class BlueprintPanel {
  /** Called after the screen closes. `resume` is true when play should continue. */
  onClose: (resume: boolean) => void = () => {};

  private readonly backdrop = h('div', 'mp-backdrop hidden');
  private readonly dialog = h('div', 'mp bp');
  private readonly list = h('div', 'bp-list');

  constructor(private readonly game: Game) {
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
        'then Enter. Only machines are copied. Hold a blueprint, then right-click to stamp it as ghosts (R turns it); ' +
        'build over the ghosts to make them real.',
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
      const line = h('div', 'bp-row');
      line.append(input, h('span', 'bp-count', `${count} machines`), hold, del);
      this.list.append(line);
    });
  }
}
