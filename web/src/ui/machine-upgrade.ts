// The "Upgrade to Mk N" button on a machine's panel and a storage box's screen: raises the building one Mk with kits from
// the inventory (`machine_upgrade`, `upgrade_machine`; the same action as a kit in hand). It is hidden for a building
// with no higher Mk, and greyed out while the tech is missing or kits are short. Read from the engine on every update.

import './machine-upgrade.css';
import type { Game } from '../wasm/engine.js';
import { button } from './dom';

export class MachineUpgrade {
  readonly el: HTMLButtonElement;
  private pos: [number, number, number] = [0, 0, 0];

  constructor(private readonly game: Game) {
    this.el = button('secondary-btn machine-upgrade hidden', '', () => game.upgrade_machine(...this.pos));
  }

  /** Redraws the button for the building at `pos`; hides it when there is nothing to upgrade to. */
  update(pos: [number, number, number]): void {
    this.pos = pos;
    const g = this.game;
    const [next, kit, kits, held, unlocked] = g.machine_upgrade(...pos);
    this.el.classList.toggle('hidden', next === undefined);
    if (next === undefined) return;
    const cost = `${kits} ${g.item_name(kit)}${kits === 1 ? '' : 's'}`;
    const text = `Upgrade to Mk${next} (${cost}, you have ${held})`;
    if (this.el.textContent !== text) this.el.textContent = text;
    this.el.disabled = !unlocked || held < kits;
    this.el.title = unlocked
      ? `Raises this building to Mk${next} for ${cost}; its contents and connections stay.`
      : `Research Mk${next} for this building first (the research screen).`;
  }
}
