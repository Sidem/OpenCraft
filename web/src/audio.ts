// Procedural sound effects. Like the textures, every sound is synthesised at startup, so the game
// ships zero audio assets. The engine emits positional events (see crates/engine/src/sound.rs);
// this module turns them into Web Audio voices with distance attenuation and stereo panning.

/** Event kinds; must match `crates/engine/src/sound.rs`. */
export const Sfx = { Dig: 0, Break: 1, Place: 2, Step: 3, Land: 4, Pickup: 5, Drop: 6 } as const;

/** Sound materials in engine order; must match `block::sound` in the engine. */
const MATERIALS = ['stone', 'dirt', 'grass', 'sand', 'wood', 'leaves'] as const;

const EVENT_FLOATS = 6;
const VARIANTS = 4;
const MAX_VOICES = 24;
const HEARING_RANGE = 40;
const MUTE_KEY = 'opencraft.muted';

type Bank = 'hit' | 'break' | 'pickup' | 'drop';

/** Mix level, base pitch and source bank for every event kind. */
const KINDS: Record<number, { gain: number; rate: number; jitter: number; bank: Bank }> = {
  [Sfx.Dig]: { gain: 0.32, rate: 1.0, jitter: 0.1, bank: 'hit' },
  [Sfx.Break]: { gain: 0.75, rate: 1.0, jitter: 0.08, bank: 'break' },
  [Sfx.Place]: { gain: 0.6, rate: 0.82, jitter: 0.08, bank: 'hit' },
  [Sfx.Step]: { gain: 0.2, rate: 1.1, jitter: 0.12, bank: 'hit' },
  [Sfx.Land]: { gain: 0.42, rate: 0.72, jitter: 0.06, bank: 'hit' },
  [Sfx.Pickup]: { gain: 0.2, rate: 1.0, jitter: 0.35, bank: 'pickup' },
  [Sfx.Drop]: { gain: 0.28, rate: 1.0, jitter: 0.1, bank: 'drop' },
};

interface Profile {
  /** Filtered-noise body of the impact. */
  filter: FilterType;
  freq: number;
  q: number;
  /** 0 = smooth noise, 1 = sparse grains (crunchy, gritty). */
  crackle: number;
  /** Exponential decay time constant in seconds. */
  decay: number;
  /** Resonant modes: [frequency Hz, decay s, gain]. */
  tones: [number, number, number][];
}

const PROFILES: Record<(typeof MATERIALS)[number], Profile> = {
  stone: { filter: 'bp', freq: 2600, q: 0.8, crackle: 0.3, decay: 0.035, tones: [[170, 0.035, 0.6], [1250, 0.02, 0.15]] },
  dirt: { filter: 'lp', freq: 900, q: 0.7, crackle: 0.55, decay: 0.06, tones: [[90, 0.05, 0.45]] },
  grass: { filter: 'lp', freq: 1700, q: 0.6, crackle: 0.7, decay: 0.07, tones: [[110, 0.04, 0.25]] },
  sand: { filter: 'bp', freq: 3000, q: 0.5, crackle: 0.85, decay: 0.08, tones: [] },
  wood: { filter: 'bp', freq: 1100, q: 1.2, crackle: 0.2, decay: 0.03, tones: [[205, 0.09, 1.0], [470, 0.06, 0.5], [880, 0.04, 0.25]] },
  leaves: { filter: 'bp', freq: 4200, q: 0.7, crackle: 0.8, decay: 0.11, tones: [] },
};

type FilterType = 'lp' | 'hp' | 'bp';

