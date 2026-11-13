// One inventory-screen slot as drawn (inventory.ts): its icon, its count or wear bar, and its hover text (name, uses
// left, what worn gear does). Redraws the icon only when the item or count changed. To show more on a slot: here.

import type { Game } from '../wasm/engine.js';
import { showAmount } from './hud';

export const ICON_PX = 64;

export interface SlotView {
  root: HTMLDivElement;
  icon: HTMLCanvasElement;
  count: HTMLSpanElement;
  item: number;
  n: number;
}

/** Shows `n` of `item` (nothing when `n` is 0) in slot `s`; `data-item` tells the controls bar what it holds. */
export function drawSlot(game: Game, icon: (item: number) => HTMLCanvasElement, s: SlotView, item: number, n: number): void {
  const uses = game.tool_uses(item);
  const amount = uses > 0 ? ` (${n} of ${uses} uses left)` : n > 1 ? ` ×${n}` : '';
  const effect = n > 0 ? game.gear_blurb(item) : '';
  s.root.title = n > 0 ? `${game.item_name(item)}${amount}${effect ? `: ${effect}` : ''}` : '';
  s.root.dataset.item = n > 0 ? String(item) : '';
  if (item === s.item && n === s.n) return;
  const ctx = s.icon.getContext('2d')!;
  ctx.clearRect(0, 0, ICON_PX, ICON_PX);
  if (n > 0) ctx.drawImage(icon(item), 0, 0);
  showAmount(game, s.root, s.count, item, n);
  s.item = item;
  s.n = n;
}
