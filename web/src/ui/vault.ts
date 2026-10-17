// Vault door panel: opens when the player right-clicks a vault's door in a ruin (`game.vault_rings(x, y, z)` answers
// non-zero; main.ts tries it after the logbook). Six dials of digits 0 to 4, the first ring on the right so the dials read
// as one number. Under them the engine's own answer (`game.vault_probe`): the dials' number multiplied by itself, and the
// door's number, all fours; rings light from the right while their digits agree. The engine decides whether the door opens
// (`game.vault_open`); nothing about the puzzle is computed here. The frame is the machine panel's (machine.css).

import './machine.css';
import './vault.css';
import type { Game } from '../wasm/engine.js';
import { button, h } from './dom';

export class VaultPanel {
  /** Called after the screen closes. `resume` is true when play should continue. */
  onClose: (resume: boolean) => void = () => {};

  private readonly backdrop = h('div', 'mp-backdrop hidden');
  private readonly dialog = h('div', 'mp vault');
  private readonly body = h('div', 'mp-body');
  private readonly rings = h('div', 'vault-row');
  private readonly squares = h('div', 'vault-row vault-squares');
  private readonly door = h('div', 'vault-row vault-door');
  private readonly note = h('p', 'vault-note', '');
  private digits: number[] = [];
  private at: [number, number, number] = [0, 0, 0];

  constructor(private readonly game: Game) {
    this.dialog.setAttribute('role', 'dialog');
    this.dialog.setAttribute('aria-modal', 'true');
    this.dialog.tabIndex = -1;
    const head = h('div', 'mp-head');
    const close = button('close-btn', '×', () => this.close(true));
    close.setAttribute('aria-label', 'Close');
    head.append(h('h2', '', 'A door of rings'), h('span', 'mp-keys', 'E or click outside to return'), close);
    const open = button('secondary-btn', 'Try the door', () => this.tryDoor());
    this.body.append(
      h('p', 'vault-label', 'Rings'), this.rings,
      h('p', 'vault-label', 'The rings, times themselves'), this.squares,
      h('p', 'vault-label', 'The door'), this.door,
      this.note, open,
    );
    this.dialog.append(head, this.body);
    this.backdrop.append(this.dialog);
    this.backdrop.addEventListener('pointerdown', (e) => {
      if (e.target === this.backdrop) this.close(true);
    });
    this.backdrop.addEventListener('contextmenu', (e) => e.preventDefault());
    window.addEventListener('keydown', (e) => {
      if (!this.isOpen || e.repeat) return;
      if (e.code === 'KeyE') {
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

  /** Shows the door at a block; false (and nothing opens) when no vault door stands there. */
  open(x: number, y: number, z: number): boolean {
    const count = this.game.vault_rings(x, y, z);
    if (count === 0) return false;
    this.at = [x, y, z];
    this.digits = new Array<number>(count).fill(0);
    this.note.textContent = '';
    this.rings.replaceChildren(...this.digits.map((_, i) => this.dial(i)).reverse());
    this.refresh();
    this.backdrop.classList.remove('hidden');
    this.dialog.focus();
    return true;
  }

  close(resume: boolean): void {
    if (!this.isOpen) return;
    this.backdrop.classList.add('hidden');
    this.onClose(resume);
  }

  /** One ring: a digit between two arrows. */
  private dial(i: number): HTMLElement {
    const value = h('span', 'vault-digit', '0');
    const turn = (by: number) => {
      this.digits[i] = (this.digits[i] + by + 5) % 5;
      value.textContent = String(this.digits[i]);
      this.note.textContent = '';
      this.refresh();
    };
    const up = button('vault-turn', '▲', () => turn(1));
    const down = button('vault-turn', '▼', () => turn(-1));
    up.setAttribute('aria-label', `Ring ${i + 1} up`);
    down.setAttribute('aria-label', `Ring ${i + 1} down`);
    const dial = h('div', 'vault-dial');
    dial.append(up, value, down);
    return dial;
  }

  /** The product row and the door's row, lit from the right as far as they agree. */
  private refresh(): void {
    const [lit, ...square] = this.game.vault_probe(new Uint8Array(this.digits));
    const cell = (text: string, i: number) => h('span', `vault-cell${i < lit ? ' lit' : ''}`, text);
    this.squares.replaceChildren(...square.map((d, i) => cell(String(d), i)).reverse());
    this.door.replaceChildren(...square.map((_, i) => cell('4', i)).reverse());
  }

  private tryDoor(): void {
    const [lit] = this.game.vault_probe(new Uint8Array(this.digits));
    const [x, y, z] = this.at;
    this.game.vault_open(x, y, z, new Uint8Array(this.digits));
    if (lit < this.digits.length) {
      this.note.textContent = 'The door does not move.';
      return;
    }
    this.close(true);
  }
}
