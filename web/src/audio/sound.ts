// Plays the engine's positional sound events (crates/engine/src/sound.rs) through Web Audio with
// distance attenuation and stereo panning. Every sound is synthesised from the current sound design
// (settings.ts) on first use, and again after a dial changes, so edits in the sound designer are
// heard immediately, in the designer and in the game. The game ships zero audio assets.

import {
  ACTIONS,
  MATERIALS,
  clampDial,
  loadSettings,
  saveSettings,
  type ActionDial,
  type ActionName,
  type ActionParams,
  type MaterialDial,
  type MaterialParams,
  type SoundDesign,
  type SoundSettings,
} from './settings';
import { blip, breakSound, hitSound, mulberry32, tick, whoosh, type HitMods } from './synth';

const EVENT_FLOATS = 6;
const MAX_VOICES = 24;
const HEARING_RANGE = 40;
/** Background synthesis budget per slice, so warming the cache never stalls a frame. */
const WARM_SLICE_MS = 4;
/** Warm-up waits this long after the last dial change, instead of redoing work mid-drag. */
const WARM_DELAY_MS = 400;

/** Engine event kinds, in order. */
const EVENT_ACTIONS: readonly ActionName[] = ['dig', 'break', 'place', 'step', 'land', 'pickup', 'drop'];

interface ActionBase {
  /** Mix level before the Volume dials. */
  gain: number;
  /** Random playback-rate spread per play, scaled by the material's Variation. */
  jitter: number;
  variants: number;
  /** Uses the material's sound (as opposed to one shared sound). */
  material: boolean;
  pitch: number;
  length: number;
  weight: number;
}

/** Built-in shaping per action; the designer's action dials adjust it around these values. */
const ACTION_BASE: Record<ActionName, ActionBase> = {
  dig: { gain: 0.32, jitter: 0.1, variants: 4, material: true, pitch: 1, length: 1, weight: 0 },
  break: { gain: 0.75, jitter: 0.08, variants: 4, material: true, pitch: 1, length: 1.4, weight: 0 },
  place: { gain: 0.6, jitter: 0.08, variants: 4, material: true, pitch: 0.85, length: 1.1, weight: 0.15 },
  step: { gain: 0.2, jitter: 0.12, variants: 4, material: true, pitch: 1.08, length: 0.85, weight: 0 },
  land: { gain: 0.42, jitter: 0.06, variants: 4, material: true, pitch: 0.75, length: 1.3, weight: 0.25 },
  pickup: { gain: 0.2, jitter: 0.35, variants: 1, material: false, pitch: 1, length: 1, weight: 0 },
  drop: { gain: 0.28, jitter: 0.1, variants: 2, material: false, pitch: 1, length: 1, weight: 0 },
  click: { gain: 0.12, jitter: 0, variants: 1, material: false, pitch: 1, length: 1, weight: 0 },
};

/** Typical in-game event volume and distance, so previews match what you hear while playing. */
const PREVIEW_VOLUME: Partial<Record<ActionName, number>> = { step: 0.6, land: 0.7 };
const PREVIEW_DISTANCE = 2;

/** Dial position (0–100, 50 = normal) to gain: silent at 0, +12 dB at 100. */
const level = (dial: number) => (dial / 50) ** 2;
const attenuation = (d: number) => 1 / (1 + (d / 10) ** 2);

interface Bank {
  action: ActionName;
  material: number;
  buffers: (AudioBuffer | undefined)[];
}

export class SoundSystem {
  readonly settings: SoundSettings = loadSettings();
  /** Sound material of the most recent material sound, so the designer can open on it. */
  lastMaterial = 0;

  private ctx: AudioContext | null = null;
  private master: GainNode | null = null;
  private readonly banks = new Map<string, Bank>();
  private readonly listeners = new Set<() => void>();
  private voices = 0;
  private saveTimer = 0;
  private warmTimer = 0;

  get muted(): boolean {
    return this.settings.muted;
  }

  get design(): SoundDesign {
    return this.settings.design;
  }

