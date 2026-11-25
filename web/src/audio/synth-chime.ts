// Synthesised Codex chimes (ui/codex-toast.ts): `discover` for a new element and `achieve` for an achievement. Each
// note is a struck glass bell: a sine with a few inharmonic partials that fade faster than the fundamental, a soft
// attack and a long ring, plus a slightly detuned twin that beats slowly so the tail shimmers instead of buzzing.

import { normalize, type HitMods } from './synth';

/** Bell partials as (frequency multiple, level); the upper ones die away first. */
const PARTIALS: readonly (readonly [number, number])[] = [
  [1, 1],
  [1.003, 0.45],
  [2, 0.3],
  [2.76, 0.18],
  [5.4, 0.06],
];

/** Adds one bell note into `out`, starting `start` seconds in. */
function bell(out: Float32Array, sr: number, start: number, freq: number, amp: number, decay: number): void {
  const from = Math.floor(start * sr);
  for (let i = from; i < out.length; i++) {
    const t = (i - from) / sr;
    const env = Math.min(1, t / 0.005) * Math.exp(-t / decay);
    if (t > 0.05 && env < 1e-4) break;
    let s = 0;
    for (const [m, a] of PARTIALS) {
      const fade = m >= 2 ? Math.exp(-t / (decay * 0.3)) : 1;
      s += a * fade * Math.sin(2 * Math.PI * freq * m * t);
    }
    out[i] += amp * env * s;
  }
}

/** Renders `notes` as (start s, frequency Hz, level, ring s) bells into a buffer `len` seconds long. */
function render(sr: number, mods: HitMods, len: number, notes: readonly (readonly number[])[]): Float32Array {
  const out = new Float32Array(Math.ceil(sr * len * mods.length));
  for (const [start, freq, amp, ring] of notes) bell(out, sr, start, freq * mods.pitch, amp, ring * mods.length);
  return normalize(out, 0.85);
}

/** A new element: two bright notes rising a fifth (E6, then B6), ringing out. */
export function discover(sr: number, mods: HitMods): Float32Array {
  return render(sr, mods, 1.7, [
    [0, 1318.5, 0.7, 0.42],
    [0.11, 1975.5, 0.55, 0.62],
  ]);
}

/** An achievement: a quick rising C major arpeggio landing on a ringing C6 chord, a sparkle on top. */
export function achieve(sr: number, mods: HitMods): Float32Array {
  return render(sr, mods, 2.6, [
    [0, 523.25, 0.5, 0.35],
    [0.085, 659.25, 0.5, 0.35],
    [0.17, 783.99, 0.55, 0.4],
    [0.28, 1046.5, 0.75, 1.1],
    [0.28, 1318.5, 0.32, 0.95],
    [0.28, 1567.98, 0.28, 0.85],
    [0.4, 2093, 0.22, 0.6],
    [0.52, 3136, 0.1, 0.35],
  ]);
}
