// The factory as an instrument (F-BEAT): turns nearby working machines into a warm, non-fatiguing
// polyrhythmic groove locked to the engine's 60 Hz tick (15 ticks per beat step = 0.25 s).
// Follows the same channel pattern as `Ambience` (ambience.ts): owns its volume & mute in localStorage,
// connects to the shared `AudioContext` master (`start`), polls each frame (`poll`), and reports its
// live `activity` (0..1) so `Music` ducks gently underneath it.
//
// Signal path: dry note voices -> subtle warm wooden-room stereo cross-delay (dark LP 2 kHz, ~18% wet)
// -> master warmth filter (LP 5.2 kHz) -> channel gain -> master out. Zero core state, zero save impact.

import {
  BeatTracker,
  DEFAULT_BEAT_SETTINGS,
  FAMILY_COUNT,
  loadBeatSettings,
  saveBeatSettings,
  TICKS_PER_STEP,
  VOICES,
  type BeatNote,
  type BeatSettings,
} from './beat-score';
import { renderBeatNote } from './synth-beat';

export type { BeatSettings };

/** What `FactoryBeat.poll` reads from the wasm engine. */
export interface BeatGame {
  factory_beat(): Float32Array;
  factory_beat_tick(): number;
  factory_beat_radius(): number;
  camera_yaw(): number;
}

/** Maximum skipped beat steps before resyncing silently (e.g. after `skip_time` or tab resume). */
const MAX_CATCHUP_STEPS = 2;
/** Simultaneous voice cap for the factory beat channel. */
const MAX_BEAT_VOICES = 12;
/** Small lookahead (seconds) so scheduled Web Audio notes land cleanly with zero jitter. */
const SCHEDULE_LOOKAHEAD = 0.015;

const clamp = (v: number) => Math.round(Math.min(100, Math.max(0, v)));

export class FactoryBeat {
  readonly settings = loadBeatSettings();
  /** Exposed for inspection and headless verification. */
  readonly tracker = new BeatTracker();

  private ctx: AudioContext | null = null;
  private out: GainNode | null = null;
  private dryBus: GainNode | null = null;
  private readonly buffers = new Map<string, AudioBuffer>();
  private away = false;
  private lastStep = -1;
  private voices = 0;
  private radius = 26;

  /** `changed` is called after the volume or mute state changes. */
  constructor(private readonly changed: () => void) {}

  get audible(): boolean {
    return !this.settings.muted && this.settings.volume > 0 && !this.away;
  }

  /** Live audible groove strength (0..1) for music ducking; always 0 when muted or away. */
  get activity(): number {
    return this.audible ? this.tracker.activity : 0;
  }

  /** Connects the factory beat bus (with its dark wooden-room stereo echo) to `to`. */
  start(ctx: AudioContext, to: AudioNode): void {
    if (this.ctx) return;
    this.ctx = ctx;

    const out = ctx.createGain();
    out.gain.value = this.level();
    out.connect(to);

    // Master warmth rolloff: removes any harshness in the upper presence band.
    const masterLp = ctx.createBiquadFilter();
    masterLp.type = 'lowpass';
    masterLp.frequency.value = 5200;
    masterLp.Q.value = 0.55;
    masterLp.connect(out);

    const dryBus = ctx.createGain();
    dryBus.gain.value = 1;
    dryBus.connect(masterLp);

    // Subtle dark wooden-room stereo cross-delay (L: 112 ms, R: 178 ms, damped at 2 kHz).
    const delayL = ctx.createDelay(0.5);
    const delayR = ctx.createDelay(0.5);
    delayL.delayTime.value = 0.112;
    delayR.delayTime.value = 0.178;

    const dampL = ctx.createBiquadFilter();
    const dampR = ctx.createBiquadFilter();
    dampL.type = 'lowpass';
    dampR.type = 'lowpass';
    dampL.frequency.value = 2000;
    dampR.frequency.value = 1850;

    const fbL = ctx.createGain();
    const fbR = ctx.createGain();
    fbL.gain.value = 0.24;
    fbR.gain.value = 0.24;

    const wetL = ctx.createGain();
    const wetR = ctx.createGain();
    wetL.gain.value = 0.18;
    wetR.gain.value = 0.18;

    const panL = ctx.createStereoPanner();
    const panR = ctx.createStereoPanner();
    panL.pan.value = -0.65;
    panR.pan.value = 0.65;

    dryBus.connect(delayL).connect(dampL);
    dryBus.connect(delayR).connect(dampR);
    dampL.connect(fbL).connect(delayR);
    dampR.connect(fbR).connect(delayL);
    dampL.connect(wetL).connect(panL).connect(masterLp);
    dampR.connect(wetR).connect(panR).connect(masterLp);

    this.out = out;
    this.dryBus = dryBus;
  }

