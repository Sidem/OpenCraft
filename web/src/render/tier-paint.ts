// Recolouring the baked tier layers for the colour-blind palette (engine `tier_palette.rs`). The stripes, the studs' paint
// and the kit cases are texture layers (some hand-painted by tools/art), so a palette is applied to their pixels: every
// texel that carries the tier colour takes the new palette's hue, with its saturation and brightness scaled by how the new
// colour differs from the old, while grey metal and white pips stay as they are. Pure functions on RGBA arrays; `TierPaint`
// keeps the standard layers so any palette can be applied from them. Presentation only. Changing a texel's look: `recolour`.

/** What a tier layer is, as the engine numbers it (`game.tier_layers()`). */
export const STRIPE = 0, STUD_PAINT = 1, KIT_CASE = 2;

export interface TierLayer {
  layer: number;
  tier: number;
  kind: number;
}

/** The engine's flat (layer, tier, kind) triples as objects. */
export function parseTierLayers(flat: ArrayLike<number>): TierLayer[] {
  const out: TierLayer[] = [];
  for (let i = 0; i + 2 < flat.length; i += 3) out.push({ layer: flat[i], tier: flat[i + 1], kind: flat[i + 2] });
  return out;
}

/** Texels less saturated than this are metal or highlights, not the tier colour (a stripe's steel is about 0.10). */
const TINT_MIN = 0.2;
/** A stud's paint is a pale tint of the tier colour (`textures/stripes.rs` `paint`): 30% colour, 70% white. */
const PALE = 0.3;

type Rgb = [number, number, number];
const split = (c: number): Rgb => [(c >> 16) & 255, (c >> 8) & 255, c & 255];

function toHsv([r, g, b]: Rgb): Rgb {
  const v = Math.max(r, g, b), m = Math.min(r, g, b), d = v - m;
  let h = 0;
  if (d > 0) {
    if (v === r) h = ((g - b) / d + 6) % 6;
    else if (v === g) h = (b - r) / d + 2;
    else h = (r - g) / d + 4;
  }
  return [h, v > 0 ? d / v : 0, v / 255];
}

function fromHsv(h: number, s: number, v: number): Rgb {
  const f = (n: number) => {
    const k = (n + h) % 6;
    return Math.round(255 * v * (1 - s * Math.max(0, Math.min(k, 4 - k, 1))));
  };
  return [f(5), f(3), f(1)];
}

/** The stud paint of tier colour `c`: the engine's pale tint (`textures/stripes.rs`). */
const pale = (c: number): Rgb => split(c).map((v) => v * PALE + 255 * (1 - PALE)) as Rgb;

/**
 * A copy of the layer `src` (RGBA, a stripe, a stud's paint or a kit's case) with tier colour `from` (0xRRGGBB) turned
 * into `to`. Alpha is kept. Colours are only moved for texels that carry the tier colour, so white pips and steel stay.
 */
export function recolour(src: Uint8Array, kind: number, from: number, to: number): Uint8Array {
  const out = new Uint8Array(src);
  if (from === to) return out;
  if (kind === STUD_PAINT) {
    // The whole layer is the tint, with a little noise: scale the new tint by each texel's brightness against the old.
    const [was, now] = [pale(from), pale(to)];
    for (let i = 0; i < out.length; i += 4) {
      const k = Math.max(src[i], src[i + 1], src[i + 2]) / Math.max(...was);
      for (let c = 0; c < 3; c++) out[i + c] = Math.min(255, Math.round(now[c] * k));
    }
    return out;
  }
  const [, s0, v0] = toHsv(split(from));
  const [h1, s1, v1] = toHsv(split(to));
  const sRatio = s1 / Math.max(s0, 0.01), vRatio = v1 / Math.max(v0, 0.01);
  for (let i = 0; i < out.length; i += 4) {
    const [, s, v] = toHsv([src[i], src[i + 1], src[i + 2]]);
    if (s < TINT_MIN) continue;
    const [r, g, b] = fromHsv(h1, Math.min(1, s * sRatio), Math.min(1, v * vRatio));
    out[i] = r;
    out[i + 1] = g;
    out[i + 2] = b;
  }
  return out;
}

/** The standard look of every tier layer, kept so a palette is always applied from it, never on top of another. */
export class TierPaint {
  private readonly base = new Map<number, Uint8Array>();
  private readonly layers: TierLayer[];

  /** `pixels` is the whole texture stack (`size` × `size` RGBA a layer); `layers` are the engine's tier layers. */
  constructor(pixels: Uint8Array, size: number, layers: TierLayer[]) {
    this.layers = layers; // no parameter properties: node's test runner strips types only
    const bytes = size * size * 4;
    for (const { layer } of layers) this.base.set(layer, pixels.slice(layer * bytes, (layer + 1) * bytes));
  }

  /** Every tier layer painted in palette `to` (five colours, Mk1 first), from the standard palette `from`: layer → pixels. */
  paint(from: number[], to: number[]): Map<number, Uint8Array> {
    const out = new Map<number, Uint8Array>();
    for (const { layer, tier, kind } of this.layers) out.set(layer, recolour(this.base.get(layer)!, kind, from[tier], to[tier]));
    return out;
  }
}
