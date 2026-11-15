// Pure helpers for folding a building's Mks (Mk1, Mk2, ...) into one build-menu tile (craft-family.ts). The engine's
// `recipe_order` gives every Mk of a building one rank and the Mk as the low three bits (recipes/menu_order.rs), so
// tiles of one category with the same rank are the Mks of one building.

/** What the helpers read off a build-menu tile. */
export interface Ranked {
  group: number;
  order: number;
  state: string;
}

/** The Mks of one building share `order >> 3`: `RANK` is the engine's multiplier. */
const RANK = 8;

/** Which building a tile is a Mk of. */
export function familyKey(t: Ranked): string {
  return `${t.group}:${Math.floor(t.order / RANK)}`;
}

/** The tile's Mk, from 1. */
export function mkOf(t: Ranked): number {
  return (t.order % RANK) + 1;
}

/** Folds `tiles` (in menu order) into one list per building, each sitting where its first tile was and listing its Mks
 * lowest first. Guide tiles never fold. A building with one tile on show is a list of one. */
export function foldFamilies<T extends Ranked>(tiles: T[]): T[][] {
  const lists: T[][] = [];
  const byKey = new Map<string, T[]>();
  for (const t of tiles) {
    const list = t.state === 'guide' ? undefined : byKey.get(familyKey(t));
    if (list) {
      list.push(t);
      continue;
    }
    const fresh = [t];
    lists.push(fresh);
    if (t.state !== 'guide') byKey.set(familyKey(t), fresh);
  }
  for (const list of lists) if (list.length > 1) list.sort((a, b) => a.order - b.order);
  return lists;
}

/** A building's name without its Mk: "Smelter Mk2" is "Smelter". */
export function baseName(name: string): string {
  return name.replace(/\s*\bMk\s*\d+\b/i, '').trim() || name;
}

/** "Mk1–4" for the Mks on show (just "Mk3" for one). */
export function mkRange(mks: number[]): string {
  const lo = Math.min(...mks);
  const hi = Math.max(...mks);
  return lo === hi ? `Mk${lo}` : `Mk${lo}–${hi}`;
}

/** The state a group of tiles shows as: craftable if any Mk is, else short of items, else locked. */
export function bestState(states: string[]): 'ready' | 'short' | 'locked' {
  if (states.includes('ready')) return 'ready';
  return states.includes('short') ? 'short' : 'locked';
}
