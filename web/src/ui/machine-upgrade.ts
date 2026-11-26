// The upgrade button, drawn rather than written: an up arrow and the next Mk, the kit's icon and "held/needed" (red while
// short; a padlock instead of the arrow while research is missing). The long explanation is the tooltip. `UpgradeButton`
// is the drawing, shared with the worn gear's buttons (`gear-upgrade.ts`); `MachineUpgrade` is the one on a machine's
// panel and a storage box's screen (`machine_upgrade`, `upgrade_machine`; the same action as a kit in hand). Hidden for a
// building with no higher Mk. Read from the engine on every update.

import './machine-upgrade.css';
import type { Game } from '../wasm/engine.js';
import { button, h } from './dom';

const ICON_PX = 64;
const ARROW =
  '<svg viewBox="0 0 16 16" aria-hidden="true"><path d="M8 13V3M3.5 7.5 8 3l4.5 4.5" fill="none" stroke="currentColor" ' +
  'stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/></svg>';
const LOCK =
  '<svg viewBox="0 0 16 16" aria-hidden="true"><rect x="3.5" y="7" width="9" height="6.5" rx="1.5" fill="currentColor"/>' +
  '<path d="M5.5 7V5a2.5 2.5 0 0 1 5 0v2" fill="none" stroke="currentColor" stroke-width="1.6"/></svg>';

export class UpgradeButton {
  readonly el: HTMLButtonElement;
  private drawn = '';

  constructor(
    private readonly icon: (item: number) => HTMLCanvasElement,
    onClick: () => void,
    className = '',
  ) {
    this.el = button(`secondary-btn machine-upgrade hidden ${className}`, '', onClick);
  }

  /** Hides the button. */
  hide(): void {
    this.el.classList.add('hidden');
  }

  /** Shows an upgrade to `mk` ("Mk4") costing `kits` of `kit`, of which `held` are in the inventory. */
  show(mk: string, kit: number, kits: number, held: number, unlocked: boolean, title: string): void {
    this.el.classList.remove('hidden');
    this.el.disabled = !unlocked || held < kits;
    if (this.el.title !== title) this.el.title = title;
    const key = `${mk}|${kit}|${kits}|${held}|${unlocked}`;
    if (key === this.drawn) return;
    this.drawn = key;
    const label = h('span', 'mu-to');
    label.innerHTML = unlocked ? ARROW : LOCK;
    label.append(mk);
    const icon = h('canvas', 'mu-kit');
    icon.width = icon.height = ICON_PX;
    icon.getContext('2d')!.drawImage(this.icon(kit), 0, 0);
    const cost = h('span', held < kits ? 'mu-cost short' : 'mu-cost');
    cost.append(icon, `${held}/${kits}`);
    this.el.replaceChildren(label, cost);
  }
}

export class MachineUpgrade {
  private readonly view: UpgradeButton;
  private pos: [number, number, number] = [0, 0, 0];

  constructor(
    private readonly game: Game,
    icon: (item: number) => HTMLCanvasElement,
  ) {
    this.view = new UpgradeButton(icon, () => game.upgrade_machine(...this.pos));
  }

  get el(): HTMLButtonElement {
    return this.view.el;
  }

  /** Redraws the button for the building at `pos`; hides it when there is nothing to upgrade to. */
  update(pos: [number, number, number]): void {
    this.pos = pos;
    const g = this.game;
    const [next, kit, kits, held, unlocked] = g.machine_upgrade(...pos);
    if (next === undefined) return this.view.hide();
    const cost = `${kits} ${g.item_name(kit)}${kits === 1 ? '' : 's'}`;
    this.view.show(
      `Mk${next}`,
      kit,
      kits,
      held,
      unlocked === 1,
      unlocked
        ? `Upgrade this building to Mk${next} for ${cost} (you have ${held}); its contents and connections stay.`
        : `Research Mk${next} for this building first (the research screen).`,
    );
  }
}
