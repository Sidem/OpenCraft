// Plays the engine's positional sound events (crates/engine/src/sound.rs) through Web Audio with
// distance attenuation and stereo panning. Every sound is synthesised from the current sound design
// (settings.ts) on first use, and again after a dial changes, so edits in the sound designer are
// heard immediately, in the designer and in the game. The only audio files are the music pieces, which
// `music` streams (music.ts); `ambience` (ambience.ts) adds the sounds of the place. This class owns the
// shared AudioContext, so effects, music and ambience are three channels with a level and a mute each, all ending at one
// master (`out`: a level and a mute over everything; K mutes it, so each channel keeps its own settings).

import {
  ACTIONS,
  MATERIALS,
  DEFAULT_LEVELS,
  clampDial,
  loadSettings,
  masterGain,
  saveSettings,
  type ActionDial,
  type ActionName,
  type ActionParams,
  type MaterialDial,
  type MaterialParams,
  type SoundDesign,
  type SoundSettings,
} from './settings';
import { Ambience } from './ambience';
import { FactoryBeat, type BeatGame } from './factory-beat';
import { FlameLoop } from './flame-loop';
import { Music, type MusicGame } from './music';
import { ACTION_BASE, EVENT_ACTIONS, PREVIEW_DISTANCE, PREVIEW_VOLUME } from './sound-actions';
import { blip, breakSound, denied, hitSound, mulberry32, ping, powerDown, powerUp, splash, tick, whoosh, type HitMods } from './synth';
import { achieve, discover } from './synth-chime';
import { flame } from './synth-fire';
import { launch, podDown, podUp } from './synth-orbit';
import { thump } from './synth-seismic';

