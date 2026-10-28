// Plays the engine's positional sound events (crates/engine/src/sound.rs) through Web Audio with
// distance attenuation and stereo panning. The sounds are files (effects.ts), fetched and decoded when audio
// starts; the sound designer sets a volume per material and per action (settings.ts). `music` streams the
// music pieces (music.ts) and `ambience` (ambience.ts) adds the sounds of the place. This class owns the
// shared AudioContext, so effects, music and ambience are three channels with a level and a mute each.

import { MATERIALS, clampDial, loadSettings, saveSettings, type ActionName, type SoundDesign, type SoundSettings } from './settings';
import { Ambience } from './ambience';
import { EffectBank, effectGain, effectName } from './effects';
import { Music } from './music';

const EVENT_FLOATS = 6;
const MAX_VOICES = 24;
const HEARING_RANGE = 40;

/** Engine event kinds, in order. */
const EVENT_ACTIONS: readonly ActionName[] = ['dig', 'break', 'place', 'step', 'land', 'pickup', 'drop', 'splash', 'powerdown', 'powerup', 'launch', 'podup', 'poddown', 'denied'];

interface ActionBase {
  /** Mix level before the Volume dials. */
  gain: number;
  /** Random playback-rate spread per play, so repeats never sound identical. */
  jitter: number;
  /** Uses the material's sound (as opposed to one shared sound). */
  material: boolean;
  /** Heard this many times as far as an ordinary sound (40 blocks), fading as slowly; 1 when left out. */
  reach?: number;
}

/** Mix level and behaviour per action. Variants are however many files a sound has (effects.ts). */
const ACTION_BASE: Record<ActionName, ActionBase> = {
  dig: { gain: 0.32, jitter: 0.1, material: true },
  break: { gain: 0.75, jitter: 0.08, material: true },
  place: { gain: 0.6, jitter: 0.08, material: true },
  step: { gain: 0.2, jitter: 0.12, material: true },
  land: { gain: 0.42, jitter: 0.06, material: true },
  pickup: { gain: 0.2, jitter: 0.35, material: false },
  drop: { gain: 0.28, jitter: 0.1, material: false },
  click: { gain: 0.12, jitter: 0, material: false },
  scan: { gain: 0.22, jitter: 0, material: false },
  splash: { gain: 0.45, jitter: 0.1, material: false },
  powerdown: { gain: 0.4, jitter: 0.03, material: false },
  powerup: { gain: 0.4, jitter: 0.03, material: false },
  launch: { gain: 0.9, jitter: 0.04, material: false, reach: 12 },
  podup: { gain: 0.6, jitter: 0.05, material: false, reach: 3 },
  poddown: { gain: 0.7, jitter: 0.05, material: false, reach: 3 },
  denied: { gain: 0.35, jitter: 0.03, material: false },
};

/** Typical in-game event volume and distance, so previews match what you hear while playing. */
const PREVIEW_VOLUME: Partial<Record<ActionName, number>> = { step: 0.6, land: 0.7 };
const PREVIEW_DISTANCE = 2;

/** Dial position (0–100, 50 = normal) to gain: silent at 0, +12 dB at 100. */
const level = (dial: number) => (dial / 50) ** 2;
const attenuation = (d: number) => 1 / (1 + (d / 10) ** 2);

export class SoundSystem {
  readonly settings: SoundSettings = loadSettings();
  /** Sound material of the most recent material sound, so the designer can open on it. */
  lastMaterial = 0;
  readonly music = new Music(() => this.notify());
  readonly ambience = new Ambience(() => this.notify());

  private ctx: AudioContext | null = null;
  private master: GainNode | null = null;
  private effects: EffectBank | null = null;
  private readonly listeners = new Set<() => void>();
  private voices = 0;
  private saveTimer = 0;
  /** The tab or window is not in focus (not saved). */
  private away = false;

  constructor() {
    // Switching tabs or windows hushes whatever is set to be muted in the background.
    const focus = () => this.setAway(document.hidden || !document.hasFocus());
    document.addEventListener('visibilitychange', focus);
    window.addEventListener('focus', focus);
    window.addEventListener('blur', focus);
    this.setAway(document.hidden || !document.hasFocus());
  }

