// Background music: plays what `MusicDirector` (music-director.ts) decides, from the library in music-pieces.ts. A few
// times a second `poll` reads the player's place (`game.surroundings()` and the time of day, as ambience does), asks the
// director what should play, and streams that piece through a media element into the sound system's AudioContext
// (`start`, called from `SoundSystem.unlock`, which a user gesture reaches). Streaming keeps a piece out of memory (a
// decoded one was 70 to 80 MB); each piece plays once and silence falls between pieces. Owns the music level, mute and
// mode (kept in localStorage under its own key); a piece is fetched only when it is to play, so nothing loads while
// music is muted. A change of place or mode crossfades. To add a mode: a row in `MODES`, a `GAP` row in the director.

import { duckFor, MusicDirector, type Mode } from './music-director';
import { PIECES, type Piece } from './music-pieces';
import { contextFor, S } from './surroundings';

export interface ModeInfo {
  id: Mode;
  label: string;
  help: string;
}

export const MODES: readonly ModeInfo[] = [
  {
    id: 'context',
    label: 'By place',
    help: 'The music follows where you are and your tech era: biomes, sea, caves, towns, factories and orbit. A piece plays now and then, with a short quiet between.',
  },
  { id: 'calm', label: 'Always calm', help: 'Soft, slow pieces one after another. Good for building and tending a factory.' },
  { id: 'lively', label: 'Always adventurous', help: 'Fuller pieces with more going on, one after another. Good for exploring.' },
];

export interface MusicSettings {
  /** 0–100. */
  volume: number;
  muted: boolean;
  mode: Mode;
  /** Silent while the tab or window is not in focus (`setAway`). */
  muteInBackground: boolean;
}

const STORAGE_KEY = 'opencraft.music';
const DEFAULTS: MusicSettings = { volume: 20, muted: false, mode: 'context', muteInBackground: true };
const FADE_IN = 3;
const CROSSFADE = 4;
/** Seconds between readings of the place. */
const POLL = 0.5;

interface Playing {
  piece: Piece;
  el: HTMLAudioElement;
  source: MediaElementAudioSourceNode;
  gain: GainNode;
}

function loadSettings(): MusicSettings {
  const settings = { ...DEFAULTS };
  try {
    const saved: unknown = JSON.parse(localStorage.getItem(STORAGE_KEY) ?? 'null');
    if (typeof saved === 'object' && saved !== null) {
      const s = saved as Record<string, unknown>;
      if (typeof s.volume === 'number' && Number.isFinite(s.volume)) settings.volume = clamp(s.volume);
      settings.muted = s.muted === true;
      settings.muteInBackground = s.muteInBackground !== false;
      if (MODES.some((m) => m.id === s.mode)) settings.mode = s.mode as Mode;
      else if (s.track === 'adventurous') settings.mode = 'lively'; // before modes: "ambient" was the default, so it maps to the new one
    }
  } catch {
    // Storage unavailable or corrupt: use the defaults.
  }
  return settings;
}

const clamp = (v: number) => Math.round(Math.min(100, Math.max(0, v)));

/** What `Music.poll` reads from the game (surroundings, time of day and completed research era). */
export interface MusicGame {
  surroundings(): Float32Array;
  time_of_day(): number;
  tech_count(): number;
  tech_done(t: number): boolean;
  tech_endless(t: number): boolean;
  tech_packs(t: number): Uint16Array;
}

/** Highest science-pack tier (0 free .. 5 gold/orbit) among completed non-endless technologies. */
function researchEra(game: MusicGame): number {
  let era = 0;
  const n = game.tech_count();
  for (let t = 0; t < n; t++) {
    if (!game.tech_done(t) || game.tech_endless(t)) continue;
    const packs = game.tech_packs(t).length;
    if (packs > era) era = packs;
  }
  return era;
}

export class Music {
  readonly settings = loadSettings();
  /** Public for the debug overlay and tests in the pane. */
  readonly director = new MusicDirector(PIECES, Math.random, this.settings.mode);

  private ctx: AudioContext | null = null;
  private out: GainNode | null = null;
  private current: Playing | null = null;
  /** The tab or window is not in focus (not saved). */
  private away = false;
  /** The master is muted (not saved). */
  private hushed = false;
  /** How much working machinery lowers the music (1 for none). */
  private duck = 1;
  private since = POLL;

  /** `changed` is called after the volume, mute or mode changes. */
  constructor(private readonly changed: () => void) {}

  get audible(): boolean {
    return !this.settings.muted && this.settings.volume > 0;
  }

  /** The piece being played, for the debug overlay. */
  get nowPlaying(): string | null {
    return this.current?.piece.id ?? null;
  }

