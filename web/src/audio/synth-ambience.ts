// Procedural sounds of the open world for ambience.ts: two seamless noise beds (white, for wind, surf and running
// water, which the host filters; and a low rumble for the underground) and the one-shots that sit on top of them
// (a bird's call, a cricket's chirps, a cave drip, a raindrop). Built from noise and sines like synth.ts's foley, with a seeded
// random so every run sounds the same. To add one: a function here, a row in ambience.ts.

import { biquad, normalize, type Rand } from './synth';

const TAU = 2 * Math.PI;

/** Seamless looping noise of `seconds`: white, or `rumble` (brown-ish, mostly below 150 Hz). */
export function bed(sr: number, seconds: number, rand: Rand, kind: 'white' | 'rumble'): Float32Array {
  const n = Math.floor(sr * seconds);
  const out = new Float32Array(n);
  for (let i = 0; i < n; i++) out[i] = rand() * 2 - 1;
  if (kind === 'white') return out;
  // Filtering the buffer twice over and keeping the second pass starts the loop with the filter settled.
  const twice = new Float32Array(2 * n);
  twice.set(out);
  twice.set(out, n);
  biquad(twice, sr, 'lp', 150, 0.7);
  biquad(twice, sr, 'lp', 220, 0.5);
  return normalize(twice.slice(n), 0.9);
}

/** A bird: one to four short notes, each a whistle that bends and trembles, in a high register. */
export function birdCall(sr: number, rand: Rand): Float32Array {
  const notes = 1 + Math.floor(rand() * 4);
  const base = 2300 + rand() * 1700;
  const parts: Float32Array[] = [];
  for (let k = 0; k < notes; k++) {
    const len = Math.floor(sr * (0.05 + rand() * 0.09));
    const note = new Float32Array(len);
    const f0 = base * (0.85 + rand() * 0.45), f1 = f0 * (0.65 + rand() * 0.9);
    const trill = 20 + rand() * 40, depth = rand() * 0.05;
    let phase = 0;
    for (let i = 0; i < len; i++) {
      const u = i / len, t = i / sr;
      phase += (TAU * (f0 + (f1 - f0) * u) * (1 + depth * Math.sin(TAU * trill * t))) / sr;
      note[i] = Math.sin(phase) * Math.sin(Math.PI * u) ** 2;
    }
    parts.push(note, new Float32Array(Math.floor(sr * (0.04 + rand() * 0.08))));
  }
  const out = new Float32Array(parts.reduce((n, p) => n + p.length, 0));
  let at = 0;
  for (const p of parts) {
    out.set(p, at);
    at += p.length;
  }
  return normalize(out, 0.5);
}

/** A cricket: a few short bursts of rapid pulses on a high carrier. */
export function cricket(sr: number, rand: Rand): Float32Array {
  const carrier = 4000 + rand() * 700;
  const groups = 2 + Math.floor(rand() * 3), pulses = 3 + Math.floor(rand() * 3);
  const pulse = Math.floor(sr * 0.013), gap = Math.floor(sr * 0.02), pause = Math.floor(sr * 0.16);
  const out = new Float32Array(groups * (pulses * (pulse + gap) + pause));
  let at = 0;
  for (let g = 0; g < groups; g++) {
    for (let p = 0; p < pulses; p++) {
      for (let i = 0; i < pulse; i++) {
        out[at + i] = Math.sin((TAU * carrier * i) / sr) * Math.sin((Math.PI * i) / pulse) * (0.8 + 0.2 * rand());
      }
      at += pulse + gap;
    }
    at += pause;
  }
  return normalize(out, 0.35);
}

/** One raindrop landing: a tick of filtered noise whose brightness, length and loudness differ from drop to drop, and now
 *  and then a tiny watery ping behind it. Played at random, many a second, over the rain bed so it never loops audibly. */
export function raindrop(sr: number, rand: Rand): Float32Array {
  const n = Math.floor(sr * 0.09);
  const out = new Float32Array(n);
  const tau = 0.004 + rand() * 0.012;
  for (let i = 0; i < n; i++) out[i] = (rand() * 2 - 1) * Math.exp(-i / sr / tau);
  biquad(out, sr, 'bp', 1800 + rand() * 4200, 0.8 + rand() * 1.2);
  if (rand() < 0.4) {
    const f = 2200 + rand() * 2600, decay = 0.008 + rand() * 0.012;
    for (let i = 0; i < n; i++) out[i] += 0.5 * Math.sin((TAU * f * i) / sr) * Math.exp(-i / sr / decay);
  }
  return normalize(out, 0.5);
}

/** A drop of water landing in a cave pool: a short falling blip and a quieter ring behind it. */
export function drip(sr: number, rand: Rand): Float32Array {
  const n = Math.floor(sr * 0.5);
  const out = new Float32Array(n);
  const f = 900 + rand() * 700;
  const ring = Math.floor(sr * (0.07 + rand() * 0.05));
  for (let i = 0; i < n; i++) {
    const t = i / sr;
    const blip = Math.sin(TAU * f * (1 - 0.45 * (1 - Math.exp(-t / 0.03))) * t) * Math.exp(-t / 0.035);
    const echo = i >= ring ? Math.sin(TAU * f * 0.9 * t) * Math.exp(-(t - ring / sr) / 0.05) * 0.25 : 0;
    out[i] = blip + echo;
  }
  return normalize(out, 0.6);
}
