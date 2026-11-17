// Connects the tier-colour setting (ui/tier-palette.ts) to what draws tier colours: the engine (outlines, item chips), the
// GPU texture array (stripes, studs, kit cases: `Renderer.paintLayer`) and the HUD's own copies of the layers (item icons).
// Called once from main.ts; the choice is a local setting, so nothing here reaches a save or the state hash.

import type { Game } from '../wasm/engine.js';
import { Hud } from '../ui/hud';
import { PALETTE_NAMES, TierPalettePanel, TierPaletteStore } from '../ui/tier-palette';
import type { Renderer } from './renderer';
import { parseTierLayers, TierPaint } from './tier-paint';

const STANDARD = 0;

/** Sets up the palette setting: its menu row goes into `host`, and the saved choice is applied now. */
export function wireTierPalette(game: Game, renderer: Renderer, hud: Hud, pixels: Uint8Array, size: number, host: HTMLElement): void {
  const store = new TierPaletteStore();
  const colours = PALETTE_NAMES.map((_, i) => Array.from(game.tier_palette_colours(i)));
  host.append(new TierPalettePanel(store, colours).el);
  const paint = new TierPaint(pixels, size, parseTierLayers(game.tier_layers()));
  let shown = STANDARD;
  store.subscribe(() => {
    game.set_tier_palette(store.palette);
    if (store.palette === shown) return;
    shown = store.palette;
    const layers = paint.paint(colours[STANDARD], colours[shown]);
    for (const [layer, data] of layers) renderer.paintLayer(layer, data);
    renderer.refreshMipmaps();
    hud.repaintLayers(layers);
  });
}