  /** `to` is where the music goes: the sound system's master. */
  start(ctx: AudioContext, to: AudioNode): void {
    if (this.ctx) return;
    this.ctx = ctx;
    this.out = ctx.createGain();
    this.out.gain.value = this.level();
    this.out.connect(to);
  }

  /** The master is muted (not saved): a piece waits where it is, as it does in the background. */
  setHushed(hushed: boolean): void {
    this.hushed = hushed;
    this.apply();
  }

  /** Volume, mute and the background setting back to their defaults (the mode stays). */
  resetLevels(): void {
    const { volume, muteInBackground } = DEFAULTS;
    Object.assign(this.settings, { volume, muted: false, muteInBackground });
    this.update();
  }

  setVolume(volume: number): void {
    this.settings.volume = clamp(volume);
    this.update();
  }

  setMuted(muted: boolean): void {
    this.settings.muted = muted;
    this.update();
  }

  setMuteInBackground(on: boolean): void {
    this.settings.muteInBackground = on;
    this.update();
  }

  /** Called when the tab or window loses or gains focus: fades out (or back in) if `muteInBackground`. */
  setAway(away: boolean): void {
    this.away = away;
    this.apply();
  }

  setMode(mode: Mode): void {
    if (!MODES.some((m) => m.id === mode)) return;
    this.settings.mode = mode;
    this.director.setMode(mode);
    this.update();
  }

  /** Call every frame with the game: reads the place twice a second and starts or fades pieces as the director says. */
  poll(game: MusicGame, dt: number): void {
    if (!this.ctx || !this.out) return;
    const hushed = !this.audible || this.hushed || (this.away && this.settings.muteInBackground);
    this.pauseWhile(hushed);
    if (hushed) return;
    this.since += dt;
    if (this.since < POLL) return;
    const elapsed = this.since;
    this.since = 0;
    const s = game.surroundings();
    this.duck = duckFor(s[S.din]);
    this.apply();
    const play = this.director.update(elapsed, contextFor(s, game.time_of_day(), researchEra(game)));
    if (play) this.begin(play.piece as Piece, play.crossfade);
  }

  private level(): number {
    const hushed = this.hushed || (this.away && this.settings.muteInBackground);
    return this.audible && !hushed ? (this.settings.volume / 100) ** 2 * this.duck : 0;
  }

  private apply(): void {
    const { ctx, out } = this;
    if (ctx && out) out.gain.setTargetAtTime(this.level(), ctx.currentTime, 0.4);
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

  /** A hushed piece waits where it is (no decoding in the background) and goes on when the music is heard again. */
  private pauseWhile(hushed: boolean): void {
    const el = this.current?.el;
    if (!el) return;
    if (hushed && !el.paused) el.pause();
    else if (!hushed && el.paused && !el.ended) void el.play().catch(() => undefined);
  }

  private begin(piece: Piece, crossfade: boolean): void {
    const { ctx, out } = this;
    if (!ctx || !out) return;
    const el = new Audio();
    el.preload = 'auto';
    el.src = piece.url;
    const source = ctx.createMediaElementSource(el);
    const gain = ctx.createGain();
    const now = ctx.currentTime;
    const fade = crossfade && this.current ? CROSSFADE : FADE_IN;
    gain.gain.setValueAtTime(0, now);
    gain.gain.linearRampToValueAtTime(piece.gain, now + fade);
    source.connect(gain).connect(out);
    const playing: Playing = { piece, el, source, gain };
    el.onended = () => {
      if (this.current !== playing) return;
      this.current = null;
      dispose(playing);
      this.director.ended();
    };
    el.onerror = () => {
      console.warn(`Music "${piece.id}" could not be played`);
      if (this.current === playing) this.current = null;
      dispose(playing);
      this.director.failed(piece.id);
    };
    void el.play().catch((e) => {
      console.warn(`Music "${piece.id}" was not allowed to play`, e);
      el.onerror?.(new Event('error'));
    });
    const old = this.current;
    this.current = playing;
    if (old) {
      old.el.onended = null;
      old.el.onerror = null;
      old.gain.gain.cancelScheduledValues(now);
      old.gain.gain.setValueAtTime(old.gain.gain.value, now);
      old.gain.gain.linearRampToValueAtTime(0, now + fade);
      setTimeout(() => dispose(old), (fade + 0.5) * 1000);
    }
  }
}

/** Stops a piece and lets go of its file and nodes. */
function dispose(p: Playing): void {
  p.el.pause();
  p.el.removeAttribute('src');
  p.el.load();
  p.source.disconnect();
  p.gain.disconnect();
}
