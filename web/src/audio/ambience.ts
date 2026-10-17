// Ambient sound: the beds and one-shots of the place the player is in. A few times a second it reads
// `game.surroundings()` (crates/engine/src/surroundings/mod.rs) and the time of day, turns them into target levels
// (`mixFor`: wind rising with height and openness, surf by the sea, running water, a low room tone and drips
// underground, birds by day near trees, crickets at night), and glides the beds to them; the one-shots (birds,
// crickets, drips) are scheduled at random by their level. Working machines duck everything. Owns its own level,
// mute and settings (localStorage); plays through the sound system's AudioContext (`start`, from
// `SoundSystem.unlock`). To add a sound: a field in `Mix` and `mixFor`, then a bed in `start` or a row in `SHOTS`.

import { bed, birdCall, cricket, drip } from './synth-ambience';
import { mulberry32, type Rand } from './synth';

/** Indices of the floats in `Game.surroundings()`; must match the constants in `surroundings/mod.rs`. */
const S = { biome: 0, sky: 1, open: 2, height: 3, depth: 4, sea: 5, fresh: 6, trees: 7, din: 8 } as const;
/** `Biome` variants in the engine's order. */
const DESERT = 1, HIGHLANDS = 2, BASALT = 4;

/** Target levels, each 0 to 1 (see `mixFor`). */
export interface Mix {
  wind: number;
  /** 0 to 1: how dry and hissing the wind is (deserts, basalt fields). */
  dry: number;
  surf: number;
  river: number;
  hum: number;
  birds: number;
  insects: number;
  drips: number;
  /** How much working machinery quiets everything else. */
  duck: number;
}

const clamp01 = (v: number) => Math.min(1, Math.max(0, v));
const smooth = (a: number, b: number, v: number) => {
  const t = clamp01((v - a) / (b - a));
  return t * t * (3 - 2 * t);
};

/** What should be heard for a `surroundings` reading at the time of day `tod` (0 midnight, 0.5 noon). */
export function mixFor(s: ArrayLike<number>, tod: number): Mix {
  const sky = s[S.sky], open = s[S.open], trees = s[S.trees], biome = s[S.biome];
  const day = smooth(0.2, 0.3, tod) * (1 - smooth(0.7, 0.8, tod));
  const cave = (1 - sky) * clamp01(s[S.depth] / 8);
  const dryland = biome === DESERT ? 1 : biome === BASALT ? 0.5 : 0;
  const high = clamp01((s[S.height] - 20) / 120 + (biome === HIGHLANDS ? 0.15 : 0));
  const wind = sky * (0.3 + 0.7 * open) * (0.1 + 0.9 * high + 0.25 * dryland) * (1 - 0.5 * trees);
  const green = biome === DESERT || biome === BASALT ? 0 : 1;
  return {
    wind: clamp01(wind),
    dry: dryland,
    surf: s[S.sea] * (0.3 + 0.7 * sky),
    river: s[S.fresh] * (0.3 + 0.7 * sky),
    hum: cave,
    birds: day * trees * clamp01(sky * 1.5 - 0.2) * green,
    insects: (1 - day) * sky * (0.4 + 0.6 * trees) * (biome === HIGHLANDS ? 0.4 : biome === DESERT ? 0.3 : green),
    drips: cave * (s[S.depth] > 6 ? 1 : 0.5),
    duck: 1 - 0.55 * s[S.din],
  };
}

export interface AmbienceSettings {
  /** 0–100. */
  volume: number;
  muted: boolean;
}

const STORAGE_KEY = 'opencraft.ambience';
const DEFAULTS: AmbienceSettings = { volume: 50, muted: false };
/** Seconds between readings of the surroundings, and the time a bed takes to settle on a new level. */
const POLL = 0.25;
const GLIDE = 1.2;
const BED_SECONDS = 3;

/** The one-shots: its level in `Mix`, how many variants are synthesised, the wait in seconds at full and at no level. */
interface Shot {
  field: 'birds' | 'insects' | 'drips';
  make: (sr: number, rand: Rand) => Float32Array;
  variants: number;
  gain: number;
  wait: [number, number];
}
const SHOTS: readonly Shot[] = [
  { field: 'birds', make: birdCall, variants: 6, gain: 0.5, wait: [2.5, 14] },
  { field: 'insects', make: cricket, variants: 3, gain: 0.5, wait: [1.2, 6] },
  { field: 'drips', make: drip, variants: 4, gain: 0.7, wait: [2.5, 12] },
];

