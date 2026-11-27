// Shift+F raises a flare where the player looks (GEAR-FLARE; the engine draws it: `crates/engine/src/flares.rs`).
// Alone it is lit at once; in co-op it goes through the session, so every player sees it in the raiser's colour.

import type { Game } from './wasm/engine';
import type { Coop } from './net/coop';

/** Raises a flare on the ground in view; does nothing when the view meets no ground in reach. */
export function raiseFlare(game: Game, coop: Coop | null): void {
  const spot = game.flare_spot();
  if (spot.length !== 3) return;
  const [x, y, z] = spot;
  if (coop) coop.flare(x, y, z);
  else game.light_flare(x, y, z, game.local_player());
}
