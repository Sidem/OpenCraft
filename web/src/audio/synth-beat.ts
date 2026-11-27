// Procedural acoustic synthesis for the eight factory-instrument voices (F-BEAT). Pure DSP returning
// Float32Arrays (no Web Audio dependency), tested directly in Node (`web/test/factory-beat.test.mjs`).
//
// Psychoacoustic design goals (zero ear fatigue over multi-hour sessions):
//   - Warm acoustic physical models: bass marimba, wooden slit-drum, felt vibraphone, thumb kalimba,
//     concert rosewood marimba (undercut 4x partial), silky Rhodes tine, glass harp, soft celesta.
//   - Even harmonics & subtle tape warmth (`tanh`) for musical body; steep high-frequency rolloff
//     above each register's natural formant to avoid harshness in the 2.5-5 kHz ear-canal resonance band.
//   - Pitch-scaled exponential decays (lower notes bloom longer, higher notes fade crisply).
//   - Smooth click-free onset and zero-crossing end fades, calibrated by perceived RMS loudness.

/** Deterministic PRNG so every synthesised note buffer is identical across runs. */
function prng(seed: number): () => number {
  let a = seed >>> 0;
  return () => {
    a = (a + 0x6d2b79f5) >>> 0;
    let t = a;
    t = Math.imul(t ^ (t >>> 15), t | 1);
    t ^= t + Math.imul(t ^ (t >>> 7), t | 61);
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
}

/** RBJ biquad filter applied in place (self-contained so Node tests run without extra imports). */
function filter(x: Float32Array, sr: number, type: 'lp' | 'bp' | 'hp', freq: number, q: number): Float32Array {
  const w = (2 * Math.PI * Math.min(freq, sr * 0.44)) / sr;
  const cos = Math.cos(w);
  const alpha = Math.sin(w) / (2 * q);
  let b0: number, b1: number, b2: number;
  if (type === 'lp') [b0, b1, b2] = [(1 - cos) / 2, 1 - cos, (1 - cos) / 2];
  else if (type === 'hp') [b0, b1, b2] = [(1 + cos) / 2, -(1 + cos), (1 + cos) / 2];
  else [b0, b1, b2] = [alpha, 0, -alpha];
  const a0 = 1 + alpha;
  const a1 = -2 * cos;
  const a2 = 1 - alpha;
  let x1 = 0, x2 = 0, y1 = 0, y2 = 0;
  for (let i = 0; i < x.length; i++) {
    const x0 = x[i];
    const y0 = (b0 * x0 + b1 * x1 + b2 * x2 - a1 * y1 - a2 * y2) / a0;
    x2 = x1; x1 = x0; y2 = y1; y1 = y0;
    x[i] = y0;
  }
  return x;
}

/** Removes DC bias, enforces click-free fade-in/fade-out, normalises perceived loudness, and soft-limits. */
function polish(x: Float32Array, sr: number, targetRms: number, attackMs: number): Float32Array {
  // Remove any tiny DC offset from asymmetric transients.
  let mean = 0;
  for (let i = 0; i < x.length; i++) mean += x[i];
  mean /= x.length || 1;
  for (let i = 0; i < x.length; i++) x[i] -= mean;

  // Perceived loudness over the main body (first 160 ms).
  const win = Math.min(x.length, Math.floor(sr * 0.16));
  let sum = 0;
  for (let i = 0; i < win; i++) sum += x[i] * x[i];
  const rms = Math.sqrt(sum / (win || 1)) || 1;
  const scale = targetRms / rms;

  const fadeIn = Math.max(8, Math.floor((attackMs / 1000) * sr));
  const fadeOut = Math.max(16, Math.floor(0.02 * sr));
  for (let i = 0; i < x.length; i++) {
    let v = Math.tanh(x[i] * scale * 1.08) / 1.08;
    if (i < fadeIn) v *= 0.5 - 0.5 * Math.cos((Math.PI * i) / fadeIn);
    const left = x.length - 1 - i;
    if (left < fadeOut) v *= 0.5 - 0.5 * Math.cos((Math.PI * left) / fadeOut);
    x[i] = v;
  }

  // Hard ceiling well below 0 dBFS so chords never clip.
  let peak = 0;
  for (let i = 0; i < x.length; i++) peak = Math.max(peak, Math.abs(x[i]));
  if (peak > 0.88) {
    const k = 0.88 / peak;
    for (let i = 0; i < x.length; i++) x[i] *= k;
  }
  return x;
}

/** Adds a soft band-limited mallet/pluck strike transient into `out`. */
function addMallet(out: Float32Array, sr: number, rand: () => number, centerHz: number, decayS: number, amp: number): void {
  const n = Math.min(out.length, Math.ceil(sr * decayS * 6));
  const noise = new Float32Array(n);
  // Pink-leaning noise (simple 1-pole smooth) sounds organic and wooden rather than hissy.
  let prev = 0;
  for (let i = 0; i < n; i++) {
    const t = i / sr;
    prev = prev * 0.65 + (rand() * 2 - 1) * 0.35;
    noise[i] = prev * Math.exp(-t / decayS);
  }
  filter(noise, sr, 'bp', centerHz, 1.4);
  for (let i = 0; i < n; i++) out[i] += noise[i] * amp;
}

/** 0. Power: deep felt bass marimba / warm wooden sub-pluck. */
function bassMarimba(sr: number, freq: number, rand: () => number): Float32Array {
  const decay = 0.44;
  const out = new Float32Array(Math.ceil(sr * 1.15));
  const w = (2 * Math.PI * freq) / sr;
  for (let i = 0; i < out.length; i++) {
    const t = i / sr;
    const e1 = Math.exp(-t / decay);
    const e2 = Math.exp(-t / (decay * 0.42));
    const e3 = Math.exp(-t / (decay * 0.16));
    // Gentle 2nd harmonic makes bass clearly audible on small laptop speakers without mud.
    out[i] = Math.sin(w * i) * e1 + 0.42 * Math.sin(2 * w * i) * e2 + 0.09 * Math.sin(3 * w * i) * e3;
  }
  addMallet(out, sr, rand, Math.min(260, freq * 2.5), 0.014, 0.22);
  filter(out, sr, 'lp', 620, 0.7);
  return polish(out, sr, 0.3, 6);
}

/** 1. Mining: hollow wooden slit-drum / bass marimba knock. */
function slitDrum(sr: number, freq: number, rand: () => number): Float32Array {
  const decay = 0.24;
  const out = new Float32Array(Math.ceil(sr * 0.72));
  const w = (2 * Math.PI * freq) / sr;
  for (let i = 0; i < out.length; i++) {
    const t = i / sr;
    // Slight downward pitch settle on impact mimics a struck wooden cavity.
    const bend = 1 + 0.025 * Math.exp(-t / 0.015);
    const p = w * i * bend;
    const e1 = Math.exp(-t / decay);
    const eWood = Math.exp(-t / 0.032);
    const e2 = Math.exp(-t / 0.085);
    out[i] = Math.sin(p) * e1 + 0.22 * Math.sin(2 * p) * e2 + 0.16 * Math.sin(2.76 * w * i) * eWood;
  }
  addMallet(out, sr, rand, freq * 2.4, 0.011, 0.3);
  filter(out, sr, 'lp', 1350, 0.7);
  return polish(out, sr, 0.27, 3.5);
}

/** 2. Smelting: warm felt-mallet vibraphone with slow organic motor shimmer. */
function vibraphone(sr: number, freq: number, rand: () => number): Float32Array {
  const decay = 0.56;
  const out = new Float32Array(Math.ceil(sr * 1.45));
  const w = (2 * Math.PI * freq) / sr;
  const lfo = (2 * Math.PI * 4.4) / sr;
  for (let i = 0; i < out.length; i++) {
    const t = i / sr;
    const trem = 0.9 + 0.1 * Math.sin(lfo * i);
    const e1 = Math.exp(-t / decay);
    const e2 = Math.exp(-t / 0.14);
    const e4 = Math.exp(-t / 0.032);
    out[i] = (Math.sin(w * i) * e1 + 0.24 * Math.sin(2 * w * i) * e2 + 0.07 * Math.sin(4 * w * i) * e4) * trem;
  }
  addMallet(out, sr, rand, freq * 2.8, 0.012, 0.12);
  filter(out, sr, 'lp', 1950, 0.7);
  return polish(out, sr, 0.25, 7);
}

/** 3. Refining: warm thumb kalimba (wooden resonator box + plucked metal tine). */
function kalimba(sr: number, freq: number, rand: () => number): Float32Array {
  const decay = 0.28;
  const out = new Float32Array(Math.ceil(sr * 0.82));
  const w = (2 * Math.PI * freq) / sr;
  for (let i = 0; i < out.length; i++) {
    const t = i / sr;
    const e1 = Math.exp(-t / decay);
    const e2 = Math.exp(-t / 0.11);
    const eTine = Math.exp(-t / 0.038);
    const eHigh = Math.exp(-t / 0.014);
    out[i] =
      Math.sin(w * i) * e1 +
      0.2 * Math.sin(2 * w * i) * e2 +
      0.22 * Math.sin(2.92 * w * i) * eTine +
      0.06 * Math.sin(5.4 * w * i) * eHigh;
  }
  addMallet(out, sr, rand, Math.min(2200, freq * 5), 0.007, 0.18);
  filter(out, sr, 'lp', 2800, 0.7);
  return polish(out, sr, 0.24, 2);
}

/** 4. Shaping: concert rosewood marimba (undercut bar with pure 4.0x second partial). */
function marimba(sr: number, freq: number, rand: () => number): Float32Array {
  const decay = 0.23 * Math.pow(330 / freq, 0.35);
  const out = new Float32Array(Math.ceil(sr * 0.68));
  const w = (2 * Math.PI * freq) / sr;
  for (let i = 0; i < out.length; i++) {
    const t = i / sr;
    const e1 = Math.exp(-t / decay);
    const eBody = Math.exp(-t / 0.055);
    const eBar = Math.exp(-t / 0.026);
    out[i] = Math.sin(w * i) * e1 + 0.11 * Math.sin(2 * w * i) * eBody + 0.22 * Math.sin(3.99 * w * i) * eBar;
  }
  addMallet(out, sr, rand, Math.min(1800, freq * 3.5), 0.008, 0.16);
  filter(out, sr, 'lp', 3200, 0.7);
  return polish(out, sr, 0.24, 3);
}

/** 5. Assembly: silky Rhodes electric piano pluck (fast-decaying 2:1 FM tine into warm sine). */
function rhodes(sr: number, freq: number, rand: () => number): Float32Array {
  const decay = 0.42 * Math.pow(440 / freq, 0.3);
  const out = new Float32Array(Math.ceil(sr * 1.15));
  const w = (2 * Math.PI * freq) / sr;
  for (let i = 0; i < out.length; i++) {
    const t = i / sr;
    const e1 = Math.exp(-t / decay);
    const fmIndex = 0.52 * Math.exp(-t / 0.042);
    const carrier = Math.sin(w * i + fmIndex * Math.sin(2 * w * i));
    const twin = Math.sin(w * 1.002 * i) * 0.22;
    out[i] = (carrier + twin) * e1;
  }
  addMallet(out, sr, rand, Math.min(2000, freq * 3), 0.007, 0.08);
  filter(out, sr, 'lp', 3400, 0.7);
  return polish(out, sr, 0.22, 4);
}

/** 6. Chemistry: liquid glass harp / warm plucked harmonic shimmer. */
function glassHarp(sr: number, freq: number, rand: () => number): Float32Array {
  const decay = 0.48 * Math.pow(440 / freq, 0.35);
  const out = new Float32Array(Math.ceil(sr * 1.3));
  const w = (2 * Math.PI * freq) / sr;
  for (let i = 0; i < out.length; i++) {
    const t = i / sr;
    const e1 = Math.exp(-t / decay);
    const e2 = Math.exp(-t / 0.15);
    const e3 = Math.exp(-t / 0.065);
    out[i] =
      (Math.sin(w * i) + 0.26 * Math.sin(w * 1.003 * i)) * e1 +
      0.3 * Math.sin(2 * w * i) * e2 +
      0.1 * Math.sin(3 * w * i) * e3;
  }
  addMallet(out, sr, rand, Math.min(2400, freq * 3.2), 0.009, 0.06);
  filter(out, sr, 'lp', 3900, 0.7);
  return polish(out, sr, 0.2, 8);
}

/** 7. Research: airy soft celesta / music-box glass chime (gentle Fletcher-Munson trim). */
function celesta(sr: number, freq: number, rand: () => number): Float32Array {
  const decay = 0.34 * Math.pow(880 / freq, 0.45);
  const out = new Float32Array(Math.ceil(sr * 0.95));
  const w = (2 * Math.PI * freq) / sr;
  for (let i = 0; i < out.length; i++) {
    const t = i / sr;
    const e1 = Math.exp(-t / decay);
    const e2 = Math.exp(-t / 0.048);
    const eBell = Math.exp(-t / 0.02);
    out[i] =
      (Math.sin(w * i) + 0.32 * Math.sin(w * 1.0022 * i)) * e1 +
      0.14 * Math.sin(2 * w * i) * e2 +
      0.045 * Math.sin(2.76 * w * i) * eBell;
  }
  addMallet(out, sr, rand, Math.min(3000, freq * 2.5), 0.006, 0.05);
  filter(out, sr, 'lp', 4600, 0.7);
  return polish(out, sr, 0.17, 5);
}

const SYNTHS: readonly ((sr: number, freq: number, rand: () => number) => Float32Array)[] = [
  bassMarimba,
  slitDrum,
  vibraphone,
  kalimba,
  marimba,
  rhodes,
  glassHarp,
  celesta,
];

/** Renders one acoustic note buffer for machine family `family` (0..7) at `freq` Hz. */
export function renderBeatNote(sr: number, family: number, freq: number): Float32Array {
  const fn = SYNTHS[family] ?? marimba;
  const seed = ((family + 1) * 2654435761) ^ Math.round(freq * 100);
  return fn(sr, freq, prng(seed));
}
