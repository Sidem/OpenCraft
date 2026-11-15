// The hauler pack's upgrade button, beside the equipment slots in the inventory screen: raises the worn pack one tier
// with kits from the inventory (`pack_upgrade`, `upgrade_pack`), so a full pack never has to come off. Drawn like a
// building's (`machine-upgrade.ts`: arrow and Mk, kit icon, held/needed). It is hidden with no pack worn or at the last
// tier, and greyed out while the tech is missing or kits are short. Read from the engine each time the inventory redraws.

import type { Game } from '../wasm/engine.js';
import { UpgradeButton } from './machine-upgrade';

export class PackUpgrade {
  private readonly view: UpgradeButton;

  constructor(
    private readonly game: Game,
    icon: (item: number) => HTMLCanvasElement,
  ) {
    this.view = new UpgradeButton(icon, () => game.upgrade_pack(), 'pack-upgrade');
  }

  get el(): HTMLButtonElement {
    return this.view.el;
  }

  update(): void {
    const g = this.game;
    const [next, kit, kits, held, unlocked] = g.pack_upgrade();
    if (next === undefined) return this.view.hide();
    const cost = `${kits} ${g.item_name(kit)}${kits === 1 ? '' : 's'}`;
    this.view.show(
      g.item_name(next).replace('Hauler Pack ', ''),
      kit,
      kits,
      held,
      unlocked === 1,
      unlocked
        ? `Raises the pack you wear to a ${g.item_name(next)} for ${cost} (you have ${held}); nothing in your backpack moves.`
        : `Research the ${g.item_name(next)} first (the research screen).`,
    );
  }
}
