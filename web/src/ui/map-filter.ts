// Which points of interest the world map and the corner minimap show. A filter is a set of hidden categories, kept
// in localStorage (a per-viewer convenience, not part of any world): the machines, towns, terraforming sites and
// satellite cover as a group each, one category per ore (it covers deposits, ore seen at the surface and survey
// guesses of that ore, told apart by the mark's colour) and one per pin kind. The rules are pure functions (tested
// in web/test/map-filter.test.mjs: no DOM and no enums in here), and so is reading the hover lines the engine
// answers with; the chips that change the filter are in map-filter-panel.ts. To filter a new kind of mark: give it
// a category in `markCategory`.

import { hex, MARK_CHANNEL, MARK_COVER, MARK_GUESS, MARK_MACHINE, MARK_RANGE, MARK_SITE, MARK_TOWN } from './pins.ts';

const STORAGE_KEY = 'opencraft.map.filter';

/** The category of a mark, or null for one that always shows (a long-distance pole's reach, an aid while placing). */
export function markCategory(shape: number, colour: number): string | null {
  if (shape >= MARK_TOWN) return 'towns';
  if (shape >= MARK_CHANNEL) return 'machines'; // pads, launchers and launch sites
  if (shape === MARK_RANGE) return null;
  if (shape >= MARK_COVER) return 'cover';
  if (shape >= MARK_GUESS) return oreCategory(colour);
  if (shape === MARK_SITE) return 'sites';
  if (shape === MARK_MACHINE) return 'machines';
  return oreCategory(colour); // a deposit or ore seen at the surface
}

export function oreCategory(colour: number): string {
  return `ore:${colour}`;
}

export function pinCategory(kindId: string): string {
  return `pin:${kindId}`;
}

/** The hidden categories saved as `text` (nothing hidden for text that is missing or wrong). */
export function parseHidden(text: string | null): Set<string> {
  try {
    const list: unknown = JSON.parse(text ?? '[]');
    return new Set(Array.isArray(list) ? list.filter((c): c is string => typeof c === 'string') : []);
  } catch {
    return new Set();
  }
}

/** One line of the hover box: a title, a sentence, and the mark's colour as CSS (none for a reach, which has no colour). */
export interface Info {
  colour: string | null;
  title: string;
  detail: string;
}

/** Parses `world_map_describe`'s lines (`shape TAB colour TAB title TAB detail`, nearest first), keeping the marks the
 * filter shows. Footprints and a pole's ring carry a reach in blocks where the colour would be. */
export function parseInfo(text: string, filter: { showsMark(shape: number, colour: number): boolean }): Info[] {
  const out: Info[] = [];
  for (const line of text.split('\n')) {
    const [shape, colour, title, detail] = line.split('\t');
    if (title === undefined || detail === undefined || !filter.showsMark(Number(shape), Number(colour))) continue;
    const reach = Number(shape) >= MARK_COVER && Number(shape) <= MARK_RANGE;
    out.push({ colour: reach ? null : hex(Number(colour)), title, detail });
  }
  return out;
}

/** The records of `marks` (`fields` numbers each: x, z, colour, shape) the filter lets through. */
export function visibleMarks(marks: ArrayLike<number>, fields: number, hidden: ReadonlySet<string>): number[] {
  const out: number[] = [];
  for (let i = 0; i + fields <= marks.length; i += fields) {
    const category = markCategory(marks[i + 3], marks[i + 2]);
    if (category === null || !hidden.has(category)) for (let k = 0; k < fields; k++) out.push(marks[i + k]);
  }
  return out;
}

export class MapFilter {
  private readonly hidden: Set<string>;
  private readonly listeners: (() => void)[] = [];

  constructor() {
    this.hidden = parseHidden(load());
  }

  /** Calls `fn` after every change. */
  subscribe(fn: () => void): void {
    this.listeners.push(fn);
  }

  shows(category: string | null): boolean {
    return category === null || !this.hidden.has(category);
  }

  /** Whether a mark with this colour and shape shows. */
  showsMark(shape: number, colour: number): boolean {
    return this.shows(markCategory(shape, colour));
  }

  showsPin(kindId: string): boolean {
    return this.shows(pinCategory(kindId));
  }

  /** The engine marks (`fields` numbers each) that show. */
  marks(marks: ArrayLike<number>, fields: number): number[] {
    return visibleMarks(marks, fields, this.hidden);
  }

  toggle(category: string): void {
    this.set([category], !this.shows(category));
  }

  /** Shows or hides several categories at once. */
  set(categories: readonly string[], show: boolean): void {
    for (const c of categories) {
      if (show) this.hidden.delete(c);
      else this.hidden.add(c);
    }
    save([...this.hidden]);
    this.listeners.forEach((fn) => fn());
  }
}

// Storage can be unavailable (private windows, blocked site data); the filter then lasts until the page closes.
function load(): string | null {
  try {
    return localStorage.getItem(STORAGE_KEY);
  } catch {
    return null;
  }
}

function save(hidden: string[]): void {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(hidden));
  } catch {
    // Not remembered; see load.
  }
}
