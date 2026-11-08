// Quality settings: how sharp the picture is, how far the world reaches, antialiasing, and whether the game may lower them itself
// (auto-quality.ts). `QualityStore` keeps them in localStorage and tells subscribers (main.ts applies them, ui/quality.ts shows
// them). Presentation only: nothing here reaches the core or a save. `?rd=` is a test setting and wins for the page, and then
// nothing is stored. To add a setting: a field in `QualitySettings` and `DEFAULTS`, a line in `load`, a row in ui/quality.ts.

/** Percent of the screen's pixels drawn, best first: `AutoQuality` levels index this. */
export const SCALES = [100, 75, 50] as const;
/** Render distances offered, in chunks. */
export const RADII = [4, 6, 8, 12, 16] as const;
export const DEFAULT_RADIUS = 8;

export interface QualitySettings {
  scale: number;
  radius: number;
  /** Applies at the next page load: a WebGL context can't change it. */
  antialias: boolean;
  auto: boolean;
}

export const DEFAULTS: QualitySettings = { scale: 100, radius: DEFAULT_RADIUS, antialias: true, auto: true };

const STORAGE_KEY = 'opencraft.quality';

function load(): QualitySettings {
  const s = { ...DEFAULTS };
  try {
    const rec = JSON.parse(localStorage.getItem(STORAGE_KEY) ?? 'null') as Record<string, unknown> | null;
    if (typeof rec !== 'object' || rec === null) return s;
    if ((SCALES as readonly unknown[]).includes(rec.scale)) s.scale = rec.scale as number;
    if ((RADII as readonly unknown[]).includes(rec.radius)) s.radius = rec.radius as number;
    if (typeof rec.antialias === 'boolean') s.antialias = rec.antialias;
    if (typeof rec.auto === 'boolean') s.auto = rec.auto;
  } catch {
    // Storage unavailable or corrupt: the defaults.
  }
  return s;
}

export class QualityStore {
  settings: QualitySettings;
  /** The render distance came from the address, so this page stores nothing. */
  private readonly temporary: boolean;
  private readonly listeners: (() => void)[] = [];

  /** `radius`: the page's `?rd=`, or null. */
  constructor(radius: number | null = null) {
    this.temporary = radius !== null;
    this.settings = radius === null ? load() : { ...load(), radius };
  }

  /** Calls `fn` after every change, and now unless `now` is false. */
  subscribe(fn: () => void, now = true): void {
    this.listeners.push(fn);
    if (now) fn();
  }

  set<K extends keyof QualitySettings>(key: K, value: QualitySettings[K]): void {
    this.settings = { ...this.settings, [key]: value };
    this.changed();
  }

  /** The player chose: the game stops changing things for them. */
  choose<K extends keyof QualitySettings>(key: K, value: QualitySettings[K]): void {
    this.settings = { ...this.settings, [key]: value, auto: false };
    this.changed();
  }

  reset(): void {
    this.settings = { ...DEFAULTS };
    this.changed();
  }

  private changed(): void {
    try {
      if (!this.temporary) localStorage.setItem(STORAGE_KEY, JSON.stringify(this.settings));
    } catch {
      // Not persisted; the settings still apply for this session.
    }
    for (const fn of this.listeners) fn();
  }
}
