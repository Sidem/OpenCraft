// Pure seismic cross-section and 3D volume types, coordinate geometry, material classification and palettes (F-SEIS).
// Shared by `seismic-render.ts`, `seismic-volume.ts` and `seismic.ts`; unit-tested under Node (`web/test/seismic.test.mjs`).

export type DisplayMode = 'tomo' | 'wiggle' | 'volume';

/** Engine signal classes (`crates/engine/src/seismic/mod.rs`). */
export const CLASS_SKY = 0;
export const CLASS_ROCK = 1;
export const CLASS_SOIL = 2;
export const CLASS_CAVE = 3;
export const CLASS_TUNNEL = 4;
export const CLASS_WATER = 5;
export const CLASS_STRUCTURE = 6;
export const CLASS_BEDROCK = 7;
export const CLASS_ORE_UNKNOWN = 8;
export const CLASS_ORE_METAL = 9;
export const CLASS_ORE_CARBON = 10;
export const CLASS_ORE_MINERAL = 11;

export interface SliceData {
  anchorX: number;
  anchorY: number;
  anchorZ: number;
  dir: number;
  perpOffset: number;
  tier: number;
  powerShare: number;
  kw: number;
  width: number;
  height: number;
  surfaceAtThumper: number;
  status: number;
  seaLevel: number;
  playerS: number;
  playerY: number;
  playerPerp: number;
  snr: number;
  surface: Int32Array;
  cells: Uint8Array;
  anomalies: Int32Array;
}

export interface VolumeData {
  size: number;
  topY: number;
  anchorX: number;
  anchorY: number;
  anchorZ: number;
  dir: number;
  tier: number;
  seaLevel: number;
  playerDx: number;
  playerY: number;
  playerDz: number;
  surface: Int32Array;
  voxels: Uint8Array;
}

export interface VolumeFilters {
  ores: boolean;
  tunnels: boolean;
  caves: boolean;
  water: boolean;
  surface: boolean;
  slicePlane: boolean;
}

export interface VolumePick {
  lx: number;
  y: number;
  lz: number;
  wx: number;
  wy: number;
  wz: number;
  imp: number;
  cls: number;
  detail: number;
}

export interface GridPoint {
  s: number;
  y: number;
}

export interface PlotRect {
  x: number;
  y: number;
  w: number;
  h: number;
}

const COMPASS_ENDS: [string, string][] = [
  ['S', 'N'],
  ['W', 'E'],
  ['N', 'S'],
  ['E', 'W'],
];

const DIR_AXES: [number, number][] = [
  [0, -1],
  [1, 0],
  [0, 1],
  [-1, 0],
];

export function compassEnds(dir: number): [string, string] {
  return COMPASS_ENDS[dir & 3] ?? ['W', 'E'];
}

export function dirAxis(dir: number): [number, number] {
  return DIR_AXES[dir & 3] ?? [1, 0];
}

export function perpLabel(dir: number, perpOffset: number): string {
  if (perpOffset === 0) return 'Center (0m)';
  const [neg, pos] = compassEnds((dir + 1) & 3);
  return `${perpOffset > 0 ? '+' : '−'}${Math.abs(perpOffset)}m ${perpOffset > 0 ? pos : neg}`;
}

export function worldCoord(
  d: Pick<SliceData, 'anchorX' | 'anchorZ' | 'dir' | 'perpOffset' | 'width'>,
  s: number,
  y: number,
): [number, number, number] {
  const off = s - (d.width >> 1);
  const [ax, az] = dirAxis(d.dir);
  const [px, pz] = dirAxis(d.dir + 1);
  return [d.anchorX + ax * off + px * d.perpOffset, y, d.anchorZ + az * off + pz * d.perpOffset];
}

export function offsetLabel(d: Pick<SliceData, 'dir' | 'width'>, s: number): string {
  const off = s - (d.width >> 1);
  if (off === 0) return '0m';
  const [neg, pos] = compassEnds(d.dir);
  return `${off > 0 ? '+' : '−'}${Math.abs(off)}m ${off > 0 ? pos : neg}`;
}

export function plotRect(w: number, h: number): PlotRect {
  return { x: 46, y: 28, w: Math.max(64, w - 62), h: Math.max(64, h - 56) };
}

export function cellToCanvas(plot: PlotRect, d: Pick<SliceData, 'width' | 'height'>, s: number, y: number): [number, number] {
  return [plot.x + ((s + 0.5) / d.width) * plot.w, plot.y + (1 - (y + 0.5) / d.height) * plot.h];
}

export function canvasToCell(plot: PlotRect, d: Pick<SliceData, 'width' | 'height'>, px: number, py: number): GridPoint | null {
  if (px < plot.x || px > plot.x + plot.w || py < plot.y || py > plot.y + plot.h) return null;
  const s = Math.min(d.width - 1, Math.max(0, Math.floor(((px - plot.x) / plot.w) * d.width)));
  const y = Math.min(d.height - 1, Math.max(0, Math.floor((1 - (py - plot.y) / plot.h) * d.height)));
  return { s, y };
}

