// Music theory and groove planner for the factory instrument (F-BEAT): pure logic with no Web Audio,
// tested directly in Node (`web/test/factory-beat.test.mjs`). Keep to syntax Node can strip types from
// (no enums, no constructor parameter properties).
//
// Harmonic foundation: D major pentatonic {D, E, F#, A, B}. Every note is 100% diatonic in all three
// OpenCraft factory soundtrack keys (Copper & Cogs: D major; Iron Cadence: E Dorian; Silicon Pulse:
// F# minor / A major), with zero semitone or tritone clashes across any combination of machines.
// A slow 4-bar harmonic drift (Dadd9 -> F#m11 -> Em9 -> A9sus4, 16 seconds per cycle) keeps long
// sessions warm and alive, while interlocking Euclidean/hocket patterns per machine family weave
// together without muddying the mix. Starved or blocked machines drop beats proportionally to their
// stalled share, leaving audible gaps in the groove before the player even turns around.

/** Simulation ticks (60 Hz) per beat subdivision: 15 ticks = 0.25 s (120 BPM eighth notes). */
export const TICKS_PER_STEP = 15;
/** Subdivisions per bar (4 seconds). */
export const STEPS_PER_BAR = 16;
/** Bars in one harmonic phrase (16 seconds). */
export const BARS_PER_CYCLE = 4;
/** Hard cap on simultaneously sounding machine families (docs/DECISIONS.md 2026-10-16). */
export const MAX_ACTIVE_FAMILIES = 4;
/** Layout of `Game.factory_beat()` (crates/engine/src/factory/beat.rs). */
export const FAMILY_COUNT = 8;
export const FAMILY_STRIDE = 8;

/** Index offsets inside one family's slice of `factory_beat()`. */
export const F = { strength: 0, dist: 1, rx: 2, rz: 3, health: 4, pace: 5, phase: 6, tier: 7 } as const;

export type FamilyId = 'power' | 'mining' | 'smelting' | 'refining' | 'shaping' | 'assembly' | 'chemistry' | 'research';

export interface VoiceSpec {
  id: FamilyId;
  label: string;
  /** Four register-partitioned D-major-pentatonic frequencies (Hz) for this family. */
  notes: readonly [number, number, number, number];
  /** Interlocking 16-step beat indices where this family sounds. */
  steps: readonly number[];
  /** Relative musical velocity (0..1) per step in `steps`. */
  accents: readonly number[];
  /** Slot indices dropped first when machines in this family starve or back up. */
  dropOrder: readonly number[];
  /** Note index (0..3 into `notes`) played at each slot across the 4-bar harmonic cycle. */
  bars: readonly [readonly number[], readonly number[], readonly number[], readonly number[]];
  /** Base mix trim (calibrated for equal-loudness balance across registers). */
  gain: number;
}

// Equal-temperament A4 = 440 Hz frequencies in D major pentatonic {D, E, F#, A, B}.
const D2 = 73.42, E2 = 82.41, FS2 = 92.5, A2 = 110.0, B2 = 123.47;
const D3 = 146.83, E3 = 164.81, FS3 = 185.0, A3 = 220.0, B3 = 246.94;
const D4 = 293.66, E4 = 329.63, FS4 = 369.99, A4 = 440.0, B4 = 493.88;
const D5 = 587.33, FS5 = 739.99, A5 = 880.0, B5 = 987.77, D6 = 1174.66;

