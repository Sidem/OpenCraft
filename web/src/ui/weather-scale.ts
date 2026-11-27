// The weather's numbers as the UI shows them (map-weather.ts, weather-chip.ts): the map's weather layers, how each
// colours a sample, the grid step for a zoom, the samples parsed from the engine's `weather_map` (api/weather_map.rs),
// and wind as a compass direction. Pure, so `web/test/weather-scale.test.mjs` covers it. To add a layer: a row in
// `LAYERS` and its case in `sampleFill`.

/** One measured point of `game.weather_map`, in the engine's field order. */
export interface Sample {
  x: number;
  z: number;
  temp: number; // °C
  precip: number; // 0..1
  cover: number; // 0..1
  snow: boolean;
  pressure: number; // hPa at sea level
  east: number; // wind toward the east, m/s
  south: number; // wind toward the south, m/s
  humidity: number; // %
}

/** The map's weather layers: what each shows (tooltip) and its icon (seasons.ts `icon`). */
export const LAYERS = [
  { key: 'off', name: 'No weather on the map', icon: null },
  { key: 'temp', name: 'Temperature', icon: 'thermo' },
  { key: 'rain', name: 'Rain, snow and clouds', icon: 'rain' },
  { key: 'pressure', name: 'Air pressure (blue low, orange high)', icon: 'gauge' },
  { key: 'wind', name: 'Wind', icon: 'wind' },
] as const;
export type Layer = (typeof LAYERS)[number]['key'];

/** Pressure from deep low to strong high, for the pressure layer's colours (hPa). */
const LOW_HPA = 990, HIGH_HPA = 1035;
/** Wind speed that gets the strongest colour and the longest arrow (m/s). */
export const STRONG_WIND = 15;

export function parseSamples(a: ArrayLike<number>, n: number): Sample[] {
  const out: Sample[] = [];
  for (let i = 0; i + n <= a.length; i += n) {
    out.push({
      x: a[i], z: a[i + 1], temp: a[i + 2], precip: a[i + 3], cover: a[i + 4], snow: a[i + 5] > 0.5,
      pressure: a[i + 6], east: a[i + 7], south: a[i + 8], humidity: a[i + 9],
    });
  }
  return out;
}

/** The sample spacing in blocks for `bpp` blocks per screen pixel: a power of two at least 8, about `px` pixels apart. */
export function gridStep(bpp: number, px: number): number {
  let step = 8;
  while (step < bpp * px) step *= 2;
  return step;
}

/** A temperature's colour: blue at -10 °C and below, through green, to red at 32 °C and above. */
export function tempColour(c: number): string {
  const t = Math.min(1, Math.max(0, (c + 10) / 42));
  return `hsl(${Math.round(210 - 210 * t)} 75% 68%)`;
}

/** How a layer fills a sample's square: a CSS colour and its opacity, or null to leave it clear. */
export function sampleFill(layer: Layer, s: Sample): { colour: string; alpha: number } | null {
  switch (layer) {
    case 'temp':
      return { colour: tempColour(s.temp), alpha: 0.5 };
    case 'rain':
      if (s.precip > 0.02) return { colour: s.snow ? '#f4f8ff' : '#3f7fe0', alpha: 0.35 + 0.45 * Math.min(1, s.precip * 2) };
      return s.cover > 0.05 ? { colour: '#c9d2e0', alpha: 0.5 * s.cover } : null;
    case 'pressure': {
      const t = Math.min(1, Math.max(0, (s.pressure - LOW_HPA) / (HIGH_HPA - LOW_HPA)));
      return { colour: `hsl(${Math.round(250 - 220 * t)} 70% 62%)`, alpha: 0.5 };
    }
    case 'wind':
      return { colour: windColour(speed(s)), alpha: 0.3 };
    default:
      return null;
  }
}

export function speed(s: { east: number; south: number }): number {
  return Math.hypot(s.east, s.south);
}

/** Calm pale green to strong red-violet as the wind rises to `STRONG_WIND`. */
export function windColour(ms: number): string {
  const t = Math.min(1, ms / STRONG_WIND);
  return `hsl(${Math.round(500 - 200 * t) % 360} 70% 64%)`;
}

const POINTS = ['north', 'north-east', 'east', 'south-east', 'south', 'south-west', 'west', 'north-west'];

/** Where wind blowing `east`, `south` (m/s) comes from, as a compass word ("north-west"), or "" when calm. */
export function windFrom(east: number, south: number): string {
  if (Math.hypot(east, south) < 0.3) return '';
  // The source is opposite the motion; north is -z, bearings run clockwise from north.
  return compassPoint((Math.atan2(-east, south) * 180) / Math.PI);
}

/** The nearest of the eight compass points to a bearing in degrees (0 north, 90 east). */
export function compassPoint(deg: number): string {
  return POINTS[Math.round((((deg % 360) + 360) % 360) / 45) % 8];
}