  setVolume(volume: number): void {
    this.settings.volume = clamp(volume);
    this.update();
  }

  setMuted(muted: boolean): void {
    this.settings.muted = muted;
    if (muted) this.tracker.reset();
    this.update();
  }

  resetLevels(): void {
    Object.assign(this.settings, DEFAULT_BEAT_SETTINGS);
    this.update();
  }

  /** Hushed while the tab is unfocused (if set) or when master is muted (`SoundSystem.hush`). */
  setAway(away: boolean): void {
    this.away = away;
    if (away) this.lastStep = -1;
    this.apply();
  }

  /** Plays a short warm Dadd9 arpeggio so players can judge the mixer level by ear. */
  preview(): void {
    if (!this.ctx || !this.dryBus || !this.audible || this.ctx.state !== 'running') return;
    const notes: BeatNote[] = [
      { family: 0, noteIndex: 0, freq: VOICES[0].notes[0], gain: 0.65, pan: 0, bright: 0.9, delayMs: 0 },
      { family: 4, noteIndex: 2, freq: VOICES[4].notes[2], gain: 0.46, pan: -0.2, bright: 0.95, delayMs: 110 },
      { family: 5, noteIndex: 1, freq: VOICES[5].notes[1], gain: 0.42, pan: 0.2, bright: 0.95, delayMs: 220 },
    ];
    for (const note of notes) this.trigger(note);
  }

  /** Call once per visible frame: tracks nearby machines and fires notes on each 15-tick beat. */
  poll(game: BeatGame, dt: number): void {
    if (!this.ctx || !this.dryBus || !this.audible || this.ctx.state !== 'running') {
      this.tracker.update([], this.radius, dt);
      return;
    }
    this.radius = game.factory_beat_radius();
    const readings = game.factory_beat();
    this.tracker.update(readings, this.radius, dt);

    const step = Math.floor(game.factory_beat_tick() / TICKS_PER_STEP);
    if (this.lastStep < 0 || step < this.lastStep || step - this.lastStep > MAX_CATCHUP_STEPS) {
      this.lastStep = step;
      return;
    }
    if (step === this.lastStep) return;

    const yaw = game.camera_yaw();
    for (let s = this.lastStep + 1; s <= step; s++) {
      const notes = this.tracker.planStep(s, readings, this.radius, yaw);
      for (const note of notes) this.trigger(note);
    }
    this.lastStep = step;
  }

  private level(): number {
    return this.audible ? 1.15 * (this.settings.volume / 100) ** 2 : 0;
  }

  private apply(): void {
    if (this.ctx && this.out) this.out.gain.setTargetAtTime(this.level(), this.ctx.currentTime, 0.05);
  }

  private update(): void {
    saveBeatSettings(this.settings);
    this.apply();
    this.changed();
  }

  private buffer(family: number, noteIndex: number, freq: number): AudioBuffer {
    const fam = Math.max(0, Math.min(FAMILY_COUNT - 1, family));
    const key = `${fam}:${noteIndex}`;
    let buf = this.buffers.get(key);
    if (!buf) {
      const data = renderBeatNote(this.ctx!.sampleRate, fam, freq);
      buf = this.ctx!.createBuffer(1, data.length, this.ctx!.sampleRate);
      buf.getChannelData(0).set(data);
      this.buffers.set(key, buf);
    }
    return buf;
  }

  private trigger(note: BeatNote): void {
    const { ctx, dryBus } = this;
    if (!ctx || !dryBus || this.voices >= MAX_BEAT_VOICES || note.gain <= 0.005) return;

    const src = ctx.createBufferSource();
    src.buffer = this.buffer(note.family, note.noteIndex, note.freq);

    // Subtle distance/brownout darkening filter (warm and acoustic, never shrill).
    const tone = ctx.createBiquadFilter();
    tone.type = 'lowpass';
    tone.frequency.value = 1400 + 3800 * note.bright;
    tone.Q.value = 0.5;

    const gain = ctx.createGain();
    gain.gain.value = note.gain;

    const pan = ctx.createStereoPanner();
    pan.pan.value = note.pan;

    src.connect(tone).connect(gain).connect(pan).connect(dryBus);
    this.voices++;
    src.onended = () => {
      this.voices--;
      src.disconnect();
      tone.disconnect();
      gain.disconnect();
      pan.disconnect();
    };
    src.start(ctx.currentTime + SCHEDULE_LOOKAHEAD + note.delayMs / 1000);
  }
}