/** Human-readable material / anomaly interpretation, respecting Mk tier masking. */
export function describeCell(
  cls: number,
  detail: number,
  imp: number,
  oreName: (detail: number) => string,
): { label: string; color: string } {
  switch (cls) {
    case CLASS_SKY: return { label: 'Open air', color: '#94a3b8' };
    case CLASS_BEDROCK: return { label: 'Bedrock basement', color: '#f87171' };
    case CLASS_WATER: return { label: 'Water / aquifer', color: '#38bdf8' };
    case CLASS_TUNNEL: return { label: 'Excavated tunnel', color: '#2dd4bf' };
    case CLASS_CAVE: return { label: 'Natural cavity / void', color: '#a78bfa' };
    case CLASS_STRUCTURE: return { label: 'Built structure / machine', color: '#fbbf24' };
    case CLASS_ORE_UNKNOWN: return { label: imp >= 205 ? 'Very dense anomaly' : 'Dense ore band', color: '#fb923c' };
    case CLASS_ORE_METAL: {
      const name = detail > 0 ? oreName(detail) : '';
      return { label: name ? `${name} (metallic)` : 'Metallic ore reflector', color: '#fbbf24' };
    }
    case CLASS_ORE_CARBON: {
      const name = detail > 0 ? oreName(detail) : '';
      return { label: name ? `${name} (hydrocarbon)` : 'Hydrocarbon / carbon seam', color: '#e879f9' };
    }
    case CLASS_ORE_MINERAL: {
      const name = detail > 0 ? oreName(detail) : '';
      return { label: name ? `${name} (mineral)` : 'Mineral / crystalline band', color: '#67e8f9' };
    }
    case CLASS_SOIL: return { label: 'Soft overburden / soil', color: '#a8a29e' };
    default: return { label: imp > 155 ? 'Dense host rock' : 'Host rock strata', color: '#94a3b8' };
  }
}

const lerp = (a: number, b: number, t: number) => a + (b - a) * t;

export function colorForCell(
  mode: DisplayMode,
  imp: number,
  cls: number,
  detail: number,
  oreColors: Map<number, [number, number, number]>,
): [number, number, number] {
  const f = imp / 255;
  if (cls === CLASS_SKY) return [11, 15, 24];
  if (cls === CLASS_BEDROCK) return [52, 20, 26];

  if (mode === 'volume') {
    if (cls === CLASS_TUNNEL) return [24, 212, 188];
    if (cls === CLASS_CAVE) return [116, 92, 186];
    if (cls === CLASS_WATER) return [24, 162, 240];
    if (cls === CLASS_STRUCTURE) return [248, 192, 64];
    if (cls >= CLASS_ORE_UNKNOWN) {
      const oc = detail > 0 ? oreColors.get(detail) : undefined;
      if (oc) return [Math.round(lerp(oc[0], 255, 0.25)), Math.round(lerp(oc[1], 235, 0.2)), Math.round(lerp(oc[2], 180, 0.15))];
      if (cls === CLASS_ORE_CARBON) return [Math.round(lerp(168, 245, f)), Math.round(lerp(78, 145, f)), Math.round(lerp(208, 255, f))];
      if (cls === CLASS_ORE_MINERAL) return [Math.round(lerp(82, 165, f)), Math.round(lerp(195, 246, f)), Math.round(lerp(222, 255, f))];
      return [Math.round(lerp(205, 255, f)), Math.round(lerp(105, 205, f)), Math.round(lerp(34, 72, f))];
    }
    const base = cls === CLASS_SOIL ? 28 : Math.round(18 + f * 24);
    return [base, base + 5, base + 14];
  }

  let rgb: [number, number, number];
  if (cls === CLASS_CAVE) rgb = [8, 10, 20];
  else if (cls === CLASS_TUNNEL) rgb = [22, 186, 168];
  else if (cls === CLASS_WATER) rgb = [14, Math.round(115 + f * 75), Math.round(196 + f * 50)];
  else if (cls === CLASS_STRUCTURE) rgb = [235, 185, 72];
  else if (cls >= CLASS_ORE_UNKNOWN) {
    const oc = detail > 0 ? oreColors.get(detail) : undefined;
    if (oc) {
      const lift = 0.2 + f * 0.35;
      rgb = [
        Math.min(255, Math.round(oc[0] * (0.72 + f * 0.45) + 45 * lift)),
        Math.min(255, Math.round(oc[1] * (0.72 + f * 0.45) + 35 * lift)),
        Math.min(255, Math.round(oc[2] * (0.72 + f * 0.45) + 18 * lift)),
      ];
    } else if (cls === CLASS_ORE_CARBON) {
      rgb = [Math.round(lerp(150, 240, f)), Math.round(lerp(66, 134, f)), Math.round(lerp(195, 250, f))];
    } else if (cls === CLASS_ORE_MINERAL) {
      rgb = [Math.round(lerp(64, 145, f)), Math.round(lerp(184, 242, f)), Math.round(lerp(215, 252, f))];
    } else {
      rgb = [Math.round(lerp(195, 255, f)), Math.round(lerp(82, 214, f)), Math.round(lerp(22, 74, f))];
    }
  } else if (cls === CLASS_SOIL) {
    rgb = [Math.round(lerp(42, 76, f)), Math.round(lerp(44, 72, f)), Math.round(lerp(52, 68, f))];
  } else {
    rgb = [Math.round(lerp(18, 66, f)), Math.round(lerp(28, 86, f)), Math.round(lerp(48, 122, f))];
  }
  return mode === 'wiggle'
    ? [Math.round(rgb[0] * 0.38 + 6), Math.round(rgb[1] * 0.38 + 9), Math.round(rgb[2] * 0.38 + 15)]
    : rgb;
}

export function legendEntries(tier: number): [string, string][] {
  return tier === 0
    ? [['#fb923c', 'Dense band'], ['#38bdf8', 'Water'], ['#745cba', 'Void / cave'], ['#475569', 'Rock'], ['#7f1d1d', 'Bedrock']]
    : [['#fbbf24', 'Metallic'], ['#e879f9', 'Hydrocarbon'], ['#67e8f9', 'Mineral'], ['#2dd4bf', 'Tunnel'], ['#38bdf8', 'Water'], ['#745cba', 'Cave']];
}
