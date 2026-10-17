// Sky settings: how the night sky, the weather and the dusk are drawn. `SkyStore` keeps them in localStorage and tells
// subscribers (the renderer, the menu section ui/sky.ts) when one changes. Presentation only: nothing here reaches the engine
// or a save. Weather Off and red dusk Off together draw exactly the sky of before step S2.
// To add a setting: a field in `SkySettings` and `DEFAULTS`, a line in `load`, then use it in sky.ts / stars.ts and add a
// control in ui/sky.ts.

/** 0 the old sky (flat hash stars, a full moon), 1 real stars and moon phases, 2 the same plus the Milky Way. */
export type SkyQuality = 0 | 1 | 2;
export const QUALITY_NAMES = ['Classic', 'Stars', 'Stars and Milky Way'] as const;

export interface SkySettings {
  quality: SkyQuality;
  /** Draw the figures of the constellations (faint lines between their stars). */
  lines: boolean;
  /** Let the stars shimmer. */
  twinkle: boolean;
  /** Weather in the sky (sky/weather.ts): 0 off, 1 low (clouds and gloom), 2 full (clouds lit from the side as well). */
  weather: WeatherQuality;
  /** The red glow of dusk and dawn. */
  dusk: boolean;
  /** Rain and snow falling, wet ground and snow cover (render/precip.ts; needs Weather on): 0 off, 1 low, 2 full. */
  precip: WeatherQuality;
}

/** 0 off, 1 low, 2 full. */
export type WeatherQuality = 0 | 1 | 2;
export const WEATHER_NAMES = ['Off', 'Low', 'Full'] as const;

export const DEFAULTS: SkySettings = { quality: 2, lines: false, twinkle: true, weather: 2, dusk: true, precip: 2 };

const STORAGE_KEY = 'opencraft.sky';

function load(): SkySettings {
  const s = { ...DEFAULTS };
  try {
    const rec = JSON.parse(localStorage.getItem(STORAGE_KEY) ?? 'null') as Record<string, unknown> | null;
    if (typeof rec !== 'object' || rec === null) return s;
    if (rec.quality === 0 || rec.quality === 1 || rec.quality === 2) s.quality = rec.quality;
    if (typeof rec.lines === 'boolean') s.lines = rec.lines;
    if (typeof rec.twinkle === 'boolean') s.twinkle = rec.twinkle;
    if (rec.weather === 0 || rec.weather === 1 || rec.weather === 2) s.weather = rec.weather;
    if (typeof rec.dusk === 'boolean') s.dusk = rec.dusk;
    if (rec.precip === 0 || rec.precip === 1 || rec.precip === 2) s.precip = rec.precip;
  } catch {
    // Storage unavailable or corrupt: the defaults.
  }
  return s;
}

export class SkyStore {
  settings: SkySettings = load();
  private readonly listeners: (() => void)[] = [];

  /** Calls `fn` now and after every change. */
  subscribe(fn: () => void): void {
    this.listeners.push(fn);
    fn();
  }

  set<K extends keyof SkySettings>(key: K, value: SkySettings[K]): void {
    this.settings = { ...this.settings, [key]: value };
    this.changed();
  }

  reset(): void {
    this.settings = { ...DEFAULTS };
    this.changed();
  }

  private changed(): void {
    try {
      localStorage.setItem(STORAGE_KEY, JSON.stringify(this.settings));
    } catch {
      // Not persisted; the settings still apply for this session.
    }
    for (const fn of this.listeners) fn();
  }
}
