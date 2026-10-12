// Number and colour helpers shared by the analytics hub's tabs (analytics.ts and analytics-*.ts): short numbers, time
// labels, a palette of distinct line colours and the `Palette` that hands them out. Nothing here knows what is charted.

/** A short number: 950, 1.2k, 34k, 2.5M. */
export function short(v: number): string {
  const n = Math.abs(v);
  if (n >= 1e6) return `${(v / 1e6).toFixed(1)}M`;
  if (n >= 1e4) return `${Math.round(v / 1000)}k`;
  if (n >= 1000) return `${(v / 1000).toFixed(1)}k`;
  return n < 10 ? String(Math.round(v * 10) / 10) : String(Math.round(v));
}

/** Power: "850 kW", "1.2 MW". */
export function kw(v: number): string {
  return Math.abs(v) >= 1000 ? `${(v / 1000).toFixed(1).replace(/\.0$/, '')} MW` : `${Math.round(v)} kW`;
}

/** "now", "45 s", "5 min", "1.5 h": how long before now. */
export function ago(secs: number): string {
  if (secs < 1) return 'now';
  if (secs < 90) return `${Math.round(secs)} s`;
  if (secs < 5400) return `${Math.round(secs / 60)} min`;
  return `${(secs / 3600).toFixed(1).replace(/\.0$/, '')} h`;
}

/** `0xRRGGBB` as a CSS colour. */
export function hex(rgb: number): string {
  return `#${rgb.toString(16).padStart(6, '0')}`;
}

/** Colours that stay apart on the dark panel, in the order they are handed out (the first few are the most distinct). */
export const PALETTE = [
  '#5ab4ff', '#ff7a59', '#7bd88f', '#ffd166', '#c792ea', '#2ee6d6',
  '#ff8fd1', '#b8e04a', '#9aa5ff', '#ffa94d', '#e5e7eb', '#c98f5a',
];

/** Gives each key (an item, a machine kind) the first palette colour nobody holds, and takes it back on release. */
export class Palette {
  private readonly held = new Map<number, number>();

  get size(): number {
    return this.held.size;
  }

  get full(): boolean {
    return this.held.size >= PALETTE.length;
  }

  /** The key's colour, taking a free one first if it has none (null when every colour is held). */
  take(key: number): string | null {
    let slot = this.held.get(key);
    if (slot === undefined) {
      const used = new Set(this.held.values());
      slot = PALETTE.findIndex((_, i) => !used.has(i));
      if (slot < 0) return null;
      this.held.set(key, slot);
    }
    return PALETTE[slot];
  }

  colour(key: number): string | null {
    const slot = this.held.get(key);
    return slot === undefined ? null : PALETTE[slot];
  }

  release(key: number): void {
    this.held.delete(key);
  }

  clear(): void {
    this.held.clear();
  }
}

/** The graph rows of one engine call (`analytics_*_graphs`: `[points, rows, then points values per row]`). */
export interface Graph {
  points: number;
  rows: number;
  row(i: number): Float32Array;
}

export function readGraph(data: Float32Array): Graph {
  const points = data[0], rows = data[1];
  return { points, rows, row: (i) => data.subarray(2 + i * points, 2 + (i + 1) * points) };
}

/** The newest value of a row that is not blank. */
export function lastOf(values: ArrayLike<number>): number {
  for (let i = values.length - 1; i >= 0; i--) if (!Number.isNaN(values[i])) return values[i];
  return 0;
}