function loadSettings(): AmbienceSettings {
  const settings = { ...DEFAULTS };
  try {
    const saved: unknown = JSON.parse(localStorage.getItem(STORAGE_KEY) ?? 'null');
    if (typeof saved === 'object' && saved !== null) {
      const s = saved as Record<string, unknown>;
      if (typeof s.volume === 'number' && Number.isFinite(s.volume)) settings.volume = Math.round(Math.min(100, Math.max(0, s.volume)));
      settings.muted = s.muted === true;
    }
  } catch {
    // Storage unavailable or corrupt: use the defaults.
  }
  return settings;
}

/** A looping bed: its `level` (the node the mix sets) and the `filters` the mix may retune. */
interface Bed {
  level: GainNode;
  filters: BiquadFilterNode[];
}

export class Ambience {
  readonly settings = loadSettings();
  /** The latest target levels, for the debug overlay and tests in the pane. */
  readonly mix: Mix = mixFor([0, 1, 1, 0, 0, 0, 0, 0, 0], 0.5);

  private ctx: AudioContext | null = null;
  private out: GainNode | null = null;
  private bus: GainNode | null = null;
  private beds: Record<'wind' | 'surf' | 'river' | 'hum', Bed> | null = null;
  private readonly shotBuffers = new Map<string, AudioBuffer>();
  private readonly waits = SHOTS.map(() => 1);
  private readonly rand = mulberry32(0xa11b1e);
  private since = POLL;
  /** Set by the sound system when the tab is not in focus and effects are to be muted there (not saved). */
  private away = false;

  /** `changed` is called after the volume or mute changes. */
  constructor(private readonly changed: () => void) {}

  get audible(): boolean {
    return !this.settings.muted && this.settings.volume > 0 && !this.away;
  }

  start(ctx: AudioContext): void {
    if (this.ctx) return;
    this.ctx = ctx;
    this.out = ctx.createGain();
    this.out.gain.value = this.level();
    this.out.connect(ctx.destination);
    this.bus = ctx.createGain();
    this.bus.connect(this.out);
    const rand = mulberry32(7);
    const white = this.loop(bed(ctx.sampleRate, BED_SECONDS, rand, 'white'));
    const rumble = this.loop(bed(ctx.sampleRate, BED_SECONDS, rand, 'rumble'));
    this.beds = {
      wind: this.makeBed(white, [filter(ctx, 'bandpass', 500, 0.6)], 0.17, 0.25),
      surf: this.makeBed(white, [filter(ctx, 'lowpass', 750, 0.7)], 0.09, 0.5),
      river: this.makeBed(white, [filter(ctx, 'highpass', 300, 0.5), filter(ctx, 'lowpass', 3200, 0.5)], 5.5, 0.1),
      hum: this.makeBed(rumble, [], 0.05, 0.1),
    };
  }

  setVolume(volume: number): void {
    this.settings.volume = Math.round(Math.min(100, Math.max(0, volume)));
    this.update();
  }

  setMuted(muted: boolean): void {
    this.settings.muted = muted;
    this.update();
  }

  /** Hushed while the tab or window is not in focus (when the sound effects are set to be). */
  setAway(away: boolean): void {
    this.away = away;
    this.apply();
  }

  /** Call every frame with the game: reads the surroundings a few times a second and plays the one-shots. */
  poll(game: { surroundings(): Float32Array; time_of_day(): number }, dt: number): void {
    if (!this.ctx || !this.beds || !this.audible) return;
    this.since += dt;
    if (this.since >= POLL) {
      this.since = 0;
      this.retarget(mixFor(game.surroundings(), game.time_of_day()));
    }
    SHOTS.forEach((shot, i) => {
      this.waits[i] -= dt;
      if (this.waits[i] > 0) return;
      const level = this.mix[shot.field];
      this.waits[i] = level < 0.05 ? 1 : shot.wait[1] + (shot.wait[0] - shot.wait[1]) * level * (0.6 + 0.8 * this.rand());
      if (level >= 0.05) this.play(shot, level);
    });
  }

