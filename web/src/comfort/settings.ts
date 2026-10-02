// Comfort settings for players prone to motion sickness: field of view, mouse sensitivity, the
// movement vignette, third-person view and the crosshair. `ComfortStore` holds them, keeps them in localStorage and tells
// subscribers (the renderer, the crosshair, the vignette, the menu section) when one changes.
// To add a setting: a field in `ComfortSettings` and `DEFAULTS` (a number also gets a row in `RANGES`),
// then use it where it matters and add a control in ui/comfort.ts.

export type CrosshairStyle = 'cross' | 'dot' | 'both';
export const CROSSHAIR_STYLES: readonly CrosshairStyle[] = ['cross', 'dot', 'both'];
/** Vignette strength choices: 0 is off. */
export const VIGNETTE_LEVELS = [0, 1, 2, 3] as const;

export interface ComfortSettings {
  /** Vertical field of view in degrees (Minecraft's slider uses the same measure). */
  fov: number;
  /** Mouse look speed, percent of the base speed. */
  sensitivity: number;
  /** How far the edges of the view darken while turning or moving fast: 0 off, 1 low, 2 medium, 3 high. */
  vignette: number;
  crosshairStyle: CrosshairStyle;
  /** Camera behind the player instead of in the head (V toggles). */
  thirdPerson: boolean;
  /** Blocks behind the head in third person. */
  thirdDistance: number;
  /** CSS pixels. */
  crosshairSize: number;
  crosshairThickness: number;
  /** Percent. */
  crosshairOpacity: number;
}

/** [min, max, step] of every numeric setting; the menu sliders and the loader share them. */
export const RANGES = {
  fov: [50, 100, 1],
  sensitivity: [25, 200, 5],
  vignette: [0, 3, 1],
  thirdDistance: [2, 8, 0.5],
  crosshairSize: [12, 60, 1],
  crosshairThickness: [1, 6, 1],
  crosshairOpacity: [30, 100, 5],
} as const;

export const DEFAULTS: ComfortSettings = {
  fov: 72,
  sensitivity: 100,
  vignette: 1,
  thirdPerson: false,
  thirdDistance: 4,
  crosshairStyle: 'cross',
  crosshairSize: 28,
  crosshairThickness: 3,
  crosshairOpacity: 100,
};

const STORAGE_KEY = 'opencraft.comfort';

const clamp = (key: keyof typeof RANGES, v: number): number => {
  const [min, max, step] = RANGES[key];
  return Math.min(max, Math.max(min, Math.round(v / step) * step));
};

function load(): ComfortSettings {
  const s = { ...DEFAULTS };
  try {
    const saved: unknown = JSON.parse(localStorage.getItem(STORAGE_KEY) ?? 'null');
    if (typeof saved !== 'object' || saved === null) return s;
    const rec = saved as Record<string, unknown>;
    for (const key of Object.keys(RANGES) as (keyof typeof RANGES)[]) {
      const v = rec[key];
      if (typeof v === 'number' && Number.isFinite(v)) s[key] = clamp(key, v);
    }
    if (typeof rec.thirdPerson === 'boolean') s.thirdPerson = rec.thirdPerson;
    if (CROSSHAIR_STYLES.includes(rec.crosshairStyle as CrosshairStyle)) s.crosshairStyle = rec.crosshairStyle as CrosshairStyle;
  } catch {
    // Storage unavailable or corrupt: the defaults.
  }
  return s;
}

export class ComfortStore {
  settings: ComfortSettings = load();
  private readonly listeners: (() => void)[] = [];

  /** Calls `fn` now and after every change. */
  subscribe(fn: () => void): void {
    this.listeners.push(fn);
    fn();
  }

  set<K extends keyof ComfortSettings>(key: K, value: ComfortSettings[K]): void {
    this.settings = { ...this.settings, [key]: typeof value === 'number' && key in RANGES ? clamp(key as keyof typeof RANGES, value) : value };
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