/** Orchestration table in engine family order (0..7). */
export const VOICES: readonly VoiceSpec[] = [
  {
    id: 'power',
    label: 'Power',
    notes: [D2, FS2, E2, A2],
    steps: [0, 8, 12],
    accents: [1.0, 0.78, 0.54],
    dropOrder: [2, 1, 0],
    bars: [
      [0, 3, 0],
      [1, 3, 1],
      [2, 0, 2],
      [3, 2, 3],
    ],
    gain: 0.72,
  },
  {
    id: 'mining',
    label: 'Mining',
    notes: [A2, B2, D3, E3],
    steps: [0, 6, 12],
    accents: [0.95, 0.8, 0.7],
    dropOrder: [2, 1, 0],
    bars: [
      [2, 0, 2],
      [0, 1, 0],
      [3, 1, 2],
      [0, 3, 2],
    ],
    gain: 0.56,
  },
  {
    id: 'smelting',
    label: 'Smelting',
    notes: [D3, E3, FS3, A3],
    steps: [2, 10, 14],
    accents: [0.9, 0.78, 0.66],
    dropOrder: [2, 1, 0],
    bars: [
      [2, 3, 0],
      [3, 2, 1],
      [1, 0, 2],
      [3, 1, 0],
    ],
    gain: 0.54,
  },
  {
    id: 'refining',
    label: 'Refining',
    notes: [E3, FS3, A3, B3],
    steps: [4, 10, 14],
    accents: [0.92, 0.76, 0.68],
    dropOrder: [2, 1, 0],
    bars: [
      [2, 1, 0],
      [3, 2, 1],
      [3, 0, 1],
      [2, 3, 0],
    ],
    gain: 0.5,
  },
  {
    id: 'shaping',
    label: 'Shaping',
    notes: [D4, E4, FS4, A4],
    steps: [0, 3, 6, 10, 13],
    accents: [0.94, 0.64, 0.84, 0.76, 0.6],
    dropOrder: [4, 1, 3, 2, 0],
    bars: [
      [0, 1, 2, 3, 2],
      [2, 3, 1, 2, 0],
      [1, 2, 3, 1, 0],
      [3, 2, 1, 0, 1],
    ],
    gain: 0.5,
  },
  {
    id: 'assembly',
    label: 'Assembly',
    notes: [FS4, A4, B4, D5],
    steps: [2, 5, 8, 11, 14],
    accents: [0.88, 0.64, 0.82, 0.66, 0.74],
    dropOrder: [3, 1, 4, 2, 0],
    bars: [
      [0, 1, 3, 1, 0],
      [1, 2, 0, 2, 1],
      [2, 3, 0, 1, 2],
      [1, 2, 3, 2, 1],
    ],
    gain: 0.52,
  },
  {
    id: 'chemistry',
    label: 'Chemistry',
    notes: [E4, A4, B4, D5],
    steps: [1, 7, 11, 15],
    accents: [0.74, 0.88, 0.72, 0.64],
    dropOrder: [3, 0, 2, 1],
    bars: [
      [0, 1, 3, 1],
      [1, 2, 0, 2],
      [2, 3, 0, 3],
      [1, 0, 2, 3],
    ],
    gain: 0.48,
  },
  {
    id: 'research',
    label: 'Research',
    notes: [FS5, A5, B5, D6],
    steps: [3, 9, 15],
    accents: [0.78, 0.88, 0.66],
    dropOrder: [2, 0, 1],
    bars: [
      [0, 1, 3],
      [1, 2, 0],
      [2, 3, 1],
      [1, 2, 3],
    ],
    gain: 0.38,
  },
];

export interface BeatSettings {
  /** 0–100. */
  volume: number;
  muted: boolean;
}

const BEAT_STORAGE_KEY = 'opencraft.beat';
/** Balanced starting level: clearly musical beside a factory without crowding effects. */
export const DEFAULT_BEAT_SETTINGS: Readonly<BeatSettings> = { volume: 40, muted: false };

const clampDial = (v: number) => Math.round(Math.min(100, Math.max(0, v)));

export function loadBeatSettings(): BeatSettings {
  const settings: BeatSettings = { ...DEFAULT_BEAT_SETTINGS };
  try {
    const saved: unknown = JSON.parse(localStorage.getItem(BEAT_STORAGE_KEY) ?? 'null');
    if (typeof saved === 'object' && saved !== null) {
      const s = saved as Record<string, unknown>;
      if (typeof s.volume === 'number' && Number.isFinite(s.volume)) settings.volume = clampDial(s.volume);
      settings.muted = s.muted === true;
    }
  } catch {
    // Storage unavailable or corrupt: keep defaults.
  }
  return settings;
}