  /** Aims the beds at `target` (they glide there). */
  private retarget(target: Mix): void {
    Object.assign(this.mix, target);
    const { ctx, beds, bus } = this;
    if (!ctx || !beds || !bus) return;
    const now = ctx.currentTime;
    const to = (param: AudioParam, value: number) => param.setTargetAtTime(value, now, GLIDE);
    const { wind, dry, surf, river, hum, duck } = target;
    to(bus.gain, duck);
    to(beds.wind.level.gain, wind * 0.55);
    to(beds.wind.filters[0].frequency, 280 + 900 * wind + 900 * dry);
    to(beds.wind.filters[0].Q, 0.6 + 0.9 * dry);
    to(beds.surf.level.gain, surf * 0.5);
    to(beds.river.level.gain, river * 0.3);
    to(beds.hum.level.gain, hum * 0.7);
  }

  private play(shot: Shot, level: number): void {
    const { ctx, bus } = this;
    if (!ctx || !bus) return;
    const variant = Math.floor(this.rand() * shot.variants);
    const key = `${shot.field}:${variant}`;
    let buffer = this.shotBuffers.get(key);
    if (!buffer) {
      const data = shot.make(ctx.sampleRate, mulberry32(1000 * SHOTS.indexOf(shot) + variant));
      buffer = ctx.createBuffer(1, data.length, ctx.sampleRate);
      buffer.getChannelData(0).set(data);
      this.shotBuffers.set(key, buffer);
    }
    const src = ctx.createBufferSource();
    src.buffer = buffer;
    src.playbackRate.value = 0.94 + this.rand() * 0.12;
    const gain = ctx.createGain();
    gain.gain.value = shot.gain * (0.4 + 0.6 * level) * (0.7 + 0.3 * this.rand());
    const pan = ctx.createStereoPanner();
    pan.pan.value = this.rand() * 1.6 - 0.8;
    src.connect(gain).connect(pan).connect(bus);
    src.onended = () => {
      src.disconnect();
      gain.disconnect();
      pan.disconnect();
    };
    src.start();
  }

  private loop(data: Float32Array): AudioBuffer {
    const buffer = this.ctx!.createBuffer(1, data.length, this.ctx!.sampleRate);
    buffer.getChannelData(0).set(data);
    return buffer;
  }

  /** `source` looping through `filters` into a swell stage (a slow oscillator moving it by `depth`) and a silent level. */
  private makeBed(source: AudioBuffer, filters: BiquadFilterNode[], swellHz: number, depth: number): Bed {
    const ctx = this.ctx!;
    const src = ctx.createBufferSource();
    src.buffer = source;
    src.loop = true;
    src.loopStart = 0;
    const swell = ctx.createGain();
    swell.gain.value = 1 - depth;
    const lfo = ctx.createOscillator();
    lfo.frequency.value = swellHz;
    const lfoDepth = ctx.createGain();
    lfoDepth.gain.value = depth;
    lfo.connect(lfoDepth).connect(swell.gain);
    const level = ctx.createGain();
    level.gain.value = 0;
    let node: AudioNode = src;
    for (const f of filters) node = node.connect(f);
    node.connect(swell).connect(level).connect(this.bus!);
    src.start(0, this.rand() * BED_SECONDS);
    lfo.start();
    return { level, filters };
  }

  private level(): number {
    return this.audible ? 0.8 * (this.settings.volume / 100) ** 2 : 0;
  }

  private apply(): void {
    const { ctx, out } = this;
    if (ctx && out) out.gain.setTargetAtTime(this.level(), ctx.currentTime, 0.05);
    if (this.audible) this.since = POLL; // read at once when it comes back
  }

  private update(): void {
    try {
      localStorage.setItem(STORAGE_KEY, JSON.stringify(this.settings));
    } catch {
      // Not persisted; the settings still apply for this session.
    }
    this.apply();
    this.changed();
  }
}

function filter(ctx: AudioContext, type: BiquadFilterType, frequency: number, q: number): BiquadFilterNode {
  const f = ctx.createBiquadFilter();
  f.type = type;
  f.frequency.value = frequency;
  f.Q.value = q;
  return f;
}
