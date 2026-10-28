// Procedural sounds of the space age (engine events LAUNCH, POD_UP and POD_DOWN, from orbit_fx.rs): a rocket's long
// rumbling roar, a drop pod's pneumatic launch and its retro-burn and touchdown. Built from noise and low sines like
// synth.ts's foley, and heard much further away (`reach` in sound.ts). To add one: a function here, an action in
// settings.ts, its line in sound.ts's `synthesize`.

import { biquad, normalize, type HitMods, type Rand } from './synth';

/** White noise, `n` samples. */
function noise(n: number, rand: Rand): Float32Array {
  const out = new Float32Array(n);
  for (let i = 0; i < n; i++) out[i] = rand() * 2 - 1;
  return out;
}

/** A rocket lifting off: an ignition thump, then a deep crackling roar that swells and fades as it climbs away. */
export function launch(sr: number, rand: Rand, mods: HitMods): Float32Array {
  const len = 4.5 * mods.length;
  const n = Math.ceil(sr * len);
  const roar = biquad(noise(n, rand), sr, 'lp', 260 * mods.pitch, 0.7);
  const hiss = biquad(noise(n, rand), sr, 'bp', 1400 * mods.pitch, 0.6);
  const out = new Float32Array(n);
  for (let i = 0; i < n; i++) {
    const t = i / sr, u = t / len;
    const swell = Math.min(1, t / (0.6 * mods.length)) * (1 - u) ** 1.6;
    const crackle = rand() < 0.004 * (1 - u) ? (rand() * 2 - 1) * 1.5 : 0;
    const thump = Math.sin(2 * Math.PI * 42 * mods.pitch * t) * Math.exp(-t / (0.25 * mods.length));
    out[i] = (roar[i] * 3 + hiss[i] * 0.4 * (1 - u) + crackle) * swell + thump * 1.2;
  }
  return normalize(out);
}

/** A drop pod leaving its tube: a pneumatic thump and a whoosh climbing out of hearing. */
export function podUp(sr: number, rand: Rand, mods: HitMods): Float32Array {
  const len = 1.3 * mods.length;
  const n = Math.ceil(sr * len);
  const out = new Float32Array(n);
  const block = 64;
  for (let start = 0; start < n; start += block) {
    const u = start / n;
    const seg = noise(Math.min(block, n - start), rand);
    // Piecewise filtering is fine for noise (synth.ts `whoosh`).
    biquad(seg, sr, 'bp', (300 + 2200 * u) * mods.pitch, 1.0);
    const env = Math.min(1, u / 0.05) * (1 - u) ** 2;
    for (let i = 0; i < seg.length; i++) out[start + i] = seg[i] * env;
  }
  for (let i = 0; i < n; i++) {
    const t = i / sr;
    out[i] += Math.sin(2 * Math.PI * 70 * mods.pitch * t) * Math.exp(-t / (0.08 * mods.length)) * 0.8;
  }
  return normalize(out);
}

/** A drop pod coming down: a falling retro-burn hiss, then a heavy thud and a short rattle of settling dust. */
export function podDown(sr: number, rand: Rand, mods: HitMods): Float32Array {
  const burn = 0.7 * mods.length, tail = 0.5 * mods.length;
  const n = Math.ceil(sr * (burn + tail));
  const hiss = biquad(noise(n, rand), sr, 'bp', 900 * mods.pitch, 0.5);
  const dust = biquad(noise(n, rand), sr, 'lp', 600 * mods.pitch, 0.7);
  const out = new Float32Array(n);
  for (let i = 0; i < n; i++) {
    const t = i / sr, after = t - burn;
    const burning = after < 0 ? Math.min(1, t / 0.1) * (0.4 + 0.6 * (t / burn)) : Math.exp(-after / 0.03);
    out[i] = hiss[i] * burning * 0.6;
    if (after >= 0) {
      out[i] += Math.sin(2 * Math.PI * 55 * mods.pitch * after) * Math.exp(-after / (0.12 * mods.length)) * 1.4;
      out[i] += dust[i] * Math.exp(-after / (0.15 * mods.length)) * 1.2;
    }
  }
  return normalize(out);
}
