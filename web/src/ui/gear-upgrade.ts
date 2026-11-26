// The in-place upgrade buttons under the equipment slots in the inventory screen: one each for the worn hauler pack,
// Tool Belt and work gloves (`gear_upgrade`, `upgrade_gear`), raising the worn piece one tier for what that step takes
// from the inventory, so a full pack never has to come off. Drawn like a building's (`machine-upgrade.ts`: arrow and Mk,
// cost icon, held/needed). A button hides with nothing worn there or at the last tier, and greys out while the tech is
// missing or the cost is short. Read from the engine each time the inventory redraws. Another ladder: its slot in SLOTS.

import type { Game } from '../wasm/engine.js';
import { h } from './dom';
import { UpgradeButton } from './machine-upgrade';

/** Equipment slots whose gear is a ladder (`equipment::next_tier`): back, tool belt, gloves. */
const SLOTS = [0, 3, 5];

export class GearUpgrades {
  readonly el = h('div', 'gear-upgrades');
  private readonly views: UpgradeButton[];

  constructor(
    private readonly game: Game,
    icon: (item: number) => HTMLCanvasElement,
  ) {
    this.views = SLOTS.map((slot) => new UpgradeButton(icon, () => game.upgrade_gear(slot), 'gear-upgrade'));
    this.el.append(...this.views.map((v) => v.el));
  }

  update(): void {
    const g = this.game;
    SLOTS.forEach((slot, i) => {
      const view = this.views[i];
      const [next, cost, n, held, unlocked] = g.gear_upgrade(slot);
      if (next === undefined) return view.hide();
      const name = g.item_name(next);
      view.show(
        name.match(/Mk\d$/)?.[0] ?? name,
        cost,
        n,
        held,
        unlocked === 1,
        unlocked
          ? `Raises what you wear to a ${name} for ${n} ${g.item_name(cost)} (you have ${held}).`
          : `Research the ${name} first (the research screen).`,
      );
    });
  }
}
