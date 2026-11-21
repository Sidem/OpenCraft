// Synthesised flamethrower and fire buffers (FIRE2): warm pink/brown combustion noise, ignition "pffft-fwoomph!"
// burst, seamless long pink loop for the live continuous blowtorch (`flame-loop.ts`), and roaring one-shot flame
// sound for tree ignitions, co-op peers and the Sound Designer preview.

import { biquad, normalize, type HitMods, type Rand } from './synth';

/** Generates warm pink noise (Paul Kellet 7-pole approximation) — natural 1/f fire spectrum instead of harsh white hiss. */
function fillPink(out: Float32Array, rand: Rand): void {
  let b0 = 0, b1 = 0, b2 = 0, b3 = 0, b4 = 0, b5 = 0, b6 = 0;
  for (let i = 0; i < out.length; i++) {
    const w = rand() * 2 - 1;
    b0 = 0.99886 * b0 + w * 0.0555179;
    b1 = 0.99332 * b1 + w * 0.0750759;
    b2 = 0.96900 * b2 + w * 0.1538520;
    b3 = 0.86650 * b3 + w * 0.3104856;
    b4 = 0.55000 * b4 + w * 0.5329522;
    b5 = -0.7616 * b5 - w * 0.0168980;
    out[i] = (b0 + b1 + b2 + b3 + b4 + b5 + b6 + w * 0.5362) * 0.11;
    b6 = w * 0.115926;
  }
}

/** Seamless looping pink noise buffer (`seconds` long, crossfaded at the boundary so it never clicks or repeats audibly). */
export function pinkLoop(sr: number, seconds: number, rand: Rand): Float32Array {
  const n = Math.ceil(sr * seconds);
  const fade = Math.min(Math.floor(sr * 0.12), Math.floor(n / 4));
  const raw = new Float32Array(n + fade);
  fillPink(raw, rand);
  const out = raw.subarray(0, n);
  for (let i = 0; i < fade; i++) {
    const a = i / fade;
    out[i] = out[i] * a + raw[n + i] * (1 - a);
  }
  return normalize(out, 0.9);
}

/**
 * Pressurized flamethrower ignition burst ("pffft-FWOOOMPH!"): crisp valve/piezo click + short high-pressure gas hiss
 * leading into a deep chesty sub-bass and mid-roar combustion bloom as the fuel vapor catches fire.
 */
export function ignitionBurst(sr: number, rand: Rand, pitch = 1): Float32Array {
  const len = 0.38;
  const n = Math.ceil(sr * len);
  const hiss = new Float32Array(n);
  const bloom = new Float32Array(n);
  const sub = new Float32Array(n);
  fillPink(bloom, rand);
  fillPink(sub, rand);

  // 1. Sharp metallic piezo click + brief pressurized gas spurt (0..45 ms).
  for (let i = 0; i < n; i++) {
    const t = i / sr;
    const click = t < 0.008 ? Math.sin(t * Math.PI * 2 * 3400 * pitch) * (1 - t / 0.008) ** 2 * 0.45 : 0;
    const gasEnv = t < 0.065 ? Math.min(1, t / 0.004) * (1 - t / 0.065) ** 1.4 : 0;
    hiss[i] = (rand() * 2 - 1) * gasEnv + click;
  }
  biquad(hiss, sr, 'bp', 2800 * pitch, 0.9);

  // 2. Sweeping combustion bloom ("FWOOOMPH"): rises fast at 14 ms as vapor ignites, swells then settles into the loop.
  const block = 64;
  for (let start = 0; start < n; start += block) {
    const t = start / sr;
    const u = Math.min(1, t / len);
    const seg = bloom.subarray(start, Math.min(n, start + block));
    const cutoff = (170 + 460 * Math.sin(Math.min(1, u * 1.4) * Math.PI * 0.65)) * pitch;
    biquad(seg, sr, 'bp', cutoff, 0.95);
    const att = t < 0.012 ? 0 : Math.min(1, (t - 0.012) / 0.032);
    const rel = (1 - u) ** 1.25;
    const flutter = 0.85 + 0.15 * Math.sin(t * 72);
    for (let i = 0; i < seg.length; i++) seg[i] *= att * rel * flutter;
  }

  // 3. Deep sub-bass ignition thump (55..135 Hz) felt in the chest when the fireball blooms.
  biquad(sub, sr, 'lp', 145 * pitch, 1.3);
  const out = new Float32Array(n);
  for (let i = 0; i < n; i++) {
    const t = i / sr;
    const u = i / n;
    const thump = t > 0.01 && t < 0.22 ? Math.sin((t - 0.01) * Math.PI * 2 * (88 - 35 * u) * pitch) * Math.exp(-(t - 0.01) * 16) * 0.55 : 0;
    const subEnv = (t < 0.01 ? 0 : Math.min(1, (t - 0.01) / 0.025)) * (1 - u) ** 1.3;
    const raw = hiss[i] * 0.65 + bloom[i] * 1.35 + sub[i] * subEnv * 1.6 + thump;
    out[i] = Math.tanh(raw * 1.35);
  }
  return normalize(out, 0.95);
}

/** One-shot roaring fire burst (tree ignitions, co-op remote flamethrowers, and Sound Designer preview). */
export function flame(sr: number, rand: Rand, mods: HitMods): Float32Array {
  const len = 0.52 * mods.length;
  const n = Math.ceil(sr * len);
  const roar = new Float32Array(n);
  const low = new Float32Array(n);
  const jet = new Float32Array(n);
  fillPink(roar, rand);
  fillPink(low, rand);
  for (let i = 0; i < n; i++) jet[i] = rand() * 2 - 1;

  const block = 64;
  for (let start = 0; start < n; start += block) {
    const t = start / sr;
    const u = Math.min(1, t / len);
    const seg = roar.subarray(start, Math.min(n, start + block));
    // Warm sweeping fiery roar (480 -> 240 Hz) with turbulent 11 Hz + 17 Hz flame-lick flutter.
    biquad(seg, sr, 'bp', (480 - 230 * u) * mods.pitch, 1.0);
    const flutter = 0.72 + 0.18 * Math.sin(t * 69) + 0.10 * Math.sin(t * 107 + 1.3);
    const env = Math.min(1, t / 0.032) * (1 - u * 0.92) ** 1.25 * flutter;
    for (let i = 0; i < seg.length; i++) seg[i] *= env;
  }
  biquad(low, sr, 'lp', 165 * mods.pitch, 1.4);
  biquad(jet, sr, 'bp', 2400 * mods.pitch, 0.9);

  const out = new Float32Array(n);
  for (let i = 0; i < n; i++) {
    const t = i / sr;
    const u = i / n;
    const flutter = 0.75 + 0.25 * Math.sin(t * 62 + Math.sin(t * 19));
    const lowEnv = Math.min(1, t / 0.025) * (1 - u) ** 1.15 * flutter;
    const jetEnv = Math.min(1, t / 0.015) * (1 - u) ** 1.7;
    const crackle = rand() < 0.0018 * (1 - u * 0.4) ? (rand() * 2 - 1) * 0.55 : 0;
    out[i] = Math.tanh((roar[i] * 1.45 + low[i] * lowEnv * 1.7 + jet[i] * jetEnv * 0.35 + crackle) * 1.2);
  }
  return normalize(out, 0.92);
}
