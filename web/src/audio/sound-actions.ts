// The built-in shaping of every sound effect (data for sound.ts): its mix level, how it varies, how many variants are
// synthesised, whether it uses the material's sound and how far it carries; the engine's event kinds in order; and how
// the designer's previews are placed. To add an effect: a row in `ACTION_BASE` (and in `ACTIONS` in settings.ts), plus
// `EVENT_ACTIONS` if the engine raises it.

import type { ActionName } from './settings';

/** Engine event kinds, in order (crates/engine/src/sound.rs). */
export const EVENT_ACTIONS: readonly ActionName[] = ['dig', 'break', 'place', 'step', 'land', 'pickup', 'drop', 'splash', 'powerdown', 'powerup', 'launch', 'podup', 'poddown', 'denied', 'flame'];

export interface ActionBase {
  /** Mix level before the Volume dials. */
  gain: number;
  /** Random playback-rate spread per play, scaled by the material's Variation. */
  jitter: number;
  variants: number;
  /** Uses the material's sound (as opposed to one shared sound). */
  material: boolean;
  pitch: number;
  length: number;
  weight: number;
  /** Heard this many times as far as an ordinary sound (40 blocks), fading as slowly; 1 when left out. */
  reach?: number;
}

/** Built-in shaping per action; the designer's action dials adjust it around these values. */
export const ACTION_BASE: Record<ActionName, ActionBase> = {
  dig: { gain: 0.32, jitter: 0.1, variants: 4, material: true, pitch: 1, length: 1, weight: 0 },
  break: { gain: 0.75, jitter: 0.08, variants: 4, material: true, pitch: 1, length: 1.4, weight: 0 },
  place: { gain: 0.6, jitter: 0.08, variants: 4, material: true, pitch: 0.85, length: 1.1, weight: 0.15 },
  step: { gain: 0.2, jitter: 0.12, variants: 4, material: true, pitch: 1.08, length: 0.85, weight: 0 },
  land: { gain: 0.42, jitter: 0.06, variants: 4, material: true, pitch: 0.75, length: 1.3, weight: 0.25 },
  pickup: { gain: 0.2, jitter: 0.35, variants: 1, material: false, pitch: 1, length: 1, weight: 0 },
  drop: { gain: 0.28, jitter: 0.1, variants: 2, material: false, pitch: 1, length: 1, weight: 0 },
  click: { gain: 0.12, jitter: 0, variants: 1, material: false, pitch: 1, length: 1, weight: 0 },
  scan: { gain: 0.22, jitter: 0, variants: 1, material: false, pitch: 1, length: 1, weight: 0 },
  thump: { gain: 0.56, jitter: 0.04, variants: 2, material: false, pitch: 1, length: 1, weight: 0 },
  splash: { gain: 0.45, jitter: 0.1, variants: 3, material: false, pitch: 1, length: 1, weight: 0 },
  powerdown: { gain: 0.4, jitter: 0.03, variants: 2, material: false, pitch: 1, length: 1, weight: 0 },
  powerup: { gain: 0.4, jitter: 0.03, variants: 2, material: false, pitch: 1, length: 1, weight: 0 },
  launch: { gain: 0.9, jitter: 0.04, variants: 1, material: false, pitch: 1, length: 1, weight: 0, reach: 12 },
  podup: { gain: 0.6, jitter: 0.05, variants: 2, material: false, pitch: 1, length: 1, weight: 0, reach: 3 },
  poddown: { gain: 0.7, jitter: 0.05, variants: 2, material: false, pitch: 1, length: 1, weight: 0, reach: 3 },
  denied: { gain: 0.35, jitter: 0.03, variants: 1, material: false, pitch: 1, length: 1, weight: 0 },
  flame: { gain: 0.52, jitter: 0.1, variants: 3, material: false, pitch: 1, length: 1, weight: 0 },
};

/** Typical in-game event volume and distance, so previews match what you hear while playing. */
export const PREVIEW_VOLUME: Partial<Record<ActionName, number>> = { step: 0.6, land: 0.7 };
export const PREVIEW_DISTANCE = 2;