function mulberry32(seed: number): () => number {
  let a = seed >>> 0;
  return () => {
    a = (a + 0x6d2b79f5) >>> 0;
    let t = a;
    t = Math.imul(t ^ (t >>> 15), t | 1);
    t ^= t + Math.imul(t ^ (t >>> 7), t | 61);
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
}

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

function normalize(x: Float32Array, peak = 0.9): Float32Array {
  let m = 0;
  for (const v of x) m = Math.max(m, Math.abs(v));
  if (m > 0) for (let i = 0; i < x.length; i++) x[i] *= peak / m;
  return x;
}

/**
 * Matches perceived loudness across materials: scales the first 100 ms to a target RMS, then
 * soft-limits with tanh. Plain peak normalisation leaves gritty sounds (sand, leaves) far too
 * quiet, because a few grain spikes set the peak while carrying little energy.
 */
function loudness(x: Float32Array, sr: number, targetRms = 0.28): Float32Array {
  const n = Math.min(x.length, Math.floor(sr * 0.1));
  let sum = 0;
  for (let i = 0; i < n; i++) sum += x[i] * x[i];
  const k = targetRms / (Math.sqrt(sum / n) || 1);
  for (let i = 0; i < x.length; i++) x[i] = Math.tanh(x[i] * k);
  return x;
}

/** A single impact: decaying filtered noise (optionally grainy) plus resonant modes. */
function impact(sr: number, rand: () => number, p: Profile, decayScale = 1): Float32Array {
  const decay = p.decay * decayScale;
  const n = Math.ceil(sr * Math.min(0.5, decay * 6 + 0.02));
  const noise = new Float32Array(n);
  const attack = 0.002 * sr;
  for (let i = 0; i < n; i++) {
    const env = Math.min(1, i / attack) * Math.exp(-i / (decay * sr));
    const white = rand() * 2 - 1;
    const grain = rand() < 0.03 ? (rand() * 2 - 1) * 4 : 0;
    noise[i] = ((1 - p.crackle) * white + p.crackle * grain) * env;
  }
  biquad(noise, sr, p.filter, p.freq * (0.9 + rand() * 0.2), p.q);
  normalize(noise, 1);
  for (const [freq, tDecay, gain] of p.tones) {
    const f = freq * (0.95 + rand() * 0.1);
    const phase = rand() * Math.PI * 2;
    for (let i = 0; i < n; i++) {
      noise[i] += Math.sin((2 * Math.PI * f * i) / sr + phase) * Math.exp(-i / (tDecay * decayScale * sr)) * gain * Math.min(1, i / attack);
    }
  }
  return normalize(noise);
}

/** A block breaking: a quick cluster of impacts with a longer tail. */
function crumble(sr: number, rand: () => number, p: Profile): Float32Array {
  const out = new Float32Array(Math.ceil(sr * 0.45));
  const grains = 4 + Math.floor(rand() * 3);
  for (let g = 0; g < grains; g++) {
    const grain = impact(sr, rand, p, 1.4);
    const offset = g === 0 ? 0 : Math.floor(rand() * 0.2 * sr);
    const amp = 1 - g * 0.13;
    for (let i = 0; i < grain.length && offset + i < out.length; i++) out[offset + i] += grain[i] * amp;
  }
  return normalize(out);
}

/** Item pickup: a short rising blip. */
function blip(sr: number): Float32Array {
  const n = Math.ceil(sr * 0.12);
  const out = new Float32Array(n);
  let phase = 0;
  for (let i = 0; i < n; i++) {
    const t = i / sr;
    const f = 620 + 820 * Math.min(1, t / 0.05);
    phase += (2 * Math.PI * f) / sr;
    const env = Math.min(1, t / 0.003) * Math.exp(-t / 0.035);
    out[i] = (Math.sin(phase) + 0.3 * Math.sin(2 * phase)) * env;
  }
  return normalize(out);
}

/** Throwing an item: a soft band-passed whoosh sweeping upwards. */
function whoosh(sr: number, rand: () => number): Float32Array {
  const n = Math.ceil(sr * 0.2);
  const out = new Float32Array(n);
  const block = 64;
  for (let start = 0; start < n; start += block) {
    const t = start / n;
    const seg = new Float32Array(Math.min(block, n - start));
    for (let i = 0; i < seg.length; i++) seg[i] = rand() * 2 - 1;
    // Piecewise filtering is fine for noise: there is no tone for coefficient steps to click on.
    biquad(seg, sr, 'bp', 600 + 2000 * t, 1.2);
    const env = Math.sin(Math.PI * t) ** 2;
    for (let i = 0; i < seg.length; i++) out[start + i] = seg[i] * env;
  }
  return normalize(out);
}

function tick(sr: number): Float32Array {
  const n = Math.ceil(sr * 0.03);
  const out = new Float32Array(n);
  for (let i = 0; i < n; i++) out[i] = Math.sin((2 * Math.PI * 1900 * i) / sr) * Math.exp(-i / (0.006 * sr));
  return normalize(out, 0.6);
}

export class SoundSystem {
  private ctx: AudioContext | null = null;
  private master: GainNode | null = null;
  private banks: Record<Bank, AudioBuffer[][]> = { hit: [], break: [], pickup: [], drop: [] };
  private uiTick: AudioBuffer | null = null;
  private voices = 0;
  muted = false;

  constructor() {
    try {
      this.muted = localStorage.getItem(MUTE_KEY) === '1';
    } catch {
      // Storage unavailable (private mode, blocked site data): default to sound on.
    }
  }

  /** Must be called from a user gesture (browsers block audio until then). Safe to call repeatedly. */
  unlock(): void {
    if (!this.ctx) {
      const ctx = new AudioContext({ latencyHint: 'interactive' });
      const compressor = ctx.createDynamicsCompressor();
      compressor.threshold.value = -12;
      compressor.ratio.value = 6;
      compressor.connect(ctx.destination);
      this.master = ctx.createGain();
      this.master.gain.value = this.muted ? 0 : 0.8;
      this.master.connect(compressor);
      this.ctx = ctx;
      this.synthesize(ctx);
    }
    if (this.ctx.state === 'suspended') void this.ctx.resume();
  }

  toggleMute(): boolean {
    this.muted = !this.muted;
    try {
      localStorage.setItem(MUTE_KEY, this.muted ? '1' : '0');
    } catch {
      // Not persisted; the toggle still applies to this session.
    }
    if (this.master && this.ctx) this.master.gain.setTargetAtTime(this.muted ? 0 : 0.8, this.ctx.currentTime, 0.02);
    return this.muted;
  }

  /** Plays this frame's engine sound events. `yaw` orients the stereo image to the camera. */
  playEvents(events: Float32Array, count: number, yaw: number): void {
    if (!this.ctx || this.ctx.state !== 'running' || this.muted) return;
    const rx = Math.cos(yaw), rz = Math.sin(yaw);
    for (let i = 0; i < count; i++) {
      const o = i * EVENT_FLOATS;
      const [kind, material, x, y, z, volume] = events.subarray(o, o + EVENT_FLOATS);
      const spec = KINDS[kind];
      const variants = spec && this.banks[spec.bank][spec.bank === 'hit' || spec.bank === 'break' ? material : 0];
      if (!variants) continue;
      const d = Math.hypot(x, y, z);
      if (d > HEARING_RANGE) continue;
      const pan = Math.max(-1, Math.min(1, (x * rx + z * rz) / Math.max(d, 1.5))) * 0.75;
      const rate = spec.rate * (1 + (Math.random() * 2 - 1) * spec.jitter);
      this.voice(variants[Math.floor(Math.random() * variants.length)], spec.gain * volume / (1 + (d / 10) ** 2), rate, pan);
    }
  }

  /** Hotbar selection click. */
  ui(): void {
    if (this.ctx?.state === 'running' && !this.muted && this.uiTick) this.voice(this.uiTick, 0.12, 1, 0);
  }

  private voice(buffer: AudioBuffer, gain: number, rate: number, pan: number): void {
    const ctx = this.ctx!;
    if (this.voices >= MAX_VOICES) return;
    const src = ctx.createBufferSource();
    src.buffer = buffer;
    src.playbackRate.value = rate;
    const g = ctx.createGain();
    g.gain.value = gain;
    const p = ctx.createStereoPanner();
    p.pan.value = pan;
    src.connect(g).connect(p).connect(this.master!);
    this.voices++;
    src.onended = () => {
      this.voices--;
      src.disconnect();
      g.disconnect();
      p.disconnect();
    };
    src.start();
  }

  private synthesize(ctx: AudioContext): void {
    const sr = ctx.sampleRate;
    const toBuffer = (data: Float32Array) => {
      const b = ctx.createBuffer(1, data.length, sr);
      b.getChannelData(0).set(data);
      return b;
    };
    MATERIALS.forEach((name, m) => {
      const p = PROFILES[name];
      const hits: AudioBuffer[] = [], breaks: AudioBuffer[] = [];
      for (let v = 0; v < VARIANTS; v++) {
        hits.push(toBuffer(loudness(impact(sr, mulberry32(m * 100 + v), p), sr)));
        breaks.push(toBuffer(loudness(crumble(sr, mulberry32(m * 100 + 50 + v), p), sr)));
      }
      this.banks.hit[m] = hits;
      this.banks.break[m] = breaks;
    });
    this.banks.pickup[0] = [toBuffer(blip(sr))];
    this.banks.drop[0] = [0, 1].map((v) => toBuffer(whoosh(sr, mulberry32(900 + v))));
    this.uiTick = toBuffer(tick(sr));
  }
}
