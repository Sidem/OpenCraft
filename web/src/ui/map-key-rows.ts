// What each engine map mark means, one row per mark the map key shows (map-key.ts draws them with `drawMark`). Pure
// data, tested in web/test/map-key.test.mjs: every shape constant of the engine's marks.rs must fall in a row, and
// pins.ts must hold the same numbers. To explain a new mark: a row here.

import {
  MARK_CHANNEL, MARK_COVER, MARK_DEPOSIT, MARK_DRY, MARK_DRY_MINER, MARK_GUESS, MARK_LAUNCH, MARK_MACHINE, MARK_ORE,
  MARK_RANGE, MARK_SITE, MARK_TOWN,
} from './pins.ts';

/** How a row's sample is coloured: in an ore's colour, a machine's, or the number the shape reads (a reach). */
export type KeyTint = 'ore' | 'machine' | 'reach';

export interface KeyRow {
  /** The first shape the row stands for, and how many from there (a guess's three strengths, a satellite's kinds). */
  shape: number;
  span: number;
  /** The shape drawn as the row's sample, when not `shape`. */
  sample?: number;
  tint: KeyTint;
  label: string;
}

export const KEY_ROWS: KeyRow[] = [
  { shape: MARK_DEPOSIT, span: 1, tint: 'ore', label: 'Deposit: scanned, seen from orbit or under your miner' },
  { shape: MARK_DRY, span: 1, tint: 'ore', label: 'Worked out: no ore left' },
  { shape: MARK_ORE, span: 1, tint: 'ore', label: 'Ore seen at the surface' },
  { shape: MARK_GUESS, span: 3, sample: MARK_GUESS + 1, tint: 'ore', label: 'Possible ore: a survey guess' },
  { shape: MARK_MACHINE, span: 1, tint: 'machine', label: 'Your machine' },
  { shape: MARK_DRY_MINER, span: 1, tint: 'machine', label: 'Miner stopped: its deposit is worked out' },
  { shape: MARK_SITE, span: 1, tint: 'machine', label: 'Terraforming site' },
  { shape: MARK_COVER, span: 4, tint: 'reach', label: 'Satellite cover' },
  { shape: MARK_RANGE, span: 1, tint: 'reach', label: 'Long-distance pole reach (while you hold one)' },
  { shape: MARK_CHANNEL, span: 1, tint: 'machine', label: 'Landing pad' },
  { shape: MARK_CHANNEL + 1, span: 1, tint: 'machine', label: 'Pod launcher' },
  { shape: MARK_LAUNCH, span: 1, tint: 'machine', label: 'Launch site' },
  { shape: MARK_TOWN, span: 1, tint: 'machine', label: 'Town' },
  { shape: MARK_TOWN + 1, span: 1, tint: 'machine', label: 'Town short of upkeep' },
  { shape: MARK_TOWN + 2, span: 1, tint: 'machine', label: 'Dormant town (its depot is broken)' },
];

/** Whether some row stands for `shape`. */
export function keyCovers(shape: number): boolean {
  return KEY_ROWS.some((r) => shape >= r.shape && shape < r.shape + r.span);
}