  /** Sound effects are muted. */
  get muted(): boolean {
    return this.settings.muted;
  }

  /** Effects, music and ambience are all muted. */
  get silent(): boolean {
    return this.muted && this.music.settings.muted && this.ambience.settings.muted;
  }

  get design(): SoundDesign {
    return this.settings.design;
  }

  /** Calls `fn` after a volume or mute state changes, effects or music. */
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
      this.effects = new EffectBank(ctx);
      this.music.start(ctx);
      this.ambience.start(ctx);
    }
    if (this.ctx.state === 'suspended') void this.ctx.resume();
  }

  /** Call every frame (not while hidden): ambience and music read the player's place a few times a second. */
  poll(game: { surroundings(): Float32Array; time_of_day(): number }, dt: number): void {
    this.ambience.poll(game, dt);
    this.music.poll(game, dt);
  }

  setMuted(muted: boolean): void {
    this.settings.muted = muted;
    this.masterChanged();
  }

  /** The K key: mutes everything, or unmutes everything when it was all off. */
  toggleMute(): boolean {
    const off = !this.silent;
    this.setMuted(off);
    this.music.setMuted(off);
    this.ambience.setMuted(off);
    return off;
  }

  setMuteInBackground(on: boolean): void {
    this.settings.muteInBackground = on;
    this.ambience.setAway(this.away && on);
    this.masterChanged();
  }

  private setAway(away: boolean): void {
    this.away = away;
    this.music.setAway(away);
    this.ambience.setAway(away && this.settings.muteInBackground);
    if (this.master && this.ctx) this.master.gain.setTargetAtTime(this.masterGain(), this.ctx.currentTime, 0.02);
  }

  setVolume(volume: number): void {
    this.settings.volume = clampDial(volume);
    this.masterChanged();
  }

  setMaterialVolume(material: number, value: number): void {
    this.design.materials[MATERIALS[material]].volume = clampDial(value);
    this.save();
  }

  setActionVolume(action: ActionName, value: number): void {
    this.design.actions[action].volume = clampDial(value);
    this.save();
  }

  setDesign(design: SoundDesign): void {
    this.settings.design = design;
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
      const reach = ACTION_BASE[action].reach ?? 1;
      if (d > HEARING_RANGE * reach) continue;
      if (ACTION_BASE[action].material) this.lastMaterial = material;
      const pan = Math.max(-1, Math.min(1, (x * rx + z * rz) / Math.max(d, 1.5))) * 0.75;
      this.play(action, material, events[o + 5] * attenuation(d / reach), pan);
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

  /** A scanner or core-drill reading arrived. */
  scan(): void {
    if (this.audible()) this.play('scan', 0, 1, 0);
  }

  private audible(): boolean {
    return this.ctx?.state === 'running' && this.masterGain() > 0;
  }

  private masterGain(): number {
    const hushed = this.muted || (this.away && this.settings.muteInBackground);
    return hushed ? 0 : 1.25 * (this.settings.volume / 100) ** 2;
  }

  private masterChanged(): void {
    if (this.master && this.ctx) this.master.gain.setTargetAtTime(this.masterGain(), this.ctx.currentTime, 0.02);
    this.save();
    this.notify();
  }

  private notify(): void {
    for (const fn of this.listeners) fn();
  }

  private save(): void {
    clearTimeout(this.saveTimer);
    this.saveTimer = window.setTimeout(() => saveSettings(this.settings), 300);
  }

  private play(action: ActionName, material: number, volume: number, pan: number): void {
    if (this.voices >= MAX_VOICES) return;
    const base = ACTION_BASE[action];
    const name = effectName(action, base.material ? MATERIALS[material] : undefined);
    const buffer = this.effects?.pick(name);
    if (!buffer) return;
    const m = this.design.materials[MATERIALS[material]];
    const gain = base.gain * effectGain(name) * level(this.design.actions[action].volume) * (base.material ? level(m.volume) : 1) * volume;
    if (gain <= 0) return;
    this.voice(buffer, gain, 1 + (Math.random() * 2 - 1) * base.jitter, pan);
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
}