export function saveBeatSettings(settings: BeatSettings): void {
  try {
    localStorage.setItem(BEAT_STORAGE_KEY, JSON.stringify(settings));
  } catch {
    // Not persisted; settings still apply for this session.
  }
}

export interface BeatNote {
  family: number;
  noteIndex: number;
  freq: number;
  gain: number;
  pan: number;
  /** Subtle low-pass warmth factor (0.55 brownout/distant .. 1.0 close/high-tier). */
  bright: number;
  /** Millisecond offset for organic swing and transient separation. */
  delayMs: number;
}

const clamp01 = (v: number) => Math.min(1, Math.max(0, v));

/** How much an active factory groove ducks background music (1 = none, down to 0.55). */
export const beatDuckFor = (activity: number): number => 1 - 0.45 * clamp01(activity);

/** Smooth distance attenuation: 1 right beside the machine, exact 0 at or past `radius`. */
export function beatAttenuation(dist: number, radius: number): number {
  if (radius <= 0 || dist >= radius) return 0;
  const u = clamp01(1 - dist / radius);
  return u ** 1.4;
}

/** Gentle stereo pan (-0.55..+0.55) from camera-relative position and camera yaw. */
export function beatPan(rx: number, rz: number, dist: number, yaw: number): number {
  const rightX = Math.cos(yaw);
  const rightZ = Math.sin(yaw);
  const proj = (rx * rightX + rz * rightZ) / Math.max(dist, 2.5);
  return Math.max(-1, Math.min(1, proj)) * 0.55;
}

/** Whether slot `slot` of `voice` fires given its family's health (working / total) and power pace. */
export function slotActive(voice: VoiceSpec, slot: number, health: number, pace: number): boolean {
  const n = voice.steps.length;
  const vitality = clamp01(health) * (0.45 + 0.55 * clamp01(pace));
  const keep = Math.min(n, Math.max(1, Math.floor(n * vitality + 0.35)));
  const dropped = n - keep;
  for (let i = 0; i < dropped; i++) {
    if (voice.dropOrder[i] === slot) return false;
  }
  return true;
}

/** Hysteresis bonus for currently-selected families so walking near a midpoint never flickers voices. */
const SELECT_HYSTERESIS = 0.06;
/** Presence smoothing time constants (seconds). */
const RISE_TAU = 0.55;
const FALL_TAU = 0.75;
/** Minimum smoothed presence to emit a note. */
const AUDIBLE_FLOOR = 0.03;
/** Organic swing delay (ms) on odd sixteenth subdivisions. */
const SWING_MS = 10;
/** Micro-flam separation (ms) between simultaneous notes so transients stay crystal clear. */
const FLAM_MS = 3.5;

/**
 * Tracks smoothed family presence and voice selection across frames and plans the exact notes
 * to schedule on each 15-tick beat step.
 */
export class BeatTracker {
  readonly presence = new Float32Array(FAMILY_COUNT);
  private readonly selected = new Uint8Array(FAMILY_COUNT);
  private smoothActivity = 0;

  /** Overall audible groove strength (0..1), used to gently duck background music. */
  get activity(): number {
    return this.smoothActivity;
  }

  /** Clears smoothed state (e.g. when muted or switching worlds). */
  reset(): void {
    this.presence.fill(0);
    this.selected.fill(0);
    this.smoothActivity = 0;
  }

