// Background music: one piece ("Open Ground") in two seamless loops, a fuller adventurous version and a
// calm ambient one rendered at the same loudness. Owns the music level, mute and track choice (kept in
// localStorage under its own key) and plays the chosen loop through the sound system's AudioContext
// (`start`, called from `SoundSystem.unlock`, which a user gesture reaches). A loop is fetched and decoded
// only once music is audible, kept only while it plays, and looped as a decoded buffer because that is the
// one gapless way in browsers. Switching tracks crossfades.
// To add a track: drop an .ogg loop in audio/music/ and add a row to `TRACKS`.

export interface Track {
  id: string;
  label: string;
  help: string;
  url: string;
}

export const TRACKS: readonly Track[] = [
  {
    id: 'adventurous',
    label: 'Adventurous',
    help: 'A full orchestra with a fanfare. Good for exploring.',
    url: new URL('./music/overworld_day_loop.ogg', import.meta.url).href,
  },
  {
    id: 'ambient',
    label: 'Ambient',
    help: 'Soft and slow, with no drums. Good for building and tending a factory.',
    url: new URL('./music/overworld_day_ambient_loop.ogg', import.meta.url).href,
  },
];

export interface MusicSettings {
  /** 0–100. */
  volume: number;
  muted: boolean;
  /** A `TRACKS` id. */
  track: string;
  /** Silent while the tab or window is not in focus (`setAway`). */
  muteInBackground: boolean;
}

const STORAGE_KEY = 'opencraft.music';
const DEFAULTS: MusicSettings = { volume: 20, muted: false, track: 'ambient', muteInBackground: true };
const FADE_IN = 2;
const CROSSFADE = 3;

interface Playing {
  id: string;
  source: AudioBufferSourceNode;
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
      if (TRACKS.some((t) => t.id === s.track)) settings.track = s.track as string;
    }
  } catch {
    // Storage unavailable or corrupt: use the defaults.
  }
  return settings;
}

const clamp = (v: number) => Math.round(Math.min(100, Math.max(0, v)));

export class Music {
  readonly settings = loadSettings();

  private ctx: AudioContext | null = null;
  private out: GainNode | null = null;
  private current: Playing | null = null;
  /** The tab or window is not in focus (not saved). */
  private away = false;
  /** The track that should be playing; a load that finishes after the choice changed is dropped. */
  private wanted = this.settings.track;
  private readonly buffers = new Map<string, Promise<AudioBuffer | null>>();

  /** `changed` is called after the volume, mute or track changes. */
  constructor(private readonly changed: () => void) {}

  get audible(): boolean {
    return !this.settings.muted && this.settings.volume > 0;
  }

  start(ctx: AudioContext): void {
    if (this.ctx) return;
    this.ctx = ctx;
    this.out = ctx.createGain();
    this.out.gain.value = this.level();
    this.out.connect(ctx.destination);
    this.sync();
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
    this.sync();
  }

  setTrack(id: string): void {
    if (!TRACKS.some((t) => t.id === id)) return;
    this.settings.track = id;
    this.update();
  }

  private level(): number {
    const hushed = this.away && this.settings.muteInBackground;
    return this.audible && !hushed ? (this.settings.volume / 100) ** 2 : 0;
  }

  private update(): void {
    try {
      localStorage.setItem(STORAGE_KEY, JSON.stringify(this.settings));
    } catch {
      // Not persisted; the settings still apply for this session.
    }
    this.sync();
    this.changed();
  }

  /** Applies the level and starts the chosen track if it isn't the one playing. */
  private sync(): void {
    const { ctx, out } = this;
    if (!ctx || !out) return;
    out.gain.setTargetAtTime(this.level(), ctx.currentTime, 0.05);
    this.wanted = this.settings.track;
    if (this.audible && this.current?.id !== this.wanted) void this.play(this.wanted);
  }

  private async play(id: string): Promise<void> {
    const buffer = await this.load(id);
    const { ctx, out } = this;
    if (!buffer || !ctx || !out || this.wanted !== id || this.current?.id === id) {
      if (buffer && this.current?.id !== id) this.buffers.delete(id); // chosen away from while it loaded
      return;
    }
    const now = ctx.currentTime;
    const fade = this.current ? CROSSFADE : FADE_IN;
    const source = ctx.createBufferSource();
    source.buffer = buffer;
    source.loop = true;
    const gain = ctx.createGain();
    gain.gain.setValueAtTime(0, now);
    gain.gain.linearRampToValueAtTime(1, now + fade);
    source.connect(gain).connect(out);
    source.start();
    const old = this.current;
    this.current = { id, source, gain };
    if (old) {
      old.gain.gain.cancelScheduledValues(now);
      old.gain.gain.setValueAtTime(old.gain.gain.value, now);
      old.gain.gain.linearRampToValueAtTime(0, now + fade);
      old.source.stop(now + fade);
      old.source.onended = () => {
        old.source.disconnect();
        old.gain.disconnect();
      };
      this.buffers.delete(old.id); // a decoded loop is tens of megabytes; the file stays in the HTTP cache
    }
  }

  private load(id: string): Promise<AudioBuffer | null> {
    let loading = this.buffers.get(id);
    if (!loading) {
      const track = TRACKS.find((t) => t.id === id)!;
      loading = fetch(track.url)
        .then((r) => r.arrayBuffer())
        .then((bytes) => this.ctx!.decodeAudioData(bytes))
        .catch((e) => {
          console.warn(`Music "${id}" could not be loaded`, e);
          this.buffers.delete(id);
          return null;
        });
      this.buffers.set(id, loading);
    }
    return loading;
  }
}
