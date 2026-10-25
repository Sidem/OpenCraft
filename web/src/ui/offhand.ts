// The left-hand slot beside the hotbar (J swaps it with the selected slot; engine `offhand.rs`). It shows what the left
// hand holds, redrawn by the HUD whenever the inventory changes. The same stack is a slot in the inventory screen
// (`inventory.ts`), where a click puts the cursor stack in or takes it out.

import './offhand.css';
import type { Game } from '../wasm/engine.js';
import { h } from './dom';
import { showAmount } from './hud';

const ICON_PX = 64;

export class OffhandSlot {
  readonly root = h('div', 'slot offhand');
  private readonly icon = h('canvas');
  private readonly count = h('span', 'count');
  private item = -1;
  private n = -1;

  constructor(private readonly game: Game, private readonly drawIcon: (item: number) => HTMLCanvasElement) {
    this.icon.width = this.icon.height = ICON_PX;
    this.root.append(h('span', 'key', 'J'), this.icon, this.count);
    this.root.title = 'Left hand: J swaps it with the selected slot';
  }

  /** Redraws the slot if its stack changed. */
  update(): void {
    const item = this.game.offhand_item(), n = this.game.offhand_count();
    if (item === this.item && n === this.n) return;
    const ctx = this.icon.getContext('2d')!;
    ctx.clearRect(0, 0, ICON_PX, ICON_PX);
    if (n > 0) ctx.drawImage(this.drawIcon(item), 0, 0);
    showAmount(this.game, this.root, this.count, item, n);
    this.root.title = n > 0 ? `${this.game.item_name(item)} in the left hand (J swaps)` : 'Left hand: J swaps it with the selected slot';
    this.item = item;
    this.n = n;
  }
}
