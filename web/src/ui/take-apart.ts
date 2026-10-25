// Taking a crafted item apart from the build menu (crafting.ts): right-click a tile to take one batch apart (what one
// craft makes), Shift-right-click for up to `BULK`. The engine decides what comes back (`take_apart_parts`: most of
// the inputs, a little iron as scrap) and refuses tools and the few items that would open a loop. The info card gets a
// line saying what one batch gives back and how many the player holds.
// To change what the card says: `takeApartNote`.

import './take-apart.css';
import type { Game } from '../wasm/engine.js';
import { h } from './dom';

/** Shift-right-click takes up to this many batches apart at once. */
const BULK = 5;

/** Takes one batch of recipe `r` apart (up to `BULK` with Shift); whether anything was. */
export function takeApart(game: Game, r: number, shift: boolean): boolean {
  return game.take_apart(r, shift ? BULK : 1) > 0;
}

/** The card's line for recipe `r`: what a batch gives back, or null when it cannot be taken apart or none is held. */
export function takeApartNote(game: Game, r: number): HTMLElement | null {
  const times = game.take_apart_times(r);
  const flat = game.take_apart_parts(r);
  if (times === 0 || flat.length === 0) return null;
  const parts: string[] = [];
  for (let k = 0; k + 1 < flat.length; k += 2) parts.push(`${flat[k + 1]} ${game.item_name(flat[k])}`);
  const n = game.recipe_output_count(r);
  const what = n > 1 && game.tool_uses(game.recipe_output(r)) === 0 ? `${n} of them` : 'one';
  return h('p', 'craft-apart', `Right-click to take ${what} apart: you get back ${parts.join(', ')}.`);
}