const EVENT_FLOATS = 6;
const MAX_VOICES = 24;
const HEARING_RANGE = 40;
/** Background synthesis budget per slice, so warming the cache never stalls a frame. */
const WARM_SLICE_MS = 4;
/** Warm-up waits this long after the last dial change, instead of redoing work mid-drag. */
const WARM_DELAY_MS = 400;

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
  readonly music = new Music(() => this.notify());
  readonly ambience = new Ambience(() => this.notify());
  readonly beat = new FactoryBeat(() => this.notify());
  private readonly torch = new FlameLoop();

  private ctx: AudioContext | null = null;
  /** The effects channel's level; its sounds go through the compressor, then `out`. */
  private effects: GainNode | null = null;
  /** The master: effects, music and ambience all end here, then the speakers. */
  private out: GainNode | null = null;
  private readonly banks = new Map<string, Bank>();
  private readonly listeners = new Set<() => void>();
  private voices = 0;
  private saveTimer = 0;
  private warmTimer = 0;
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

  /** Nothing is heard: the master is muted, or effects, music and ambience all are. */
  get silent(): boolean {
    return this.settings.masterMuted || this.channelsMuted;
  }

  private get channelsMuted(): boolean {
    return this.muted && this.music.settings.muted && this.ambience.settings.muted && this.beat.settings.muted;
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
      const out = ctx.createGain();
      out.gain.value = masterGain(this.settings.master, this.settings.masterMuted);
      out.connect(ctx.destination);
      const compressor = ctx.createDynamicsCompressor();
      compressor.threshold.value = -12;
      compressor.ratio.value = 6;
      compressor.connect(out);
      this.effects = ctx.createGain();
      this.effects.gain.value = this.effectsGain();
      this.effects.connect(compressor);
      this.out = out;
      this.ctx = ctx;
      this.scheduleWarm(0);
      this.music.start(ctx, out);
      this.ambience.start(ctx, out);
      this.beat.start(ctx, out);
      this.torch.start(ctx, this.effects);
      this.hush();
    }
    if (this.ctx.state === 'suspended') void this.ctx.resume();
  }

  /** Call every frame (not while hidden): ambience, factory beat, music and the continuous flamethrower torch read game state. */
  poll(game: MusicGame & BeatGame & { torch_burning?: () => boolean; camera_yaw?: () => number; camera_pitch?: () => number }, dt: number): void {
    this.ambience.poll(game, dt);
    this.beat.poll(game, dt);
    this.music.setBeatActivity(this.beat.activity);
    this.music.poll(game, dt);
    const a = this.design.actions.flame;
    this.torch.poll(Boolean(game.torch_burning?.()), game.camera_yaw?.() ?? 0, game.camera_pitch?.() ?? 0, dt, level(a.volume), ACTION_BASE.flame.pitch * 2 ** ((a.pitch / 100 - 0.5) * 2));
  }

  setMuted(muted: boolean): void {
    this.settings.muted = muted;
    this.effectsChanged();
  }

  /** The master dial: 0–100 over everything, leaving each channel's own level alone. */
  setMasterVolume(volume: number): void {
    this.settings.master = clampDial(volume);
    this.masterChanged();
  }

  setMasterMuted(muted: boolean): void {
    this.settings.masterMuted = muted;
    this.masterChanged();
  }

  /** The K key: mutes everything at the master (each channel keeps its level and mute), or unmutes everything. */
  toggleMute(): boolean {
    const off = !this.silent;
    this.setMasterMuted(off);
    if (!off && this.channelsMuted) {
      this.setMuted(false);
      this.music.setMuted(false);
      this.ambience.setMuted(false);
      this.beat.setMuted(false);
    }
    return off;
  }

  /** Master, effects, music and ambience back to their starting levels, unmuted (the music's mode and the sound design stay). */
  resetLevels(): void {
    this.settings.master = DEFAULT_LEVELS.master;
    this.settings.masterMuted = false;
    this.settings.volume = DEFAULT_LEVELS.volume;
    this.settings.muted = false;
    this.settings.muteInBackground = false;
    this.music.resetLevels();
    this.ambience.resetLevels();
    this.beat.resetLevels();
    this.effectsChanged();
    this.masterChanged();
  }

  setMuteInBackground(on: boolean): void {
    this.settings.muteInBackground = on;
    this.hush();
    this.effectsChanged();
  }

  private setAway(away: boolean): void {
    this.away = away;
    this.music.setAway(away);
    this.hush();
    this.glide(this.effects, this.effectsGain());
  }

  setVolume(volume: number): void {
    this.settings.volume = clampDial(volume);
    this.effectsChanged();
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

  /** A centred one-off cue: a scanner reading (`scan`), a Seismic Thumper pulse (`thump`), a Codex chime. */
  cue(action: 'scan' | 'thump' | 'discover' | 'achieve'): void {
    if (this.audible()) this.play(action, 0, 1, 0);
  }

  private audible(): boolean {
    return this.ctx?.state === 'running' && this.effectsGain() > 0 && masterGain(this.settings.master, this.settings.masterMuted) > 0;
  }

  private effectsGain(): number {
    const hushed = this.muted || (this.away && this.settings.muteInBackground);
    return hushed ? 0 : 1.25 * (this.settings.volume / 100) ** 2;
  }

  /** A short glide, so a level change never clicks. */
  private glide(node: GainNode | null, gain: number): void {
    if (node && this.ctx) node.gain.setTargetAtTime(gain, this.ctx.currentTime, 0.02);
  }

  private effectsChanged(): void {
    this.glide(this.effects, this.effectsGain());
    this.save();
    this.notify();
  }

  private masterChanged(): void {
    this.glide(this.out, masterGain(this.settings.master, this.settings.masterMuted));
    this.hush();
    this.save();
    this.notify();
  }

  /** Music and ambience stop (rather than play on unheard) while the master is muted, and ambience also in the background if set. */
  private hush(): void {
    const awayBg = this.settings.masterMuted || (this.away && this.settings.muteInBackground);
    this.music.setHushed(this.settings.masterMuted);
    this.ambience.setAway(awayBg);
    this.beat.setAway(awayBg);
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
    src.connect(g).connect(p).connect(this.effects!);
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
    const data =
      action === 'break' ? breakSound(sr, rand, m, mods, 1 + Math.round((a.debris / 100) * 10))
      : action === 'pickup' ? blip(sr, mods) : action === 'drop' ? whoosh(sr, rand, mods)
      : action === 'click' ? tick(sr, mods) : action === 'scan' ? ping(sr, mods) : action === 'thump' ? thump(sr, rand, mods)
      : action === 'splash' ? splash(sr, rand, mods) : action === 'powerdown' ? powerDown(sr, rand, mods)
      : action === 'powerup' ? powerUp(sr, rand, mods) : action === 'launch' ? launch(sr, rand, mods)
      : action === 'podup' ? podUp(sr, rand, mods) : action === 'poddown' ? podDown(sr, rand, mods)
      : action === 'denied' ? denied(sr, mods) : action === 'flame' ? flame(sr, rand, mods)
      : action === 'discover' ? discover(sr, mods) : action === 'achieve' ? achieve(sr, mods)
      : hitSound(sr, rand, m, mods);
    const buffer = ctx.createBuffer(1, data.length, sr);
    buffer.getChannelData(0).set(data);
    return buffer;
  }
}