  /** Advances presence smoothing and active-family selection by `dt` seconds. */
  update(readings: ArrayLike<number>, radius: number, dt: number): void {
    const scores: { family: number; strength: number; score: number }[] = [];
    for (let i = 0; i < FAMILY_COUNT; i++) {
      const o = i * FAMILY_STRIDE;
      const raw = readings[o + F.strength] ?? 0;
      const dist = readings[o + F.dist] ?? radius + 1;
      const strength = dist < radius ? clamp01(raw) * beatAttenuation(dist, radius) : 0;
      if (strength > 0.001) {
        const bonus = this.selected[i] ? SELECT_HYSTERESIS : 0;
        scores.push({ family: i, strength, score: strength + bonus });
      }
    }
    scores.sort((a, b) => b.score - a.score || a.family - b.family);
    this.selected.fill(0);
    const chosen = Math.min(MAX_ACTIVE_FAMILIES, scores.length);
    const target = new Float32Array(FAMILY_COUNT);
    for (let k = 0; k < chosen; k++) {
      this.selected[scores[k].family] = 1;
      target[scores[k].family] = scores[k].strength;
    }

    const safeDt = Math.max(0, Math.min(dt, 0.25));
    let sumSq = 0;
    let peak = 0;
    for (let i = 0; i < FAMILY_COUNT; i++) {
      const cur = this.presence[i];
      const dst = target[i];
      const tau = dst > cur ? RISE_TAU : FALL_TAU;
      const alpha = 1 - Math.exp(-safeDt / tau);
      const next = cur + (dst - cur) * alpha;
      this.presence[i] = next < 0.002 && dst === 0 ? 0 : next;
      sumSq += this.presence[i] * this.presence[i];
      if (this.presence[i] > peak) peak = this.presence[i];
    }
    const rawActivity = clamp01(Math.max(peak, Math.sqrt(sumSq) * 0.65));
    const actTau = rawActivity > this.smoothActivity ? 0.6 : 1.1;
    this.smoothActivity += (rawActivity - this.smoothActivity) * (1 - Math.exp(-safeDt / actTau));
    if (this.smoothActivity < 0.002 && rawActivity === 0) this.smoothActivity = 0;
  }

  /** Plans the notes for global beat index `step` (`Math.floor(simTick / TICKS_PER_STEP)`). */
  planStep(step: number, readings: ArrayLike<number>, radius: number, yaw: number): BeatNote[] {
    const s = ((step % STEPS_PER_BAR) + STEPS_PER_BAR) % STEPS_PER_BAR;
    const bar = ((Math.floor(step / STEPS_PER_BAR) % BARS_PER_CYCLE) + BARS_PER_CYCLE) % BARS_PER_CYCLE;
    const swing = s % 2 === 1 ? SWING_MS : 0;
    const notes: BeatNote[] = [];

    for (let fam = 0; fam < FAMILY_COUNT; fam++) {
      const pres = this.presence[fam];
      if (pres < AUDIBLE_FLOOR) continue;
      const o = fam * FAMILY_STRIDE;
      const dist = readings[o + F.dist] ?? radius + 1;
      const rawStrength = readings[o + F.strength] ?? 0;
      if (rawStrength <= 0 || dist >= radius) continue;

      const voice = VOICES[fam];
      const slot = voice.steps.indexOf(s);
      if (slot < 0) continue;

      const health = clamp01(readings[o + F.health] ?? 1);
      const pace = clamp01(readings[o + F.pace] ?? 1);
      if (!slotActive(voice, slot, health, pace)) continue;

      const phase = clamp01(readings[o + F.phase] ?? 0);
      const tier = Math.max(0, Math.min(4, readings[o + F.tier] ?? 0));
      const noteIndex = voice.bars[bar][slot];
      const freq = voice.notes[noteIndex];

      // Subtle phrase contour: gentle breath as a cycle nears completion, softened during brownouts.
      const cycleLift = 0.92 + 0.14 * Math.sin(Math.PI * phase);
      const paceScale = 0.6 + 0.4 * pace;
      const gain = voice.gain * voice.accents[slot] * pres * cycleLift * paceScale;
      const pan = beatPan(readings[o + F.rx] ?? 0, readings[o + F.rz] ?? 0, dist, yaw);
      const bright = clamp01(0.55 + 0.25 * pace + 0.12 * (1 - dist / radius) + 0.02 * tier);

      notes.push({ family: fam, noteIndex, freq, gain, pan, bright, delayMs: swing });
    }

    // Slight bass-to-treble micro-flam so simultaneous chord tones separate cleanly without peak clipping.
    for (let i = 0; i < notes.length; i++) {
      notes[i].delayMs += i * FLAM_MS;
    }
    return notes;
  }
}
