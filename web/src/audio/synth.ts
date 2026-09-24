// Procedural foley synthesis. A material hit is built from four layers, each steered by dials from
// settings.ts:
//   excitation  enveloped noise bursts: Snap (attack), Length (decay), Crunch (smooth hiss or
//               sparse grains), Scatter (one burst or many small ones spread over time)
//   body        the excitation through a band filter: Tone (muffled..bright), Pitch
//   thud        a low sine thump with a slight downward glide: Thud, Pitch
//   ring        the excitation through three tuned resonators with bar-like mode ratios: Ring, Pitch
// Layers are mixed at matched loudness and the result is loudness-normalised, so dials change a
// sound's character rather than its level; Volume is applied at playback.

import type { MaterialParams } from './settings';

export type Rand = () => number;

export function mulberry32(seed: number): Rand {
  let a = seed >>> 0;
  return () => {
    a = (a + 0x6d2b79f5) >>> 0;
    let t = a;
    t = Math.imul(t ^ (t >>> 15), t | 1);
    t ^= t + Math.imul(t ^ (t >>> 7), t | 61);
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
}

/** How an action (placing, landing, ...) reshapes a material's sound. */
export interface HitMods {
  /** Frequency multiplier. */
  pitch: number;
  /** Duration multiplier. */
  length: number;
  /** Added to the Thud dial, on a 0..1 scale. */
  weight: number;
}

type FilterType = 'lp' | 'hp' | 'bp';

/** RBJ-cookbook biquad, applied in place. */
function biquad(x: Float32Array, sr: number, type: FilterType, freq: number, q: number): Float32Array {
  const w = (2 * Math.PI * Math.min(freq, sr * 0.45)) / sr;
  const cos = Math.cos(w), alpha = Math.sin(w) / (2 * q);
  let b0: number, b1: number, b2: number;
  if (type === 'lp') [b0, b1, b2] = [(1 - cos) / 2, 1 - cos, (1 - cos) / 2];
  else if (type === 'hp') [b0, b1, b2] = [(1 + cos) / 2, -(1 + cos), (1 + cos) / 2];
  else [b0, b1, b2] = [alpha, 0, -alpha];
  const a0 = 1 + alpha, a1 = -2 * cos, a2 = 1 - alpha;
  let x1 = 0, x2 = 0, y1 = 0, y2 = 0;
  for (let i = 0; i < x.length; i++) {
    const x0 = x[i];
    const y0 = (b0 * x0 + b1 * x1 + b2 * x2 - a1 * y1 - a2 * y2) / a0;
    x2 = x1; x1 = x0; y2 = y1; y1 = y0;
    x[i] = y0;
  }
  return x;
}

/** Two-pole resonator: rings at `freq` with an amplitude time constant of `decay` seconds. */
function resonate(x: Float32Array, sr: number, freq: number, decay: number): Float32Array {
  const r = Math.exp(-1 / (decay * sr));
  const c = 2 * r * Math.cos((2 * Math.PI * freq) / sr), r2 = r * r;
  const y = new Float32Array(x.length);
  let y1 = 0, y2 = 0;
  for (let i = 0; i < x.length; i++) {
    const v = x[i] + c * y1 - r2 * y2;
    y2 = y1; y1 = v;
    y[i] = v;
  }
  return y;
}

/** Scales `x` to an RMS of 1 over its first 100 ms, so layers can be mixed by perceived level. */
function unitRms(x: Float32Array, sr: number): Float32Array {
  const n = Math.min(x.length, Math.floor(sr * 0.1));
  let sum = 0;
  for (let i = 0; i < n; i++) sum += x[i] * x[i];
  const k = 1 / (Math.sqrt(sum / n) || 1);
  for (let i = 0; i < x.length; i++) x[i] *= k;
  return x;
}

function mix(into: Float32Array, x: Float32Array, gain: number, offset = 0): void {
  const n = Math.min(x.length, into.length - offset);
  for (let i = 0; i < n; i++) into[offset + i] += x[i] * gain;
}

function normalize(x: Float32Array, peak = 0.9): Float32Array {
  let m = 0;
  for (const v of x) m = Math.max(m, Math.abs(v));
  if (m > 0) for (let i = 0; i < x.length; i++) x[i] *= peak / m;
  return x;
}

/**
 * Matches perceived loudness across materials: scales the first 100 ms to a target RMS, then
 * soft-limits with tanh. Plain peak normalisation leaves gritty sounds far too quiet, because a
 * few grain spikes set the peak while carrying little energy.
 */
function loudness(x: Float32Array, sr: number, targetRms = 0.28): Float32Array {
  const n = Math.min(x.length, Math.floor(sr * 0.1));
  let sum = 0;
  for (let i = 0; i < n; i++) sum += x[i] * x[i];
  const k = targetRms / (Math.sqrt(sum / n) || 1);
  for (let i = 0; i < x.length; i++) x[i] = Math.tanh(x[i] * k);
  return x;
}

/** One impact in physical units. */
interface Hit {
  /** Frequency multiplier from Pitch. */
  pitch: number;
  /** Top of the body's noise band, Hz. */
  tone: number;
  attack: number;
  /** Noise envelope time constant, s. */
  decay: number;
  crunch: number;
  bursts: number;
  /** Time over which the bursts are scattered, s. */
  spread: number;
  thud: number;
  thudDecay: number;
  ring: number;
  ringDecay: number;
}

/** Resonator modes for Ring: frequency ratio, decay factor and gain (free-bar ratios sound woody). */
const MODES: [number, number, number][] = [[1, 1, 1], [2.76, 0.6, 0.5], [5.4, 0.35, 0.25]];
const RING_HZ = 420;
const THUD_HZ = 90;

const clamp01 = (v: number) => Math.min(1, Math.max(0, v));

/** Maps dial positions to physical units. Each variant is nudged around the dials by Variation. */
function hitFromDials(m: MaterialParams, mods: HitMods, rand: Rand): Hit {
  const dial = (k: keyof MaterialParams) => m[k] / 100;
  const vary = dial('variation');
  const wobble = (amount: number) => 1 + (rand() * 2 - 1) * amount * vary;
  const pitch = 2 ** ((dial('pitch') - 0.5) * 4) * mods.pitch * wobble(0.06); // ±2 octaves
  const scatter = dial('scatter');
  const thud = clamp01(dial('thud') + mods.weight);
  const ring = dial('ring');
  return {
    pitch,
    tone: 250 * 48 ** dial('tone') * pitch * wobble(0.25), // 250 Hz .. 12 kHz
    attack: 0.0002 * 150 ** (1 - dial('snap')), // 30 ms .. 0.2 ms
    decay: 0.005 * 50 ** dial('length') * mods.length * wobble(0.3), // 5 .. 250 ms
    crunch: dial('crunch'),
    bursts: 1 + Math.round(scatter * 8),
    spread: (0.02 + 0.2 * scatter) * mods.length,
    thud,
    thudDecay: (0.025 + 0.06 * thud) * mods.length,
    ring,
    ringDecay: (0.008 + ring * ring * 0.35) * mods.length,
  };
}

function renderHit(sr: number, rand: Rand, h: Hit): Float32Array {
  const tail = Math.max(h.decay * 7, h.thud > 0.01 ? h.thudDecay * 6 : 0, h.ring > 0.01 ? h.ringDecay * 6 : 0);
  const n = Math.ceil(sr * Math.min(1.5, h.attack + h.spread + tail + 0.005));
  const attack = Math.max(1, h.attack * sr);

  // Excitation: noise bursts with a linear attack and exponential decay. Each burst draws its
  // samples from its own generator, so turning one dial doesn't reshuffle the others' randomness.
  const exc = new Float32Array(n);
  const grainChance = 0.05 - 0.042 * h.crunch; // grittier also means sparser, bigger grains
  for (let b = 0; b < h.bursts; b++) {
    const start = b === 0 ? 0 : Math.floor(rand() ** 1.5 * h.spread * sr);
    const amp = b === 0 ? 1 : 0.25 + 0.6 * rand();
    const fall = Math.exp(-1 / (h.decay * (0.6 + 0.8 * rand()) * sr));
    const noise = mulberry32(Math.floor(rand() * 4294967296));
    let env = amp;
    for (let i = start; i < n; i++) {
      const t = i - start;
      let e: number;
      if (t < attack) e = (amp * t) / attack;
      else {
        e = env;
        env *= fall;
        if (env < 1e-4) break;
      }
      const white = noise() * 2 - 1;
      const grain = noise() < grainChance ? (noise() * 2 - 1) * 4 : 0;
      exc[i] += ((1 - h.crunch) * white + h.crunch * grain) * e;
    }
  }

  // Body: the excitation band-limited by Tone.
  const out = exc.slice();
  biquad(out, sr, 'lp', h.tone, Math.SQRT1_2);
  biquad(out, sr, 'hp', Math.max(40, h.tone * 0.12), Math.SQRT1_2);
  unitRms(out, sr);

  if (h.thud > 0.01) {
    const thud = new Float32Array(n);
    const f0 = (THUD_HZ * h.pitch ** 0.7) / sr;
    const fall = Math.exp(-1 / (h.thudDecay * sr));
    const glideFall = Math.exp(-1 / (0.012 * sr));
    const rise = Math.max(attack, 0.001 * sr);
    let env = 1, glide = 0.6, phase = 0;
    for (let i = 0; i < n; i++) {
      phase += 2 * Math.PI * f0 * (1 + glide);
      glide *= glideFall;
      let e: number;
      if (i < rise) e = i / rise;
      else {
        e = env;
        env *= fall;
        if (env < 1e-4) break;
      }
      thud[i] = Math.sin(phase) * e;
    }
    mix(out, unitRms(thud, sr), h.thud * 0.9);
  }

  if (h.ring > 0.01) {
    const ring = new Float32Array(n);
    for (const [ratio, decay, gain] of MODES) {
      const f = RING_HZ * h.pitch * ratio * (0.97 + 0.06 * rand());
      if (f < sr * 0.45) mix(ring, unitRms(resonate(exc, sr, f, h.ringDecay * decay), sr), gain);
    }
    mix(out, unitRms(ring, sr), h.ring * 0.9);
  }
  return out;
}

/** A single hit of a material: mining, placing, footsteps and landing. */
export function hitSound(sr: number, rand: Rand, m: MaterialParams, mods: HitMods): Float32Array {
  return loudness(renderHit(sr, rand, hitFromDials(m, mods, rand)), sr);
}

/** A block breaking: a cluster of `pieces` hits scattered over a quarter second. */
export function breakSound(sr: number, rand: Rand, m: MaterialParams, mods: HitMods, pieces: number): Float32Array {
  const spread = 0.25 * mods.length;
  const parts: { start: number; data: Float32Array; amp: number }[] = [];
  let n = 0;
  for (let p = 0; p < pieces; p++) {
    const pitch = mods.pitch * (p === 0 ? 1 : 0.85 + 0.35 * rand());
    const data = renderHit(sr, rand, hitFromDials(m, { ...mods, pitch }, rand));
    const start = p === 0 ? 0 : Math.floor(rand() ** 1.3 * spread * sr);
    const amp = p === 0 ? 1 : (0.3 + 0.5 * rand()) * (1 - (0.4 * p) / pieces);
    parts.push({ start, data, amp });
    n = Math.max(n, start + data.length);
  }
  const out = new Float32Array(n);
  for (const { start, data, amp } of parts) mix(out, data, amp, start);
  return loudness(out, sr);
}

/** Item pickup: a short rising blip. */
export function blip(sr: number, mods: HitMods): Float32Array {
  const n = Math.ceil(sr * 0.12 * mods.length);
  const out = new Float32Array(n);
  let phase = 0;
  for (let i = 0; i < n; i++) {
    const t = i / sr;
    const f = (620 + 820 * Math.min(1, t / (0.05 * mods.length))) * mods.pitch;
    phase += (2 * Math.PI * f) / sr;
    const env = Math.min(1, t / 0.003) * Math.exp(-t / (0.035 * mods.length));
    out[i] = (Math.sin(phase) + 0.3 * Math.sin(2 * phase)) * env;
  }
  return normalize(out);
}

/** Throwing an item: a soft band-passed whoosh sweeping upwards. */
export function whoosh(sr: number, rand: Rand, mods: HitMods): Float32Array {
  const n = Math.ceil(sr * 0.2 * mods.length);
  const out = new Float32Array(n);
  const block = 64;
  for (let start = 0; start < n; start += block) {
    const t = start / n;
    const seg = new Float32Array(Math.min(block, n - start));
    for (let i = 0; i < seg.length; i++) seg[i] = rand() * 2 - 1;
    // Piecewise filtering is fine for noise: there is no tone for coefficient steps to click on.
    biquad(seg, sr, 'bp', (600 + 2000 * t) * mods.pitch, 1.2);
    const env = Math.sin(Math.PI * t) ** 2;
    for (let i = 0; i < seg.length; i++) out[start + i] = seg[i] * env;
  }
  return normalize(out);
}

/** Hotbar selection click. */
export function tick(sr: number, mods: HitMods): Float32Array {
  const n = Math.ceil(sr * 0.03 * mods.length);
  const out = new Float32Array(n);
  const w = (2 * Math.PI * 1900 * mods.pitch) / sr;
  for (let i = 0; i < n; i++) out[i] = Math.sin(w * i) * Math.exp(-i / (0.006 * mods.length * sr));
  return normalize(out, 0.6);
}
