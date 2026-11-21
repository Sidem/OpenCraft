// Synthesised seismic thumper pulse (F-SEIS): heavy hydraulic steel anvil strike, chest-felt sub-bass P-wave drop
// sweeping into bedrock, filtered subterranean reflection rumble and a subtle crystalline telemetry return chime.

import { biquad, normalize, type HitMods, type Rand } from './synth';

/** Deep geophysical seismic pulse: anvil strike + sub-bass drop + subterranean reflection tail + telemetry lock. */
export function thump(sr: number, rand: Rand, mods: HitMods): Float32Array {
  const len = 0.88 * mods.length;
  const n = Math.ceil(sr * len);
  const strike = new Float32Array(n);
  const rumble = new Float32Array(n);
  const out = new Float32Array(n);

  // 1. Heavy hydraulic drop-weight anvil impact (0..45 ms).
  for (let i = 0; i < n; i++) {
    const t = i / sr;
    const noise = rand() * 2 - 1;
    const hitEnv = t < 0.04 ? Math.exp(-t * 85) : 0;
    const clang =
      t < 0.09
        ? (Math.sin(t * Math.PI * 2 * 195 * mods.pitch) * 0.55 + Math.sin(t * Math.PI * 2 * 510 * mods.pitch) * 0.3) *
          Math.exp(-t * 42)
        : 0;
    strike[i] = noise * hitEnv * 0.75 + clang;
    // Subterranean reflection noise bed.
    const echoEnv = Math.min(1, t / 0.025) * Math.exp(-t * 3.8);
    // Faint secondary reflection wave arriving at ~180 ms and ~340 ms (deep rock & bedrock bounce).
    const bounce1 = Math.exp(-(((t - 0.18) / 0.055) ** 2)) * 0.45;
    const bounce2 = Math.exp(-(((t - 0.36) / 0.075) ** 2)) * 0.28;
    rumble[i] = noise * (echoEnv + bounce1 + bounce2);
  }
  biquad(strike, sr, 'bp', 780 * mods.pitch, 0.8);
  biquad(rumble, sr, 'lp', 135 * mods.pitch, 1.4);

  // 2. Massive sub-bass P-wave sweep (98 Hz -> 34 Hz) + delicate geophone return chime at ~420 ms.
  let phase = 0;
  for (let i = 0; i < n; i++) {
    const t = i / sr;
    const u = i / n;
    const freq = (34 + 64 * Math.exp(-t * 14)) * mods.pitch;
    phase += (Math.PI * 2 * freq) / sr;
    const subEnv = Math.min(1, t / 0.006) * Math.exp(-t * 5.2);
    const sub = Math.sin(phase) * subEnv * 1.15;

    // Subtle rising geophone data-return harmonic shimmer as the echo settles.
    const chimeT = t - 0.38;
    const chime =
      chimeT > 0
        ? (Math.sin(chimeT * Math.PI * 2 * 660 * mods.pitch) * 0.11 +
            Math.sin(chimeT * Math.PI * 2 * 990 * mods.pitch) * 0.06) *
          Math.min(1, chimeT / 0.02) *
          Math.exp(-chimeT * 7.5)
        : 0;

    const fade = u > 0.9 ? (1 - u) / 0.1 : 1;
    out[i] = Math.tanh((sub + strike[i] * 0.65 + rumble[i] * 1.25 + chime) * 1.1) * fade;
  }

  return normalize(out, 0.95);
}
