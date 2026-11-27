// The icon check (GEAR-ICONS): renders every tiered item's icon at 32 px and fails a tier pair (Mk n and Mk n+1 of one
// machine family or gear ladder, `tier_ladders`) whose outlines match while nothing on the icon tells them apart.
// Run it in the game page: `await opencraft.iconCheck()` (main.ts imports this file only then); it prints a table and
// returns the failures. `compareIcons` is pure (tested in web/test/icon-check.test.mjs).
//
// The measures, both on the icons composited over the slot's colour:
// - overlap: the silhouettes' intersection over their union (1 is the same outline);
// - mean: the mean per-pixel colour difference over the whole icon, 0-255 (how the gear review measured "3-18");
// - patch: the same mean inside the most different 8x8 window (a quarter of the icon's side). The tier chip is a
//   local cue, a corner a player looks at, so the check reads the patch; the whole-icon mean cannot pass 40 for any
//   mark smaller than about half the icon, even one that differs completely.

import type { Game } from '../wasm/engine.js';

export const SIZE = 32;
/** A pair fails when its outlines overlap at least this much and its most different patch differs by less than PATCH_MIN. */
export const SAME_OUTLINE = 0.9;
export const PATCH_MIN = 40;
const PATCH = 8;
const ALPHA_MIN = 40;
const SLOT = [0x23, 0x2b, 0x38];

export interface Comparison {
  overlap: number;
  mean: number;
  patch: number;
}

/** Compares two `size`×`size` RGBA images. */
export function compareIcons(a: ArrayLike<number>, b: ArrayLike<number>, size: number): Comparison {
  let both = 0, either = 0;
  const diff = new Float32Array(size * size);
  for (let p = 0; p < size * size; p++) {
    const [inA, inB] = [a[p * 4 + 3] >= ALPHA_MIN, b[p * 4 + 3] >= ALPHA_MIN];
    if (inA && inB) both++;
    if (inA || inB) either++;
    let d = 0;
    for (let c = 0; c < 3; c++) d += Math.abs(over(a, p, c) - over(b, p, c));
    diff[p] = d / 3;
  }
  let mean = 0, patch = 0;
  for (const d of diff) mean += d;
  for (let y = 0; y + PATCH <= size; y += PATCH / 2) {
    for (let x = 0; x + PATCH <= size; x += PATCH / 2) {
      let s = 0;
      for (let j = 0; j < PATCH; j++) for (let i = 0; i < PATCH; i++) s += diff[(y + j) * size + x + i];
      patch = Math.max(patch, s / (PATCH * PATCH));
    }
  }
  return { overlap: either ? both / either : 1, mean: mean / (size * size), patch };
}

/** Channel `c` of pixel `p` over the slot's colour. */
function over(img: ArrayLike<number>, p: number, c: number): number {
  const alpha = img[p * 4 + 3] / 255;
  return img[p * 4 + c] * alpha + SLOT[c] * (1 - alpha);
}

export interface PairRow extends Comparison {
  pair: string;
  fails: boolean;
}

/** Renders and compares every tier pair; prints the table and returns every row (failures marked). */
export function runIconCheck(game: Game, icon: (item: number) => HTMLCanvasElement): PairRow[] {
  const ladders = game.tier_ladders(), rows: PairRow[] = [];
  for (let i = 0; i < ladders.length; i += ladders[i] + 1) {
    const items = Array.from(ladders.subarray(i + 1, i + 1 + ladders[i]));
    for (let t = 0; t + 1 < items.length; t++) {
      const c = compareIcons(pixels(icon(items[t])), pixels(icon(items[t + 1])), SIZE);
      const fails = c.overlap >= SAME_OUTLINE && c.patch < PATCH_MIN;
      const pair = `${game.item_name(items[t])} → ${game.item_name(items[t + 1])}`;
      rows.push({ pair, fails, overlap: round(c.overlap, 2), mean: round(c.mean, 0), patch: round(c.patch, 0) });
    }
  }
  console.table(rows);
  return rows;
}

function pixels(src: HTMLCanvasElement): Uint8ClampedArray {
  const c = document.createElement('canvas');
  c.width = c.height = SIZE;
  const ctx = c.getContext('2d')!;
  ctx.imageSmoothingQuality = 'high';
  ctx.drawImage(src, 0, 0, SIZE, SIZE);
  return ctx.getImageData(0, 0, SIZE, SIZE).data;
}

function round(v: number, digits: number): number {
  return Math.round(v * 10 ** digits) / 10 ** digits;
}