  /** Calls `fn` after the master volume or mute state changes. */
  subscribe(fn: () => void): void {
    this.listeners.add(fn);
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
      this.master.gain.value = this.masterGain();
      this.master.connect(compressor);
      this.ctx = ctx;
      this.scheduleWarm(0);
    }
    if (this.ctx.state === 'suspended') void this.ctx.resume();
  }

  setMuted(muted: boolean): void {
    this.settings.muted = muted;
    this.masterChanged();
  }

  toggleMute(): boolean {
    this.setMuted(!this.muted);
    return this.muted;
  }

  setVolume(volume: number): void {
    this.settings.volume = clampDial(volume);
    this.masterChanged();
  }

  setMaterialDial(material: number, dial: MaterialDial, value: number): void {
    this.design.materials[MATERIALS[material]][dial] = clampDial(value);
    // Volume is applied at playback; everything else is baked into the synthesised buffers.
    if (dial !== 'volume') this.invalidate((b) => b.material === material && ACTION_BASE[b.action].material);
    this.save();
  }

  setMaterial(material: number, params: MaterialParams): void {
    Object.assign(this.design.materials[MATERIALS[material]], params);
    this.invalidate((b) => b.material === material && ACTION_BASE[b.action].material);
    this.save();
  }

  setActionDial(action: ActionName, dial: ActionDial, value: number): void {
    this.design.actions[action][dial] = clampDial(value);
    if (dial !== 'volume') this.invalidate((b) => b.action === action);
    this.save();
  }

  setAction(action: ActionName, params: ActionParams): void {
    Object.assign(this.design.actions[action], params);
    this.invalidate((b) => b.action === action);
    this.save();
  }

  setDesign(design: SoundDesign): void {
    this.settings.design = design;
    this.invalidate(() => true);
    this.save();
  }

  /** Plays this frame's engine sound events. `yaw` orients the stereo image to the camera. */
  playEvents(events: Float32Array, count: number, yaw: number): void {
    if (!this.audible()) return;
    const rx = Math.cos(yaw), rz = Math.sin(yaw);
    for (let i = 0; i < count; i++) {
      const o = i * EVENT_FLOATS;
      const action = EVENT_ACTIONS[events[o]];
      if (!action) continue;
      const material = Math.min(MATERIALS.length - 1, Math.max(0, events[o + 1] | 0));
      const x = events[o + 2], y = events[o + 3], z = events[o + 4];
      const d = Math.hypot(x, y, z);
      if (d > HEARING_RANGE) continue;
      if (ACTION_BASE[action].material) this.lastMaterial = material;
      const pan = Math.max(-1, Math.min(1, (x * rx + z * rz) / Math.max(d, 1.5))) * 0.75;
      this.play(action, material, events[o + 5] * attenuation(d), pan);
    }
  }

  /** Plays an action on a material as it would sound in the game, centred. For the sound designer. */
  preview(action: ActionName, material: number): void {
    if (this.audible()) this.play(action, material, (PREVIEW_VOLUME[action] ?? 1) * attenuation(PREVIEW_DISTANCE), 0);
  }

  /** Hotbar selection click. */
  ui(): void {
    if (this.audible()) this.play('click', 0, 1, 0);
  }

  private audible(): boolean {
    return this.ctx?.state === 'running' && !this.muted && this.settings.volume > 0;
  }

  private masterGain(): number {
    return this.muted ? 0 : 1.25 * (this.settings.volume / 100) ** 2;
  }

  private masterChanged(): void {
    if (this.master && this.ctx) this.master.gain.setTargetAtTime(this.masterGain(), this.ctx.currentTime, 0.02);
    this.save();
    for (const fn of this.listeners) fn();
  }

  private save(): void {
    clearTimeout(this.saveTimer);
    this.saveTimer = window.setTimeout(() => saveSettings(this.settings), 300);
  }

  private play(action: ActionName, material: number, volume: number, pan: number): void {
    if (this.voices >= MAX_VOICES) return;
    const base = ACTION_BASE[action];
    const m = this.design.materials[MATERIALS[material]];
    const gain = base.gain * level(this.design.actions[action].volume) * (base.material ? level(m.volume) : 1) * volume;
    if (gain <= 0) return;
    const spread = base.material ? m.variation / 50 : 1;
    const rate = 1 + (Math.random() * 2 - 1) * base.jitter * spread;
    this.voice(this.buffer(action, material, Math.floor(Math.random() * base.variants)), gain, rate, pan);
  }

  private voice(buffer: AudioBuffer, gain: number, rate: number, pan: number): void {
    const ctx = this.ctx!;
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

  private bank(action: ActionName, material: number): Bank {
    const m = ACTION_BASE[action].material ? material : 0;
    const key = `${action}:${m}`;
    let bank = this.banks.get(key);
    if (!bank) {
      bank = { action, material: m, buffers: [] };
      this.banks.set(key, bank);
    }
    return bank;
  }

  private buffer(action: ActionName, material: number, variant: number): AudioBuffer {
    const bank = this.bank(action, material);
    return (bank.buffers[variant] ??= this.synthesize(bank.action, bank.material, variant));
  }

  /** Drops the cached buffers that `affected` selects; they are rebuilt on demand and by warm-up. */
  private invalidate(affected: (bank: Bank) => boolean): void {
    for (const bank of this.banks.values()) if (affected(bank)) bank.buffers = [];
    this.scheduleWarm(WARM_DELAY_MS);
  }

  /** Synthesises every missing buffer in the background, a few milliseconds at a time. */
  private scheduleWarm(delay: number): void {
    if (!this.ctx) return;
    clearTimeout(this.warmTimer);
    this.warmTimer = window.setTimeout(() => {
      const t0 = performance.now();
      for (const action of ACTIONS) {
        const base = ACTION_BASE[action];
        for (let m = 0; m < (base.material ? MATERIALS.length : 1); m++) {
          for (let v = 0; v < base.variants; v++) {
            if (this.bank(action, m).buffers[v]) continue;
            if (performance.now() - t0 > WARM_SLICE_MS) return this.scheduleWarm(0);
            this.buffer(action, m, v);
          }
        }
      }
    }, delay);
  }

  private synthesize(action: ActionName, material: number, variant: number): AudioBuffer {
    const ctx = this.ctx!;
    const sr = ctx.sampleRate;
    const base = ACTION_BASE[action];
    const a = this.design.actions[action];
    const m = this.design.materials[MATERIALS[material]];
    // A fixed seed per sound: moving a dial changes only what that dial controls, not the randomness.
    const rand = mulberry32((ACTIONS.indexOf(action) + 1) * 7919 + material * 101 + variant);
    const mods: HitMods = {
      pitch: base.pitch * 2 ** ((a.pitch / 100 - 0.5) * 2), // ±1 octave
      length: base.length * 4 ** (a.length / 100 - 0.5), // ×0.5 .. ×2
      weight: base.weight + (a.weight / 100 - 0.5),
    };
    let data: Float32Array;
    if (action === 'break') data = breakSound(sr, rand, m, mods, 1 + Math.round((a.debris / 100) * 10));
    else if (action === 'pickup') data = blip(sr, mods);
    else if (action === 'drop') data = whoosh(sr, rand, mods);
    else if (action === 'click') data = tick(sr, mods);
    else data = hitSound(sr, rand, m, mods);
    const buffer = ctx.createBuffer(1, data.length, sr);
    buffer.getChannelData(0).set(data);
    return buffer;
  }
}
