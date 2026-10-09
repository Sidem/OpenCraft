// The hauler pack's upgrade button, beside the equipment slots in the inventory screen: raises the worn pack one tier
// with kits from the inventory (`pack_upgrade`, `upgrade_pack`), so a full pack never has to come off. It is hidden
// with no pack worn or at the last tier, and greyed out while the tech is missing or kits are short. Everything is
// read from the engine each time the inventory redraws.

import type { Game } from '../wasm/engine.js';
import { h } from './dom';

export class PackUpgrade {
  readonly el = h('button', 'secondary-btn pack-upgrade hidden') as HTMLButtonElement;

  constructor(private readonly game: Game) {
    this.el.type = 'button';
    this.el.addEventListener('click', () => game.upgrade_pack());
  }

  update(): void {
    const g = this.game;
    const [next, kit, kits, held, unlocked] = g.pack_upgrade();
    this.el.classList.toggle('hidden', next === undefined);
    if (next === undefined) return;
    const cost = `${kits} ${g.item_name(kit)}${kits === 1 ? '' : 's'}`;
    this.el.textContent = `Upgrade pack to ${g.item_name(next).replace('Hauler Pack ', '')} (${cost}, you have ${held})`;
    this.el.disabled = !unlocked || held < kits;
    this.el.title = unlocked
      ? `Raises the pack you wear to a ${g.item_name(next)} for ${cost}; nothing in your backpack moves.`
      : `Research the ${g.item_name(next)} first (the research screen).`;
  }
}
